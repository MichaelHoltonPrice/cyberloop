from __future__ import annotations

import importlib.util
import json
import urllib.error
from pathlib import Path

import pytest


CYBERLOOP_ROOT = Path(__file__).resolve().parent.parent
CONTROLLER_PATH = CYBERLOOP_ROOT / "docker" / "cua_controller_io.py"
ENTRYPOINT_PATH = CYBERLOOP_ROOT / "docker" / "cua_controller_entrypoint.sh"


def _load_controller(tmp_path, monkeypatch):
    spec = importlib.util.spec_from_file_location(
        "cua_controller_io", CONTROLLER_PATH)
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    monkeypatch.setattr(module, "INPUT_SAVE", tmp_path / "input" / "save")
    monkeypatch.setattr(module, "OUTPUT_STATE", tmp_path / "output_state")
    monkeypatch.setattr(module, "OUTPUT_TRACE", tmp_path / "output_trace")
    return module


def test_preload_uploads_input_save_and_waits_for_running(tmp_path, monkeypatch):
    controller = _load_controller(tmp_path, monkeypatch)
    controller.INPUT_SAVE.parent.mkdir(parents=True)
    controller.INPUT_SAVE.write_bytes(b"SAVE")
    calls = []

    def fake_request(method, path, data=None):
        calls.append((method, path, data))
        if method == "GET" and path == "/health":
            return json.dumps({
                "app": {"status": "running"},
            }).encode("utf-8")
        return b"{}"

    monkeypatch.setattr(controller, "_request", fake_request)

    controller.preload()

    assert ("POST", "/files/decker_state/load.save", b"SAVE") in calls
    assert any(call[0] == "POST" and call[1] == "/reset" for call in calls)
    reset_payload = json.loads(
        next(call[2] for call in calls if call[1] == "/reset").decode("utf-8")
    )
    assert reset_payload["wait_for_running"] is True


def test_preload_deletes_stale_save_when_no_input(tmp_path, monkeypatch):
    controller = _load_controller(tmp_path, monkeypatch)
    calls = []

    def fake_request(method, path, data=None):
        calls.append((method, path, data))
        if method == "GET" and path == "/health":
            return json.dumps({
                "app": {"status": "running"},
            }).encode("utf-8")
        return b"{}"

    monkeypatch.setattr(controller, "_request", fake_request)

    controller.preload()

    assert ("DELETE", "/files/decker_state/load.save", None) in calls


def test_preload_writes_desktop_unreachable_artifacts(tmp_path, monkeypatch):
    controller = _load_controller(tmp_path, monkeypatch)

    def fake_health(**_kwargs):
        raise RuntimeError("desktop unavailable")

    monkeypatch.setattr(controller, "_wait_for_health", fake_health)

    with pytest.raises(RuntimeError):
        controller.preload()

    error = json.loads(
        (controller.OUTPUT_TRACE / "desktop_unreachable.json").read_text(
            encoding="utf-8")
    )
    assert error["phase"] == "preload"
    assert "desktop unavailable" in error["message"]
    assert (controller.OUTPUT_STATE / "missing.json").is_file()


def test_export_records_error_when_desktop_unreachable(tmp_path, monkeypatch):
    controller = _load_controller(tmp_path, monkeypatch)

    def fake_health(**_kwargs):
        raise RuntimeError("desktop unavailable")

    monkeypatch.setattr(controller, "_wait_for_health", fake_health)

    controller.export()

    error = json.loads(
        (controller.OUTPUT_TRACE / "export_error.json").read_text(
            encoding="utf-8")
    )
    assert error["phase"] == "export_health"
    assert (controller.OUTPUT_STATE / "missing.json").is_file()


def test_export_writes_health_even_when_state_files_are_missing(
        tmp_path, monkeypatch):
    controller = _load_controller(tmp_path, monkeypatch)

    def fake_request(method, path, data=None):
        if method == "GET" and path == "/health":
            return json.dumps({"app": {"status": "running"}}).encode("utf-8")
        if method == "GET" and path == "/screenshot?format=png":
            return b"PNG"
        raise urllib.error.HTTPError(
            url=path,
            code=404,
            msg="not found",
            hdrs=None,
            fp=None,
        )

    monkeypatch.setattr(controller, "_request", fake_request)

    controller.export()

    assert (controller.OUTPUT_TRACE / "desktop_health.json").is_file()
    assert (controller.OUTPUT_TRACE / "final.png").read_bytes() == b"PNG"
    assert (controller.OUTPUT_TRACE / "export_error.json").is_file()
    assert (controller.OUTPUT_STATE / "missing.json").is_file()


def test_entrypoint_preserves_agent_rc_when_export_fails():
    script = ENTRYPOINT_PATH.read_text(encoding="utf-8")
    after_agent = script.split("/app/entrypoint.sh", 1)[1]

    assert "RC=$?" in after_agent
    assert "EXPORT_RC=$?" in after_agent
    assert 'exit "$RC"' in after_agent
    assert "set -e" not in after_agent
