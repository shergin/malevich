# malevich

**Terminal plotting for Rust: a small grammar of marks, honest axes, millions of
points.**

![Malevich terminal rendering with cell glyphs and real pixels](examples/suprematist-composition.png)

Eight marks. A real statistics layer. Ten million points in tens of milliseconds on
the [recorded baseline](BENCHMARKS.md). Axes placed by the same algorithm the
visualization literature settled on — with labels that are exact decimals, never
`0.30000000000000004`. All of it in plain values whose explicit render paths produce
a deterministic `String`, degrade gracefully on any terminal, and never take over it.

```rust
println!("{}", malevich::line(&[1.0, 5.0, 2.0, 8.0][..]));
```

<!-- generated:readme_sample -->
```text
8 ┤                         ▄▀
  │                       ▄▀
  │       ▄▞▀▚▄▖       ▗▞▀
4 ┤    ▄▞▀     ▝▀▚▄▖ ▗▞▘
  │ ▄▞▀            ▝▀▘
0 ┤▀
  └┬────────┬───────┬────────┬
   0        1       2        3
```
<!-- /generated -->

```rust
println!("{}", malevich::bar(["mon", "tue", "wed", "thu", "fri"], &[3.0, 7.0, 4.5, 8.0, 6.0][..]));
```

<!-- generated:readme_bars -->
```text
8 ┤         ▂▂▂▂▂         █████
  │         █████         █████  ▃▃▃▃▃
  │         █████  ▁▁▁▁▁  █████  █████
4 ┤         █████  █████  █████  █████
  │  █████  █████  █████  █████  █████
  │  █████  █████  █████  █████  █████
0 ┤  █████  █████  █████  █████  █████
  └─────────────────────────────────────
      mon    tue    wed    thu    fri
```
<!-- /generated -->

And the charts no other terminal library ships — box plots, violins, densities, 2D
histograms:

<!-- generated:boxes -->
```text
                 flipper length by species
  230 ┤                                        ▀▀▜▀▀
      │                                          ▐
  220 ┤                                       ███████▌
      │                                       ━━━━━━━━
  210 ┤        ▄▄▄▄▄           ▀▀▜▀▀          ▀▀▀▜▀▀▀▘
m     │          ▌               ▐             ▄▄▟▄▄
m 200 ┤          ▌           ▐███████▌
      │      ▗▄▄▄▙▄▄▄        ▐━━━━━━━━
  190 ┤      ━━━━━━━━━       ▝▀▀▀▜▀▀▀▘
      │      ▝▀▀▀▛▀▀▀            ▐
  180 ┤          ▌               ▐
      │        ▄▄▙▄▄           ▀▀▀▀▀
  170 ┤          ▘
      └─────────────────────────────────────────────────────
              Adelie         Chinstrap        Gentoo
```
<!-- /generated -->

And the classic asciichart look, one glyph per column, whenever you want charts
this quiet — with real axes underneath, which the original never had:

```rust
Plot::new().layer(Line::y(&values[..]).style(LineStyle::Corners))
```

<!-- generated:corners -->
```text
                          the corners style
 15 ┤              ╭───────────╮
    │            ╭─╯           ╰──╮
 10 ┤          ╭─╯                ╰─╮
    │        ╭─╯                    ╰╮
  5 ┤      ╭─╯                       ╰─╮
    │     ─╯                           ╰─╮
  0 ┤                                    ╰╮
    │                                     ╰─╮
 -5 ┤                                       ╰─╮
    │                                         ╰─╮                  ╭──
-10 ┤                                           ╰─╮              ╭─╯
    │                                             ╰──╮       ╭───╯
-15 ┤                                                ╰───────╯
    └┬──────────┬─────────┬──────────┬──────────┬──────────┬─────────┬
     0         10        20         30         40         50        60
```
<!-- /generated -->

