# Changelog

The Python rim is versioned independently of the Rust crate until the API is
1.x. The renderer underneath is whatever `supremus.ENGINE_VERSION` reports.

## Unreleased

## 0.1.0 — 2026-10-18

First public PyPI release. A PyO3 rim around malevich 1.24: the eight-mark
grammar, every preset the crate documents, shared goldens against the crate,
and the Python hosts a chart shows up in. One `abi3` wheel per platform, no
runtime dependencies, Python 3.11+.

- Marks: `Line`, `Points`, `Bars`, `Area`, `Cells`, `Range`, `Rule`, `Text`,
  with every channel the crate's serde carries — including `Bars.horizontal`,
  `Rule.h_span` / `v_span`, and `Line.grade`, which the JS rim does not expose
  yet. `Plot` with domains, one-sided bounds, scales, units, `axes(False)`,
  palette, colorbar, viewport. `Grid`.
- Presets: `line`, `scatter`, `bar`, `sparkline`, `stairs`, `hist`, `ecdf`,
  `density`, `trend`, `hist2d`, `heatmap`, `contour`, `contourf`, `box_plot`,
  `violin`, `describe`, `table`, `error_bars`, `error_bars_asymmetric`,
  `quiver`. The statistics run in the engine; keyword arguments are the
  fields of the crate's `_with` options.
- Series: lists (`None` is a gap, extracted in Rust in one pass),
  `array('d')`, numpy, and anything with `to_numpy()` (pandas, polars,
  pyarrow). Datetimes become unix seconds and the x axis becomes `Time`.
- Hosts: `print(plot)` and `repr` detect a frame; `_repr_mimebundle_`,
  `_repr_html_`, and `_mime_` show the HTML card in Jupyter, VS Code, and
  marimo; `__rich_console__` yields segments from the cell grid;
  `supremus.textual.PlotWidget` with `PlotState` (the ratatui and Ink gesture
  grammar), `link_x`; a pandas plotting backend (`plotting.backend =
  "supremus"`); `uvx supremus` as a tour and stdin plotter.
- `Raster`, `Mapping`, `Viewport`, `Frame.detect` (the crate's ladder, read
  from the Python process), `detect_graphics` and `render_best` for pixel
  terminals, `to_svg`, `to_svg_pixels`.
- Measured on an Apple M1 Pro: a 501 KB wheel, 1 ms import, ten million
  points in 37 ms with the GIL released.
