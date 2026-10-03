# Getting started

## Install

```sh
cargo add malevich
```

Two tiny required dependencies, `terminal_size` and `unicode-width`. No build script. `#![forbid(unsafe_code)]`. The crate is 1.x: the public API follows semver, guarded in CI by `cargo-semver-checks` against the last published release. Features are opt-in and named later on this page.

## The first plot

```rust
println!("{}", malevich::line(&[1.0, 5.0, 2.0, 8.0][..]));
```

{{figure start_line}}

That is the whole program. `line` is a preset: a function that hands you a `Plot`. Printing a `Plot` looks at stdout and picks a frame — how wide, about a third as tall, whether color is safe, which glyphs are safe. Send it down a pipe and the text comes out clean, with no escape byte in it.

The other presets take the same shape. Bars take categories and values. A histogram takes a column of numbers and chooses its bins. A scatter takes two columns.

```rust
println!("{}", malevich::bar(["mon", "tue", "wed", "thu", "fri"], &[3.0, 7.0, 4.5, 8.0, 6.0][..]));
```

{{figure start_bar}}

{{figure start_hist}}

{{figure start_scatter}}

Anything series-shaped goes in: slices, arrays, and vectors of any primitive numeric type, or an iterator. Conversion happens exactly once, at the rim, into `f64` where `NaN` is a gap. A borrowed `&[f64]` crosses for free. Inside, the core is monomorphic. No `Float` bound ever appears in a public signature ([why](../../principles/conversion-at-the-rim/)).

## A gap is a gap

Missing data is `NaN`. It renders as a visible break, never interpolated across, never dropped silently. That is the field's convention, and here it is universal: every stat, every mark, every axis speaks it.

{{figure start_gap}}

## Taking the lid off

Every preset is the short spelling of a longer plot. `line(values)` is `Plot::new().layer(Line::y(values))`, and a test checks that the two print the same bytes. When you want a second series, a reference line, or a title, you write that longer plot. The picture does not change.

```rust
use malevich::{Frame, Line, LineStyle, Plot, Rule};

let loss = [4.0, 2.8, 1.9, 1.2, 0.8, 0.6, 0.55, 0.5, 0.48];
let chart = Plot::new()
    .layer(Line::y(&loss[..]).label("loss").style(LineStyle::Corners))
    .layer(Rule::h(0.5).label("target"))
    .title("training")
    .x_label("epoch");
println!("{}", chart.render(&Frame::plain(60, 12)));
```

{{figure start_layers}}

Layers stack on shared scales. The axis domains are the union of every layer's data, resolved at render time. A `label` puts a layer in the legend. A `Rule` is a reference line at one value. The [grammar page](../grammar/) builds a six-layer chart this way, one plate per step.

## Frames

A `Plot` describes a chart. A `Frame` describes one rendering of it: width and height in cells, a charset, a color mode, a theme. `Plot::render(&frame)` is a pure function of the two — call it with a different frame and the same plot lays itself out again.

| constructor | what it is for |
|---|---|
| `Frame::detect()` | reads the terminal: size, `NO_COLOR`, `COLORTERM`, `TERM`, `MALEVICH_CHARSET`, `COLORFGBG` |
| `Frame::plain(w, h)` | braille glyphs, no color — the snapshot form tests use |
| `Frame::portable(w, h)` | quadrants, no color — the conservative Unicode form |
| `Frame { width: 100, ..Frame::detect() }` | any field, overridden |

The three environment-reading conveniences are `Display`, `Frame::detect`, and `render_best`. Everything else inspects nothing, which is why a plot can be built on one thread and rendered on another, snapshot-tested as a string, or serialized and rendered on the far side of a socket ([why](../../principles/frame-is-run-state/)).

{{sizes start_layers 60x12 40x9 26x6 | The same plot value in three frames. When the frame shrinks, furniture sheds before data: the legend, then the title, then tick density. The data region is the last thing standing.}}

## In a pipe, in a log, in a test

`Frame::detect` sees that stdout is not a terminal and drops color. The charset choice never emits anything a file cannot hold. So the plot you print in CI is the plot you read in the log, and the plot a language model reads is the same grid with the same exact labels. `Frame::plain` is that form on demand:

{{plain start_layers | What a pipe sees. The braille tier is the deterministic snapshot form; nothing a chart says lives only in its color.}}

## What is in the box

The presets, re-exported at the crate root and each provably equal to its expansion. The shape of data you have, and the preset that draws it, is [which chart](../choose/).

| preset | chart |
|---|---|
| `line`, `scatter`, `bar`, `sparkline` | the first look |
| `hist`, `hist_with`, `density`, `density_with`, `ecdf`, `ecdf_with`, `stairs`, `stairs_with` | one distribution |
| `box_plot`, `box_plot_with`, `violin`, `violin_with` | distributions per category |
| `trend`, `trend_with`, `error_bars`, `error_bars_asymmetric` | relationships and measurements |
| `heatmap`, `heatmap_with`, `hist2d`, `hist2d_with`, `contour`, `contour_with`, `contourf`, `contourf_with`, `quiver` | grids and fields |
| `table`, `table_with`, `try_table`, `describe`, `describe_with` | numbers as a stat table |

A `_with` variant takes an options value and returns a typed error for invalid data or options. Its defaults reproduce the plain preset exactly. A `try_` prefix marks the checked twin of an otherwise identical convenience.

The features:

| feature | adds |
|---|---|
| `ratatui` | a `PlotWidget` for any plot, and a stateful, interactive one (depends only on `ratatui-core`) — [interaction](../interaction/) |
| `pixel` | the plot panel as a real sixel, kitty, or iTerm2 image — [real pixels](../pixels/) |
| `evcxr` | rich output in Jupyter through the Evcxr kernel — [notebooks](../notebooks/) |
| `serde` | every spec type round-trips; `Document` is the versioned envelope — [specs as data](../serde/) |
| `ndarray` | one-dimensional arrays and views plot directly, contiguous storage zero-copy |

The HTML and SVG cards (`Plot::to_html`, `Plot::to_svg`) need no feature at all: a notebook, a README, or this page is one more terminal.

## From the shell and from JavaScript

The same renderer, from any shell. [`kaz`](../../cli/) is a stdin-first plotter, one subcommand per chart. The plot goes to stderr, so data can flow on through stdout.

```sh
cargo install malevich-cli
cat loss.tsv | kaz line -t training
awk '{print $5}' access.log | kaz hist
kaz scatter penguins.tsv -H --by species --emit-code   # the equivalent Rust program
```

And the same engine, compiled to wasm, on [npm](../../js/). The Ink widget shares the ratatui adapter's interaction grammar:

```js
import { line } from "malevich";
console.log(line([1, 5, 2, 8]));
```

## Where next

- [The grammar](../grammar/) — one chart, built up mark by mark.
- [The eight marks](../marks/) — every channel, one plate each.
- [The gallery](../../gallery/) — fifty-odd charts with their sources, as a ladder.
- [The playground](../../playground/) — your numbers, any chart, any frame, and the Rust it would take.
