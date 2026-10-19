"""The seam to the native module: one place that imports it, one error type.

Every call into Rust goes through :func:`call`, which turns the native
``EngineError`` into :class:`SupremusError` with the crate's error code. The
JSON the builders hand across is produced by :func:`dumps`, which refuses
``NaN`` so a gap can only ever travel as ``None`` or as a column.
"""

from __future__ import annotations

import json
from collections.abc import Callable
from typing import Any, TypeVar

from . import _native

#: The malevich crate this build renders with.
ENGINE_VERSION = "1.24.1"

T = TypeVar("T")


class SupremusError(Exception):
    """The engine refused a request.

    ``code`` names the crate's error variant (``"UnequalChannels"``,
    ``"NonRectangular"``, ``"EmptyDimension"``, ``"NonFiniteDomain"``,
    ``"IncompatibleScale"``, ``"InvalidParameter"``, ``"DimensionTooLarge"``,
    ``"AllocationFailed"``) or the rim's (``"Format"``, ``"UnknownPreset"``,
    ``"MissingColumn"``). ``message`` is the engine's sentence.
    """

    def __init__(self, code: str, message: str) -> None:
        super().__init__(message)
        self.code = code
        self.message = message

    def __str__(self) -> str:
        return self.message


def call(function: Callable[..., T], *args: Any) -> T:
    """Calls into the native module, translating its error."""
    try:
        return function(*args)
    except _native.EngineError as error:
        if len(error.args) == 2:
            code, message = error.args
        else:  # pragma: no cover - the native module always sends a pair
            code, message = "Engine", str(error)
        raise SupremusError(str(code), str(message)) from None


def dumps(value: Any) -> str:
    """Compact JSON for the wire; ``NaN`` and infinities are refused."""
    return json.dumps(value, separators=(",", ":"), allow_nan=False)


__all__ = ["ENGINE_VERSION", "SupremusError", "call", "dumps"]
