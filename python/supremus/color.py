"""Colors, palettes, colormaps, scales, and units — the wire forms the crate reads.

Named colors serialize as the crate's enum (``"Cyan"``); ``"cyan"`` and
``"bright_cyan"`` are accepted at the rim and canonicalized. A palette is the
categorical color scale for ``color_by``; a colormap is the sequential or
diverging ramp a ``Cells`` grid or a heatmap preset draws with.
"""

from __future__ import annotations

from collections.abc import Iterable
from typing import Any, Literal

__all__ = [
    "Color",
    "ColorValue",
    "Colormap",
    "NamedColor",
    "Palette",
    "Reducer",
    "Scale",
    "Unit",
    "canonicalize_color",
    "color_json",
    "scale_json",
]

NamedColor = Literal[
    "Default",
    "Black",
    "Red",
    "Green",
    "Yellow",
    "Blue",
    "Magenta",
    "Cyan",
    "White",
    "BrightBlack",
    "BrightRed",
    "BrightGreen",
    "BrightYellow",
    "BrightBlue",
    "BrightMagenta",
    "BrightCyan",
    "BrightWhite",
]

#: A color as the API accepts it: a name, ``Color.ansi256(n)``, or ``Color.rgb(r, g, b)``.
ColorValue = str | dict[str, Any]

Reducer = Literal["Count", "Sum", "Mean", "Median", "Min", "Max"] | dict[str, float]

_NAMED: dict[str, str] = {
    "default": "Default",
    "black": "Black",
    "red": "Red",
    "green": "Green",
    "yellow": "Yellow",
    "blue": "Blue",
    "magenta": "Magenta",
    "cyan": "Cyan",
    "white": "White",
    "brightblack": "BrightBlack",
    "brightred": "BrightRed",
    "brightgreen": "BrightGreen",
    "brightyellow": "BrightYellow",
    "brightblue": "BrightBlue",
    "brightmagenta": "BrightMagenta",
    "brightcyan": "BrightCyan",
    "brightwhite": "BrightWhite",
}

#: The sixteen named colors in packed-raster index order (``Black`` is 0).
NAMED_BY_INDEX: tuple[str, ...] = (
    "Black",
    "Red",
    "Green",
    "Yellow",
    "Blue",
    "Magenta",
    "Cyan",
    "White",
    "BrightBlack",
    "BrightRed",
    "BrightGreen",
    "BrightYellow",
    "BrightBlue",
    "BrightMagenta",
    "BrightCyan",
    "BrightWhite",
)


class Color:
    """Color constructors. Names are also accepted as plain strings."""

    Default = "Default"
    Black = "Black"
    Red = "Red"
    Green = "Green"
    Yellow = "Yellow"
    Blue = "Blue"
    Magenta = "Magenta"
    Cyan = "Cyan"
    White = "White"
    BrightBlack = "BrightBlack"
    BrightRed = "BrightRed"
    BrightGreen = "BrightGreen"
    BrightYellow = "BrightYellow"
    BrightBlue = "BrightBlue"
    BrightMagenta = "BrightMagenta"
    BrightCyan = "BrightCyan"
    BrightWhite = "BrightWhite"

    @staticmethod
    def ansi256(index: int) -> dict[str, Any]:
        """One of the 256 indexed colors."""
        if not 0 <= int(index) <= 255:
            raise ValueError("an ansi256 index is 0..=255")
        return {"Ansi256": int(index)}

    @staticmethod
    def rgb(r: int, g: int, b: int) -> dict[str, Any]:
        """A truecolor triple; quantized honestly on lesser terminals."""
        triple = (int(r), int(g), int(b))
        if any(not 0 <= v <= 255 for v in triple):
            raise ValueError("rgb components are 0..=255")
        return {"Rgb": list(triple)}


def canonicalize_color(color: ColorValue) -> ColorValue:
    """``"cyan"`` → ``"Cyan"``; ``"bright_cyan"`` → ``"BrightCyan"``; dicts pass through."""
    if isinstance(color, dict):
        if "Ansi256" in color or "Rgb" in color:
            return color
        if "ansi256" in color:
            return Color.ansi256(color["ansi256"])
        if "rgb" in color:
            return Color.rgb(*color["rgb"])
        raise TypeError(f"not a color: {color!r}")
    if isinstance(color, str):
        named = _NAMED.get(color.replace("_", "").replace("-", "").lower())
        if named is None:
            raise ValueError(f"unknown color {color!r}")
        return named
    raise TypeError(f"not a color: {type(color).__name__}")


