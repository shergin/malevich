"""Plot and Grid: the retained spec, and every way to draw it.

A plot is a value. Each builder method returns a new plot; nothing here reads
the environment except the two named conveniences, ``str(plot)`` and
``repr(plot)``, which detect a frame the way ``println!("{plot}")`` does in
Rust. ``render``, ``raster``, ``mapping``, and the cards are pure functions of
the plot and a frame.
"""

from __future__ import annotations

import copy
import math
from collections.abc import Iterable
from typing import Any

from . import _native
from ._engine import call, dumps
from .color import Palette, Unit, scale_json
from .frame import Frame
from .graphics import Graphics, detect_graphics
from .mapping import Mapping, Viewport, Windows
from .mark import Mark
from .raster import Raster, unpack
from .series import Column, to_column

__all__ = ["DocumentJSON", "Grid", "Plot"]

DocumentJSON = dict[str, Any]

#: The notebook card's frame: 100×26 quadrants on the dark card, the Evcxr default.
CARD_FRAME = Frame.portable(100, 26)


def _empty_spec() -> dict[str, Any]:
    return {
        "layers": [],
        "title": None,
        "x": "Auto",
        "y": "Auto",
        "x_label": None,
        "y_label": None,
        "x_domain": None,
        "y_domain": None,
        "colorbar": False,
    }


def _finite(value: float, who: str) -> float:
    number = float(value)
    if not math.isfinite(number):
        raise ValueError(f"{who} requires a finite bound")
    return number


def _ends(domain: Any) -> dict[str, float]:
    if domain is None:
        return {}
    if isinstance(domain, list):
        return {"min": domain[0], "max": domain[1]}
    return dict(domain)


def _sided(domain: Any, end: str, value: float, who: str) -> Any:
    ends = {**_ends(domain), end: _finite(value, who)}
    if "min" in ends and "max" in ends:
        return [ends["min"], ends["max"]]
    return ends


def _as_pair(value: Any) -> tuple[float, float] | None:
    if value is None:
        return None
    low, high = float(value[0]), float(value[1])
    if not (math.isfinite(low) and math.isfinite(high)):
        return None
    return (low, high)


def _inline(column: Column) -> list[float | None]:
    data = column.data
    values = data.tolist() if hasattr(data, "tolist") else list(data)
    return [None if v is None or (isinstance(v, float) and v != v) else float(v) for v in values]


def _rebase(value: Any, offset: int) -> Any:
    """Shifts every ``{"col": n}`` reference in a spec by ``offset``."""
    if isinstance(value, list):
        return [_rebase(item, offset) for item in value]
    if isinstance(value, dict):
        if set(value) == {"col"} and isinstance(value["col"], int):
            return {"col": value["col"] + offset}
        return {key: _rebase(item, offset) for key, item in value.items()}
    return value


