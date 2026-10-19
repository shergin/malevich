"""The interaction controller: the Python spelling of the ratatui and Ink ``PlotState``.

Mutable, like a list's selection state: the host keeps one instance, the
widget captures each render's mapping, and :meth:`PlotState.on_mouse` runs
the fixed gesture grammar. The controller never reads input; the host feeds
it :class:`Mouse` values in widget cell coordinates. Hover, zoom, pan, and
rubber-band selection are compositions of the engine's ``Mapping`` and
``Viewport``; nothing here draws.
"""

from __future__ import annotations

import math
from collections.abc import Callable
from dataclasses import dataclass
from typing import Literal

from .mapping import Mapping, Panel, Viewport, Windows

__all__ = ["Mouse", "MouseButton", "MouseKind", "PlotState", "link_x"]

#: Wheel and keyboard zoom steps: one notch in, its exact inverse out.
ZOOM_IN = 0.8
ZOOM_OUT = 1.25
#: The keyboard pan step, as a fraction of the visible window.
PAN_STEP = 0.1
#: One horizontal scroll notch's pan, small because trackpads emit many.
SCROLL_PAN = 0.03

MouseButton = Literal["left", "right", "middle"]
MouseKind = Literal[
    "moved", "down", "drag", "up", "scroll_up", "scroll_down", "scroll_left", "scroll_right"
]


@dataclass(frozen=True)
class Mouse:
    """Backend-neutral mouse input, in widget cell coordinates (0-based)."""

    kind: MouseKind
    column: int
    row: int
    button: MouseButton | None = None


Hover = tuple[str, int, int]  # ("cell", column, row) or ("column", column, -1)
Point = tuple[int, int]


