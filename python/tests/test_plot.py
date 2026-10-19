import json

import pytest

from supremus import Frame, Grid, Line, Palette, Plot, Scale, SupremusError, Unit, Viewport, bar, line

PLAIN = Frame.plain(40, 10)


def test_domains_fold_one_sided_bounds_into_a_pair() -> None:
    spec = line([1, 2]).x_min(0).to_json()["spec"]
    assert spec["x_domain"] == {"min": 0.0}
    spec = line([1, 2]).x_min(0).x_max(10).to_json()["spec"]
    assert spec["x_domain"] == [0.0, 10.0]
    spec = line([1, 2]).y_domain(5, -5).to_json()["spec"]
    assert spec["y_domain"] == [-5.0, 5.0]
    with pytest.raises(ValueError, match="finite"):
        line([1]).x_domain(0, float("inf"))


def test_scales_units_axes_and_palette() -> None:
    spec = (
        line([1, 2])
        .x_scale(Scale.bands(["a", "b"]))
        .y_scale("log")
        .x_unit(Unit.si("B"))
        .y_unit("%")
        .axes(False)
        .colorbar()
        .palette(Palette.OKABE_ITO)
        .title("t")
        .x_label("x")
        .y_label("y")
        .to_json()["spec"]
    )
    assert spec["x"] == {"Bands": ["a", "b"]} and spec["y"] == "Log"
    assert spec["x_unit"] == {"Si": "B"} and spec["y_unit"] == {"Suffix": "%"}
    assert spec["axes"] is False and spec["colorbar"] is True
    assert spec["palette"]["colors"][0] == {"Rgb": [230, 159, 0]}
    assert (spec["title"], spec["x_label"], spec["y_label"]) == ("t", "x", "y")
    assert line([1]).time_x().to_json()["spec"]["x"] == "Time"
    assert line([1]).log_x().to_json()["spec"]["x"] == "Log"
    with pytest.raises(ValueError, match="unknown scale"):
        line([1]).x_scale("polar")


def test_viewport_applies_windows_and_zoom_narrows() -> None:
    plot = line([1, 5, 2, 8, 3, 9, 4, 10])
    mapping = plot.mapping(PLAIN)
    view = mapping.viewport().zoom_x(0.5, 3.5)
    assert isinstance(view, Viewport) and view.x is not None
    low, high = view.x
    assert 0 < low < high < 7
    zoomed = plot.viewport(view)
    assert zoomed.to_json()["spec"]["x_domain"] == [low, high]
    assert zoomed.render(PLAIN) != plot.render(PLAIN)
    assert plot.viewport({}).to_json()["spec"]["x_domain"] is None
    assert plot.viewport(None) is plot


def test_document_round_trip_inlines_gaps() -> None:
    plot = Plot().layer(Line.xy([0, 1, 2], [1, None, 3]).label("l")).title("doc")
    text = plot.to_document()
    document = json.loads(text)
    assert document["spec"]["layers"][0]["Line"]["y"] == [1.0, None, 3.0]
    again = Plot.from_document(document)
    assert again.render(PLAIN) == plot.render(PLAIN)
    with_refs = plot.to_json()
    assert with_refs["spec"]["layers"][0]["Line"]["y"] == {"col": 1}
    assert len(plot.columns()) == 2
    assert Plot.from_document(with_refs, plot.columns()).render(PLAIN) == plot.render(PLAIN)
    with pytest.raises(ValueError, match="plot document"):
        Plot.from_document({"version": 1, "kind": "grid", "spec": {}})


def test_validate_and_engine_errors_carry_codes() -> None:
    line([1, 2]).validate()
    bad = Plot.from_document(
        {"version": 1, "kind": "plot", "spec": {"layers": [{"Cells": {"columns": 2, "values": [1, 2, 3], "extents": None, "colormap": {"stops": [[0, 0, 0], [1, 1, 1]]}}}]}}
    )
    with pytest.raises(SupremusError) as caught:
        bad.validate()
    assert caught.value.code == "NonRectangular"
    assert "values" in str(caught.value)
    with pytest.raises(SupremusError):
        bad.render(PLAIN)


def test_render_requires_a_frame() -> None:
    with pytest.raises(TypeError, match="Frame"):
        line([1]).render((40, 10))  # type: ignore[arg-type]


def test_layers_and_columns_are_copies() -> None:
    plot = line([1, 2, 3])
    plot.layers[0]["Line"]["label"] = "mutated"
    assert plot.to_json()["spec"]["layers"][0]["Line"]["label"] is None


def test_grid_rebases_column_references() -> None:
    grid = Grid(2).with_(line([1, 2]).title("one")).with_(bar(["a"], [3]).title("two"))
    document = grid.to_json()
    assert document["kind"] == "grid" and document["spec"]["columns"] == 2
    assert document["spec"]["plots"][0]["layers"][0]["Line"]["y"] == {"col": 0}
    assert document["spec"]["plots"][1]["layers"][0]["Bars"]["values"] == {"col": 1}
    assert len(grid.columns()) == 2
    rendered = grid.render(Frame.plain(80, 12))
    assert "one" in rendered and "two" in rendered
    grid.validate()
    with pytest.raises(ValueError):
        Grid(0)
