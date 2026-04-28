"""Train -> Eval pattern execution through Flywheel."""

from __future__ import annotations

import json
import shutil
import subprocess
from pathlib import Path
from unittest.mock import patch

import pytest
from flywheel.cli import main
from flywheel.container import ContainerResult
from flywheel.pattern_declaration import PatternDeclaration
from flywheel.pattern_execution import PatternRunError, run_pattern
from flywheel.workspace import Workspace

CYBERLOOP_ROOT = Path(__file__).resolve().parent.parent


def test_train_eval_pattern_file_parses():
    pattern = PatternDeclaration.from_yaml(
        CYBERLOOP_ROOT / "foundry" / "templates"
        / "patterns" / "train_eval.yaml")

    assert pattern.name == "train_eval"
    assert [step.name for step in pattern.steps] == ["train", "eval"]
    assert pattern.steps[1].cohort.members[0].inputs["checkpoint"]
    assert pattern.steps[1].cohort.members[0].args == [
        "--episodes", "4000", "--backend", "numpy",
    ]


def test_improve_bot_patterns_use_lane_local_loop_grammar():
    pattern_dir = (
        CYBERLOOP_ROOT / "foundry" / "templates" / "patterns")
    full = PatternDeclaration.from_yaml(pattern_dir / "improve_bot.yaml")
    sonnet_1lane = PatternDeclaration.from_yaml(
        pattern_dir / "improve_bot_sonnet_1lane.yaml")
    sonnet = PatternDeclaration.from_yaml(
        pattern_dir / "improve_bot_sonnet_2lane.yaml")

    assert full.steps == []
    assert full.lanes == ["lane_0", "lane_1", "lane_2"]
    assert full.patterns["improve_bot_lane"].body
    assert sonnet_1lane.steps == []
    assert sonnet_1lane.lanes == ["lane_0"]
    assert sonnet_1lane.params["max_evals"].default == 3
    assert sonnet.steps == []
    assert sonnet.lanes == ["lane_0", "lane_1"]
    assert sonnet.params["max_evals"].default == 5


