# Cyberloop

Decker is a roguelike deckbuilder. Cyberloop wraps it as an evaluation environment
for AI improvement loops orchestrated by [flywheel](../flywheel/).

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

## Build and test

```bash
cargo test              # engine unit tests
cargo run -p decker-client   # GUI (requires Bevy)
```

## Platform

Windows 11, Docker Desktop (WSL2). NVIDIA RTX GPU for future training work.
