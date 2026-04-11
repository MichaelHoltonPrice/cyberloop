#!/usr/bin/env python3
"""Baseline RL training for all three Decker subclasses.

Runs six training runs per subclass (dueling, two_handed, defense),
each a three-phase IMPALA pipeline orchestrated via flywheel:
  1. Combat    — 15M steps, card-play decisions only
  2. Curation  — 5M steps, reward/deck decisions (combat heads frozen)
  3. Combined  — 5M steps, all phases jointly

Each phase is followed by a 4000-episode evaluation. All runs use
deterministic seeds derived from a single baseline seed.

Requires:
  - flywheel installed (pip install -e ../flywheel)
  - Docker images built: cyberloop-train:latest, cyberloop-eval-rl:latest
  - CWD = cyberloop project root (where flywheel.yaml lives)

Usage:
    cd cyberloop
    python research/baseline_rl.py
    python research/baseline_rl.py --baseline-seed 123
"""

import json
import random
import subprocess
import sys
import time
import uuid
from pathlib import Path

import yaml

TEMPLATE = "train_eval"
PROJECT_ROOT = Path(__file__).resolve().parent.parent
FOUNDRY_DIR = PROJECT_ROOT / "foundry"

SUBCLASSES = ["dueling", "two_handed", "defense"]
RUNS_PER_SUBCLASS = 6

COMBAT_STEPS = 15_000_000
CURATION_STEPS = 5_000_000
COMBINED_STEPS = 5_000_000
EVAL_EPISODES = 4000


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
    """Find the most recently produced checkpoint artifact ID."""
    ws = load_workspace_yaml(ws_name)
    checkpoint_artifacts = [
        (aid, entry) for aid, entry in ws.get("artifacts", {}).items()
        if entry["name"] == "checkpoint" and entry["kind"] == "copy"
    ]
    if not checkpoint_artifacts:
        print("ERROR: no checkpoint artifacts found in workspace", file=sys.stderr)
        sys.exit(1)
    checkpoint_artifacts.sort(key=lambda x: x[1]["created_at"])
    return checkpoint_artifacts[-1][0]


def find_model_pt(ws_name, artifact_id):
    """Find final_model.pt within an artifact directory.

    Returns the path relative to the artifact root (matches
    the container mount point).
    """
    artifact_dir = workspace_path(ws_name) / "artifacts" / artifact_id
    matches = list(artifact_dir.rglob("final_model.pt"))
    if not matches:
        print(f"ERROR: no final_model.pt in {artifact_dir}", file=sys.stderr)
        sys.exit(1)
    return matches[0].relative_to(artifact_dir).as_posix()


def run_single_pipeline(subclass, seed, ws_name):
    """Run one combat → curation → combined pipeline in a workspace."""

    train_common = [
        "--subclass", subclass,
        "--seed", str(seed),
        "--checkpoint-dir", "/output",
    ]
    eval_common = [
        "--subclass", subclass,
        "--episodes", str(EVAL_EPISODES),
        "--seed", str(seed),
        "--output-dir", "/output",
    ]

    tag = f"{subclass}/seed-{seed}"
    ws = str(workspace_path(ws_name))
    timings = {}

    # ── Create workspace ─────────────────────────────────────────
    run_flywheel([
        "create", "workspace",
        "--name", ws_name,
        "--template", TEMPLATE,
    ], f"{tag}/create")

    # ── Phase 1: Combat (15M steps) ──────────────────────────────
    timings["train_combat"] = run_flywheel([
        "run", "block", "--workspace", ws, "--block", "train",
        "--template", TEMPLATE, "--",
        "--combat-only", "--combat-timesteps", str(COMBAT_STEPS),
        *train_common,
    ], f"{tag}/train/combat")

    combat_ckpt = latest_checkpoint_artifact(ws_name)
    combat_relpath = find_model_pt(ws_name, combat_ckpt)

    timings["eval_combat"] = run_flywheel([
        "run", "block", "--workspace", ws, "--block", "eval",
        "--template", TEMPLATE, "--bind", f"checkpoint={combat_ckpt}", "--",
        "--checkpoint", f"/input/checkpoint/{combat_relpath}",
        *eval_common,
    ], f"{tag}/eval/combat")

    # ── Phase 2: Curation (5M steps, frozen combat heads) ────────
    timings["train_curation"] = run_flywheel([
        "run", "block", "--workspace", ws, "--block", "train",
        "--template", TEMPLATE, "--bind", f"checkpoint={combat_ckpt}", "--",
        "--curation-only", "--curation-timesteps", str(CURATION_STEPS),
        "--init-model", f"/input/checkpoint/{combat_relpath}",
        *train_common,
    ], f"{tag}/train/curation")

    curation_ckpt = latest_checkpoint_artifact(ws_name)
    curation_relpath = find_model_pt(ws_name, curation_ckpt)

    timings["eval_curation"] = run_flywheel([
        "run", "block", "--workspace", ws, "--block", "eval",
        "--template", TEMPLATE, "--bind", f"checkpoint={curation_ckpt}", "--",
        "--checkpoint", f"/input/checkpoint/{curation_relpath}",
        *eval_common,
    ], f"{tag}/eval/curation")

    # ── Phase 3: Combined (5M steps, all phases) ─────────────────
    timings["train_combined"] = run_flywheel([
        "run", "block", "--workspace", ws, "--block", "train",
        "--template", TEMPLATE, "--bind", f"checkpoint={curation_ckpt}", "--",
        "--shared", "--shared-timesteps", str(COMBINED_STEPS),
        "--init-model", f"/input/checkpoint/{curation_relpath}",
        *train_common,
    ], f"{tag}/train/combined")

    combined_ckpt = latest_checkpoint_artifact(ws_name)
    combined_relpath = find_model_pt(ws_name, combined_ckpt)

    timings["eval_combined"] = run_flywheel([
        "run", "block", "--workspace", ws, "--block", "eval",
        "--template", TEMPLATE, "--bind", f"checkpoint={combined_ckpt}", "--",
        "--checkpoint", f"/input/checkpoint/{combined_relpath}",
        *eval_common,
    ], f"{tag}/eval/combined")

    return {
        "combat": combat_ckpt,
        "curation": curation_ckpt,
        "combined": combined_ckpt,
    }, timings


