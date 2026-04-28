"""Shared workspace + template scaffolding for cyberloop tests.

Builds a minimal cyberloop project on disk (``flywheel.yaml``,
the production template, every block YAML cyberloop ships, and
a clean :class:`Workspace`) and returns the trio that nearly
every cyberloop test needs.

Lives outside :file:`conftest.py` so callers can either grab
the ``project_setup`` pytest fixture *or* build the trio
explicitly (useful for parameterized tests that need several
independent workspaces inside a single test).

The template and per-block YAML strings here are intentionally
literal copies of the production files under
``foundry/templates/workspaces/`` and
``foundry/templates/blocks/``.  Mirroring
them keeps tests independent of the on-disk repo layout (so
the suite still runs from a clean checkout) while making the
tested shape obvious.  When a new production block lands, add
its YAML constant here and append the name to ``TEMPLATE_YAML``
to keep one source of truth for the test project's shape.
"""

from __future__ import annotations

from datetime import UTC, datetime
from pathlib import Path

from flywheel.blocks.registry import BlockRegistry
from flywheel.template import Template
from flywheel.workspace import Workspace


TEMPLATE_YAML = """\
artifacts:
  - name: game_engine
    kind: git
    repo: "."
    path: crates
  - name: checkpoint
    kind: copy
  - name: score
    kind: copy
  - name: bot
    kind: copy
  - name: decker_state
    kind: copy
  - name: cua_trace
    kind: copy
  - name: game_step
    kind: copy
  - name: gui_action_decider
    kind: copy
  - name: escalation_request
    kind: copy

blocks:
  - Train
  - Eval
  - EvalBot
  - ImproveBot
  - DeckerDesktop
  - CuaPlayAgent
  - StepThroughGui
  - GuiEscalationGate
  - ImproveGuiActionDecider
"""

DECKER_DESKTOP_BLOCK_YAML = """\
name: DeckerDesktop
runner: container
lifecycle: workspace_persistent
image: cyberloop-decker-desktop:latest
docker_args:
  - --restart=unless-stopped
  - --network=cyberloop-cua
  - --network-alias=decker-desktop
env:
  DESKTOP_API_PORT: "8080"
  DESKTOP_WIDTH: "1280"
  DESKTOP_HEIGHT: "720"
  DESKTOP_DPI: "96"
state: none
outputs:
  normal: []
"""

CUA_PLAY_AGENT_BLOCK_YAML = """\
name: CuaPlayAgent
runner: container
image: cyberloop-cua-play-agent:latest
docker_args:
  - -v
  - claude-auth:/home/claude/.claude:rw
  - --network=cyberloop-cua
env:
  MAX_TURNS: "120"
  MODEL: claude-sonnet-4-6[1m]
  COMPACT_TOKEN_LIMIT: "200000"
  MCP_SERVER_MOUNT_DIR: /app/agent/mcp_servers
  MCP_SERVERS: cyberloop_cua
  DESKTOP_URL: http://decker-desktop:8080
  CUA_SCREENSHOT_DIR: /output/cua_trace/screenshots
  HANDOFF_TOOLS: mcp__cyberloop_cua__finish_segment
  HANDOFF_TERMINATION_REASON: segment_complete
  HANDOFF_PLACEHOLDER_MARKER: Segment complete.
  HANDOFF_RESULT_LABEL: Segment
state: managed
inputs:
  - name: decker_state
    container_path: /input/decker_state
    optional: true
outputs:
  segment_complete:
    - name: decker_state
      container_path: /output/decker_state
    - name: cua_trace
      container_path: /output/cua_trace
      sequence:
        name: cua_trace
        scope: enclosing_lane
        role: segment_trace
  normal:
    - name: decker_state
      container_path: /output/decker_state
    - name: cua_trace
      container_path: /output/cua_trace
      sequence:
        name: cua_trace
        scope: enclosing_lane
        role: segment_trace
  desktop_unreachable:
    - name: decker_state
      container_path: /output/decker_state
    - name: cua_trace
      container_path: /output/cua_trace
      sequence:
        name: cua_trace
        scope: enclosing_lane
        role: segment_trace
"""

