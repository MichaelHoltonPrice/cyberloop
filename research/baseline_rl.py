#!/usr/bin/env python3
"""Baseline RL training pipeline for the Decker gauntlet.

Runs three training phases inside a flywheel workspace, with 4000-episode
evaluations after each:
  1. Combat    — 15M steps, card-play decisions only
  2. Curation  — 5M steps, reward/deck decisions (combat heads frozen)
  3. Combined  — 5M steps, all phases jointly

Each phase is a flywheel block execution: one Docker container, with
checkpoint artifacts chained between phases via input/output slots.

Requires:
  - flywheel installed (pip install -e ../flywheel)
  - Docker images built: cyberloop-train:latest, cyberloop-eval-rl:latest
  - CWD = cyberloop project root (where flywheel.yaml lives)

Usage:
    cd cyberloop
    python research/baseline_rl.py --subclass dueling --seed 42
"""

import argparse
import json
import subprocess
import sys
import time
import uuid
from pathlib import Path

import yaml

TEMPLATE = "train_eval"
PROJECT_ROOT = Path(__file__).resolve().parent.parent
FOUNDRY_DIR = PROJECT_ROOT / "foundry"


def run_flywheel(args, label):
    """Run a flywheel CLI command from the project root."""
    cmd = ["flywheel"] + args
    print(f"\n{'=' * 60}")
    print(f"[{label}] {' '.join(cmd)}")
    print("=" * 60)
    t0 = time.time()
    result = subprocess.run(cmd, cwd=PROJECT_ROOT)
    elapsed = time.time() - t0
    if result.returncode != 0:
        print(f"\n[{label}] FAILED (exit {result.returncode})")
        sys.exit(result.returncode)
    print(f"[{label}] done in {elapsed:.0f}s")
    return elapsed


def workspace_path(name):
    """Return the filesystem path to a workspace."""
    return FOUNDRY_DIR / "workspaces" / name


def load_workspace_yaml(name):
    """Load and return parsed workspace.yaml."""
    ws_yaml = workspace_path(name) / "workspace.yaml"
    with open(ws_yaml) as f:
        return yaml.safe_load(f)


def latest_checkpoint_artifact(ws_name):
    """Find the most recently produced checkpoint artifact ID in the workspace."""
    ws = load_workspace_yaml(ws_name)
    checkpoint_artifacts = [
        (aid, entry) for aid, entry in ws.get("artifacts", {}).items()
        if entry["name"] == "checkpoint" and entry["kind"] == "copy"
    ]
    if not checkpoint_artifacts:
        print("ERROR: no checkpoint artifacts found in workspace", file=sys.stderr)
        sys.exit(1)
    # Sort by creation time, take the latest.
    checkpoint_artifacts.sort(key=lambda x: x[1]["created_at"])
    return checkpoint_artifacts[-1][0]


def find_model_pt(ws_name, artifact_id):
    """Find the final_model.pt file within an artifact directory.

    Returns the path relative to the artifact directory root, which
    corresponds to the container mount point.
    """
    artifact_dir = workspace_path(ws_name) / "artifacts" / artifact_id
    matches = list(artifact_dir.rglob("final_model.pt"))
    if not matches:
        print(f"ERROR: no final_model.pt in {artifact_dir}", file=sys.stderr)
        sys.exit(1)
    # Take the first (there should be exactly one for single-stage runs).
    return matches[0].relative_to(artifact_dir).as_posix()


