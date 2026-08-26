# Terminology

The codebase contract: every public concept is named here before it is named in code.
When a concept is added, renamed, or changes meaning, this file is updated in the same
change. Each entry: what the word means in the wider literature, why this word, and what
it maps to in the crate. Types marked *(planned)* do not exist yet.

## Plot

The retained description of a chart: layers plus scales plus furniture (title, labels,
legend). A plain value — cloneable, inspectable, serializable — with no connection to a
terminal. Rendering is a pure function of a `Plot` and a `Frame`. Maps to `plot::Plot`
(re-exported at the root).

## Layer

One mark bound to data and options, stacked with other layers on shared scales — axis
domains are the union of all layers' data, re-resolved at render time. The layering
concept of every grammar of graphics (Wilkinson 2005; Vega-Lite `layer`; UnicodePlots
`lineplot!`). Maps to `Plot::layer`.

## Mark

A family of geometric primitives that draw data: `Line`, `Points`, `Bars`, `Area`,
`Cells`, `Range`, `Rule`, `Text`. The word follows Observable Plot and Vega-Lite
("mark"), chosen over matplotlib's "artist" (too broad) and "geom" (ggplot jargon).
Chart types are compositions of marks, never peers of them. Maps to the `mark`
module — currently `mark::Line` (points, paired series, or a sampled function; rendered through
subpixels or as box-drawing corners via `LineStyle`),
`mark::Points`, `mark::Bars` (bands or numeric spans, zero-baseline), `mark::Area`
(baseline fills and bands), `mark::Rule` (reference lines), and `mark::Text`
(annotations at data coordinates), `mark::Cells` (value grids as shade ramp plus
colormap, or direct-color images via `Cells::rgb` with a luma shade fallback),
and `mark::Range` (intervals with optional body and marker channels —
error bars, boxes, event ticks), joined under the closed `mark::Mark` enum. The
family is complete.

## Channel

A per-mark visual variable fed from data or set constant: `x`, `y`, `y2`, `color`,
`label`, …. Follows Vega-Lite/Observable Plot "encoding channel". Position channels
accept anything series-shaped (see Series) through constructor arguments; constant
channels are builder methods (`color`, `label`, `style`). The data-bound color
channel is `color_by(categories)` on `Line`, `Points`, `Bars`, and `Range`: distinct
categories in first-appearance order take colors from the plot's categorical
Palette, name themselves in the legend, and — in colorless output — cycle the
default point markers so groups stay separable. Internally, the channel is one stable
label table plus one integer identity per datum. Drawing maps those identities through
the palette directly; a line category transition is an explicit path boundary, and
category-aware M4 preserves that topology while downsampling.

## Series

One column of scalar data after ingestion: contiguous `f64`, where `NaN` is a gap (see
Gap). The ingestion boundary is the `IntoSeries` trait — slices, arrays, and vectors of
any primitive numeric type convert exactly once at the rim (borrowed `f64` slices are
zero-copy), iterators arrive via `FromIterator`, and function sampling arrives with the
marks. The core is monomorphic `f64`. Maps to `data::Series` and `data::IntoSeries`.

## Stat

A data operation that runs before scales see the data. The word follows
seaborn.objects (`Stat`) and ggplot (`stat_*`). It is the module-level umbrella, not
one execution algebra: a stat may be an online accumulator, a reducer, keyed
orchestration, or a batch transform. Maps to the `stat` module: `stat::M4` (with
`stat::m4`, auto-inserted for large line layers), `stat::Bins`/`stat::bins2`
(histograms), `stat::Agg` (group-by with the shared reducer vocabulary),
`stat::BoxStats` (type-7 quartiles, Tukey whiskers), `stat::kde` (Silverman
bandwidth, linear binning), `stat::Window` (trailing rolling reduces), `stat::ecdf`,
`stat::stack`, `stat::lttb`, `stat::Moments`, and `stat::Fit` (streaming ordinary
least squares — bivariate Welford accumulation with Chan's merge; slope, intercept,
R², and the standard error of the mean response, feeding the `trend` preset's line
and confidence band).

## Online accumulator

A bounded state updated one observation at a time. Some accumulators also merge
partial states: `stat::Moments` and `stat::Fit` use order-independent summary state;
`stat::Bins` requires identical geometry; `stat::M4` requires chunks to retain series
order because gaps and first/last points are path topology. Each type states its own
identity, merge preconditions, and ordering requirement. Floating-point merge results
are understood over a fixed reduction tree, not as bitwise-independent reassociation.

## Reducer

