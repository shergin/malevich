"""The eight marks: Line, Points, Bars, Area, Cells, Range, Rule, Text.

Constructors are static methods named as in Rust and JS (``Line.y``,
``Line.xy``, ``Bars.new``, ``Rule.h``, ``Text.at``). Channel methods return a
new mark. Position channels take anything series-shaped (see
:mod:`supremus.series`); constant channels take the value. Each mark
serializes to the crate's wire form with its series as ``{"col": n}``
references, so a plot's document is the one the crate reads.
"""

from __future__ import annotations

import copy
import datetime as _dt
import math
from collections.abc import Iterable
from typing import Any, Literal

from .color import Colormap, ColorValue, Reducer, canonicalize_color, color_json
from .series import Column, to_column, to_seconds

__all__ = [
    "Align",
    "Area",
    "Bars",
    "Cells",
    "Dash",
    "Line",
    "LineStyle",
    "Mark",
    "PointStyle",
    "Points",
    "Range",
    "Rule",
    "Text",
]

LineStyle = Literal["Pixels", "Corners"]
Dash = Literal["Solid", "Dashed", "Dotted"]
PointStyle = Literal["Dot", "Plus", "Cross", "Asterisk", "Circle"]
Align = Literal["Left", "Center", "Right"]

_LINE_STYLES = ("Pixels", "Corners")
_DASHES = ("Solid", "Dashed", "Dotted")
_POINT_STYLES = ("Dot", "Plus", "Cross", "Asterisk", "Circle")
_ALIGNS = ("Left", "Center", "Right")
_REDUCERS = ("Count", "Sum", "Mean", "Median", "Min", "Max")

Layer = tuple[dict[str, Any], list[Column]]


def _ref(index: int) -> dict[str, int]:
    return {"col": index}


def _choice(value: str, choices: tuple[str, ...], who: str) -> str:
    for choice in choices:
        if value == choice or value.lower() == choice.lower():
            return choice
    raise ValueError(f"{who} is one of {', '.join(choices)}; got {value!r}")


def _scalar(value: Any, who: str) -> float:
    """A finite data coordinate; datetimes become unix seconds."""
    if isinstance(value, (_dt.datetime, _dt.date)):
        return to_seconds(value)
    number = float(value)
    if not math.isfinite(number):
        raise ValueError(f"{who} requires a finite position")
    return number


def _same_length(who: str, first: str, a: Column, second: str, b: Column) -> None:
    if len(a) != len(b):
        raise ValueError(
            f"{who}: {first} and {second}: channels differ in length ({len(a)} and {len(b)})"
        )


def _categories(categories: Iterable[str]) -> list[str]:
    return [str(c) for c in categories]


class Mark:
    """A geometric primitive bound to data. Subclasses are the closed family."""

    __slots__ = ()

    @property
    def _time_x(self) -> bool:
        return False

    def _layer(self, start: int) -> Layer:
        raise NotImplementedError

    def _copy(self) -> Any:
        return copy.copy(self)

    def __repr__(self) -> str:
        layer, columns = self._layer(0)
        (name, body), *_ = layer.items()
        return f"{name}({len(columns)} columns, {body!r})"


