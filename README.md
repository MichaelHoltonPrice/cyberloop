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
scripts/          RL training, checkpoint eval, bot eval, model, env wrapper
cyberloop/        Validators loaded by Flywheel
docker/           Block container Dockerfiles (Dockerfile.eval, ...)
foundry/          Templates and Flywheel workspaces
tests/            Unit tests for validators and block declarations
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

# Flywheel itself is installed editable from the sibling repo:
.venv/Scripts/pip install -e ../flywheel
.venv/Scripts/pip install -e ".[dev]"
```

## Orchestration

Train and Eval can be run ad hoc through Flywheel's canonical one-shot
container pipeline, and `train_eval` runs the same blocks as a pattern.
Cyberloop also includes an `improve_bot` pattern: a Claude battery block edits
`bot.py` and can request `EvalBot` by committing a bot artifact under the
`eval_requested` termination reason.
The currently supported surface:

- `foundry/templates/blocks/Train.yaml` runs `train_impala.py` and writes a
  flat `checkpoint` artifact with `checkpoint.pt` and `run.json`.
- `foundry/templates/blocks/Eval.yaml` runs `eval_checkpoint.py` against a
  pre-staged checkpoint artifact and writes a `score` artifact.
- `foundry/templates/blocks/EvalBot.yaml` runs `eval_bot.py` against a
  Python `bot.py` artifact and writes a `score` artifact.
- `foundry/templates/blocks/ImproveBot.yaml` runs a Cyberloop image
  derived from Flywheel's Claude battery and routes `eval_requested`
  to `EvalBot`.
- `foundry/templates/workspaces/cyberloop.yaml` declares the
  `checkpoint`, `score`, and `bot` artifact contract.
- `foundry/templates/patterns/train_eval.yaml` declares the canonical
  Train to Eval pattern.
- `foundry/templates/patterns/improve_bot.yaml` declares the agent-improves-bot
  pattern: three run-scoped lanes, each seeded from the checked-in
  baseline bot fixture before the first ImproveBot execution.
- `cyberloop.artifact_validators` validates checkpoint and score
  artifacts through Flywheel's `artifact_validators` hook.

Build the Claude battery image, then the Cyberloop image that bakes
in the ImproveBot prompt:

```bash
docker build -f ../flywheel/batteries/claude/Dockerfile.claude -t flywheel-claude:latest ../flywheel/batteries/claude
docker build -f docker/Dockerfile.improve-bot-agent -t cyberloop-improve-bot-agent:latest .
```

The `improve_bot` pattern materializes the checked-in baseline bot as
a real `bot` artifact fixture for each lane at pattern start. There is
no manual import step for the normal pattern.

Ad hoc training uses the base Flywheel block command from the cyberloop root:

```bash
flywheel run block --workspace foundry/workspaces/<workspace> --block Train --template cyberloop -- --subclass dueling --combat-only
```

## License

Licensed under the **PolyForm Shield License 1.0.0**. See [LICENSE](LICENSE).

Copyright (c) 2026 Heartland AI (dba Hopewell AI)