def main():
    parser = argparse.ArgumentParser(
        description="Baseline RL: combat -> curation -> combined, "
                    "with flywheel block executions and evaluations")
    parser.add_argument("--subclass", required=True,
                        help="Fighter subclass (dueling, two_handed, defense)")
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--workspace", default=None,
                        help="Workspace name (default: baseline-<subclass>-<seed>)")
    parser.add_argument("--race", default="human")
    parser.add_argument("--synergy-group", default=None)
    parser.add_argument("--combat-steps", type=int, default=15_000_000)
    parser.add_argument("--curation-steps", type=int, default=5_000_000)
    parser.add_argument("--combined-steps", type=int, default=5_000_000)
    parser.add_argument("--eval-episodes", type=int, default=4000)
    args = parser.parse_args()

    ws_name = args.workspace or f"baseline-{uuid.uuid4().hex[:8]}"

    # Common train args passed through to the container entrypoint.
    train_common = [
        "--subclass", args.subclass,
        "--seed", str(args.seed),
        "--checkpoint-dir", "/output",
    ]
    if args.race != "human":
        train_common += ["--race", args.race]
    if args.synergy_group:
        train_common += ["--synergy-group", args.synergy_group]

    # Common eval args.
    eval_common = [
        "--subclass", args.subclass,
        "--episodes", str(args.eval_episodes),
        "--seed", str(args.seed),
        "--output-dir", "/output",
    ]
    if args.race != "human":
        eval_common += ["--race", args.race]
    if args.synergy_group:
        eval_common += ["--synergy-group", args.synergy_group]

    timings = {}
    t_start = time.time()

    # ── Create workspace ─────────────────────────────────────────
    run_flywheel([
        "create", "workspace",
        "--name", ws_name,
        "--template", TEMPLATE,
    ], "create workspace")

    # ── Phase 1: Combat (15M steps) ──────────────────────────────
    timings["train_combat"] = run_flywheel([
        "run", "block",
        "--workspace", str(workspace_path(ws_name)),
        "--block", "train",
        "--template", TEMPLATE,
        "--",
        "--combat-only",
        "--combat-timesteps", str(args.combat_steps),
        *train_common,
    ], "train/combat")

    combat_ckpt = latest_checkpoint_artifact(ws_name)
    combat_model_relpath = find_model_pt(ws_name, combat_ckpt)

    timings["eval_combat"] = run_flywheel([
        "run", "block",
        "--workspace", str(workspace_path(ws_name)),
        "--block", "eval",
        "--template", TEMPLATE,
        "--bind", f"checkpoint={combat_ckpt}",
        "--",
        "--checkpoint", f"/input/checkpoint/{combat_model_relpath}",
        *eval_common,
    ], "eval/combat")

    # ── Phase 2: Curation (5M steps, frozen combat heads) ────────
    timings["train_curation"] = run_flywheel([
        "run", "block",
        "--workspace", str(workspace_path(ws_name)),
        "--block", "train",
        "--template", TEMPLATE,
        "--bind", f"checkpoint={combat_ckpt}",
        "--",
        "--curation-only",
        "--curation-timesteps", str(args.curation_steps),
        "--init-model", f"/input/checkpoint/{combat_model_relpath}",
        *train_common,
    ], "train/curation")

    curation_ckpt = latest_checkpoint_artifact(ws_name)
    curation_model_relpath = find_model_pt(ws_name, curation_ckpt)

    timings["eval_curation"] = run_flywheel([
        "run", "block",
        "--workspace", str(workspace_path(ws_name)),
        "--block", "eval",
        "--template", TEMPLATE,
        "--bind", f"checkpoint={curation_ckpt}",
        "--",
        "--checkpoint", f"/input/checkpoint/{curation_model_relpath}",
        *eval_common,
    ], "eval/curation")

    # ── Phase 3: Combined (5M steps, all phases) ─────────────────
    timings["train_combined"] = run_flywheel([
        "run", "block",
        "--workspace", str(workspace_path(ws_name)),
        "--block", "train",
        "--template", TEMPLATE,
        "--bind", f"checkpoint={curation_ckpt}",
        "--",
        "--shared",
        "--shared-timesteps", str(args.combined_steps),
        "--init-model", f"/input/checkpoint/{curation_model_relpath}",
        *train_common,
    ], "train/combined")

    combined_ckpt = latest_checkpoint_artifact(ws_name)
    combined_model_relpath = find_model_pt(ws_name, combined_ckpt)

    timings["eval_combined"] = run_flywheel([
        "run", "block",
        "--workspace", str(workspace_path(ws_name)),
        "--block", "eval",
        "--template", TEMPLATE,
        "--bind", f"checkpoint={combined_ckpt}",
        "--",
        "--checkpoint", f"/input/checkpoint/{combined_model_relpath}",
        *eval_common,
    ], "eval/combined")

    # ── Summary ──────────────────────────────────────────────────
    total_time = time.time() - t_start
    ws_dir = workspace_path(ws_name)

    # Collect eval scores from score artifacts.
    ws = load_workspace_yaml(ws_name)
    score_artifacts = [
        (aid, entry) for aid, entry in ws.get("artifacts", {}).items()
        if entry["name"] == "score" and entry["kind"] == "copy"
    ]
    score_artifacts.sort(key=lambda x: x[1]["created_at"])

    scores = {}
    phase_labels = ["combat", "curation", "combined"]
    for i, (aid, _) in enumerate(score_artifacts):
        scores_path = ws_dir / "artifacts" / aid / "scores.json"
        if scores_path.exists() and i < len(phase_labels):
            scores[phase_labels[i]] = json.loads(scores_path.read_text())

    # Write research manifest alongside workspace.
    manifest = {
        "pipeline": "baseline_rl",
        "workspace": ws_name,
        "subclass": args.subclass,
        "seed": args.seed,
        "phases": {
            "combat": {"timesteps": args.combat_steps, "checkpoint": combat_ckpt},
            "curation": {"timesteps": args.curation_steps, "checkpoint": curation_ckpt},
            "combined": {"timesteps": args.combined_steps, "checkpoint": combined_ckpt},
        },
        "eval_episodes": args.eval_episodes,
        "timings_s": timings,
        "total_time_s": total_time,
        "scores_summary": {
            phase: {k: v for k, v in s.items() if k != "fights_won"}
            for phase, s in scores.items()
        },
    }
    manifest_path = ws_dir / "research_manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2))

    print(f"\n{'=' * 60}")
    print("Baseline RL pipeline complete.")
    print(f"  Workspace: {ws_dir}")
    print(f"  Time:      {total_time:.0f}s")
    for phase in phase_labels:
        if phase in scores:
            s = scores[phase]
            print(f"  {phase:10s}  mean={s['mean']:.1f}  median={s['median']:.0f}  "
                  f"std={s['std']:.1f}  [{s['min']}-{s['max']}]")
    print(f"  Manifest:  {manifest_path}")


if __name__ == "__main__":
    main()