class Line(Mark):
    """A line through points, or a paired series."""

    __slots__ = ("_x", "_y", "_color", "_label", "_style", "_dash", "_glow", "_color_by", "_grade")

    def __init__(self, y: Column, x: Column | None = None) -> None:
        self._x = x
        self._y = y
        self._color: ColorValue | None = None
        self._label: str | None = None
        self._style: str = "Pixels"
        self._dash: str = "Solid"
        self._glow = False
        self._color_by: list[str] | None = None
        self._grade: tuple[Column, Colormap] | None = None

    @staticmethod
    def y(values: Any) -> Line:
        """A line over ``values`` at ``0, 1, 2, …``."""
        return Line(to_column(values, what="Line y"))

    @staticmethod
    def xy(x: Any, y: Any) -> Line:
        """A line through ``(x, y)`` pairs; x may be datetimes."""
        xs, ys = to_column(x, what="Line x"), to_column(y, what="Line y")
        _same_length("Line", "x", xs, "y", ys)
        return Line(ys, xs)

    @property
    def _time_x(self) -> bool:
        return self._x is not None and self._x.time

    def style(self, style: LineStyle) -> Line:
        """``"Pixels"`` (subpixel) or ``"Corners"`` (box-drawing ╭╮╰╯)."""
        other = self._copy()
        other._style = _choice(style, _LINE_STYLES, "Line.style")
        return other

    def color(self, color: ColorValue) -> Line:
        other = self._copy()
        other._color = canonicalize_color(color)
        return other

    def color_by(self, categories: Iterable[str]) -> Line:
        """One color per category, in first-appearance order; legend entries follow."""
        other = self._copy()
        other._color_by = _categories(categories)
        return other

    def label(self, label: str) -> Line:
        other = self._copy()
        other._label = str(label)
        return other

    def dash(self, dash: Dash) -> Line:
        other = self._copy()
        other._dash = _choice(dash, _DASHES, "Line.dash")
        return other

    def glow(self) -> Line:
        other = self._copy()
        other._glow = True
        return other

    def grade(self, values: Any, colormap: Colormap | None = None) -> Line:
        """Colors the line by a third series through a colormap."""
        column = to_column(values, what="Line grade")
        _same_length("Line", "grade", column, "y", self._y)
        other = self._copy()
        other._grade = (column, colormap or Colormap.VIRIDIS)
        return other

    def _layer(self, start: int) -> Layer:
        columns: list[Column] = []
        index = start
        x: Any = None
        if self._x is not None:
            x = _ref(index)
            columns.append(self._x)
            index += 1
        y = _ref(index)
        columns.append(self._y)
        index += 1
        body: dict[str, Any] = {
            "x": x,
            "y": y,
            "color": color_json(self._color),
            "label": self._label,
            "style": self._style,
        }
        if self._color_by is not None:
            body["color_by"] = list(self._color_by)
        if self._glow:
            body["glow"] = True
        if self._dash != "Solid":
            body["dash"] = self._dash
        if self._grade is not None:
            values, colormap = self._grade
            body["grade"] = [_ref(index), colormap.to_json()]
            columns.append(values)
            index += 1
        return {"Line": body}, columns


class Points(Mark):
    """Dots at data positions."""

    __slots__ = ("_x", "_y", "_color", "_label", "_style", "_color_by", "_opacity", "_density")

    def __init__(self, y: Column, x: Column | None = None) -> None:
        self._x = x
        self._y = y
        self._color: ColorValue | None = None
        self._label: str | None = None
        self._style: str = "Dot"
        self._color_by: list[str] | None = None
        self._opacity: float | None = None
        self._density = False

    @staticmethod
    def y(values: Any) -> Points:
        return Points(to_column(values, what="Points y"))

    @staticmethod
    def xy(x: Any, y: Any) -> Points:
        xs, ys = to_column(x, what="Points x"), to_column(y, what="Points y")
        _same_length("Points", "x", xs, "y", ys)
        return Points(ys, xs)

    @property
    def _time_x(self) -> bool:
        return self._x is not None and self._x.time

    def style(self, style: PointStyle) -> Points:
        other = self._copy()
        other._style = _choice(style, _POINT_STYLES, "Points.style")
        return other

    def color(self, color: ColorValue) -> Points:
        other = self._copy()
        other._color = canonicalize_color(color)
        return other

    def color_by(self, categories: Iterable[str]) -> Points:
        other = self._copy()
        other._color_by = _categories(categories)
        return other

    def label(self, label: str) -> Points:
        other = self._copy()
        other._label = str(label)
        return other

    def opacity(self, opacity: float) -> Points:
        """A constant opacity in ``(0, 1]``; overlapping points add up."""
        value = float(opacity)
        if not (math.isfinite(value) and 0.0 < value <= 1.0):
            raise ValueError("Points.opacity requires a value in (0, 1]")
        other = self._copy()
        other._opacity = value
        return other

    def density(self) -> Points:
        """Shades each cell by how many points land in it."""
        other = self._copy()
        other._density = True
        return other

    def _layer(self, start: int) -> Layer:
        columns: list[Column] = []
        index = start
        x: Any = None
        if self._x is not None:
            x = _ref(index)
            columns.append(self._x)
            index += 1
        columns.append(self._y)
        body: dict[str, Any] = {
            "x": x,
            "y": _ref(index),
            "color": color_json(self._color),
            "label": self._label,
            "style": self._style,
        }
        if self._color_by is not None:
            body["color_by"] = list(self._color_by)
        if self._opacity is not None:
            body["opacity"] = self._opacity
        if self._density:
            body["density"] = True
        return {"Points": body}, columns