class Plot:
    """Layers on shared scales, plus furniture. Immutable: every method returns a new plot."""

    __slots__ = ("_spec", "_columns")

    def __init__(self) -> None:
        self._spec: dict[str, Any] = _empty_spec()
        self._columns: list[Column] = []

    @staticmethod
    def from_document(document: DocumentJSON, columns: Iterable[Any] = ()) -> Plot:
        """A plot from a version-1 plot document, with any ``{"col": n}`` columns bound."""
        if document.get("kind") != "plot":
            raise ValueError("Plot.from_document requires a plot document")
        if document.get("version") != 1:
            raise ValueError(f"unsupported document version {document.get('version')!r}")
        plot = Plot()
        plot._spec = {**_empty_spec(), **copy.deepcopy(document["spec"])}
        plot._columns = [to_column(c) for c in columns]
        return plot

    def _clone(self) -> Plot:
        other = Plot.__new__(Plot)
        other._spec = {
            **self._spec,
            "layers": list(self._spec["layers"]),
            "x_domain": copy.copy(self._spec["x_domain"]),
            "y_domain": copy.copy(self._spec["y_domain"]),
        }
        other._columns = list(self._columns)
        return other

    def _with(self, **patch: Any) -> Plot:
        other = self._clone()
        other._spec.update(patch)
        return other

    # ---- layers and scales -------------------------------------------------

    def layer(self, mark: Mark) -> Plot:
        """Stacks a mark on the shared scales. A mark whose x is time sets the
        x scale to ``Time`` unless a scale was chosen already."""
        if not isinstance(mark, Mark):
            raise TypeError(f"Plot.layer takes a mark, not {type(mark).__name__}")
        other = self._clone()
        layer, columns = mark._layer(len(other._columns))
        other._columns.extend(columns)
        other._spec["layers"].append(layer)
        if mark._time_x and other._spec["x"] == "Auto":
            other._spec["x"] = "Time"
        return other

    def title(self, title: str) -> Plot:
        return self._with(title=str(title))

    def x_label(self, label: str) -> Plot:
        return self._with(x_label=str(label))

    def y_label(self, label: str) -> Plot:
        return self._with(y_label=str(label))

    def x_domain(self, low: float, high: float) -> Plot:
        """Fixes both ends of the x axis."""
        a, b = _finite(low, "Plot.x_domain"), _finite(high, "Plot.x_domain")
        return self._with(x_domain=[min(a, b), max(a, b)])

    def y_domain(self, low: float, high: float) -> Plot:
        a, b = _finite(low, "Plot.y_domain"), _finite(high, "Plot.y_domain")
        return self._with(y_domain=[min(a, b), max(a, b)])

    def x_min(self, low: float) -> Plot:
        """Fixes the x axis's low end; the high end fits the data."""
        return self._with(x_domain=_sided(self._spec["x_domain"], "min", low, "Plot.x_min"))

    def x_max(self, high: float) -> Plot:
        return self._with(x_domain=_sided(self._spec["x_domain"], "max", high, "Plot.x_max"))

    def y_min(self, low: float) -> Plot:
        return self._with(y_domain=_sided(self._spec["y_domain"], "min", low, "Plot.y_min"))

    def y_max(self, high: float) -> Plot:
        return self._with(y_domain=_sided(self._spec["y_domain"], "max", high, "Plot.y_max"))

    def x_scale(self, scale: Any) -> Plot:
        """``"Auto"``, ``"Linear"``, ``"Integer"``, ``"Log"``, ``"Time"``, or ``Scale.bands([...])``."""
        return self._with(x=scale_json(scale))

    def y_scale(self, scale: Any) -> Plot:
        return self._with(y=scale_json(scale))

    def time_x(self) -> Plot:
        """Unix seconds on x, calendar ticks."""
        return self.x_scale("Time")

    def log_x(self) -> Plot:
        return self.x_scale("Log")

    def log_y(self) -> Plot:
        return self.y_scale("Log")

    def x_unit(self, unit: Any) -> Plot:
        """What x labels are counted in: ``Unit.si("B")``, ``Unit.BYTES``, ``Unit.suffix("%")``."""
        return self._with(x_unit=_unit_json(unit))

    def y_unit(self, unit: Any) -> Plot:
        return self._with(y_unit=_unit_json(unit))

    def axes(self, shown: bool) -> Plot:
        """Whether axes are drawn at all; ``axes(False)`` is a sparkline's furniture."""
        return self._with(axes=bool(shown))

    def colorbar(self) -> Plot:
        return self._with(colorbar=True)

    def palette(self, palette: Palette) -> Plot:
        """The categorical palette ``color_by`` groups take."""
        if not isinstance(palette, Palette):
            raise TypeError("Plot.palette takes a Palette")
        return self._with(palette=palette.to_json())

    def viewport(self, view: Viewport | Windows | None) -> Plot:
        """Applies an axis window pair as domains; a missing axis stays as it is."""
        if view is None:
            return self
        windows = view.windows() if isinstance(view, Viewport) else dict(view)
        plot: Plot = self
        x = _as_pair(windows.get("x"))
        y = _as_pair(windows.get("y"))
        if x is not None:
            plot = plot.x_domain(*x)
        if y is not None:
            plot = plot.y_domain(*y)
        return plot

    # ---- the spec ----------------------------------------------------------

    @property
    def layers(self) -> list[dict[str, Any]]:
        """The layers' wire forms, with ``{"col": n}`` references into :meth:`columns`."""
        return copy.deepcopy(self._spec["layers"])

    def to_json(self) -> DocumentJSON:
        """The version-1 document, series as ``{"col": n}`` references into :meth:`columns`."""
        return {"version": 1, "kind": "plot", "spec": copy.deepcopy(self._spec)}

    def columns(self) -> list[Any]:
        """The bound series, in reference order."""
        return [c.data for c in self._columns]

    def to_document(self) -> str:
        """A standalone JSON document with every series inlined (gaps as ``null``)."""
        inlined = [_inline(c) for c in self._columns]

        def splice(value: Any) -> Any:
            if isinstance(value, list):
                return [splice(item) for item in value]
            if isinstance(value, dict):
                if set(value) == {"col"} and isinstance(value["col"], int):
                    return inlined[value["col"]]
                return {key: splice(item) for key, item in value.items()}
            return value

        return dumps({"version": 1, "kind": "plot", "spec": splice(self._spec)})

    def validate(self) -> None:
        """Raises :class:`SupremusError` if the engine refuses this spec."""
        call(_native.validate_columns, dumps(self.to_json()), self.columns())

    # ---- rendering ---------------------------------------------------------

    def _request(self, frame: Frame) -> tuple[str, str, list[Any]]:
        if not isinstance(frame, Frame):
            raise TypeError(f"a Frame is required, not {type(frame).__name__}")
        return dumps(self.to_json()), dumps(frame.to_json()), self.columns()

    def render(self, frame: Frame) -> str:
        """The chart as a string of glyphs and SGR for ``frame``. Pure."""
        return call(_native.render_columns, *self._request(frame))

    def raster(self, frame: Frame) -> Raster:
        """The cell grid under :meth:`render`, chrome included."""
        return unpack(call(_native.raster_columns, *self._request(frame)))

    def mapping(self, frame: Frame) -> Mapping:
        """The resolved geometry of one render."""
        return Mapping(call(_native.mapping_columns, *self._request(frame)))

    def to_html(self, frame: Frame | None = None) -> str:
        """The self-contained HTML terminal card (default ``Frame.portable(100, 26)``)."""
        return call(_native.html_columns, *self._request(frame or CARD_FRAME))

    def to_svg(self, frame: Frame | None = None) -> str:
        """The SVG terminal card."""
        return call(_native.svg_columns, *self._request(frame or CARD_FRAME))

    def to_svg_pixels(self, frame: Frame | None = None, cell_size: tuple[int, int] = (8, 16)) -> str:
        """The SVG card with the panel as device pixels."""
        document, frame_json, columns = self._request(frame or CARD_FRAME)
        return call(_native.svg_pixels_columns, document, frame_json, columns, *cell_size)

    def render_pixels(self, frame: Frame, graphics: Graphics) -> str:
        """The hybrid pixel render for a named protocol: text chrome, image panel."""
        document, frame_json, columns = self._request(frame)
        return call(
            _native.render_pixels_columns,
            document,
            frame_json,
            columns,
            graphics.protocol,
            *graphics.cell_size,
        )

    def render_best(self, frame: Frame) -> str:
        """Pixels when the environment advertises a protocol, else cells. Reads
        the environment; not for snapshots."""
        graphics = detect_graphics()
        if graphics is None:
            return self.render(frame)
        try:
            return self.render_pixels(frame, graphics)
        except Exception:  # noqa: BLE001 - the ladder degrades, it never fails
            return self.render(frame)

    # ---- hosts -------------------------------------------------------------

    def __str__(self) -> str:
        return self.render(Frame.detect())

    def __repr__(self) -> str:
        return self.render(Frame.detect())

    def _repr_pretty_(self, printer: Any, cycle: bool) -> None:
        printer.text(repr(self))

    def _repr_mimebundle_(
        self, include: Iterable[str] | None = None, exclude: Iterable[str] | None = None
    ) -> dict[str, str]:
        """``text/plain`` and the HTML card; Jupyter, VS Code, and marimo pick the richest."""
        bundle = {"text/plain": str(self), "text/html": self.to_html()}
        if include is not None:
            bundle = {k: v for k, v in bundle.items() if k in set(include)}
        if exclude is not None:
            bundle = {k: v for k, v in bundle.items() if k not in set(exclude)}
        return bundle

    def _repr_html_(self) -> str:
        return self.to_html()

    def _mime_(self) -> tuple[str, str]:
        """marimo's display protocol."""
        return ("text/html", self.to_html())

    def __rich_console__(self, console: Any, options: Any) -> Any:
        from ._rich import segments

        yield from segments(self, console, options)

    def __rich_measure__(self, console: Any, options: Any) -> Any:
        from rich.measure import Measurement

        return Measurement(min(20, options.max_width), options.max_width)


