#!/usr/bin/env python3
"""Execute one bot-selected Decker action through the GUI.

This block treats the Rust engine as the transition oracle and the desktop
as the frontend under test. It predicts the post-action state by applying
the selected semantic action to a cloned engine save, executes GUI actions
through editable Python code, exports the actual GUI-backed save, and writes
a game_step artifact describing the comparison.
"""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import sys
import time
import traceback
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any

import decker_pyenv as decker

from action_parser import parse_action_label
from gui_state_compare import compare_observations, normalize_observation


DESKTOP_URL = os.environ.get("DESKTOP_URL", "http://decker-desktop:8080").rstrip("/")
SUBCLASS = os.environ.get("SUBCLASS", "dueling")
RACE = os.environ.get("RACE", "human")
BACKGROUND = os.environ.get("BACKGROUND", "soldier")
SEED = int(os.environ.get("SEED", "1"))
POST_ACTION_WAIT_MS = int(os.environ.get("POST_ACTION_WAIT_MS", "800"))
SNAP_TO_PREDICTION_ON_MISMATCH = (
    os.environ.get("SNAP_TO_PREDICTION_ON_MISMATCH", "false").lower()
    in {"1", "true", "yes", "on"}
)

INPUT_STATE = Path("/input/decker_state/decker_state.save")
BOT_PATH = Path("/input/bot/bot.py")
GUI_DECIDER_DIR = Path("/input/gui_action_decider")

OUTPUT_STATE = Path("/output/decker_state")
OUTPUT_STEP = Path("/output/game_step")
OUTPUT_TRACE = Path("/output/cua_trace")
TERMINATION_PATH = Path("/flywheel/termination")


class DesktopError(RuntimeError):
    """Raised when the desktop service cannot be reached or used."""