class Bars(Mark):
    """Bars over bands, contiguous spans, free positions, or explicit intervals."""

    __slots__ = ("_placement", "_values", "_base", "_color", "_label", "_color_by", "_horizontal")

    def __init__(self, placement: dict[str, Any], values: Column) -> None:
        self._placement = placement
        self._values = values
        self._base: Column | None = None
        self._color: ColorValue | None = None
        self._label: str | None = None
        self._color_by: list[str] | None = None
        self._horizontal = False

    @staticmethod
    def new(categories: Iterable[str], values: Any) -> Bars:
        """One bar per category."""
        names = _categories(categories)
        series = to_column(values, what="Bars values")
        if len(names) != len(series):
            raise ValueError(
                f"Bars: categories and values: channels differ in length ({len(names)} and {len(series)})"
            )
        return Bars({"kind": "bands", "categories": names}, series)

    @staticmethod
    def spans(start: float, width: float, values: Any) -> Bars:
        """Histogram shape: bar ``i`` covers ``[start + i*width, start + (i+1)*width)``."""
        if not (math.isfinite(float(start)) and math.isfinite(float(width)) and float(width) > 0):
            raise ValueError("Bars.spans requires a finite start and a positive width")
        return Bars(
            {"kind": "spans", "start": float(start), "width": float(width)},
            to_column(values, what="Bars values"),
        )

    @staticmethod
    def at(x: Any, width: float, values: Any) -> Bars:
        """Bars centered at numeric positions with a shared width."""
        if not (math.isfinite(float(width)) and float(width) > 0):
            raise ValueError("Bars.at requires a finite positive width")
        xs = to_column(x, what="Bars x")
        series = to_column(values, what="Bars values")
        _same_length("Bars", "x", xs, "values", series)
        return Bars({"kind": "at", "x": xs, "width": float(width)}, series)

    @staticmethod
    def intervals(starts: Any, ends: Any, values: Any) -> Bars:
        """Bar ``i`` covers ``[starts[i], ends[i]]``, each its own width."""
        a = to_column(starts, what="Bars starts")
        b = to_column(ends, what="Bars ends")
        series = to_column(values, what="Bars values")
        _same_length("Bars", "starts", a, "values", series)
        _same_length("Bars", "ends", b, "values", series)
        return Bars({"kind": "intervals", "starts": a, "ends": b}, series)

    @property
    def _time_x(self) -> bool:
        placement = self._placement
        if placement["kind"] == "at":
            return bool(placement["x"].time)
        if placement["kind"] == "intervals":
            return bool(placement["starts"].time)
        return False

    def color(self, color: ColorValue) -> Bars:
        other = self._copy()
        other._color = canonicalize_color(color)
        return other

    def color_by(self, categories: Iterable[str]) -> Bars:
        other = self._copy()
        other._color_by = _categories(categories)
        return other

    def label(self, label: str) -> Bars:
        other = self._copy()
        other._label = str(label)
        return other

    def base(self, base: Any) -> Bars:
        """Starts each bar at a per-bar base: stacked bars, waterfalls."""
        series = to_column(base, what="Bars base")
        _same_length("Bars", "base", series, "values", self._values)
        other = self._copy()
        other._base = series
        return other

    def horizontal(self) -> Bars:
        """Bands down the y axis in reading order, values along x."""
        other = self._copy()
        other._horizontal = True
        return other

    def _layer(self, start: int) -> Layer:
        columns: list[Column] = []
        index = start
        placement = self._placement
        wire: Any
        if placement["kind"] == "bands":
            wire = {"Bands": list(placement["categories"])}
        elif placement["kind"] == "spans":
            wire = {"Spans": {"start": placement["start"], "width": placement["width"]}}
        elif placement["kind"] == "intervals":
            wire = {"Intervals": {"starts": _ref(index), "ends": _ref(index + 1)}}
            columns.extend([placement["starts"], placement["ends"]])
            index += 2
        else:
            wire = {"At": {"x": _ref(index), "width": placement["width"]}}
            columns.append(placement["x"])
            index += 1
        values = _ref(index)
        columns.append(self._values)
        index += 1
        body: dict[str, Any] = {
            "placement": wire,
            "values": values,
            "color": color_json(self._color),
            "label": self._label,
        }
        if self._base is not None:
            body["base"] = _ref(index)
            columns.append(self._base)
            index += 1
        if self._color_by is not None:
            body["color_by"] = list(self._color_by)
        if self._horizontal:
            body["horizontal"] = True
        return {"Bars": body}, columns


