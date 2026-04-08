"""Bootstrap confidence intervals from evaluation scores.

Reads scores.json files produced by eval_checkpoint.py and computes
bootstrapped confidence intervals for the mean fights won.

Usage:
    python scripts/bootstrap_ci.py runs/sweep/dueling_run1/eval/scores.json
    python scripts/bootstrap_ci.py runs/sweep/*/eval/scores.json
    python scripts/bootstrap_ci.py runs/calibration/shared_*/eval/scores.json
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import numpy as np


def bootstrap_ci(
    fights_won: list[int],
    n_boot: int = 10000,
    ci: float = 95.0,
    seed: int = 42,
) -> tuple[float, float, float]:
    """Compute bootstrapped confidence interval for the mean.

    Args:
        fights_won: Per-episode fight counts.
        n_boot: Number of bootstrap resamples.
        ci: Confidence level (e.g., 95.0 for 95% CI).
        seed: RNG seed for reproducibility.

    Returns:
        Tuple of (mean, ci_lower, ci_upper).
    """
    rng = np.random.default_rng(seed)
    data = np.array(fights_won)
    means = np.array([
        np.mean(rng.choice(data, size=len(data), replace=True))
        for _ in range(n_boot)
    ])
    lo = np.percentile(means, (100 - ci) / 2)
    hi = np.percentile(means, 100 - (100 - ci) / 2)
    return float(np.mean(data)), float(lo), float(hi)


def main(argv: list[str] | None = None) -> None:
    """Parse arguments and print bootstrap CIs for each scores file.

    Args:
        argv: Command-line arguments. Defaults to sys.argv when None.
    """
    parser = argparse.ArgumentParser(
        description="Bootstrap confidence intervals from scores.json files.",
    )
    parser.add_argument(
        "files", nargs="+", type=Path,
        help="Path(s) to scores.json files.",
    )
    parser.add_argument(
        "--n-boot", type=int, default=10000,
        help="Number of bootstrap resamples (default: 10000).",
    )
    parser.add_argument(
        "--ci", type=float, default=95.0,
        help="Confidence level (default: 95.0).",
    )
    args = parser.parse_args(argv)

    for path in args.files:
        if not path.exists():
            print(f"{path}: not found", file=sys.stderr)
            continue

        with open(path) as f:
            data = json.load(f)

        fights = data.get("fights_won")
        if fights is None:
            print(f"{path}: no 'fights_won' field", file=sys.stderr)
            continue

        mean, lo, hi = bootstrap_ci(fights, n_boot=args.n_boot, ci=args.ci)
        label = str(path.parent.parent.name)
        print(f"{label:30s}  mean={mean:.2f}  {args.ci}% CI=[{lo:.2f}, {hi:.2f}]  n={len(fights)}")


if __name__ == "__main__":
    main()
