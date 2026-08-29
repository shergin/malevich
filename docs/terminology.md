# Terminology

The public vocabulary, and a codebase contract: every public concept is named
here before it is named in code. When a concept is added, renamed, or changes
meaning, this file is updated in the same change. Each entry gives the word's
meaning in the wider literature and what it maps to in the crate. Design
arguments live in [vision](vision.md) and [principles](principles/); entries
here only name their conclusions.

## Plot

The retained description of a chart: layers plus scales plus furniture
(title, labels, legend). A plain value — cloneable, inspectable,
serializable — with no connection to a terminal. Rendering is a pure function
of a `Plot` and a `Frame`. Maps to `plot::Plot` (re-exported at the root).
See [The frame is run state](principles/frame-is-run-state.md).

## Layer

One mark bound to data and options, stacked with other layers on shared
scales — axis domains are the union of all layers' data, re-resolved at
render time. The layering concept of every grammar of graphics (Wilkinson
2005; Vega-Lite `layer`). Maps to `Plot::layer`.

## Mark

A family of geometric primitives that draw data. The word follows Observable
Plot and Vega-Lite ("mark"), chosen over matplotlib's "artist" (too broad)
and "geom" (ggplot jargon). Eight marks, joined under the closed `mark::Mark`
enum: `Line` (points, paired series, or a sampled function), `Points`,
`Bars` (bands, contiguous numeric spans, or free positions; rising from the
zero baseline, or from a per-bar `base` — the y2-style channel that makes
stacked bars, grouped bars, and waterfalls plain compositions), `Area`
(baseline fills and bands), `Cells` (value grids, rgb images, or categorical
class regions),
`Range` (intervals with optional body and marker channels), `Rule`
(reference lines), and `Text` (annotations at data coordinates). Chart types
are compositions of marks, never peers of them. The family is complete. See
[What earns a concept](principles/what-earns-a-concept.md).

## Channel

