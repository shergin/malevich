# Pixels

The ladder's top rung (feature `pixel`): where the terminal speaks an image
protocol, the plot panel is drawn as a real image while title, axes, and
legend stay crisp text cells. The result is still a deterministic `String`.

The output is hybrid on purpose. Malevich owns no font rasterizer — that is
the small-dependency budget — so text stays the terminal's job, exactly as
in cell rendering, and only the plot rectangle becomes pixels. Marks
rasterize at device-pixel resolution through the same pipeline (M4 buckets
per pixel column; heatmaps sample per pixel), and the undrawn panel stays
transparent to your terminal background.

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

`Capabilities` is a plain value: log it, cache it, override it. When it
offers no protocol, every render falls back to cells — there is no
"unsupported terminal" error anywhere.

## Protocols

| protocol | what it is | standing |
|---|---|---|
| `Kitty` | raw RGBA with alpha | the most capable |
| `Sixel` | DEC 1987, palette-banded | the most widely spoken |
| `ITerm2` | an inline PNG pinned to the panel's cell box | iTerm2 |

Encoders are hand-rolled and dependency-free; each is a thin layer over the
shared pixel panel.

## Detection: sniff and probe

Two tiers with different licenses:

- **Sniffing** reads environment variables — free, instant, wrong only by
  omission. It may run anywhere.
- **Probing** asks the terminal itself over one raw-mode `/dev/tty` round
  trip: the kitty graphics query, XTVERSION, XTSMGRAPHICS, and `CSI 16 t`
  for the cell size, with DA1 as the ordering barrier. Ground truth that
  survives ssh, about 100 ms, once per process — and licensed only where
  writing escapes is safe: the actual output destination is a tty, no
  tmux or screen in between, `TERM` not dumb.

`Capabilities::detect_for(&destination)` keys the probe decision to the
stream that will receive the plot; `detect()` is the stdout convenience.
An unanswered probe is not evidence — it degrades to the sniff answer. The
value records which tier answered (`Source::Probed` or `Source::Sniffed`).

## Rough edges

- A multiplexer (tmux, screen) blocks probing by design; sniffed answers
  still apply, and cells always work.
- Font coverage is irrelevant here, but cell size in device pixels matters
  for sharpness; when the terminal will not report it, a documented
  fallback size is used.
- The first probe is a real terminal round trip; render paths themselves
  never write queries.

The design argument is
[Degradation is the contract](principles/degradation-is-the-contract.md)
and [The frame is run state](principles/frame-is-run-state.md); vocabulary
is in [terminology](terminology.md) under Graphics, Capabilities, and
Protocol.