A named aggregation shared by every aggregating stat: `Count`, `Sum`, `Mean`,
`Median`, `Min`, `Max`, `Percentile(q)` (type-7, the same estimator the box plot's
quartiles use). One vocabulary across bins, groups, and windows (the Observable Plot
convention): `Agg::reduce` and `Window::reduce` take it directly (their named
methods are sugar over it), `stat::binned` reduces a paired series per histogram
bin, and `stat::quantiles` evaluates many percentiles over one sort. The common
reducers compile to streaming state; median and percentile retain and sort their
finite sample. A reducer promises a result for one collection, not a public merge
operation. Maps to `stat::Reducer`.

## Batch transform

An operation that consumes a complete ordered collection and emits another
collection or structured result. `stat::Window`, `stat::kde`, `stat::ecdf`,
`stat::lttb`, contours, stacking, `bins2`, and `BoxStats` are batch transforms.
They may use online accumulators internally, but that does not make the transform
itself mergeable.

## Scale

A mapping from data domain to raster range with the d3-scale contract: `nice`,
`ticks(n)`, `invert`, and a tick formatter. Position scales: `Linear`, `Log`, `Time`,
`Band`; color scales: sequential, diverging, categorical. Maps to the `scale`
module — currently `scale::Linear` (the affine map, including the raster y-flip),
`scale::Band` (categories across a range, d3 padding model), and `scale::Ticks`
(extended-Wilkinson linear placement, `Ticks::log10` decades, and `Ticks::time`
calendar ticks over unix seconds — UTC, exact Gregorian arithmetic); the axis
specification is `scale::Scale` (`Linear | Log | Time | Bands`) set via
`Plot::x_scale`/`y_scale` (with `log_y()`-style sugar kept). `Bands` works on
either axis: on x it is the bar-family categorical axis, on y it labels matrix
rows — band 0 is the top band, so a Cells grid reads in matrix order, and the
grid's rows and columns must match their axis's bands exactly. Axis titles come from
`Plot::x_label`/`y_label` (x centered below, y vertical along the left edge), and
log/time axes are also
enabled per plot with `Plot::log_x`/`Plot::log_y`/`Plot::time_x`.
`scale::Colormap` covers the sequential and diverging color scales: the curated named
constants (`VIRIDIS`, `MAGMA`, `CIVIDIS`, `GREYS`; diverging `RED_BLUE`,
`PURPLE_ORANGE` — resolvable by name through `Colormap::named`) stay distinguishable
down the whole color ladder, static palettes use `new`, and runtime-generated or
configured palettes move into `try_from_stops`, exposing their RGB stops read-only
through `stops`. A diverging map becomes one by anchoring: `centered_at(mid)` pins a
data value to the ramp middle and the value range spans the larger side
symmetrically, so equal magnitudes get equal intensity and the colorbar admits the
widened span. A sequential map becomes logarithmic with `log()`: positions by
decade, values at or below zero render as gaps (the log-axis rule), decade ticks
on the colorbar; logarithmic and centered are mutually exclusive.
`scale::Palette` is the categorical color scale `color_by` channels
draw from — Okabe–Ito (Wong 2011, print-black omitted) by default, replaceable per
plot with `Plot::palette`; categories past the palette wrap, with marker-shape
cycling keeping them separable where color cannot.

## Ticks

The axis values a scale chooses to label, placed by the extended Wilkinson algorithm
(Talbot, Lin, Hanrahan, InfoVis 2010) — scored for simplicity, coverage, density, and
legibility. Ticks are computed, never supplied as strings, and carry exact-decimal
labels (integer mantissa times a power of ten): labels parse back to their values, share
one fraction width per axis, and never show float artifacts. Maps to `scale::Ticks`.

## Frame

Where and how to render: width and height in cells, charset, color mode (theme joins
later). Frame is render state, not plot state — the same `Plot` renders into many
frames. `Frame::detect()` is the cell-rendering convenience that inspects terminal
size, color variables, locale, and whether stdout is a terminal; pixel capability
detection is the separate, explicitly documented auto boundary. `Plot::render` with
an explicit frame inspects neither.
`Frame::plain()` is the legacy braille snapshot form and `Frame::portable()` is the
conservative deterministic Unicode form. Maps to `plot::Frame` and
`plot::ColorMode`.

## Surface

The subpixel grid that marks draw on during rasterization, before glyphs exist
(raster convention: origin top-left, y down; the data-space flip happens in scales). A
charset codec maps each cell's subpixel pattern to a glyph with independent foreground
and background colors; heatmap half-blocks use both channels for two vertical samples.
Text shares the grid and wins over pixels. Drawing is infallible: out-of-surface clips
and non-finite coordinates draw nothing. Control characters are dropped at the cell
grid — a title, label, or category carrying escape bytes can never smuggle them into
any encoder's output; the only escapes in ANSI output are the encoder's own SGR
sequences (a regression test pins this contract). Maps to `render::Surface`.

