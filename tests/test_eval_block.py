"""Tests for the ``Eval`` block and its wiring.

Covers the ``eval_only`` pattern and the per-block executor
selection through :class:`cyberloop.project.ProjectHooks`.

Three concentric rings of coverage:

1.  *Mirror-shape* tests use the ``project_setup`` fixture's
    in-tmp YAML strings to lock the parsed-block / parsed-template
    contract the rest of the test suite depends on.

2.  *Wiring* tests run the real ``ProjectHooks`` against a real
    ``BlockDefinition`` from the fixture's template and assert
    the block lands on :class:`ProcessExitExecutor` (one-shot
    container path).  No Docker is touched.

3.  *Production-file* tests load the actual on-disk
    ``workforce/blocks/Eval.yaml``,
    ``foundry/templates/cyberloop.yaml``, and
    ``patterns/eval_only.yaml`` so that drift between the test
    mirror strings and the production files is caught loudly.

The ``project_setup`` fixture is provided by ``conftest.py``.
"""

from __future__ import annotations

from pathlib import Path

from flywheel.blocks.registry import BlockRegistry
from flywheel.executor import ProcessExitExecutor
from flywheel.pattern import ContinuousTrigger, Pattern
from flywheel.template import Template

from cyberloop.project import ProjectHooks


CYBERLOOP_ROOT = Path(__file__).resolve().parent.parent


class TestEvalBlockYaml:
    """``Eval.yaml`` parses into the expected shape.

    Asserts the fields the runner cares about
    (runner, image, lifecycle, I/O slots) on the
    :class:`BlockDefinition` the fixture's registry produces.
    """

    def test_eval_is_registered(self, project_setup):
        _, _, project = project_setup
        registry = BlockRegistry.from_directory(
            project / "workforce" / "blocks")
        assert "Eval" in registry

    def test_runner_image_lifecycle(self, project_setup):
        _, _, project = project_setup
        registry = BlockRegistry.from_directory(
            project / "workforce" / "blocks")
        eval_def = registry.get("Eval")
        assert eval_def.runner == "container"
        assert eval_def.image == "cyberloop-eval:latest"
        # ``one_shot`` is the default but we pin it: a flip to
        # ``workspace_persistent`` would silently change the
        # executor-selection branch in production hooks.
        assert eval_def.lifecycle == "one_shot"
        assert eval_def.state is False

    def test_input_and_output_slots(self, project_setup):
        _, _, project = project_setup
        registry = BlockRegistry.from_directory(
            project / "workforce" / "blocks")
        eval_def = registry.get("Eval")

        assert len(eval_def.inputs) == 1
        checkpoint_in = eval_def.inputs[0]
        assert checkpoint_in.name == "checkpoint"
        assert checkpoint_in.container_path == "/input/checkpoint"

        assert len(eval_def.outputs) == 1
        score_out = eval_def.outputs[0]
        assert score_out.name == "score"
        assert score_out.container_path == "/output/score"

    def test_no_post_check(self, project_setup):
        # ``ProcessExitExecutor`` does not invoke ``post_check``
        # for one-shot blocks today (only RequestResponseExecutor
        # does).  Declaring one on Eval would silently no-op,
        # which would be confusing — assert the absence so a
        # future addition has to update this test deliberately.
        _, _, project = project_setup
        registry = BlockRegistry.from_directory(
            project / "workforce" / "blocks")
        eval_def = registry.get("Eval")
        assert eval_def.post_check is None


class TestCyberloopTemplate:
    """The cyberloop template wires Eval and the shared artifacts.

    Pins the ``checkpoint`` / ``score`` artifact contract and
    that Eval is the (only) block declared so far.
    """

    def test_artifact_contract(self, project_setup):
        _, template, _ = project_setup
        kinds = {a.name: a.kind for a in template.artifacts}
        assert kinds == {"checkpoint": "copy", "score": "copy"}

    def test_eval_block_in_template(self, project_setup):
        _, template, _ = project_setup
        block_names = [b.name for b in template.blocks]
        assert block_names == ["Eval"]


