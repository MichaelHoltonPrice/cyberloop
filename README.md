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

From the cyberloop repo root, create and activate a virtual environment:

Windows cmd:

```bat
python -m venv .venv
.venv\Scripts\activate.bat
```

PowerShell:

```powershell
python -m venv .venv
.venv\Scripts\Activate.ps1
```

Linux/macOS:

```bash
python -m venv .venv
source .venv/bin/activate
```

Then install the Python dependencies and Flywheel editable package:

```bash
# Build and test the engine
cargo test

# Play interactively (requires Bevy dependencies)
cargo run -p decker-client

python -m pip install -r requirements.txt
cd crates/pyenv
python -m maturin develop --release
cd ../..

# Flywheel itself is installed editable from the sibling repo:
python -m pip install -e ../flywheel
python -m pip install -e ".[dev]"
```

## Orchestration

Train and Eval can be run ad hoc through Flywheel's canonical one-shot
container pipeline, and `train_eval` runs the same blocks as a pattern.
Cyberloop also includes an `improve_bot` pattern: a Claude battery block edits
`bot.py`, requests evaluation through a battery-provided tool, commits the
candidate as a bot artifact, and invokes `EvalBot`.
The currently supported surface:

- `foundry/templates/blocks/Train.yaml` runs `train_impala.py` and writes a
  flat `checkpoint` artifact with `checkpoint.pt` and `run.json`.
- `foundry/templates/blocks/Eval.yaml` runs `eval_checkpoint.py` against a
  pre-staged checkpoint artifact and writes a `score` artifact.
- `foundry/templates/blocks/EvalBot.yaml` runs `eval_bot.py` against a
  Python `bot.py` artifact and writes a `score` artifact.
- `foundry/templates/blocks/ImproveBot.yaml` runs a Cyberloop image
  derived from Flywheel's Claude battery and routes the `request_eval`
  tool to `EvalBot`. It mounts the checked-in Decker engine source as
  the `game_engine` git artifact at `/source`.
- `foundry/templates/blocks/DeckerDesktop.yaml` runs the Bevy Decker GUI
  as a workspace-persistent desktop service derived from Flywheel's
  desktop battery. It exposes screenshot/input APIs on the
  `cyberloop-cua` Docker network.
- `foundry/templates/blocks/CuaPlayAgent.yaml` runs a Claude controller
  that drives `DeckerDesktop` through computer-use tools and writes
  `decker_state` and `cua_trace` artifacts at segment exit.
- `foundry/templates/workspaces/cyberloop.yaml` declares the
  `game_engine`, `checkpoint`, `score`, `bot`, `decker_state`, and
  `cua_trace` artifact contract.
- `foundry/templates/patterns/train_eval.yaml` declares the canonical
  Train to Eval pattern.
- `foundry/templates/patterns/improve_bot.yaml` declares the agent-improves-bot
  pattern: three run-scoped lanes, each seeded from the checked-in
  baseline bot fixture before running a lane-local ImproveBot loop.
- `foundry/templates/patterns/improve_bot_sonnet_1lane.yaml` declares the
  narrow Sonnet ImproveBot pattern: one run-scoped lane with a five-evaluation
  budget.
- `foundry/templates/patterns/improve_bot_sonnet_2lane.yaml` declares the
  laptop-friendly Sonnet ImproveBot pattern: two run-scoped lanes, each
  seeded from the same baseline bot fixture and run to completion before
  the next lane starts.
- `cyberloop.artifact_validators` validates checkpoint and score
  artifacts through Flywheel's `artifact_validators` hook.

### Container images

Build the Claude battery image, the desktop battery image, the Cyberloop
eval image, then the Cyberloop agent/service images:

```bash
docker build -f ../flywheel/batteries/claude/Dockerfile.claude -t flywheel-claude:latest ../flywheel/batteries/claude
docker build -f ../flywheel/batteries/desktop/Dockerfile.desktop -t flywheel-desktop:latest ../flywheel/batteries/desktop
docker build -f docker/Dockerfile.eval -t cyberloop-eval:latest .
docker build -f docker/Dockerfile.improve-bot-agent -t cyberloop-improve-bot-agent:latest .
docker build -f docker/Dockerfile.decker-desktop -t cyberloop-decker-desktop:latest .
docker build -f docker/Dockerfile.cua-play-agent -t cyberloop-cua-play-agent:latest .
```

### Claude auth volume

The ImproveBot block uses the `claude-auth` Docker volume. After
logging in with Claude Code on the host (`/login` inside Claude Code),
refresh that volume from the cyberloop root.

Windows cmd:

```bat
docker volume create claude-auth
docker run --rm -v claude-auth:/auth -v "%USERPROFILE%\.claude:/host-claude:ro" python:3.12-slim sh -c "cp -a /host-claude/. /auth/ && chown -R 1000:1000 /auth"
```

PowerShell:

```powershell
docker volume create claude-auth
docker run --rm -v claude-auth:/auth -v "$env:USERPROFILE\.claude:/host-claude:ro" python:3.12-slim sh -c "cp -a /host-claude/. /auth/ && chown -R 1000:1000 /auth"
```

Linux/macOS:

```bash
docker volume create claude-auth
docker run --rm -v claude-auth:/auth -v "$HOME/.claude:/host-claude:ro" python:3.12-slim sh -c "cp -a /host-claude/. /auth/ && chown -R 1000:1000 /auth"
```

