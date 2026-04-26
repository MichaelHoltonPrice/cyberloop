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
from flywheel.pattern_declaration import PatternDeclaration
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
        assert eval_def.state == "none"

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
        assert kinds == {
            "game_engine": "git",
            "checkpoint": "copy",
            "score": "copy",
            "bot": "copy",
        }

    def test_eval_block_in_template(self, project_setup):
        _, template, _ = project_setup
        block_names = [b.name for b in template.blocks]
        assert block_names == ["Train", "Eval", "EvalBot", "ImproveBot"]

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

    def test_improve_bot_declares_eval_invocation(self, project_setup):
        _, template, _ = project_setup
        improve = next(b for b in template.blocks if b.name == "ImproveBot")
        assert improve.image == "cyberloop-improve-bot-agent:latest"
        assert improve.state == "managed"
        assert [slot.name for slot in improve.inputs] == [
            "game_engine", "bot", "score"]
        assert improve.inputs[0].container_path == "/source"
        assert improve.inputs[1].optional is False
        assert improve.inputs[2].optional is True
        assert [slot.name for slot in improve.outputs_for("eval_requested")] == [
            "bot"]
        assert [slot.name for slot in improve.outputs_for("done")] == ["bot"]
        invocation = improve.on_termination["eval_requested"][0]
        assert invocation.block == "EvalBot"
        assert invocation.bind["bot"].parent_output == "bot"
        assert "--episodes" in invocation.args
        assert "${params.eval_episodes}" in invocation.args

        eval_bot = next(b for b in template.blocks if b.name == "EvalBot")
        assert eval_bot.docker_args == ["--entrypoint", "python"]
        assert [slot.name for slot in eval_bot.inputs] == ["bot"]
        assert eval_bot.outputs_for("normal")[0].name == "score"

    def test_improve_bot_agent_image_bakes_prompt(self):
        dockerfile = (
            CYBERLOOP_ROOT / "docker" / "Dockerfile.improve-bot-agent")
        text = dockerfile.read_text(encoding="utf-8")

        assert "FROM flywheel-claude:latest" in text
        assert (
            "COPY foundry/templates/assets/improve_bot_prompt/prompt.md "
            "/app/agent/prompt.md"
        ) in text



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
        assert {"Train", "Eval", "EvalBot", "ImproveBot"}.issubset(
            set(registry.names()))

    def test_template_loads_and_resolves_eval(self):
        registry = BlockRegistry.from_directory(self.BLOCKS_DIR)
        template = Template.from_yaml(
            self.TEMPLATE_PATH, block_registry=registry)
        assert "Eval" in [b.name for b in template.blocks]
        assert "Train" in [b.name for b in template.blocks]
        assert "EvalBot" in [b.name for b in template.blocks]
        assert "ImproveBot" in [b.name for b in template.blocks]
        assert {a.name for a in template.artifacts} == {
            "game_engine", "checkpoint", "score", "bot"}

    def test_improve_bot_pattern_declares_lanes_and_fixture(self):
        pattern = PatternDeclaration.from_yaml(
            CYBERLOOP_ROOT
            / "foundry"
            / "templates"
            / "patterns"
            / "improve_bot.yaml"
        )

        assert pattern.lanes == ["A", "B", "C"]
        assert pattern.params["model"].default == "claude-sonnet-4-6"
        assert pattern.params["eval_episodes"].default == 4000
        assert pattern.fixtures["bot"].source == (
            "foundry/templates/assets/bots/baseline")
        assert [step.name for step in pattern.steps] == [
            "improve_1",
            "improve_2",
        ]
        assert [
            (member.name, member.lane, member.block)
            for member in pattern.steps[0].cohort.members
        ] == [
            ("A", "A", "ImproveBot"),
            ("B", "B", "ImproveBot"),
            ("C", "C", "ImproveBot"),
        ]
        assert [
            member.env["MODEL"]
            for member in pattern.steps[0].cohort.members
        ] == [
            "${params.model}",
            "${params.model}",
            "${params.model}",
        ]
