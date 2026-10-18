"""The cell grid under a render: glyphs and colors, chrome included.

A :class:`Raster` is what a TUI host paints instead of decoding ANSI. Rows
skip continuation cells (the right half of a wide glyph), and ``runs`` merges
neighbouring cells of one style so a host emits one segment per run.
"""

from __future__ import annotations

from collections.abc import Iterator
from dataclasses import dataclass

from .color import NAMED_BY_INDEX, ColorValue

__all__ = ["Raster", "RasterCell", "unpack"]


@dataclass(frozen=True)
class RasterCell:
    """One encoded cell."""

    glyph: str
    foreground: ColorValue
    background: ColorValue
    #: Display columns: 2 for a wide glyph, 1 for an ordinary cell, 0 for a continuation.
    columns: int = 1


class Raster:
    """``width * height`` cells, row-major, origin top-left."""

    __slots__ = ("width", "height", "cells")

    def __init__(self, width: int, height: int, cells: list[RasterCell]) -> None:
        if len(cells) != width * height:
            raise ValueError("a raster has width * height cells")
        self.width = width
        self.height = height
        self.cells = cells

    def is_empty(self) -> bool:
        return not self.cells

    def cell(self, column: int, row: int) -> RasterCell | None:
        if column < 0 or row < 0 or column >= self.width or row >= self.height:
            return None
        return self.cells[row * self.width + column]

    def rows(self) -> Iterator[list[RasterCell]]:
        """Each row's cells, continuation cells skipped."""
        for row in range(self.height):
            start = row * self.width
            yield [c for c in self.cells[start : start + self.width] if c.columns != 0]

    def runs(self, row: int) -> list[tuple[str, ColorValue, ColorValue]]:
        """One row as ``(text, foreground, background)`` runs of equal style."""
        runs: list[tuple[str, ColorValue, ColorValue]] = []
        start = row * self.width
        for cell in self.cells[start : start + self.width]:
            if cell.columns == 0:
                continue
            if runs and runs[-1][1] == cell.foreground and runs[-1][2] == cell.background:
                text, fg, bg = runs[-1]
                runs[-1] = (text + cell.glyph, fg, bg)
            else:
                runs.append((cell.glyph, cell.foreground, cell.background))
        return runs

    def to_plain(self) -> str:
        """The glyphs alone, one line per row, trailing spaces trimmed."""
        lines = ["".join(c.glyph for c in row).rstrip() for row in self.rows()]
        return "\n".join(lines)

    def __repr__(self) -> str:
        return f"Raster({self.width}x{self.height})"


def unpack(planes: tuple[int, int, str, bytes, bytes, bytes]) -> Raster:
    """Builds a :class:`Raster` from the native module's packed planes."""
    width, height, glyphs, foreground, background, columns = planes
    cells = [
        RasterCell(
            glyph,
            _unpack_color(foreground, i * 4),
            _unpack_color(background, i * 4),
            columns[i] if i < len(columns) else 1,
        )
        for i, glyph in enumerate(glyphs)
    ]
    return Raster(width, height, cells)


def _unpack_color(plane: bytes, offset: int) -> ColorValue:
    tag = plane[offset]
    if tag == 1:
        return NAMED_BY_INDEX[plane[offset + 1]]
    if tag == 2:
        return {"Ansi256": plane[offset + 1]}
    if tag == 3:
        return {"Rgb": [plane[offset + 1], plane[offset + 2], plane[offset + 3]]}
    return "Default"


def color_rgb(color: ColorValue) -> tuple[int, int, int] | None:
    """An approximate RGB for a color, for hosts that need one; ``None`` for the default."""
    if isinstance(color, dict):
        if "Rgb" in color:
            r, g, b = color["Rgb"]
            return (r, g, b)
        if "Ansi256" in color:
            return _ansi256_rgb(color["Ansi256"])
        return None
    return _NAMED_RGB.get(color)


_NAMED_RGB: dict[str, tuple[int, int, int]] = {
    "Black": (0, 0, 0),
    "Red": (205, 0, 0),
    "Green": (0, 205, 0),
    "Yellow": (205, 205, 0),
    "Blue": (0, 0, 238),
    "Magenta": (205, 0, 205),
    "Cyan": (0, 205, 205),
    "White": (229, 229, 229),
    "BrightBlack": (127, 127, 127),
    "BrightRed": (255, 0, 0),
    "BrightGreen": (0, 255, 0),
    "BrightYellow": (255, 255, 0),
    "BrightBlue": (92, 92, 255),
    "BrightMagenta": (255, 0, 255),
    "BrightCyan": (0, 255, 255),
    "BrightWhite": (255, 255, 255),
}


def _ansi256_rgb(index: int) -> tuple[int, int, int]:
    if index < 16:
        return _NAMED_RGB[NAMED_BY_INDEX[index]]
    if index >= 232:
        level = 8 + (index - 232) * 10
        return (level, level, level)
    index -= 16
    r, g, b = index // 36, (index // 6) % 6, index % 6
    scale = (0, 95, 135, 175, 215, 255)
    return (scale[r], scale[g], scale[b])

