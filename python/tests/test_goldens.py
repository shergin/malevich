"""The shared goldens: one oracle (the crate), one fixture, read by JS and Python."""

import json

from supremus import (
    Area,
    Bars,
    Cells,
    Colormap,
    Frame,
    Line,
    Palette,
    Plot,
    Range,
    Rule,
    Text,
    _native,
    bar,
    heatmap,
    hist,
    line,
)
from supremus._engine import dumps


def frame_of(golden: dict) -> Frame:
    f = golden["frame"]
    return Frame(f["width"], f["height"], f["charset"], f["color"])


def test_every_golden_document_renders_byte_identically(goldens) -> None:
    assert len(goldens) >= 10
    for golden in goldens:
        got = _native.render_columns(dumps(golden["document"]), dumps(golden["frame"]), [])
        assert got == golden["expected"], golden["name"]
        again = Plot.from_document(golden["document"]).render(frame_of(golden))
        assert again == golden["expected"], f"{golden['name']} via Plot.from_document"


def test_builders_reproduce_the_goldens(golden) -> None:
    plain = Frame.plain(40, 10)
    portable = Frame.portable(40, 10)
    built = {
        "line": (line([1, 5, 2, 8]), plain),
        "bar": (bar(["mon", "tue", "wed"], [3, 7, 4.5]), plain),
        "hist": (hist([1.0, 2.0, 2.5, 2.7, 3.0, 3.1, 3.2, 4.0, 5.5]), plain),
        "annotated": (
            Plot()
            .layer(Line.y([1, 5, 2, 8]))
            .layer(Rule.h(4.0).label("mid"))
            .layer(Text.at(2.0, 6.0, "hi"))
            .title("annotated"),
            plain,
        ),
        "area": (Plot().layer(Area.y([1, 3, 2, 4]).label("fill")), plain),
        "stacked-bars": (
            Plot()
            .layer(Bars.new(["a", "b", "c"], [1, 2, 1.5]).label("low"))
            .layer(Bars.new(["a", "b", "c"], [0.5, 1, 0.5]).base([1, 2, 1.5]).label("high"))
            .title("stack"),
            plain,
        ),
        "cells": (
            Plot().layer(Cells.matrix(4, list(range(16))).colormap(Colormap.VIRIDIS)).title("grid"),
            portable,
        ),
        "range": (
            Plot()
            .layer(
                Range.over(["a", "b"], [1, 2], [4, 5]).body([2, 3], [3, 4]).marker([2.5, 3.5]).label("box")
            )
            .title("intervals"),
            plain,
        ),
        "heatmap": (heatmap(4, list(range(16))), portable),
        "color-by": (
            Plot()
            .layer(Line.y([1, 2, 3, 4]).color_by(["a", "a", "b", "b"]).style("Corners"))
            .palette(Palette.OKABE_ITO)
            .title("groups"),
            plain,
        ),
    }
    for name, (plot, frame) in built.items():
        assert plot.render(frame) == golden(name)["expected"], name


def test_the_wire_matches_the_crate_document(golden) -> None:
    """The builder's inlined document is the crate's, field for field."""
    plot = (
        Plot()
        .layer(Line.y([1, 2, 3, 4]).color_by(["a", "a", "b", "b"]).style("Corners"))
        .palette(Palette.OKABE_ITO)
        .title("groups")
    )
    assert json.loads(plot.to_document()) == golden("color-by")["document"]