def color_json(color: ColorValue | None) -> Any:
    """The wire form, or ``None``."""
    if color is None:
        return None
    return canonicalize_color(color)


class Palette:
    """A categorical palette: the colors ``color_by`` groups take, in order."""

    OKABE_ITO: Palette
    BRIGHT: Palette
    MUTED: Palette

    __slots__ = ("colors",)

    def __init__(self, colors: Iterable[ColorValue]) -> None:
        self.colors = tuple(canonicalize_color(c) for c in colors)
        if not self.colors:
            raise ValueError("a palette has at least one color")

    @staticmethod
    def of(colors: Iterable[ColorValue]) -> Palette:
        return Palette(colors)

    def to_json(self) -> dict[str, Any]:
        return {"colors": list(self.colors)}

    def __eq__(self, other: object) -> bool:
        return isinstance(other, Palette) and other.colors == self.colors

    def __hash__(self) -> int:
        return hash(("Palette", repr(self.colors)))

    def __repr__(self) -> str:
        return f"Palette({list(self.colors)!r})"


Palette.OKABE_ITO = Palette(
    [
        Color.rgb(230, 159, 0),
        Color.rgb(86, 180, 233),
        Color.rgb(0, 158, 115),
        Color.rgb(213, 94, 0),
        Color.rgb(204, 121, 167),
        Color.rgb(0, 114, 178),
        Color.rgb(240, 228, 66),
    ]
)
Palette.BRIGHT = Palette(
    [
        Color.rgb(68, 119, 170),
        Color.rgb(238, 102, 119),
        Color.rgb(34, 136, 51),
        Color.rgb(204, 187, 68),
        Color.rgb(102, 204, 238),
        Color.rgb(170, 51, 119),
        Color.rgb(187, 187, 187),
    ]
)
Palette.MUTED = Palette(
    [
        Color.rgb(51, 34, 136),
        Color.rgb(136, 204, 238),
        Color.rgb(68, 170, 153),
        Color.rgb(17, 119, 51),
        Color.rgb(153, 153, 51),
        Color.rgb(221, 204, 119),
        Color.rgb(204, 102, 119),
        Color.rgb(136, 34, 85),
        Color.rgb(170, 68, 153),
    ]
)


class Colormap:
    """A sequential or diverging ramp over ``stops``, with the crate's options.

    Every method returns a new colormap. ``centered_at`` anchors a diverging
    map to a data value; ``log`` makes a sequential map logarithmic;
    ``domain`` fixes the value range, with ``under``/``over`` disclosing what
    falls outside; ``steps`` and ``thresholds`` quantize the ramp.
    """

    VIRIDIS: Colormap
    MAGMA: Colormap
    CIVIDIS: Colormap
    GREYS: Colormap
    RED_BLUE: Colormap
    PURPLE_ORANGE: Colormap

    __slots__ = ("_spec",)

    def __init__(self, stops: Iterable[tuple[int, int, int]], **options: Any) -> None:
        listed = [tuple(int(v) for v in stop) for stop in stops]
        if len(listed) < 2:
            raise ValueError("a colormap has at least two stops")
        if any(len(stop) != 3 or not all(0 <= v <= 255 for v in stop) for stop in listed):
            raise ValueError("a colormap stop is an (r, g, b) triple of 0..=255")
        self._spec: dict[str, Any] = {"stops": [list(stop) for stop in listed], **options}

    @staticmethod
    def of(stops: Iterable[tuple[int, int, int]]) -> Colormap:
        return Colormap(stops)

    @staticmethod
    def named(name: str) -> Colormap:
        key = name.strip().lower().replace("_", "-")
        maps = {
            "viridis": Colormap.VIRIDIS,
            "magma": Colormap.MAGMA,
            "cividis": Colormap.CIVIDIS,
            "greys": Colormap.GREYS,
            "grays": Colormap.GREYS,
            "red-blue": Colormap.RED_BLUE,
            "purple-orange": Colormap.PURPLE_ORANGE,
        }
        try:
            return maps[key]
        except KeyError:
            raise ValueError(f"unknown colormap {name!r}") from None

    def _with(self, **options: Any) -> Colormap:
        spec = {**self._spec, **options}
        stops = spec.pop("stops")
        return Colormap(stops, **spec)

    def centered_at(self, midpoint: float) -> Colormap:
        _finite(midpoint, "Colormap.centered_at")
        return self._with(midpoint=float(midpoint))

    def log(self) -> Colormap:
        return self._with(log=True)

    def domain(self, low: float, high: float) -> Colormap:
        _finite(low, "Colormap.domain")
        _finite(high, "Colormap.domain")
        if not low < high:
            raise ValueError("Colormap.domain requires low < high")
        return self._with(domain=[float(low), float(high)])

    def under(self, color: ColorValue) -> Colormap:
        return self._with(under=canonicalize_color(color))

    def over(self, color: ColorValue) -> Colormap:
        return self._with(over=canonicalize_color(color))

    def steps(self, count: int) -> Colormap:
        if int(count) < 2:
            raise ValueError("Colormap.steps requires at least two bands")
        return self._with(steps=int(count))

    def thresholds(self, values: Iterable[float]) -> Colormap:
        listed = [float(v) for v in values]
        for value in listed:
            _finite(value, "Colormap.thresholds")
        return self._with(thresholds=listed)

    def to_json(self) -> dict[str, Any]:
        return {k: (list(v) if isinstance(v, list) else v) for k, v in self._spec.items()}

    def __eq__(self, other: object) -> bool:
        return isinstance(other, Colormap) and other._spec == self._spec

    def __hash__(self) -> int:
        return hash(("Colormap", repr(sorted(self._spec.items()))))

    def __repr__(self) -> str:
        return f"Colormap({self._spec!r})"


