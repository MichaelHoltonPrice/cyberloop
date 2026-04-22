"""Test configuration for cyberloop."""

from __future__ import annotations

from pathlib import Path

import pytest


@pytest.fixture()
def project_setup(tmp_path: Path):
    """Fresh cyberloop project with workspace + template.

    Thin wrapper over
    :func:`tests._project_setup.build_project` so tests can
    grab the trio with a single fixture argument.  Use the
    underlying helper directly when a single test needs more
    than one independent project.
    """
    from tests._project_setup import build_project
    return build_project(tmp_path)
