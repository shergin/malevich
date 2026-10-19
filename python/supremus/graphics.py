"""Pixel-graphics detection: which protocol the terminal advertises.

This is the sniffing tier of the crate's ``pixel::detect`` ported to Python,
because only the Python process knows its own environment and streams. It
reads variables and never writes to the terminal. Unknown terminals get
cells, never garbage.
"""

from __future__ import annotations

import os
from collections.abc import Mapping
from dataclasses import dataclass
from typing import Literal

__all__ = ["Graphics", "Protocol", "detect_graphics"]

Protocol = Literal["sixel", "kitty", "iterm2"]


@dataclass(frozen=True)
class Graphics:
    """A pixel protocol and the terminal's cell size in device pixels."""

    protocol: Protocol
    cell_size: tuple[int, int] = (8, 16)


def detect_graphics(env: Mapping[str, str] | None = None) -> Graphics | None:
    """The best protocol the environment advertises, or ``None`` for cells.

    ``MALEVICH_GRAPHICS`` (or ``SUPREMUS_GRAPHICS``) names a protocol or
    ``none`` and outranks every inference. A multiplexer (``TMUX``, a
    ``screen``/``tmux`` ``TERM``) means cells. kitty and Ghostty speak kitty;
    iTerm2 and WezTerm their inline-image protocol; a sixel ``TERM``,
    Windows Terminal, foot, and Konsole speak sixel.
    """
    variables = os.environ if env is None else env

    def variable(name: str) -> str | None:
        value = variables.get(name)
        return value if value else None

    for name in ("MALEVICH_GRAPHICS", "SUPREMUS_GRAPHICS"):
        forced = variable(name)
        if forced is None:
            continue
        named = forced.strip().lower()
        if named in {"none", "cells"}:
            return None
        if named in {"kitty", "sixel"}:
            return Graphics(named)  # type: ignore[arg-type]
        if named in {"iterm2", "iterm"}:
            return Graphics("iterm2")
    if variable("TMUX") is not None:
        return None
    term = variable("TERM") or ""
    if term in {"dumb", "unknown"} or term.startswith(("screen", "tmux")):
        return None
    if (
        variable("KITTY_WINDOW_ID") is not None
        or variable("KITTY_PID") is not None
        or variable("GHOSTTY_BIN_DIR") is not None
        or "kitty" in term
        or "ghostty" in term
    ):
        return Graphics("kitty")
    program = (variable("TERM_PROGRAM") or "").lower()
    if "ghostty" in program:
        return Graphics("kitty")
    iterm = "iterm" in program or "iterm" in (variable("LC_TERMINAL") or "").lower()
    if iterm or "wezterm" in program or variable("WEZTERM_EXECUTABLE") is not None:
        return Graphics("iterm2")
    if "sixel" in term or variable("WT_SESSION") is not None or "foot" in term or "konsole" in program:
        return Graphics("sixel")
    return None