def test_improve_bot_sonnet_2lane_pattern_runs_lane_local_loop(
    project_setup,
):
    ws, template, project = project_setup
    baseline = project / "foundry" / "templates" / "assets" / "bots" / "baseline"
    baseline.mkdir(parents=True)
    (baseline / "bot.py").write_text("BASE", encoding="utf-8")
    (project / "crates").mkdir()
    (project / "crates" / "README.md").write_text("engine", encoding="utf-8")
    (project / ".gitignore").write_text(
        "foundry/workspaces/\n", encoding="utf-8")
    _commit_project(project)
    pattern = PatternDeclaration.from_yaml(
        CYBERLOOP_ROOT / "foundry" / "templates"
        / "patterns" / "improve_bot_sonnet_2lane.yaml")
    improve_calls = 0
    seen_improve: list[tuple[str, bool, str]] = []
    seen_eval_args: list[list[str] | None] = []

    def fake_run_container(config, args=None):
        nonlocal improve_calls
        mounts = {
            container: Path(host)
            for host, container, _mode in config.mounts
        }
        (mounts["/flywheel"] / "termination").write_text(
            "normal", encoding="utf-8")
        if config.image == "cyberloop-improve-bot-agent:latest":
            improve_calls += 1
            lane_index = 0 if improve_calls <= 2 else 1
            lane_iter = 1 if improve_calls in (1, 3) else 2
            source = (mounts["/input/bot"] / "bot.py").read_text(
                encoding="utf-8")
            has_score = "/input/score" in mounts
            seen_improve.append((source, has_score, config.env["MODEL"]))
            output = mounts["/output/bot"] / "bot.py"
            output.parent.mkdir(parents=True, exist_ok=True)
            output.write_text(
                f"L{lane_index}-I{lane_iter}", encoding="utf-8")
            state_file = mounts["/flywheel"] / "state" / "session.txt"
            state_file.write_text(
                f"L{lane_index}-I{lane_iter}", encoding="utf-8")
            if lane_iter == 1:
                (mounts["/flywheel"] / "termination").write_text(
                    "eval_requested", encoding="utf-8")
            return ContainerResult(exit_code=0, elapsed_s=0.1)
        if config.image == "cyberloop-eval:latest":
            seen_eval_args.append(args)
            score = mounts["/output/score"] / "scores.json"
            score.parent.mkdir(parents=True, exist_ok=True)
            score.write_text(json.dumps({
                "mean": 1.0,
                "median": 1.0,
                "std": 0.0,
                "min": 1,
                "max": 1,
                "p25": 1.0,
                "p75": 1.0,
                "episodes": 4000,
                "fights_won": [1],
                "total_steps": 1,
                "elapsed_s": 0.1,
                "timed_out": 0,
                "errors": 0,
            }), encoding="utf-8")
            return ContainerResult(exit_code=0, elapsed_s=0.1)
        raise AssertionError(config.image)

    with patch(
        "flywheel.execution.run_container",
        side_effect=fake_run_container,
    ):
        result = run_pattern(ws, pattern, template, project)

    reloaded = Workspace.load(ws.path)
    run = reloaded.runs[result.run_id]
    assert run.status == "succeeded"
    assert run.lanes == ["lane_0", "lane_1"]
    assert seen_improve == [
        ("BASE", False, "claude-sonnet-4-6[1m]"),
        ("L0-I1", True, "claude-sonnet-4-6[1m]"),
        ("BASE", False, "claude-sonnet-4-6[1m]"),
        ("L1-I1", True, "claude-sonnet-4-6[1m]"),
    ]
    assert [step.kind for step in run.steps] == ["run_until", "run_until"]
    assert [len(step.members) for step in run.steps] == [2, 2]
    assert [member.lane for step in run.steps for member in step.members] == [
        "lane_0", "lane_0", "lane_1", "lane_1"]
    assert all(args is not None and "4000" in args for args in seen_eval_args)
    lane_0_first, lane_0_second = run.steps[0].members
    first_exec = reloaded.executions[lane_0_first.execution_id]
    second_exec = reloaded.executions[lane_0_second.execution_id]
    assert second_exec.input_bindings["bot"] == first_exec.output_bindings["bot"]
    invocation = reloaded.invocations[lane_0_first.invocation_ids[0]]
    score_exec = reloaded.executions[invocation.invoked_execution_id]
    assert second_exec.input_bindings["score"] == (
        score_exec.output_bindings["score"])


