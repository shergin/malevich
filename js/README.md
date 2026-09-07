# malevich

**Terminal plotting for JavaScript: a small grammar of marks, honest axes,
millions of points.**

The engine is the [Rust crate](https://crates.io/crates/malevich) 1.x, compiled
to WASM. A plot is a value. Rendering is a pure function of that value and a
frame. The library never owns the terminal. Zero native dependencies.

![Loss curves, a calendar time axis, and smoothing](https://raw.githubusercontent.com/shergin/malevich/main/examples/showcase-lines.png)

```sh
npm install malevich
npx malevich              # a tour, sized to your terminal
printf '1 5 2 8' | npx malevich line
```

ESM only (Node 18+, Bun, Deno). `require()` is not exported.

```js
import { line } from "malevich";

console.log(line([1, 5, 2, 8]));
```

```text
7.5 ┤                                 ⡠⠊
    │                               ⡠⠊
    │                             ⡠⠊
5.0 ┤         ⣀⠔⠊⠑⠢⢄⡀           ⡠⠊
    │      ⣀⠔⠊      ⠈⠑⠢⢄⡀     ⡠⠊
2.5 ┤   ⡠⠔⠉             ⠈⠑⠢⢄⡠⠊
    │⡠⠔⠉
0.0 ┤
    └┬──────────┬───────────┬──────────┬
     0          1           2          3
```

```js
import { Frame, Line, Plot } from "malevich";

const loss = [4, 2.8, 1.9, 1.2, 0.8, 0.6];
const chart = new Plot()
  .layer(Line.y(loss).label("loss").style("Corners"))
  .title("training");
console.log(chart.render(Frame.detect()));
```

`console.log(plot)` and `String(plot)` detect a frame. `plot.render(frame)` is
the pure path — pass `Frame.plain` or `Frame.portable` in tests.

This is **0.x**: the JS API can still move. The renderer is the 1.x Rust crate
(`engineVersion`).

## Why malevich

The JS terminal already has charts. What it does not have is this engine.

- **A small grammar, not a chart zoo.** Eight marks (line, points, bars, area,
  cells, range, rule, text) × a stats layer × shared scales compose into the
  basic catalog. Presets (`line`, `hist`, `boxPlot`, `violin`, `trend`, …) are
  packaging: the same plot you would write by hand.
- **Axes that are actually good.** Extended-Wilkinson tick placement, exact
  decimal labels that parse back to their values, one SI prefix per axis, log
  axes, calendar time, band axes. Never `0.30000000000000004`.
- **Millions of points.** Large lines reduce by M4, bucketed by the rendered
  column — pixel-identical to drawing every point. Ten million points is a
  CLI one-shot, not a 50 fps zoom loop in WASM; still tens of milliseconds
  for typical sizes.
- **Renders everywhere, honestly.** Charset and color ladders from Unicode 16
  octants down to plain ASCII, truecolor down to a clean pipe. `NaN` is always
  a visible gap.
- **A plot is a value.** Immutable builders, no hidden terminal state.
  `Plot.render` / `Plot.raster` inspect nothing. Detection lives in
  `Frame.detect`.

The design is argued in the crate's [docs/vision.md](https://github.com/shergin/malevich/blob/main/docs/vision.md).
This package is the JS rim around that crate — one oracle, not a rewrite.

## Presets and the grammar

```js
import {
  Area, Frame, Grid, Line, Plot, Points, Rule, Text,
  bar, boxPlot, density, describe, ecdf, heatmap, hist, hist2d,
  line, scatter, stairs, table, trend, violin,
} from "malevich";

console.log(bar(["mon", "tue", "wed", "thu", "fri"], [3, 7, 4.5, 8, 6]));
console.log(hist(samples));
console.log(boxPlot(["train", "val"], [trainLoss, valLoss]));
console.log(describe(["train", "val"], [trainLoss, valLoss]));
```

Eight marks: `Line`, `Points`, `Bars`, `Area`, `Cells`, `Range`, `Rule`,
`Text`. A preset is a proven composition. Shared goldens prove JS output is
byte-identical to the crate for the same document and frame.

```js
import { Bars, Cells, Colormap, Range } from "malevich";

new Plot().layer(Cells.matrix(4, values).colormap(Colormap.VIRIDIS));
new Plot().layer(Range.over(["a", "b"], low, high).body(q1, q3).marker(median));
new Plot()
  .layer(Bars.new(["a", "b"], lower).label("a"))
  .layer(Bars.new(["a", "b"], upper).base(lower).label("b"));
```

```js
const chart = new Plot()
  .layer(Line.y(train).label("train").color("Cyan"))
  .layer(Line.y(val).label("val").color("Yellow"))
  .layer(Rule.h(0.5).label("target").dash("Dotted"))
  .layer(Text.at(80, 3.2, "overfit?"))
  .title("loss")
  .xLabel("step")
  .yLabel("loss");
```

`null` / `undefined` in a series become gaps (`NaN`). `Float64Array` is kept
by reference. Nested `{x, y}[]` is not a series — a mark that wants two
channels takes two series (`Line.xy(x, y)`, `scatter(x, y)`).

## Frame

Where and how to render: size, charset, color, theme. Frame is run state, not
plot state — the same plot renders into many frames.

| constructor | what |
|---|---|
| `Frame.detect()` | reads `process.env` and the stream: size, `NO_COLOR`, `COLORTERM`, `TERM`, `MALEVICH_CHARSET`, `COLORFGBG` |
| `Frame.plain(w, h)` | braille, no color — the snapshot form |
| `Frame.portable(w, h)` | quadrants, no color — conservative Unicode |
| `frame.with({ height: 8 })` | copy with fields replaced |

Wasm never reads the environment. Snapshot tests always pass an explicit frame.

## Raster

`plot.render(frame)` is a string. `plot.raster(frame)` is the cell grid
underneath — glyphs and colors, chrome included — so a TUI host can paint
cells instead of decoding ANSI.

```js
const raster = chart.raster(Frame.plain(40, 10));
for (const row of raster.rows()) {
  // row: { glyph, foreground, background }[]
}
```

Continuation cells (`columns === 0`) sit to the right of a wide glyph; `rows()`
skips them.

## Mapping and viewport

`plot.mapping(frame)` is the resolved geometry of one render: cell ↔ data both
ways, axis-formatted labels, the plot rectangle. `Viewport` is a pair of
optional axis windows — a zoom is a scale option, not a render mode, so M4
re-aggregates to the visible window on the next render.

```js
const mapping = chart.mapping(frame);
const data = mapping.dataAt(column, row);      // [x, y] or undefined
const view = mapping.viewport().zoomX(0.8, data[0]);
console.log(chart.viewport(view.windows()).render(frame));
```

Hosts that want different gestures than the Ink widget drive this physics
directly.

## Ink

Optional. Install `ink` and `react`, then:

```js
import { line } from "malevich";
import { PlotWidget } from "malevich/ink";

<PlotWidget plot={line(loss)} width={80} height={16} />
```

That is fire-and-forget. For interaction, keep a `PlotState` in a ref — the
same controller as the ratatui widget — and feed it mouse coordinates. The
widget never reads the terminal.

```js
import { useRef, useState } from "react";
import { line } from "malevich";
import { PlotState, PlotWidget, usePlotInteraction } from "malevich/ink";

function Chart({ loss }) {
  const state = useRef(new PlotState()).current;
  const [, bump] = useState(0);
  usePlotInteraction(state, { onChange: () => bump((n) => n + 1) });
  return <PlotWidget plot={line(loss).title("training")} state={state} width={80} height={16} />;
}
```

The gesture grammar, fixed on purpose:

| input | effect |
|---|---|
| hover | crosshair; the readout snaps to the data |
| wheel | x zoom anchored at the data under the cursor |
| left drag | pan, every continuous axis |
| right drag | rubber-band selection; zooms to it on release |
| `+` / `-` / arrows / `r` | zoom, pan, reset (`usePlotInteraction` keys) |

Coordinates outside the plot rectangle are ignored. Band axes have no
continuous window and stay untouched. A gap at the snapped x reads as `—`,
never an interpolation. `crosshair={false}`, `readout={false}`, `snap={false}`
suppress the overlays. Overlays draw into the cells only — the plot value
renders identically with or without them.

`usePlotInteraction` is a proven composition: it enables DECSET mouse tracking
on Ink's stdout and parses SGR from Ink's stdin. One hook per app — for two
panes, parse at the app level and route (see `examples/ink-linked.tsx`). Skip
the hook and drive `PlotState.onMouse` yourself if you want different policy.
Enable and parse helpers (`enableMouse`, `parseMouse`, `linkX`) are public.

**Linked panes.** Two stacked charts share an x view by assignment, not by
feature. Route the event to the pane it landed on, then:

```js
import { linkX } from "malevich/ink";

linkX(active, passive);   // share the x window; mirror the cursor
```

Each pane keeps its own y. The passive pane draws a vertical-only crosshair at
*its* column for that x (`mapping.columnAt`), snaps its own series, and reads
out the same instant.

Wrap stacked panes in `PlotColumn` and each `PlotWidget` gets `origin` from
the heights above it — no manual row math. Pass `origin` yourself only when
the layout is not a column.

```js
import { PlotColumn, PlotWidget } from "malevich/ink";

<PlotColumn>
  <Text>header</Text>
  <PlotWidget plot={main} state={mainState} height={16} />
  <PlotWidget plot={ctx} state={ctxState} height={8} />
</PlotColumn>
```

A live tour: `npx tsx examples/ink-zoom.tsx` in this repo (two million points,
wheel-zoom into any spike). Linked panes: `npx tsx examples/ink-linked.tsx`.

Pixels, when the terminal speaks them:

```js
console.log(chart.renderBest(Frame.detect()));           // sniff env, then cells or image
console.log(chart.renderPixels(frame, { protocol: "kitty" }));
```

Detection stays in JS. The wasm path is pure over the protocol you name.

## What it will not be

Not a TUI framework (it never owns the terminal or handles input). No
animations. No file parsing or dataframes — conversion happens once, at the
rim, into `Float64Array` (`NaN` = gap). No config-object kitchen sink. Not a
browser charting library — `renderBest` is still a terminal string.

## Engine

| | |
|---|---|
| npm | `malevich` 0.x |
| crate | [`malevich`](https://crates.io/crates/malevich) 1.x (`engineVersion`) |
| artifact | `malevich_js_bg.wasm` (~256 KB gzipped) |
| license | MIT or Apache-2.0 |

## More

A full colored tour, sized to your terminal:

```sh
cd js && npm run build && npm run showcase
```

Examples live in [`examples/`](examples/). Crate docs: [interaction](https://github.com/shergin/malevich/blob/main/docs/interaction.md),
[terminology](https://github.com/shergin/malevich/blob/main/docs/terminology.md),
[vision](https://github.com/shergin/malevich/blob/main/docs/vision.md).

## License

MIT or Apache-2.0.

## Build from this repo

```sh
cd js
npm install
npm run build
npm test
node examples/hello.mjs
npm run showcase
```

Requires a Rust toolchain with `wasm32-unknown-unknown` and `wasm-pack`.
