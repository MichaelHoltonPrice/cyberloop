#!/usr/bin/env python3
"""Cyberloop CUA tools for driving the desktop service."""

from __future__ import annotations

import json
import os
import time
import urllib.parse
import urllib.request
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

from mcp.server.fastmcp import FastMCP


mcp = FastMCP("cyberloop_cua")

DESKTOP_URL = os.environ.get("DESKTOP_URL", "http://decker-desktop:8080").rstrip("/")
SCREENSHOT_DIR = Path(os.environ.get(
    "CUA_SCREENSHOT_DIR", "/scratch/cua_screenshots"))


def _trace_dir() -> Path:
    if "CUA_TRACE_DIR" in os.environ:
        return Path(os.environ["CUA_TRACE_DIR"])
    if SCREENSHOT_DIR.name == "screenshots":
        return SCREENSHOT_DIR.parent
    return SCREENSHOT_DIR


def _record_action(action: str, payload: dict[str, Any]) -> None:
    trace_dir = _trace_dir()
    trace_dir.mkdir(parents=True, exist_ok=True)
    event = {
        "ts": datetime.now(UTC).isoformat(),
        "action": action,
        "payload": payload,
    }
    with (trace_dir / "actions.jsonl").open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(event, sort_keys=True) + "\n")


def _json_request(method: str, path: str, payload: dict[str, Any] | None = None) -> Any:
    data = None if payload is None else json.dumps(payload).encode("utf-8")
    request = urllib.request.Request(
        f"{DESKTOP_URL}{path}",
        data=data,
        method=method,
        headers={"Content-Type": "application/json"} if data is not None else {},
    )
    with urllib.request.urlopen(request, timeout=30) as response:
        raw = response.read()
    if not raw:
        return None
    return json.loads(raw.decode("utf-8"))


@mcp.tool()
def screenshot(format: str = "png") -> str:
    """Capture the desktop and save it in the CUA trace directory."""
    fmt = "png" if format.lower() == "png" else "jpeg"
    SCREENSHOT_DIR.mkdir(parents=True, exist_ok=True)
    suffix = "png" if fmt == "png" else "jpg"
    path = SCREENSHOT_DIR / f"screenshot_{int(time.time() * 1000)}.{suffix}"
    params = {"format": fmt}
    if fmt == "jpeg":
        params["quality"] = "85"
    query = urllib.parse.urlencode(params)
    request = urllib.request.Request(f"{DESKTOP_URL}/screenshot?{query}")
    with urllib.request.urlopen(request, timeout=30) as response:
        path.write_bytes(response.read())
    _record_action("screenshot", {"path": str(path), "format": fmt})
    return (
        f"Screenshot saved to {path}. "
        "Use the Read tool on that image path before deciding the next action."
    )


@mcp.tool()
def click(x: int, y: int, button: str = "left", double: bool = False) -> str:
    """Click a window-relative coordinate."""
    payload = {
        "x": x,
        "y": y,
        "button": button,
        "double": double,
    }
    _record_action("click", payload)
    _json_request("POST", "/click", {
        "x": x,
        "y": y,
        "button": button,
        "double": double,
    })
    return f"Clicked {button} at ({x}, {y})."


@mcp.tool()
def move(x: int, y: int) -> str:
    """Move the pointer to a window-relative coordinate."""
    payload = {"x": x, "y": y}
    _record_action("move", payload)
    _json_request("POST", "/move", payload)
    return f"Moved pointer to ({x}, {y})."


@mcp.tool()
def type_text(text: str) -> str:
    """Type text into the active desktop window."""
    payload = {"text": text}
    _record_action("type_text", {"length": len(text)})
    _json_request("POST", "/type", payload)
    return "Typed text."


@mcp.tool()
def key(key: str, modifiers: list[str] | None = None) -> str:
    """Press a key, optionally with modifiers such as ctrl or shift."""
    payload = {
        "key": key,
        "modifiers": modifiers or [],
    }
    _record_action("key", payload)
    _json_request("POST", "/key", payload)
    return f"Pressed {key}."


@mcp.tool()
def drag(
    from_x: int,
    from_y: int,
    to_x: int,
    to_y: int,
    duration_ms: int = 250,
) -> str:
    """Drag from one window-relative coordinate to another."""
    payload = {
        "from": [from_x, from_y],
        "to": [to_x, to_y],
        "duration_ms": duration_ms,
    }
    _record_action("drag", payload)
    _json_request("POST", "/drag", payload)
    return f"Dragged from ({from_x}, {from_y}) to ({to_x}, {to_y})."


@mcp.tool()
def wait(duration_ms: int = 500) -> str:
    """Wait for the desktop to update."""
    payload = {"duration_ms": duration_ms}
    _record_action("wait", payload)
    _json_request("POST", "/wait", payload)
    return f"Waited {duration_ms} ms."


@mcp.tool()
def finish_segment() -> str:
    """End the current CUA segment so Flywheel can export state."""
    _record_action("finish_segment", {})
    return "Segment complete."


if __name__ == "__main__":
    mcp.run()
