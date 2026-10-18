"""Frame and Theme: where and how a drawing goes.

A frame is run state, not plot state. The same plot renders into many frames.
``Frame.plain`` and ``Frame.portable`` are deterministic; ``Frame.detect``
reads the environment and the stream, the way the crate's ``Frame::detect``
does, and it is the only function in this module that does. The native
module never reads the environment.
"""

from __future__ import annotations

import os
import sys
from collections.abc import Mapping
from dataclasses import dataclass, replace
from typing import Any, ClassVar, Literal

__all__ = ["Charset", "ColorMode", "Frame", "Theme"]

Charset = Literal["Ascii", "HalfBlocks", "Quadrants", "Sextants", "Octants", "Braille"]
ColorMode = Literal["Plain", "Ansi16", "Ansi256", "TrueColor"]

_CHARSETS: dict[str, Charset] = {
    "ascii": "Ascii",
    "half": "HalfBlocks",
    "halfblock": "HalfBlocks",
    "halfblocks": "HalfBlocks",
    "quad": "Quadrants",
    "quadrant": "Quadrants",
    "quadrants": "Quadrants",
    "sextant": "Sextants",
    "sextants": "Sextants",
    "octant": "Octants",
    "octants": "Octants",
    "braille": "Braille",
}


@dataclass(frozen=True)
class Theme:
    """The six series colors, in order. ``DARK`` is the default; ``LIGHT`` keeps
    yellow off a white background."""

    palette: tuple[str, str, str, str, str, str]

    DARK: ClassVar[Theme]
    LIGHT: ClassVar[Theme]

    @property
    def is_light(self) -> bool:
        return self == Theme.LIGHT

    @staticmethod
    def detect(env: Mapping[str, str] | None = None) -> Theme:
        """``COLORFGBG`` distinguishes a light background; everything else is dark."""
        variables = os.environ if env is None else env
        background = (variables.get("COLORFGBG") or "").split(";")[-1]
        return Theme.LIGHT if background in {"7", "15"} else Theme.DARK

    def to_json(self) -> dict[str, Any]:
        return {"palette": list(self.palette)}


Theme.DARK = Theme(("Cyan", "Yellow", "Green", "Magenta", "Blue", "Red"))
Theme.LIGHT = Theme(("Blue", "Red", "Green", "Magenta", "Cyan", "Black"))


@dataclass(frozen=True)
class Frame:
    """Size, charset, color, theme: where and how this drawing goes."""

    width: int
    height: int
    charset: Charset = "Quadrants"
    color: ColorMode = "Plain"
    theme: Theme = Theme.DARK

    def __post_init__(self) -> None:
        if self.width < 0 or self.height < 0:
            raise ValueError("a frame's width and height are non-negative")

    @staticmethod
    def plain(width: int, height: int) -> Frame:
        """Deterministic braille, no color: the snapshot frame."""
        return Frame(width, height, "Braille", "Plain", Theme.DARK)

    @staticmethod
    def portable(width: int, height: int) -> Frame:
        """Conservative Unicode: quadrants, no color."""
        return Frame(width, height, "Quadrants", "Plain", Theme.DARK)

    @staticmethod
    def detect(stream: Any = None, env: Mapping[str, str] | None = None) -> Frame:
        """Reads the environment and ``stream`` (default ``sys.stdout``).

        Size: the terminal's, a third of its height clamped to 8–24 rows;
        without a terminal, ``COLUMNS`` and ``LINES``; else 80×16. Charset:
        ``MALEVICH_CHARSET`` (or ``SUPREMUS_CHARSET``), then ASCII for a dumb
        ``TERM`` or a non-UTF locale, else quadrants. Color: ``NO_COLOR`` wins;
        a non-terminal is plain unless ``CLICOLOR_FORCE`` or ``FORCE_COLOR``;
        then ``TERM`` and ``COLORTERM`` pick the tier. A tool that plots to
        stderr passes ``sys.stderr``.
        """
        variables = os.environ if env is None else env
        target = sys.stdout if stream is None else stream
        width, height = _detect_size(target, variables)
        return Frame(
            width,
            height,
            _detect_charset(variables),
            _detect_color(_is_terminal(target), variables),
            Theme.detect(variables),
        )

    def with_(self, **changes: Any) -> Frame:
        """A copy with fields replaced: ``frame.with_(height=8)``."""
        return replace(self, **changes)

    def to_json(self) -> dict[str, Any]:
        return {
            "width": self.width,
            "height": self.height,
            "charset": self.charset,
            "color": self.color,
            "theme": self.theme.to_json(),
        }


def _variable(variables: Mapping[str, str], name: str) -> str | None:
    value = variables.get(name)
    return value if value else None


def _is_terminal(stream: Any) -> bool:
    try:
        return bool(stream.isatty())
    except (AttributeError, ValueError):
        return False


def _rows_to_height(rows: int) -> int:
    return min(24, max(8, rows // 3))


def _measured(stream: Any) -> tuple[int, int] | None:
    candidates = [stream]
    for name in ("__stdout__", "__stderr__", "__stdin__"):
        candidate = getattr(sys, name, None)
        if candidate is not None and candidate is not stream:
            candidates.append(candidate)
    for candidate in candidates:
        try:
            size = os.get_terminal_size(candidate.fileno())
        except (AttributeError, OSError, ValueError):
            continue
        if size.columns > 0 and size.lines > 0:
            return size.columns, size.lines
    return None


def _detect_size(stream: Any, variables: Mapping[str, str]) -> tuple[int, int]:
    measured = _measured(stream)
    if measured is not None:
        return measured[0], _rows_to_height(measured[1])

    def exported(name: str) -> int | None:
        value = _variable(variables, name)
        if value is None:
            return None
        try:
            parsed = int(value.strip())
        except ValueError:
            return None
        return parsed if parsed > 0 else None

    width = exported("COLUMNS") or 80
    lines = exported("LINES")
    return width, 16 if lines is None else _rows_to_height(lines)


def _is_dumb(term: str) -> bool:
    return term in {"dumb", "unknown"}


def _detect_charset(variables: Mapping[str, str]) -> Charset:
    for name in ("MALEVICH_CHARSET", "SUPREMUS_CHARSET"):
        forced = _variable(variables, name)
        if forced is not None:
            named = _CHARSETS.get(forced.strip().lower())
            if named is not None:
                return named
    term = _variable(variables, "TERM")
    if term is not None and _is_dumb(term):
        return "Ascii"
    for name in ("LC_ALL", "LC_CTYPE", "LANG"):
        locale = _variable(variables, name)
        if locale is not None:
            return "Quadrants" if "utf" in locale.lower() else "Ascii"
    return "Quadrants"


def _detect_color(is_terminal: bool, variables: Mapping[str, str]) -> ColorMode:
    if variables.get("NO_COLOR") is not None:
        return "Plain"

    def forcing(name: str) -> bool:
        value = _variable(variables, name)
        return value is not None and value != "0"

    if not (forcing("CLICOLOR_FORCE") or forcing("FORCE_COLOR")) and not is_terminal:
        return "Plain"
    term = _variable(variables, "TERM") or ""
    if _is_dumb(term):
        return "Plain"
    colorterm = _variable(variables, "COLORTERM") or ""
    truecolor = colorterm in {"truecolor", "24bit"} or term.endswith("-direct")
    if term.startswith("screen"):
        return "Ansi256" if truecolor or "256color" in term else "Ansi16"
    if truecolor:
        return "TrueColor"
    if "256color" in term:
        return "Ansi256"
    return "Ansi16"
