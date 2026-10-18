# supremus

**Terminal plotting for Python: a small grammar of marks, honest axes,
millions of points.**

The engine is the [malevich](https://crates.io/crates/malevich) Rust crate,
compiled into one small wheel. A plot is a value. Hand it a frame and it draws;
the function is pure. The library never owns the terminal, it has no runtime
dependencies, and `pip install` never compiles anything.

The docs put a plate on every mark, stat, and scale, and the playground runs
this same engine in the browser:
[shergin.github.io/malevich](https://shergin.github.io/malevich/).

```sh
pip install supremus
uvx supremus                   # a tour, sized to your terminal
printf '1 5 2 8' | uvx supremus line
```

Python 3.11+. One `abi3` wheel per platform.

```python
from supremus import line

print(line([1, 5, 2, 8]))
```

```text
7.5 ┤                                 ⡠⠊
    │                               ⡠⠊
    │                             ⡠⠊
5.0 ┤         ⣀⠔⠊⠑⠢⢄⡀           ⡠⠊
    │      ⣀⠔⠊      ⠈⠑⠢⢄⡀     ⡠⠊
2.5 ┤   ⡠⠔⠉             ⠈⠑⠢⢄⡠⠊
    │⡠⠔⠉
0.0 ┤
    └┬──────────┬───────────┬──────────┬
     0          1           2          3
```

```python
from supremus import Frame, Line, Plot

loss = [4, 2.8, 1.9, 1.2, 0.8, 0.6]
chart = Plot().layer(Line.y(loss).label("loss").style("Corners")).title("training")
print(chart.render(Frame.detect()))
```

`print(plot)` and `str(plot)` detect a frame. `plot.render(frame)` is the
pure path. In a test, pass `Frame.plain` or `Frame.portable`. In a notebook,
end the cell with the plot.

This is **0.x**. The Python API can still move. The renderer is the 1.x Rust
crate (`supremus.ENGINE_VERSION`).

## Why supremus

Python has terminal charts. What it does not have is this engine.

- **Eight marks, and that is the catalog.** Line, points, bars, area, cells,
  range, rule, and text, composed with a stats layer and shared scales.
  Presets (`line`, `hist`, `box_plot`, `violin`, `trend`, …) are packaging:
  the same plot you would write by hand.
- **Axes that are actually good.** Ticks are placed by extended Wilkinson.
  Labels are exact decimals, and they parse back to their values. One SI
  prefix per axis. Log axes, calendar time, band axes. Never
  `0.30000000000000004`.
- **Millions of points.** A long line reduces by M4, one bucket per rendered
  column, and the pixels match drawing every point. Ten million points render
  in tens of milliseconds, with the GIL released.
- **A bad terminal still gets a chart.** The ladders run from Unicode 16
  octants down to plain ASCII, and from truecolor down to a clean pipe.
  `None` is always a visible gap.
- **A plot is a value.** Builders are immutable. There is no hidden terminal
  state, no current figure. `Plot.render` inspects nothing. Detection lives
  in `Frame.detect`.
- **It shows up where you look.** `print` in a terminal, the HTML card in
  Jupyter and marimo, a renderable in Rich, a widget in Textual, a backend
  for `df.plot()`. The plain text is the same chart an agent reads.

The argument is in the crate's [docs/vision.md](https://github.com/shergin/malevich/blob/main/docs/vision.md).
This package is the Python rim around that crate. One oracle, not a rewrite.
The name is Malevich's own title for the work: *Supremus*.

## Presets and the grammar

```python
from supremus import (
    Area, Frame, Grid, Line, Plot, Points, Rule, Text,
    bar, box_plot, density, describe, ecdf, heatmap, hist, hist2d,
    line, scatter, stairs, table, trend, violin,
)

print(bar(["mon", "tue", "wed", "thu", "fri"], [3, 7, 4.5, 8, 6]))
print(hist(samples))
print(box_plot(["train", "val"], [train_loss, val_loss]))
print(describe(["train", "val"], [train_loss, val_loss]))
```

Eight marks: `Line`, `Points`, `Bars`, `Area`, `Cells`, `Range`, `Rule`,
`Text`. A preset is a proven composition. Shared goldens prove the Python
output is byte-identical to the crate, for the same document and frame.

```python
from supremus import Bars, Cells, Colormap, Range

Plot().layer(Cells.matrix(4, values).colormap(Colormap.VIRIDIS))
Plot().layer(Range.over(["a", "b"], low, high).body(q1, q3).marker(median))
(Plot()
    .layer(Bars.new(["a", "b"], lower).label("a"))
    .layer(Bars.new(["a", "b"], upper).base(lower).label("b")))
```

```python
chart = (
    Plot()
    .layer(Line.y(train).label("train").color("cyan"))
    .layer(Line.y(val).label("val").color("yellow"))
    .layer(Rule.h(0.5).label("target").dash("Dotted"))
    .layer(Text.at(80, 3.2, "overfit?"))
    .title("loss")
    .x_label("step")
    .y_label("loss")
)
```

A series is a list (`None` is a gap), an `array('d')`, a numpy array, or
anything with `to_numpy()`: a pandas `Series` or `Index`, a polars `Series`,
a pyarrow array. Lists are read in one pass in Rust; buffers are copied
once. Datetimes become unix seconds and the x axis becomes a `Time` scale.
Nested `[(x, y), ...]` pairs are not a series. A mark that wants two channels
takes two series (`Line.xy(x, y)`, `scatter(x, y)`).

## Frame

Size, charset, color, theme: where and how this drawing goes. A frame is
run state, not plot state. The same plot renders into many frames.

| constructor | what |
|---|---|
| `Frame.detect()` | reads the environment and the stream: size, `NO_COLOR`, `COLORTERM`, `TERM`, `MALEVICH_CHARSET`, `COLORFGBG` |
| `Frame.plain(w, h)` | braille, no color — the snapshot form |
| `Frame.portable(w, h)` | quadrants, no color — conservative Unicode |
| `frame.with_(height=8)` | copy with fields replaced |

The native module never reads the environment. A snapshot test always passes
an explicit frame.

## Notebooks

A cell that ends in a plot shows the terminal card: the exact cell grid, as
a `<pre>` of inline-styled spans, in Jupyter, VS Code, and marimo. The
default card is 100×26 quadrants on a dark ground; `plot.to_html(frame)`
and `plot.to_svg(frame)` are the pure forms.

```python
chart                      # the last expression of a cell
chart.to_html(Frame.portable(120, 30).with_(theme=Theme.LIGHT))
```

## Rich and Textual

A plot is a Rich renderable. It fills the width Rich offers and the height
Rich asks for, as segments from the cell grid, so `Panel`, `Layout`,
`Columns`, and `Live` just work.

```python
from rich.console import Console
from rich.panel import Panel

Console().print(Panel(line(loss), title="training"))
```

The Textual widget is optional: `pip install supremus[textual]`.

```python
from supremus.textual import PlotWidget

yield PlotWidget(line(loss).title("training"))
```

That paints on its own and resizes with its container. For interaction,
give it a `PlotState`, the same controller as the ratatui and Ink widgets:
hover crosshair with a snapped readout, wheel zoom at the cursor, left-drag
pan, right-drag rubber band, `+`/`-`/arrows/`r` keys. Band axes stay put.
Two stacked panes share an x window by `link_x(active, passive)`.

## pandas

```python
import pandas as pd

pd.set_option("plotting.backend", "supremus")
df.plot()                   # a line per column
df["latency"].plot(kind="hist")
```

`line`, `bar`, `barh`, `hist`, `box`, `kde`, `area`, `scatter`, and
`hexbin` (as rectangular bins) map onto presets; `pie` is refused. The
result is a `Plot`: print it, or keep composing.

## Raster, mapping, viewport

`plot.render(frame)` is a string. `plot.raster(frame)` is the cell grid
under it. `plot.mapping(frame)` is the geometry of one render: cell to data,
data to cell, labels as the axes format them. A `Viewport` is a pair of
axis windows; a zoom is a scale option, so the engine re-aggregates to the
visible window on the next render.

```python
mapping = chart.mapping(frame)
x, y = mapping.data_at(column, row)
view = mapping.viewport().zoom_x(0.8, x)
print(chart.viewport(view).render(frame))
```

## Pixels

Where the terminal speaks kitty, iTerm2, or sixel, `plot.render_best(frame)`
draws the panel as a real image with text chrome around it;
`detect_graphics()` is the sniff and `render_pixels(frame, Graphics("kitty"))`
the pure path. `MALEVICH_GRAPHICS=none` keeps cells.

## Errors

Builders raise `ValueError` or `TypeError` at the call that is wrong: unequal
channels, a non-finite bound, text where numbers were expected. `render`
raises `SupremusError` only for a spec the engine refuses, with `code`
naming the crate's error variant. A frame too small for its furniture sheds
the furniture; it never raises.

## License

MIT OR Apache-2.0, like the crate.