Colormap.VIRIDIS = Colormap([(68, 1, 84), (59, 82, 139), (33, 145, 140), (94, 201, 98), (253, 231, 37)])
Colormap.MAGMA = Colormap([(0, 0, 4), (81, 18, 124), (183, 55, 121), (252, 137, 97), (252, 253, 191)])
Colormap.CIVIDIS = Colormap(
    [(0, 32, 77), (65, 77, 107), (124, 123, 120), (188, 175, 111), (255, 233, 69)]
)
Colormap.GREYS = Colormap([(64, 64, 64), (250, 250, 250)])
Colormap.RED_BLUE = Colormap(
    [(202, 0, 32), (244, 165, 130), (247, 247, 247), (146, 197, 222), (5, 113, 176)]
)
Colormap.PURPLE_ORANGE = Colormap(
    [(94, 60, 153), (178, 171, 210), (247, 247, 247), (253, 184, 99), (230, 97, 1)]
)


class Scale:
    """Axis scales: ``Scale.AUTO``, ``LINEAR``, ``INTEGER``, ``LOG``, ``TIME``, or ``Scale.bands([...])``."""

    AUTO = "Auto"
    LINEAR = "Linear"
    INTEGER = "Integer"
    LOG = "Log"
    TIME = "Time"

    @staticmethod
    def bands(categories: Iterable[str]) -> dict[str, Any]:
        """A categorical axis with these bands, in order."""
        return {"Bands": [str(c) for c in categories]}


def scale_json(scale: Any) -> Any:
    """The crate's wire form of a scale spelled any accepted way."""
    if isinstance(scale, str):
        canonical = {s.lower(): s for s in ("Auto", "Linear", "Integer", "Log", "Time")}
        try:
            return canonical[scale.strip().lower()]
        except KeyError:
            raise ValueError(f"unknown scale {scale!r}") from None
    if isinstance(scale, dict):
        if "Bands" in scale:
            return {"Bands": [str(c) for c in scale["Bands"]]}
        if "bands" in scale:
            return {"Bands": [str(c) for c in scale["bands"]]}
    if isinstance(scale, (list, tuple)):
        return {"Bands": [str(c) for c in scale]}
    raise TypeError(f"not a scale: {scale!r}")


class Unit:
    """What an axis's labels are counted in: ``Unit.si("B")``, ``Unit.BYTES``, ``Unit.suffix("%")``."""

    PLAIN = "Plain"
    BYTES = "Bytes"

    @staticmethod
    def si(unit: str) -> dict[str, Any]:
        """The axis's one SI prefix before this unit: ``2.5 MB``, ``100 µs``."""
        return {"Si": str(unit)}

    @staticmethod
    def suffix(suffix: str) -> dict[str, Any]:
        """A bare suffix, never a prefix: ``45%``."""
        return {"Suffix": str(suffix)}


def _finite(value: float, who: str) -> None:
    import math

    if not math.isfinite(float(value)):
        raise ValueError(f"{who} requires a finite value")
