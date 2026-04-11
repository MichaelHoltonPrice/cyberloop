# Baseline RL experiment

Three-phase IMPALA training pipeline for the Decker gauntlet, orchestrated
via flywheel block executions within a single workspace.

| Phase    | Steps | Description                                 |
|----------|------:|---------------------------------------------|
| Combat   |  15M  | Card-play decisions only                    |
| Curation |   5M  | Reward/deck decisions (combat heads frozen) |
| Combined |   5M  | All phases jointly                          |

Each phase is followed by a 4000-episode evaluation.

## Prerequisites

1. **Docker images built** (from cyberloop root):
   ```
   docker build -f docker/Dockerfile.train -t cyberloop-train:latest .
   docker build -f docker/Dockerfile.eval-rl -t cyberloop-eval-rl:latest .
   ```

2. **Flywheel installed** in the cyberloop venv (see Setup below).

## Setup

Create the venv and install dependencies (one-time):

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
python research/baseline_rl.py --subclass dueling --seed 42
```

## What it does

1. Creates a flywheel workspace named `baseline-<uuid>` from the `train_eval` template.
2. Runs `flywheel run block --block train` three times (combat, curation, combined),
   chaining the checkpoint artifact from each phase into the next via `--bind`.
3. Runs `flywheel run block --block eval` after each training phase (4000 episodes).
4. Writes `research_manifest.json` into the workspace with scores, timings, and
   artifact IDs for reproducibility.

## Options

| Flag               | Default    | Description                          |
|--------------------|------------|--------------------------------------|
| `--subclass`       | (required) | `dueling`, `two_handed`, or `defense`|
| `--seed`           | `42`       | RNG seed for reproducibility         |
| `--workspace`      | auto       | Override workspace name              |
| `--combat-steps`   | `15000000` | Combat training timesteps            |
| `--curation-steps` | `5000000`  | Curation training timesteps          |
| `--combined-steps` | `5000000`  | Combined training timesteps          |
| `--eval-episodes`  | `4000`     | Episodes per evaluation              |

## Output

All artifacts live in `foundry/workspaces/<workspace-name>/`:

```
foundry/workspaces/baseline-a3f7b2c1/
  workspace.yaml            # full provenance graph
  research_manifest.json    # scores, timings, config
  artifacts/
    checkpoint@<id>/        # combat checkpoint
    score@<id>/             # combat eval scores
    checkpoint@<id>/        # curation checkpoint
    score@<id>/             # curation eval scores
    checkpoint@<id>/        # combined checkpoint
    score@<id>/             # combined eval scores
```
