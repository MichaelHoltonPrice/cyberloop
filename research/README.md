# Baseline RL experiment

Three-phase IMPALA training for all three Decker subclasses (dueling,
two_handed, defense), with 6 runs per subclass using deterministic seeds.

| Phase    | Steps | Description                                 |
|----------|------:|---------------------------------------------|
| Combat   |  15M  | Card-play decisions only                    |
| Curation |   5M  | Reward/deck decisions (combat heads frozen) |
| Combined |   5M  | All phases jointly                          |

Each phase is followed by a 4000-episode evaluation. Total: 18 runs
(3 subclasses x 6 runs), each with 6 flywheel block executions.

## Prerequisites

1. **Docker images built** (from cyberloop root):
   ```
   docker build -f docker/Dockerfile.train -t cyberloop-train:latest .
   docker build -f docker/Dockerfile.eval-rl -t cyberloop-eval-rl:latest .
   ```

2. **Flywheel installed** in a Python environment (see Setup below).

## Setup

```bash
cd C:\Users\micha\github\cyber-root\cyberloop
python -m venv .venv

# Windows cmd
.venv\Scripts\activate

# Git Bash / WSL / macOS / Linux
source .venv/Scripts/activate

pip install pyyaml
pip install -e ../flywheel
```

## Run

```bash
cd C:\Users\micha\github\cyber-root\cyberloop

# Windows cmd
.venv\Scripts\activate

# Git Bash / WSL / macOS / Linux
source .venv/Scripts/activate

python research/baseline_rl.py
python research/baseline_rl.py --baseline-seed 123  # different seed
```

## What it does

1. Derives 6 per-run seeds for each subclass from a single baseline seed (default 42).
2. For each subclass and seed, creates a flywheel workspace and runs the three-phase
   pipeline (combat → curation → combined), with evaluations after each phase.
3. All 18 runs execute sequentially.
4. Writes `research/baseline_rl_manifest.json` with all scores, timings, seeds,
   and workspace names.

## Output

Each run creates a flywheel workspace in `foundry/workspaces/`:

```
foundry/workspaces/baseline-dueling-a3f7b2c1/
  workspace.yaml            # full provenance graph
  artifacts/
    checkpoint@<id>/        # combat / curation / combined checkpoints
    score@<id>/             # eval scores (scores.json)
```

The experiment-level manifest at `research/baseline_rl_manifest.json` aggregates
results across all runs for analysis.