def test_bot_gui_escalate_pattern_runs_gate_and_improver(project_setup):
    ws, template, project = project_setup
    baseline = project / "foundry" / "templates" / "assets" / "bots" / "baseline"
    baseline.mkdir(parents=True)
    (baseline / "bot.py").write_text("BOT", encoding="utf-8")
    gui_seed = (
        project / "foundry" / "templates" / "assets"
        / "gui_action_decider_seed")
    gui_seed.mkdir(parents=True)
    (gui_seed / "gui_action_decider.py").write_text(
        "def plan_actions(context):\n    return []\n",
        encoding="utf-8",
    )
    patterns = project / "foundry" / "templates" / "patterns"
    patterns.mkdir(parents=True)
    shutil.copy2(
        CYBERLOOP_ROOT / "foundry" / "templates" / "patterns"
        / "bot_gui_escalate_sonnet_1lane.yaml",
        patterns / "bot_gui_escalate_sonnet_1lane.yaml",
    )
    pattern = PatternDeclaration.from_yaml(
        patterns / "bot_gui_escalate_sonnet_1lane.yaml")
    calls: list[str] = []
    step_calls = 0

    def fake_run_container(config, args=None):
        nonlocal step_calls
        del args
        calls.append(config.image)
        all_mounts = {
            container: Path(host)
            for host, container, _mode in config.mounts
        }
        writable_mounts = {
            container: Path(host)
            for host, container, mode in config.mounts
            if mode == "rw"
        }
        (writable_mounts["/flywheel"] / "termination").write_text(
            "normal", encoding="utf-8")
        if config.image == "cyberloop-gui-step:latest":
            step_calls += 1
            if step_calls == 2:
                mounted = (
                    all_mounts["/input/gui_action_decider"]
                    / "gui_action_decider.py"
                ).read_text(encoding="utf-8")
                assert "# v2" in mounted
            _write_decker_state(writable_mounts["/output/decker_state"])
            _write_game_step(
                writable_mounts["/output/game_step"],
                prediction_correct=step_calls == 2,
            )
            _write_trace(writable_mounts["/output/cua_trace"])
            (writable_mounts["/flywheel"] / "termination").write_text(
                "action_taken", encoding="utf-8")
            return ContainerResult(exit_code=0, elapsed_s=0.1)
        if config.image == "cyberloop-gui-escalation-gate:latest":
            if step_calls == 1:
                out = writable_mounts["/output/escalation_request"]
                out.mkdir(parents=True, exist_ok=True)
                (out / "escalation_request.json").write_text(json.dumps({
                    "reason": "prediction_mismatch",
                }), encoding="utf-8")
                (writable_mounts["/flywheel"] / "termination").write_text(
                    "escalation_requested", encoding="utf-8")
            return ContainerResult(exit_code=0, elapsed_s=0.1)
        if config.image == "cyberloop-improve-gui-action-decider-agent:latest":
            out = writable_mounts["/output/gui_action_decider"]
            out.mkdir(parents=True, exist_ok=True)
            (out / "gui_action_decider.py").write_text(
                "# v2\ndef plan_actions(context):\n    return []\n",
                encoding="utf-8",
            )
            return ContainerResult(exit_code=0, elapsed_s=0.1)
        raise AssertionError(config.image)

    with patch(
        "flywheel.execution.run_container",
        side_effect=fake_run_container,
    ):
        result = run_pattern(
            ws,
            pattern,
            template,
            project,
            param_overrides={"max_actions": "2"},
        )

    reloaded = Workspace.load(ws.path)
    run = reloaded.runs[result.run_id]
    assert run.status == "succeeded"
    assert calls == [
        "cyberloop-gui-step:latest",
        "cyberloop-gui-escalation-gate:latest",
        "cyberloop-improve-gui-action-decider-agent:latest",
        "cyberloop-gui-step:latest",
        "cyberloop-gui-escalation-gate:latest",
    ]
    assert run.steps[0].kind == "run_until"
    assert run.steps[0].stop_kind == "budget_exhausted"
    assert len(reloaded.sequence_entries) == 5
    assert any(
        entry.sequence_name == "gui_step_trace"
        for entry in reloaded.sequence_entries
    )
    assert any(
        entry.sequence_name == "gui_escalation"
        for entry in reloaded.sequence_entries
    )


def test_bot_gui_escalate_pattern_fails_cleanly_on_desktop_unreachable(
    project_setup,
):
    ws, template, project = project_setup
    _prepare_bot_gui_pattern_assets(project)
    pattern = PatternDeclaration.from_yaml(
        project / "foundry" / "templates" / "patterns"
        / "bot_gui_escalate_sonnet_1lane.yaml")

    def fake_run_container(config, args=None):
        del args
        mounts = {
            container: Path(host)
            for host, container, mode in config.mounts
            if mode == "rw"
        }
        (mounts["/flywheel"] / "termination").write_text(
            "normal", encoding="utf-8")
        assert config.image == "cyberloop-gui-step:latest"
        _write_decker_state(mounts["/output/decker_state"])
        _write_game_step(mounts["/output/game_step"], prediction_correct=False)
        _write_trace(mounts["/output/cua_trace"])
        (mounts["/flywheel"] / "termination").write_text(
            "desktop_unreachable", encoding="utf-8")
        return ContainerResult(exit_code=0, elapsed_s=0.1)

    with patch(
        "flywheel.execution.run_container",
        side_effect=fake_run_container,
    ):
        with pytest.raises(PatternRunError, match="desktop_unreachable"):
            run_pattern(ws, pattern, template, project)

    run = next(iter(Workspace.load(ws.path).runs.values()))
    assert run.status == "failed"
    assert run.steps[0].stop_kind == "fail_on"
    assert run.steps[0].terminal_reason == "desktop_unreachable"


