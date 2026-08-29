# Changelog

Notable changes, written for humans. Since 1.0, breaking changes mean a major
release; the pre-1.0 entries below recorded breakage freely, without apology.

## Unreleased

- The widget becomes interactive — without malevich handling input. The
  core gains the physics: `Plot::mapping(&frame)` exposes the resolved
  geometry of a render as a queryable value (plot rectangle, cell ↔ data
  both ways, per-cell data spans, values formatted the way the axes format
  their labels); `Viewport` is zoom/pan as pure domain arithmetic over
  `x_domain`/`y_domain` (decade space on log axes, `tail` for streams),
  seeded from `mapping.viewport()`; `stat::nearest` snaps a cursor to the
  datum that exists. On top, the ratatui adapter grows a `StatefulWidget`:
  `PlotState` caches the render's mapping for hit-testing, applies its
  viewport on the next draw, and interprets the default gestures — hover
  crosshair with an axis-formatted readout, wheel zoom anchored under the
  cursor, left-drag pan, right-drag rubber-band zoom — from a
  backend-neutral `Mouse` vocabulary the host feeds it (a six-line match
  from crossterm, shown in the docs). The cursor snaps to the data: for
  every point-backed line and points layer the readout lists the datum
  nearest the cursor's x — `label: value`, axis-formatted, its cell
  highlighted, gaps as `—` and never an interpolation — instead of the
  cursor's own coordinates; `snap(false)` returns plain coordinates.
  Overlays draw into the buffer only:
  the plot value and the stateless widget render byte-identically as
  before. Because a zoom is just a domain window, M4 re-aggregates to the
  visible window every frame — `cargo run --release --example zoom
  --features ratatui` pans and zooms through millions of points with the
  drawn line pixel-identical to plotting every point — measured: the new
  `widget` bench records a two-pane dashboard frame at 1.3 ms, a zoomed
  ten-million-point frame at 18.6 ms, and a hovered one (the snap scan
  included) at 25 ms on the BENCHMARKS.md baseline. `docs/interaction.md`
  is the guide: the three-layer model, the gesture table, and the patterns
  that need no library support — linked panes (a view is a value; sharing
  it is assignment), selection → statistics (a rubber-band window
  summarized with the ordinary stat vocabulary), follow-the-stream.
  `fred` wears all of it on its series view: the full gesture set, a
  year-over-year context strip linked to the main chart by mirroring the
  x window, and a footer that describes the visible window when zoomed;
  `sysmon` pins its streaming axes with `tail`, holding a stable
  two-minute window while the rings fill. `pan_left`/`pan_right` join the
  keyboard sugar (h/l in fred, arrow keys in the zoom example), and
  gestures stacked between renders compound instead of re-reading a stale
  mapping — drag pans included, which an event-drained burst previously
  collapsed to a single step. And with the `pixel` feature, the interactive widget draws
  real images: `widget().graphics(g)` reserves the area in the buffer
  (skip cells, fresh ground on layout change) and stores the hybrid
  chrome-plus-image block in the `PlotState`; `present_pixels` emits the
  pending blocks after `terminal.draw` in one synchronized write with
  atomic kitty replacement, `clear_pixels`/`invalidate_pixels` handle
  view switches. Interaction chrome upgrades to annotation marks drawn
  into the image — anti-aliased crosshair rules, snap markers, in-panel
  readout — with automatic axes pinned to the last frame so hovering
  never jitters them (viewport-fixed axes are never pinned: the window a
  gesture just set always renders). The pixel render paces itself: within a
  ~33 ms window an unchanged-view render reuses the image already on
  screen, so hover floods and tick redraws cost nearly nothing and even
  a one-event-per-frame host loop stops falling behind (a changed
  viewport always renders — a zoomed window never shows stale). And
  repaints never flicker and never rely on replacement semantics: each
  panel's kitty image data travels transmit-only (`a=t`) under a stable
  per-panel image id, and the presenter then creates a fresh placement
  under an alternating placement id before retiring the one on screen —
  create-before-delete in one synchronized write, so there is no gap on
  any terminal, including those whose same-placement replacement
  misbehaves; a panel whose content already matches the screen is not
  transmitted at all;
  `clear_pixels` retires images by id, never touching other
  applications'. fred
  renders its series view this way wherever the terminal speaks sixel,
  kitty, or iTerm2 — at native device-pixel density, so the image is
  crisp on Retina cells (`p` toggles pixel drawing live, `--fast` halves
  the density for slow links, `--cells` forces glyphs from the start) — and both fred and the zoom example drain
  their event queues before redrawing, collapsing input bursts into one
  repaint.

- The hand-rolled deflate truncates every LZ77 match at 32 KiB
  window-aligned output boundaries. Bounded blocks alone never protected a
  streaming inflater — block output positions drift, so matches still
  straddled the drain boundaries where Zig ≤0.15's flate (Ghostty ≤1.3.1)
  aborts a fixed-Huffman transmission mid-match; a real chart stream
  carried four hundred straddling matches. (Measured against Zig 0.15.2's
  exact buffered decode path, those particular streams happened to
  survive — the truncation is defense in depth against the documented
  abort, not the explanation of any observed freeze.) Costs one shortened
  match per 32 KiB (+0.2% on a measured chart stream); an emission-site
  assertion and a five-window regression test pin the invariant.

- The design argument is public. `docs/` now carries the vision and its five
  rules, seven principle files — each arguing one constraint and ending in a
  "Spelled today" section that may rot while the argument must not — and
  guides for terminals, pixels, notebooks, performance, and serde.
  `TERMINOLOGY.md` and `SERDE.md` moved there (stubs remain at the old
  paths). The gallery now reads as a ladder of sections, and principle files
  demonstrate their claims with generated witness charts, spliced and
  CI-verified like every chart in the docs.

## 1.19.0 (White on White) — 2026-08-27

The ink release. The pixel canvas learns coverage, and everything drawn on
it turns from stamped rectangles into graded light: anti-aliased strokes,
glow, translucent washes, dashes that flow through joints, gradient
trajectories, density scatters, bilinear heatmaps. Every effect degrades
honestly on glyph targets, the wire format is byte-stable, and the whole
gallery wears the new ink.

