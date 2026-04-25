# Cyberloop

Decker is a roguelike deckbuilder. Cyberloop wraps it as an evaluation
environment for RL training loops orchestrated by [flywheel](../flywheel/).

## Crate structure

Two independent Cargo workspaces keep rendering dependencies out of the engine:

**Engine workspace** (`/Cargo.toml`):
- `crates/engine` — core state machine: combat, cards, enemies, RNG. No rendering.
- `crates/content` — card/enemy/encounter definitions (Fighter class, three subclasses).
- `crates/gauntlet` — 50-fight gauntlet game mode with progressive scaling.
- `crates/pyenv` — PyO3 bindings exposing GauntletEnv to Python.

**Client workspace** (`/client/Cargo.toml`):
- `client/crates/client` — Bevy GUI for interactive play.
- `client/crates/card_renderer` — reusable card rendering.

The client depends on engine crates via relative paths but compiles independently.

## Python surface

- `scripts/` — RL training (`train_impala.py`), evaluation
  (`eval_checkpoint.py`, `eval_common.py`, `numpy_eval.py`), shared model
  (`model.py`), Gym wrapper (`decker_env.py`), and bot harness.
- `cyberloop/` - project hooks and validators loaded by Flywheel.
- `foundry/templates/blocks/` - Train and Eval block templates.
- `foundry/templates/workspaces/` - workspace templates, including the
  shared `checkpoint`/`score` artifact contract.
- `foundry/templates/patterns/` - pattern templates, including the
  canonical Train to Eval pattern.
- `docker/` - block container Dockerfiles.

## Build and test

```bash
cargo test                      # engine unit tests
cargo run -p decker-client      # GUI (requires Bevy)
pytest                          # cyberloop project-hook tests
```

## Platform

Windows 11, Docker Desktop (WSL2). NVIDIA RTX GPU for training.