def test_bot_gui_escalate_pattern_stops_cleanly_on_game_over(project_setup):
    ws, template, project = project_setup
    _prepare_bot_gui_pattern_assets(project)
    pattern = PatternDeclaration.from_yaml(
        project / "foundry" / "templates" / "patterns"
        / "bot_gui_escalate_sonnet_1lane.yaml")
    calls: list[str] = []

    def fake_run_container(config, args=None):
        del args
        calls.append(config.image)
        mounts = {
            container: Path(host)
            for host, container, mode in config.mounts
            if mode == "rw"
        }
        (mounts["/flywheel"] / "termination").write_text(
            "normal", encoding="utf-8")
        assert config.image == "cyberloop-gui-step:latest"
        _write_decker_state(mounts["/output/decker_state"])
        _write_game_step(mounts["/output/game_step"], prediction_correct=True)
        _write_trace(mounts["/output/cua_trace"])
        (mounts["/flywheel"] / "termination").write_text(
            "game_over", encoding="utf-8")
        return ContainerResult(exit_code=0, elapsed_s=0.1)

    with patch(
        "flywheel.execution.run_container",
        side_effect=fake_run_container,
    ):
        result = run_pattern(ws, pattern, template, project)

    run = Workspace.load(ws.path).runs[result.run_id]
    assert run.status == "succeeded"
    assert run.steps[0].stop_kind == "stop_on"
    assert run.steps[0].terminal_reason == "game_over"
    assert calls == ["cyberloop-gui-step:latest"]


def test_bot_gui_escalate_pattern_fails_cleanly_on_bot_error(project_setup):
    ws, template, project = project_setup
    _prepare_bot_gui_pattern_assets(project)
    pattern = PatternDeclaration.from_yaml(
        project / "foundry" / "templates" / "patterns"
        / "bot_gui_escalate_sonnet_1lane.yaml")

    def fake_run_container(config, args=None):
        del args
        mounts = {
            container: Path(host)
            for host, container, mode in config.mounts
            if mode == "rw"
        }
        (mounts["/flywheel"] / "termination").write_text(
            "normal", encoding="utf-8")
        assert config.image == "cyberloop-gui-step:latest"
        _write_decker_state(mounts["/output/decker_state"])
        _write_game_step(mounts["/output/game_step"], prediction_correct=False)
        _write_trace(mounts["/output/cua_trace"])
        (mounts["/flywheel"] / "termination").write_text(
            "bot_error", encoding="utf-8")
        return ContainerResult(exit_code=0, elapsed_s=0.1)

    with patch(
        "flywheel.execution.run_container",
        side_effect=fake_run_container,
    ):
        with pytest.raises(PatternRunError, match="bot_error"):
            run_pattern(ws, pattern, template, project)

    run = next(iter(Workspace.load(ws.path).runs.values()))
    assert run.status == "failed"
    assert run.steps[0].stop_kind == "fail_on"
    assert run.steps[0].terminal_reason == "bot_error"


def test_train_eval_pattern_runs_through_base_flywheel_cli(
    project_setup,
    monkeypatch,
):
    ws, _template, project = project_setup
    patterns = project / "foundry" / "templates" / "patterns"
    patterns.mkdir(parents=True)
    shutil.copy2(
        CYBERLOOP_ROOT / "foundry" / "templates"
        / "patterns" / "train_eval.yaml",
        patterns / "train_eval.yaml",
    )
    monkeypatch.chdir(project)
    seen_args: list[list[str] | None] = []

    def fake_run_container(config, args=None):
        seen_args.append(args)
        mounts = {
            container: Path(host)
            for host, container, mode in config.mounts
            if mode == "rw"
        }
        (mounts["/flywheel"] / "termination").write_text(
            "normal", encoding="utf-8")
        if "/output/checkpoint" in mounts:
            _write_checkpoint(mounts["/output/checkpoint"])
        if "/output/score" in mounts:
            _write_score(mounts["/output/score"])
        return ContainerResult(exit_code=0, elapsed_s=1.0)

    with patch(
        "flywheel.execution.run_container",
        side_effect=fake_run_container,
    ):
        main([
            "run", "pattern", "train_eval",
            "--workspace", str(ws.path),
            "--template", "cyberloop",
        ])

    reloaded = Workspace.load(ws.path)
    run = next(iter(reloaded.runs.values()))
    assert run.status == "succeeded"
    assert [step.name for step in run.steps] == ["train", "eval"]
    train_member = run.steps[0].members[0]
    eval_member = run.steps[1].members[0]
    train_execution = reloaded.executions[train_member.execution_id]
    eval_execution = reloaded.executions[eval_member.execution_id]
    assert eval_execution.input_bindings == {
        "checkpoint": train_execution.output_bindings["checkpoint"]
    }
    assert set(eval_execution.output_bindings) == {"score"}
    assert seen_args == [
        ["--subclass", "dueling", "--combat-only"],
        ["--episodes", "4000", "--backend", "numpy"],
    ]