## Charset

A glyph tier used to encode the surface. Glyph tables are data, not code. Maps to
`render::Charset`: `Ascii`, `HalfBlocks`, `Quadrants`, `Sextants` (Unicode 13),
`Octants` (Unicode 16 — braille density with solid ink), and `Braille`.
`Frame::detect` sniffs the environment (never probes): quadrants in UTF-8, ASCII for
`TERM=dumb` and non-UTF-8 locales. Dense tiers are explicit because a terminal name
cannot establish the configured font's coverage; `MALEVICH_CHARSET` overrides the
automatic choice.

## Canvas

The drawing-target contract marks rasterize through, generic over fidelity: the cell
`Surface` fills with eighth-block ramps and glyph textures, the pixel canvas with
exact rectangles and real pixels — same mark code, monomorphized per target.
Mid-level operations (`point`, `bar`, `marker`, `patch`) exist precisely where the two
fidelities diverge. Crate-private; maps to `render::Canvas`, implemented by
`render::Surface` and `pixel::PixelCanvas`.

## Graphics

How to draw the plot panel as a real image (feature `pixel`): which protocol, at
what cell size in device pixels. Render state like `Frame`, and a plain value like
everything else — `Graphics::detect()` is stdout-oriented sugar for
`Capabilities::detect().best()`, while `Graphics::detect_for(destination)` keys the
choice to another stream. `None` means the caller falls back to cells. Output stays
hybrid: chrome as text, only the plot rectangle as pixels, undrawn panel transparent.
Maps to `pixel::Graphics`.

## Capabilities

What the terminal can do, as a plain queryable value: the protocols it accepts
(best first), its cell size in device pixels, and how the answer was obtained
(`Source::Probed` or `Source::Sniffed`). Two detection tiers: sniffing reads
environment variables — free, instant, wrong only by omission; probing asks the
terminal itself over one raw-mode `/dev/tty` round trip (kitty graphics query,
XTVERSION, XTSMGRAPHICS, `CSI 16 t`, with DA1 as the ordering barrier) — ground
truth that survives ssh, ~100 ms once per process, and only where writing escapes
is safe: the actual output destination is a tty, no tmux/screen between, `TERM` not
dumb. `Capabilities::detect_for(destination)` supplies that destination explicitly;
the returned value can then drive pure `render_with_capabilities` calls. An
unanswered probe is not evidence; it degrades to the sniff answer. Maps to
`pixel::Capabilities` and `pixel::Source`.

## Protocol

A terminal image protocol the panel can be emitted in: `Sixel` (DEC 1987,
palette-banded, the most widely spoken), `Kitty` (raw RGBA with alpha, the most
capable), `ITerm2` (an inline PNG pinned to the panel's cell box). Encoders are
hand-rolled and dependency-free; each is a thin layer over the shared pixel panel.
Maps to `pixel::Protocol`.

## Theme

Colors and styles as a value you pass, never a global. Today: the layer palette, with
dark and light variants and `COLORFGBG` detection; role colors and cell aspect ratio
join later. Maps to `Theme` (a field of `Frame`).

## Grid

Small multiples: independently rendered plots pasted side by side, one blank column
between neighbors and one blank row between stacked rows (escape-aware padding), cells
filled left to right. Axis sharing is a composition —
fix domains with `Plot::x_domain`/`y_domain` — never a hidden mode. Maps to
`plot::Grid` (re-exported at the root).

## Preset

A plain function composing the grammar into a named chart type: `line()`, `hist()`,
`scatter()`, …. Every preset is provably equal to its grammar expansion (asserted
bit-identical in tests). Presets are the front door; the grammar is discovered, not
required. Statistical presets keep simple defaults while their `_with` variants accept
option values for histogram caps/grids, KDE resolution, violin resolution, and
contour levels; these configured functions return typed errors for invalid data or
options. `_with` means configuration, while a `try_` prefix identifies the checked
twin of an otherwise identical convenience operation. Maps to functions and option
types re-exported at the crate root.

## Stream

Live data machinery, kept at the edge of the crate: `stream::Ring` (a sliding window
shared across threads — the one lock in the library; producers push, the render side
snapshots), `stream::Rate` (counters into deltas), and `stream::Live` (in-place
repaint: cursor up, erase down, one buffered write — flicker-free, scrollback-safe,
never owning the screen). The core stays pure; only this module knows time passes.

## Gap

Missing data, encoded as `NaN` in a series and rendered as a visible break — never
interpolated across, never dropped silently. The de-facto convention of the terminal
plotting field.