- The pixel canvas learned coverage: pixels are straight RGBA with alpha
  as coverage, and everything drawn on it is anti-aliased — strokes with
  round caps and sub-pixel endpoints, discs, rings. Fringes composite
  over any terminal background; kitty and iTerm2 transmit the raster
  verbatim, sixel thresholds to solid ink.

- New ink, mark by mark: `Line::glow` (a soft halo fading from the
  stroke), `Line::dash` and `Rule::dash` (dashes and dots whose phase
  flows through polyline joints), `Line::grade` (gradient strokes
  through a colormap), `Area::opacity` (translucent fills and bands),
  `Points::opacity` + `Points::density` (accumulated ink — overplotting
  reads as brightness), `Cells::smooth` (bilinear heatmaps), and text
  annotations that ride their exact data anchor on pixel targets.

- Two long-standing fill bugs died on the way: area fills stopped one
  gutter short of the plot's far edge, and isolated line points (the
  first point of every NaN-jointed contour segment) drew with the
  marker pen instead of the stroke's weight.

- The showcase tour opens an effects corner: glow over a wash with
  dashed annotations, a trajectory graded by step, fifteen thousand
  points as accumulated ink, and a smooth loss landscape under its
  contours.

## 1.18.6 — 2026-08-27

- `Graphics::stroke` overrides the cell-derived line width in device
  pixels. Hosts that transmit reduced-density rasters into a scaled
  placement rectangle (`c=`/`r=`, since 1.18.4) can keep the ink weight
  they had at native density instead of inheriting a hairline from the
  smaller cell.

## 1.18.5 — 2026-08-27

- Pixel blocks anchor every text row, including at column 0. Flush-left
  blocks used to stay escape-free, which is only safe in cooked mode:
  raw-mode LF does not return the carriage, so a TUI printing a
  column-0 block watched its chrome staircase across the screen.

## 1.18.4 — 2026-08-27

- Kitty images now carry their placement rectangle (`c=`/`r=`): the
  image is pinned to the panel's cells and the terminal scales it as
  needed. This makes transmitted resolution a host-side knob — send a
  standard-density raster into a Retina-sized panel for a fraction of
  the decode and upload cost — and keeps placement correct even when
  cell-size detection was off. iTerm2 already behaved this way; sixel
  has no placement scaling and is unchanged.

## 1.18.3 — 2026-08-27

- The pixel encode path is ~4.5× faster (a 2744×1230 kitty panel drops
  from ~81 ms to ~18 ms on an M-series laptop). Deflate compares matches
  a word at a time and indexes only the fringes of long matches (zlib's
  `max_insert_length` trick — hashing every byte of a flat run dominated
  the whole compressor), and the kitty encoder crops the canvas straight
  into RGBA, skipping the intermediate `Image` buffer. Compressed size
  is unchanged within a fraction of a percent.

## 1.18.2 — 2026-08-27

- The deflate stream now splits into bounded blocks (16 KiB of input each)
  instead of one stream-length fixed-Huffman block. The old shape was
  valid DEFLATE but crashed terminals built on Zig ≤ 0.15's inflater —
  Ghostty 1.3 aborts the moment a compressed kitty image arrives, because
  a fixed-Huffman block that decodes past the 32 KiB drain window hits
  unreachable code (fixed on Zig master). Blocks share one LZ77 window, so
  the split costs ~10 bits per block: a 700 KB panel grew by 54 bytes.

## 1.18.1 — 2026-08-27