def main() -> int:
    _ensure_outputs()
    started_at = time.time()
    env = _load_or_create_env()
    current_save = env.save()
    current_obs = json.loads(env.observe())

    try:
        _load_save_into_desktop(current_save)
    except Exception as exc:  # noqa: BLE001
        _write_current_state(env)
        _write_trace_json("desktop_unreachable.json", _error_payload(exc))
        _write_game_step({
            "schema_version": 1,
            "status": "desktop_unreachable",
            "error": _error_payload(exc),
            "pre_observation": normalize_observation(current_obs),
            "elapsed_s": time.time() - started_at,
        })
        _terminate("desktop_unreachable")
        return 0

    labels = env.legal_action_labels()
    if not labels or env.is_done:
        _write_current_state(env)
        _write_game_step({
            "schema_version": 1,
            "status": "game_over",
            "pre_observation": normalize_observation(current_obs),
            "legal_action_labels": labels,
            "elapsed_s": time.time() - started_at,
        })
        _terminate("game_over")
        return 0

    try:
        action_index = _choose_action(env, current_obs, labels)
    except Exception as exc:  # noqa: BLE001
        error = _error_payload(exc)
        _write_current_state(env)
        _write_trace_json("bot_error.json", error)
        _write_game_step({
            "schema_version": 1,
            "status": "bot_error",
            "error": error,
            "pre_observation": normalize_observation(current_obs),
            "legal_action_labels": labels,
            "elapsed_s": time.time() - started_at,
        })
        _terminate("bot_error")
        return 0
    action_label = labels[action_index]
    parsed_action = parse_action_label(action_label)
    prediction_env = decker.GauntletEnv.from_save(current_save)
    predicted_obs_json, reward, done = prediction_env.step(action_index)
    predicted_obs = json.loads(predicted_obs_json)
    predicted_save = prediction_env.save()
    predicted_hash = _sha256_text(predicted_save)

    pre_screenshot = OUTPUT_TRACE / "screenshots" / "pre_action.png"
    post_screenshot = OUTPUT_TRACE / "screenshots" / "post_action.png"
    execution_error: dict[str, Any] | None = None
    gui_operations: list[dict[str, Any]] = []
    try:
        _download_screenshot(pre_screenshot)
        context = {
            "action_index": action_index,
            "action_label": action_label,
            "parsed_action": list(parsed_action),
            "observation": current_obs,
            "legal_action_labels": labels,
            "screenshot_path": str(pre_screenshot),
            "desktop": {"width": 1280, "height": 720},
        }
        gui_operations = _plan_gui_operations(context)
        _apply_gui_operations(gui_operations, OUTPUT_TRACE / "screenshots")
        _post("/wait", {"duration_ms": POST_ACTION_WAIT_MS})
        _download_screenshot(post_screenshot)
    except Exception as exc:  # noqa: BLE001
        execution_error = _error_payload(exc)
        _write_trace_json("execution_error.json", execution_error)

    try:
        actual_save, actual_env = _read_valid_desktop_save(
            retries=12, delay_s=0.25)
    except Exception as exc:  # noqa: BLE001
        if execution_error is None:
            execution_error = _error_payload(exc)
            _write_trace_json("execution_error.json", execution_error)
        actual_save = current_save.encode("utf-8")
        actual_env = decker.GauntletEnv.from_save(current_save)
    actual_obs = json.loads(actual_env.observe())
    comparison = compare_observations(predicted_obs, actual_obs)
    state_to_write = actual_save
    env_to_write = actual_env
    decker_state_source = "actual"
    if (
        SNAP_TO_PREDICTION_ON_MISMATCH
        and execution_error is None
        and not comparison["equal"]
    ):
        state_to_write = predicted_save.encode("utf-8")
        env_to_write = decker.GauntletEnv.from_save(predicted_save)
        decker_state_source = "prediction"
    _write_exported_state(state_to_write, env_to_write)

    trace = {
        "schema_version": 1,
        "desktop_url": DESKTOP_URL,
        "gui_decider_sha256": _hash_tree(GUI_DECIDER_DIR),
        "bot_sha256": _sha256_bytes(BOT_PATH.read_bytes()),
        "action_index": action_index,
        "action_label": action_label,
        "parsed_action": list(parsed_action),
        "operations": gui_operations,
        "prediction_reward": reward,
        "prediction_done": done,
        "prediction_save_sha256": predicted_hash,
        "comparison": comparison,
        "execution_error": execution_error,
        "decker_state_source": decker_state_source,
    }
    _write_trace_json("step_trace.json", trace)

    status = (
        "execution_error"
        if execution_error is not None
        else "prediction_mismatch" if not comparison["equal"]
        else "action_taken"
    )
    _write_game_step({
        "schema_version": 1,
        "status": status,
        "action_index": action_index,
        "action_label": action_label,
        "parsed_action": list(parsed_action),
        "prediction_correct": comparison["equal"] and execution_error is None,
        "prediction_error": execution_error,
        "comparison": comparison,
        "pre_observation": normalize_observation(current_obs),
        "predicted_observation": normalize_observation(predicted_obs),
        "actual_observation": normalize_observation(actual_obs),
        "predicted_save_sha256": predicted_hash,
        "actual_save_sha256": _sha256_bytes(actual_save),
        "decker_state_source": decker_state_source,
        "gui_decider_sha256": trace["gui_decider_sha256"],
        "bot_sha256": trace["bot_sha256"],
        "elapsed_s": time.time() - started_at,
    })
    _terminate("action_taken")
    return 0


def _ensure_outputs() -> None:
    for path in (OUTPUT_STATE, OUTPUT_STEP, OUTPUT_TRACE / "screenshots"):
        path.mkdir(parents=True, exist_ok=True)


def _load_or_create_env():
    if INPUT_STATE.is_file():
        return decker.GauntletEnv.from_save(INPUT_STATE.read_text(encoding="utf-8"))
    return decker.GauntletEnv(SUBCLASS, SEED, RACE, background=BACKGROUND)


