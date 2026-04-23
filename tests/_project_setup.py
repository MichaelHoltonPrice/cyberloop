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
``foundry/templates/`` and ``workforce/blocks/``.  Mirroring
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
  - name: checkpoint
    kind: copy
  - name: score
    kind: copy

blocks:
  - Train
  - Eval
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


def build_project(
        tmp_path: Path) -> tuple[Workspace, Template, Path]:
    """Create a minimal cyberloop project under ``tmp_path``.

    Returns ``(workspace, template, project_root)``: a fresh
    workspace whose template declares cyberloop's artifact
    contract (``checkpoint``, ``score``) and the ``Eval`` block,
    plus the project root directory containing
    ``flywheel.yaml``.  The on-disk shape mirrors a real
    cyberloop checkout: ``foundry/templates/`` for templates
    and ``workforce/blocks/`` for block YAMLs.
    """
    project = tmp_path / "project"
    project.mkdir()
    (project / "flywheel.yaml").write_text(
        "foundry_dir: foundry\n"
        "project_hooks: cyberloop.project:ProjectHooks\n"
    )

    templates = project / "foundry" / "templates"
    templates.mkdir(parents=True)
    (templates / "cyberloop.yaml").write_text(TEMPLATE_YAML)

    blocks_dir = project / "workforce" / "blocks"
    blocks_dir.mkdir(parents=True)
    (blocks_dir / "Train.yaml").write_text(TRAIN_BLOCK_YAML)
    (blocks_dir / "Eval.yaml").write_text(EVAL_BLOCK_YAML)
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
