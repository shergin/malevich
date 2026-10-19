import datetime as dt
import json
import math

import pytest

import supremus as sp
from supremus import Frame, SupremusError, Whiskers
from supremus._engine import dumps

PLAIN = Frame.plain(40, 10)
PORTABLE = Frame.portable(40, 12)
SAMPLES = [math.sin(i * 0.37) * 2 + (i % 7) * 0.2 for i in range(200)]
XS = [float(i) for i in range(50)]
YS = [0.5 * x + math.sin(x) for x in XS]
GRID = [((i % 6) - 2.5) ** 2 + (i // 6) for i in range(36)]


@pytest.mark.parametrize(
    "plot",
    [
        sp.line(SAMPLES),
        sp.scatter(XS, YS),
        sp.bar(["a", "b", "c"], [1, 2, 3]),
        sp.stairs([1, 3, 2, 4], direction="Mid"),
        sp.hist(SAMPLES, max_bins=12, normalization="Percent", cumulative=True),
        sp.ecdf(SAMPLES, band=0.05),
        sp.density(SAMPLES, samples=64, bandwidth_scale=1.5),
        sp.trend(XS, YS, band=2.0),
        sp.hist2d(XS, YS, columns=10, rows=8, colorbar=False),
        sp.heatmap(6, GRID, colormap=sp.Colormap.MAGMA),
        sp.contour(6, GRID, levels=4),
        sp.contour(6, GRID, levels=[2.0, 5.0]),
        sp.contourf(6, GRID),
        sp.box_plot(["a", "b"], [SAMPLES, YS], whiskers=Whiskers.MINMAX),
        sp.box_plot(["a", "b"], [SAMPLES, YS], whiskers=Whiskers.percentiles(0.05, 0.95)),
        sp.violin(["a", "b"], [SAMPLES, YS], samples=32),
        sp.describe(["a", "b"], [SAMPLES, YS], histogram=8),
        sp.table(["r1", "r2"], ["c1", "c2"], [1, 2, 3, 4], colormap=sp.Colormap.GREYS),
        sp.error_bars([1, 2, 3], [4, 6, 5], [0.5, 1, 0.4]),
        sp.error_bars_asymmetric([1, 2, 3], [4, 6, 5], [0.5, 1, 0.4], [1.5, 0.3, 0.9]),
        sp.quiver([0, 1, 2], [0, 1, 0], [0.4, 0.3, -0.2], [0.2, -0.3, 0.4]),
    ],
    ids=lambda p: p.to_json()["spec"]["layers"][0] and list(p.to_json()["spec"]["layers"][0])[0],
)
def test_every_preset_renders(plot) -> None:
    rendered = plot.render(PORTABLE)
    assert rendered.strip()
    assert "┤" in rendered or "│" in rendered or "─" in rendered


def test_sparkline_is_its_engine_expansion() -> None:
    values = [1, 5, 2, 8, None, 3]
    composed = sp.sparkline(values)
    expanded = sp.Plot.from_document(
        json.loads(sp._native.expand_preset("sparkline", "{}", [values]))
    )
    frame = Frame.portable(12, 1)
    assert composed.render(frame) == expanded.render(frame)
    assert composed.to_json()["spec"]["axes"] is False


def test_line_scatter_bar_are_compositions() -> None:
    assert sp.line([1, 2]).to_json()["spec"]["layers"][0]["Line"]["x"] is None
    assert "Points" in sp.scatter([1], [2]).to_json()["spec"]["layers"][0]
    assert sp.bar(["a"], [1]).to_json()["spec"]["layers"][0]["Bars"]["placement"] == {"Bands": ["a"]}


def test_hist_options_change_the_chart() -> None:
    plain = sp.hist(SAMPLES).render(PLAIN)
    assert sp.hist(SAMPLES, max_bins=5).render(PLAIN) != plain
    assert sp.hist(SAMPLES, cumulative=True).render(PLAIN) != plain
    assert sp.hist(SAMPLES).to_json()["spec"]["y"] == "Integer"


def test_preset_errors_are_errors_not_crashes() -> None:
    with pytest.raises(SupremusError) as caught:
        sp.box_plot(["a"], [[1, 2], [3, 4]])
    assert caught.value.code == "Format"
    with pytest.raises(SupremusError) as caught:
        sp.contour(5, [1, 2, 3])
    assert caught.value.code == "NonRectangular"
    with pytest.raises(SupremusError) as caught:
        sp.hist(SAMPLES, normalization="Bogus")
    assert caught.value.code == "Format"
    with pytest.raises(SupremusError) as caught:
        sp.quiver([1], [1], [1], [1, 2])
    assert caught.value.code == "Format"


def test_preset_names_cover_the_engine() -> None:
    names = sp.preset_names()
    for name in ("hist", "box_plot", "violin", "contourf", "quiver", "sparkline"):
        assert name in names


def test_time_x_survives_the_engine_presets() -> None:
    stamps = [dt.datetime(2026, 1, 1) + dt.timedelta(days=i) for i in range(10)]
    plot = sp.trend(stamps, list(range(10)))
    assert plot.to_json()["spec"]["x"] == "Time"


def test_dumps_refuses_nan() -> None:
    with pytest.raises(ValueError):
        dumps({"x": float("nan")})
