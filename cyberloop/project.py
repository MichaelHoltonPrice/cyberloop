"""Cyberloop project hooks for ``flywheel run pattern``.

Implements :class:`flywheel.project_hooks.ProjectHooks`.

Cyberloop drives reinforcement-learning training of a model
that plays the Decker gauntlet, with periodic evaluation
between training segments.  Both training and evaluation run
as one-shot containers launched by the substrate's
:class:`flywheel.executor.ProcessExitExecutor`; there is no
agent battery on the cyberloop critical path.

``init()`` constructs a single
:class:`~flywheel.executor.ProcessExitExecutor` and returns an
``executor_factory`` that hands that executor to every block
the pattern runner asks about.  Project-specific CLI args
(``--subclass``, ``--episodes``, training hyperparameters,
...) are deferred to the blocks themselves: the patterns thread
them through ``overrides`` (CLI-flag substitutions) so the YAML
declares the wiring and the project hooks stay agnostic.

This is the only project-side hooks class cyberloop ships.
"""

from __future__ import annotations

from pathlib import Path
from typing import Any

from flywheel.executor import ProcessExitExecutor
from flywheel.template import Template
from flywheel.workspace import Workspace


class ProjectHooks:
    """Resource lifecycle for cyberloop patterns.

    Stateful across ``init`` / ``teardown`` so a future
    teardown can release resources (none today; process exit
    cleans things up).
    """

    def __init__(self) -> None:
        self._workspace: Workspace | None = None
        self._template: Template | None = None
        self._executor: ProcessExitExecutor | None = None

    def init(
        self,
        workspace: Workspace,
        template: Template,
        project_root: Path,
        args: list[str],
    ) -> dict[str, Any]:
        """Construct the executor and return launcher overrides.

        Cyberloop has no project-level CLI flags today; ``args``
        is accepted for protocol compatibility and intentionally
        ignored.  When per-run knobs need to land outside the
        pattern YAML (for example a one-off training-time cap),
        parse them here and surface them via ``defaults`` or
        ``per_instance_runtime_config`` rather than mutating
        process-global state.
        """
        del project_root, args  # Reserved by the protocol.
        self._workspace = workspace
        self._template = template

        executor = ProcessExitExecutor(template)
        self._executor = executor

        def executor_factory(block_def):
            """Return the cyberloop executor for ``block_def``.

            Every cyberloop block today is a one-shot
            container, so a single
            :class:`ProcessExitExecutor` services every
            dispatch.  When a block ever needs the
            request-response or agent shape, branch on
            ``block_def.lifecycle`` / ``block_def.runner``
            here.
            """
            del block_def  # Single-executor policy today.
            return executor

        return {
            "executor_factory": executor_factory,
        }

    def teardown(self) -> None:
        """No-op today.

        Kept for symmetry with the protocol; the runner's
        attribute check skips the call when the method is
        absent, but having it documented makes the intentional
        no-op explicit.
        """
