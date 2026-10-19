import datetime as dt
import math
from array import array

import pytest

from supremus import Column, Frame, Line, Plot, line, to_column

PLAIN = Frame.plain(40, 10)


def test_none_is_a_gap_and_equals_nan() -> None:
    with_none = line([1, 5, None, 8]).render(PLAIN)
    with_nan = line([1, 5, math.nan, 8]).render(PLAIN)
    assert with_none == with_nan
    assert with_none != line([1, 5, 2, 8]).render(PLAIN)


def test_ints_bools_and_floats_mix() -> None:
    assert line([1, 5, 2, 8]).render(PLAIN) == line([1.0, 5.0, 2.0, 8.0]).render(PLAIN)
    assert line([True, False, 1]).render(PLAIN) == line([1, 0, 1]).render(PLAIN)


def test_tuples_ranges_and_generators_are_series() -> None:
    expected = line([0, 1, 2, 3]).render(PLAIN)
    assert line((0, 1, 2, 3)).render(PLAIN) == expected
    assert line(range(4)).render(PLAIN) == expected
    assert line(i for i in range(4)).render(PLAIN) == expected


def test_array_d_is_a_buffer_and_other_typecodes_cast() -> None:
    expected = line([1, 5, 2, 8]).render(PLAIN)
    assert line(array("d", [1, 5, 2, 8])).render(PLAIN) == expected
    assert line(array("i", [1, 5, 2, 8])).render(PLAIN) == expected
    assert line(array("f", [1, 5, 2, 8])).render(PLAIN) == expected


def test_text_and_nesting_are_refused() -> None:
    with pytest.raises(TypeError, match="not text"):
        line("1 5 2 8")
    with pytest.raises(TypeError, match="two series"):
        line([(1, 2), (3, 4)])
    with pytest.raises(TypeError, match="not text"):
        line(["a", "b"])
    with pytest.raises(TypeError, match="mapping"):
        line({"a": 1})
    with pytest.raises(TypeError, match="numbers and None"):
        line([1, "b"]).render(PLAIN)


def test_datetimes_become_seconds_and_a_time_axis() -> None:
    times = [dt.datetime(2026, 1, 1, tzinfo=dt.timezone.utc), dt.datetime(2026, 1, 2, tzinfo=dt.timezone.utc)]
    column = to_column(times)
    assert column.time
    assert list(column.data) == [1767225600.0, 1767312000.0]
    naive = to_column([dt.datetime(2026, 1, 1), dt.datetime(2026, 1, 2)])
    assert list(naive.data) == list(column.data)
    assert to_column([dt.date(2026, 1, 1), None]).data[1] is None
    plot = Plot().layer(Line.xy(times, [1, 2]))
    assert plot.to_json()["spec"]["x"] == "Time"
    explicit = Plot().x_scale("Linear").layer(Line.xy(times, [1, 2]))
    assert explicit.to_json()["spec"]["x"] == "Linear"


def test_column_passthrough() -> None:
    column = Column([1, 2, 3])
    assert to_column(column) is column
    assert len(column) == 3


numpy = pytest.importorskip("numpy")


def test_numpy_dtypes_cast_once() -> None:
    expected = line([1, 5, 2, 8]).render(PLAIN)
    assert line(numpy.array([1, 5, 2, 8], dtype="int64")).render(PLAIN) == expected
    assert line(numpy.array([1, 5, 2, 8], dtype="float32")).render(PLAIN) == expected
    assert line(numpy.array([1, 5, 2, 8], dtype="float64")[::1]).render(PLAIN) == expected
    strided = numpy.array([1, 0, 5, 0, 2, 0, 8, 0], dtype="float64")[::2]
    assert line(strided).render(PLAIN) == expected
    with pytest.raises(TypeError, match="one-dimensional"):
        line(numpy.zeros((2, 2)))
    with pytest.raises(TypeError, match="not text"):
        line(numpy.array(["a", "b"]))


def test_numpy_datetime64_and_nat() -> None:
    values = numpy.array(["2026-01-01", "NaT", "2026-01-02"], dtype="datetime64[ms]")
    column = to_column(values)
    assert column.time
    assert column.data[0] == 1767225600.0
    assert math.isnan(column.data[1])
    deltas = to_column(numpy.array([1, 2], dtype="timedelta64[s]"))
    assert not deltas.time
    assert list(deltas.data) == [1.0, 2.0]


def test_pandas_columns_and_nullables() -> None:
    pandas = pytest.importorskip("pandas")
    expected = line([1, 5, None, 8]).render(PLAIN)
    assert line(pandas.Series([1, 5, None, 8])).render(PLAIN) == expected
    assert line(pandas.Series([1, 5, pandas.NA, 8], dtype="Int64")).render(PLAIN) == expected
    index = pandas.date_range("2026-01-01", periods=3, freq="D", tz="UTC")
    column = to_column(index)
    assert column.time
    assert column.data[0] == 1767225600.0
    local = pandas.date_range("2026-01-01", periods=2, freq="D", tz="Europe/Berlin")
    assert to_column(local).data[0] == 1767225600.0 - 3600


def test_polars_and_pyarrow_columns() -> None:
    polars = pytest.importorskip("polars")
    pyarrow = pytest.importorskip("pyarrow")
    expected = line([1, 5, None, 8]).render(PLAIN)
    assert line(polars.Series([1.0, 5.0, None, 8.0])).render(PLAIN) == expected
    assert line(polars.Series([1, 5, None, 8])).render(PLAIN) == expected
    assert line(pyarrow.array([1.0, 5.0, None, 8.0])).render(PLAIN) == expected
    assert line(pyarrow.chunked_array([[1.0, 5.0], [None, 8.0]])).render(PLAIN) == expected
    stamps = polars.Series([dt.datetime(2026, 1, 1), dt.datetime(2026, 1, 2)])
    assert to_column(stamps).time
