export {
  engineVersion,
  viewportAuto,
  type JsMapping,
  type JsViewport,
  type Panel,
} from "./engine.js";
export {
  Color,
  canonicalizeColor,
  type Charset,
  type Color as ColorValue,
  type ColorMode,
  type NamedColor,
  type ScaleJSON,
  Theme,
} from "./color.js";
export { MalevichError, type MalevichErrorCode } from "./error.js";
export { Frame, type FrameJSON } from "./frame.js";
export {
  Area,
  Bars,
  Line,
  Points,
  Rule,
  Text,
  type Align,
  type Dash,
  type LineStyle,
  type PointStyle,
} from "./mark.js";
export { Grid, Plot, type DocumentJSON, type PlotSpec, type Viewport } from "./plot.js";
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
  trend,
  violin,
} from "./presets.js";
