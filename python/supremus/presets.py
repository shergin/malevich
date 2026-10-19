"""Presets: chart types as plain functions over the grammar.

A preset is a proven composition. ``line``, ``scatter``, ``bar``, and
``sparkline`` are spelled here with the marks. The statistics presets run in
the engine (one KDE, one set of quartiles, one histogram binner, shared with
Rust and JS) and come back as a plot whose layers are their expansion. Each
keyword argument is a field of the crate's ``_with`` options struct.
"""

from __future__ import annotations

import json
from collections.abc import Iterable
from typing import Any

from . import _native
from ._engine import call, dumps
from .color import Colormap
from .mark import Bars, Line, Points
from .plot import Plot
from .series import to_column

__all__ = [
    "Whiskers",
    "bar",
    "box_plot",
    "contour",
    "contourf",
    "density",
    "describe",
    "ecdf",
    "error_bars",
    "error_bars_asymmetric",
    "heatmap",
    "hist",
    "hist2d",
    "line",
    "preset_names",
    "quiver",
    "scatter",
    "sparkline",
    "stairs",
    "table",
    "trend",
    "violin",
]


def line(values: Any) -> Plot:
    """A line over ``values`` at ``0, 1, 2, …``."""
    return Plot().layer(Line.y(values))


def scatter(x: Any, y: Any) -> Plot:
    """Points at ``(x, y)`` pairs."""
    return Plot().layer(Points.xy(x, y))


def bar(categories: Iterable[str], values: Any) -> Plot:
    """One labeled bar per category, rising from zero."""
    return Plot().layer(Bars.new(categories, values))


def sparkline(values: Any) -> Plot:
    """One bar per value with no axes; render it in a one-row frame."""
    return Plot().layer(Bars.spans(0.0, 1.0, values)).axes(False)


def stairs(values: Any, *, direction: str = "Post") -> Plot:
    """A step line; ``direction`` is ``"Post"``, ``"Pre"``, or ``"Mid"``."""
    return _preset("stairs", {"direction": direction}, [values])


def hist(
    values: Any,
    *,
    max_bins: int = 60,
    normalization: str = "Count",
    cumulative: bool = False,
) -> Plot:
    """A histogram on an integer count axis; ``normalization`` is ``"Count"``,
    ``"Probability"``, ``"Percent"``, or ``"Density"``."""
    return _preset(
        "hist",
        {"max_bins": int(max_bins), "normalization": normalization, "cumulative": bool(cumulative)},
        [values],
    )


def ecdf(values: Any, *, band: float | None = None) -> Plot:
    """The empirical CDF; ``band`` draws the DKW band at that confidence level."""
    return _preset("ecdf", _drop_none({"band": band}), [values])


def density(
    values: Any,
    *,
    samples: int = 256,
    bandwidth: float | None = None,
    bandwidth_scale: float | None = None,
    cumulative: bool = False,
) -> Plot:
    """A kernel density estimate (Silverman's bandwidth unless fixed or scaled)."""
    return _preset(
        "density",
        _drop_none(
            {
                "samples": int(samples),
                "bandwidth": bandwidth,
                "bandwidth_scale": bandwidth_scale,
                "cumulative": cumulative or None,
            }
        ),
        [values],
    )


def trend(x: Any, y: Any, *, band: float | None = None, band_samples: int | None = None) -> Plot:
    """A scatter with its least-squares line; ``band`` adds a confidence band."""
    return _preset("trend", _drop_none({"band": band, "band_samples": band_samples}), [x, y])


def hist2d(
    x: Any,
    y: Any,
    *,
    columns: int = 48,
    rows: int = 32,
    colormap: Colormap | None = None,
    colorbar: bool | None = None,
) -> Plot:
    """Two-dimensional bin counts as cells."""
    return _preset(
        "hist2d",
        _drop_none(
            {
                "columns": int(columns),
                "rows": int(rows),
                "colormap": _colormap(colormap),
                "colorbar": colorbar,
            }
        ),
        [x, y],
    )


def heatmap(
    columns: int, values: Any, *, colormap: Colormap | None = None, colorbar: bool | None = None
) -> Plot:
    """A row-major grid of ``values`` with ``columns`` per row, with a colorbar."""
    return _preset(
        "heatmap",
        _drop_none({"columns": int(columns), "colormap": _colormap(colormap), "colorbar": colorbar}),
        [values],
    )


