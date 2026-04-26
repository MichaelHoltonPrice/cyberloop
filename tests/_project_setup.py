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

blocks:
  - Train
  - Eval
  - EvalBot
  - ImproveBot
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
  normal:
    - name: bot
      container_path: /output/bot
on_termination:
  normal:
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