class Area(Mark):
    """A fill from a baseline, or a band between two series."""

    __slots__ = ("_x", "_low", "_high", "_color", "_label", "_opacity", "_horizontal")

    def __init__(self, high: Column, x: Column | None = None, low: Column | None = None) -> None:
        self._x = x
        self._low = low
        self._high = high
        self._color: ColorValue | None = None
        self._label: str | None = None
        self._opacity: float | None = None
        self._horizontal = False

    @staticmethod
    def y(values: Any) -> Area:
        return Area(to_column(values, what="Area y"))

    @staticmethod
    def xy(x: Any, y: Any) -> Area:
        xs, ys = to_column(x, what="Area x"), to_column(y, what="Area y")
        _same_length("Area", "x", xs, "y", ys)
        return Area(ys, xs)

    @staticmethod
    def between(x: Any, low: Any, high: Any) -> Area:
        """The band between ``low`` and ``high`` over ``x``."""
        xs = to_column(x, what="Area x")
        lo = to_column(low, what="Area low")
        hi = to_column(high, what="Area high")
        _same_length("Area", "x", xs, "low", lo)
        _same_length("Area", "x", xs, "high", hi)
        return Area(hi, xs, lo)

    @property
    def _time_x(self) -> bool:
        return self._x is not None and self._x.time

    def color(self, color: ColorValue) -> Area:
        other = self._copy()
        other._color = canonicalize_color(color)
        return other

    def label(self, label: str) -> Area:
        other = self._copy()
        other._label = str(label)
        return other

    def opacity(self, opacity: float) -> Area:
        """A constant opacity in ``(0, 1]`` (pixel renders; cells shade)."""
        value = float(opacity)
        if not (math.isfinite(value) and 0.0 < value <= 1.0):
            raise ValueError("Area.opacity requires a value in (0, 1]")
        other = self._copy()
        other._opacity = value
        return other

    def horizontal(self) -> Area:
        """The fill extends along x from the baseline."""
        other = self._copy()
        other._horizontal = True
        return other

    def _layer(self, start: int) -> Layer:
        columns: list[Column] = []
        index = start
        x: Any = None
        low: Any = None
        if self._x is not None:
            x = _ref(index)
            columns.append(self._x)
            index += 1
        if self._low is not None:
            low = _ref(index)
            columns.append(self._low)
            index += 1
        columns.append(self._high)
        body: dict[str, Any] = {
            "x": x,
            "low": low,
            "high": _ref(index),
            "horizontal": self._horizontal,
            "color": color_json(self._color),
            "label": self._label,
        }
        if self._opacity is not None:
            body["opacity"] = self._opacity
        return {"Area": body}, columns


