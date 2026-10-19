"""Shared fixtures. Snapshots are plain: ``NO_COLOR`` is set before anything detects."""

from __future__ import annotations

import json
import os
from pathlib import Path

import pytest

os.environ["NO_COLOR"] = "1"

from supremus import Frame  # noqa: E402

GOLDENS = Path(__file__).resolve().parents[2] / "tests" / "fixtures" / "js" / "goldens.json"


@pytest.fixture(scope="session")
def goldens() -> list[dict]:
    if not GOLDENS.exists():
        pytest.skip("shared goldens live in the repository checkout")
    return json.loads(GOLDENS.read_text(encoding="utf-8"))


@pytest.fixture(scope="session")
def golden(goldens: list[dict]):
    by_name = {g["name"]: g for g in goldens}

    def lookup(name: str) -> dict:
        return by_name[name]

    return lookup


@pytest.fixture
def plain() -> Frame:
    return Frame.plain(40, 10)


@pytest.fixture
def portable() -> Frame:
    return Frame.portable(40, 10)


@pytest.fixture
def env(monkeypatch: pytest.MonkeyPatch) -> dict[str, str]:
    """A clean environment mapping for detection tests."""
    return {}


class Pipe:
    """A stream that is not a terminal."""

    def isatty(self) -> bool:
        return False

    def fileno(self) -> int:
        raise OSError("no fd")


class Tty(Pipe):
    def isatty(self) -> bool:
        return True


@pytest.fixture
def pipe() -> Pipe:
    return Pipe()


@pytest.fixture
def tty() -> Tty:
    return Tty()
