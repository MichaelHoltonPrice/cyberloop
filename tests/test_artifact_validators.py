"""Tests for Cyberloop's Flywheel artifact validators."""

from __future__ import annotations

import json
from pathlib import Path

import pytest

from cyberloop.artifact_validators import build_registry
from flywheel.artifact_validator import ArtifactValidationError


def test_checkpoint_validator_accepts_flat_checkpoint(tmp_path: Path):
    candidate = tmp_path / "checkpoint"
    candidate.mkdir()
    (candidate / "checkpoint.pt").write_bytes(b"weights")
    (candidate / "run.json").write_text(json.dumps({
        "schema_version": 1,
        "artifact_type": "cyberloop.checkpoint",
        "model_path": "checkpoint.pt",
        "stage": "combat",
        "subclass": "dueling",
        "race": "human",
        "synergy_group": None,
        "seed": 123,
        "timesteps": 200000,
    }), encoding="utf-8")
    (candidate / "output_manifest.json").write_text(
        '{"artifacts": [{"type": "checkpoint", "path": "checkpoint.pt"}]}',
        encoding="utf-8",
    )

    build_registry().validate("checkpoint", None, candidate)


def test_checkpoint_validator_rejects_nested_only_checkpoint(tmp_path: Path):
    candidate = tmp_path / "checkpoint"
    nested = candidate / "dueling_20260424_010537_b5a35d1e"
    nested.mkdir(parents=True)
    (nested / "final_model.pt").write_bytes(b"weights")
    (candidate / "output_manifest.json").write_text(
        '{"artifacts": [{"type": "checkpoint"}]}',
        encoding="utf-8",
    )

    with pytest.raises(ArtifactValidationError, match="checkpoint.pt"):
        build_registry().validate("checkpoint", None, candidate)


def test_score_validator_accepts_successful_scores(tmp_path: Path):
    candidate = tmp_path / "score"
    candidate.mkdir()
    (candidate / "scores.json").write_text(json.dumps({
        "mean": 1.0,
        "median": 1.0,
        "std": 0.0,
        "min": 1,
        "max": 1,
        "p25": 1.0,
        "p75": 1.0,
        "episodes": 2,
        "fights_won": [1, 1],
        "total_steps": 100,
        "elapsed_s": 1.2,
        "timed_out": 0,
        "errors": 0,
    }), encoding="utf-8")
    (candidate / "output_manifest.json").write_text(
        '{"artifacts": [{"type": "score", "path": "scores.json"}]}',
        encoding="utf-8",
    )

    build_registry().validate("score", None, candidate)


def test_score_validator_rejects_worker_errors(tmp_path: Path):
    candidate = tmp_path / "score"
    candidate.mkdir()
    (candidate / "scores.json").write_text(json.dumps({
        "mean": 0.0,
        "median": 0.0,
        "std": 0.0,
        "min": 0,
        "max": 0,
        "p25": 0.0,
        "p75": 0.0,
        "episodes": 4000,
        "fights_won": [0] * 4000,
        "total_steps": 0,
        "elapsed_s": 2.7,
        "timed_out": 0,
        "errors": 4000,
    }), encoding="utf-8")
    (candidate / "output_manifest.json").write_text(
        '{"artifacts": [{"type": "score", "path": "scores.json"}]}',
        encoding="utf-8",
    )

    with pytest.raises(ArtifactValidationError, match="errors"):
        build_registry().validate("score", None, candidate)