class Cells(Mark):
    """A value grid, an RGB image, or categorical class regions."""

    __slots__ = ("_columns", "_values", "_rgb", "_classes", "_extents", "_colormap", "_reduce", "_smooth")

    def __init__(self, columns: int) -> None:
        self._columns = columns
        self._values: Column | None = None
        self._rgb: list[list[int]] | None = None
        self._classes: list[str] | None = None
        self._extents: tuple[tuple[float, float], tuple[float, float]] | None = None
        self._colormap: Colormap | None = None
        self._reduce: Any = None
        self._smooth = False

    @staticmethod
    def matrix(columns: int, values: Any) -> Cells:
        """A row-major grid with ``columns`` values per row."""
        count = int(columns)
        if count < 1:
            raise ValueError("Cells.matrix requires a positive column count")
        series = to_column(values, what="Cells values")
        if len(series) % count != 0:
            raise ValueError("Cells.matrix requires columns to divide the value count evenly")
        cells = Cells(count)
        cells._values = series
        return cells

    @staticmethod
    def rgb(columns: int, pixels: Iterable[tuple[int, int, int]]) -> Cells:
        """An image: row-major ``(r, g, b)`` triples."""
        count = int(columns)
        if count < 1:
            raise ValueError("Cells.rgb requires a positive column count")
        listed = [[int(v) for v in pixel] for pixel in pixels]
        if len(listed) % count != 0:
            raise ValueError("Cells.rgb requires columns to divide the pixel count evenly")
        if any(len(p) != 3 or not all(0 <= v <= 255 for v in p) for p in listed):
            raise ValueError("Cells.rgb pixels are (r, g, b) triples of 0..=255")
        cells = Cells(count)
        cells._rgb = listed
        return cells

    @staticmethod
    def classes(columns: int, labels: Iterable[str]) -> Cells:
        """Categorical regions: one label per cell, colored by palette."""
        count = int(columns)
        if count < 1:
            raise ValueError("Cells.classes requires a positive column count")
        listed = [str(label) for label in labels]
        if len(listed) % count != 0:
            raise ValueError("Cells.classes requires columns to divide the label count evenly")
        cells = Cells(count)
        cells._classes = listed
        return cells

    def extents(self, x: tuple[float, float], y: tuple[float, float]) -> Cells:
        """The data rectangle the grid covers: ``(x0, x1)``, ``(y0, y1)``."""
        xs = (float(x[0]), float(x[1]))
        ys = (float(y[0]), float(y[1]))
        if not all(math.isfinite(v) for v in (*xs, *ys)) or xs[0] == xs[1] or ys[0] == ys[1]:
            raise ValueError("Cells.extents requires finite, non-empty bounds")
        other = self._copy()
        other._extents = (xs, ys)
        return other

    def colormap(self, colormap: Colormap) -> Cells:
        if not isinstance(colormap, Colormap):
            raise TypeError("Cells.colormap takes a Colormap")
        other = self._copy()
        other._colormap = colormap
        return other

    def reduce(self, reducer: Reducer) -> Cells:
        """How a grid denser than the raster reduces into each screen bucket."""
        if isinstance(reducer, dict):
            if set(reducer) != {"Percentile"}:
                raise ValueError("a reducer is a name or {'Percentile': q}")
            wire: Any = {"Percentile": float(reducer["Percentile"])}
        else:
            wire = _choice(str(reducer), _REDUCERS, "Cells.reduce")
        other = self._copy()
        other._reduce = wire
        return other

    def smooth(self) -> Cells:
        """Bilinear resampling when the grid is sparser than the raster."""
        other = self._copy()
        other._smooth = True
        return other

    def _layer(self, start: int) -> Layer:
        columns: list[Column] = []
        body: dict[str, Any] = {
            "columns": self._columns,
            "values": _ref(start) if self._values is not None else [],
            "extents": [list(self._extents[0]), list(self._extents[1])] if self._extents else None,
            "colormap": (self._colormap or Colormap.VIRIDIS).to_json(),
        }
        if self._values is not None:
            columns.append(self._values)
        if self._rgb is not None:
            body["rgb"] = [list(p) for p in self._rgb]
        if self._classes is not None:
            body["classes"] = list(self._classes)
        if self._reduce is not None and self._reduce != "Mean":
            body["reduce"] = self._reduce
        if self._smooth:
            body["smooth"] = True
        return {"Cells": body}, columns


