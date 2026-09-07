export { engineVersion, init, type Panel } from "./engine.js";
export {
  Color,
  Colormap,
  Palette,
  canonicalizeColor,
  type Charset,
  type Color as ColorValue,
  type ColorMode,
  type ColormapJSON,
  type NamedColor,
  type PaletteJSON,
  type Reducer,
  type ScaleJSON,
  Theme,
} from "./color.js";
export { Mapping, Viewport } from "./mapping.js";
export { MalevichError, type MalevichErrorCode } from "./error.js";
export { Frame, type FrameJSON } from "./frame.js";
export {
  Area,
  Bars,
  Cells,
  Line,
  Points,
  Range,
  Rule,
  Text,
  type Align,
  type Dash,
  type LineStyle,
  type PointStyle,
} from "./mark.js";
export { Grid, Plot, type DocumentJSON, type PlotSpec, type ViewportWindows } from "./plot.js";
export { Raster, type RasterCell } from "./raster.js";
export { toFloat64, type Gap, type SeriesLike } from "./series.js";
export {
  bar,
  boxPlot,
  density,
  describe,
  ecdf,
  errorBars,
  heatmap,
  hist,
  hist2d,
  histWith,
  line,
  scatter,
  stairs,
  table,
  tableWith,
  trend,
  violin,
} from "./presets.js";
