"""The pandas plotting backend: ``pd.set_option("plotting.backend", "supremus")``.

pandas hands this module the raw ``Series`` or ``DataFrame`` and a ``kind``;
each kind maps onto a preset or a two-layer composition and returns a
:class:`~supremus.Plot`. Only the keywords a plot has a home for are
accepted: ``title``, ``xlabel``, ``ylabel``, ``logx``, ``logy``, ``bins``,
``gridsize``, ``stacked``. Anything else is a ``TypeError`` naming it.
"""

from __future__ import annotations

from typing import Any

from .mark import Area, Bars, Line
from .plot import Plot
from .presets import box_plot, density, hist, hist2d, scatter as scatter_preset

__all__ = ["boxplot_frame", "hist_frame", "hist_series", "plot"]

_FURNITURE = {"title", "xlabel", "ylabel", "logx", "logy"}
_ACCEPTED = _FURNITURE | {"bins", "gridsize", "stacked", "ax", "figsize", "grid", "legend", "label"}


def plot(data: Any, x: Any = None, y: Any = None, kind: str = "line", **kwargs: Any) -> Plot:
    """pandas' entry point."""
    import pandas as pd

    kwargs = {k: v for k, v in kwargs.items() if v is not None}
    bw_method = kwargs.pop("bw_method", None)
    unknown = sorted(set(kwargs) - _ACCEPTED)
    if unknown:
        raise TypeError(
            f"supremus backend does not take {', '.join(unknown)}; a plot is marks, scales, and furniture"
        )
    kind = {"density": "kde"}.get(kind, kind)
    frame = data.to_frame(name=data.name if data.name is not None else "value") if isinstance(data, pd.Series) else data
    if kind == "pie":
        raise NotImplementedError("pie is not a terminal chart; use kind='bar'")
    if kind == "scatter":
        chart = _scatter(frame, x, y)
    elif kind == "hexbin":
        chart = _hexbin(frame, x, y, kwargs.get("gridsize"))
    else:
        xs, x_name = _x_of(frame, x)
        columns = _y_columns(frame, y, exclude=x)
        if kind == "line":
            chart = _lines(frame, xs, columns)
        elif kind == "area":
            chart = _areas(frame, xs, columns, bool(kwargs.get("stacked", True)))
        elif kind in {"bar", "barh"}:
            chart = _bars(frame, columns, kind == "barh", bool(kwargs.get("stacked", False)))
        elif kind == "hist":
            chart = _hist(frame, columns, kwargs.get("bins"))
        elif kind == "box":
            chart = box_plot([str(c) for c in columns], [frame[c] for c in columns])
        elif kind == "kde":
            scale = float(bw_method) if isinstance(bw_method, (int, float)) else None
            chart = _one(frame, columns, "kde", lambda values: density(values, bandwidth_scale=scale))
        else:
            raise ValueError(f"unknown plot kind {kind!r}")
        if kind in {"line", "area"} and x_name is not None and kwargs.get("xlabel") is None:
            chart = chart.x_label(str(x_name))
    return _furnish(chart, kwargs)


def hist_series(self: Any, bins: int = 10, **kwargs: Any) -> Plot:
    """``series.hist()``."""
    return _furnish(hist(self, max_bins=int(bins)), _only_furniture(kwargs))


def hist_frame(data: Any, column: Any = None, bins: int = 10, **kwargs: Any) -> Plot:
    """``df.hist()``: one column at a time in a terminal."""
    columns = _y_columns(data, column, exclude=None)
    return _furnish(_hist(data, columns, bins), _only_furniture(kwargs))


def boxplot_frame(data: Any, column: Any = None, **kwargs: Any) -> Plot:
    """``df.boxplot()``."""
    columns = _y_columns(data, column, exclude=None)
    return _furnish(box_plot([str(c) for c in columns], [data[c] for c in columns]), _only_furniture(kwargs))


def _only_furniture(kwargs: dict[str, Any]) -> dict[str, Any]:
    return {k: v for k, v in kwargs.items() if k in _FURNITURE}


