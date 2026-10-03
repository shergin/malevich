# Terminology

This is the public vocabulary, and a codebase contract. Every public concept
is named here before it is named in code. When a concept is added, renamed,
or changes meaning, this file is updated in the same change. Each entry gives
the word's meaning in the wider literature, and what it maps to in the crate.
Design arguments live in [vision](vision.md) and [principles](principles/).
Entries here only name their conclusions.

## Plot

A plot is the retained description of a chart: layers, plus scales, plus
furniture. Furniture is the title, the labels, the legend, and whether the
axes are drawn at all. `axes(false)` is a sparkline's furniture. A plot is a
plain value — cloneable, inspectable, serializable — with no connection to a
terminal. Rendering is a pure function of a `Plot` and a `Frame`. Maps to
`plot::Plot` (re-exported at the root). See
[The frame is run state](principles/frame-is-run-state.md).

## Layer

A layer is one mark bound to data and options, stacked with other layers on
shared scales. An axis domain is the union of all layers' data, and it is
re-resolved at render time. The word is the layering concept of every grammar
of graphics (Wilkinson 2005; Vega-Lite `layer`). Maps to `Plot::layer`.

## Mark

A mark is a family of geometric primitives that draw data. The word follows
Observable Plot and Vega-Lite ("mark"), chosen over matplotlib's "artist"
(too broad) and "geom" (ggplot jargon). Eight marks are joined under the
closed `mark::Mark` enum: `Line`, `Points`, `Bars`, `Area`, `Cells`,
`Range`, `Rule`, and `Text`.

`Line` is made from points, a paired series, or a sampled function. `Bars`
is bands, contiguous numeric spans, free positions, or explicit intervals.
`Bars::intervals(starts, ends, values)` is the histogram with irregular bins.
Bars rise from the zero baseline, or from a per-bar `base` — the y2-style
channel that makes stacked bars, grouped bars, and waterfalls plain
compositions. `horizontal` turns any placement sideways: the bands go down
the y axis in reading order, and the values go along x. That sideways
placement is the `barh` of the catalog.

`Area` is baseline fills and bands. `Cells` is value grids, rgb images, or
categorical class regions. `Range` is intervals with optional body and marker
channels. `Rule` is reference lines at one value, and spans. `h_span` and
`v_span` wash the band between two values across the plot: a recession, a
warm-up phase, a tolerance window. `Text` is annotations at data coordinates.

Chart types are compositions of marks, never peers of them. The family is
complete. See [What earns a concept](principles/what-earns-a-concept.md).

## Channel

A channel is a per-mark visual variable, fed from data or set constant: `x`,
`y`, `y2`, `color`, `label`, and the rest. The word follows Vega-Lite and
Observable Plot ("encoding channel"). Position channels accept anything
series-shaped, through constructor arguments. Constant channels are builder
methods.

The data-bound color channel is `color_by(categories)` on `Line`, `Points`,
`Bars`, and `Range`. Categories take palette colors in first-appearance
order, name themselves in the legend, and cycle marker shapes in colorless
output, so groups never vanish in a pipe.

The alignment channel is `align(Align)` on `Text` — `Left` (the default:
start at the anchor and extend right), `Center`, and `Right` (end at the
anchor). On a `Bands` x axis, the band nearest the anchor becomes the box,
with exactly the geometry the band's own header label uses — its rounded
center, its step-wide budget — so aligned text and band labels land in
lockstep. Text wider than the box clips to it, ending with a truncation `.`.
Digits from a neighboring column are never mixed into a number. Stat tables
and annotated matrices are set in this channel.

## Series

A series is one column of scalar data after ingestion: contiguous `f64`,
where `NaN` is a gap (see Gap). The ingestion boundary is the `IntoSeries`
trait. Slices, arrays, and vectors of any primitive numeric type convert
exactly once, at the rim. Borrowed `f64` slices are zero-copy. The core is
monomorphic `f64`. Maps to `data::Series` and `data::IntoSeries`. See
[Conversion lives at the rim](principles/conversion-at-the-rim.md).

## Stat