def _prepare_bot_gui_pattern_assets(project: Path) -> None:
    baseline = project / "foundry" / "templates" / "assets" / "bots" / "baseline"
    baseline.mkdir(parents=True)
    (baseline / "bot.py").write_text("BOT", encoding="utf-8")
    gui_seed = (
        project / "foundry" / "templates" / "assets"
        / "gui_action_decider_seed")
    gui_seed.mkdir(parents=True)
    (gui_seed / "gui_action_decider.py").write_text(
        "def plan_actions(context):\n    return []\n",
        encoding="utf-8",
    )
    patterns = project / "foundry" / "templates" / "patterns"
    patterns.mkdir(parents=True)
    shutil.copy2(
        CYBERLOOP_ROOT / "foundry" / "templates" / "patterns"
        / "bot_gui_escalate_sonnet_1lane.yaml",
        patterns / "bot_gui_escalate_sonnet_1lane.yaml",
    )


def _commit_project(project: Path) -> None:
    subprocess.run(["git", "init"], cwd=project, check=True, capture_output=True)
    subprocess.run(
        ["git", "config", "user.email", "test@example.com"],
        cwd=project,
        check=True,
        capture_output=True,
    )
    subprocess.run(
        ["git", "config", "user.name", "Test"],
        cwd=project,
        check=True,
        capture_output=True,
    )
    subprocess.run(["git", "add", "."], cwd=project, check=True,
                   capture_output=True)
    subprocess.run(
        ["git", "commit", "-m", "test project"],
        cwd=project,
        check=True,
        capture_output=True,
    )


def _write_checkpoint(path: Path) -> None:
    (path / "checkpoint.pt").write_bytes(b"weights")
    (path / "run.json").write_text(json.dumps({
        "schema_version": 1,
        "artifact_type": "cyberloop.checkpoint",
        "model_path": "checkpoint.pt",
        "stage": "combat",
        "subclass": "dueling",
        "race": "human",
        "synergy_group": None,
        "seed": 1,
        "timesteps": 10,
    }), encoding="utf-8")
    (path / "output_manifest.json").write_text(json.dumps({
        "artifacts": [{
            "type": "checkpoint",
            "path": "checkpoint.pt",
        }],
    }), encoding="utf-8")


def _write_score(path: Path) -> None:
    (path / "scores.json").write_text(json.dumps({
        "mean": 1.0,
        "median": 1.0,
        "std": 0.0,
        "min": 1,
        "max": 1,
        "p25": 1.0,
        "p75": 1.0,
        "episodes": 2,
        "fights_won": [1, 1],
        "total_steps": 20,
        "elapsed_s": 1.0,
        "timed_out": 0,
        "errors": 0,
    }), encoding="utf-8")
    (path / "output_manifest.json").write_text(json.dumps({
        "artifacts": [{
            "type": "score",
            "path": "scores.json",
        }],
    }), encoding="utf-8")


def _write_decker_state(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)
    (path / "decker_state.save").write_text("SAVE", encoding="utf-8")
    (path / "state.json").write_text("{}", encoding="utf-8")
    (path / "actions.json").write_text("[]", encoding="utf-8")


def _write_game_step(path: Path, *, prediction_correct: bool) -> None:
    path.mkdir(parents=True, exist_ok=True)
    (path / "game_step.json").write_text(json.dumps({
        "schema_version": 1,
        "prediction_correct": prediction_correct,
    }), encoding="utf-8")


def _write_trace(path: Path) -> None:
    path.mkdir(parents=True, exist_ok=True)
    (path / "trace.json").write_text("{}", encoding="utf-8")
