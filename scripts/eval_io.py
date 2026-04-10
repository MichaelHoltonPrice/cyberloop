"""Shared evaluation output utilities.

Used by both eval_checkpoint.py (RL eval) and eval_bot.py (bot eval)
to write scores and print summaries in a consistent format.
"""

import json
from pathlib import Path


def write_scores(output_dir: Path, stats: dict) -> Path:
    """Write evaluation stats to scores.json.

    Args:
        output_dir: Directory to write into (created if needed).
        stats: Stats dict including per-episode fights_won.

    Returns:
        Path to the written scores.json file.
    """
    output_dir.mkdir(parents=True, exist_ok=True)
    scores_path = output_dir / "scores.json"
    scores_path.write_text(json.dumps(stats, indent=2))
    return scores_path


def print_summary(stats: dict) -> None:
    """Print evaluation summary to stdout.

    Excludes the per-episode fights_won list for brevity.
    """
    summary = {k: v for k, v in stats.items() if k != "fights_won"}
    print(json.dumps(summary))
