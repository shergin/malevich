import pytest

pd = pytest.importorskip("pandas")

from supremus import Frame, Plot  # noqa: E402

PLAIN = Frame.plain(60, 12)


@pytest.fixture(autouse=True)
def backend():
    pd.set_option("plotting.backend", "supremus")
    yield
    pd.set_option("plotting.backend", "matplotlib")


def test_dataframe_plot_is_a_line_per_column() -> None:
    frame = pd.DataFrame({"a": [1, 2, 3], "b": [3, 2, 1]})
    plot = frame.plot()
    assert isinstance(plot, Plot)
    layers = plot.to_json()["spec"]["layers"]
    assert [list(layer)[0] for layer in layers] == ["Line", "Line"]
    assert [layer["Line"]["label"] for layer in layers] == ["a", "b"]
    assert "a" in plot.render(PLAIN)


def test_datetime_index_becomes_the_time_axis() -> None:
    frame = pd.DataFrame({"v": [1, 2, 3]}, index=pd.date_range("2026-01-01", periods=3, freq="D"))
    plot = frame.plot(title="t", ylabel="v")
    spec = plot.to_json()["spec"]
    assert spec["x"] == "Time" and spec["title"] == "t" and spec["y_label"] == "v"


def test_kinds_map_onto_presets() -> None:
    frame = pd.DataFrame({"x": [1, 2, 3, 4], "y": [2, 4, 1, 3]})
    assert "Bars" in frame["y"].plot(kind="bar").to_json()["spec"]["layers"][0]
    assert frame["y"].plot(kind="barh").to_json()["spec"]["layers"][0]["Bars"]["horizontal"] is True
    assert frame["y"].plot(kind="hist", bins=4).to_json()["spec"]["y"] == "Integer"
    assert frame["y"].plot.kde().render(PLAIN)
    assert frame.plot(kind="box").render(PLAIN)
    assert "Points" in frame.plot(kind="scatter", x="x", y="y").to_json()["spec"]["layers"][0]
    assert "Cells" in frame.plot(kind="hexbin", x="x", y="y", gridsize=6).to_json()["spec"]["layers"][0]
    assert "Area" in frame.plot(kind="area").to_json()["spec"]["layers"][0]
    stacked = frame.plot(kind="bar", stacked=True).to_json()["spec"]["layers"]
    assert "base" in stacked[1]["Bars"]
    assert frame["y"].hist().render(PLAIN)
    assert frame.boxplot().render(PLAIN)


def test_refusals_are_explicit() -> None:
    frame = pd.DataFrame({"x": [1, 2], "y": [2, 1]})
    with pytest.raises(TypeError, match="does not take"):
        frame.plot(color="red")
    with pytest.raises(NotImplementedError):
        frame["x"].plot(kind="pie")
    with pytest.raises(TypeError, match="one column"):
        frame.plot(kind="bar")
    with pytest.raises(TypeError, match="scatter requires"):
        frame.plot(kind="scatter")