A per-mark visual variable fed from data or set constant: `x`, `y`, `y2`,
`color`, `label`, …. Follows Vega-Lite and Observable Plot ("encoding
channel"). Position channels accept anything series-shaped through
constructor arguments; constant channels are builder methods. The data-bound
color channel is `color_by(categories)` on `Line`, `Points`, `Bars`, and
`Range`: categories take palette colors in first-appearance order, name
themselves in the legend, and cycle marker shapes in colorless output so
groups never vanish in a pipe.

## Series

One column of scalar data after ingestion: contiguous `f64`, where `NaN` is a
gap (see Gap). The ingestion boundary is the `IntoSeries` trait — slices,
arrays, and vectors of any primitive numeric type convert exactly once at the
rim; borrowed `f64` slices are zero-copy. The core is monomorphic `f64`.
Maps to `data::Series` and `data::IntoSeries`. See
[Conversion lives at the rim](principles/conversion-at-the-rim.md).

## Stat

A data operation that runs before scales see the data. The word follows
seaborn.objects (`Stat`) and ggplot (`stat_*`). It is the module-level
umbrella, not one execution algebra: a stat may be an online accumulator, a
reducer, keyed orchestration, or a batch transform. Maps to the `stat`
module — `M4`, `Bins`/`bins2`, `Agg`, `BoxStats`, `kde`, `Window`, `ecdf`,
`roc`/`auc`, `ewma`, `stack`, `lttb`, `Moments`, `Fit` (streaming least
squares behind the `trend` preset), and `nearest` (the crosshair-snapping
lookup: the index of the closest finite value, so cursor readouts show a
datum that exists rather than an interpolation).

## Online accumulator

A bounded state updated one observation at a time. Some accumulators also
merge partial states: `stat::Moments` and `stat::Fit` use order-independent
summary state; `stat::Bins` requires identical geometry; `stat::M4` requires
chunks in series order because gaps and first/last points are path topology.
Each type states its own identity, merge preconditions, and ordering
requirement. Merge results are understood over a fixed reduction tree, not as
bitwise-independent reassociation.

## Reducer

A named aggregation shared by every aggregating stat: `Count`, `Sum`,
`Mean`, `Median`, `Min`, `Max`, `Percentile(q)` (type-7, the estimator the
box plot's quartiles use). One vocabulary across bins, groups, and windows —
the Observable Plot convention — so a rolling p95 or a binned median is one
call. A reducer promises a result for one collection, not a public merge
operation. Maps to `stat::Reducer`.

## Batch transform

An operation that consumes a complete ordered collection and emits another
collection or structured result: `Window`, `kde`, `ecdf`, `roc`, `auc`,
`ewma`, `lttb`, contours, stacking, `bins2`, `BoxStats`. A batch transform
may use online accumulators internally; that does not make the transform
itself mergeable.

## Scale

A mapping from data domain to raster range with the d3-scale contract:
`nice`, `ticks(n)`, `invert`, and a tick formatter. Position scales:
`Linear`, `Log`, `Time`, `Band` — the axis specification is `scale::Scale`
(`Linear | Log | Time | Bands`), set via `Plot::x_scale`/`y_scale`. `Bands`
works on either axis: on x it is the bar-family categorical axis, on y it
labels matrix rows in matrix order. Color scales: `scale::Colormap` covers
sequential and diverging ramps (curated named constants — `VIRIDIS`,
`MAGMA`, `CIVIDIS`, `GREYS`, `RED_BLUE`, `PURPLE_ORANGE`; `centered_at(mid)`
anchors a diverging map to a data value, `log()` makes a sequential map
logarithmic with decade ticks); `scale::Palette` is the categorical scale
`color_by` draws from — Okabe–Ito (Wong 2011) by default.

## Ticks

The axis values a scale chooses to label, placed by the extended Wilkinson
algorithm (Talbot, Lin, Hanrahan 2010) — scored for simplicity, coverage,
density, and legibility. Ticks are computed, never supplied as strings, and
carry exact-decimal labels: they parse back to their values, share one
fraction width and one SI prefix per axis, and never show float artifacts.
Maps to `scale::Ticks`. See
[The axes are the product](principles/axes-are-the-product.md).

## Frame

Where and how to render: width and height in cells, charset, color mode,
theme. Frame is run state, not plot state — the same `Plot` renders into many
frames. `Frame::detect()` is the convenience that inspects the environment;
`Plot::render` with an explicit frame inspects nothing. `Frame::plain()` is
the deterministic braille snapshot form; `Frame::portable()` is the
conservative Unicode form. Maps to `plot::Frame` and `plot::ColorMode`. See
[The frame is run state](principles/frame-is-run-state.md).

## Mapping

The resolved geometry of one render, as a queryable value: the plot rectangle
in cells, the resolved axis windows, and the cell ↔ data mapping both ways —
the `invert` half of the scale contract, reachable at plot level. Obtained
purely from `Plot::mapping(&frame)`; the ratatui stateful widget caches one
per render. Queries answer in the coordinate conventions marks use (band
indices, unix seconds), name the plot panel as a `Panel` value rather than a
bare tuple, expose a categorical axis's labels (`x_categories`), disclose
cell quantization (`x_span_at`/`y_span_at`), and format values the way the
axis formats its own labels (`format_x`/`format_y`).
Derived state, deliberately not serializable. This is the physics interactive
hosts build on: malevich never handles input — a host maps its events to
questions a `Mapping` can answer. Maps to `plot::Mapping`.

## Viewport

An axis window pair for interactive viewing: zoom and pan as pure domain
arithmetic over `x_domain`/`y_domain` — a scale option, never a render mode,
which is why zooming into millions of points re-aggregates (M4) to the new
window with no special machinery. `None` on an axis means automatic; a window
is seeded from `Mapping::viewport` ("the view I am looking at") and
transformed by value: `zoom` around an anchor (decade space on log axes),
`pan` by a fraction, `clamp` to an extent, `tail` for follow-the-stream,
`reset` to automatic. Applied with `Plot::viewport`. Serializable — it is
spec-shaped state a host may persist. Maps to `plot::Viewport`.

## Widget

The ratatui adapter (feature `ratatui`, depending only on `ratatui-core`):
`Plot::widget()` renders any plot into a `Buffer` as cells and styles. The
stateless `Widget` impl is fire-and-forget; the `StatefulWidget` impl threads
a `PlotState` — the interaction controller: it caches the render's `Mapping`
for hit-testing, applies its `Viewport` on the next draw, and interprets the
default mouse gestures (hover crosshair, wheel zoom at the cursor, left-drag
pan, right-drag rubber-band zoom) from the backend-neutral `Mouse` vocabulary
the host feeds it. The cursor snaps to the data: for every point-backed line
and points layer, the readout lists the value of the datum nearest the
cursor's x inside the visible window — axis-formatted, its cell highlighted,
a gap shown as `—` rather than an interpolation (`snap(false)` returns to
plain cursor coordinates). The widget never reads the terminal: event loops,
mouse capture, and key policy stay in the host, and the gestures are a preset
over the public physics — a host with different policy drives `Viewport` and
`Mapping` directly. Interaction chrome (crosshair, snap highlights, selection
band, readout) draws into the buffer only; the plot value renders
byte-identically with or without it. With the `pixel` feature,
`widget().graphics(g)` renders the panel as a real image: the buffer holds
skip-reserved ground, the `PlotState` carries the encoded block, and the host
emits it after `terminal.draw` with `Graphics::present` (one synchronized
write, new placements created before old ones are retired;
`Graphics::retire` deletes a view's images on the way out).
Interaction chrome then becomes annotation marks drawn into the image, and
hit-testing is unchanged — the mapping answers in cells regardless of what
fills them. The pixel render paces itself (~30 full encodes per second):
within the window, an unchanged-view render reuses the image already on
screen, so hover floods and tick redraws cost nearly nothing; a changed
viewport or rectangle always renders.

## Surface

The subpixel grid that marks draw on during rasterization, before glyphs
exist (raster convention: origin top-left, y down; the data-space flip
happens in scales). A charset codec maps each cell's subpixel pattern to a
glyph with independent foreground and background colors. Text shares the grid
and wins over pixels. Drawing is infallible: out-of-surface clips,
non-finite coordinates draw nothing, and control characters are dropped at
the cell grid, so no input string can smuggle escape bytes into any encoder's
output. Maps to `render::Surface`.

## Charset

A glyph tier used to encode the surface; glyph tables are data, not code.
Maps to `render::Charset`: `Ascii`, `HalfBlocks`, `Quadrants`, `Sextants`
(Unicode 13), `Octants` (Unicode 16), and `Braille`. `Frame::detect` sniffs
the environment, never probes; dense tiers are explicit because a terminal
name cannot establish the configured font's coverage. See
[Degradation is the contract](principles/degradation-is-the-contract.md).

## Canvas

The drawing-target contract marks rasterize through, generic over fidelity:
the cell `Surface` fills with eighth-block ramps and glyph textures, the
pixel canvas with exact rectangles and real pixels — same mark code,
monomorphized per target. Crate-private; maps to `render::Canvas`.

## Graphics

How to draw the plot panel as a real image (feature `pixel`): which protocol,
at what cell size in device pixels. Render state like `Frame`, and a plain
value like everything else. `None` means the caller falls back to cells.
Output stays hybrid: chrome as text, only the plot rectangle as pixels. Maps
to `pixel::Graphics`. See [the pixels guide](pixels.md).

## Capabilities

What the terminal can do, as a plain queryable value: the protocols it
accepts, its cell size in device pixels, and how the answer was obtained
(`Source::Probed` or `Source::Sniffed`). Sniffing reads environment
variables — free, wrong only by omission. Probing asks the terminal itself
over one raw-mode round trip — ground truth, and only where writing escapes
is safe. An unanswered probe is not evidence; it degrades to the sniff
answer. Maps to `pixel::Capabilities` and `pixel::Source`.

## Protocol

A terminal image protocol the panel can be emitted in: `Sixel` (DEC 1987,
the most widely spoken), `Kitty` (raw RGBA with alpha, the most capable),
`ITerm2` (an inline PNG). Encoders are hand-rolled and dependency-free. Maps
to `pixel::Protocol`.

## Theme

Colors and styles as a value you pass, never a global. Today: the layer
palette, with dark and light variants and `COLORFGBG` detection. Maps to
`Theme` (a field of `Frame`).

## Grid

Small multiples: independently rendered plots pasted side by side
(escape-aware padding), cells filled left to right. Axis sharing is a
composition — fix domains with `Plot::x_domain`/`y_domain` — never a hidden
mode. Maps to `plot::Grid` (re-exported at the root).

## Preset

A plain function composing the grammar into a named chart type: `line()`,
`hist()`, `scatter()`, …. Every preset is provably equal to its grammar
expansion (asserted byte-identical in tests). Presets are the front door; the
grammar is discovered, not required. `_with` means "configured with an
options value"; a `try_` prefix identifies the checked twin of an otherwise
identical convenience. Maps to functions and option types re-exported at the
crate root. See [Presets are packaging](principles/presets-are-packaging.md).

## Stream

Live data machinery, kept at the edge of the crate: `stream::Ring` (a
sliding window shared across threads — the one lock in the library),
`stream::Rate` (counters into deltas), and `stream::Live` (in-place repaint:
cursor up, erase down, one buffered write — flicker-free, scrollback-safe,
never owning the screen). The core stays pure; time enters only at the rims —
this module, and the ratatui widget's pixel pacing (below), which uses a
monotonic clock to skip redundant re-encodes. Every render that does run
remains a pure function of its inputs.

## Gap

Missing data, encoded as `NaN` in a series and rendered as a visible break —
never interpolated across, never dropped silently. The de-facto convention of
the terminal plotting field.

## Further reading

- Wilkinson, *The Grammar of Graphics* (2005).
- Talbot, Lin, Hanrahan, "An Extension of Wilkinson's Algorithm for
  Positioning Tick Labels on Axes" (InfoVis 2010).
- Jugel, Fischer, Mahlmann, Markl, "M4: A Visualization-Oriented Time Series
  Data Aggregation" (PVLDB 2014).
- Hyndman and Fan, "Sample Quantiles in Statistical Packages" (1996) — the
  type-7 estimator.
- Wong, "Points of view: Color blindness" (Nature Methods 2011) — the
  Okabe–Ito palette.
- [d3-scale](https://github.com/d3/d3-scale),
  [Vega-Lite](https://vega.github.io/vega-lite/), and
  [Observable Plot](https://observablehq.com/plot/) — the conventions the
  vocabulary follows.
