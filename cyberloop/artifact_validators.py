"""Cyberloop artifact validators wired into Flywheel."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from flywheel.artifact_validator import (
    ArtifactValidationError,
    ArtifactValidatorRegistry,
)
from flywheel.template import ArtifactDeclaration


def build_registry() -> ArtifactValidatorRegistry:
    """Return Cyberloop's Flywheel artifact validator registry."""
    registry = ArtifactValidatorRegistry()
    registry.register("checkpoint", validate_checkpoint)
    registry.register("score", validate_score)
    return registry


def validate_checkpoint(
    name: str,
    declaration: ArtifactDeclaration,
    staged_path: Path,
) -> None:
    """Validate a Cyberloop checkpoint artifact candidate."""
    del name, declaration
    checkpoint = staged_path / "checkpoint.pt"
    run_json = staged_path / "run.json"
    manifest = staged_path / "output_manifest.json"

    _require_file(checkpoint, "checkpoint.pt")
    if checkpoint.stat().st_size == 0:
        raise ArtifactValidationError("checkpoint.pt is empty")
    run_info = _load_json_file(run_json, "run.json")
    _load_json_file(manifest, "output_manifest.json")

    _require_equal(run_info, "schema_version", 1)
    _require_equal(
        run_info, "artifact_type", "cyberloop.checkpoint")
    _require_equal(run_info, "model_path", "checkpoint.pt")
    _require_non_empty_str(run_info, "stage")
    _require_non_empty_str(run_info, "subclass")
    _require_non_empty_str(run_info, "race")
    _require_int(run_info, "seed")
    _require_positive_int(run_info, "timesteps")


def validate_score(
    name: str,
    declaration: ArtifactDeclaration,
    staged_path: Path,
) -> None:
    """Validate a Cyberloop score artifact candidate."""
    del name, declaration
    scores = _load_json_file(staged_path / "scores.json", "scores.json")
    _load_json_file(staged_path / "output_manifest.json", "output_manifest.json")

    episodes = _require_positive_int(scores, "episodes")
    fights_won = scores.get("fights_won")
    if not isinstance(fights_won, list):
        raise ArtifactValidationError("scores.json fights_won must be a list")
    if len(fights_won) != episodes:
        raise ArtifactValidationError(
            "scores.json fights_won length must match episodes")
    _require_equal(scores, "errors", 0)
    total_steps = _require_int(scores, "total_steps")
    if total_steps <= 0:
        raise ArtifactValidationError("scores.json total_steps must be positive")
    _require_int(scores, "timed_out")
    for key in ("mean", "median", "std", "min", "max", "p25", "p75"):
        value = scores.get(key)
        if not isinstance(value, int | float):
            raise ArtifactValidationError(
                f"scores.json {key} must be numeric")


def _require_file(path: Path, label: str) -> None:
    if not path.is_file():
        raise ArtifactValidationError(f"{label} is required")


def _load_json_file(path: Path, label: str) -> dict[str, Any]:
    _require_file(path, label)
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        raise ArtifactValidationError(
            f"{label} is invalid JSON: {exc}") from exc
    if not isinstance(value, dict):
        raise ArtifactValidationError(f"{label} must contain a JSON object")
    return value


def _require_equal(
    data: dict[str, Any],
    key: str,
    expected: Any,
) -> Any:
    value = data.get(key)
    if value != expected:
        raise ArtifactValidationError(
            f"{key} must be {expected!r}, got {value!r}")
    return value


def _require_non_empty_str(data: dict[str, Any], key: str) -> str:
    value = data.get(key)
    if not isinstance(value, str) or not value:
        raise ArtifactValidationError(f"{key} must be a non-empty string")
    return value


def _require_int(data: dict[str, Any], key: str) -> int:
    value = data.get(key)
    if not isinstance(value, int):
        raise ArtifactValidationError(f"{key} must be an integer")
    return value


def _require_positive_int(data: dict[str, Any], key: str) -> int:
    value = _require_int(data, key)
    if value <= 0:
        raise ArtifactValidationError(f"{key} must be positive")
    return value