Every chart in these docs is real program output, spliced in by
`cargo run --example regen_docs` and verified in CI — never typed by hand. More in the
gallery: [EXAMPLES.md](EXAMPLES.md), and `cargo run --example showcase` renders a
colored tour sized to your terminal.

In a terminal it looks like this — `cargo run --example showcase --features pixel`
renders every chart twice, cells on the left and real pixels (sixel / kitty /
iTerm2) on the right, from the same plot values:

![Loss curves, a calendar time axis, and smoothing — cell rendering beside pixel rendering](examples/showcase-lines.png)

![A 2D density, contour lines, and a vector field — cell rendering beside pixel rendering](examples/showcase-2d.png)

## Why malevich

- **A small grammar, not a chart zoo.** Eight marks (line, points, bars, area, cells,
  range, rule, text) × a stats layer × shared scales compose into the whole basic
  chart catalog. Every preset — `line`, `scatter`, `bar`, `hist`, `stairs`, `ecdf`,
  `heatmap`, `hist2d`, `density`, `box_plot`, `violin`, `error_bars`, `trend` — is
  proven bit-identical to its grammar expansion in tests. **Color speaks data**: a
  `color_by` channel colors points, lines, bars, and intervals by category —
  colorblind-safe Okabe–Ito colors, a categorical legend, and portable marker
  shapes (`•`, `+`, `x`, `*`, `o`) that cycle automatically in colorless output so
  groups never vanish in a pipe. Grouped scatters, volcano plots, Manhattan plots,
  and candlesticks are a few grammar lines each, never presets
  ([EXAMPLES.md](EXAMPLES.md)).
- **The statistical set no terminal library has.** Box plots with type-7 quartiles
  and Tukey whiskers, violins from a real KDE (Silverman bandwidth), least-squares
  trend lines with R² and a confidence band (`trend`; `stat::Fit` is a mergeable
  online accumulator, so it fits streams and parallel reduction trees),
  ECDFs with an optional DKW confidence band, symmetric and asymmetric error
  bars, 2D densities (with a colorbar legending the value scale) — the charts
  science and ML actually need. One `Reducer` vocabulary — count, sum, mean,
  median, min, max, and type-7 percentiles, the same estimator the box plot
  uses — reduces groups, rolling windows, and histogram bins alike, so a
  rolling p95 or a binned median is one call. Defaults stay one-call; configurable
  `_with` variants return typed errors for invalid data or options while exposing
  histogram geometry, KDE/violin resolution, contour levels, and colormaps when the
  data or host needs control. The named colormaps are a curated set that
  stays distinguishable down the whole color ladder — sequential
  (`Colormap::VIRIDIS`, `MAGMA`, `CIVIDIS`, `GREYS`) and diverging
  (`RED_BLUE`, `PURPLE_ORANGE`): center one on a data value
  (`Colormap::RED_BLUE.centered_at(0.0)`) and correlation or log-fold-change
  renders honestly, opposite signs in opposite colors and a symmetric colorbar.
- **Millions of points, measured.** Large line layers are aggregated by M4 —
  min/max/first/last per raster column, bucketed by the column each point renders
  into, so the reduction is *pixel-identical* to drawing every point. Ten million
  points render end to end in tens of milliseconds single-threaded on the
  [dated baseline](BENCHMARKS.md); `cargo bench --bench render` carries the complete
  suite. Online accumulators (`Moments`, `Fit`, `Bins`, M4) expose their merge laws;
  reducers and batch transforms keep distinct execution contracts instead of
  pretending every statistic has the same algebra.
- **Axes that are actually good.** Extended-Wilkinson tick placement (Talbot, Lin,
  Hanrahan 2010), exact-decimal labels that parse back to their values, one shared SI
  prefix per axis (`2.5M`, `100µ`), log axes with superscript decades, calendar time
  axes with multi-scale labels (`14:05`, `Aug 2`, `2027`), typed axis specs
  (`Scale::{Auto, Linear, Log, Time, Bands}`), axis titles, band scales with fitted
  category labels, collision-aware layout that sheds furniture instead of failing.
