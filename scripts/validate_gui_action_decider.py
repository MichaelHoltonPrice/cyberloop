#!/usr/bin/env python3
"""Validate a Cyberloop GUI action decider artifact."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path


def main(argv: list[str]) -> int:
    root = Path(argv[1]) if len(argv) > 1 else Path("/output/gui_action_decider")
    path = root / "gui_action_decider.py"
    if not path.is_file():
        print(f"missing {path}", file=sys.stderr)
        return 1
    spec = importlib.util.spec_from_file_location("gui_action_decider", path)
    if spec is None or spec.loader is None:
        print(f"could not import {path}", file=sys.stderr)
        return 1
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    plan_actions = getattr(module, "plan_actions", None)
    if not callable(plan_actions):
        print("gui_action_decider.py must define plan_actions(context)",
              file=sys.stderr)
        return 1
    context = {
        "action_index": 0,
        "action_label": "CombatAction(EndTurn)",
        "parsed_action": ["end_turn"],
        "observation": {
            "hand": [],
            "enemies": [],
        },
        "legal_action_labels": ["CombatAction(EndTurn)"],
        "screenshot_path": "/tmp/nonexistent.png",
        "desktop": {"width": 1280, "height": 720},
    }
    operations = plan_actions(context)
    if not isinstance(operations, list):
        print("plan_actions(context) must return a list", file=sys.stderr)
        return 1
    for index, operation in enumerate(operations):
        if not isinstance(operation, dict):
            print(f"operation {index} is not a dict", file=sys.stderr)
            return 1
        op_type = operation.get("type")
        if op_type not in {"click", "wait", "key", "type", "move", "drag"}:
            print(f"operation {index} has unknown type {op_type!r}",
                  file=sys.stderr)
            return 1
    print(f"validated {path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
