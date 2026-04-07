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
scripts/          Headless play harness, player implementations
docker/           Container definitions for training and evaluation
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
```

## Orchestration

Game runs are orchestrated by [Flywheel](../flywheel/), a sibling project that
wires Docker containers into measurable improvement loops. See the flywheel repo
for details on running training pipelines, agent sweeps, and evaluations.

## License

Licensed under the **PolyForm Shield License 1.0.0**. See [LICENSE](LICENSE).

Copyright (c) 2026 Heartland AI (dba Hopewell AI)
