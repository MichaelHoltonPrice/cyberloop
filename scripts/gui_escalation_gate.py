#!/usr/bin/env python3
"""Request GUI-action-code escalation when the latest game step failed."""

from __future__ import annotations

import json
import os
from pathlib import Path
from typing import Any


GAME_STEP_HISTORY = Path("/input/game_step_history")
OUTPUT_PATH = Path("/output/escalation_request/escalation_request.json")
TERMINATION_PATH = Path("/flywheel/termination")


def main() -> int:
    interval = int(os.environ.get("ESCALATION_INTERVAL", "0"))
    latest, step_count = _latest_step(GAME_STEP_HISTORY)
    request = _build_request(latest, interval, step_count)
    if request is None:
        TERMINATION_PATH.write_text("normal\n", encoding="utf-8")
        return 0
    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    OUTPUT_PATH.write_text(
        json.dumps(request, indent=2, sort_keys=True),
        encoding="utf-8",
    )
    TERMINATION_PATH.write_text("escalation_requested\n", encoding="utf-8")
    return 0


def _build_request(
    step: dict[str, Any] | None,
    interval: int,
    step_count: int,
) -> dict[str, Any] | None:
    """Build an escalation request.

    ``step_count`` is the length of the staged sequence manifest, including
    the step that just completed. An interval of 3 therefore reviews steps
    3, 6, 9, and so on.
    """
    if step is None:
        return {
            "reason": "missing_game_step",
            "message": "No game_step entry was found in the lane history.",
        }
    if step.get("schema_version") != 1:
        return {
            "reason": "unsupported_game_step_schema",
            "message": (
                "Expected game_step schema_version 1, got "
                f"{step.get('schema_version')!r}."
            ),
            "step_count": step_count,
        }
    status = step.get("status")
    prediction_correct = step.get("prediction_correct")
    action_index = step.get("action_index")
    if status == "execution_error" or step.get("prediction_error"):
        reason = "execution_error"
    # action_taken steps are expected to set prediction_correct explicitly.
    elif prediction_correct is False:
        reason = "prediction_mismatch"
    elif interval > 0 and step_count > 0 and step_count % interval == 0:
        reason = "periodic_review"
    else:
        return None
    return {
        "reason": reason,
        "status": status,
        "action_index": action_index,
        "step_count": step_count,
        "action_label": step.get("action_label"),
        "parsed_action": step.get("parsed_action"),
        "prediction_correct": prediction_correct,
        "comparison": step.get("comparison"),
        "prediction_error": step.get("prediction_error"),
        "gui_decider_sha256": step.get("gui_decider_sha256"),
        "bot_sha256": step.get("bot_sha256"),
    }


def _latest_step(history_root: Path) -> tuple[dict[str, Any] | None, int]:
    manifest_path = history_root / "manifest.json"
    if not manifest_path.is_file():
        return None, 0
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    entries = manifest.get("entries", [])
    if not isinstance(entries, list):
        return None, 0
    for entry in reversed(entries):
        if not isinstance(entry, dict):
            continue
        directory = entry.get("directory")
        if not isinstance(directory, str):
            continue
        step_path = history_root / directory / "game_step.json"
        if step_path.is_file():
            data = json.loads(step_path.read_text(encoding="utf-8"))
            return data if isinstance(data, dict) else None, len(entries)
    return None, len(entries)


if __name__ == "__main__":
    raise SystemExit(main())
