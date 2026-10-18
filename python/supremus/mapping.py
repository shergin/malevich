"""The geometry of one render, and the axis windows a zoom is made of.

``Plot.mapping(frame)`` resolves a plot into a :class:`Mapping`: cell to
data and back, labels as the axes format them, the plot rectangle. A
:class:`Viewport` is a pair of optional axis windows with the zoom and pan
arithmetic; a zoom is a scale option, so the engine re-aggregates to the
visible window on the next render. These are the physics. Gesture policy
lives in :mod:`supremus.textual`.
"""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from . import _native

__all__ = ["Mapping", "Panel", "Viewport", "Windows"]

#: An axis window pair as ``Plot.viewport`` accepts it; a missing axis is automatic.
Windows = dict[str, tuple[float, float]]


@dataclass(frozen=True)
class Panel:
    """A rectangle in frame cells."""

    column: int
    row: int
    width: int
    height: int

    def contains(self, column: int, row: int) -> bool:
        return (
            self.column <= column < self.column + self.width
            and self.row <= row < self.row + self.height
        )


class Viewport:
    """An axis window pair. Every transform returns a new viewport; log axes
    stay in decade space because the arithmetic runs in the engine."""

    __slots__ = ("_inner",)

    def __init__(self, inner: Any) -> None:
        self._inner = inner

    @staticmethod
    def auto() -> Viewport:
        """Both axes automatic. Seed from a mapping before transforming."""
        return Viewport(_native.Viewport.auto())

    @property
    def x(self) -> tuple[float, float] | None:
        return self._inner.x

    @property
    def y(self) -> tuple[float, float] | None:
        return self._inner.y

    @property
    def is_auto(self) -> bool:
        return bool(self._inner.is_auto)

    def windows(self) -> Windows:
        """The window pair ``Plot.viewport`` accepts."""
        out: Windows = {}
        if self.x is not None:
            out["x"] = self.x
        if self.y is not None:
            out["y"] = self.y
        return out

    def zoom_x(self, factor: float, anchor: float) -> Viewport:
        return Viewport(self._inner.zoom_x(factor, anchor))

    def zoom_y(self, factor: float, anchor: float) -> Viewport:
        return Viewport(self._inner.zoom_y(factor, anchor))

    def pan_x(self, fraction: float) -> Viewport:
        return Viewport(self._inner.pan_x(fraction))

    def pan_y(self, fraction: float) -> Viewport:
        return Viewport(self._inner.pan_y(fraction))

    def with_x(self, low: float, high: float) -> Viewport:
        return Viewport(self._inner.with_x(low, high))

    def with_y(self, low: float, high: float) -> Viewport:
        return Viewport(self._inner.with_y(low, high))

    def reset(self) -> Viewport:
        return Viewport(self._inner.reset())

    def reset_x(self) -> Viewport:
        return Viewport(self._inner.reset_x())

    def reset_y(self) -> Viewport:
        return Viewport(self._inner.reset_y())

    def clamp_x(self, low: float, high: float) -> Viewport:
        return Viewport(self._inner.clamp_x(low, high))

    def clamp_y(self, low: float, high: float) -> Viewport:
        return Viewport(self._inner.clamp_y(low, high))

    def tail(self, latest: float, width: float) -> Viewport:
        """The window ending at ``latest`` and ``width`` wide: a live chart's x."""
        return Viewport(self._inner.tail(latest, width))

    def __repr__(self) -> str:
        return f"Viewport(x={self.x}, y={self.y})"


class Mapping:
    """Resolved geometry of one render: hit-testing, axis-formatted labels,
    the plot rectangle, a viewport seed."""

    __slots__ = ("_inner",)

    def __init__(self, inner: Any) -> None:
        self._inner = inner

    def data_at(self, column: int, row: int) -> tuple[float, float] | None:
        """Data coordinates at the center of frame cell ``(column, row)``."""
        if column < 0 or row < 0:
            return None
        return self._inner.data_at(column, row)

    def cell_at(self, x: float, y: float) -> tuple[int, int] | None:
        """The frame cell where ``(x, y)`` draws."""
        return self._inner.cell_at(x, y)

    def column_at(self, x: float) -> int | None:
        """The frame column where the data value ``x`` draws."""
        return self._inner.column_at(x)

    def format_x(self, value: float) -> str:
        return self._inner.format_x(value)

    def format_y(self, value: float) -> str:
        return self._inner.format_y(value)

    @property
    def x_domain(self) -> tuple[float, float]:
        return self._inner.x_domain

    @property
    def y_domain(self) -> tuple[float, float]:
        return self._inner.y_domain

    @property
    def plot_area(self) -> Panel | None:
        area = self._inner.plot_area
        return None if area is None else Panel(*area)

    @property
    def x_categories(self) -> list[str] | None:
        """Band labels on a categorical x axis; ``None`` on a continuous one."""
        return self._inner.x_categories

    @property
    def y_categories(self) -> list[str] | None:
        return self._inner.y_categories

    def viewport(self) -> Viewport:
        """A viewport seeded from the resolved domains."""
        return Viewport(self._inner.viewport())

    def __repr__(self) -> str:
        return f"Mapping(x={self.x_domain}, y={self.y_domain}, area={self.plot_area})"
