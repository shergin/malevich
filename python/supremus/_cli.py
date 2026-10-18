"""``supremus`` on the command line: a tour, or a stdin plotter.

    supremus              a colored tour sized to this terminal
    supremus line         plot stdin numbers as a line
    supremus hist         plot stdin numbers as a histogram
    supremus bar          plot stdin as category<TAB>value bars

This is the ``npx malevich`` of the Python package, not a ``kaz`` rewrite:
the Rust binary stays the full CLI.
"""

from __future__ import annotations

import math
import sys

from .frame import Frame
from .mark import Line, Rule, Text
from .plot import Plot
from .presets import bar, box_plot, hist, line

__all__ = ["main"]

USAGE = """supremus — terminal plotting

Usage:
  supremus              colored tour sized to this terminal
  supremus line         plot stdin numbers as a line
  supremus hist         plot stdin numbers as a histogram
  supremus bar          plot stdin as category<TAB>value bars
"""


def main(argv: list[str] | None = None) -> int:
    args = sys.argv[1:] if argv is None else list(argv)
    command = args[0] if args else "showcase"
    if command in {"-h", "--help", "help"}:
        sys.stdout.write(USAGE)
        return 0
    if command == "showcase":
        showcase()
        return 0
    frame = Frame.detect()
    text = sys.stdin.read()
    if command == "line":
        numbers = parse_numbers(text)
        if not numbers:
            return fail("supremus line: expected numbers on stdin")
        sys.stdout.write(line(numbers).render(frame) + "\n")
        return 0
    if command == "hist":
        numbers = parse_numbers(text)
        if not numbers:
            return fail("supremus hist: expected numbers on stdin")
        sys.stdout.write(hist(numbers).render(frame) + "\n")
        return 0
    if command == "bar":
        categories, values = parse_bars(text)
        if not categories:
            return fail("supremus bar: expected lines of `category<TAB>value`")
        sys.stdout.write(bar(categories, values).render(frame) + "\n")
        return 0
    return fail(f"unknown command '{command}' (try --help)")


def fail(message: str) -> int:
    sys.stderr.write(message + "\n")
    return 1


def parse_numbers(text: str) -> list[float | None]:
    numbers: list[float | None] = []
    for token in text.replace(",", " ").split():
        if token.lower() in {"nan", "null", "none", "-"}:
            numbers.append(None)
            continue
        try:
            numbers.append(float(token))
        except ValueError:
            continue
    return numbers


def parse_bars(text: str) -> tuple[list[str], list[float]]:
    categories: list[str] = []
    values: list[float] = []
    for raw in text.splitlines():
        if not raw.strip():
            continue
        parts = raw.split("\t") if "\t" in raw else raw.rsplit(None, 1)
        if len(parts) != 2:
            continue
        try:
            value = float(parts[1])
        except ValueError:
            continue
        categories.append(parts[0].strip())
        values.append(value)
    return categories, values


def showcase() -> None:
    frame = Frame.detect()
    steps = list(range(80))
    train = [3.8 * math.exp(-0.035 * s) + 0.32 + 0.05 * math.sin(s * 0.7) for s in steps]
    val = [4.0 * math.exp(-0.03 * s) + 0.55 + 0.08 * math.cos(s * 0.35) for s in steps]

    def show(plot: Plot, size: Frame = frame) -> None:
        sys.stdout.write(plot.render(size) + "\n\n")

    show(
        Plot()
        .layer(Line.xy(steps, train).label("train").color("cyan"))
        .layer(Line.xy(steps, val).label("val").color("yellow"))
        .layer(Rule.h(0.5).label("target").dash("Dotted"))
        .layer(Text.at(60, 1.4, "plateau"))
        .title("training")
        .x_label("step")
        .y_label("loss")
    )
    samples = [math.sin(i * 0.37) * 2 + math.cos(i * 0.11) * 1.5 + (i % 7) * 0.2 for i in range(600)]
    show(hist(samples).title("hist(samples)"), frame.with_(height=min(frame.height, 12)))
    show(
        bar(["mon", "tue", "wed", "thu", "fri"], [3, 7, 4.5, 8, 6]).title("bar"),
        frame.with_(height=min(frame.height, 10)),
    )
    show(
        box_plot(["train", "val"], [train, val]).title("box_plot"),
        frame.with_(height=min(frame.height, 10)),
    )
    sys.stdout.write("pip install supremus — https://shergin.github.io/malevich/python/\n")


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())