class Range(Mark):
    """Intervals, with an optional body and marker: error bars, boxes, candles."""

    __slots__ = ("_placement", "_low", "_high", "_body", "_marker", "_color", "_label", "_color_by")

    def __init__(self, placement: dict[str, Any], low: Column, high: Column) -> None:
        self._placement = placement
        self._low = low
        self._high = high
        self._body: tuple[Column, Column] | None = None
        self._marker: Column | None = None
        self._color: ColorValue | None = None
        self._label: str | None = None
        self._color_by: list[str] | None = None

    @staticmethod
    def xy(x: Any, low: Any, high: Any) -> Range:
        xs = to_column(x, what="Range x")
        lo = to_column(low, what="Range low")
        hi = to_column(high, what="Range high")
        _same_length("Range", "x", xs, "low", lo)
        _same_length("Range", "x", xs, "high", hi)
        return Range({"kind": "numeric", "x": xs}, lo, hi)

    @staticmethod
    def y(low: Any, high: Any) -> Range:
        """Intervals at ``0, 1, 2, …``."""
        lo = to_column(low, what="Range low")
        hi = to_column(high, what="Range high")
        _same_length("Range", "low", lo, "high", hi)
        return Range({"kind": "numeric", "x": None}, lo, hi)

    @staticmethod
    def over(categories: Iterable[str], low: Any, high: Any) -> Range:
        """One interval per category."""
        names = _categories(categories)
        lo = to_column(low, what="Range low")
        hi = to_column(high, what="Range high")
        if len(names) != len(lo) or len(names) != len(hi):
            raise ValueError("Range.over requires one category per interval")
        return Range({"kind": "bands", "categories": names}, lo, hi)

    @property
    def _time_x(self) -> bool:
        x = self._placement.get("x")
        return x is not None and x.time

    def body(self, low: Any, high: Any) -> Range:
        """A thicker inner interval: a box plot's box, a candle's body."""
        lo = to_column(low, what="Range body low")
        hi = to_column(high, what="Range body high")
        if len(lo) != len(self._low) or len(hi) != len(self._low):
            raise ValueError("Range.body requires series matching the range length")
        other = self._copy()
        other._body = (lo, hi)
        return other

    def marker(self, values: Any) -> Range:
        """A tick inside each interval: a median, a close."""
        series = to_column(values, what="Range marker")
        if len(series) != len(self._low):
            raise ValueError("Range.marker requires a series matching the range length")
        other = self._copy()
        other._marker = series
        return other

    def color(self, color: ColorValue) -> Range:
        other = self._copy()
        other._color = canonicalize_color(color)
        return other

    def label(self, label: str) -> Range:
        other = self._copy()
        other._label = str(label)
        return other

    def color_by(self, categories: Iterable[str]) -> Range:
        other = self._copy()
        other._color_by = _categories(categories)
        return other

    def _layer(self, start: int) -> Layer:
        columns: list[Column] = []
        index = start
        placement = self._placement
        wire: Any
        if placement["kind"] == "bands":
            wire = {"Bands": list(placement["categories"])}
        elif placement["x"] is not None:
            wire = {"Numeric": _ref(index)}
            columns.append(placement["x"])
            index += 1
        else:
            wire = {"Numeric": None}
        low = _ref(index)
        columns.append(self._low)
        index += 1
        high = _ref(index)
        columns.append(self._high)
        index += 1
        body: dict[str, Any] = {
            "placement": wire,
            "low": low,
            "high": high,
            "color": color_json(self._color),
            "label": self._label,
        }
        if self._body is not None:
            body["body"] = [_ref(index), _ref(index + 1)]
            columns.extend(self._body)
            index += 2
        if self._marker is not None:
            body["marker"] = _ref(index)
            columns.append(self._marker)
            index += 1
        if self._color_by is not None:
            body["color_by"] = list(self._color_by)
        return {"Range": body}, columns


