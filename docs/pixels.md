# Pixels

The ladder's top rung (feature `pixel`). Where the terminal speaks an image
protocol, the plot panel is a real image, and title, axes, and legend stay
crisp text cells. The result is still a deterministic `String`.

The output is hybrid on purpose. Malevich owns no font rasterizer. That is
the small-dependency budget, so text stays the terminal's job, as in cell
rendering, and only the plot rectangle becomes pixels. Marks rasterize at
device-pixel resolution through the same pipeline: M4 buckets per pixel
column, heatmaps sample per pixel. The undrawn panel stays transparent to
the terminal background.

## Turn it on

```sh
cargo add malevich --features pixel
cargo run --example showcase --features pixel   # every chart, cells beside pixels
```

One call, stdout, the best tier the terminal offers:

```rust
println!("{}", plot.render_best(&frame));
```

The explicit, pure path — detect once, render many:

```rust
let caps = malevich::pixel::Capabilities::detect_for(&std::io::stdout());
println!("{}", plot.render_with_capabilities(&frame, &caps));
```

`Capabilities` is a plain value. Log it, cache it, override it. When it
offers no protocol, every render falls back to cells. There is no
"unsupported terminal" error anywhere.

## Protocols

| protocol | what it is | standing |
|---|---|---|
| `Kitty` | raw RGBA with alpha | the most capable |
| `Sixel` | DEC 1987, palette-banded | the most widely spoken |
| `ITerm2` | an inline PNG pinned to the panel's cell box | iTerm2 |

Encoders are hand-rolled and dependency-free. Each is a thin layer over the
shared pixel panel.

## Detection: sniff and probe

Two tiers with different licenses:

- **Sniffing** reads environment variables. It is free, instant, and wrong
  only by omission. It may run anywhere. It knows kitty (`KITTY_WINDOW_ID`,
  `KITTY_PID`, a kitty `TERM`), Ghostty (`GHOSTTY_BIN_DIR`, its `TERM` or
  `TERM_PROGRAM`), iTerm2 (`TERM_PROGRAM`, or `LC_TERMINAL`, which ssh
  forwards), WezTerm (`TERM_PROGRAM` or `WEZTERM_EXECUTABLE`), Rio, Warp,
  foot, mlterm, Konsole 22.04+, and Windows Terminal. `TMUX`, a `screen`
  or `tmux` `TERM`, `dumb`, and `unknown` mean cells.
  `MALEVICH_GRAPHICS=kitty|sixel|iterm2|none` outranks all of it, for the
  user who has arranged passthrough or wants cells.
- **Probing** asks the terminal itself, one raw-mode `/dev/tty` round trip:
  the kitty graphics query, XTVERSION, XTSMGRAPHICS, and `CSI 16 t` for the
  cell size, with DA1 as the ordering barrier. It is ground truth that
  survives ssh, about 100 ms, once per process, and licensed only where
  writing escapes is safe: the actual output destination is a tty, no tmux
  or screen in between, `TERM` not dumb.

`Capabilities::detect_for(&destination)` keys the probe decision to the
stream that will receive the plot. `detect()` is the stdout convenience.
An unanswered probe is not evidence. It degrades to the sniff answer. The
value records which tier answered (`Source::Probed` or `Source::Sniffed`).
A host that already knows its terminal — a remote session's, a second
window's — states the answer with `Capabilities::new(protocols, cell_size)`
(`Source::Declared`) and skips detection.

## Inside a ratatui app

With both `pixel` and `ratatui` enabled, the same panels render inside a
TUI. `chart.widget().graphics(g)` on a stateful widget reserves its area in
the buffer — spaces, skipped, so ratatui's diff leaves the image alone —
and stores the encoded block in the `PlotState`. The host emits it after
`terminal.draw` with `Graphics::present`: one synchronized write, new kitty
placements created before the old are retired. Probe capabilities *before*
`ratatui::init()`. The query reads terminal replies a raw-mode event loop
would swallow. Zoom, pan, crosshair, and snap keep working over the image.
The chrome becomes anti-aliased marks drawn into the panel itself. The
mechanics live in
[interaction.md](interaction.md#real-pixels). `cargo run -p fred` in a
kitty/sixel/iTerm2 terminal is the live proof.

## On a page

The same hybrid, for a host that draws SVG. Chrome stays the cell card:
rectangles and text the host's font draws. The panel is the device-pixel
raster, one path of rectangles per color. There is no image element and no
dependency.
GitHub's sanitizer keeps rectangles and drops embedded images. `Plot::to_svg`
stays the cell-grid card and needs no feature.

```rust
use malevich::pixel::{Graphics, Protocol};

let svg = plot.to_svg_pixels(&frame, &Graphics::new(Protocol::Kitty));
```

`Graphics::protocol` is terminal transport and does not change the bytes.
Any protocol is fine. Cell size and stroke are the ones the panel uses, the
same fields `render_pixels` reads. `economical()` is the caller's choice, as
it is for a terminal. Default-colored marks take the card foreground, so an
unlabeled series matches the axes. Named colors freeze to the same concrete
RGB the pixel canvas uses. Ink centered on the fitted domain edge clips, as
it does on the pixel canvas; widen the domain with `x_max` or `y_max` when
the whole marker has to stay inside. A text-only plot, a zero cell size, and
a frame with no panel degrade to `Plot::to_svg`. In-panel text marks use the
pixel font. Axis text does not.

The size follows the ink, not the frame. A run of one color along a pixel row
is one rectangle, and it grows down while the rows below repeat it. Bars and
flat fills cost little; anti-aliased lines and translucent fills cost more. A
smooth color field changes color at nearly every pixel and runs to megabytes.
For that plot, `Plot::to_svg` or a smaller cell size is the better card.

## Rough edges

- A multiplexer (tmux, screen) blocks probing by design. Sniffed answers
  still apply, and cells always work.
- Font coverage is irrelevant here. Cell size in device pixels matters for
  sharpness. When the terminal will not report it, a documented fallback
  size is used.
- The first probe is a real terminal round trip. Render paths never write
  queries.

The argument is
[Degradation is the contract](principles/degradation-is-the-contract.md)
and [The frame is run state](principles/frame-is-run-state.md). Vocabulary
is in [terminology](terminology.md), under Graphics, Capabilities, and
Protocol.
