"""Tests for :class:`cyberloop.project.ProjectHooks`.

Cyberloop pattern execution is intentionally deferred while Flywheel's
pattern surface is rebuilt.  The hook remains importable because
``flywheel.yaml`` points at it, but invoking it should fail loudly
instead of routing through deprecated executor machinery.
"""

from __future__ import annotations

import pytest

from cyberloop.project import ProjectHooks


class TestInit:
    """``init`` marks Cyberloop pattern execution as deferred."""

    def test_init_raises_clear_deferred_error(self, project_setup):
        ws, template, project = project_setup
        hooks = ProjectHooks()

        with pytest.raises(
            NotImplementedError,
            match="Cyberloop pattern execution is deferred",
        ):
            hooks.init(ws, template, project, [])

    def test_init_raises_even_with_extra_args(self, project_setup):
        ws, template, project = project_setup
        hooks = ProjectHooks()

        with pytest.raises(NotImplementedError):
            hooks.init(ws, template, project, ["--unknown", "value"])


class TestTeardown:
    """``teardown`` is a no-op today but must be safe to call."""

    def test_teardown_after_failed_init_is_noop(self, project_setup):
        ws, template, project = project_setup
        hooks = ProjectHooks()
        with pytest.raises(NotImplementedError):
            hooks.init(ws, template, project, [])
        hooks.teardown()

    def test_teardown_without_init_is_noop(self):
        ProjectHooks().teardown()
