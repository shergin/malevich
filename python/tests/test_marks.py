import datetime as dt

import pytest

from supremus import Area, Bars, Cells, Color, Colormap, Frame, Line, Plot, Points, Range, Rule, Text

PLAIN = Frame.plain(40, 10)


def layer_of(mark):
    layer, columns = mark._layer(0)
    (name, body), *_ = layer.items()
    return name, body, columns


def test_line_wire_form_and_immutability() -> None:
    base = Line.xy([1, 2, 3], [4, 5, 6])
    styled = base.style("Corners").dash("Dotted").glow().label("l").color("cyan").color_by(["a", "b", "c"])
    name, body, columns = layer_of(styled)
    assert name == "Line"
    assert body["x"] == {"col": 0} and body["y"] == {"col": 1}
    assert body["style"] == "Corners" and body["dash"] == "Dotted" and body["glow"] is True
    assert body["color"] == "Cyan" and body["label"] == "l" and body["color_by"] == ["a", "b", "c"]
    assert len(columns) == 2
    _, plain_body, _ = layer_of(base)
    assert "dash" not in plain_body and "glow" not in plain_body and plain_body["color"] is None
    graded = base.grade([0.1, 0.5, 0.9], Colormap.MAGMA)
    _, body, columns = layer_of(graded)
    assert body["grade"][0] == {"col": 2} and body["grade"][1]["stops"][0] == [0, 0, 4]
    assert len(columns) == 3


def test_channel_errors_name_the_call() -> None:
    with pytest.raises(ValueError, match=r"Line: x and y: channels differ in length \(3 and 2\)"):
        Line.xy([1, 2, 3], [1, 2])
    with pytest.raises(ValueError, match="Line.style"):
        Line.y([1]).style("Wobbly")
    with pytest.raises(ValueError, match="Points.opacity"):
        Points.y([1]).opacity(0)
    with pytest.raises(ValueError, match="Rule.h"):
        Rule.h(float("inf"))
    with pytest.raises(ValueError, match="Text.at"):
        Text.at(float("nan"), 1, "x")
    with pytest.raises(ValueError, match="Bars.spans"):
        Bars.spans(0, 0, [1])
    with pytest.raises(ValueError, match="Cells.matrix"):
        Cells.matrix(3, [1, 2, 3, 4])
    with pytest.raises(ValueError, match="unknown color"):
        Line.y([1]).color("chartreuse")


def test_points_and_area() -> None:
    _, body, _ = layer_of(Points.xy([1], [2]).style("Plus").opacity(0.5).density())
    assert body["style"] == "Plus" and body["opacity"] == 0.5 and body["density"] is True
    _, body, columns = layer_of(Area.between([0, 1], [1, 1], [3, 3]).opacity(0.3).horizontal())
    assert body["x"] == {"col": 0} and body["low"] == {"col": 1} and body["high"] == {"col": 2}
    assert body["horizontal"] is True and body["opacity"] == 0.3
    assert len(columns) == 3


def test_bars_placements() -> None:
    _, body, _ = layer_of(Bars.new(["a", "b"], [1, 2]).base([0.5, 0.5]).horizontal().color_by(["x", "y"]))
    assert body["placement"] == {"Bands": ["a", "b"]}
    assert body["values"] == {"col": 0} and body["base"] == {"col": 1}
    assert body["horizontal"] is True and body["color_by"] == ["x", "y"]
    _, body, _ = layer_of(Bars.spans(0.0, 0.5, [1, 2]))
    assert body["placement"] == {"Spans": {"start": 0.0, "width": 0.5}}
    _, body, columns = layer_of(Bars.at([1, 2], 0.8, [3, 4]))
    assert body["placement"] == {"At": {"x": {"col": 0}, "width": 0.8}} and body["values"] == {"col": 1}
    _, body, columns = layer_of(Bars.intervals([0, 1], [1, 2], [3, 4]))
    assert body["placement"] == {"Intervals": {"starts": {"col": 0}, "ends": {"col": 1}}}
    assert body["values"] == {"col": 2} and len(columns) == 3
    with pytest.raises(ValueError, match="categories and values"):
        Bars.new(["a"], [1, 2])


def test_cells_variants() -> None:
    _, body, columns = layer_of(Cells.matrix(2, [1, 2, 3, 4]).extents((0, 1), (0, 1)).reduce("Max").smooth())
    assert body["columns"] == 2 and body["values"] == {"col": 0}
    assert body["extents"] == [[0.0, 1.0], [0.0, 1.0]] and body["reduce"] == "Max" and body["smooth"] is True
    assert body["colormap"] == Colormap.VIRIDIS.to_json()
    _, body, columns = layer_of(Cells.rgb(1, [(1, 2, 3)]))
    assert body["rgb"] == [[1, 2, 3]] and body["values"] == [] and columns == []
    _, body, _ = layer_of(Cells.classes(2, ["a", "b"]))
    assert body["classes"] == ["a", "b"]
    _, body, _ = layer_of(Cells.matrix(1, [1]).reduce({"Percentile": 95}))
    assert body["reduce"] == {"Percentile": 95.0}


def test_range_and_rule_and_text() -> None:
    _, body, columns = layer_of(Range.over(["a"], [1], [4]).body([2], [3]).marker([2.5]).color_by(["g"]))
    assert body["placement"] == {"Bands": ["a"]}
    assert body["body"] == [{"col": 2}, {"col": 3}] and body["marker"] == {"col": 4} and len(columns) == 5
    _, body, _ = layer_of(Range.y([1], [2]))
    assert body["placement"] == {"Numeric": None}
    _, body, _ = layer_of(Range.xy([0], [1], [2]))
    assert body["placement"] == {"Numeric": {"col": 0}}
    _, body, _ = layer_of(Rule.v_span(1, 2).dash("Dashed"))
    assert body["orientation"] == {"VerticalSpan": [1.0, 2.0]} and body["dash"] == "Dashed"
    _, body, _ = layer_of(Rule.h_span(1, 2))
    assert body["orientation"] == {"HorizontalSpan": [1.0, 2.0]}
    _, body, _ = layer_of(Rule.v(dt.datetime(2026, 1, 1, tzinfo=dt.timezone.utc)))
    assert body["orientation"] == {"Vertical": 1767225600.0}
    _, body, _ = layer_of(Text.at(1, 2, "hi").align("Right").color(Color.rgb(1, 2, 3)))
    assert body["align"] == "Right" and body["color"] == {"Rgb": [1, 2, 3]}


def test_marks_are_values() -> None:
    base = Line.y([1, 2, 3])
    labeled = base.label("x")
    assert layer_of(base)[1]["label"] is None
    assert layer_of(labeled)[1]["label"] == "x"
    plot = Plot().layer(base)
    plot.title("t")
    assert plot.to_json()["spec"]["title"] is None


def test_time_x_marks_choose_the_time_scale() -> None:
    stamps = [dt.datetime(2026, 1, 1), dt.datetime(2026, 1, 2)]
    for mark in (
        Line.xy(stamps, [1, 2]),
        Points.xy(stamps, [1, 2]),
        Area.xy(stamps, [1, 2]),
        Range.xy(stamps, [0, 0], [1, 2]),
        Bars.at(stamps, 3600.0, [1, 2]),
    ):
        assert Plot().layer(mark).to_json()["spec"]["x"] == "Time", type(mark).__name__
    assert Plot().layer(Line.y([1, 2])).to_json()["spec"]["x"] == "Auto"
    rendered = Plot().layer(Line.xy(stamps, [1, 2])).render(PLAIN)
    assert "Jan" in rendered or "01" in rendered
