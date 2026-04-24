"""Ad hoc cyberloop block execution through Flywheel's base CLI path."""

from __future__ import annotations

import json
from pathlib import Path
from unittest.mock import patch

from flywheel.cli import main
from flywheel.container import ContainerResult
from flywheel.workspace import Workspace


def test_train_runs_through_base_run_block_cli(
    project_setup, monkeypatch,
):
    """``flywheel run block`` drives Train through canonical run_block."""
    ws, _template, project = project_setup
    seen: dict[str, object] = {}

    def fake_run_container(config, args=None):
        seen["image"] = config.image
        seen["args"] = args
        seen["mounts"] = config.mounts
        mounts = {
            container: Path(host)
            for host, container, mode in config.mounts
            if mode == "rw"
        }
        (mounts["/flywheel"] / "termination").write_text(
            "normal", encoding="utf-8",
        )
        checkpoint_dir = mounts["/output/checkpoint"]
        (checkpoint_dir / "checkpoint.pt").write_text(
            "weights", encoding="utf-8",
        )
        (checkpoint_dir / "run.json").write_text(json.dumps({
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
        (checkpoint_dir / "output_manifest.json").write_text(
            json.dumps({
                "artifacts": [{
                    "type": "checkpoint",
                    "path": "checkpoint.pt",
                }],
            }),
            encoding="utf-8",
        )
        return ContainerResult(exit_code=0, elapsed_s=1.25)

    monkeypatch.chdir(project)
    with (
        patch(
            "flywheel.execution.run_container",
            side_effect=fake_run_container,
        ),
        patch(
            "flywheel.project_hooks.load_project_hooks_class",
            side_effect=AssertionError(
                "run block must not load project hooks"),
        ),
    ):
        main([
            "run", "block",
            "--workspace", str(ws.path),
            "--block", "Train",
            "--template", "cyberloop",
            "--",
            "--example-flag", "example-value",
        ])

    reloaded = Workspace.load(ws.path)
    execution = next(iter(reloaded.executions.values()))
    assert execution.block_name == "Train"
    assert execution.status == "succeeded"
    assert execution.termination_reason == "normal"
    assert execution.runner == "container_one_shot"
    assert set(execution.output_bindings) == {"checkpoint"}

    checkpoint_id = execution.output_bindings["checkpoint"]
    checkpoint = reloaded.path / "artifacts" / checkpoint_id
    assert (checkpoint / "checkpoint.pt").read_text() == "weights"
    assert json.loads((checkpoint / "run.json").read_text())[
        "artifact_type"] == "cyberloop.checkpoint"
    assert seen["image"] == "cyberloop-train:latest"
    assert seen["args"] == ["--example-flag", "example-value"]

    mounts = seen["mounts"]
    assert ("/output/checkpoint", "rw") in [
        (container, mode) for _host, container, mode in mounts
    ]
    assert ("/flywheel", "rw") in [
        (container, mode) for _host, container, mode in mounts
    ]
    assert "/input/checkpoint" not in [
        container for _host, container, _mode in mounts
    ]
