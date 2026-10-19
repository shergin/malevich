"""The Rich renderable: segments straight from the cell grid.

Rich is imported here only, when ``console.print(plot)`` happens. The plot
re-renders at the width Rich offers and the height it asks for (or a third
of the console, like ``Frame.detect``), and yields one ``Segment`` per run
of equal style — never a raw ANSI string, which would break ``Panel``
alignment.
"""

from __future__ import annotations

from collections.abc import Iterator
from typing import TYPE_CHECKING, Any

from .color import ColorValue
from .frame import Frame, Theme

if TYPE_CHECKING:  # pragma: no cover
    from .plot import Grid, Plot

_RICH_NAMES = {
    "Black": "black",
    "Red": "red",
    "Green": "green",
    "Yellow": "yellow",
    "Blue": "blue",
    "Magenta": "magenta",
    "Cyan": "cyan",
    "White": "white",
    "BrightBlack": "bright_black",
    "BrightRed": "bright_red",
    "BrightGreen": "bright_green",
    "BrightYellow": "bright_yellow",
    "BrightBlue": "bright_blue",
    "BrightMagenta": "bright_magenta",
    "BrightCyan": "bright_cyan",
    "BrightWhite": "bright_white",
}


def rich_color(color: ColorValue) -> Any:
    """A ``rich.color.Color`` for a cell color, or ``None`` for the terminal default."""
    from rich.color import Color

    if isinstance(color, dict):
        if "Rgb" in color:
            r, g, b = color["Rgb"]
            return Color.from_rgb(r, g, b)
        if "Ansi256" in color:
            return Color.from_ansi(int(color["Ansi256"]))
        return None
    name = _RICH_NAMES.get(color)
    return None if name is None else Color.parse(name)


def frame_for(console: Any, options: Any) -> Frame:
    """The frame a Rich render fills: Rich's width, its height or a third of the console."""
    width = max(1, int(options.max_width))
    height = options.height
    if height is None:
        rows = console.height or 24
        height = min(24, max(8, rows // 3))
        if options.max_height:
            height = min(height, int(options.max_height))
    theme = Theme.detect()
    color = "Plain" if getattr(console, "no_color", False) else "TrueColor"
    return Frame(width, max(1, int(height)), "Quadrants", color, theme)


def _rows(raster: Any) -> Iterator[Any]:
    from rich.segment import Segment
    from rich.style import Style

    styles: dict[tuple[str, str], Style] = {}
    for row in range(raster.height):
        for text, fg, bg in raster.runs(row):
            key = (repr(fg), repr(bg))
            style = styles.get(key)
            if style is None:
                style = Style(color=rich_color(fg), bgcolor=rich_color(bg))
                styles[key] = style
            yield Segment(text, style)
        yield Segment.line()


def segments(plot: Plot, console: Any, options: Any) -> Iterator[Any]:
    yield from _rows(plot.raster(frame_for(console, options)))


def grid_segments(grid: Grid, console: Any, options: Any) -> Iterator[Any]:
    """A grid has no raster; its string is parsed back into text."""
    from rich.text import Text

    frame = frame_for(console, options)
    yield Text.from_ansi(grid.render(frame))