A stat is a data operation that runs before scales see the data. The word
follows seaborn.objects (`Stat`) and ggplot (`stat_*`). It is the
module-level umbrella, not one execution algebra. A stat may be an online
accumulator, a reducer, keyed orchestration, or a batch transform. Maps to
the `stat` module: `M4`, `Bins`/`bins2`, `calendar_bins`, `Agg`, `BoxStats`,
`kde`/`kde_with`, `Window`, `jitter`, `steps`,
`cumsum`/`diff`/`rank`/`normalize`, `ecdf`, `roc`/`auc`, `ewma`,
`stack`/`stack_with`, `dodge`, `lttb`, `Moments`, `Fit`, and `nearest`.

`Bins::heights` rescales the counts under a `Normalization` — count,
probability, percent, density per unit of x — and accumulates them. `hist`
and `kaz hist` share it.

`calendar_bins` counts per hour, day, ISO week, month, or year — a
`TimeUnit` — over unix timestamps. Buckets keep their true length, empties
are kept, and the result feeds `Bars::intervals`.

`BoxStats` uses type-7 quartiles. The whiskers are `Whiskers::Tukey(k)`,
`Percentiles(lo, hi)`, or `MinMax`.

`kde` and `kde_with` use Silverman's bandwidth by default, scaled or fixed
via `Bandwidth`. The bounds reflect the kernels, so a latency density stays
above zero. There is also the cumulative form.

`Window` is a sliding window, anchored at its end by default, or centered or
leading via `WindowAnchor`. `strict` gaps the positions whose window is
incomplete.

`jitter` uses van der Corput offsets that spread a strip of points evenly,
with no seed.

`steps` is the piecewise-constant expansion `stairs` draws, changing after,
before, or midway between samples per `StepDirection`.

`cumsum`, `diff`, `rank`, and `normalize` are the series maps: running sums,
differences, ranks, and division by a reducer of the whole series — index
charts, percent-of-peak.

`stack` and `stack_with` build cumulative bands, positives above the baseline
and negatives below it. `StackOffset::Normalize` is the 100 % stack, `Center`
the streamgraph silhouette, and `StackOrder::Sum` piles the largest series
first.

`dodge` is side-by-side positions for grouped bars, one series per value
series, fed to `Bars::at` — stack's sibling, for bars beside each other.

`Fit` is streaming least squares behind the `trend` preset. `nearest` is the
crosshair-snapping lookup: the index of the closest finite value, so cursor
readouts show a datum that exists rather than an interpolation.

## Online accumulator

An online accumulator is a bounded state updated one observation at a time.
Some accumulators also merge partial states. `stat::Moments` and `stat::Fit`
use order-independent summary state. `stat::Bins` requires identical
geometry. `stat::M4` requires chunks in series order, because gaps and
first/last points are path topology. Each type states its own identity, merge
preconditions, and ordering requirement. Merge results are understood over a
fixed reduction tree, not as bitwise-independent reassociation.

## Reducer