def contour(
    columns: int,
    values: Any,
    *,
    levels: int | Iterable[float] | None = None,
    colormap: Colormap | None = None,
) -> Plot:
    """Iso-lines of a grid; ``levels`` is a target count or explicit values."""
    return _preset(
        "contour",
        _drop_none({"columns": int(columns), "levels": _levels(levels), "colormap": _colormap(colormap)}),
        [values],
    )


def contourf(
    columns: int,
    values: Any,
    *,
    levels: int | Iterable[float] | None = None,
    colormap: Colormap | None = None,
) -> Plot:
    """Filled contours: a heatmap under a colormap split at the levels."""
    return _preset(
        "contourf",
        _drop_none({"columns": int(columns), "levels": _levels(levels), "colormap": _colormap(colormap)}),
        [values],
    )


class Whiskers:
    """How far a box plot's whiskers reach."""

    MINMAX = "MinMax"

    @staticmethod
    def tukey(k: float = 1.5) -> dict[str, float]:
        """``k`` × IQR beyond the box (the default, with ``k = 1.5``)."""
        return {"Tukey": float(k)}

    @staticmethod
    def percentiles(low: float, high: float) -> dict[str, list[float]]:
        """Whiskers at two quantile positions in ``[0, 1]``: ``percentiles(0.05, 0.95)``."""
        return {"Percentiles": [float(low), float(high)]}


def box_plot(categories: Iterable[str], groups: Iterable[Any], *, whiskers: Any = None) -> Plot:
    """One box per group over its category."""
    names = [str(c) for c in categories]
    return _preset("box_plot", _drop_none({"categories": names, "whiskers": whiskers}), list(groups))


def violin(
    categories: Iterable[str],
    groups: Iterable[Any],
    *,
    samples: int = 128,
    bandwidth: float | None = None,
    bandwidth_scale: float | None = None,
) -> Plot:
    """One mirrored density per group over its category."""
    names = [str(c) for c in categories]
    return _preset(
        "violin",
        _drop_none(
            {
                "categories": names,
                "samples": int(samples),
                "bandwidth": bandwidth,
                "bandwidth_scale": bandwidth_scale,
            }
        ),
        list(groups),
    )


def describe(names: Iterable[str], groups: Iterable[Any], *, histogram: int | None = None) -> Plot:
    """A summary table: count, mean, sd, min, quartiles, max per group."""
    return _preset(
        "describe", _drop_none({"names": [str(n) for n in names], "histogram": histogram}), list(groups)
    )


def table(
    rows: Iterable[str], columns: Iterable[str], values: Any, *, colormap: Colormap | None = None
) -> Plot:
    """A labeled matrix of numbers, optionally shaded by a colormap."""
    return _preset(
        "table",
        _drop_none(
            {"rows": [str(r) for r in rows], "columns": [str(c) for c in columns], "colormap": _colormap(colormap)}
        ),
        [values],
    )


def error_bars(x: Any, y: Any, error: Any) -> Plot:
    """Points with symmetric ``±error`` intervals."""
    return _preset("error_bars", {}, [x, y, error])


def error_bars_asymmetric(x: Any, y: Any, minus: Any, plus: Any) -> Plot:
    """Points reaching down by ``minus`` and up by ``plus``."""
    return _preset("error_bars_asymmetric", {}, [x, y, minus, plus])


def quiver(x: Any, y: Any, u: Any, v: Any) -> Plot:
    """Arrows from ``(x, y)`` along ``(u, v)``."""
    return _preset("quiver", {}, [x, y, u, v])


def preset_names() -> list[str]:
    """The presets the engine expands by name."""
    return list(_native.preset_names())


def _preset(name: str, options: dict[str, Any], series: list[Any]) -> Plot:
    columns = [to_column(values, what=f"{name} series {index}") for index, values in enumerate(series)]
    document = call(_native.expand_preset, name, dumps(options), [c.data for c in columns])
    plot = Plot.from_document(json.loads(document))
    if columns and columns[0].time and name in {"stairs", "trend"}:
        return plot.time_x()
    return plot


def _drop_none(options: dict[str, Any]) -> dict[str, Any]:
    return {key: value for key, value in options.items() if value is not None}


def _colormap(colormap: Colormap | None) -> dict[str, Any] | None:
    if colormap is None:
        return None
    if not isinstance(colormap, Colormap):
        raise TypeError("colormap must be a Colormap")
    return colormap.to_json()


def _levels(levels: int | Iterable[float] | None) -> Any:
    if levels is None:
        return None
    if isinstance(levels, int):
        return levels
    return [float(v) for v in levels]