The Claude battery entrypoint scrubs the volume at container start so the
agent sees only the credentials and synthesized settings it needs, not host
conversation history.

### Computer-use desktop network

The Decker desktop service and CUA controller communicate over a project
Docker network. Create it once before running the computer-use pattern:

```bash
docker network create cyberloop-cua
```

If the network already exists, Docker will report that and no further action is
needed. `DeckerDesktop` joins this network with the alias
`decker-desktop`; `CuaPlayAgent` uses `DESKTOP_URL=http://decker-desktop:8080`.
The desktop API is project convention, not a Flywheel core service concept.

### Workspaces and pattern runs

Create a workspace from the cyberloop root. The workspace template includes
a `game_engine` git artifact, so create workspaces from a clean git tree:

```bash
python -m flywheel create workspace --name improve-bot-sonnet-2lane --template cyberloop
```

The ImproveBot patterns materialize the checked-in baseline bot as a real
`bot` artifact fixture for each lane at pattern start. There is no manual
artifact import step for these patterns.

The pattern stores its resolved parameters on the Flywheel run record.
ImproveBot runs use `COMPACT_TOKEN_LIMIT=200000`, so Claude compacts
before context is too large to compact reliably. Sonnet patterns default
to the 1M-context model name (`claude-sonnet-4-6[1m]`).

The one-lane Sonnet pattern is the smallest full ImproveBot run. It runs one
lane as a bounded loop with the five-evaluation budget:

```bash
python -m flywheel create workspace --name improve-bot-sonnet-1lane --template cyberloop
python -m flywheel run pattern improve_bot_sonnet_1lane --workspace foundry/workspaces/improve-bot-sonnet-1lane --template cyberloop
```

The laptop-friendly two-lane Sonnet pattern runs each lane as a
bounded loop. In one lane, ImproveBot keeps resuming the same managed
Claude session until the agent exits normally or has used five
`request_eval` calls. Each `request_eval` commits the candidate bot,
invokes EvalBot, and returns the EvalBot result to the resumed Claude
session via a resume prompt; the score artifact is also mounted at
`/input/score/scores.json` for the next ImproveBot iteration. The
`request_eval` tool result in session history remains an honest
acknowledgement that evaluation was requested.

Run it with:

```bash
python -m flywheel run pattern improve_bot_sonnet_2lane --workspace foundry/workspaces/improve-bot-sonnet-2lane --template cyberloop
```

The full three-lane ImproveBot pattern uses the same five-evaluation
budget per lane, default 1M-context Sonnet model, and 4000-episode
EvalBot runs:

```bash
python -m flywheel run pattern improve_bot --workspace foundry/workspaces/<workspace> --template cyberloop
```

To run the three-lane pattern with Opus instead, override the model at run
start. Opus 4.7 uses the 1M context window by default; the same
200K-token compaction limit still applies through the ImproveBot block
environment. The resolved model is stored on the run record:

```bash
python -m flywheel run pattern improve_bot --workspace foundry/workspaces/<workspace> --template cyberloop --param model=claude-opus-4-7
```

The one-lane computer-use pattern drives the Decker GUI through the
workspace-persistent desktop service. Warm the desktop service first; this
records a no-op `DeckerDesktop` execution and leaves the persistent container
running:

```bash
python -m flywheel create workspace --name cua-play-sonnet-1lane --template cyberloop
python -m flywheel run block DeckerDesktop --workspace foundry/workspaces/cua-play-sonnet-1lane --template cyberloop
python -m flywheel run pattern cua_play_sonnet_1lane --workspace foundry/workspaces/cua-play-sonnet-1lane --template cyberloop
```

Each `CuaPlayAgent` segment uploads the previous `decker_state` artifact to
the desktop service if one exists, resets the GUI from that save, drives the
desktop, then exports `decker_state` and `cua_trace` artifacts before commit.
The `cua_trace` artifact is also appended to a lane-scoped `cua_trace`
sequence, so later blocks can consume the ordered segment history. Screenshots
captured by the agent live under `cua_trace/screenshots/` inside each segment
trace artifact. The desktop framebuffer and process are non-durable service
state; durable state lives in Flywheel artifacts.

Run `CuaPlayAgent` through the pattern, not as an ad hoc block. Its
`cua_trace` output appends to an enclosing lane sequence, so ad hoc execution
does not have enough run/lane context. If the controller cannot reach the
desktop, the block records `desktop_unreachable`, commits diagnostic
`cua_trace` and `decker_state` artifacts, and the pattern fails intentionally.

The warm-up command records a normal no-op `DeckerDesktop` execution each time
it starts or health-checks the persistent container. That ledger row is the
current cost of using the ordinary `flywheel run block` surface instead of a
separate service-start command.

`train_eval` is the currently supported RL checkpoint pattern:

```bash
python -m flywheel run pattern train_eval --workspace foundry/workspaces/<workspace> --template cyberloop
```

Ad hoc training uses the base Flywheel block command from the cyberloop root:

```bash
python -m flywheel run block --workspace foundry/workspaces/<workspace> --block Train --template cyberloop -- --subclass dueling --combat-only
```

## License

Licensed under the **PolyForm Shield License 1.0.0**. See [LICENSE](LICENSE).

Copyright (c) 2026 Heartland AI (dba Hopewell AI)
