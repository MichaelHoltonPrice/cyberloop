"""State normalization and comparison for GUI/engine checks."""

from __future__ import annotations

from typing import Any


IGNORED_TOP_LEVEL_KEYS = {
    "generated_at",
    "timestamp",
    "desktop",
    "render",
    "window",
    "screenshot",
    "trace",
}


def normalize_observation(value: Any) -> Any:
    """Return a deterministic gameplay-state projection.

    The engine observation is already the right comparison surface for
    Cyberloop: it is what bots see and excludes desktop/session details.
    This helper keeps the contract explicit and leaves room to drop
    unstable metadata if future observation fields add any.
    """
    return _normalize(value, at_top=True)


def compare_observations(
    predicted: Any,
    actual: Any,
    *,
    max_diffs: int = 50,
) -> dict[str, Any]:
    """Compare two normalized observations and return a small diff payload."""
    left = normalize_observation(predicted)
    right = normalize_observation(actual)
    diffs: list[dict[str, Any]] = []
    _collect_diffs(left, right, "$", diffs, max_diffs)
    return {
        "equal": not diffs,
        "diffs": diffs,
        "diff_count": len(diffs),
        "truncated": len(diffs) >= max_diffs,
    }


def _normalize(value: Any, *, at_top: bool = False) -> Any:
    if isinstance(value, dict):
        result = {}
        for key in sorted(value):
            if at_top and key in IGNORED_TOP_LEVEL_KEYS:
                continue
            result[key] = _normalize(value[key])
        return result
    if isinstance(value, list):
        return [_normalize(item) for item in value]
    return value


def _collect_diffs(
    left: Any,
    right: Any,
    path: str,
    diffs: list[dict[str, Any]],
    max_diffs: int,
) -> None:
    if len(diffs) >= max_diffs:
        return
    if type(left) is not type(right):  # noqa: E721 - exact JSON type matters
        diffs.append({
            "path": path,
            "predicted": left,
            "actual": right,
            "reason": "type_mismatch",
        })
        return
    if isinstance(left, dict):
        keys = sorted(set(left) | set(right))
        for key in keys:
            if len(diffs) >= max_diffs:
                return
            if key not in left:
                diffs.append({
                    "path": f"{path}.{key}",
                    "predicted": None,
                    "actual": right[key],
                    "reason": "missing_predicted_key",
                })
            elif key not in right:
                diffs.append({
                    "path": f"{path}.{key}",
                    "predicted": left[key],
                    "actual": None,
                    "reason": "missing_actual_key",
                })
            else:
                _collect_diffs(left[key], right[key],
                               f"{path}.{key}", diffs, max_diffs)
        return
    if isinstance(left, list):
        if len(left) != len(right):
            diffs.append({
                "path": f"{path}.length",
                "predicted": len(left),
                "actual": len(right),
                "reason": "length_mismatch",
            })
        for index, (left_item, right_item) in enumerate(zip(left, right)):
            if len(diffs) >= max_diffs:
                return
            _collect_diffs(
                left_item, right_item, f"{path}[{index}]", diffs, max_diffs)
        return
    if left != right:
        diffs.append({
            "path": path,
            "predicted": left,
            "actual": right,
            "reason": "value_mismatch",
        })
