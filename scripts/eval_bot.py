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
import sys
from pathlib import Path

# Ensure scripts/ is importable.
_scripts_dir = str(Path(__file__).resolve().parent)
if _scripts_dir not in sys.path:
    sys.path.insert(0, _scripts_dir)

from eval_io import print_summary, write_scores
from play_bot import play_parallel


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
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--n-workers", type=int, default=None,
                        help="Parallel workers (default: CPU count)")
    parser.add_argument(
        "--output-dir", default="/output",
        help="Directory for output files")
    args = parser.parse_args()

    output_dir = Path(args.output_dir)

    stats = play_parallel(
        bot_path=args.bot,
        subclass=args.subclass,
        episodes=args.episodes,
        seed=args.seed,
        race=args.race,
        background=args.background,
        n_workers=args.n_workers,
        verbose=True,
    )

    # Write scores (even for aborted evals, for debugging).
    write_scores(output_dir, stats)
    print_summary(stats)

    # If the eval was aborted (speed screen or timeout gate),
    # exit with code 2 so flywheel records it as a failed execution.
    if stats.get("aborted"):
        print(stats.get("abort_message", "Eval aborted"),
              file=sys.stderr)
        sys.exit(2)


if __name__ == "__main__":
    main()