def collect_scores(ws_name):
    """Read eval scores from score artifacts in a workspace."""
    ws_dir = workspace_path(ws_name)
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
    return scores


def main():
    baseline_seed = 42
    if len(sys.argv) > 1 and sys.argv[1] == "--baseline-seed":
        baseline_seed = int(sys.argv[2])

    # Derive per-run seeds deterministically.
    rng = random.Random(baseline_seed)
    run_seeds = {
        subclass: [rng.randint(0, 2**31 - 1) for _ in range(RUNS_PER_SUBCLASS)]
        for subclass in SUBCLASSES
    }

    print(f"Baseline RL experiment")
    print(f"  Baseline seed: {baseline_seed}")
    print(f"  Subclasses:    {', '.join(SUBCLASSES)}")
    print(f"  Runs each:     {RUNS_PER_SUBCLASS}")
    print(f"  Phases:        combat ({COMBAT_STEPS/1e6:.0f}M) → "
          f"curation ({CURATION_STEPS/1e6:.0f}M) → "
          f"combined ({COMBINED_STEPS/1e6:.0f}M)")
    print(f"  Eval episodes: {EVAL_EPISODES}")
    total_runs = len(SUBCLASSES) * RUNS_PER_SUBCLASS
    print(f"  Total runs:    {total_runs}")

    results = {}
    t_start = time.time()

    for subclass in SUBCLASSES:
        for run_idx, seed in enumerate(run_seeds[subclass]):
            ws_name = f"baseline-{subclass}-{uuid.uuid4().hex[:8]}"
            label = f"{subclass} run {run_idx + 1}/{RUNS_PER_SUBCLASS} (seed={seed})"
            print(f"\n{'#' * 60}")
            print(f"# {label}")
            print(f"# workspace: {ws_name}")
            print(f"{'#' * 60}")

            checkpoints, timings = run_single_pipeline(subclass, seed, ws_name)
            scores = collect_scores(ws_name)

            results.setdefault(subclass, []).append({
                "run_index": run_idx,
                "seed": seed,
                "workspace": ws_name,
                "checkpoints": checkpoints,
                "timings_s": timings,
                "scores": {
                    phase: {k: v for k, v in s.items() if k != "fights_won"}
                    for phase, s in scores.items()
                },
            })

    total_time = time.time() - t_start

    # Write experiment manifest.
    manifest = {
        "experiment": "baseline_rl",
        "baseline_seed": baseline_seed,
        "config": {
            "combat_steps": COMBAT_STEPS,
            "curation_steps": CURATION_STEPS,
            "combined_steps": COMBINED_STEPS,
            "eval_episodes": EVAL_EPISODES,
            "runs_per_subclass": RUNS_PER_SUBCLASS,
        },
        "run_seeds": run_seeds,
        "results": results,
        "total_time_s": total_time,
    }
    manifest_path = PROJECT_ROOT / "research" / "baseline_rl_manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2))

    # Print summary.
    print(f"\n{'=' * 60}")
    print("Baseline RL experiment complete.")
    print(f"  Total time: {total_time:.0f}s")
    for subclass in SUBCLASSES:
        runs = results.get(subclass, [])
        for phase in ("combat", "curation", "combined"):
            means = [r["scores"][phase]["mean"] for r in runs if phase in r["scores"]]
            if means:
                avg = sum(means) / len(means)
                print(f"  {subclass:12s} {phase:10s}  "
                      f"avg_mean={avg:.1f} across {len(means)} runs")
    print(f"  Manifest: {manifest_path}")


if __name__ == "__main__":
    main()