class TestExecutorFactoryDispatchesEval:
    """``executor_factory`` routes Eval to ProcessExitExecutor.

    Mirrors the production hot path: the runner asks the project
    for an executor for a block from its own template, and gets
    back the one-shot container executor.  No Docker invocation.
    """

    def test_eval_block_dispatches_via_process_exit(
            self, project_setup):
        ws, template, project = project_setup
        eval_def = next(
            b for b in template.blocks if b.name == "Eval")

        hooks = ProjectHooks()
        overrides = hooks.init(ws, template, project, [])
        executor = overrides["executor_factory"](eval_def)

        assert isinstance(executor, ProcessExitExecutor)
        assert executor is hooks._executor


class TestEvalOnlyPattern:
    """``eval_only`` parses into the one-shot eval shape.

    Pins the production pattern's instance count, trigger,
    inputs, and overrides — the contract the runner consumes.
    """

    PATTERN_PATH = (
        CYBERLOOP_ROOT / "patterns" / "eval_only.yaml")

    def _load(self) -> Pattern:
        return Pattern.from_yaml(self.PATTERN_PATH)

    def test_pattern_name_from_filename(self):
        pattern = self._load()
        assert pattern.name == "eval_only"

    def test_single_eval_instance(self):
        instances = self._load().iter_instances()
        assert len(instances) == 1
        eval_instance = instances[0]
        assert eval_instance.name == "eval"
        assert eval_instance.block == "Eval"
        assert eval_instance.cardinality == 1

    def test_trigger_is_continuous(self):
        # ``continuous`` + ``cardinality: 1`` is the “fire once
        # at run start, end the run when the container exits”
        # shape.  ``autorestart`` would loop forever — wrong
        # for an eval pass.
        eval_instance = self._load().iter_instances()[0]
        assert isinstance(
            eval_instance.trigger, ContinuousTrigger)

    def test_inputs_match_block_slot(self):
        # The pattern only consumes ``checkpoint``; ``score``
        # is the block's output slot, not a pattern input.
        eval_instance = self._load().iter_instances()[0]
        assert eval_instance.inputs == ["checkpoint"]

    def test_overrides_carry_eval_knobs(self):
        # These keys turn into ``--subclass dueling
        # --episodes 4000 --backend numpy`` argv via
        # ``ProcessExitExecutor``'s
        # ``--{key.replace('_', '-')} {value}`` convention.  The
        # container's ENTRYPOINT pre-fills ``--checkpoint`` and
        # ``--output-dir``, so those don't appear here.
        eval_instance = self._load().iter_instances()[0]
        assert eval_instance.overrides == {
            "subclass": "dueling",
            "episodes": 4000,
            "backend": "numpy",
        }

    def test_no_extra_env(self):
        # Nothing per-instance for now.  If env knobs land
        # later (e.g., per-pattern OMP_NUM_THREADS overrides),
        # they go through ``extra_env`` on the explicit
        # container-extras seam, not through ``overrides``.
        eval_instance = self._load().iter_instances()[0]
        assert eval_instance.extra_env == {}


class TestProductionFilesParseAgainstRegistry:
    """Production YAML files load cleanly via substrate parsers.

    Loads the real on-disk block / template / pattern files (not
    the test-fixture mirror strings) so drift between the two is
    caught loudly.
    """

    BLOCKS_DIR = CYBERLOOP_ROOT / "workforce" / "blocks"
    TEMPLATE_PATH = (
        CYBERLOOP_ROOT / "foundry" / "templates"
        / "cyberloop.yaml")
    PATTERN_PATH = (
        CYBERLOOP_ROOT / "patterns" / "eval_only.yaml")

    def test_workforce_blocks_directory_loads(self):
        registry = BlockRegistry.from_directory(self.BLOCKS_DIR)
        assert "Eval" in registry

    def test_template_loads_and_resolves_eval(self):
        registry = BlockRegistry.from_directory(self.BLOCKS_DIR)
        template = Template.from_yaml(
            self.TEMPLATE_PATH, block_registry=registry)
        assert "Eval" in [b.name for b in template.blocks]
        assert {a.name for a in template.artifacts} == {
            "checkpoint", "score"}

    def test_pattern_loads_with_registry(self):
        # Passing the registry exercises the runner's
        # production load path (any ``every_n_executions``
        # trigger would be validated against it).  The
        # eval-only pattern has no such trigger today, but
        # passing the registry still pins the call shape.
        registry = BlockRegistry.from_directory(self.BLOCKS_DIR)
        pattern = Pattern.from_yaml(
            self.PATTERN_PATH, block_registry=registry)
        assert pattern.name == "eval_only"
        assert len(pattern.iter_instances()) == 1