def _choose_action(env: Any, observation: dict[str, Any], labels: list[str]) -> int:
    module = _load_module(BOT_PATH, "cyberloop_bot")
    player_fn = getattr(module, "player_fn", None)
    if not callable(player_fn):
        raise RuntimeError(f"{BOT_PATH} does not define callable player_fn")
    chosen = int(player_fn(env, json.dumps(observation), labels))
    if chosen < 0 or chosen >= len(labels):
        raise RuntimeError(
            f"bot chose action index {chosen}, but {len(labels)} actions are legal")
    return chosen


def _plan_gui_operations(context: dict[str, Any]) -> list[dict[str, Any]]:
    path = _find_gui_decider(GUI_DECIDER_DIR)
    module = _load_module(path, "gui_action_decider")
    planner = getattr(module, "plan_actions", None)
    if not callable(planner):
        raise RuntimeError(f"{path} does not define callable plan_actions")
    operations = planner(context)
    if not isinstance(operations, list):
        raise RuntimeError("plan_actions must return a list of operation dicts")
    result = []
    for op in operations:
        if not isinstance(op, dict):
            raise RuntimeError("GUI operation must be a dict")
        result.append(dict(op))
    return result


def _apply_gui_operations(
    operations: list[dict[str, Any]],
    screenshot_dir: Path,
) -> None:
    for index, op in enumerate(operations, start=1):
        op_type = op.get("type")
        if op_type == "click":
            _post("/click", {
                "x": int(op["x"]),
                "y": int(op["y"]),
                "button": op.get("button", "left"),
                "double": bool(op.get("double", False)),
            })
        elif op_type == "wait":
            _post("/wait", {"duration_ms": int(op.get("duration_ms", 250))})
        elif op_type == "key":
            _post("/key", {
                "key": str(op["key"]),
                "modifiers": list(op.get("modifiers", [])),
            })
        elif op_type == "type":
            _post("/type", {"text": str(op["text"])})
        elif op_type == "move":
            _post("/move", {"x": int(op["x"]), "y": int(op["y"])})
        elif op_type == "drag":
            _post("/drag", {
                "from": list(op["from"]),
                "to": list(op["to"]),
                "duration_ms": int(op.get("duration_ms", 250)),
            })
        else:
            raise RuntimeError(f"unknown GUI operation type {op_type!r}")
        _download_screenshot(screenshot_dir / f"after_op_{index:03d}.png")


def _load_save_into_desktop(save: str) -> None:
    health = _wait_for_health()
    try:
        existing = _read_desktop_file("decker_state/state.save")
        if (
            _sha256_bytes(existing) == _sha256_text(save)
            and health.get("app", {}).get("status") == "running"
        ):
            return
    except Exception:
        pass
    _request("POST", "/files/decker_state/load.save", save.encode("utf-8"))
    _post("/reset", {
        "wait_for_running": True,
        "render_delay_ms": 1500,
        "timeout_s": 60.0,
    })
    _wait_for_health(require_app_running=True)


def _wait_for_health(
    *,
    require_app_running: bool = False,
    timeout_s: float = 60.0,
) -> dict[str, Any]:
    deadline = time.monotonic() + timeout_s
    last_error: BaseException | None = None
    while time.monotonic() < deadline:
        try:
            payload = json.loads(_request("GET", "/health").decode("utf-8"))
            if not require_app_running:
                return payload
            if payload.get("app", {}).get("status") == "running":
                return payload
            last_error = RuntimeError(
                f"desktop app status is {payload.get('app')!r}")
        except Exception as exc:  # noqa: BLE001
            last_error = exc
        time.sleep(0.5)
    raise DesktopError(f"desktop did not become healthy: {last_error}")


