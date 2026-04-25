# Cyberloop

A research testbed for studying how AI systems evolve toward competent play
through cybernetic feedback loops. *Cyber* here is used in its original Greek
sense -- *kybernetes*, "steersman" -- as in classical cybernetics: the study of
control and communication through feedback.

## What's here

Cyberloop bundles **Decker**, a roguelike deckbuilder, as its evaluation
environment. The engine provides:

- A deterministic, seeded game that can be played headlessly via Python
- Structured observations (JSON) and enumerated legal actions
- A 50-fight gauntlet with progressive difficulty scaling
- Currently: Fighter class with three subclasses (Dueling, Two-Handed, Defense)

### Directory structure

```
crates/
  engine/         Rust game engine -- combat, cards, statuses, RNG
  content/        Card definitions, enemies, encounters
  gauntlet/       Gauntlet game mode, observation serialization
  pyenv/          PyO3 bridge -- GauntletEnv accessible from Python
client/
  crates/
    client/       Bevy GUI client -- play the game interactively
    card_renderer/  Reusable card rendering
scripts/          RL training (train_impala.py), eval, model, env wrapper
cyberloop/        Project hooks and validators loaded by Flywheel
docker/           Block container Dockerfiles (Dockerfile.eval, ...)
foundry/          Templates and Flywheel workspaces
tests/            Unit tests for the project hooks and blocks
```

## Setup

```bash
# Build and test the engine
cargo test

# Play interactively (requires Bevy dependencies)
cargo run -p decker-client

# Python bindings
python -m venv .venv
.venv/Scripts/pip install -r requirements.txt
cd crates/pyenv
../../.venv/Scripts/maturin develop --release
cd ../..

# Project hooks + dev tools (editable install of the cyberloop package).
# Flywheel itself is installed editable from the sibling repo:
.venv/Scripts/pip install -e ../flywheel
.venv/Scripts/pip install -e ".[dev]"
```

## Orchestration

Train and Eval can be run ad hoc through Flywheel's canonical one-shot
container pipeline, and `train_eval` runs the same blocks as a pattern.
The currently supported surface:

- `foundry/templates/blocks/Train.yaml` runs `train_impala.py` and writes a
  flat `checkpoint` artifact with `checkpoint.pt` and `run.json`.
- `foundry/templates/blocks/Eval.yaml` runs `eval_checkpoint.py` against a
  pre-staged checkpoint artifact and writes a `score` artifact.
- `foundry/templates/workspaces/cyberloop.yaml` declares the
  `checkpoint` and `score` artifact contract.
- `foundry/templates/patterns/train_eval.yaml` declares the canonical
  Train to Eval pattern.
- `cyberloop.artifact_validators` validates checkpoint and score
  artifacts through Flywheel's `artifact_validators` hook.

Ad hoc training uses the base Flywheel block command from the cyberloop root:

```bash
flywheel run block --workspace foundry/workspaces/<workspace> --block Train --template cyberloop -- --subclass dueling --combat-only
```

## License

Licensed under the **PolyForm Shield License 1.0.0**. See [LICENSE](LICENSE).

Copyright (c) 2026 Heartland AI (dba Hopewell AI)
