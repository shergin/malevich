# Interaction

How a chart becomes interactive: the physics the core exposes, the
controller the ratatui widget runs on top of it, and the patterns that need
no library support at all. The boundary is strict — malevich never reads
input, never owns an event loop — and everything here is what that boundary
*enables*.

## The three layers

| layer | owns | lives in |
|---|---|---|
| physics | cell ↔ data mapping, window arithmetic, snapping lookups | core: `Mapping`, `Viewport`, `stat::nearest` |
| controller | gesture policy, hover/drag state, overlay drawing | the widget: `PlotState` (feature `ratatui`) |
| host | event loop, mouse capture, key bindings, app state | your code |

The controller is to interaction what presets are to the grammar: a proven
composition of the public physics. Different policy wanted? Skip `on_mouse`
and drive `Viewport` and `Mapping` yourself — the same escape hatch.

## The physics

`Plot::mapping(&frame)` runs the same resolve → layout pass rendering runs
and returns where everything landed, as a plain value:

- `data_at(column, row)` / `cell_at(x, y)` — the scale contract's `invert`,
  reachable at plot level. Band axes answer in band-index space, time axes
  in unix seconds.
- `x_span_at(column)` — the data interval one column covers; a cell-level
  cursor's honest resolution.
- `format_x` / `format_y` — a value written the way the axis writes its own
  labels: exact decimals at cell resolution, calendar instants, category
  names. Never `0.30000000000000004`, never more precision than a cell has.
- `x_domain` / `y_domain` — the resolved windows, and `viewport()` — those
  windows as a `Viewport`, the seed for zoom and pan.

`Viewport` is the view as a value: `zoom_x(factor, anchor)` (decade space on
log axes, so equal gestures cover equal factors), `pan_x(fraction)`,
`clamp_x(extent)`, `tail(latest, width)`, `reset()`. Applied with
`Plot::viewport(&view)` — pure sugar over `x_domain`/`y_domain`, which is
the load-bearing trick: **a zoom is a scale option, not a render mode**, so
M4 re-aggregates to the visible window on the next render and drilling into
ten million points is just rendering (`cargo run --release --example zoom
--features ratatui`).

## The controller

```rust
let mut chart = PlotState::default();                    // host state, once
frame.render_stateful_widget(plot.widget(), area, &mut chart);
// in the event loop:
if let Event::Mouse(raw) = event && let Some(input) = mouse(raw) {
    chart.on_mouse(input);                               // returns "changed?"
}
```

The stateful render caches the frame's `Mapping` (hit-testing answers
against exactly what is on screen), applies the state's `Viewport`, and
draws the interaction chrome. `mouse` is a six-line match from your
backend's event type to the neutral `Mouse` vocabulary — printed in full in
the `Mouse` rustdoc; mouse *capture* is yours to enable
(`EnableMouseCapture` in crossterm).

The gesture grammar, fixed on purpose:

| input | effect |
|---|---|
| hover | crosshair; the readout snaps to the data (below) |
| wheel | x zoom anchored at the data under the cursor |
| left drag | pan, every continuous axis |
| right drag | rubber-band selection; zooms to it on release |
| `reset_view()` / `zoom_in()` / `zoom_out()` | for the host's key bindings |

Coordinates outside the plot rectangle are ignored; bands axes have no
continuous window and stay untouched.

**Snapping.** For every point-backed `Line` and `Points` layer, the readout
lists the datum nearest the cursor's x inside the visible window —
`label: value`, axis-formatted, its cell highlighted. A gap reads as `—`,
never an interpolation; off-window data never snaps. `snap(false)` returns
plain cursor coordinates; `crosshair(false)` and `readout(false)` suppress
the other overlays. Overlays draw into the buffer only — the plot value
renders byte-identically with or without them.

## Patterns that need no API

**Linked panes.** Two stacked charts share one x view by mirroring the
window after each event — route the event to the pane it landed on, then:

```rust
let window = active.viewport().x();
let view = passive.viewport();
passive.set_viewport(match window {
    Some((lo, hi)) => view.with_x(lo, hi),
    None => view.reset_x(),
});
```

Zoom and pan in either pane and both move; each pane keeps its own y. fred's
series view does exactly this — a year-over-year context strip under the
main chart, linked on x. The absence of a "linking" feature is the design:
a view is a value, so sharing it is assignment.

**Selection → statistics.** A rubber-band zoom *is* a selection: after it,
`viewport().x()` is the chosen window, and the visible data summarizes with
the ordinary stat vocabulary —

```rust
if let Some((lo, hi)) = chart.viewport().x() {
    let mut visible = Moments::new();
    for (&x, &y) in xs.iter().zip(ys) {
        if (lo..=hi).contains(&x) { visible.add(y); }
    }
    // count / mean / sd / min / max, formatted by the axis:
    let label = chart.mapping().map(|m| m.format_x(lo));
}
```

fred's footer shows this: zoom into any span and the stats line describes
what is on screen, dated by `format_x`.

**Follow the stream.** A live chart tails its ring buffer in one line —
`view.tail(latest_x, width)` — and a user's zoom naturally suspends the
follow until `reset()`.

## What stays out

The widget never reads the terminal; the core never sees input; there is no
gesture configuration surface (different policy = drive the physics
directly); no animation — time belongs to the host's loop. These are the
boundaries that keep a plot a value.