def _furnish(chart: Plot, kwargs: dict[str, Any]) -> Plot:
    if kwargs.get("title") is not None:
        chart = chart.title(str(kwargs["title"]))
    if kwargs.get("xlabel") is not None:
        chart = chart.x_label(str(kwargs["xlabel"]))
    if kwargs.get("ylabel") is not None:
        chart = chart.y_label(str(kwargs["ylabel"]))
    if kwargs.get("logx"):
        chart = chart.log_x()
    if kwargs.get("logy"):
        chart = chart.log_y()
    return chart


def _x_of(frame: Any, x: Any) -> tuple[Any, Any]:
    """The x series (a column or the index) and its name; ``None`` for positions."""
    import pandas as pd

    if x is not None:
        return frame[x], x
    index = frame.index
    if isinstance(index, pd.RangeIndex) and index.start == 0 and index.step == 1:
        return None, None
    if isinstance(index, pd.DatetimeIndex) or pd.api.types.is_numeric_dtype(index):
        return index, index.name
    return None, None


def _y_columns(frame: Any, y: Any, exclude: Any) -> list[Any]:
    import pandas as pd

    if y is not None:
        return list(y) if isinstance(y, (list, tuple)) else [y]
    columns = [c for c in frame.columns if c != exclude and pd.api.types.is_numeric_dtype(frame[c])]
    if not columns:
        raise TypeError("no numeric columns to plot")
    return columns


def _lines(frame: Any, xs: Any, columns: list[Any]) -> Plot:
    chart = Plot()
    for column in columns:
        mark = Line.xy(xs, frame[column]) if xs is not None else Line.y(frame[column])
        chart = chart.layer(mark.label(str(column)))
    return chart


def _areas(frame: Any, xs: Any, columns: list[Any], stacked: bool) -> Plot:
    chart = Plot()
    running = None
    for column in columns:
        values = frame[column].to_numpy(dtype="float64", na_value=float("nan"))
        if stacked and running is not None:
            top = running + values
            mark = Area.between(xs if xs is not None else range(len(values)), running, top)
            running = top
        else:
            mark = Area.xy(xs, values) if xs is not None else Area.y(values)
            running = values if stacked else None
        chart = chart.layer(mark.label(str(column)))
    return chart


def _bars(frame: Any, columns: list[Any], horizontal: bool, stacked: bool) -> Plot:
    categories = [str(label) for label in frame.index]
    chart = Plot()
    base = None
    for column in columns:
        values = frame[column].to_numpy(dtype="float64", na_value=float("nan"))
        if len(columns) > 1 and not stacked:
            raise TypeError("supremus bars take one column, or stacked=True for several")
        mark = Bars.new(categories, values).label(str(column))
        if base is not None:
            mark = mark.base(base)
            base = base + values
        elif stacked:
            base = values
        if horizontal:
            mark = mark.horizontal()
        chart = chart.layer(mark)
    return chart


def _hist(frame: Any, columns: list[Any], bins: Any) -> Plot:
    return _one(frame, columns, "hist", lambda values: hist(values, max_bins=int(bins or 60)))


def _one(frame: Any, columns: list[Any], kind: str, make: Any) -> Plot:
    if len(columns) != 1:
        raise TypeError(f"supremus {kind} takes one column at a time (got {len(columns)})")
    return make(frame[columns[0]])


def _scatter(frame: Any, x: Any, y: Any) -> Plot:
    if x is None or y is None:
        raise TypeError("scatter requires x and y")
    chart = scatter_preset(frame[x], frame[y])
    return chart.x_label(str(x)).y_label(str(y))


def _hexbin(frame: Any, x: Any, y: Any, gridsize: Any) -> Plot:
    if x is None or y is None:
        raise TypeError("hexbin requires x and y")
    size = int(gridsize or 48)
    chart = hist2d(frame[x], frame[y], columns=size, rows=max(4, size * 2 // 3))
    return chart.x_label(str(x)).y_label(str(y))

