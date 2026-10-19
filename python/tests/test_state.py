import math

from supremus import Frame, Panel, bar, line
from supremus._overlays import apply_overlays, display_width, snap_targets
from supremus._state import Mouse, PlotState, link_x

FRAME = Frame.plain(40, 10)
PLOT = line([1, 5, 2, 8, 3, 9, 4, 10]).title("t")


def captured(plot=PLOT, frame=FRAME) -> PlotState:
    state = PlotState()
    state.capture(plot.mapping(frame), Panel(0, 0, frame.width, frame.height))
    return state


def center(state: PlotState) -> tuple[int, int]:
    area = state.plot_area()
    assert area is not None
    return (area.column + area.width // 2, area.row + area.height // 2)


def test_hover_inside_and_outside() -> None:
    state = captured()
    column, row = center(state)
    assert state.on_mouse(Mouse("moved", column, row))
    assert state.cursor() == (column, row)
    assert state.cursor_data() is not None
    assert not state.on_mouse(Mouse("moved", column, row)), "no change, no redraw"
    assert state.on_mouse(Mouse("moved", 0, 0))
    assert state.cursor() is None


def test_wheel_zooms_x_at_the_cursor() -> None:
    state = captured()
    column, row = center(state)
    assert state.on_mouse(Mouse("scroll_up", column, row))
    x = state.viewport()["x"]
    assert x[1] - x[0] < 7
    assert state.on_mouse(Mouse("scroll_down", column, row))
    assert state.viewport()["x"][1] - state.viewport()["x"][0] > x[1] - x[0]
    assert state.zoom_in() and state.pan_left() and state.pan_right() and state.zoom_out()
    state.reset_view()
    assert state.viewport() == {}


def test_drag_pans_and_rubber_band_zooms() -> None:
    state = captured()
    column, row = center(state)
    assert state.on_mouse(Mouse("down", column, row, "left"))
    assert state.on_mouse(Mouse("drag", column + 3, row, "left"))
    assert "x" in state.viewport()
    assert state.on_mouse(Mouse("up", column + 3, row, "left"))
    before = state.viewport()["x"]
    assert state.on_mouse(Mouse("down", column - 3, row - 1, "right"))
    assert state.on_mouse(Mouse("drag", column + 3, row + 1, "right"))
    assert state.selection() is not None
    assert state.on_mouse(Mouse("up", column + 3, row + 1, "right"))
    after = state.viewport()["x"]
    assert after[1] - after[0] < before[1] - before[0]
    assert state.selection() is None


def test_band_axes_keep_their_window() -> None:
    state = captured(bar(["a", "b", "c"], [1, 2, 3]))
    column, row = center(state)
    assert not state.on_mouse(Mouse("scroll_up", column, row))
    assert not state.zoom_in()
    assert state.viewport() == {}


def test_link_x_mirrors_window_and_cursor() -> None:
    active = captured()
    passive = captured(line([2, 2, 3, 3, 4, 4, 5, 5]))
    column, row = center(active)
    active.on_mouse(Mouse("scroll_up", column, row))
    active.on_mouse(Mouse("moved", column, row))
    link_x(active, passive)
    assert passive.viewport()["x"] == active.viewport()["x"]
    hover = passive.hover()
    assert hover is not None and hover[0] == "column"
    assert passive.cursor() is None, "a mirrored hover has no honest row"
    assert passive.hover_x(math.nan)
    assert passive.hover() is None


def test_overlays_draw_readout_and_crosshair() -> None:
    state = captured()
    column, row = center(state)
    state.on_mouse(Mouse("moved", column, row))
    raster = PLOT.raster(FRAME)
    overlaid = apply_overlays(raster, state, PLOT)
    assert overlaid.width == raster.width
    area = state.plot_area()
    tinted = sum(1 for c in overlaid.cells if c.background == "BrightBlack")
    assert tinted >= area.width + area.height - 1
    readout = overlaid.to_plain().splitlines()[area.row]
    assert "·" in readout or ":" in readout or any(ch.isdigit() for ch in readout)
    assert PLOT.raster(FRAME).cells == raster.cells, "the plot's own raster is untouched"
    targets = snap_targets(PLOT, state)
    assert len(targets) == 1 and targets[0].value is not None


def test_display_width_counts_wide_glyphs() -> None:
    assert display_width("abc") == 3
    assert display_width("日本") == 4
    assert display_width("a\x1bb") == 2
