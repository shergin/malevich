"""Interaction chrome drawn over a raster: selection band, crosshair, snap
highlights, readout. Overlays live in the cells only — the plot value and
its rendering stay byte-identical with or without them."""

from __future__ import annotations

import math
import unicodedata
from dataclasses import dataclass, replace
from typing import Any

from .color import ColorValue
from .mapping import Panel
from .raster import Raster, RasterCell

__all__ = ["Snapped", "apply_overlays", "display_width", "snap_targets"]


@dataclass(frozen=True)
class Snapped:
    label: str | None
    x: float
    value: float | None


def display_width(text: str) -> int:
    width = 0
    for char in text:
        if ord(char) < 32:
            continue
        width += 2 if unicodedata.east_asian_width(char) in {"W", "F"} else 1
    return width


def apply_overlays(
    raster: Raster,
    state: Any,
    plot: Any,
    *,
    crosshair: bool = True,
    readout: bool = True,
    snap: bool = True,
    light: bool = False,
) -> Raster:
    rect = state.plot_area()
    if rect is None:
        return raster
    cells = list(raster.cells)
    width = raster.width
    tint: ColorValue = "White" if light else "BrightBlack"
    highlight: ColorValue = "Black" if light else "White"
    origin = _widget_origin(state, rect, raster)

    def at(column: int, row: int) -> int | None:
        c, r = column - origin.column, row - origin.row
        if c < 0 or r < 0 or c >= raster.width or r >= raster.height:
            return None
        return r * width + c

    def set_background(column: int, row: int, color: ColorValue) -> None:
        index = at(column, row)
        if index is not None:
            cells[index] = replace(cells[index], background=color)

    def tint_if_unset(column: int, row: int) -> None:
        index = at(column, row)
        if index is not None and cells[index].background == "Default":
            cells[index] = replace(cells[index], background=tint)

    selection = state.selection()
    if selection is not None:
        (ax, ay), (cx, cy) = selection
        for y in range(min(ay, cy), max(ay, cy) + 1):
            for x in range(min(ax, cx), max(ax, cx) + 1):
                set_background(x, y, tint)

    hover = _hover_in_rect(state, rect)
    if crosshair and hover is not None:
        column, row = hover
        if row is not None:
            for x in range(rect.column, rect.column + rect.width):
                tint_if_unset(x, row)
        for y in range(rect.row, rect.row + rect.height):
            tint_if_unset(column, y)

    if hover is None:
        return Raster(raster.width, raster.height, cells)

    snapped = snap_targets(plot, state) if snap else []
    mapping = state.mapping()
    for target in snapped:
        if target.value is None or mapping is None:
            continue
        cell = mapping.cell_at(target.x, target.value)
        if cell is None:
            continue
        column = origin.column + cell[0]
        row = origin.row + cell[1]
        if column < rect.column + rect.width and row < rect.row + rect.height:
            set_background(column, row, highlight)

    if readout:
        text = _readout_line(state, snapped, max(0, rect.width - 2))
        if text:
            text_width = display_width(text)
            column = rect.column + rect.width - text_width - 1
            for char in text:
                index = at(column, rect.row)
                if index is not None:
                    cells[index] = RasterCell(char, highlight, cells[index].background, 1)
                column += display_width(char) or 1

    return Raster(raster.width, raster.height, cells)


def snap_targets(plot: Any, state: Any) -> list[Snapped]:
    cursor_x = state.hover_data_x()
    mapping = state.mapping()
    if cursor_x is None or mapping is None:
        return []
    window = mapping.x_domain
    columns = plot.columns()
    snapped: list[Snapped] = []
    for layer in plot.layers:
        body = layer.get("Line") or layer.get("Points")
        if body is None:
            continue
        y = _series(body.get("y"), columns)
        if y is None:
            continue
        x = _series(body.get("x"), columns)
        index = _nearest_visible(x, len(y), cursor_x, window)
        if index is None:
            continue
        value = y[index]
        label = body.get("label")
        snapped.append(
            Snapped(
                label if isinstance(label, str) else None,
                x[index] if x is not None else float(index),
                value if value is not None and math.isfinite(value) else None,
            )
        )
    return snapped


def _series(field: Any, columns: list[Any]) -> list[float] | None:
    if field is None:
        return None
    if isinstance(field, dict) and "col" in field:
        data = columns[field["col"]]
        values = data.tolist() if hasattr(data, "tolist") else list(data)
    elif isinstance(field, list):
        values = field
    else:
        return None
    return [math.nan if v is None else float(v) for v in values]


def _nearest_visible(
    x: list[float] | None, length: int, target: float, window: tuple[float, float]
) -> int | None:
    if x is not None:
        best: int | None = None
        nearest = math.inf
        for index, value in enumerate(x):
            if not math.isfinite(value) or value < window[0] or value > window[1]:
                continue
            distance = abs(value - target)
            if distance < nearest:
                best = index
                nearest = distance
        return best
    index = round(target)
    if index < window[0]:
        index = math.ceil(target)
    elif index > window[1]:
        index = math.floor(target)
    if index < max(window[0], 0) or index > window[1]:
        return None
    return index if index < length else None


def _readout_line(state: Any, snapped: list[Snapped], budget: int) -> str | None:
    cursor_x = state.hover_data_x()
    mapping = state.mapping()
    if cursor_x is None or mapping is None:
        return None
    x_part = mapping.format_x(snapped[0].x) if len(snapped) == 1 else mapping.format_x(cursor_x)
    parts = [x_part]
    if not snapped:
        cursor = state.cursor_data()
        if cursor is not None:
            parts.append(mapping.format_y(cursor[1]))
    for target in snapped:
        value = "—" if target.value is None else mapping.format_y(target.value)
        parts.append(f"{target.label}: {value}" if target.label else value)
    while True:
        text = " · ".join(parts)
        if display_width(text) <= budget:
            return text
        if len(parts) <= 2:
            return None
        parts.pop()


def _hover_in_rect(state: Any, rect: Panel) -> tuple[int, int | None] | None:
    hover = state.hover()
    if hover is None:
        return None
    kind, column, row = hover
    if kind == "cell":
        return (column, row) if rect.contains(column, row) else None
    if rect.column <= column < rect.column + rect.width:
        return (column, None)
    return None


def _widget_origin(state: Any, rect: Panel, raster: Raster) -> Panel:
    mapping = state.mapping()
    panel = None if mapping is None else mapping.plot_area
    if panel is None:
        return Panel(0, 0, raster.width, raster.height)
    return Panel(rect.column - panel.column, rect.row - panel.row, raster.width, raster.height)
