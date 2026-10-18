"""Terminal plotting for Python: a small grammar of marks, honest axes, millions of points.

The engine is the `malevich <https://crates.io/crates/malevich>`_ Rust crate.
A plot is a value; hand it a frame and it draws. ``print(line([1, 5, 2, 8]))``
is the front door; ``Plot().layer(Line.xy(x, y))`` is the grammar.
"""

from ._engine import ENGINE_VERSION, SupremusError
from .color import Color, Colormap, Palette, Scale, Unit
from .frame import Charset, ColorMode, Frame, Theme
from .graphics import Graphics, Protocol, detect_graphics
from .mapping import Mapping, Panel, Viewport, Windows
from .mark import Align, Area, Bars, Cells, Dash, Line, LineStyle, Mark, Points, PointStyle, Range, Rule, Text
from .plot import DocumentJSON, Grid, Plot
from .presets import (
    Whiskers,
    bar,
    box_plot,
    contour,
    contourf,
    density,
    describe,
    ecdf,
    error_bars,
    error_bars_asymmetric,
    heatmap,
    hist,
    hist2d,
    line,
    preset_names,
    quiver,
    scatter,
    sparkline,
    stairs,
    table,
    trend,
    violin,
)
from .raster import Raster, RasterCell
from .series import Column, to_column

__version__ = "0.1.0"

__all__ = [
    "ENGINE_VERSION",
    "Align",
    "Area",
    "Bars",
    "Cells",
    "Charset",
    "Color",
    "ColorMode",
    "Colormap",
    "Column",
    "Dash",
    "DocumentJSON",
    "Frame",
    "Graphics",
    "Grid",
    "Line",
    "LineStyle",
    "Mapping",
    "Mark",
    "Palette",
    "Panel",
    "Plot",
    "PointStyle",
    "Points",
    "Protocol",
    "Range",
    "Raster",
    "RasterCell",
    "Rule",
    "Scale",
    "SupremusError",
    "Text",
    "Theme",
    "Unit",
    "Viewport",
    "Whiskers",
    "Windows",
    "__version__",
    "bar",
    "box_plot",
    "contour",
    "contourf",
    "density",
    "describe",
    "detect_graphics",
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
    "to_column",
    "trend",
    "violin",
]
