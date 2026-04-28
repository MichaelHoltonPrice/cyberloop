"""Tests for deterministic GUI play/escalation helpers."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path


CYBERLOOP_ROOT = Path(__file__).resolve().parent.parent


def _load_script(name: str):
    path = CYBERLOOP_ROOT / "scripts" / f"{name}.py"
    spec = importlib.util.spec_from_file_location(name, path)
    assert spec is not None and spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def test_normalized_observation_ignores_top_level_metadata():
    compare = _load_script("gui_state_compare")

    predicted = {
        "phase": "Combat",
        "player": {"hp": 10},
        "timestamp": "a",
        "desktop": {"window": "ignored"},
    }
    actual = {
        "desktop": {"window": "different"},
        "timestamp": "b",
        "player": {"hp": 10},
        "phase": "Combat",
    }

    result = compare.compare_observations(predicted, actual)

    assert result["equal"] is True
    assert result["diffs"] == []


def test_normalized_observation_reports_gameplay_diff():
    compare = _load_script("gui_state_compare")

    result = compare.compare_observations(
        {"player": {"hp": 10}},
        {"player": {"hp": 9}},
    )

    assert result["equal"] is False
    assert result["diffs"][0]["path"] == "$.player.hp"


def test_gui_escalation_gate_requests_on_prediction_mismatch(tmp_path, monkeypatch):
    gate = _load_script("gui_escalation_gate")
    history = tmp_path / "history"
    entry = history / "00000_game_step"
    entry.mkdir(parents=True)
    (history / "manifest.json").write_text(json.dumps({
        "entries": [{
            "index": 0,
            "role": "game_step",
            "directory": "00000_game_step",
        }],
    }), encoding="utf-8")
    (entry / "game_step.json").write_text(json.dumps({
        "schema_version": 1,
        "status": "prediction_mismatch",
        "action_index": 3,
        "action_label": "CombatAction(EndTurn)",
        "prediction_correct": False,
        "comparison": {"equal": False},
    }), encoding="utf-8")

    monkeypatch.setattr(gate, "GAME_STEP_HISTORY", history)
    step, step_count = gate._latest_step(history)
    request = gate._build_request(step, 0, step_count)

    assert request is not None
    assert request["reason"] == "prediction_mismatch"
    assert request["action_label"] == "CombatAction(EndTurn)"


def test_gui_escalation_gate_skips_matching_step(tmp_path):
    gate = _load_script("gui_escalation_gate")
    history = tmp_path / "history"
    entry = history / "00000_game_step"
    entry.mkdir(parents=True)
    (history / "manifest.json").write_text(json.dumps({
        "entries": [{
            "index": 0,
            "role": "game_step",
            "directory": "00000_game_step",
        }],
    }), encoding="utf-8")
    (entry / "game_step.json").write_text(json.dumps({
        "schema_version": 1,
        "status": "action_taken",
        "action_index": 1,
        "prediction_correct": True,
        "comparison": {"equal": True},
    }), encoding="utf-8")

    step, step_count = gate._latest_step(history)
    assert gate._build_request(step, 0, step_count) is None
