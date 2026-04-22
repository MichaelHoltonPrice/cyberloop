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
cyberloop/        Project hooks loaded by `flywheel run pattern`
tests/            Unit tests for the project hooks
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

Pattern runs are driven by [Flywheel](../flywheel/), the sibling substrate that
launches block containers and stitches their I/O through workspace artifacts.
The repo currently ships the project-hooks skeleton only: `flywheel.yaml`
points at `cyberloop.project:ProjectHooks`, which hands the substrate a single
`ProcessExitExecutor` for every block.  Real block definitions
(`workforce/blocks/`) and patterns (`patterns/`) — starting with a
training-segment block, an evaluation block, and an alternating train-eval
pattern — land in follow-up commits.  See the flywheel repo for details on
creating workspaces and running patterns once those blocks land.

## License

Licensed under the **PolyForm Shield License 1.0.0**. See [LICENSE](LICENSE).

Copyright (c) 2026 Heartland AI (dba Hopewell AI)