- **Renders everywhere, honestly.** ASCII is the guaranteed fallback; UTF-8 auto
  detection conservatively uses old block-element quadrants. Braille, sextants, and
  Unicode 16 octants remain explicit high-density choices for fonts that cover them
  (`--charset` or `MALEVICH_CHARSET`). Four color tiers (truecolor → 256 → 16 →
  plain) quantize honestly downhill; heatmap half-blocks carry independent upper and
  lower colors while plain output retains an averaged shade. Piped output is clean
  plain text; CJK labels stay aligned, combining marks are deliberately dropped at
  the cell grid, and `NaN` is always a visible gap, never interpolated away.
- **Real pixels where the terminal speaks them (feature `pixel`).** The ladder's
  top rung: `plot.render_pixels(&frame, &graphics)` keeps title, axes, and legend
  as crisp text cells and draws the plot rectangle as an actual image — sixel,
  kitty graphics, or iTerm2 inline PNG, all hand-rolled, no new dependencies.
  Marks rasterize at device-pixel resolution through the same pipeline (M4 buckets
  per pixel column; heatmaps sample per pixel), undrawn panel stays transparent to
  your terminal background, and the result is still a deterministic `String`.
  `Capabilities::detect_for(&destination)` uses the stream that will receive the
  plot to decide whether its cached terminal probe is safe, while `detect()`
  remains the stdout convenience. Pass that value to
  `plot.render_with_capabilities(&frame, &caps)` for a pure, explicit auto-render;
  `plot.render_best(&frame)` is the one-call stdout ladder top. Every gallery
  example upgrades with
  `--features pixel`, and `cargo run --example showcase --features pixel` renders
  each chart side by side, cells against pixels.
- **Rich Evcxr notebooks, still just cells (feature `evcxr`).** End a Jupyter cell
  with a `Plot` and Evcxr calls `evcxr_display`, rendering the complete chart as a
  self-contained HTML terminal card. Quadrants and box-drawing stay crisp, mark colors
  become RGB spans, chrome follows the card foreground, and plot text is HTML-escaped.
  The adapter adds no dependency; `plot.to_html(&frame)` is the pure, deterministic
  path for custom sizes and mark palettes. `Theme::LIGHT` selects the light card;
  other custom palettes currently use the dark card foreground/background because
  card colors are not theme roles yet. In an Evcxr notebook:

  ```rust
  :dep malevich = { version = "1.17", features = ["evcxr"] }
  use malevich::{Line, Plot};

  let values = [1.0, 5.0, 2.0, 8.0];
  Plot::new().layer(Line::y(&values[..])).title("training")
  ```

  Redirect `cargo run --example evcxr --features evcxr > plot.html` for a standalone
  fragment you can inspect in a browser. The same cell also renders in the terminal
  `evcxr` REPL (a plain plot via a `text/plain` fallback), and with the `pixel`
  feature also enabled it becomes a real sixel/kitty/iTerm2 image there.
- **Small multiples and fixed axes.** `Grid` pastes plots side by side
  (escape-aware alignment); `x_domain`/`y_domain` fix axes matplotlib-style — so
  shared scales across a dashboard are an explicit composition, not a mode.
