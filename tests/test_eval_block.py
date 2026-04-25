"""Tests for the ``Eval`` block and its wiring.

Three concentric rings of coverage:

1.  *Mirror-shape* tests use the ``project_setup`` fixture's
    in-tmp YAML strings to lock the parsed-block / parsed-template
    contract the rest of the test suite depends on.

2.  *Production-file* tests load the actual on-disk
    ``foundry/templates/blocks/Eval.yaml`` and
    ``foundry/templates/workspaces/cyberloop.yaml`` so that drift
    between the test mirror strings and the production files is
    caught loudly.

The ``project_setup`` fixture is provided by ``conftest.py``.
"""

from __future__ import annotations

from pathlib import Path

from flywheel.blocks.registry import BlockRegistry
from flywheel.template import Template

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
            project / "foundry" / "templates" / "blocks")
        assert "Eval" in registry

    def test_runner_image_lifecycle(self, project_setup):
        _, _, project = project_setup
        registry = BlockRegistry.from_directory(
            project / "foundry" / "templates" / "blocks")
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
            project / "foundry" / "templates" / "blocks")
        eval_def = registry.get("Eval")

        assert len(eval_def.inputs) == 1
        checkpoint_in = eval_def.inputs[0]
        assert checkpoint_in.name == "checkpoint"
        assert checkpoint_in.container_path == "/input/checkpoint"

        normal_outputs = eval_def.outputs_for("normal")
        assert len(normal_outputs) == 1
        score_out = normal_outputs[0]
        assert score_out.name == "score"
        assert score_out.container_path == "/output/score"

    def test_no_post_check(self, project_setup):
        # The current one-shot container path does not invoke ``post_check``.
        # Declaring one on Eval would silently no-op, which would
        # be confusing; assert the absence so a future addition has
        # to update this test deliberately.
        _, _, project = project_setup
        registry = BlockRegistry.from_directory(
            project / "foundry" / "templates" / "blocks")
        eval_def = registry.get("Eval")
        assert eval_def.post_check is None


class TestCyberloopTemplate:
    """The cyberloop template wires Eval and the shared artifacts.

    Pins the ``checkpoint`` / ``score`` artifact contract and
    that Train/Eval are the declared one-shot blocks.
    """

    def test_artifact_contract(self, project_setup):
        _, template, _ = project_setup
        kinds = {a.name: a.kind for a in template.artifacts}
        assert kinds == {"checkpoint": "copy", "score": "copy"}

    def test_eval_block_in_template(self, project_setup):
        _, template, _ = project_setup
        block_names = [b.name for b in template.blocks]
        assert block_names == ["Train", "Eval"]

    def test_train_block_in_template(self, project_setup):
        _, template, _ = project_setup
        train = next(b for b in template.blocks if b.name == "Train")
        assert train.runner == "container"
        assert train.lifecycle == "one_shot"
        assert train.image == "cyberloop-train:latest"
        assert "--gpus" in train.docker_args

        assert len(train.inputs) == 1
        checkpoint_in = train.inputs[0]
        assert checkpoint_in.name == "checkpoint"
        assert checkpoint_in.optional is True
        assert checkpoint_in.container_path == "/input/checkpoint"

        normal_outputs = train.outputs_for("normal")
        assert len(normal_outputs) == 1
        checkpoint_out = normal_outputs[0]
        assert checkpoint_out.name == "checkpoint"
        assert checkpoint_out.container_path == "/output/checkpoint"



class TestProductionFilesParseAgainstRegistry:
    """Production YAML files load cleanly via substrate parsers.

    Loads the real on-disk block and workspace-template files
    (not the test-fixture mirror strings) so drift between the
    two is caught loudly.
    """

    BLOCKS_DIR = CYBERLOOP_ROOT / "foundry" / "templates" / "blocks"
    TEMPLATE_PATH = (
        CYBERLOOP_ROOT / "foundry" / "templates" / "workspaces"
        / "cyberloop.yaml")

    def test_block_templates_directory_loads(self):
        registry = BlockRegistry.from_directory(self.BLOCKS_DIR)
        assert {"Train", "Eval"}.issubset(set(registry.names()))

    def test_template_loads_and_resolves_eval(self):
        registry = BlockRegistry.from_directory(self.BLOCKS_DIR)
        template = Template.from_yaml(
            self.TEMPLATE_PATH, block_registry=registry)
        assert "Eval" in [b.name for b in template.blocks]
        assert "Train" in [b.name for b in template.blocks]
        assert {a.name for a in template.artifacts} == {
            "checkpoint", "score"}
