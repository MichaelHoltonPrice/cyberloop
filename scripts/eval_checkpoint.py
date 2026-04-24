#!/usr/bin/env python3
"""Evaluate an RL checkpoint by running game episodes in parallel.

Loads a checkpoint, reconstructs the model, runs greedy episodes
across multiple worker processes, writes scores to /output and
an output manifest.

Designed to run inside a Docker container as the eval entry point.

Supports two backends:
    numpy  — pure-numpy inference, ~5x faster, no torch dependency
    torch  — original PyTorch inference (for validation/debugging)

Container conventions:
    /input/checkpoint/checkpoint.pt — the checkpoint to evaluate
    /input/checkpoint/run.json      — checkpoint metadata
    /output/score/scores.json       — evaluation results
    /output/score/output_manifest.json — declares produced artifacts

Usage:
    python eval_checkpoint.py \
        --checkpoint /input/checkpoint/checkpoint.pt \
        --episodes 4000 \
        --output-dir /output
"""

import argparse
import json
import sys
from pathlib import Path

# Ensure scripts/ is importable.
_scripts_dir = str(Path(__file__).resolve().parent)
if _scripts_dir not in sys.path:
    sys.path.insert(0, _scripts_dir)

from eval_io import print_summary, write_scores


def _fail(message: str) -> None:
    """Print a user-facing error and exit nonzero."""
    print(f"ERROR: {message}", file=sys.stderr)
    sys.exit(1)


def write_flywheel_termination(reason: str = "normal") -> None:
    """Announce successful completion to the Flywheel sidecar if mounted."""
    termination = Path("/flywheel/termination")
    if termination.parent.exists():
        termination.write_text(reason, encoding="utf-8")


def write_manifest(output_dir, artifacts):
    """Write output_manifest.json declaring produced artifacts."""
    manifest = {"artifacts": artifacts}
    (output_dir / "output_manifest.json").write_text(
        json.dumps(manifest, indent=2))


def load_checkpoint_metadata(checkpoint_path: Path) -> dict:
    """Load and validate the Cyberloop checkpoint artifact metadata."""
    if not checkpoint_path.exists():
        _fail(f"checkpoint file does not exist: {checkpoint_path}")
    metadata_path = checkpoint_path.parent / "run.json"
    if not metadata_path.exists():
        _fail(f"checkpoint metadata does not exist: {metadata_path}")
    try:
        metadata = json.loads(metadata_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        _fail(f"checkpoint metadata is invalid JSON: {exc}")
    if metadata.get("schema_version") != 1:
        _fail("checkpoint run.json must have schema_version 1")
    if metadata.get("artifact_type") != "cyberloop.checkpoint":
        _fail("checkpoint run.json has wrong artifact_type")
    if metadata.get("model_path") != checkpoint_path.name:
        _fail(
            "checkpoint run.json model_path must match "
            f"{checkpoint_path.name!r}")
    subclass = metadata.get("subclass")
    if not isinstance(subclass, str) or not subclass:
        _fail("checkpoint run.json must declare a non-empty subclass")
    return metadata


def _run_numpy(args, dims, output_dir):
    """Run evaluation using numpy backend (no torch needed)."""
    from numpy_eval import evaluate_parallel

    stats = evaluate_parallel(
        checkpoint_path=args.checkpoint,
        subclass=args.subclass,
        n_episodes=args.episodes,
        dims=dims,
        seed=args.seed,
        race=args.race,
        synergy_group=args.synergy_group,
        n_workers=args.n_workers,
    )
    return stats


def _run_torch(args, dims, output_dir):
    """Run evaluation using torch backend."""
    from eval_common import (
        evaluate_parallel,
        load_checkpoint,
    )

    _state_dict, model_kwargs = load_checkpoint(args.checkpoint)

    if model_kwargs is None:
        model_kwargs = {
            "global_dim": dims[0],
            "card_feat_dim": dims[1],
            "enemy_feat_dim": dims[2],
            "action_feat_dim": dims[3],
            "vocab_size": dims[4],
        }
        print(
            "Warning: legacy checkpoint without model_kwargs, "
            "resolved dims from game engine",
            file=sys.stderr)

    stats = evaluate_parallel(
        checkpoint_path=args.checkpoint,
        model_kwargs=model_kwargs,
        subclass=args.subclass,
        n_episodes=args.episodes,
        dims=dims,
        seed=args.seed,
        race=args.race,
        synergy_group=args.synergy_group,
        n_workers=args.n_workers,
    )
    return stats


def main():
    parser = argparse.ArgumentParser(
        description="Evaluate an RL checkpoint through "
                    "parallel game episodes")
    parser.add_argument(
        "--checkpoint", required=True,
        help="Path to checkpoint .pt file")
    parser.add_argument("--subclass", default=None)
    parser.add_argument("--episodes", type=int, default=4000)
    parser.add_argument("--race", default="human")
    parser.add_argument("--synergy-group", default=None)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--n-workers", type=int, default=None,
                        help="Parallel workers (default: CPU count)")
    parser.add_argument(
        "--backend", choices=["numpy", "torch"], default="numpy",
        help="Inference backend (default: numpy)")
    parser.add_argument(
        "--output-dir", default="/output",
        help="Directory for output files")
    args = parser.parse_args()
    if args.episodes <= 0:
        _fail("--episodes must be positive")

    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    checkpoint_path = Path(args.checkpoint)
    checkpoint_metadata = load_checkpoint_metadata(checkpoint_path)
    checkpoint_subclass = checkpoint_metadata["subclass"]
    if args.subclass is None:
        args.subclass = checkpoint_subclass
    elif args.subclass != checkpoint_subclass:
        _fail(
            f"requested subclass {args.subclass!r} does not match "
            f"checkpoint subclass {checkpoint_subclass!r}")

    # Resolve dims from game engine (works with either backend).
    from numpy_eval import resolve_dims
    dims = resolve_dims(
        args.subclass, args.race, args.synergy_group)

    # Run evaluation.
    if args.backend == "numpy":
        stats = _run_numpy(args, dims, output_dir)
    else:
        stats = _run_torch(args, dims, output_dir)

    # Write scores to file.
    write_scores(output_dir, stats)

    # Write manifest.
    write_manifest(output_dir, [
        {"type": "score", "path": "scores.json",
             "metadata": {
                 "mean": stats["mean"],
                 "median": stats["median"],
                 "episodes": stats["episodes"],
                 "errors": stats.get("errors", 0),
             }},
    ])

    # Print summary to stdout (per-episode fights_won stays in scores.json only).
    print_summary(stats)
    write_flywheel_termination("normal")


if __name__ == "__main__":
    main()
