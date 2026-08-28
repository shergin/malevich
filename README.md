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

And the ML set: attention maps and confusion matrices on token-labeled band
axes, decision boundaries as categorical cells, images as rgb cells, loss
landscapes with optimizer trajectories — every one a grammar composition, none
a preset. A logarithmic colormap keeps weights spanning decades apart, and the
causal mask's zeros render as honest gaps:

<!-- generated:attention -->
```text
                    attention, layer 7 head 3
        │  █████                                            █
    The ┤  █████                                            █
        │  █████  █████                                     █
  robot ┤  █████  █████                                     █ 10⁻¹
        │  █████  █████                                     ▓
q   ate ┤  ▓▓▓▓▓  █████ █████                               ▓
u       │  ▓▓▓▓▓  █████ █████                               ▓
e       │  ▓▓▓▓▓  ▓▓▓▓▓ █████  █████                        ▓
r   the ┤  ▓▓▓▓▓  ▓▓▓▓▓ █████  █████                        ▒
y       │  ▒▒▒▒▒  ▓▓▓▓▓ ▓▓▓▓▓  █████  █████                 ▒ 10⁻³
    red ┤  ▒▒▒▒▒  ▓▓▓▓▓ ▓▓▓▓▓  █████  █████                 ▒
        │  ░░░░░  ▒▒▒▒▒ ▓▓▓▓▓  ▓▓▓▓▓  █████  █████          ▒
  apple ┤  ░░░░░  ▒▒▒▒▒ ▓▓▓▓▓  ▓▓▓▓▓  █████  █████          ░
        │  ░░░░░  █████ ▒▒▒▒▒  ▒▒▒▒▒  ▓▓▓▓▓  █████  █████   ░
      . ┤  ░░░░░  █████ ▒▒▒▒▒  ▒▒▒▒▒  ▓▓▓▓▓  █████  █████   ░
        │                                                   ░ 10⁻⁵
        └──────────────────────────────────────────────────
            The   robot   ate    the    red  apple    .
                                 key
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

The design is argued in [docs/vision.md](docs/vision.md); the constraints it
assumes live in [docs/principles/](docs/principles/). The short version:

- **A small grammar, not a chart zoo.** Eight marks (line, points, bars, area,
  cells, range, rule, text) × a stats layer × shared scales compose into the
  whole basic chart catalog. Every preset — `line`, `hist`, `box_plot`,
  `violin`, `trend`, … — is proven byte-identical to its grammar expansion in
  tests; grouped scatters, volcano plots, Manhattan plots, and candlesticks are
  a few grammar lines each, never presets ([EXAMPLES.md](EXAMPLES.md)).
- **The statistical set no terminal library has.** Box plots with type-7
  quartiles and Tukey whiskers, violins from a real KDE, streaming
  least-squares trend lines with R² and a confidence band, ECDFs with an
  optional DKW band, ROC curves with their area, 2D densities, debiased EWMA
  smoothing — and one `Reducer` vocabulary across bins, groups, and rolling
  windows, so a rolling p95 or a binned median is one call. A `color_by`
  channel colors marks by category: colorblind-safe Okabe–Ito colors, a
  categorical legend, and marker shapes that keep groups separable in
  colorless output. Curated sequential, diverging, and logarithmic colormaps
  keep heatmaps honest — signed data centered, decades distinguishable, zeros
  as gaps.
- **Millions of points, measured.** Large lines reduce by M4, bucketed by the
  rendered column — pixel-identical to drawing every point. Ten million points
  render in tens of milliseconds on the dated baseline; grids denser than the
  raster reduce bucket-exactly through the same `Reducer` vocabulary.
  Mechanisms and numbers: [docs/performance.md](docs/performance.md).
- **Axes that are actually good.** Extended-Wilkinson tick placement (Talbot,
  Lin, Hanrahan 2010), exact-decimal labels that parse back to their values,
  one SI prefix per axis (`2.5M`, `100µ`), log axes with superscript decades,
  calendar time axes, band axes that label matrix rows in matrix order — and
  collision-aware layout that sheds furniture instead of failing.
- **Renders everywhere, honestly.** Charset and color ladders from Unicode 16
  octants down to plain ASCII and truecolor down to a clean pipe; CJK labels
  stay aligned and `NaN` is always a visible gap. Detection rules and
  overrides: [docs/terminal.md](docs/terminal.md).
- **Real pixels where the terminal speaks them** (feature `pixel`). Sixel,
  kitty graphics, or iTerm2 inline PNG, all hand-rolled: chrome stays crisp
  text, the panel becomes an actual image, and the result is still a
  deterministic `String`. `render_best` is the one-call ladder top.
  [docs/pixels.md](docs/pixels.md).
- **Rich Evcxr notebooks, still just cells** (feature `evcxr`). End a Jupyter
  cell with a `Plot` and it renders as a self-contained HTML terminal card;
  the terminal REPL gets a plain fallback that the `pixel` feature upgrades to
  a real image. [docs/notebooks.md](docs/notebooks.md).

  ```rust
  :dep malevich = { version = "1.18", features = ["evcxr"] }
  use malevich::{Line, Plot};

  let values = [1.0, 5.0, 2.0, 8.0];
  Plot::new().layer(Line::y(&values[..])).title("training")
  ```
- **Composition over modes.** `Grid` pastes small multiples side by side;
  `x_domain`/`y_domain` fix axes matplotlib-style, so shared scales are an
  explicit composition, not a mode. A ratatui widget (feature `ratatui`,
  depending only on `ratatui-core`) drops any chart into a TUI — and rendered
  stateful, makes it interactive without malevich ever handling input: the
  widget caches the render's cell↔data `Mapping` for hit-testing, applies a
  `Viewport` (zoom and pan as pure domain arithmetic), and interprets default
  mouse gestures — a crosshair that snaps to the nearest datum and reads out
  its value axis-formatted (gaps as `—`, never interpolated), wheel zoom
  under the cursor, drag pan, rubber-band zoom — from coordinates the host
  feeds it. Zooming is just a domain window, so M4 re-aggregates per frame:
  a zoomed ten-million-point frame renders in under 19 ms on the
  [recorded baseline](BENCHMARKS.md) — past 50 fps — and
  `cargo run --release --example zoom --features ratatui` is that claim,
  live. [`demos/`](demos/) holds full apps: `fred`, a
  five-view Federal Reserve data browser wearing the full gesture set;
  `sysmon`, a live system monitor; and `learn`, a two-moons MLP trained by
  [topos](https://crates.io/crates/topos), charting as it trains.
- **Serializable specs, no lies** (feature `serde`).
  [`Document`](docs/serde.md) is the validated, versioned format for files,
  caches, and network messages; gaps encode as `null` and decode back to
  gaps, and a function-backed line refuses to serialize rather than silently
  drop its curve.
- **Data arrives at the rim.** Anything series-shaped converts exactly once
  into contiguous `f64` — the `ndarray` feature ingests arrays and views
  (contiguous storage zero-copy), and polars needs no feature at all, because
  a contiguous column is already a borrowed slice and its null-yielding
  iterator maps straight onto the gap convention:

  ```rust
  // Contiguous and null-free: borrowed, no copy.
  let chart = malevich::line(df.column("loss")?.f64()?.cont_slice()?);

  // Anything else: nulls become gaps (NaN), converted once at ingestion.
  let series = df.column("loss")?.f64()?.iter().map(|v| v.unwrap_or(f64::NAN));
  let chart = malevich::line(series.collect::<Vec<_>>());
  ```
- **Live charts without a framework.** A thread-shared sliding window plus an
  in-place repaint handle (cursor up, erase down, one write): flicker-free
  streaming that survives in scrollback and never takes over your terminal
  (`cargo run --example live`).
- **Plots are plain values.** `Clone + Send + Sync`; `Plot::render`,
  `to_html`, and `render_with_capabilities` are pure functions of explicit
  values — build on one thread, render on another, snapshot-test the strings.
  `Display`, `Frame::detect`, and `render_best` are the documented
  conveniences that read the environment. Two tiny required dependencies
  (`terminal_size`, `unicode-width`).

**Stability**: the crate is 1.x — the public API follows semver (breaking changes
mean a 2.0), guarded in CI by `cargo-semver-checks` against the last published
release. The concept vocabulary is documented in [docs/terminology.md](docs/terminology.md)
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
