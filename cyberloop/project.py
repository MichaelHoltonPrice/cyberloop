"""Cyberloop project hooks for ``flywheel run pattern``.

Pattern execution is intentionally deferred while Flywheel's pattern
surface is rebuilt on the canonical block-execution pipeline.  Ad hoc
Cyberloop training and evaluation should use the base
``flywheel run block`` command.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from flywheel.template import Template
from flywheel.workspace import Workspace


class ProjectHooks:
    """Boundary object for deferred Cyberloop pattern execution."""

    def init(
        self,
        workspace: Workspace,
        template: Template,
        project_root: Path,
        args: list[str],
    ) -> dict[str, Any]:
        """Fail explicitly because Cyberloop patterns are deferred."""
        del workspace, template, project_root, args
        raise NotImplementedError(
            "Cyberloop pattern execution is deferred while Flywheel "
            "patterns are rebuilt on the canonical block execution "
            "pipeline. Use `flywheel run block` for ad hoc Train/Eval "
            "execution."
        )

    def teardown(self) -> None:
        """No-op today."""
