You improve Cyberloop's GUI action decider.

The deterministic play block has already chosen a legal semantic game action
with the Rust engine and predicted the post-action state. The current
`gui_action_decider.py` mapped that semantic action to desktop clicks/keys, but
the GUI-backed exported state did not match the engine prediction or execution
failed.

Inputs:

- `/input/gui_action_decider/gui_action_decider.py`: the current decider code.
- `/input/escalation_request/escalation_request.json`: concise failure reason.
- `/input/game_step/game_step.json`: action, prediction, actual observation,
  comparison diffs, and code hashes.
- `/input/cua_trace`: screenshots and operation trace from the failed segment.
- `/input/gui_step_trace_history/manifest.json`: ordered prior GUI traces.
- `/input/game_step_history/manifest.json`: ordered prior game-step records.

Task:

1. Copy `/input/gui_action_decider/gui_action_decider.py` to
   `/output/gui_action_decider/gui_action_decider.py`.
2. Inspect the game step, screenshots, and trace data.
3. Edit only `/output/gui_action_decider/gui_action_decider.py`.
4. Preserve the public interface:
   `plan_actions(context) -> list[dict]`.
5. Return operations using the desktop API operation dictionaries:
   `click`, `wait`, `key`, `type`, `move`, or `drag`.
6. Before finishing, run:
   `python /app/validate_gui_action_decider.py /output/gui_action_decider`

Do not modify `bot.py`. The bot chooses semantic engine actions; this code only
translates one semantic action into GUI operations. Keep the solution narrow and
grounded in the observed failure.

Operation schemas:

- `{"type": "click", "x": int, "y": int, "button": "left"|"right"|"middle", "double": bool}`
- `{"type": "wait", "duration_ms": int}`
- `{"type": "key", "key": str, "modifiers": list[str]}`
- `{"type": "type", "text": str}`
- `{"type": "move", "x": int, "y": int}`
- `{"type": "drag", "from": [x, y], "to": [x, y], "duration_ms": int}`

Look for repeated failure modes in the history manifests, not only the most
recent failure.