STEP_THROUGH_GUI_BLOCK_YAML = """\
name: StepThroughGui
runner: container
image: cyberloop-gui-step:latest
docker_args:
  - --network=cyberloop-cua
env:
  DESKTOP_URL: http://decker-desktop:8080
  SUBCLASS: dueling
  RACE: human
  BACKGROUND: soldier
  SEED: "1"
  POST_ACTION_WAIT_MS: "800"
  SNAP_TO_PREDICTION_ON_MISMATCH: "false"
state: none
inputs:
  - name: bot
    container_path: /input/bot
  - name: gui_action_decider
    container_path: /input/gui_action_decider
  - name: decker_state
    container_path: /input/decker_state
    optional: true
outputs:
  action_taken:
    - name: decker_state
      container_path: /output/decker_state
    - name: game_step
      container_path: /output/game_step
      sequence:
        name: decker_gui_game
        scope: enclosing_lane
        role: game_step
    - name: cua_trace
      container_path: /output/cua_trace
      sequence:
        name: gui_step_trace
        scope: enclosing_lane
        role: gui_step_trace
  game_over:
    - name: decker_state
      container_path: /output/decker_state
    - name: game_step
      container_path: /output/game_step
      sequence:
        name: decker_gui_game
        scope: enclosing_lane
        role: game_step
    - name: cua_trace
      container_path: /output/cua_trace
      sequence:
        name: gui_step_trace
        scope: enclosing_lane
        role: gui_step_trace
  desktop_unreachable:
    - name: decker_state
      container_path: /output/decker_state
    - name: game_step
      container_path: /output/game_step
      sequence:
        name: decker_gui_game
        scope: enclosing_lane
        role: game_step
    - name: cua_trace
      container_path: /output/cua_trace
      sequence:
        name: gui_step_trace
        scope: enclosing_lane
        role: gui_step_trace
  bot_error:
    - name: decker_state
      container_path: /output/decker_state
    - name: game_step
      container_path: /output/game_step
      sequence:
        name: decker_gui_game
        scope: enclosing_lane
        role: game_step
    - name: cua_trace
      container_path: /output/cua_trace
      sequence:
        name: gui_step_trace
        scope: enclosing_lane
        role: gui_step_trace
"""

GUI_ESCALATION_GATE_BLOCK_YAML = """\
name: GuiEscalationGate
runner: container
image: cyberloop-gui-escalation-gate:latest
state: none
env:
  ESCALATION_INTERVAL: "0"
inputs:
  - name: game_step_history
    container_path: /input/game_step_history
    sequence:
      name: decker_gui_game
      scope: enclosing_lane
outputs:
  escalation_requested:
    - name: escalation_request
      container_path: /output/escalation_request
      sequence:
        name: gui_escalation
        scope: enclosing_lane
        role: request
  normal: []
on_termination:
  escalation_requested:
    invoke:
      - block: ImproveGuiActionDecider
        bind:
          escalation_request: escalation_request
"""

IMPROVE_GUI_ACTION_DECIDER_BLOCK_YAML = """\
name: ImproveGuiActionDecider
runner: container
image: cyberloop-improve-gui-action-decider-agent:latest
docker_args:
  - -v
  - claude-auth:/home/claude/.claude:rw
env:
  MAX_TURNS: "200"
  MODEL: claude-sonnet-4-6[1m]
  COMPACT_TOKEN_LIMIT: "200000"
  TOOLS: Read,Write,Edit,Glob,Grep,Bash
  ALLOWED_TOOLS: Read,Write,Edit,Glob,Grep,Bash
state: none
inputs:
  - name: escalation_request
    container_path: /input/escalation_request
  - name: gui_action_decider
    container_path: /input/gui_action_decider
  - name: game_step
    container_path: /input/game_step
  - name: cua_trace
    container_path: /input/cua_trace
  - name: game_step_history
    container_path: /input/game_step_history
    sequence:
      name: decker_gui_game
      scope: enclosing_lane
  - name: gui_step_trace_history
    container_path: /input/gui_step_trace_history
    sequence:
      name: gui_step_trace
      scope: enclosing_lane
outputs:
  normal:
    - name: gui_action_decider
      container_path: /output/gui_action_decider
"""

TRAIN_BLOCK_YAML = """\
name: Train
runner: container
image: cyberloop-train:latest
docker_args:
  - --gpus
  - all
  - --shm-size
  - 8g
inputs:
  - name: checkpoint
    container_path: /input/checkpoint
    optional: true
outputs:
  normal:
    - name: checkpoint
      container_path: /output/checkpoint
"""

EVAL_BLOCK_YAML = """\
name: Eval
runner: container
image: cyberloop-eval:latest
inputs:
  - name: checkpoint
    container_path: /input/checkpoint
outputs:
  normal:
    - name: score
      container_path: /output/score
"""

