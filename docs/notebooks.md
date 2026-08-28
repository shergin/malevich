# Notebooks

Malevich runs in [Evcxr](https://github.com/evcxr/evcxr), the Rust Jupyter
kernel and REPL. There is no wrapper API: the `evcxr` feature adds rich cell
output; everything else is the ordinary crate.

```sh
cargo install --locked evcxr_jupyter
evcxr_jupyter --install
```

First cell:

```rust
:dep malevich = { version = "1.18", features = ["evcxr"] }
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
Quadrants and box-drawing stay crisp, mark colors become RGB spans, chrome
follows the card foreground, and plot text is HTML-escaped. The default
frame is 100×26 quadrants on the dark card; `Theme::LIGHT` selects the
light card.

HTML rather than an image is a design consequence, not a shortcut: malevich
owns no font rasterizer, so it hands text drawing to the browser — the same
offload it makes to the terminal. The adapter adds no dependency.

Quadrants are the default for the same reason they are the terminal
default: a notebook's monospace font is a gamble, and 2×2 blocks plus
box-drawing are in virtually every one. Denser tiers are one frame away.

## Custom frames

`Plot::to_html(&frame)` is the pure, deterministic path — snapshot-testable
like every render path:

```rust
plot.to_html(&Frame::portable(120, 30))
```

Redirect `cargo run --example evcxr --features evcxr > plot.html` for a
standalone fragment you can inspect in a browser.

## The terminal REPL

The same cell renders in the `evcxr` terminal REPL through a `text/plain`
fallback — a plain 80×24 plot. With the `pixel` feature also enabled, that
fallback upgrades itself: the REPL being a real terminal, the plot arrives
as a sixel, kitty, or iTerm2 image where one is spoken.

## For other crates

A crate that renders its own types beside malevich charts can join the same
card: `malevich::evcxr::card_colors(theme)` returns the exact background
and foreground `to_html` paints with, and `malevich::evcxr::mime_bundle`
emits the stdout protocol. Both are pure functions; output built on them
stays snapshot-testable.

## Rough edges

- The kernel cannot know the notebook's width; the 100×26 default plus an
  explicit `Frame` is the honest interface.
- A cell is a real compile; the first `:dep` is the slow one. Evcxr's
  `:cache 500` helps.
- Views are static: no hover, no zoom. The terminal thesis, kept.