def _download_screenshot(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(_request("GET", "/screenshot?format=png"))


def _read_desktop_file(
    remote: str,
    *,
    retries: int = 1,
    delay_s: float = 0.0,
) -> bytes:
    last_error: BaseException | None = None
    quoted = urllib.parse.quote(remote)
    for _ in range(max(retries, 1)):
        try:
            return _request("GET", f"/files/{quoted}")
        except Exception as exc:  # noqa: BLE001
            last_error = exc
            if delay_s:
                time.sleep(delay_s)
    raise DesktopError(f"could not read desktop file {remote!r}: {last_error}")


def _read_valid_desktop_save(
    *,
    retries: int,
    delay_s: float,
) -> tuple[bytes, Any]:
    last_error: BaseException | None = None
    for _ in range(max(retries, 1)):
        try:
            data = _read_desktop_file("decker_state/state.save")
            json.loads(data.decode("utf-8"))
            env = decker.GauntletEnv.from_save(data.decode("utf-8"))
            return data, env
        except Exception as exc:  # noqa: BLE001
            last_error = exc
            time.sleep(delay_s)
    raise DesktopError(f"desktop save never became readable: {last_error}")


def _request(method: str, path: str, data: bytes | None = None) -> bytes:
    request = urllib.request.Request(
        f"{DESKTOP_URL}{path}",
        data=data,
        method=method,
        headers={"Content-Type": "application/json"} if data else {},
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            return response.read()
    except urllib.error.URLError as exc:
        raise DesktopError(str(exc)) from exc


def _post(path: str, payload: dict[str, Any]) -> bytes:
    return _request("POST", path, json.dumps(payload).encode("utf-8"))


def _write_exported_state(save: bytes, env: Any) -> None:
    OUTPUT_STATE.mkdir(parents=True, exist_ok=True)
    (OUTPUT_STATE / "decker_state.save").write_bytes(save)
    (OUTPUT_STATE / "state.json").write_text(env.observe(), encoding="utf-8")
    (OUTPUT_STATE / "actions.json").write_text(
        json.dumps(env.legal_action_labels(), indent=2),
        encoding="utf-8",
    )


def _write_current_state(env: Any) -> None:
    save = env.save()
    _write_exported_state(save.encode("utf-8"), env)


def _write_game_step(payload: dict[str, Any]) -> None:
    OUTPUT_STEP.mkdir(parents=True, exist_ok=True)
    (OUTPUT_STEP / "game_step.json").write_text(
        json.dumps(payload, indent=2, sort_keys=True),
        encoding="utf-8",
    )


def _write_trace_json(name: str, payload: dict[str, Any]) -> None:
    OUTPUT_TRACE.mkdir(parents=True, exist_ok=True)
    (OUTPUT_TRACE / name).write_text(
        json.dumps(payload, indent=2, sort_keys=True),
        encoding="utf-8",
    )


def _terminate(reason: str) -> None:
    TERMINATION_PATH.parent.mkdir(parents=True, exist_ok=True)
    TERMINATION_PATH.write_text(f"{reason}\n", encoding="utf-8")


def _find_gui_decider(root: Path) -> Path:
    preferred = root / "gui_action_decider.py"
    if preferred.is_file():
        return preferred
    candidates = sorted(root.glob("*.py"))
    if candidates:
        return candidates[0]
    raise RuntimeError(f"no GUI action decider .py file found in {root}")


def _load_module(path: Path, name: str):
    spec = importlib.util.spec_from_file_location(name, path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"could not load module from {path}")
    module = importlib.util.module_from_spec(spec)
    sys.modules[name] = module
    spec.loader.exec_module(module)
    return module


def _hash_tree(root: Path) -> str:
    digest = hashlib.sha256()
    for path in sorted(p for p in root.rglob("*") if p.is_file()):
        digest.update(path.relative_to(root).as_posix().encode("utf-8"))
        digest.update(b"\0")
        digest.update(path.read_bytes())
        digest.update(b"\0")
    return digest.hexdigest()


def _sha256_text(value: str) -> str:
    return _sha256_bytes(value.encode("utf-8"))


def _sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def _error_payload(exc: BaseException) -> dict[str, Any]:
    return {
        "kind": type(exc).__name__,
        "message": str(exc),
        "traceback": traceback.format_exc(limit=8),
    }


if __name__ == "__main__":
    raise SystemExit(main())