EVAL_BOT_BLOCK_YAML = """\
name: EvalBot
runner: container
image: cyberloop-eval:latest
docker_args:
  - --entrypoint
  - python
inputs:
  - name: bot
    container_path: /input/bot
outputs:
  normal:
    - name: score
      container_path: /output/score
  aborted:
    - name: score
      container_path: /output/score
"""

IMPROVE_BOT_BLOCK_YAML = """\
name: ImproveBot
runner: container
image: cyberloop-improve-bot-agent:latest
docker_args:
  - -v
  - claude-auth:/home/claude/.claude:rw
env:
  MAX_TURNS: "400"
  COMPACT_TOKEN_LIMIT: "200000"
  MCP_SERVER_MOUNT_DIR: /app/agent/mcp_servers
  MCP_SERVERS: cyberloop
  HANDOFF_TOOLS: mcp__cyberloop__request_eval
  HANDOFF_TERMINATION_REASON: eval_requested
  HANDOFF_REQUIRED_PATHS: /output/bot/bot.py
  HANDOFF_RESULT_PATH: /input/score/scores.json
  HANDOFF_RESULT_LABEL: Evaluation
state: managed
inputs:
  - name: game_engine
    container_path: /source
  - name: bot
    container_path: /input/bot
  - name: score
    container_path: /input/score
    optional: true
outputs:
  eval_requested:
    - name: bot
      container_path: /output/bot
  normal:
    - name: bot
      container_path: /output/bot
on_termination:
  eval_requested:
    invoke:
      - block: EvalBot
        bind:
          bot: bot
        args:
          - /app/scripts/eval_bot.py
          - --bot
          - /input/bot/bot.py
          - --output-dir
          - /output/score
          - --subclass
          - dueling
          - --episodes
          - ${params.eval_episodes}
"""


def build_project(
        tmp_path: Path) -> tuple[Workspace, Template, Path]:
    """Create a minimal cyberloop project under ``tmp_path``.

    Returns ``(workspace, template, project_root)``: a fresh
    workspace whose template declares cyberloop's artifact
    contract (``checkpoint``, ``score``) and the ``Eval`` block,
    plus the project root directory containing
    ``flywheel.yaml``.  The on-disk shape mirrors a real
    cyberloop checkout: ``foundry/templates/workspaces/`` for
    workspace templates and ``foundry/templates/blocks/`` for
    block YAMLs.
    """
    project = tmp_path / "project"
    project.mkdir()
    (project / "flywheel.yaml").write_text(
        "foundry_dir: foundry\n"
        "artifact_validators: cyberloop.artifact_validators:build_registry\n"
    )

    templates = project / "foundry" / "templates" / "workspaces"
    templates.mkdir(parents=True)
    (templates / "cyberloop.yaml").write_text(TEMPLATE_YAML)

    blocks_dir = project / "foundry" / "templates" / "blocks"
    blocks_dir.mkdir(parents=True)
    (blocks_dir / "Train.yaml").write_text(TRAIN_BLOCK_YAML)
    (blocks_dir / "Eval.yaml").write_text(EVAL_BLOCK_YAML)
    (blocks_dir / "EvalBot.yaml").write_text(EVAL_BOT_BLOCK_YAML)
    (blocks_dir / "ImproveBot.yaml").write_text(IMPROVE_BOT_BLOCK_YAML)
    (blocks_dir / "DeckerDesktop.yaml").write_text(DECKER_DESKTOP_BLOCK_YAML)
    (blocks_dir / "CuaPlayAgent.yaml").write_text(CUA_PLAY_AGENT_BLOCK_YAML)
    (blocks_dir / "StepThroughGui.yaml").write_text(
        STEP_THROUGH_GUI_BLOCK_YAML)
    (blocks_dir / "GuiEscalationGate.yaml").write_text(
        GUI_ESCALATION_GATE_BLOCK_YAML)
    (blocks_dir / "ImproveGuiActionDecider.yaml").write_text(
        IMPROVE_GUI_ACTION_DECIDER_BLOCK_YAML)
    registry = BlockRegistry.from_directory(blocks_dir)

    workspaces = project / "foundry" / "workspaces"
    workspaces.mkdir()

    ws_path = workspaces / "test-run"
    ws_path.mkdir()
    (ws_path / "artifacts").mkdir()

    template = Template.from_yaml(
        templates / "cyberloop.yaml",
        block_registry=registry,
    )
    ws = Workspace(
        name="test-run",
        path=ws_path,
        template_name="cyberloop",
        created_at=datetime.now(UTC),
        artifact_declarations={
            a.name: a.kind for a in template.artifacts
        },
        artifacts={},
    )
    ws.save()

    return ws, template, project
