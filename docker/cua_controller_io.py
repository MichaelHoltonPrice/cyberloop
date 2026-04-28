#!/usr/bin/env python3
"""Move Cyberloop Decker state between Flywheel artifacts and desktop API."""

from __future__ import annotations

import json
import os
import sys
import time
import traceback
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path


DESKTOP_URL = os.environ.get("DESKTOP_URL", "http://decker-desktop:8080").rstrip("/")
INPUT_SAVE = Path("/input/decker_state/decker_state.save")
OUTPUT_STATE = Path("/output/decker_state")
OUTPUT_TRACE = Path("/output/cua_trace")


def _request(method: str, path: str, data: bytes | None = None) -> bytes:
    request = urllib.request.Request(
        f"{DESKTOP_URL}{path}",
        data=data,
        method=method,
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        return response.read()


def _ensure_output_dirs() -> None:
    OUTPUT_STATE.mkdir(parents=True, exist_ok=True)
    OUTPUT_TRACE.mkdir(parents=True, exist_ok=True)


def _write_trace_json(name: str, data: dict) -> None:
    _ensure_output_dirs()
    (OUTPUT_TRACE / name).write_text(
        json.dumps(data, indent=2, sort_keys=True),
        encoding="utf-8",
    )


def _write_state_missing(name: str, exc: BaseException) -> None:
    _ensure_output_dirs()
    (OUTPUT_STATE / name).write_text(
        json.dumps({
            "kind": type(exc).__name__,
            "message": str(exc),
        }, indent=2, sort_keys=True),
        encoding="utf-8",
    )


def _error_payload(exc: BaseException, *, phase: str) -> dict:
    return {
        "phase": phase,
        "desktop_url": DESKTOP_URL,
        "kind": type(exc).__name__,
        "message": str(exc),
        "traceback": traceback.format_exc(),
    }


def _wait_for_health(
    *,
    require_app_running: bool = False,
    timeout_s: float = 60.0,
) -> dict:
    deadline = time.monotonic() + timeout_s
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            raw = _request("GET", "/health")
            health = json.loads(raw.decode("utf-8"))
            if not require_app_running:
                return health
            if health.get("app", {}).get("status") == "running":
                return health
            last_error = RuntimeError(
                f"desktop app status is {health.get('app')!r}")
        except Exception as exc:  # noqa: BLE001
            last_error = exc
        time.sleep(0.5)
    raise RuntimeError(f"desktop did not become healthy: {last_error}")


def preload() -> None:
    _ensure_output_dirs()
    try:
        _wait_for_health()
        if INPUT_SAVE.is_file():
            data = INPUT_SAVE.read_bytes()
            _request("POST", "/files/decker_state/load.save", data=data)
        else:
            try:
                _request("DELETE", "/files/decker_state/load.save")
            except urllib.error.HTTPError as exc:
                if exc.code != 404:
                    raise
        _request(
            "POST",
            "/reset",
            data=json.dumps({
                "wait_for_running": True,
                "render_delay_ms": 1500,
                "timeout_s": 60.0,
            }).encode("utf-8"),
        )
        _wait_for_health(require_app_running=True)
    except Exception as exc:  # noqa: BLE001
        payload = _error_payload(exc, phase="preload")
        _write_trace_json("desktop_unreachable.json", payload)
        _write_state_missing("missing.json", exc)
        raise


def _download_file(remote: str, dest: Path) -> bool:
    try:
        data = _request("GET", f"/files/{urllib.parse.quote(remote)}")
    except urllib.error.HTTPError as exc:
        if exc.code == 404:
            return False
        raise
    dest.parent.mkdir(parents=True, exist_ok=True)
    dest.write_bytes(data)
    return True


def export() -> None:
    _ensure_output_dirs()
    try:
        health = _wait_for_health()
    except Exception as exc:  # noqa: BLE001
        _write_trace_json("export_error.json", _error_payload(
            exc, phase="export_health"))
        _write_state_missing("missing.json", exc)
        return

    missing: list[str] = []
    try:
        for remote, dest in [
            ("decker_state/state.json", OUTPUT_STATE / "state.json"),
            ("decker_state/state_actions.json", OUTPUT_STATE / "actions.json"),
            ("decker_state/state.save", OUTPUT_STATE / "decker_state.save"),
        ]:
            if not _download_file(remote, dest):
                missing.append(remote)
    except Exception as exc:  # noqa: BLE001
        _write_trace_json("export_error.json", _error_payload(
            exc, phase="export_state"))
        _write_state_missing("missing.json", exc)
        missing = []
    if missing:
        error = RuntimeError(f"missing desktop state files: {missing!r}")
        _write_trace_json("export_error.json", _error_payload(
            error, phase="export_state"))
        _write_state_missing("missing.json", error)

    try:
        screenshot = _request("GET", "/screenshot?format=png")
        (OUTPUT_TRACE / "final.png").write_bytes(screenshot)
    except Exception as exc:  # noqa: BLE001
        (OUTPUT_TRACE / "screenshot_error.txt").write_text(
            f"{type(exc).__name__}: {exc}\n", encoding="utf-8")

    try:
        (OUTPUT_TRACE / "desktop_health.json").write_text(
            json.dumps(health, indent=2, sort_keys=True),
            encoding="utf-8",
        )
    except Exception as exc:  # noqa: BLE001
        _write_trace_json("health_write_error.json", _error_payload(
            exc, phase="export_health_write"))


def main(argv: list[str]) -> int:
    if len(argv) != 2 or argv[1] not in {"preload", "export"}:
        print("usage: cua_controller_io.py preload|export", file=sys.stderr)
        return 2
    if argv[1] == "preload":
        preload()
    else:
        export()
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
