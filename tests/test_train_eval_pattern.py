"""Train -> Eval pattern execution through Flywheel."""

from __future__ import annotations

import json
import shutil
from pathlib import Path
from unittest.mock import patch

from flywheel.cli import main
from flywheel.container import ContainerResult
from flywheel.pattern_declaration import PatternDeclaration
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