class Grid:
    """A row-major grid of plots, each pane sized by the frame."""

    __slots__ = ("_columns_count", "_plots")

    def __init__(self, columns: int) -> None:
        if int(columns) < 1:
            raise ValueError("Grid requires at least one column")
        self._columns_count = int(columns)
        self._plots: list[Plot] = []

    def with_(self, plot: Plot) -> Grid:
        """The grid with one more pane."""
        if not isinstance(plot, Plot):
            raise TypeError("Grid.with_ takes a Plot")
        other = Grid(self._columns_count)
        other._plots = [*self._plots, plot]
        return other

    @property
    def plots(self) -> list[Plot]:
        return list(self._plots)

    def to_json(self) -> DocumentJSON:
        """The version-1 grid document, column references rebased onto :meth:`columns`."""
        specs = []
        offset = 0
        for plot in self._plots:
            specs.append(_rebase(copy.deepcopy(plot._spec), offset))
            offset += len(plot._columns)
        return {"version": 1, "kind": "grid", "spec": {"columns": self._columns_count, "plots": specs}}

    def columns(self) -> list[Any]:
        return [c.data for plot in self._plots for c in plot._columns]

    def validate(self) -> None:
        call(_native.validate_columns, dumps(self.to_json()), self.columns())

    def render(self, frame: Frame) -> str:
        if not isinstance(frame, Frame):
            raise TypeError(f"a Frame is required, not {type(frame).__name__}")
        return call(_native.render_columns, dumps(self.to_json()), dumps(frame.to_json()), self.columns())

    def __str__(self) -> str:
        return self.render(Frame.detect())

    def __repr__(self) -> str:
        return self.render(Frame.detect())

    def _repr_pretty_(self, printer: Any, cycle: bool) -> None:
        printer.text(repr(self))

    def __rich_console__(self, console: Any, options: Any) -> Any:
        from ._rich import grid_segments

        yield from grid_segments(self, console, options)


def _unit_json(unit: Any) -> Any:
    if isinstance(unit, dict):
        if set(unit) <= {"Si", "Suffix"} and len(unit) == 1:
            return unit
        raise TypeError(f"not a unit: {unit!r}")
    if isinstance(unit, str):
        if unit in {Unit.PLAIN, Unit.BYTES}:
            return unit
        if unit.lower() == "bytes":
            return Unit.BYTES
        if unit.lower() == "plain":
            return Unit.PLAIN
        # A bare string is a suffix: `y_unit("%")`.
        return Unit.suffix(unit)
    raise TypeError(f"not a unit: {type(unit).__name__}")