- **A ratatui widget, if you want one.** With the `ratatui` feature (depending only
  on `ratatui-core`), `plot.widget()` drops any chart into a TUI — cells written
  straight into the buffer, colors as styles, your app keeps the terminal
  (`cargo run --example tui --features ratatui`). For full apps, [`demos/`](demos/)
  has `fred` — a five-view Federal Reserve data browser (`cargo run -p fred`) —
  `sysmon` — a live system monitor streaming CPU/memory/network through
  `stream::Ring` into a per-core heatmap (`cargo run -p sysmon`) — and `learn` —
  a two-moons MLP trained by [topos](https://crates.io/crates/topos), its loss
  curve with EWMA smoothing and the learned decision regions charted as it
  trains (`cargo run -p learn --release`).
- **Serializable specs, no lies.** With the `serde` feature, [`Document`](SERDE.md)
  is the validated, versioned format for files, caches, and network messages; golden
  v1 fixtures keep compatibility testable. Raw spec types still round-trip for
  short-lived interchange. Gaps encode as `null` and decode back to gaps; a
  function-backed line refuses to serialize rather than silently drop its curve.
- **Plots from ndarray.** With the `ndarray` feature, one-dimensional arrays and
  views plot directly — contiguous storage zero-copy, a strided matrix column
  converted once, like any other input.
- **Plots from polars, with no dependency.** polars is too big to depend on, but it
  needs no special support: a contiguous column borrows zero-copy, and its
  null-yielding iterator maps straight onto the gap convention.

  ```rust
  // Contiguous and null-free: borrowed, no copy.
  let chart = malevich::line(df.column("loss")?.f64()?.cont_slice()?);

  // Anything else: nulls become gaps (NaN), converted once at ingestion.
  let series = df.column("loss")?.f64()?.iter().map(|v| v.unwrap_or(f64::NAN));
  let chart = malevich::line(series.collect::<Vec<_>>());
  ```
- **Live charts without a framework.** A thread-shared sliding window plus an
  in-place repaint handle (cursor up, erase down, one write): flicker-free streaming
  that survives in scrollback and never takes over your terminal
  (`cargo run --example live`).
- **Plots are plain values.** `Clone + Send + Sync`; `Plot::render`, `to_html`, and
  `render_with_capabilities` are pure functions of explicit values — build on one
  thread, render on another, snapshot-test the strings. `Display`, `Frame::detect`,
  and `render_best` are the documented conveniences that read the environment;
  pixel capability detection may also perform one bounded, cached terminal probe.
  Two tiny required dependencies (`terminal_size`, `unicode-width`).

**Stability**: the crate is 1.x — the public API follows semver (breaking changes
mean a 2.0), guarded in CI by `cargo-semver-checks` against the last published
release. The concept vocabulary is documented in [TERMINOLOGY.md](TERMINOLOGY.md)
and changes are in the [CHANGELOG](CHANGELOG.md). Maintainers use the reproducible
[release checklist](RELEASING.md).

## Command line

The same renderer, from any shell. [`kaz`](cli/) (crate
[`malevich-cli`](cli/README.md)) is a stdin-first plotter — one subcommand per
chart, plot on stderr, data passthrough on stdout so it can sit mid-pipeline:

```sh
cargo install malevich-cli               # installs the `kaz` binary
cat loss.tsv | kaz line -t training
awk '{print $5}' access.log | kaz hist
cut -f2 species.tsv | kaz count
cat data.tsv | kaz line -O | next-tool   # plot on stderr, data flows on
kaz scatter penguins.tsv -H --by species --emit-code   # the equivalent Rust program
```

It contains zero rendering logic — argument parsing, stdin framing, and calls
into this crate's public API — which makes it the proof that a pure
string-renderer is enough. Details in [cli/README.md](cli/README.md).

## What it will not be

Not a TUI framework (it never owns the terminal or handles input). No animations. No
file parsing or dataframes in core — ingestion traits only. No config-object kitchen
sink: if an option is not a mark channel, stat parameter, scale option, or theme
entry, it does not ship.

## Name

Kazimir Malevich painted a black square on a plain ground and meant it: a small
vocabulary of geometric forms, composed deliberately. That is the design budget of
this library.

## Acknowledgements

malevich stands on the shoulders of giants — the algorithms, libraries, and grammars
that taught this project what it knows are credited, specifically, in
[ACKNOWLEDGEMENTS.md](ACKNOWLEDGEMENTS.md).

## License

MIT or Apache-2.0.
