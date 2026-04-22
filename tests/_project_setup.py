"""Shared workspace + template scaffolding for cyberloop tests.

Builds a minimal cyberloop project on disk
(``flywheel.yaml``, an empty template, and a clean
:class:`Workspace`) and returns the trio that nearly every
cyberloop test needs.

Lives outside :file:`conftest.py` so callers can either grab
the ``project_setup`` pytest fixture *or* build the trio
explicitly (useful for parameterized tests that need several
independent workspaces inside a single test).

The template starts empty: cyberloop ships its own real
template once the Train and Eval blocks land; until then tests
that need declared blocks should extend the YAML in this
helper rather than reaching into ``BlockRegistry`` directly,
to keep one source of truth for the test project's shape.
"""

from __future__ import annotations

from datetime import UTC, datetime
from pathlib import Path

from flywheel.blocks.registry import BlockRegistry
from flywheel.template import Template
from flywheel.workspace import Workspace


TEMPLATE_YAML = """\
artifacts: []
blocks: []
"""


def build_project(
        tmp_path: Path) -> tuple[Workspace, Template, Path]:
    """Create a minimal cyberloop project under ``tmp_path``.

    Returns ``(workspace, template, project_root)``: a fresh
    workspace whose template declares no artifacts or blocks
    yet, plus the project root directory containing
    ``flywheel.yaml``.  The on-disk shape mirrors a real
    cyberloop checkout: ``foundry/templates/`` for templates
    and ``workforce/blocks/`` for block YAMLs (the latter
    starts empty here).
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
