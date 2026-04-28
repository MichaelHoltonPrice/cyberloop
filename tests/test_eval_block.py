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

import json
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
            "decker_state": "copy",
            "cua_trace": "copy",
            "game_step": "copy",
            "gui_action_decider": "copy",
            "escalation_request": "copy",
        }

    def test_eval_block_in_template(self, project_setup):
        _, template, _ = project_setup
        block_names = [b.name for b in template.blocks]
        assert block_names == [
            "Train",
            "Eval",
            "EvalBot",
            "ImproveBot",
            "DeckerDesktop",
            "CuaPlayAgent",
            "StepThroughGui",
            "GuiEscalationGate",
            "ImproveGuiActionDecider",
        ]

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
        assert [slot.name for slot in improve.outputs_for("normal")] == ["bot"]
        assert [slot.name for slot in improve.outputs_for(
            "eval_requested")] == ["bot"]
        assert improve.outputs_for("done") == []
        assert improve.env["MCP_SERVERS"] == "cyberloop"
        assert (
            improve.env["HANDOFF_TOOLS"]
            == "mcp__cyberloop__request_eval"
        )
        assert improve.env["HANDOFF_TERMINATION_REASON"] == "eval_requested"
        assert improve.env["HANDOFF_REQUIRED_PATHS"] == "/output/bot/bot.py"
        invocation = improve.on_termination["eval_requested"][0]
        assert invocation.block == "EvalBot"
        assert invocation.bind["bot"].parent_output == "bot"
        assert "--episodes" in invocation.args
        assert "${params.eval_episodes}" in invocation.args

        eval_bot = next(b for b in template.blocks if b.name == "EvalBot")
        assert eval_bot.docker_args == ["--entrypoint", "python"]
        assert [slot.name for slot in eval_bot.inputs] == ["bot"]
        assert eval_bot.outputs_for("normal")[0].name == "score"
        assert eval_bot.outputs_for("aborted")[0].name == "score"

    def test_improve_bot_agent_image_bakes_prompt(self):
        dockerfile = (
            CYBERLOOP_ROOT / "docker" / "Dockerfile.improve-bot-agent")
        text = dockerfile.read_text(encoding="utf-8")

        assert "FROM flywheel-claude:latest" in text
        assert (
            "COPY foundry/templates/assets/improve_bot_prompt/prompt.md "
            "/app/agent/prompt.md"
        ) in text
        assert (
            "COPY foundry/templates/assets/improve_bot_mcp_servers "
            "/app/agent/mcp_servers"
        ) in text

    def test_improve_bot_request_eval_mcp_manifest_matches_block(self):
        server_dir = (
            CYBERLOOP_ROOT
            / "foundry"
            / "templates"
            / "assets"
            / "improve_bot_mcp_servers"
        )
        manifest = json.loads(
            (server_dir / "cyberloop_mcp_server.json").read_text(
                encoding="utf-8")
        )
        server_text = (server_dir / "cyberloop_mcp_server.py").read_text(
            encoding="utf-8")
        registry = BlockRegistry.from_directory(
            CYBERLOOP_ROOT / "foundry" / "templates" / "blocks")
        improve = registry.get("ImproveBot")

        assert "def request_eval()" in server_text
        assert manifest["tools"] == [improve.env["HANDOFF_TOOLS"]]

    def test_decker_desktop_is_persistent_service_target(self, project_setup):
        _, template, _ = project_setup
        desktop = next(b for b in template.blocks if b.name == "DeckerDesktop")

        assert desktop.runner == "container"
        assert desktop.lifecycle == "workspace_persistent"
        assert desktop.image == "cyberloop-decker-desktop:latest"
        assert desktop.state == "none"
        assert "--restart=unless-stopped" in desktop.docker_args
        assert "--network=cyberloop-cua" in desktop.docker_args
        assert "--network-alias=decker-desktop" in desktop.docker_args
        assert desktop.env["DESKTOP_API_PORT"] == "8080"
        assert desktop.outputs_for("normal") == []

    def test_cua_play_agent_talks_to_desktop_by_project_convention(
            self, project_setup):
        _, template, _ = project_setup
        agent = next(b for b in template.blocks if b.name == "CuaPlayAgent")

        assert agent.runner == "container"
        assert agent.lifecycle == "one_shot"
        assert agent.state == "managed"
        assert agent.image == "cyberloop-cua-play-agent:latest"
        assert "--network=cyberloop-cua" in agent.docker_args
        assert agent.env["DESKTOP_URL"] == "http://decker-desktop:8080"
        assert agent.env["CUA_SCREENSHOT_DIR"] == (
            "/output/cua_trace/screenshots")
        assert agent.env["MCP_SERVERS"] == "cyberloop_cua"
        assert (
            agent.env["HANDOFF_TOOLS"]
            == "mcp__cyberloop_cua__finish_segment"
        )
        assert agent.env["HANDOFF_TERMINATION_REASON"] == "segment_complete"
        assert [slot.name for slot in agent.inputs] == ["decker_state"]
        assert agent.inputs[0].optional is True
        assert [slot.name for slot in agent.outputs_for("normal")] == [
            "decker_state",
            "cua_trace",
        ]
        assert [
            slot.name for slot in agent.outputs_for("segment_complete")
        ] == ["decker_state", "cua_trace"]
        trace = agent.outputs_for("segment_complete")[1]
        assert trace.sequence is not None
        assert trace.sequence.name == "cua_trace"
        assert trace.sequence.scope == "enclosing_lane"
        assert trace.sequence.role == "segment_trace"
        assert [
            slot.name for slot in agent.outputs_for("desktop_unreachable")
        ] == ["decker_state", "cua_trace"]

    def test_cua_agent_image_bakes_prompt_and_mcp_adapter(self):
        dockerfile = CYBERLOOP_ROOT / "docker" / "Dockerfile.cua-play-agent"
        text = dockerfile.read_text(encoding="utf-8")

        assert "FROM flywheel-claude:latest" in text
        assert (
            "COPY foundry/templates/assets/cua_play_prompt/prompt.md "
            "/app/agent/prompt.md"
        ) in text
        assert (
            "COPY foundry/templates/assets/cua_mcp_servers "
            "/app/agent/mcp_servers"
        ) in text
        assert "ENTRYPOINT [\"/app/cua_controller_entrypoint.sh\"]" in text

    def test_decker_desktop_image_derives_from_desktop_battery(self):
        dockerfile = CYBERLOOP_ROOT / "docker" / "Dockerfile.decker-desktop"
        text = dockerfile.read_text(encoding="utf-8")

        assert "FROM flywheel-desktop:latest" in text
        assert "DECKER_WINDOW=1280x720" in text
        assert "DECKER_FPS=5" in text
        assert (
            "DECKER_STATE_LOAD=/desktop/shared/decker_state/load.save"
            in text
        )
        assert (
            "DECKER_STATE_EXPORT=/desktop/shared/decker_state/state.json"
            in text
        )
        assert "WORKDIR /app" in text
        assert "COPY assets/ /app/assets/" in text

    def test_decker_desktop_assets_cover_loaded_sprite_paths(self):
        player_assets = CYBERLOOP_ROOT / "assets" / "sprites" / "player"
        enemy_assets = CYBERLOOP_ROOT / "assets" / "sprites" / "enemies"
        encounters = (
            CYBERLOOP_ROOT
            / "crates"
            / "content"
            / "src"
            / "encounters.rs"
        ).read_text(encoding="utf-8")

        assert (player_assets / "fighter.png").is_file()
        assert (player_assets / "fighter_dead.png").is_file()
        for enemy_id in sorted(set(
            part.split('"')[1]
            for part in encounters.split("enemy_ids: &[")[1:]
            for part in part.split("]", 1)[0].split(",")
            if '"' in part
        )):
            assert (enemy_assets / f"{enemy_id}.png").is_file()
            assert (enemy_assets / f"{enemy_id}_dead.png").is_file()

    def test_cua_mcp_manifest_matches_block_handoff(self):
        server_dir = (
            CYBERLOOP_ROOT
            / "foundry"
            / "templates"
            / "assets"
            / "cua_mcp_servers"
        )
        manifest = json.loads(
            (server_dir / "cyberloop_cua_mcp_server.json").read_text(
                encoding="utf-8")
        )
        server_text = (
            server_dir / "cyberloop_cua_mcp_server.py"
        ).read_text(encoding="utf-8")
        registry = BlockRegistry.from_directory(
            CYBERLOOP_ROOT / "foundry" / "templates" / "blocks")
        agent = registry.get("CuaPlayAgent")

        assert "def finish_segment()" in server_text
        assert agent.env["HANDOFF_TOOLS"] in manifest["tools"]

    def test_bot_gui_escalation_blocks_are_wired(self, project_setup):
        _, template, _ = project_setup
        step = next(b for b in template.blocks if b.name == "StepThroughGui")
        gate = next(b for b in template.blocks if b.name == "GuiEscalationGate")
        improver = next(
            b for b in template.blocks
            if b.name == "ImproveGuiActionDecider")

        assert step.image == "cyberloop-gui-step:latest"
        assert "--network=cyberloop-cua" in step.docker_args
        assert step.env["SNAP_TO_PREDICTION_ON_MISMATCH"] == "false"
        assert [slot.name for slot in step.inputs] == [
            "bot", "gui_action_decider", "decker_state"]
        assert step.inputs[2].optional is True
        assert [slot.name for slot in step.outputs_for("action_taken")] == [
            "decker_state", "game_step", "cua_trace"]
        game_step = step.outputs_for("action_taken")[1]
        assert game_step.sequence is not None
        assert game_step.sequence.name == "decker_gui_game"
        assert game_step.sequence.scope == "enclosing_lane"
        trace = step.outputs_for("action_taken")[2]
        assert trace.sequence is not None
        assert trace.sequence.name == "gui_step_trace"
        assert [slot.name for slot in step.outputs_for("bot_error")] == [
            "decker_state", "game_step", "cua_trace"]

        assert gate.image == "cyberloop-gui-escalation-gate:latest"
        assert gate.inputs[0].sequence is not None
        assert gate.inputs[0].sequence.name == "decker_gui_game"
        invocation = gate.on_termination["escalation_requested"][0]
        assert invocation.block == "ImproveGuiActionDecider"
        assert invocation.bind["escalation_request"].parent_output == (
            "escalation_request")

        assert improver.image == (
            "cyberloop-improve-gui-action-decider-agent:latest")
        assert improver.env["MODEL"] == "claude-sonnet-4-6[1m]"
        assert [slot.name for slot in improver.inputs] == [
            "escalation_request",
            "gui_action_decider",
            "game_step",
            "cua_trace",
            "game_step_history",
            "gui_step_trace_history",
        ]
        assert improver.inputs[5].sequence is not None
        assert improver.inputs[5].sequence.name == "gui_step_trace"
        assert [slot.name for slot in improver.outputs_for("normal")] == [
            "gui_action_decider"]



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
        assert {
            "Train",
            "Eval",
            "EvalBot",
            "ImproveBot",
            "DeckerDesktop",
            "CuaPlayAgent",
            "StepThroughGui",
            "GuiEscalationGate",
            "ImproveGuiActionDecider",
        }.issubset(
            set(registry.names()))

    def test_template_loads_and_resolves_eval(self):
        registry = BlockRegistry.from_directory(self.BLOCKS_DIR)
        template = Template.from_yaml(
            self.TEMPLATE_PATH, block_registry=registry)
        assert "Eval" in [b.name for b in template.blocks]
        assert "Train" in [b.name for b in template.blocks]
        assert "EvalBot" in [b.name for b in template.blocks]
        assert "ImproveBot" in [b.name for b in template.blocks]
        assert "DeckerDesktop" in [b.name for b in template.blocks]
        assert "CuaPlayAgent" in [b.name for b in template.blocks]
        assert "StepThroughGui" in [b.name for b in template.blocks]
        assert "GuiEscalationGate" in [b.name for b in template.blocks]
        assert "ImproveGuiActionDecider" in [b.name for b in template.blocks]
        assert {a.name for a in template.artifacts} == {
            "game_engine",
            "checkpoint",
            "score",
            "bot",
            "decker_state",
            "cua_trace",
            "game_step",
            "gui_action_decider",
            "escalation_request",
        }

    def test_improve_bot_pattern_declares_lanes_and_fixture(self):
        pattern = PatternDeclaration.from_yaml(
            CYBERLOOP_ROOT
            / "foundry"
            / "templates"
            / "patterns"
            / "improve_bot.yaml"
        )

        assert pattern.lanes == ["lane_0", "lane_1", "lane_2"]
        assert pattern.params["model"].default == "claude-sonnet-4-6[1m]"
        assert pattern.params["eval_episodes"].default == 4000
        assert pattern.params["max_evals"].default == 5
        assert pattern.fixtures["bot"].source == (
            "foundry/templates/assets/bots/baseline")
        assert pattern.steps == []
        assert "improve_bot_lane" in pattern.patterns
        lane_pattern = pattern.patterns["improve_bot_lane"]
        assert lane_pattern.params["max_evals"].type == "int"
        assert lane_pattern.body

    def test_improve_bot_sonnet_2lane_pattern_declares_two_lanes(self):
        pattern = PatternDeclaration.from_yaml(
            CYBERLOOP_ROOT
            / "foundry"
            / "templates"
            / "patterns"
            / "improve_bot_sonnet_2lane.yaml"
        )

        assert pattern.name == "improve_bot_sonnet_2lane"
        assert pattern.lanes == ["lane_0", "lane_1"]
        assert pattern.params["model"].default == "claude-sonnet-4-6[1m]"
        assert pattern.params["eval_episodes"].default == 4000
        assert pattern.params["max_evals"].default == 5
        assert pattern.fixtures["bot"].source == (
            "foundry/templates/assets/bots/baseline")
        assert pattern.steps == []
        assert pattern.patterns["improve_bot_lane"].body

    def test_improve_bot_sonnet_1lane_pattern_declares_one_lane(self):
        pattern = PatternDeclaration.from_yaml(
            CYBERLOOP_ROOT
            / "foundry"
            / "templates"
            / "patterns"
            / "improve_bot_sonnet_1lane.yaml"
        )

        assert pattern.name == "improve_bot_sonnet_1lane"
        assert pattern.lanes == ["lane_0"]
        assert pattern.params["model"].default == "claude-sonnet-4-6[1m]"
        assert pattern.params["eval_episodes"].default == 4000
        assert pattern.params["max_evals"].default == 3
        assert pattern.fixtures["bot"].source == (
            "foundry/templates/assets/bots/baseline")
        assert pattern.steps == []
        assert pattern.patterns["improve_bot_lane"].body

    def test_cua_play_pattern_uses_controller_loop(self):
        pattern = PatternDeclaration.from_yaml(
            CYBERLOOP_ROOT
            / "foundry"
            / "templates"
            / "patterns"
            / "cua_play_sonnet_1lane.yaml"
        )

        assert pattern.name == "cua_play_sonnet_1lane"
        assert pattern.params["model"].default == "claude-sonnet-4-6[1m]"
        assert pattern.params["max_turns"].default == 120
        assert pattern.params["max_segments"].default == 20
        assert pattern.lanes == ["default"]
        assert pattern.steps == []
        assert pattern.body
        run_until = pattern.body[0]
        assert run_until.fail_on == ["desktop_unreachable"]

    def test_bot_gui_escalate_pattern_uses_step_and_gate_loop(self):
        pattern = PatternDeclaration.from_yaml(
            CYBERLOOP_ROOT
            / "foundry"
            / "templates"
            / "patterns"
            / "bot_gui_escalate_sonnet_1lane.yaml"
        )

        assert pattern.name == "bot_gui_escalate_sonnet_1lane"
        assert pattern.params["max_actions"].default == 50
        assert pattern.params["escalation_interval"].default == 0
        assert pattern.params["snap_to_prediction_on_mismatch"].default is False
        assert pattern.fixtures["bot"].source == (
            "foundry/templates/assets/bots/baseline")
        assert pattern.fixtures["gui_action_decider"].source == (
            "foundry/templates/assets/gui_action_decider_seed")
        run_until = pattern.body[0]
        assert run_until.block == "StepThroughGui"
        assert set(run_until.continue_on) == {"action_taken"}
        assert run_until.stop_on == ["game_over"]
        assert run_until.fail_on == ["desktop_unreachable", "bot_error"]
        assert run_until.after_every[0].reason == "action_taken"
