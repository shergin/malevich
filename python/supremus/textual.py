"""The Textual widget: the ratatui and Ink widget, spelled in Textual.

``pip install supremus[textual]``. A :class:`PlotWidget` paints a plot into
its own size, one ``Strip`` per line from the cell grid, and resizes with
its container. Give it a :class:`PlotState` and it becomes an instrument:
hover crosshair with a snapped readout, wheel zoom at the cursor, left-drag
pan, right-drag rubber band, ``+``/``-``/arrows/``r`` keys. The widget never
owns the terminal; Textual delivers the events, the controller interprets
them, and the plot value stays untouched.
"""

from __future__ import annotations

from typing import Any

from rich.segment import Segment
from rich.style import Style
from textual import events
from textual.binding import Binding
from textual.strip import Strip
from textual.widget import Widget

from ._overlays import apply_overlays
from ._rich import rich_color
from ._state import Mouse, PlotState, link_x
from .frame import Charset, Frame, Theme
from .mapping import Panel
from .plot import Plot
from .raster import Raster

__all__ = ["Mouse", "PlotState", "PlotWidget", "link_x"]

_BUTTONS = {1: "left", 2: "middle", 3: "right"}


class PlotWidget(Widget):
    """A plot in a Textual app. Pass ``state`` for interaction."""

    DEFAULT_CSS = """
    PlotWidget {
        width: 1fr;
        height: 1fr;
    }
    """

    BINDINGS = [
        Binding("plus,equals_sign", "zoom_in", "Zoom in", show=False),
        Binding("minus", "zoom_out", "Zoom out", show=False),
        Binding("left", "pan_left", "Pan left", show=False),
        Binding("right", "pan_right", "Pan right", show=False),
        Binding("r", "reset_view", "Reset view", show=False),
    ]

    def __init__(
        self,
        plot: Plot,
        *,
        state: PlotState | None = None,
        charset: Charset = "Quadrants",
        theme: Theme | None = None,
        crosshair: bool = True,
        readout: bool = True,
        snap: bool = True,
        name: str | None = None,
        id: str | None = None,  # noqa: A002 - Textual's own parameter name
        classes: str | None = None,
    ) -> None:
        super().__init__(name=name, id=id, classes=classes)
        self._plot = plot
        self.state = state
        self.charset: Charset = charset
        self.theme = theme
        self.crosshair = crosshair
        self.readout = readout
        self.snap = snap
        self.can_focus = state is not None
        self._cache: tuple[Any, Raster] | None = None

    @property
    def plot(self) -> Plot:
        return self._plot

    @plot.setter
    def plot(self, plot: Plot) -> None:
        """Swapping the plot keeps the view; a live chart updates this way."""
        self._plot = plot
        self.invalidate()

    def invalidate(self) -> None:
        """Forces the next paint to re-render."""
        self._cache = None
        self.refresh()

    # ---- painting ---------------------------------------------------------

    def _frame(self) -> Frame:
        size = self.content_size
        theme = self.theme or Theme.detect()
        return Frame(max(1, size.width), max(1, size.height), self.charset, "TrueColor", theme)

    def _key(self) -> Any:
        state = self.state
        view = None
        if state is not None:
            windows = state.viewport()
            view = (windows.get("x"), windows.get("y"), state.hover(), state.selection())
        return (self.content_size.width, self.content_size.height, id(self._plot), view)

    def _raster(self) -> Raster:
        key = self._key()
        if self._cache is not None and self._cache[0] == key:
            return self._cache[1]
        frame = self._frame()
        state = self.state
        drawn = self._plot.viewport(state.viewport()) if state is not None else self._plot
        raster = drawn.raster(frame)
        if state is not None:
            state.capture(drawn.mapping(frame), Panel(0, 0, frame.width, frame.height))
            raster = apply_overlays(
                raster,
                state,
                self._plot,
                crosshair=self.crosshair,
                readout=self.readout,
                snap=self.snap,
                light=(self.theme or Theme.detect()).is_light,
            )
        self._cache = (key, raster)
        return raster

    def render_line(self, y: int) -> Strip:
        raster = self._raster()
        width = self.content_size.width
        if y >= raster.height:
            return Strip.blank(width)
        segments = [
            Segment(text, Style(color=rich_color(fg), bgcolor=rich_color(bg)))
            for text, fg, bg in raster.runs(y)
        ]
        return Strip(segments, raster.width).simplify()

    def on_resize(self, event: events.Resize) -> None:
        self.invalidate()

    # ---- input ------------------------------------------------------------

    def _feed(self, mouse: Mouse) -> None:
        if self.state is not None and self.state.on_mouse(mouse):
            self.invalidate()

    def on_mouse_move(self, event: events.MouseMove) -> None:
        button = _BUTTONS.get(event.button)
        if button is not None:
            self._feed(Mouse("drag", event.x, event.y, button))  # type: ignore[arg-type]
        else:
            self._feed(Mouse("moved", event.x, event.y))

    def on_mouse_down(self, event: events.MouseDown) -> None:
        button = _BUTTONS.get(event.button)
        if button is None:
            return
        if self.state is not None:
            self.capture_mouse()
        self._feed(Mouse("down", event.x, event.y, button))  # type: ignore[arg-type]

    def on_mouse_up(self, event: events.MouseUp) -> None:
        button = _BUTTONS.get(event.button)
        if self.state is not None:
            self.release_mouse()
        if button is None:
            return
        self._feed(Mouse("up", event.x, event.y, button))  # type: ignore[arg-type]

    def on_mouse_scroll_up(self, event: events.MouseScrollUp) -> None:
        self._feed(Mouse("scroll_up", event.x, event.y))
        event.stop()

    def on_mouse_scroll_down(self, event: events.MouseScrollDown) -> None:
        self._feed(Mouse("scroll_down", event.x, event.y))
        event.stop()

    def on_mouse_scroll_left(self, event: events.MouseScrollLeft) -> None:
        self._feed(Mouse("scroll_left", event.x, event.y))
        event.stop()

    def on_mouse_scroll_right(self, event: events.MouseScrollRight) -> None:
        self._feed(Mouse("scroll_right", event.x, event.y))
        event.stop()

    def on_leave(self, event: events.Leave) -> None:
        if self.state is not None and self.state.hover() is not None:
            self.state.hover_x(float("nan"))
            self.invalidate()

    def _act(self, changed: bool) -> None:
        if changed:
            self.invalidate()

    def action_zoom_in(self) -> None:
        if self.state is not None:
            self._act(self.state.zoom_in())

    def action_zoom_out(self) -> None:
        if self.state is not None:
            self._act(self.state.zoom_out())

    def action_pan_left(self) -> None:
        if self.state is not None:
            self._act(self.state.pan_left())

    def action_pan_right(self) -> None:
        if self.state is not None:
            self._act(self.state.pan_right())

    def action_reset_view(self) -> None:
        if self.state is not None:
            self.state.reset_view()
            self.invalidate()
