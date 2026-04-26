#!/usr/bin/env python3
"""Evaluate a bot.py by running gauntlet episodes in parallel.

Loads a bot module (must define player_fn), runs episodes, writes
scores to the output directory.

Designed to run inside a Docker container as the bot eval entry point.

Container conventions:
    /input/bot/bot.py      — the bot to evaluate (mounted via flywheel)
    /output/scores.json    — evaluation results

Usage:
    python eval_bot.py \
        --bot /input/bot/bot.py \
        --subclass dueling \
        --episodes 200 \
        --output-dir /output
"""

import argparse
import json
import os
import random
import sys
from pathlib import Path

# Ensure scripts/ is importable.
_scripts_dir = str(Path(__file__).resolve().parent)
if _scripts_dir not in sys.path:
    sys.path.insert(0, _scripts_dir)

from eval_io import print_summary, write_scores  # noqa: E402
from play_bot import play_parallel  # noqa: E402


def write_flywheel_termination(reason: str = "normal") -> None:
    """Announce a Flywheel termination reason when running in Flywheel."""
    termination = Path("/flywheel/termination")
    if termination.parent.exists():
        termination.write_text(reason, encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(
        description="Evaluate a bot through parallel gauntlet episodes")
    parser.add_argument(
        "--bot", default="/input/bot/bot.py",
        help="Path to bot .py file (must define player_fn)")
    parser.add_argument("--subclass", required=True)
    parser.add_argument("--episodes", type=int, default=200)
    parser.add_argument("--race", default="human")
    parser.add_argument("--background", default="soldier")
    parser.add_argument("--seed", type=int, default=None,
                        help="RNG seed (default: random)")
    parser.add_argument("--n-workers", type=int, default=None,
                        help="Parallel workers (default: CPU count)")
    parser.add_argument(
        "--output-dir", default="/output",
        help="Directory for output files")
    args = parser.parse_args()

    output_dir = Path(args.output_dir)

    seed = args.seed if args.seed is not None else random.randint(0, 2**31)

    stats = play_parallel(
        bot_path=args.bot,
        subclass=args.subclass,
        episodes=args.episodes,
        seed=seed,
        race=args.race,
        background=args.background,
        n_workers=args.n_workers,
        verbose=True,
    )

    # Record the seed used so results are reproducible.
    stats["seed"] = seed

    # Write scores (even for aborted evals, for debugging).
    write_scores(output_dir, stats)
    (output_dir / "output_manifest.json").write_text(json.dumps({
        "artifacts": [{
            "type": "score",
            "path": "scores.json",
        }],
    }, indent=2), encoding="utf-8")
    print_summary(stats)

    # If the eval was aborted (speed screen or timeout gate),
    # complete normally with the "aborted" termination reason so
    # the score artifact gets promoted and the next agent in the
    # lane can read it (the score JSON carries aborted=true and
    # abort_reason / abort_message detail).
    if stats.get("aborted"):
        print(stats.get("abort_message", "Eval aborted"),
              file=sys.stderr)
        write_flywheel_termination("aborted")
        os._exit(0)
    write_flywheel_termination("normal")
    # Force-exit to bypass ProcessPoolExecutor atexit cleanup, which
    # can hang on lingering workers and leave the container alive
    # forever after the eval result has already been written.
    os._exit(0)


if __name__ == "__main__":
    main()
