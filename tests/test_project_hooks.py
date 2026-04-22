"""Tests for :class:`cyberloop.project.ProjectHooks`.

The ProjectHooks class is consumed by ``flywheel run pattern``
and is the only project-side hooks class cyberloop ships.
This file locks down its CLI-parse + override-shape contract.

The ``project_setup`` fixture is provided by ``conftest.py``.
"""

from __future__ import annotations

from flywheel.blocks.registry import BlockDefinition
from flywheel.executor import ProcessExitExecutor

from cyberloop.project import ProjectHooks


def _init_with_args(
    project_setup, *, args: list[str],
) -> tuple[ProjectHooks, dict]:
    """Run ``ProjectHooks.init`` and return ``(hooks, overrides)``."""
    ws, template, project = project_setup
    hooks = ProjectHooks()
    overrides = hooks.init(ws, template, project, args)
    return hooks, overrides


class TestInit:
    """``init`` accepts an empty arg list and stores its inputs."""

    def test_init_accepts_empty_args(self, project_setup):
        # Cyberloop has no project-level CLI flags today; the
        # protocol still passes ``args`` through, and ``init``
        # must not raise on the empty list.
        hooks, overrides = _init_with_args(project_setup, args=[])
        assert hooks._executor is not None
        assert isinstance(overrides, dict)

    def test_init_ignores_unknown_args(self, project_setup):
        # The hooks deliberately do not parse ``args`` yet, so
        # passing extra strings on the ``-- --foo`` tail must
        # be a no-op rather than an error.  When real flags
        # land this assertion will be replaced by a parse test.
        hooks, _ = _init_with_args(
            project_setup, args=["--unknown", "value"])
        assert hooks._executor is not None

    def test_init_records_workspace_and_template(self, project_setup):
        ws, template, _ = project_setup
        hooks, _ = _init_with_args(project_setup, args=[])
        # Held so a future ``teardown`` (and tests that need
        # to reach project-side state) can access them.
        assert hooks._workspace is ws
        assert hooks._template is template


class TestOverrideShape:
    """Pin the override-dict shape returned to the runner.

    The dict returned to the runner has only the keys the
    pattern runner consumes.  Adding more is fine, but each
    addition deserves its own pinned test alongside this one.
    """

    def test_returns_executor_factory(self, project_setup):
        _, overrides = _init_with_args(project_setup, args=[])
        assert callable(overrides["executor_factory"])

    def test_does_not_return_legacy_battery_keys(
            self, project_setup):
        # The CLI's legacy fallback path (which builds an
        # AgentExecutor from battery-shaped overrides like
        # ``model``, ``mcp_servers``, ``extra_env``) only
        # fires when the hooks did *not* supply
        # ``executor_factory``.  Cyberloop's hooks must stay
        # on the explicit-factory path, so none of those
        # legacy keys may leak through.
        _, overrides = _init_with_args(project_setup, args=[])
        legacy_keys = {
            "agent_image", "model", "mcp_servers",
            "extra_env", "extra_mounts",
            "prompt_substitutions",
        }
        assert not (legacy_keys & overrides.keys())


class TestExecutorFactory:
    """Pin the per-block executor selection.

    The factory must return the project's
    :class:`ProcessExitExecutor` for every block the pattern
    runner asks about.  Today cyberloop has no
    workspace-persistent or agent blocks, so the factory
    deliberately ignores the block definition.  When a
    workspace-persistent block is added, new tests here should
    pin the branch.
    """

    def test_returns_process_exit_executor(self, project_setup):
        hooks, overrides = _init_with_args(project_setup, args=[])
        factory = overrides["executor_factory"]
        block_def = BlockDefinition(name="train_segment")
        executor = factory(block_def)
        assert executor is hooks._executor
        assert isinstance(executor, ProcessExitExecutor)

    def test_returns_same_executor_for_every_block(
            self, project_setup):
        # One executor per run keeps the in-process state
        # (CLI defaults, allowed-blocks bookkeeping) shared
        # across every dispatch.  Constructing a fresh
        # executor per block_def would be a regression.
        hooks, overrides = _init_with_args(project_setup, args=[])
        factory = overrides["executor_factory"]
        e1 = factory(BlockDefinition(name="train_segment"))
        e2 = factory(BlockDefinition(name="eval"))
        assert e1 is e2 is hooks._executor


class TestTeardown:
    """``teardown`` is a no-op today but must be safe to call.

    Pinning this catches accidental side effects from a future
    teardown that forgets to gate on ``init`` having run.
    """

    def test_teardown_after_init_is_noop(self, project_setup):
        hooks, _ = _init_with_args(project_setup, args=[])
        hooks.teardown()  # Must not raise.

    def test_teardown_without_init_is_noop(self):
        ProjectHooks().teardown()  # Must not raise.
