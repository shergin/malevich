"""The rim: where anything series-shaped becomes one column for the engine.

Conversion happens here, once. A Python ``list`` or ``tuple`` travels as it
is and the native module extracts it in one pass (ints, floats, bools, and
``None`` for a gap). Anything exposing a contiguous ``float64`` buffer
(``array('d')``, a numpy array, whatever ``to_numpy()`` returns on a pandas,
polars, or pyarrow column) is copied once in Rust. Other dtypes are cast to
``float64`` here. Datetimes — ``datetime`` objects, ``numpy.datetime64``, a
pandas ``DatetimeIndex``, a polars ``Datetime`` column, an Arrow timestamp —
become unix seconds (UTC), the unit of the engine's ``Time`` scale, and the
column remembers it was time so :class:`~supremus.Plot` can pick that scale.

Nested ``[(x, y), ...]`` pairs and data frames are not series. A mark that
wants two channels takes two series.
"""

from __future__ import annotations

import datetime as _dt
import math
from array import array
from collections.abc import Iterable
from typing import Any

__all__ = ["Column", "to_column", "to_seconds"]

_UTC = _dt.timezone.utc


class Column:
    """One ingested series: what the native module accepts, and whether it is time."""

    __slots__ = ("data", "time")

    def __init__(self, data: Any, *, time: bool = False) -> None:
        self.data = data
        self.time = time

    def __len__(self) -> int:
        return len(self.data)

    def __repr__(self) -> str:
        kind = "time" if self.time else "values"
        return f"Column({len(self)} {kind})"


def to_column(values: Any, *, what: str = "a series") -> Column:
    """Ingests ``values`` as one column. Raises ``TypeError`` for non-series input."""
    if isinstance(values, Column):
        return values
    if isinstance(values, (str, bytes, bytearray)):
        raise TypeError(f"{what} is numbers, not text")
    if isinstance(values, dict):
        raise TypeError(f"{what} is a sequence of numbers, not a mapping")
    if isinstance(values, array):
        if values.typecode == "d":
            return Column(values)
        return Column(array("d", values))
    if isinstance(values, (list, tuple)):
        return _from_sequence(values, what)
    if hasattr(values, "to_numpy"):
        return _from_numpy(_numpy_of(values), what)
    if hasattr(values, "__array__") or hasattr(values, "dtype"):
        import numpy as np

        return _from_numpy(np.asarray(values), what)
    view = _buffer(values)
    if view is not None:
        if view.ndim != 1:
            raise TypeError(f"{what} is one-dimensional (got {view.ndim} dimensions)")
        if view.format == "d":
            return Column(values)
        return Column(array("d", view.tolist()))
    if isinstance(values, Iterable):
        return _from_sequence(list(values), what)
    raise TypeError(f"cannot read {what} from {type(values).__name__}")


def to_seconds(value: Any) -> float:
    """One ``datetime`` or ``date`` as unix seconds; naive values are UTC."""
    if isinstance(value, _dt.datetime):
        if value.tzinfo is None:
            value = value.replace(tzinfo=_UTC)
        return value.timestamp()
    if isinstance(value, _dt.date):
        return _dt.datetime(value.year, value.month, value.day, tzinfo=_UTC).timestamp()
    raise TypeError(f"not a datetime: {type(value).__name__}")


def _buffer(values: Any) -> memoryview | None:
    try:
        return memoryview(values)
    except TypeError:
        return None


def _is_missing(value: Any) -> bool:
    if value is None:
        return True
    # pandas' NA and NaT are their own singletons; they are gaps too.
    return type(value).__name__ in {"NAType", "NaTType"}


def _from_sequence(values: list[Any] | tuple[Any, ...], what: str) -> Column:
    for value in values:
        if _is_missing(value):
            continue
        if isinstance(value, (_dt.datetime, _dt.date)):
            return Column(
                [None if _is_missing(v) else to_seconds(v) for v in values], time=True
            )
        if isinstance(value, (list, tuple)):
            raise TypeError(
                f"{what} is flat; a mark that wants two channels takes two series"
            )
        if isinstance(value, (str, bytes)):
            raise TypeError(f"{what} is numbers, not text")
        break
    if any(_is_missing(v) and v is not None for v in values):
        values = [None if _is_missing(v) else v for v in values]
    return Column(values)


def _numpy_of(values: Any) -> Any:
    """``to_numpy()`` of a pandas, polars, or pyarrow column, nulls included."""
    dtype = getattr(values, "dtype", None)
    if getattr(dtype, "tz", None) is not None:
        # A tz-aware pandas column: the instant in UTC, not an object array.
        return values.to_numpy(dtype="datetime64[ns]")
    try:
        return values.to_numpy()
    except Exception:  # noqa: BLE001 - pyarrow refuses zero-copy with nulls
        return values.to_numpy(zero_copy_only=False)


def _from_numpy(values: Any, what: str) -> Column:
    import numpy as np

    if values.ndim != 1:
        raise TypeError(f"{what} is one-dimensional (got shape {values.shape})")
    kind = values.dtype.kind
    if kind == "M":
        nanoseconds = values.astype("datetime64[ns]")
        seconds = nanoseconds.astype("int64").astype(np.float64) / 1e9
        seconds[np.isnat(nanoseconds)] = math.nan
        return Column(np.ascontiguousarray(seconds), time=True)
    if kind == "m":
        nanoseconds = values.astype("timedelta64[ns]")
        seconds = nanoseconds.astype("int64").astype(np.float64) / 1e9
        seconds[np.isnat(nanoseconds)] = math.nan
        return Column(np.ascontiguousarray(seconds))
    if kind == "O":
        return _from_sequence(values.tolist(), what)
    if kind in "US":
        raise TypeError(f"{what} is numbers, not text")
    if values.dtype != np.float64:
        values = values.astype(np.float64)
    return Column(np.ascontiguousarray(values))
