# Notebooks

Malevich runs in [Evcxr](https://github.com/evcxr/evcxr), the Rust Jupyter
kernel and REPL. There is no wrapper API. The `evcxr` feature adds rich cell
output. Everything else is the ordinary crate.

```sh
cargo install --locked evcxr_jupyter
evcxr_jupyter --install
```

First cell:

```rust
:dep malevich = { version = "1.23", features = ["evcxr"] }
use malevich::{Line, Plot};
```

End a cell with a `Plot` and Evcxr renders it:

```rust
let values = [1.0, 5.0, 2.0, 8.0];
Plot::new().layer(Line::y(&values[..])).title("training")
```

## What a cell shows

The chart arrives as a self-contained HTML terminal card: the exact cell
grid malevich would print to a terminal, as a `<pre>` with colored spans.
Quadrants and box-drawing stay crisp. Mark colors become RGB spans. Chrome
follows the card foreground, and plot text is HTML-escaped. The default
frame is 100×26 quadrants on the dark card. `Theme::LIGHT` selects the
light card. The same card travels as `image/svg+xml` in the bundle, so an
export that cannot carry HTML — nbconvert to PDF — still shows the chart.
JupyterLab itself prefers the HTML.

HTML rather than an image is a consequence of the design, not a shortcut.
malevich owns no font rasterizer, so it hands text drawing to the browser,
the same offload it makes to the terminal. The SVG card keeps that rule.
Block glyphs become the rectangles they denote. Everything else is text the
host's font draws. The adapter adds no dependency.

Quadrants are the default for the same reason they are the terminal
default: a notebook's monospace font is a gamble, and 2×2 blocks plus
box-drawing are in virtually every one. Denser tiers are one frame away.

## Custom frames

`Plot::to_html(&frame)` and `Plot::to_svg(&frame)` are the pure,
deterministic paths, snapshot-testable like every render path. They need
no feature, because a notebook, a README, or a static page is one more
terminal:

```rust
plot.to_html(&Frame::portable(120, 30))
plot.to_svg(&Frame::portable(120, 30))
```

With the `pixel` feature, `Plot::to_svg_pixels(&frame, &graphics)` is the
hybrid pixel render for the same kind of host: chrome stays this card, and the
panel is the device-pixel raster. See [pixels.md](pixels.md#on-a-page).

Redirect `cargo run --example evcxr > plot.html` for a standalone fragment
you can inspect in a browser, or `cargo run --example speedup -- --svg >
plot.svg` for the SVG card. That is the one the README embeds as an image,
because GitHub strips the HTML card's styles.

## The terminal-card contract

What a host may rely on, for the HTML card and the SVG cell card
(`Plot::to_html`, `Plot::to_svg`) alike:

- **The grid is the chart.** A card is the exact cell grid the terminal
  renderer would print for the same plot and frame. One `<pre>` (HTML) or
  one `<text>`-and-`<rect>` group (SVG), one cell per character, box-drawing
  and block glyphs included. Byte-identical for equal inputs. A snapshot
  test pins it.
- **Nothing external.** No stylesheet, script, font file, or image reference
  leaves the card. Colors are inline on spans. The font is whatever
  monospace the host has. A card renders the same in a static page, a
  README, a notebook, or a mail client.
- **Stripped styles still read.** Sanitizing hosts drop inline styles:
  GitHub's markup, nbconvert with `sanitize_html`, nbviewer's bleach pass.
  The HTML card then loses its colors and keeps every glyph in place, so
  the chart survives as the plain text it also is. Where color must survive,
  use the SVG card. Hosts that strip styles keep images, and the README's
  figures are SVG for that reason.
- **Plain text is agent-legible.** The same grid, colorless, is what
  `Frame::plain` renders and what a language model or a log reader sees:
  an axis with labels, marks in rows. No card carries information that the
  plain grid lacks except color, and color never carries a value alone.

## The terminal REPL

The same cell renders in the `evcxr` terminal REPL through a `text/plain`
fallback: a plain 80×24 plot. With the `pixel` feature also enabled, that
fallback upgrades itself. The REPL is a real terminal, so the plot arrives
as a sixel, kitty, or iTerm2 image where one is spoken.

## For other crates

A crate that renders its own types beside malevich charts can join the same
card. `malevich::evcxr::card_colors(theme)` returns the exact background
and foreground `to_html` paints with. `malevich::evcxr::mime_bundle` emits
the stdout protocol. Both are pure functions. Output built on them stays
snapshot-testable.

## Rough edges

- The kernel cannot know the notebook's width. The 100×26 default plus an
  explicit `Frame` is the honest interface.
- A cell is a real compile. The first `:dep` is the slow one. Evcxr's
  `:cache 500` helps.
- Views are static: no hover, no zoom. The terminal thesis, kept.