class PlotState:
    """Interaction state for one plot pane."""

    def __init__(self) -> None:
        self._area = Panel(0, 0, 0, 0)
        self._mapping: Mapping | None = None
        self._view_x: tuple[float, float] | None = None
        self._view_y: tuple[float, float] | None = None
        self._hover: Hover | None = None
        self._drag: tuple[str, Point, Point] | None = None  # ("pan", last, last) | ("select", anchor, current)

    # ---- what the host reads ----------------------------------------------

    def mapping(self) -> Mapping | None:
        """The mapping cached by the last stateful render."""
        return self._mapping

    def data_at(self, column: int, row: int) -> tuple[float, float] | None:
        """Hit-tests a widget cell against the last render."""
        if self._mapping is None:
            return None
        local_column = column - self._area.column
        local_row = row - self._area.row
        if local_column < 0 or local_row < 0:
            return None
        return self._mapping.data_at(local_column, local_row)

    def plot_area(self) -> Panel | None:
        """The plot rectangle of the last render, in widget coordinates."""
        if self._mapping is None:
            return None
        panel = self._mapping.plot_area
        if panel is None:
            return None
        return Panel(self._area.column + panel.column, self._area.row + panel.row, panel.width, panel.height)

    def cursor(self) -> Point | None:
        """The hover cell, when it is over the plot rectangle."""
        if self._hover is None or self._hover[0] != "cell":
            return None
        return (self._hover[1], self._hover[2])

    def cursor_data(self) -> tuple[float, float] | None:
        cursor = self.cursor()
        return None if cursor is None else self.data_at(*cursor)

    def hover(self) -> Hover | None:
        """The hover, including a mirrored x-only column."""
        return self._hover

    def hover_data_x(self) -> float | None:
        """The data x under the hover, a real cursor's or a mirrored one's."""
        rect = self.plot_area()
        if rect is None or self._hover is None:
            return None
        data = self.data_at(self._hover[1], rect.row)
        return None if data is None else data[0]

    def selection(self) -> tuple[Point, Point] | None:
        """A rubber-band selection in progress: ``(anchor, current)``."""
        if self._drag is None or self._drag[0] != "select":
            return None
        return (self._drag[1], self._drag[2])

    def viewport(self) -> Windows:
        """The window pair the next stateful render applies."""
        out: Windows = {}
        if self._view_x is not None:
            out["x"] = self._view_x
        if self._view_y is not None:
            out["y"] = self._view_y
        return out

    # ---- what the host writes ---------------------------------------------

    def set_viewport(self, view: Windows | Viewport) -> None:
        """Replaces the viewport: the escape hatch for a host's own gestures."""
        windows = view.windows() if isinstance(view, Viewport) else dict(view)
        self._view_x = _pair(windows.get("x"))
        self._view_y = _pair(windows.get("y"))

    def reset_view(self) -> None:
        self._view_x = None
        self._view_y = None

    def hover_x(self, x: float) -> bool:
        """Places an x-only hover from a data x: the mirrored crosshair of a
        linked pane. A non-finite x, or one off the window, clears it."""
        column = None
        if self._mapping is not None and math.isfinite(x):
            column = self._mapping.column_at(x)
        hover: Hover | None = None if column is None else ("column", self._area.column + column, -1)
        changed = hover != self._hover
        self._hover = hover
        return changed

    def zoom_in(self) -> bool:
        return self._zoom_center(ZOOM_IN)

    def zoom_out(self) -> bool:
        return self._zoom_center(ZOOM_OUT)

    def pan_left(self) -> bool:
        return self._pan_step(-PAN_STEP)

    def pan_right(self) -> bool:
        return self._pan_step(PAN_STEP)

    def capture(self, mapping: Mapping, area: Panel) -> None:
        """Called by the stateful widget after each render."""
        self._mapping = mapping
        self._area = area

    def on_mouse(self, mouse: Mouse) -> bool:
        """Feeds one input through the default gesture grammar. Returns whether
        anything changed, so a host can skip a redraw."""
        kind = mouse.kind
        column, row = mouse.column, mouse.row
        if kind == "moved":
            return self._hover_at(column, row)
        if kind == "down":
            if mouse.button == "left":
                hovered = self._hover_at(column, row)
                if self._inside(column, row):
                    self._drag = ("pan", (column, row), (column, row))
                    return True
                return hovered
            if mouse.button == "right":
                if self._inside(column, row):
                    self._drag = ("select", (column, row), (column, row))
                    return True
            return False
        if kind == "drag":
            if mouse.button == "left":
                hovered = self._hover_at(column, row)
                if self._drag is None or self._drag[0] != "pan":
                    return hovered
                last = self._drag[1]
                self._drag = ("pan", (column, row), (column, row))
                return self._pan_by(last, (column, row)) or hovered
            if mouse.button == "right":
                hovered = self._hover_at(column, row)
                if self._drag is None or self._drag[0] != "select":
                    return hovered
                rect = self.plot_area()
                if rect is None:
                    return hovered
                self._drag = ("select", self._drag[1], _clamp_into(rect, column, row))
                return True
            return False
        if kind == "up":
            if mouse.button == "left":
                panning = self._drag is not None and self._drag[0] == "pan"
                if panning:
                    self._drag = None
                return panning
            if mouse.button == "right":
                if self._drag is None or self._drag[0] != "select":
                    return False
                _, anchor, current = self._drag
                self._drag = None
                self._apply_selection(anchor, current)
                return True
            return False
        if kind == "scroll_up":
            return self._wheel(ZOOM_IN, column, row)
        if kind == "scroll_down":
            return self._wheel(ZOOM_OUT, column, row)
        if kind == "scroll_left":
            return self._scroll_pan(-SCROLL_PAN, column, row)
        if kind == "scroll_right":
            return self._scroll_pan(SCROLL_PAN, column, row)
        return False

    # ---- the grammar ------------------------------------------------------

    def _hover_at(self, column: int, row: int) -> bool:
        hover: Hover | None = ("cell", column, row) if self._inside(column, row) else None
        changed = hover != self._hover
        self._hover = hover
        return changed

    def _inside(self, column: int, row: int) -> bool:
        rect = self.plot_area()
        return rect is not None and rect.contains(column, row)

    def _x_is_bands(self) -> bool:
        return self._mapping is not None and self._mapping.x_categories is not None

    def _wheel(self, factor: float, column: int, row: int) -> bool:
        if not self._inside(column, row):
            return False
        data = self.data_at(column, row)
        if data is None or self._x_is_bands():
            return False
        return self._commit_x(lambda seeded: seeded.zoom_x(factor, data[0]))

    def _scroll_pan(self, fraction: float, column: int, row: int) -> bool:
        if not self._inside(column, row) or self._x_is_bands():
            return False
        return self._commit_x(lambda seeded: seeded.pan_x(fraction))

    def _pan_step(self, fraction: float) -> bool:
        if self._mapping is None or self._x_is_bands():
            return False
        return self._commit_x(lambda seeded: seeded.pan_x(fraction))

    def _zoom_center(self, factor: float) -> bool:
        mapping = self._mapping
        panel = None if mapping is None else mapping.plot_area
        if mapping is None or panel is None or self._x_is_bands():
            return False
        anchor = mapping.data_at(panel.column + panel.width // 2, panel.row + panel.height // 2)
        if anchor is None:
            return False
        return self._commit_x(lambda seeded: seeded.zoom_x(factor, anchor[0]))

    def _pan_by(self, start: Point, end: Point) -> bool:
        mapping = self._mapping
        panel = None if mapping is None else mapping.plot_area
        if mapping is None or panel is None:
            return False
        dx = end[0] - start[0]
        dy = end[1] - start[1]
        if dx == 0 and dy == 0:
            return False
        columns = max(panel.width, 1)
        rows = max(panel.height, 1)
        return self._commit_both(lambda seeded: seeded.pan_x(-dx / columns).pan_y(dy / rows))

    def _apply_selection(self, anchor: Point, current: Point) -> None:
        mapping = self._mapping
        if mapping is None:
            return
        if abs(anchor[0] - current[0]) < 2 or abs(anchor[1] - current[1]) < 2:
            return
        a = self.data_at(*anchor)
        c = self.data_at(*current)
        if a is None or c is None:
            return
        view = mapping.viewport()
        if mapping.x_categories is None:
            view = view.with_x(a[0], c[0])
        if mapping.y_categories is None:
            view = view.with_y(a[1], c[1])
        self._view_x = view.x
        self._view_y = view.y

    def _seeded(self, *, keep_y: bool) -> Viewport | None:
        mapping = self._mapping
        if mapping is None:
            return None
        seeded = mapping.viewport()
        if self._view_x is not None:
            seeded = seeded.with_x(*self._view_x)
        if self._view_y is not None:
            seeded = seeded.with_y(*self._view_y)
        elif not keep_y:
            seeded = seeded.reset_y()
        return seeded

    def _commit_x(self, transform: Callable[[Viewport], Viewport]) -> bool:
        seeded = self._seeded(keep_y=False)
        if seeded is None:
            return False
        after = transform(seeded)
        self._view_x = after.x
        self._view_y = after.y
        return True

    def _commit_both(self, transform: Callable[[Viewport], Viewport]) -> bool:
        seeded = self._seeded(keep_y=True)
        if seeded is None:
            return False
        after = transform(seeded)
        self._view_x = after.x
        self._view_y = after.y
        return True


def link_x(active: PlotState, passive: PlotState) -> None:
    """Shares the active pane's x window with the passive one and mirrors its
    cursor there as a vertical-only crosshair. Each pane keeps its own y."""
    window = active.viewport().get("x")
    view = passive.viewport()
    windows: Windows = {}
    if window is not None:
        windows["x"] = window
    if "y" in view:
        windows["y"] = view["y"]
    passive.set_viewport(windows)
    x = active.hover_data_x()
    passive.hover_x(math.nan if x is None else x)


def _pair(value: object) -> tuple[float, float] | None:
    if value is None:
        return None
    low, high = value  # type: ignore[misc]
    low, high = float(low), float(high)
    if not (math.isfinite(low) and math.isfinite(high)):
        return None
    return (low, high)


def _clamp_into(rect: Panel, column: int, row: int) -> Point:
    right = max(rect.column, rect.column + rect.width - 1)
    bottom = max(rect.row, rect.row + rect.height - 1)
    return (min(max(column, rect.column), right), min(max(row, rect.row), bottom))
