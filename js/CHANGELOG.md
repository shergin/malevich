# Changelog

The JS rim is versioned independently of the Rust crate until the API is 1.x.
The renderer underneath is whatever `engineVersion` reports.

## Unreleased

## 0.2.0 — 2026-09-06

Ink feature parity with the ratatui widget. A chart becomes an instrument
without malevich ever touching input: `PlotState` is the same controller,
`PlotWidget` draws the same chrome, and the host still owns the event loop.

- `PlotState` runs the default gesture grammar: hover crosshair, wheel x-zoom
  at the cursor, left-drag pan, right-drag rubber-band zoom, keyboard
  `zoomIn` / `zoomOut` / `panLeft` / `panRight` / `resetView`. Band axes stay
  put. Coordinates outside the plot rectangle are ignored. `hoverX` mirrors a
  linked pane's cursor as a vertical-only crosshair; `linkX` shares the x
  window.
- `PlotWidget` applies the state's viewport, caches the mapping, and paints
  overlays into the raster: crosshair (unset backgrounds only), snap
  highlights, rubber-band band, axis-formatted readout (`label: value`, gaps
  as `—`). `crosshair`, `readout`, `snap` props suppress each layer. `origin`
  is the widget's terminal-cell position so stacked panes hit-test.
- `usePlotInteraction` is an optional composition: DECSET mouse tracking on
  Ink's stdout, SGR parse from Ink's stdin, the default key bindings. Skip it
  and feed `onMouse` yourself. `enableMouse` / `parseMouse` are public.
- Mapping grows `xCategories` / `yCategories`. Viewport grows `withX` /
  `withY` / `resetX` / `resetY` / `clampX` / `clampY` / `tail` / `isAuto`.
  `Plot.viewport` takes a plain `{ x?, y? }` window pair.

## 0.1.0 — 2026-09-06

First public npm. WASM-backed rim: presets, the mark grammar, `Frame.detect`,
`Plot.raster`, a fire-and-forget Ink widget. ESM only.