class Rule(Mark):
    """A reference line at one value, or a span between two."""

    __slots__ = ("_orientation", "_color", "_label", "_dash")

    def __init__(self, orientation: dict[str, Any]) -> None:
        self._orientation = orientation
        self._color: ColorValue | None = None
        self._label: str | None = None
        self._dash: str = "Solid"

    @staticmethod
    def h(y: float) -> Rule:
        """A horizontal line at ``y``."""
        return Rule({"Horizontal": _scalar(y, "Rule.h")})

    @staticmethod
    def v(x: Any) -> Rule:
        """A vertical line at ``x`` (a datetime is accepted)."""
        return Rule({"Vertical": _scalar(x, "Rule.v")})

    @staticmethod
    def h_span(low: float, high: float) -> Rule:
        """A horizontal band washed between two y values."""
        return Rule({"HorizontalSpan": [_scalar(low, "Rule.h_span"), _scalar(high, "Rule.h_span")]})

    @staticmethod
    def v_span(low: Any, high: Any) -> Rule:
        """A vertical band between two x values: a recession, a warm-up phase."""
        return Rule({"VerticalSpan": [_scalar(low, "Rule.v_span"), _scalar(high, "Rule.v_span")]})

    def color(self, color: ColorValue) -> Rule:
        other = self._copy()
        other._color = canonicalize_color(color)
        return other

    def label(self, label: str) -> Rule:
        other = self._copy()
        other._label = str(label)
        return other

    def dash(self, dash: Dash) -> Rule:
        other = self._copy()
        other._dash = _choice(dash, _DASHES, "Rule.dash")
        return other

    def _layer(self, start: int) -> Layer:
        body: dict[str, Any] = {
            "orientation": copy.deepcopy(self._orientation),
            "color": color_json(self._color),
            "label": self._label,
        }
        if self._dash != "Solid":
            body["dash"] = self._dash
        return {"Rule": body}, []


class Text(Mark):
    """An annotation at data coordinates."""

    __slots__ = ("_x", "_y", "_text", "_color", "_align")

    def __init__(self, x: float, y: float, text: str) -> None:
        self._x = x
        self._y = y
        self._text = text
        self._color: ColorValue | None = None
        self._align: str = "Left"

    @staticmethod
    def at(x: Any, y: float, text: str) -> Text:
        """``text`` anchored at ``(x, y)``; x may be a datetime."""
        return Text(_scalar(x, "Text.at"), _scalar(y, "Text.at"), str(text))

    def color(self, color: ColorValue) -> Text:
        other = self._copy()
        other._color = canonicalize_color(color)
        return other

    def align(self, align: Align) -> Text:
        """``"Left"`` starts at the anchor; ``"Center"`` and ``"Right"`` end there."""
        other = self._copy()
        other._align = _choice(align, _ALIGNS, "Text.align")
        return other

    def _layer(self, start: int) -> Layer:
        body: dict[str, Any] = {
            "x": self._x,
            "y": self._y,
            "text": self._text,
            "color": color_json(self._color),
        }
        if self._align != "Left":
            body["align"] = self._align
        return {"Text": body}, []