A reducer is a named aggregation shared by every aggregating stat: `Count`,
`Sum`, `Mean`, `Median`, `Min`, `Max`, `Percentile(q)` (type-7, the
estimator the box plot's quartiles use), `Deviation`, `Variance`, `StdErr`
(sample statistics, `n − 1`, gaps below two values — the error-band
vocabulary of Vega-Lite and seaborn's `Est`), `First`, and `Last`. That is
one vocabulary across bins, groups, and windows — the Observable Plot
convention — so a rolling p95, a binned median, or a group's mean ± se is
one call. A reducer promises a result for one collection, not a public
merge operation. Maps to `stat::Reducer`.

## Batch transform

A batch transform is an operation that consumes a complete ordered collection
and emits another collection or a structured result: `Window`, `kde`, `ecdf`,
`roc`, `auc`, `ewma`, `lttb`, `cumsum`, `diff`, `rank`, `normalize`,
contours, stacking, `bins2`, and `BoxStats`. A batch transform may use online
accumulators internally. That does not make the transform itself mergeable.

## Scale

A scale is a mapping from a data domain to a raster range, with the d3-scale
contract: `nice`, `ticks(n)`, `invert`, and a tick formatter. Position
scales are `Linear`, `Log`, `Time`, and `Band`. The axis specification is
`scale::Scale` (`Linear | Integer | Log | Time | Bands`), set via
`Plot::x_scale`/`y_scale`. `Integer` is a linear axis whose tick step never
drops below one — counts, ranks, sizes — so a tall frame over `0..3` labels
`0, 1, 2, 3`, never `0.5`. `hist` counts on it. `Bands` works on either
axis: on x it is the bar-family categorical axis, and on y it labels matrix
rows in matrix order.

A linear or integer axis may also carry a unit, `scale::Unit`, set via
`Plot::x_unit`/`y_unit`. `Unit::si("B")` puts the axis's one SI prefix before
the unit (`2.5 MB`, `0 kB`). `Unit::Bytes` chooses ticks nice in the binary
unit (`512 KiB`, `1.5 GiB`). `Unit::suffix("%")` appends a bare suffix and
never a prefix (`45%`). The unit is a scale option: the ticks stay the same
ticks, only the labels change, and the `Mapping` readout speaks the same
unit.

A domain is a scale option too. `Plot::x_domain`/`y_domain` fix both ends of
an axis exactly, and `x_min`/`x_max`/`y_min`/`y_max` fix one end while the
other fits the data and grows to its outer tick — a rate chart floored at
zero whose top follows the traffic.

`scale::Colormap` covers sequential and diverging ramps. The curated named
constants are `VIRIDIS`, `MAGMA`, `CIVIDIS`, `GREYS`, `RED_BLUE`, and
`PURPLE_ORANGE`. `centered_at(mid)` anchors a diverging map to a data value.
`log()` makes a sequential map logarithmic, with decade ticks. `domain(lo, hi)`
fixes the value range so two grids read on one scale, with `under`/`over`
disclosing what falls outside instead of clamping it. `steps(n)` and
`thresholds(values)` quantize the ramp into bands the colorbar draws and
labels at their boundaries. `contourf` is `heatmap` under a map split at
`contour`'s levels.

Stops mix in OKLab, so the color halfway between two stops looks halfway —
no grey between blue and yellow. The 16-color tier picks by OKLab lightness
and hue, the two things sixteen colors can carry.

`scale::Palette` is the categorical scale `color_by` draws from. Okabe–Ito
(Wong 2011) is the default, with Paul Tol's `BRIGHT` and `MUTED` (nine
colors) beside it.

## Ticks

Ticks are the axis values a scale chooses to label, placed by the extended
Wilkinson algorithm (Talbot, Lin, Hanrahan 2010) — scored for simplicity,
coverage, density, and legibility. Ticks are computed, never supplied as
strings. They carry exact-decimal labels: a label parses back to its value,
labels share one fraction width and one SI prefix per axis (one power of ten
beyond the prefix table, `8.796e-100`), and they never show float artifacts.

A range no nice step can cover — equal bounds, a span past the exact
mantissa — falls back to its two bounds, formatted by the same formatter at
its budget. What the labels leave out, an axis prints once as a context note
(`Ticks::context`). Values agreeing in four or more leading digits read
relative to a round base, printed above the y labels or at the end of the x
title row (`+1.000G`, matplotlib's offset text). A time axis whose first
label omits its date or year prints that part (`Aug 1 2026` under hour
labels, `2026` under day or month labels — Bokeh's `context`). This is an
automatic layout rule, not an option: the y note is shed like the legend, the
x note with the title row, and a colorbar never offsets.

Maps to `scale::Ticks`. `scale::TickOptions` — a `Unit`, an `integer` flag,
and the `context` flag the axes set — is how an axis's scale options reach
the tick search, through `Ticks::linear_with`. `scale::NumberFormat` makes
the same label decisions once, for an arbitrary set of related values — one
fraction width, one SI prefix or power of ten, whole labels for whole-number
sets, gaps as `—` — the per-column formatter behind `table`, usable for any
readout. There is no second, cheaper formatter anywhere. The one value
written outside the set's resolution is the one that resolution would
misstate — rounded to zero, or to a single inexact digit — which keeps its
own, so a column of gigabytes never reads a mean of a thousand as `0`. See
[The axes are the product](principles/axes-are-the-product.md).

## Frame

A frame is where and how to render: width and height in cells, the charset,
the color mode, and the theme. A frame is run state, not plot state — the
same `Plot` renders into many frames. `Frame::detect()` is the convenience
that inspects the environment. `Plot::render` with an explicit frame inspects
nothing. `Frame::plain()` is the deterministic braille snapshot form.
`Frame::portable()` is the conservative Unicode form. Maps to `plot::Frame`
and `plot::ColorMode`. See
[The frame is run state](principles/frame-is-run-state.md).

## Mapping

A mapping is the resolved geometry of one render, as a queryable value: the
plot rectangle in cells, the resolved axis windows, and the cell ↔ data
mapping both ways. That both-ways mapping is the `invert` half of the scale
contract, reachable at plot level. A mapping is obtained purely from
`Plot::mapping(&frame)`. The ratatui stateful widget caches one per render.

Queries answer in the coordinate conventions marks use (band indices, unix
seconds). They name the plot panel as a `Panel` value rather than a bare
tuple. They expose a categorical axis's labels (`x_categories`), disclose
cell quantization (`x_span_at`/`y_span_at`), and format values the way the
axis formats its own labels (`format_x`/`format_y`). They answer the x-only
half of the invert (`column_at`) — what lets linked panes mirror one cursor
whatever their gutters.

A mapping is derived state, deliberately not serializable. This is the
physics interactive hosts build on. malevich never handles input — a host
maps its events to questions a `Mapping` can answer. Maps to `plot::Mapping`.

## Viewport

A viewport is an axis window pair for interactive viewing. Zoom and pan are
pure domain arithmetic over `x_domain`/`y_domain` — a scale option, never a
render mode, which is why zooming into millions of points re-aggregates (M4)
to the new window with no special machinery. `None` on an axis means
automatic. A window is seeded from `Mapping::viewport` ("the view I am
looking at") and transformed by value: `zoom` around an anchor (decade space
on log axes), `pan` by a fraction, `clamp` to an extent, `tail` for
follow-the-stream, and `reset` to automatic. The viewport is applied with
`Plot::viewport`. It is serializable — spec-shaped state a host may persist.
Maps to `plot::Viewport`.

## Widget

Widgets are the TUI adapters.

In Rust the feature is `ratatui`, and it depends only on `ratatui-core`.
`Plot::widget()` renders any plot into a `Buffer` as cells and styles. The
stateless `Widget` impl is fire-and-forget. The `StatefulWidget` impl threads
a `PlotState` — the interaction controller. It caches the render's `Mapping`
for hit-testing, applies its `Viewport` on the next draw, and interprets the
default mouse gestures from the backend-neutral `Mouse` vocabulary the host
feeds it: hover crosshair, wheel zoom at the cursor, left-drag pan, and
right-drag rubber-band zoom.

In JavaScript the same split lives in `malevich/ink`. `PlotWidget` paints a
`Raster` as Ink `Text` cells. `PlotState` is the same controller, overlays
and all. `usePlotInteraction` is an optional composition that enables mouse
tracking.

The cursor snaps to the data. For every point-backed line and points layer,
the readout lists the value of the datum nearest the cursor's x inside the
visible window — axis-formatted, its cell highlighted, a gap shown as `—`
rather than an interpolation. `snap(false)` returns to plain cursor
coordinates.

A linked pane mirrors another pane's cursor by data x
(`PlotState::hover_x` / `hoverX`): a vertical-only crosshair at its own
column — an x but no honest row — with snapping and the readout working as
they do under a real cursor.

The widget never reads the terminal. Event loops, mouse capture, and key
policy stay in the host, and the gestures are a preset over the public
physics — a host with different policy drives `Viewport` and `Mapping`
directly.

Interaction chrome (crosshair, snap highlights, selection band, readout)
draws into the buffer (or the raster) only. The plot value renders
byte-identically with or without it.

With the `pixel` feature, `widget().graphics(g)` renders the panel as a real
image. The buffer holds skip-reserved ground, the `PlotState` carries the
encoded block, and the host emits it after `terminal.draw` with
`Graphics::present` — one synchronized write, new placements created before
old ones are retired. `Graphics::retire` deletes a view's images on the way
out. Interaction chrome then becomes annotation marks drawn into the image,
and hit-testing is unchanged — the mapping answers in cells regardless of
what fills them.

The pixel render paces itself (~30 full encodes per second): within the
window, an unchanged-view render reuses the image already on screen, so hover
floods and tick redraws cost nearly nothing. A changed viewport or rectangle
always renders.

## Surface

A surface is the subpixel grid that marks draw on during rasterization,
before glyphs exist. The raster convention is origin top-left, y down; the
data-space flip happens in the scales. A charset codec maps each cell's
subpixel pattern to a glyph with independent foreground and background
colors. Text shares the grid and wins over pixels. Drawing is infallible:
out-of-surface clips, non-finite coordinates draw nothing, and control
characters are dropped at the cell grid, so no input string can smuggle
escape bytes into any encoder's output. Maps to `render::Surface`.

## Raster

A raster is the encoded cell grid of one render: glyphs and colors, chrome
included, as a plain value. Marks draw on a Surface in subpixels; a charset
codec maps each cell to a glyph; a raster is that snapshot. TUI hosts
(ratatui, Ink) paint it into their own buffer instead of decoding an ANSI
string. An ANSI round-trip loses cell identity: wide glyphs, independent
fg/bg, and the continuation cell. `Plot::render` is rasterize-then-encode.
`Plot::raster` stops after rasterize. The encoders are the second half —
`Raster::encode` (glyphs and SGR for a tty), `to_plain`, `to_html` (the card
a notebook draws), `to_svg` (the card an SVG host draws) — so every kind of
terminal shares one grid. Continuation cells (`columns == 0`) sit to the
right of a wide glyph; encoders skip them. Maps to `render::Raster`. The
membership test: a second host demanded cells, and no composition of the
public string renderer reproduces per-cell style.

## Card

A card is the cell grid encoded for a host that draws with markup rather than
escape codes: the HTML card (`Plot::to_html`, a `<pre>` of colored spans) and
the SVG card (`Plot::to_svg`, rectangles for block glyphs, text runs for
everything else). A card is a picture of the raster and nothing more — the
same grid a tty would print, with mark colors resolved to concrete RGB and
default-colored chrome taking the card's foreground, on the card background
the theme selects. Text is never rasterized: the host's font draws it, the
offload the string render makes to the terminal. Both cards need no feature;
the `evcxr` feature adds only the notebook protocol around them. A card that
drew something the terminal would not draw would be a figure, and a figure
is a different product.

With the `pixel` feature, `Plot::to_svg_pixels` encodes the hybrid render for
the same kind of host. Chrome is still this card. The plot panel is the
device-pixel raster the terminal would draw, as rectangles, not a second
drawing from the data and not an embedded image. `Plot::to_svg` stays the
cell card. A text-only plot, a zero cell size, or a frame with no panel
degrades to it. In-panel text marks use the pixel font, as they do in
`render_pixels`; axis text stays the host's font.

## Charset

A charset is a glyph tier used to encode the surface. Glyph tables are data,
not code. Maps to `render::Charset`: `Ascii`, `HalfBlocks`, `Quadrants`,
`Sextants` (Unicode 13), `Octants` (Unicode 16), and `Braille`. Each tier
also owns the shade ramp colorless patch output draws with — `░▒▓█`, or
`.:#@` on `Ascii` — so a heatmap, a class region, a colorbar, and their
legend swatches never carry a glyph the tier cannot show. `Frame::detect`
sniffs the environment, never probes. Dense tiers are explicit, because a
terminal name cannot establish the configured font's coverage. See
[Degradation is the contract](principles/degradation-is-the-contract.md).

## Canvas

A canvas is the drawing-target contract marks rasterize through, generic over
fidelity: the cell `Surface` fills with eighth-block ramps and glyph
textures, the pixel canvas with exact rectangles and real pixels — same mark
code, monomorphized per target. The canvas is crate-private. Maps to
`render::Canvas`.

## Graphics

Graphics is how to draw the plot panel as a real image (feature `pixel`):
which protocol, at what cell size in device pixels. It is render state like
`Frame`, and a plain value like everything else. `None` means the caller
falls back to cells. Output stays hybrid: chrome as text, only the plot
rectangle as pixels. `Plot::to_svg_pixels` is that hybrid for an SVG host.
It reads cell size and stroke. The protocol is terminal transport and does
not change the SVG. `economical()` is the slow-link trade: halve a Retina
density, keep the ink weight, a quarter of the bytes. Sixel, with no
placement scaling, stays native. Maps to `pixel::Graphics`. See
[the pixels guide](pixels.md).

## Capabilities

Capabilities are what the terminal can do, as a plain queryable value: the
protocols it accepts, its cell size in device pixels, and how the answer was
obtained (`Source::Probed`, `Source::Sniffed`, or `Source::Declared`).
Sniffing reads environment variables — free, and wrong only by omission.
Probing asks the terminal itself over one raw-mode round trip — ground
truth, and only where writing escapes is safe. An unanswered probe is not
evidence; it degrades to the sniff answer. A host that already knows its
terminal declares the value through `Capabilities::new` and skips detection.
Maps to `pixel::Capabilities` and `pixel::Source`.

## Protocol

A protocol is a terminal image protocol the panel can be emitted in: `Sixel`
(DEC 1987, the most widely spoken), `Kitty` (raw RGBA with alpha, the most
capable), and `ITerm2` (an inline PNG). Encoders are hand-rolled and
dependency-free. Maps to `pixel::Protocol`.

## Theme

A theme is colors and styles as a value you pass, never a global. Today it
is the layer palette, with dark and light variants and `COLORFGBG` detection.
Maps to `Theme` (a field of `Frame`). It is deliberately distinct from the
categorical `scale::Palette`, which lives in the spec. Layer colors are
presentation, and a frame adapts them to its background. Category → color
assignments are closer to an encoding — they travel with a serialized spec
so its legend keeps meaning wherever it renders. Two palettes, two homes,
one recorded reason.

## Grid

A grid is small multiples: independently rendered plots pasted side by side
(escape-aware padding), cells filled left to right. Axis sharing is a
composition — fix domains with `Plot::x_domain`/`y_domain` — never a hidden
mode. Maps to `plot::Grid` (re-exported at the root).

## Preset

A preset is a plain function composing the grammar into a named chart type:
`line()`, `hist()`, `scatter()`, `sparkline()`, `table()`, `describe()` (with
`describe_with` adding an inline histogram column), and the rest. It is the
short spelling of that composition, and a test checks the two print the same
bytes. Presets are the front door. The grammar can wait until you need it.
`_with` means "configured with an options value". A `try_` prefix identifies
the checked twin of an otherwise identical convenience. Maps to functions and
option types re-exported at the crate root. See
[Presets are packaging](principles/presets-are-packaging.md).

## Stream

A stream is the live data machinery, kept at the edge of the crate.
`stream::Ring` is a sliding window shared across threads — the one lock in
the library — or, from `Ring::growing`, a window that keeps every value since
the start. `stream::Rate` turns counters into deltas. `stream::Live` is an
in-place repaint: cursor up, erase down, one buffered write bracketed as a
synchronized-output frame — flicker-free, scrollback-safe, never owning the
screen. `Live::detect` repaints only on a terminal and appends plain frames
to a pipe or file, so no escape byte lands where it is not safe. The core
stays pure. Time enters only at the rims — this module, and the ratatui
widget's pixel pacing (below), which uses a monotonic clock to skip redundant
re-encodes. Every render that does run remains a pure function of its inputs.

## Gap

A gap is missing data, encoded as `NaN` in a series and rendered as a visible
break — never interpolated across, never dropped silently. This is the
de-facto convention of the terminal plotting field.

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