- Pixel transport is now compressed: a dependency-free zlib/DEFLATE
  compressor (LZ77 over a 32 KiB window into fixed-Huffman blocks) rides
  under both image encoders — kitty transmits `o=z` deflated RGBA and the
  iTerm2 PNG carries a real IDAT instead of stored blocks. A Retina-sized
  panel drops from ~22 MB to ~175 KB per repaint, which turns multi-second
  redraws (the terminal's escape parser pays for every byte) into
  imperceptible ones. Sixel was already compact and is unchanged.

- Hybrid pixel blocks (`render_pixels`, `render_pixels_at`) now own their
  full rectangle: every text row spans the frame's width instead of
  trimming trailing spaces, so a block reprinted in place fully replaces
  the previous one — a shorter title no longer leaves the old title's tail
  visible, and in-place hosts (TUIs repainting a panel) need no manual
  blanking. Ordinary cell renders keep trimming trailing spaces.

## 1.18.0 (The Knife Grinder) — 2026-08-26

The machine-learning release. Band scales on both axes, a logarithmic
colormap, two new cells channels, and bucket-exact matrix reduction turn the
charts ML actually reads — confusion matrices, attention maps, decision
boundaries, images, loss landscapes — into grammar compositions; `roc`,
`auc`, and `ewma` fill out the statistical set, and a nine-chart gallery
wave plus a showcase ML corner prove the vocabulary was sufficient.
Everything is additive, and a long-standing sub-decade log-axis bug died on
the way through.

- The showcase tour grows an ML corner: a confusion matrix on band axes, a
  log-colormap attention head with a colorbar, a learned filter as rgb cells,
  1-NN decision regions with the training scatter, a momentum trajectory over
  a bucket-reduced loss landscape, seed-variance bands with EWMA smoothing,
  and a spectrogram — every panel upgrading to real pixels beside its cells
  under `--features pixel`, like the rest of the tour.
- The loss example's training log now credits topos by its current name —
  the library was renamed from poorgrad — and the data file moved to
  `examples/data/topos_loss.csv` accordingly.
- Fixed: a log axis over a range narrower than one decade no longer loses its
  data. The linear-fallback ticks of a sub-decade log range can include zero;
  the domain grew to that tick, zero has no logarithmic position, and the
  whole scale collapsed. Log domains now refuse to grow to a non-positive
  bound, and a tick without a position on its scale is dropped instead of
  drawn at a fabricated column.
- `stat::ewma`: debiased exponentially weighted smoothing — TensorBoard's
  scalar smoothing, early outputs unbiased instead of dragged toward zero,
  gaps passing through without disturbing the state. A scan over the ordered
  series, documented as a batch transform rather than pretending a merge law.
  Gallery gains `seeds`: five runs pooled into per-step quantile bands via
  the existing reducers, with the smoothed median on top.
- `stat::roc` and `stat::auc`: the classifier threshold sweep (standard step
  construction, ties grouped, one-class data returns empty rather than
  invented rates) and the trapezoid area under a polyline, gaps contributing
  no area. Batch transforms in the `ecdf` family — order statistics, not
  mergeable accumulators — with hand-computed fixtures. Gallery gains `roc`.
- Gallery: `spectrogram` — time × frequency power as dense Cells with a log
  frequency axis and a log colormap; the exponential chirp is a straight
  ridge. The energy is synthesized analytically: no FFT enters the crate.
- Gallery: `ridgeline` — distributions over training epochs as lifted KDE
  rows, painter's algorithm back to front, no camera and no new machinery:
  the TensorBoard histogram view and the honest terminal answer to a 3D
  surface.
- Gallery: `calibration` — a reliability diagram from `stat::binned` with a
  `Mean` reducer over 0/1 outcomes; the overconfident model's curve sags
  under the diagonal. No new API: the reducer vocabulary was sufficient.
- Gallery: `landscape` — a loss landscape with a momentum trajectory,
  composed entirely from existing marks (dense Cells on a log ramp, Line and
  glyph Points on top): the gradient-descent chart, no new machinery.
- Cells grids denser than the raster now reduce honestly instead of sampling:
  every screen bucket owns the cells whose centers fall inside it (adjacent
  buckets partition the centers, proven by a property test) and shows a
  reduction over all of them — `Reducer::Mean` by default, `Cells::reduce`
  to choose; `Max` keeps sparse spikes visible that sampling silently
  dropped. Rgb grids box-filter per channel and class grids reduce to the
  modal class with deterministic ties. 4.19 million cells reduce in ~44 ms
  on the recorded baseline (BENCHMARKS.md). Buckets owning no cell center
  keep the old center-sampling, so ordinary small grids render as before.
  Gallery gains `attention-full`: one million attention weights rendered
  twice, the mean pane dissolving the long-range spikes the max pane keeps.
- `Cells::classes` draws categorical regions: a grid of class labels colored
  through the plot's categorical `Palette` with a categorical legend — the
  decision-boundary chart. Labels intern in first-appearance order exactly
  like `color_by`; in plain output each class keeps a stable shade-ramp glyph
  and the legend swatches carry the same glyphs, so regions stay separable
  with no color at all. Gallery gains `boundary`, 5-NN decision regions with
  the training scatter on top.
- `Cells::rgb` draws a grid of direct colors — an image. Raw row-major pixel
  buffers only (decoding files stays the host's job), no colormap and no
  colorbar, honest quantization down the color ladder, and in plain output
  each pixel falls back to its luma on the shade ramp so images survive a
  pipe. With the `pixel` feature the grid blits at device resolution. The
  serde encoding is additive; value grids encode exactly as before. Gallery
  gains `filters`, an AlexNet-style Gabor bank rendered as `Cells::rgb`
  small multiples.
- `Colormap::log()` makes any ramp logarithmic: equal color steps for equal
  factors, so attention weights, gradient magnitudes, and spectral power that
  span decades stay distinguishable instead of collapsing into the low end of
  a linear ramp. Values at or below zero have no logarithmic position and
  render as gaps — the same rule log axes follow — and the colorbar places
  decade ticks logarithmically. Logarithmic and `centered_at` are mutually
  exclusive (validation catches the combination, including deserialized
  specs); the serde encoding stays byte-identical for existing maps. The
  gallery gains `attention` — token-labeled bands on both axes and a log
  MAGMA ramp.
- `Scale::Bands` now works on the y axis: continuous marks position y against
  band indices exactly as they do on x, and a `Cells` matrix maps row k onto
  band k, top-down — band 0 is the top band, so labeled matrices read in matrix
  order. Cells grids must match their band axes cell-for-band (extents do not
  apply there), Bars still require a numeric y, and `Plot::y_scale` no longer
  panics on `Bands`. Confusion matrices and attention maps are now three-line
  grammar compositions; the gallery gains `confusion` as the proof.

## 1.17.0 (The Carpenter) — 2026-08-24

A code-quality release: invalid retained data now reaches one checked boundary,
the hottest statistical and rendering paths carry less incidental structure, and
the cell, pixel, and CLI front ends prepare a chart once. The public API remains
compatible; new checked twins make the remaining convenience assertions avoidable.

- `Cells::try_matrix` and `Cells::try_extents` expose typed failures for invalid
  matrix geometry and extents. Configured histogram, heatmap, and 2D-histogram
  presets use those checked paths end to end, while their infallible conveniences
  still degrade safely for trusted literals.
- Finite-range arithmetic is overflow-safe, calendar tick generation has an
  explicit work budget, and retained mark values share one validity policy. These
  close hangs and pathological allocations around enormous finite domains without
  changing ordinary axes or plots.
- M4 downsampling preserves gaps as path topology, including gaps between buckets
  and across merged chunks. Its streaming state, tests, and merge contract now say
  exactly when ordered partial reductions are equivalent to one pass. The ordinary
  affine map is selected once and each bucket updates its current run directly;
  ten million points now render in 31.9 ms on the recorded machine, 5.9% below 1.16.
- Reducers compile once into execution state. Sum, count, mean, min, and max stream
  in constant space; rolling sum and mean use their specialized sliding state;
  percentile-like reducers retain only the samples they require. Aggregation keys
  are interned in stable first-seen order, and KDE reuses moments and sample storage.
- Categorical channels are interned once per mark and rendered directly. They no
  longer expand into one masked layer per category, while stable legend order,
  palette assignment, marker cycling, line transitions, and category-aware M4
  retain the prior output contract.
- Cell and device-pixel rendering share one prepared plot. The CLI likewise shares
  one typed recipe and one set of parsed series channels between rendering and
  `--emit-code`, removing duplicate resolution and keeping both outputs in lockstep.
- Invalid civil timestamps and out-of-range numeric column selectors are rejected
  with actionable CLI errors. CI now builds and tests both demo crates, terminal
  dependency versions are aligned, and the public failure and statistical execution
  models are documented explicitly.

## 1.16.0 (Red Square) — 2026-08-15

Color speaks data. One categorical channel, a curated color vocabulary, a
least-squares stat, and one shared reducer close the gaps between malevich
and the charts science asks for first — and the CLI grows a bridge out of
the shell. Everything is additive, and the re-measured render baseline came
out slightly faster than 1.15.

- One vocabulary for every aggregation: `stat::Reducer` — `Count`, `Sum`,
  `Mean`, `Median`, `Min`, `Max`, `Percentile(q)` (type-7, exactly the box
  plot's estimator) — is now what `Agg::reduce` and `Window::reduce` take
  (their named methods are sugar over it), what the new `stat::binned` uses to
  reduce a paired series per histogram bin, and what `stat::quantiles`
  evaluates in batch over one sort. Rolling p95s, per-group percentiles,
  binned medians, and Q–Q plots (see the gallery) all fall out with no new
  API shapes.
- The line of best fit: `stat::Fit` is streaming ordinary least squares —
  bivariate Welford accumulation, mergeable like every other aggregator, with
  slope, intercept, R², prediction, and the standard error of the mean
  response. The `trend` preset draws a scatter with its fitted line;
  `trend_with` adds a confidence band around the mean response through the
  existing band mark, at a caller-chosen standard-error multiplier. Measured:
  one million pairs fit in ~5 ms single-threaded (BENCHMARKS.md).
- Interval polish: `error_bars_asymmetric(x, y, minus, plus)` covers two-sided
  deviations, and `ecdf_with` grows the Dvoretzky–Kiefer–Wolfowitz confidence
  band as a checked option — both compositions of existing marks, both proven
  equal to their grammar expansions.
- Color speaks data: `color_by(categories)` on `Line`, `Points`, `Bars`, and
  `Range` colors a layer by a categorical series. Distinct categories (first
  appearance first) take colors from the new `scale::Palette` — Okabe–Ito by
  default, colorblind-safe, replaceable with `Plot::palette` — and name
  themselves in the legend. In colorless output the default point markers cycle
  shapes per category, so groups separate in a pipe as well as a terminal. The
  channel is proven bit-identical to its masked-layer expansion; grouped
  scatter, volcano, Manhattan, and candlestick compositions join the gallery.
- `PointStyle` gains portable `Asterisk` (`*`) and `Circle` (`o`) markers, in
  cells and as geometric pixel shapes; the ASCII legend swatch for `Dot` is now
  `..`, freeing `**` for the asterisk.
- `Colormap` grows a curated named set — sequential `VIRIDIS` (the default,
  now named), `MAGMA`, `CIVIDIS`, `GREYS` and diverging `RED_BLUE`,
  `PURPLE_ORANGE` — selected to stay distinguishable after the 256- and
  16-color quantizers, with `Colormap::named` resolving the canonical names
  for CLIs and configuration.
- `Colormap::centered_at(mid)` anchors a map to a data midpoint: signed and
  centered data (correlations, log fold changes) renders with the neutral
  color at the midpoint and the value range spanned symmetrically, and the
  colorbar labels the symmetric span. `heatmap_with` joins the checked `_with`
  presets so heatmaps take a colormap without abandoning the one-call default;
  the correlation example now demonstrates the honest encoding.
- The control-character-dropping behavior of the cell grid is now a stated,
  regression-tested contract: hostile escape bytes in titles, labels, categories,
  or annotations can never reach ANSI, plain, or HTML output.

## 1.15.0 (The Aviator) — 2026-08-10

A hardening release: the paths that used to trust their inputs now refuse
impossible geometry instead of reaching for the allocator, validation rejects
combinations that never meant anything, and Windows joins CI. What opens up
alongside it is reach — the notebook helpers, the charset policy, and the
capability query stop being malevich's private business and become things a host
can drive.

- The `evcxr` feature now exposes a public `evcxr` module: `mime_bundle` writes
  Evcxr's stdout protocol and `card_colors` returns the background and foreground
  a plot card paints itself with. Both were internal, which left a crate rendering
  its own types beside a chart with nothing to do but hardcode the colors and
  reimplement the framing, then drift on the next theme change. `Plot` now draws
  through the same two functions, so the exported values cannot disagree with what
  it paints.
- Rendering and statistics now reject overflowing or over-budget geometry through
  typed fallible paths; the infallible conveniences degrade without attempting giant
  allocations. The `kaz` CLI applies corresponding bounds to user-controlled sizes.
- Automatic UTF-8 output now conservatively uses quadrants. `Frame::portable` and
  `MALEVICH_CHARSET` make deterministic and explicit charset policy available to
  hosts; braille, sextants, and octants remain opt-in dense tiers.
- Mark/scale validation now rejects meaningless combinations and `Cells` correctly
  inverts logarithmic axes. `Grid::validate`/`try_render` and exhaustive tiny-frame
  layout keep every rendered row and column inside the requested frame.
- `Document` is a validated, versioned serde envelope with committed v1 and legacy
  fixtures. Additive fields default safely; malformed specs remain representable only
  until the strict validation boundary.
- Pixel capability detection is destination-aware through
  `Capabilities::detect_for`; explicit capability-driven rendering stays pure. Probe
  replies are bounded and their parser is covered by deterministic arbitrary streams.
- Runtime-owned colormaps and checked option structs now configure histogram, 2D
  histogram, KDE, violin, and contour presets without abandoning their short defaults.
- Heatmaps render two vertical colors per terminal cell through independent
  foreground/background half-blocks. ANSI transitions, HTML spans, and ratatui styles
  preserve both channels; plain output keeps an averaged shade-ramp fallback.
- `PointStyle::{Dot, Plus, Cross}` gives point layers and legends portable shapes that
  remain distinguishable without color, with corresponding geometric pixel markers.
- Windows joins Linux and macOS CI; boundary sweeps cover extreme ticks, tiny rasters,
  image encoders, grids, and terminal replies.
- `BENCHMARKS.md` records the dated, reproducible performance baseline and CI enforces
  structural allocation ceilings. Implicit coordinates no longer allocate, profiled
  layout metadata is reused, and the plain encoder keeps the richer cell model at the
  prior end-to-end timing.

## 1.14.3 — 2026-08-05

- A compact Suprematist composition joins the examples and README, rendering the
  same layered `Area`, `Bars`, `Range`, `Line`, and `Points` marks side by side
  in octants and pixels.

## 1.14.2 — 2026-08-05

- With the `pixel` feature also enabled, `Plot::evcxr_display`'s terminal
  representation becomes a real sixel/kitty/iTerm2 image in a graphics-capable
  terminal (the `evcxr` REPL), and stays cells everywhere else — so a plot in a
  pixel-capable terminal renders as an image, not braille.
- Cell-size detection falls back to `/dev/tty` when stdout is piped (as under
  evcxr, or a mid-pipeline `kaz`), so pixel strokes are weighted for the
  terminal's real cell size instead of defaulting to a hairline.

## 1.14.1 — 2026-08-05

- `Plot::evcxr_display` now emits a `text/plain` cell plot alongside the HTML
  card, so the plot shows in frontends that cannot render HTML — notably the
  terminal `evcxr` REPL, where it previously appeared blank. Jupyter still renders
  the card; each frontend picks the richest representation it supports.

## 1.14.0 (An Englishman in Moscow) — 2026-08-04

Rich display in Rust notebooks, as the same complete cell grid malevich already
renders everywhere else. Additive, dependency-free, and behind an opt-in feature;
terminal and default-feature output are unchanged.

- New `evcxr` feature: a `Plot` ending an Evcxr Jupyter cell now renders as a
  self-contained `text/html` terminal card through the conventional
  `evcxr_display` method. The default is a notebook-sized 100×26 braille frame;
  `Plot::to_html(&frame)` is the pure deterministic path for explicit size,
  charset, and light/dark theme control.
- The cell surface has an HTML encoder parallel to its ANSI encoder: concrete RGB
  colors collapse into `<span>` runs, default-colored chrome inherits the card
  foreground, row whitespace trims identically, and every glyph escapes HTML
  content. Inline card styling contains wide plots with horizontal scrolling and
  keeps braille rows tightly connected. No new dependency and no duplicated plot
  rendering logic.
- New inspectable example:
  `cargo run --example evcxr --features evcxr > plot.html`.

## 1.13.0 (The Knife Grinder) — 2026-08-04

Enablers for the `kaz` CLI (crate `malevich-cli`, released alongside — a
stdin-pipe plotter built entirely on the public API). Additive and
cell-output-neutral.

- `Frame::detect_for(&impl IsTerminal)`: the full `detect()` ladder, but with
  the color gate keyed to the destination the caller actually writes to rather
  than always stdout. A tool that plots to stderr while data flows on stdout
  detects against stderr — otherwise a piped stdout would strip color from a
  plot going to a live terminal. `detect()` is now
  `detect_for(&std::io::stdout())`; `NO_COLOR` / `CLICOLOR_FORCE` / `TERM=dumb`
  keep their precedence, and size still comes from whichever standard stream is
  a terminal.

## 1.12.0 (Cow and Violin) — 2026-08-03

The pixel release. Malevich painted a cow onto a cubist composition to collide
two systems of representation in one picture; this release does the same to the
terminal — text chrome and a real image, woven into one deterministic string.
Cell output remains the product; pixels are the new top rung of the resolution
ladder, behind the `pixel` feature, and everything below it is untouched.

- Strokes scale with cell density: line width is `round(cell_height / 16)`
  device pixels (minimum 1) and point markers are one step heavier, derived the
  same way the in-panel font scale already is. Classic 8×16 cells keep the
  exact 1-pixel ink they always had; on retina/high-DPI terminals — where a
  cell spans ~20×44 device pixels and a 1-pixel line was a hairline — lines
  weigh what they do in cell output and scatter dots are visible again.
- `Plot::render_pixels_at(frame, graphics, column)`: pixel output anchored at
  a cell column — every text row and the image cursor walk start with an
  absolute-column jump (CHA; rows stay relative, so scrollback is safe),
  letting hosts paste a pixel plot beside other content. The showcase uses it:
  with the `pixel` feature in a capable terminal, every chart in
  `cargo run --example showcase --features pixel` renders as a side-by-side
  comparison — cells on the left, the same plot as a real image on the right.
- `pixel::Capabilities`: terminal capabilities as a plain queryable value —
  the protocols the terminal accepts (best first), its cell size in device
  pixels, and whether the answer was `Probed` or `Sniffed`.
  `Capabilities::detect()` now actively probes the terminal where that is
  safe (a real tty, no tmux/screen, `TERM` not dumb): one raw-mode
  `/dev/tty` round trip carrying the kitty graphics query, XTVERSION,
  XTSMGRAPHICS, and `CSI 16 t`, with DA1 as the ordering barrier — ground
  truth that, unlike `TERM_PROGRAM` sniffing, survives ssh. The probe runs
  at most once per process (~100 ms on answering terminals, 300 ms budget
  otherwise), an unanswered probe degrades to the sniff answer, and
  `Graphics::detect()` is now sugar for `Capabilities::detect().best()` —
  so `render_best` and the examples pick up probing for free. Try
  `cargo run --example pixels --features pixel -- --capabilities`.
- `Plot::render_best(&frame)`: renders at the best graphics tier the terminal
  offers — the plot panel becomes a real image when the `pixel` feature is on
  and a protocol is detected, and is exactly `render(&frame)` everywhere else
  (pipes, unknown terminals, tmux, or without the feature). The gallery
  examples now use it, so `cargo run --example sine --features pixel` (or any
  other example) upgrades to pixels in a capable terminal while the
  deterministic gallery output stays byte-identical. `Display` is unchanged:
  `println!("{plot}")` stays cells-only.
- Pixel graphics (new feature `pixel`): `Plot::render_pixels` renders the plot
  panel as a real image — sixel, kitty graphics, or iTerm2 inline PNG — while
  title, axes, tick labels, and legend stay text cells. Marks draw at
  device-pixel resolution through the same generic pipeline (`render::Canvas`,
  new): M4 buckets per pixel column, heatmap cells sample per pixel, bars fill
  exact rectangles, box-plot medians read as cleared gaps, and in-panel `Text`
  marks blit a baked public-domain 8×8 font. Undrawn panel area is transparent
  (sixel `P2=1`, kitty alpha, PNG alpha) so the terminal background shows
  through; output remains a deterministic `String` woven with DECSC/DECRC
  relative cursor moves. `pixel::Graphics::detect()` sniffs the terminal's best
  protocol (kitty/ghostty → kitty; iTerm2/WezTerm → iTerm2; foot, Konsole ≥
  22.04, Windows Terminal → sixel; tmux, pipes, unknown → `None`) and reads the
  cell size from `TIOCGWINSZ`. All three encoders are hand-rolled — including
  the stored-deflate PNG with its checksums — adding zero required
  dependencies (`rustix`, already in-tree, joins as an optional dep for the
  cell-size ioctl). Try `cargo run --example pixels --features pixel`.

- Second demo app: `sysmon`, a live system monitor (`cargo run -p sysmon`) — a
  sampler thread streams CPU, memory, and network readings through
  `stream::Ring` sliding windows (network counters via `stream::Rate`) into a
  dashboard of pinned-axis area charts, an SI-prefixed bytes/s network chart, and
  a per-core utilization heatmap with colorbar. Demos now live in per-app crates
  (`demos/fred`, `demos/sysmon`).
- New demo app (`demos/`, a separate unpublished workspace member): `fred`, a Federal
  Reserve economic-data browser in ratatui with five views — small-multiples overview,
  a series view (line/step/corners styles, calendar axis, log and year-over-year
  transforms, NBER recession ribbon, a 2% target rule on inflation), change histograms
  with decade box plots, a month-by-year seasonality heatmap with colorbar, and the
  Phillips-curve scatter plus the 10y-minus-fed-funds spread. Pure data and view
  layers (unit-tested) under a thin TUI shell; live refresh from FRED; heavier deps
  stay out of the malevich crate and CI. Run: `cargo run -p malevich-demos --bin fred`.
- New gallery entry `charsets`: the same curve rendered across the whole charset
  ladder — octants, sextants, quadrants, half blocks, braille, ASCII — so the
  subpixel-density trade-off is finally visible in the docs, not just described.
- Grid (side-by-side plots) now leaves a blank row between stacked rows, matching the
  blank column already between neighbors. A lower row's title no longer butts against
  the row above's axis labels — multi-row small multiples read as distinct plots.

## 1.11.1 — 2026-08-02

- Declared MSRV: Rust 1.88 (`rust-version` in `Cargo.toml`), verified by a pinned
  CI job — the crate's let-chains and edition 2024 set the floor.
- Stability guardrails in CI now that the crate is 1.x: `cargo-semver-checks` compares
  the public API against the last published release (a break requires a major bump),
  and `cargo-deny` (see `deny.toml`) scans dependency advisories, licenses, and sources.

## 1.11.0 (White on White) — 2026-08-02

Crossing into 1.x. The version lineage is kept (major bumped, minor/patch as they
were) rather than reset — this is the same crate, matured, not a rewrite. The API is
what the Polish sweep settled; semver discipline begins here, so breaking changes now
mean a 2.0. (The remaining 1.0-hygiene items — a declared MSRV, `cargo-semver-checks`
and advisory scanning in CI — are tracked as follow-ups, not blockers.)

- Colorbars: `Plot::colorbar()` draws the colormap as a labeled strip down the right
  edge, legending a `Cells` layer's value range. The `heatmap` and `hist2d` presets
  turn it on by default (a color-coded grid with no value scale is half a chart); the
  bare `Cells` grammar stays uncolored-legend for full control. Sheds on narrow frames.

## 0.11.0 (Polish) — 2026-08-02

The API review before the 1.0 freeze: the breaking changes are settled here, while
the crate is still pre-1.0 and cheap to move on. A fallible boundary makes external
specs safe; M4's headline guarantee is real again; a few names stop lying.

- `Scale::Auto` is the new default, distinct from `Scale::Linear`. An automatic axis
  adapts to its layers (categorical when a bars or band-range layer is present,
  linear otherwise); an *explicitly* chosen scale is now always honored rather than
  silently overridden by a categorical layer. `Plot::validate` rejects a categorical
  layer under a numeric x scale, and categorical layers that disagree on their bands.
- Renames (breaking, landed early so downstream churn is minimal):
  - `stat::Grid` → `stat::Histogram2d` — it was a second public `Grid`, unrelated to
    the small-multiples `Grid` at the crate root; the name now says what it is.
  - `Ticks::step()` returns `Option<f64>` instead of `f64` — `None` for a lone tick or
    the non-uniform ticks of a log/time axis, rather than a `0.0` sentinel a caller
    could mistake for a real spacing.
- M4 is pixel-exact again — and honestly so. Large lines are now reduced in *mapped
  raster space*: a cheap min/max probe fixes the layout, then M4 buckets by the exact
  column each point renders into, so the downsampled raster is bit-identical to
  drawing every point (verified against a raw-render oracle across index and xy lines
  at several sizes). The extra probe pass trades a little speed — ~45 ms for ten
  million points, up from ~28 — for the restored guarantee; a single-pass path is
  tracked for later.
- A fallible validation boundary: `Plot::validate` checks a spec's invariants
  (paired channel lengths, rectangular grids, valid colormaps, finite manual
  domains, scale/domain compatibility) and returns the first problem as a typed
  `Error`; `Plot::try_render` validates then renders. `render` stays infallible and
  lenient — this is the strict counterpart for deserialized or configured specs.

## 0.10.1

Correctness hardening from an external audit. Most fixes make existing guarantees
real under composition, deserialization, and extreme inputs.

- Fixed domains (`x_domain`/`y_domain`) are now honored exactly — they no longer
  widen to the tick range — and every mark is clipped to the plot rectangle, so
  out-of-range data can no longer leak ink into the axes or a neighboring grid cell.
- Off-screen bar and area spans are clamped before rasterizing, so distant finite
  data under a narrow domain can no longer spin a near-unbounded draw loop.
- `Bins::auto` always covers the data and respects its cap: it widens the bin
  instead of dropping observations, so counts sum to the finite input count.
- `Moments::default()` now equals `Moments::new()` (extrema start unset, not `0`).
- M4 preserves a gap that falls inside a raster column — a `NaN` between two values
  no longer reconnects them. Downsampling is described honestly as silhouette-
  preserving; true pixel-exactness (mapped-space bucketing) is tracked for later.
- Deserialized specs that violate constructor invariants (empty colormap,
  zero-column grid, ragged range/area channels) now render defensively instead of
  panicking.
- `Ticks::linear` no longer panics or hangs on extreme finite bounds; `kde` declines
  a degenerate large-magnitude sample instead of over-allocating; `hist2d` of
  constant data renders instead of coming out blank; a log axis with a non-positive
  manual domain is clamped rather than panicking, and a value that maps off a log
  axis is treated as a gap.
- `lttb` and `m4` assert equal-length inputs, like the mark constructors; the
  `contour` preset validates its geometry and treats all non-finite values as gaps.
- Range body values now participate in y-axis fitting, so a body reaching past the
  whiskers is no longer clipped.

## 0.10.0 (Reach)

- Contour lines: `stat::contours` (marching squares — canonical shared-edge
  interpolation, center-average saddles, NaN gaps) and the `contour` preset with
  tick-chosen levels, colormap-graded and legend-labeled.
- `quiver` preset: a vector field as arrows drawn in data coordinates.
- `serde` feature: every spec type round-trips (plots, marks, scales, themes,
  frames, grids). Series gaps encode as `null` in JSON and decode back to gaps;
  function-backed lines refuse to serialize honestly.
- `ndarray` feature: one-dimensional arrays and views ingest directly, zero-copy
  when contiguous.
- `Colormap` stops are copy-on-write (`Colormap::new` is still const); `Colormap`
  is no longer `Copy`.

Deliberately not added: a pie preset (no x/y scales — it fights the marks-over-scales
grammar; part-to-whole is served by `bar`) and a `polars` dependency (too large;
polars already reaches a chart with no dependency through the zero-copy slice path —
see the README).

## 0.9.0 (Red Cavalry) — 2026-08-02

Riding into the ratatui ecosystem.

- The ratatui adapter (feature `ratatui`, depending only on `ratatui-core`):
  `plot.widget()` renders any chart straight into a `Buffer` — no ANSI round-trip,
  colors map onto cell styles, the host application keeps the terminal. Charset and
  theme are widget options; `cargo run --example tui --features ratatui` shows a
  live dashboard.
- The gallery now runs on real data (`examples/data/`, with provenance and
  licenses): the Keeling curve (NOAA, public domain), Palmer penguins (CC0), and a
  genuinely real training log — 1,000 per-step losses captured from poorgrad's
  bigram model. Six entries converted; mathematical examples stay mathematical.
- The corners line style (`LineStyle::Corners`): the classic asciichart look —
  one box-drawing glyph per column, `╭╮╰╯` elbows, `│` runs — with real axes
  underneath, and an honest `+`/`-`/`|` fallback in ASCII charsets.
- Retained-plot cloning measured at ~10 µs for 12 layers × 5k points
  (`plot/clone_12x5k_owned`) — cheap enough that no copy-on-write machinery is
  warranted.

## 0.8.0 (Black Cross) — 2026-08-02

The layout release: the charset ladder completes, and plots compose into grids.

- Sextant (2×3, Unicode 13) and octant (2×4, Unicode 16) charsets: braille density
  with solid ink. `Frame::detect` now auto-selects octants on terminals known to
  render them (kitty, ghostty, WezTerm, foot, recent VTE, Windows Terminal) —
  sniffed, never probed.
- Small multiples (`Grid`): plots pasted side by side with escape-aware padding;
  share axes by fixing domains, not by a mode.
- Manual axis domains (`Plot::x_domain`, `Plot::y_domain`): matplotlib's xlim/ylim;
  data outside clips honestly.

## 0.7.0 (Eight Red Rectangles) — 2026-08-02

The quality release: typed scales, named axes, and honest ASCII — driven by the
first full audit.

- The scale specification (`Scale`: `Linear | Log | Time | Bands`, via
  `Plot::x_scale`/`y_scale`): one typed axis spec replaces the three boolean flags
  (which remain as sugar); an explicit `Scale::Bands` declares a categorical axis
  without needing a bar layer — the violin preset now uses it instead of a
  data-free range.
- Axis titles (`Plot::x_label`, `Plot::y_label`): x centered under the tick labels,
  y written vertically along the left edge; both shed when the frame is tight.
- Internal: the plot pipeline split into stage modules (resolve → layout → chrome →
  draw) — verified byte-identical by the golden suite; crate-level rustdoc rewritten
  (it had been six releases stale).

## 0.6.0 (The Knife Grinder) — 2026-08-02

Time and motion: calendar axes, rolling windows, and live charts.

- Time axes (`Plot::time_x`, `Ticks::time`): unix seconds in, calendars out — a
  1s-to-decades interval ladder aligned to real boundaries (Mondays, month firsts),
  multi-scale labels (`14:05`, but `Aug 2` at midnight and `2027` at January), exact
  Gregorian arithmetic, UTC, no dependencies.
- Rolling windows (`stat::Window`): trailing mean/sum/min/max with partial starts
  (no warm-up gap) and gap-aware reductions.
- Streaming (`stream::Ring`, `stream::Live`, `stream::Rate`): a thread-shared
  sliding window (the library's one lock — producers push, renderers snapshot),
  an in-place repaint handle (cursor up, erase down, one buffered write:
  flicker-free, scrollback-safe), and a counter-to-delta helper. One live frame
  renders in well under a millisecond (see `benches/render.rs`).

## 0.5.0 (Sportsmen) — 2026-08-01

The statistics release: the mark family is complete, and the statistical charts no
terminal library ships are here.

- Range (`mark::Range`): the eighth and final mark — vertical intervals with
  optional `body` and `marker` channels, so error bars, boxes, and candles are one
  mark with channels, not three marks. Band placement (`Range::over`) shares the
  categorical axis machinery with bars.
- Box plots (`stat::BoxStats`, `malevich::box_plot`): type-7 quartiles, Tukey 1.5×IQR
  whiskers, outliers as dots.
- Densities (`stat::kde`, `malevich::density`, `malevich::violin`): Gaussian KDE with
  Silverman bandwidth over linear binning (no FFT); violins as mirrored densities via
  the new horizontal area orientation (`Area::horizontal`).
- Error bars (`malevich::error_bars`): capped Range intervals around measured points.

## 0.4.0 (Suprematist Composition) — 2026-08-01

The daily driver: the mark family grows to seven, and the statistical presets with it.

- Cells (`mark::Cells`, `scale::Colormap`, `malevich::heatmap`, `malevich::hist2d`,
  `stat::bins2`): value grids as a shade ramp (`░▒▓█`) colored by a colormap
  (viridis-like default) — value carried by glyph and color, readable at every tier
  including plain; grids map onto data coordinates via `Cells::extents`; empty 2D
  bins stay honestly blank.

- Area (`mark::Area`): baseline fills and between-bands, drawn as vertical subpixel
  runs — solid in every charset, subpixel edge precision, gap-breaking. `stat::stack`
  turns series into cumulative bands for stacked areas.
- Annotations (`mark::Rule`, `mark::Text`): reference lines and notes at data
  coordinates; both extend the axis domains, draw in the default foreground, and
  never consume palette slots.
- Steps (`malevich::stairs`, `malevich::ecdf`, `stat::ecdf`): step charts and
  empirical distributions as presets over the line mark.

## 0.3.0 (Airplane Flying) — 2026-08-01

The pipeline release: the stat layer lands, and ten million points become cheap.

- Histograms (`stat::Bins`, `mark::Bars::spans`, `malevich::hist`): automatic bin
  counts (Sturges/Freedman–Diaconis) with nice decimal edges, mergeable bin counts,
  and contiguous span bars on a numeric axis.
- Group-by (`stat::Agg`): string-keyed grouping with the shared reducer vocabulary —
  `count`, `sum`, `mean`, `min`, `max`, `median` — feeding `Bars::new` directly.
- Log axes (`Plot::log_x`, `Plot::log_y`, `Ticks::log10`): decade ticks with
  superscript labels; values at or below zero become gaps, because a log axis cannot
  place them honestly.
- The aggregation pipeline (`stat`): M4 downsampling (`stat::M4`, `stat::m4`) —
  min/max/first/last per raster column, pixel-exact for line rendering, mergeable
  across chunks, gap-preserving — inserted automatically for line layers past four
  points per subpixel column. Ten million points render end to end in ~28 ms
  (measured; see `benches/render.rs`). Also `stat::lttb` (count-targeted,
  shape-preserving) and `stat::Moments` (Welford + Chan merge).
- SI-prefixed tick labels: axes reaching ±10⁴ (or below 10⁻³) share one prefix
  (`20k`, `2.5M`, `100µ`); the numeric part times the prefix still equals the value
  exactly, and zero stays bare.

## 0.2.0 (Red Square) — 2026-08-01

Color and the next two marks: the chart, the dots, and the bars now look considered
at every color tier.

- Half-block (`▀▄█`, 1×2) and quadrant (`▘▚▟`…, 2×2) charsets: solid-block
  alternatives to braille, selectable per frame.
- Legends: `.label("…")` on any mark grows a legend row with per-kind colored
  swatches, shed first when the frame is short.
- Themes (`Theme`, `Frame::theme`): the palette as a value — `DARK` (default),
  `LIGHT` (readable on white), `COLORFGBG` detection, or any custom palette.
- Bars (`mark::Bars`, `scale::Band`, `malevich::bar`): categorical bar charts from a
  zero baseline with eighth-block partial tops, coarse below-baseline fills for
  negative values, band-fitted category labels, and continuous layers (trend lines)
  positioning over band centers.
- Points (`mark::Points`, `malevich::scatter`): unconnected dots; marks now join
  under the closed `mark::Mark` enum and `Plot::layer(impl Into<Mark>)`.
- Color ladder (`Color::{Ansi256, Rgb}`, `ColorMode::{Plain, Ansi16, Ansi256,
  TrueColor}`): honest downhill quantization (RGB → 256-cube → nearest-16), named
  colors stay palette-relative, run-length encoding merges colors that quantize
  equal. Detection adds `CLICOLOR_FORCE`, `COLORTERM`, `256color` terms, `TERM=dumb`,
  and non-UTF-8 locale sniffing.
- Display-width discipline: labels measured in terminal columns (CJK-safe), wide
  glyphs pair with continuation cells and never corrupt alignment, truncation uses an
  ellipsis. New dependency: `unicode-width`.

## 0.1.0 (Black Square) — 2026-08-01

The vertical spine: one mark, every layer of the architecture, done properly.

- The plot pipeline (`Plot`, `Frame`, `mark::Line`, `malevich::line`): layered line
  charts over shared scales with measured (never fixed) layout, collision-aware x
  labels, chrome shedding in undersized frames, function sampling at raster
  resolution, a default palette, and `Display` via `Frame::detect`. Presets are
  asserted bit-identical to their grammar expansion.
- The examples gallery (`EXAMPLES.md` + `regen_gallery`): deterministic, CI-checked —
  the showcase and the system test in one artifact.
- Rendering (`render::Surface`, `render::Charset`, `render::Color`): one generic
  subpixel surface over charset codecs (braille 2×4 and ASCII for now), clipped
  infallible drawing, text sharing the grid with pixels, plain and run-length ANSI
  encoders.
- Data ingestion (`data::Series`, `data::IntoSeries`): zero-copy from `f64` slices,
  copy-once conversion from all primitive numeric types, `NaN` preserved as the gap
  encoding.
- Tick placement (`scale::Ticks`): extended Wilkinson (Talbot–Lin–Hanrahan) with
  exact-decimal labels — labels parse back to their values, share one fraction width
  per axis, and never show float artifacts. Placement runs in microseconds.
- Project scaffold: crate skeleton, terminology contract, CI.
