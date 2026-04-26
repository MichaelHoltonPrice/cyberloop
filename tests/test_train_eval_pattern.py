"""Train -> Eval pattern execution through Flywheel."""

from __future__ import annotations

import json
import shutil
import subprocess
from pathlib import Path
from unittest.mock import patch

from flywheel.cli import main
from flywheel.container import ContainerResult
from flywheel.pattern_declaration import PatternDeclaration
from flywheel.pattern_execution import run_pattern
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
    sonnet = PatternDeclaration.from_yaml(
        pattern_dir / "improve_bot_sonnet_2lane.yaml")

    assert full.steps == []
    assert full.lanes == ["lane_0", "lane_1", "lane_2"]
    assert full.patterns["improve_bot_lane"].body
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
