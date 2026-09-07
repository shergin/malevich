import {
  Color,
  Colormap,
  canonicalizeColor,
  colorToJSON,
  type ColormapJSON,
  type Reducer,
} from "./color.js";
import { type SeriesLike, toFloat64 } from "./series.js";

export type LineStyle = "Pixels" | "Corners";
export type Dash = "Solid" | "Dashed" | "Dotted";
export type PointStyle = "Dot" | "Plus" | "Cross" | "Asterisk" | "Circle";

export type LayerJSON = Record<string, unknown>;

export type LayerPayload = {
  json: LayerJSON;
  columns: Float64Array[];
};

export abstract class Mark {
  abstract toLayer(columnStart: number): LayerPayload;
}

export class Line extends Mark {
  private x: Float64Array | undefined;
  private y: Float64Array;
  private colorValue: Color | null = null;
  private labelValue: string | null = null;
  private styleValue: LineStyle = "Pixels";
  private dashValue: Dash = "Solid";
  private glowValue = false;
  private colorByValue: string[] | undefined;

  private constructor(y: Float64Array, x?: Float64Array) {
    super();
    this.y = y;
    this.x = x;
  }

  static y(values: SeriesLike): Line {
    return new Line(toFloat64(values));
  }

  static xy(x: SeriesLike, y: SeriesLike): Line {
    const xs = toFloat64(x);
    const ys = toFloat64(y);
    if (xs.length !== ys.length) {
      throw new TypeError(
        `Line: x and y: channels differ in length (${xs.length} and ${ys.length})`,
      );
    }
    return new Line(ys, xs);
  }

  style(style: LineStyle): Line {
    const next = this.clone();
    next.styleValue = style;
    return next;
  }

  color(color: string | Color): Line {
    const next = this.clone();
    next.colorValue = canonicalizeColor(color);
    return next;
  }

  colorBy(categories: Iterable<string>): Line {
    const next = this.clone();
    next.colorByValue = [...categories];
    return next;
  }

  label(label: string): Line {
    const next = this.clone();
    next.labelValue = label;
    return next;
  }

  dash(dash: Dash): Line {
    const next = this.clone();
    next.dashValue = dash;
    return next;
  }

  glow(): Line {
    const next = this.clone();
    next.glowValue = true;
    return next;
  }

  toLayer(columnStart: number): LayerPayload {
    const columns: Float64Array[] = [];
    let x: unknown = null;
    let y: unknown;
    let index = columnStart;
    if (this.x) {
      x = { col: index };
      columns.push(this.x);
      index += 1;
    }
    y = { col: index };
    columns.push(this.y);
    const body: Record<string, unknown> = {
      x,
      y,
      color: colorToJSON(this.colorValue),
      label: this.labelValue,
      style: this.styleValue,
    };
    if (this.colorByValue) {
      body.color_by = this.colorByValue;
    }
    if (this.glowValue) {
      body.glow = true;
    }
    if (this.dashValue !== "Solid") {
      body.dash = this.dashValue;
    }
    return { json: { Line: body }, columns };
  }

  private clone(): Line {
    const next = new Line(this.y, this.x);
    next.colorValue = this.colorValue;
    next.labelValue = this.labelValue;
    next.styleValue = this.styleValue;
    next.dashValue = this.dashValue;
    next.glowValue = this.glowValue;
    next.colorByValue = this.colorByValue;
    return next;
  }
}

export class Points extends Mark {
  private x: Float64Array | undefined;
  private y: Float64Array;
  private colorValue: Color | null = null;
  private labelValue: string | null = null;
  private styleValue: PointStyle = "Dot";
  private colorByValue: string[] | undefined;
  private opacityValue: number | undefined;
  private densityValue = false;

  private constructor(y: Float64Array, x?: Float64Array) {
    super();
    this.y = y;
    this.x = x;
  }

  static y(values: SeriesLike): Points {
    return new Points(toFloat64(values));
  }

  static xy(x: SeriesLike, y: SeriesLike): Points {
    const xs = toFloat64(x);
    const ys = toFloat64(y);
    if (xs.length !== ys.length) {
      throw new TypeError(
        `Points: x and y: channels differ in length (${xs.length} and ${ys.length})`,
      );
    }
    return new Points(ys, xs);
  }

  style(style: PointStyle): Points {
    const next = this.clone();
    next.styleValue = style;
    return next;
  }

  color(color: string | Color): Points {
    const next = this.clone();
    next.colorValue = canonicalizeColor(color);
    return next;
  }

  colorBy(categories: Iterable<string>): Points {
    const next = this.clone();
    next.colorByValue = [...categories];
    return next;
  }

  label(label: string): Points {
    const next = this.clone();
    next.labelValue = label;
    return next;
  }

  opacity(opacity: number): Points {
    if (!Number.isFinite(opacity) || opacity <= 0 || opacity > 1) {
      throw new TypeError("Points.opacity requires a value in (0, 1]");
    }
    const next = this.clone();
    next.opacityValue = opacity;
    return next;
  }

  density(): Points {
    const next = this.clone();
    next.densityValue = true;
    return next;
  }

  toLayer(columnStart: number): LayerPayload {
    const columns: Float64Array[] = [];
    let index = columnStart;
    let x: unknown = null;
    if (this.x) {
      x = { col: index };
      columns.push(this.x);
      index += 1;
    }
    columns.push(this.y);
    const body: Record<string, unknown> = {
      x,
      y: { col: index },
      color: colorToJSON(this.colorValue),
      label: this.labelValue,
      style: this.styleValue,
    };
    if (this.colorByValue) {
      body.color_by = this.colorByValue;
    }
    if (this.opacityValue !== undefined) {
      body.opacity = this.opacityValue;
    }
    if (this.densityValue) {
      body.density = true;
    }
    return { json: { Points: body }, columns };
  }

  private clone(): Points {
    const next = new Points(this.y, this.x);
    next.colorValue = this.colorValue;
    next.labelValue = this.labelValue;
    next.styleValue = this.styleValue;
    next.colorByValue = this.colorByValue;
    next.opacityValue = this.opacityValue;
    next.densityValue = this.densityValue;
    return next;
  }
}

type BarsPlacement =
  | { kind: "bands"; categories: string[] }
  | { kind: "spans"; start: number; width: number }
  | { kind: "at"; x: Float64Array; width: number };

export class Bars extends Mark {
  private placement: BarsPlacement;
  private values: Float64Array;
  private baseValue: Float64Array | undefined;
  private colorValue: Color | null = null;
  private labelValue: string | null = null;
  private colorByValue: string[] | undefined;

  private constructor(placement: BarsPlacement, values: Float64Array) {
    super();
    this.placement = placement;
    this.values = values;
  }

  static new(categories: Iterable<string>, values: SeriesLike): Bars {
    const names = [...categories];
    const series = toFloat64(values);
    if (names.length !== series.length) {
      throw new TypeError(
        `Bars: categories and values: channels differ in length (${names.length} and ${series.length})`,
      );
    }
    return new Bars({ kind: "bands", categories: names }, series);
  }

  /** Histogram shape: bar `i` covers `[start + i * width, start + (i + 1) * width)`. */
  static spans(start: number, width: number, values: SeriesLike): Bars {
    if (!Number.isFinite(start) || !Number.isFinite(width) || width <= 0) {
      throw new TypeError("Bars.spans requires a finite start and a positive width");
    }
    return new Bars({ kind: "spans", start, width }, toFloat64(values));
  }

  /** Bars centered at numeric positions with a shared width. */
  static at(x: SeriesLike, width: number, values: SeriesLike): Bars {
    if (!Number.isFinite(width) || width <= 0) {
      throw new TypeError("Bars.at requires a finite positive width");
    }
    const xs = toFloat64(x);
    const series = toFloat64(values);
    if (xs.length !== series.length) {
      throw new TypeError(
        `Bars: x and values: channels differ in length (${xs.length} and ${series.length})`,
      );
    }
    return new Bars({ kind: "at", x: xs, width }, series);
  }

  color(color: string | Color): Bars {
    const next = this.clone();
    next.colorValue = canonicalizeColor(color);
    return next;
  }

  colorBy(categories: Iterable<string>): Bars {
    const next = this.clone();
    next.colorByValue = [...categories];
    return next;
  }

  label(label: string): Bars {
    const next = this.clone();
    next.labelValue = label;
    return next;
  }

  /** Starts each bar at a per-bar base: bar `i` spans `base[i] .. base[i] + value[i]`. */
  base(base: SeriesLike): Bars {
    const series = toFloat64(base);
    if (series.length !== this.values.length) {
      throw new TypeError(
        `Bars: base and values: channels differ in length (${series.length} and ${this.values.length})`,
      );
    }
    const next = this.clone();
    next.baseValue = series;
    return next;
  }

  toLayer(columnStart: number): LayerPayload {
    const columns: Float64Array[] = [];
    let index = columnStart;
    let placement: unknown;
    if (this.placement.kind === "bands") {
      placement = { Bands: this.placement.categories };
    } else if (this.placement.kind === "spans") {
      placement = {
        Spans: { start: this.placement.start, width: this.placement.width },
      };
    } else {
      placement = { At: { x: { col: index }, width: this.placement.width } };
      columns.push(this.placement.x);
      index += 1;
    }
    const values = { col: index };
    columns.push(this.values);
    index += 1;
    const body: Record<string, unknown> = {
      placement,
      values,
      color: colorToJSON(this.colorValue),
      label: this.labelValue,
    };
    if (this.baseValue) {
      body.base = { col: index };
      columns.push(this.baseValue);
    }
    if (this.colorByValue) {
      body.color_by = this.colorByValue;
    }
    return { json: { Bars: body }, columns };
  }

  private clone(): Bars {
    const next = new Bars(this.placement, this.values);
    next.baseValue = this.baseValue;
    next.colorValue = this.colorValue;
    next.labelValue = this.labelValue;
    next.colorByValue = this.colorByValue;
    return next;
  }
}

export type Align = "Left" | "Center" | "Right";

export class Rule extends Mark {
  private orientation: { Horizontal: number } | { Vertical: number };
  private colorValue: Color | null = null;
  private labelValue: string | null = null;
  private dashValue: Dash = "Solid";

  private constructor(orientation: { Horizontal: number } | { Vertical: number }) {
    super();
    this.orientation = orientation;
  }

  static h(y: number): Rule {
    if (!Number.isFinite(y)) {
      throw new TypeError("Rule.h requires a finite position");
    }
    return new Rule({ Horizontal: y });
  }

  static v(x: number): Rule {
    if (!Number.isFinite(x)) {
      throw new TypeError("Rule.v requires a finite position");
    }
    return new Rule({ Vertical: x });
  }

  color(color: string | Color): Rule {
    const next = this.clone();
    next.colorValue = canonicalizeColor(color);
    return next;
  }

  label(label: string): Rule {
    const next = this.clone();
    next.labelValue = label;
    return next;
  }

  dash(dash: Dash): Rule {
    const next = this.clone();
    next.dashValue = dash;
    return next;
  }

  toLayer(_columnStart: number): LayerPayload {
    const body: Record<string, unknown> = {
      orientation: this.orientation,
      color: colorToJSON(this.colorValue),
      label: this.labelValue,
    };
    if (this.dashValue !== "Solid") {
      body.dash = this.dashValue;
    }
    return { json: { Rule: body }, columns: [] };
  }

  private clone(): Rule {
    const next = new Rule(this.orientation);
    next.colorValue = this.colorValue;
    next.labelValue = this.labelValue;
    next.dashValue = this.dashValue;
    return next;
  }
}

export class Text extends Mark {
  private readonly x: number;
  private readonly y: number;
  private readonly textValue: string;
  private colorValue: Color | null = null;
  private alignValue: Align = "Left";

  private constructor(x: number, y: number, text: string) {
    super();
    this.x = x;
    this.y = y;
    this.textValue = text;
  }

  static at(x: number, y: number, text: string): Text {
    if (!Number.isFinite(x) || !Number.isFinite(y)) {
      throw new TypeError("Text.at requires a finite anchor");
    }
    return new Text(x, y, text);
  }

  color(color: string | Color): Text {
    const next = this.clone();
    next.colorValue = canonicalizeColor(color);
    return next;
  }

  align(align: Align): Text {
    const next = this.clone();
    next.alignValue = align;
    return next;
  }

  toLayer(_columnStart: number): LayerPayload {
    const body: Record<string, unknown> = {
      x: this.x,
      y: this.y,
      text: this.textValue,
      color: colorToJSON(this.colorValue),
    };
    if (this.alignValue !== "Left") {
      body.align = this.alignValue;
    }
    return { json: { Text: body }, columns: [] };
  }

  private clone(): Text {
    const next = new Text(this.x, this.y, this.textValue);
    next.colorValue = this.colorValue;
    next.alignValue = this.alignValue;
    return next;
  }
}

export class Area extends Mark {
  private x: Float64Array | undefined;
  private low: Float64Array | undefined;
  private high: Float64Array;
  private colorValue: Color | null = null;
  private labelValue: string | null = null;
  private opacityValue: number | undefined;

  private constructor(high: Float64Array, x?: Float64Array, low?: Float64Array) {
    super();
    this.high = high;
    this.x = x;
    this.low = low;
  }

  static y(values: SeriesLike): Area {
    return new Area(toFloat64(values));
  }

  static xy(x: SeriesLike, y: SeriesLike): Area {
    const xs = toFloat64(x);
    const ys = toFloat64(y);
    if (xs.length !== ys.length) {
      throw new TypeError(
        `Area: x and y: channels differ in length (${xs.length} and ${ys.length})`,
      );
    }
    return new Area(ys, xs);
  }

  static between(x: SeriesLike, low: SeriesLike, high: SeriesLike): Area {
    const xs = toFloat64(x);
    const lo = toFloat64(low);
    const hi = toFloat64(high);
    if (xs.length !== lo.length || xs.length !== hi.length) {
      throw new TypeError("Area.between requires series of equal length");
    }
    return new Area(hi, xs, lo);
  }

  color(color: string | Color): Area {
    const next = this.clone();
    next.colorValue = canonicalizeColor(color);
    return next;
  }

  label(label: string): Area {
    const next = this.clone();
    next.labelValue = label;
    return next;
  }

  opacity(opacity: number): Area {
    if (!Number.isFinite(opacity) || opacity <= 0 || opacity > 1) {
      throw new TypeError("Area.opacity requires a value in (0, 1]");
    }
    const next = this.clone();
    next.opacityValue = opacity;
    return next;
  }

  toLayer(columnStart: number): LayerPayload {
    const columns: Float64Array[] = [];
    let index = columnStart;
    let x: unknown = null;
    let low: unknown = null;
    if (this.x) {
      x = { col: index };
      columns.push(this.x);
      index += 1;
    }
    if (this.low) {
      low = { col: index };
      columns.push(this.low);
      index += 1;
    }
    columns.push(this.high);
    const body: Record<string, unknown> = {
      x,
      low,
      high: { col: index },
      horizontal: false,
      color: colorToJSON(this.colorValue),
      label: this.labelValue,
    };
    if (this.opacityValue !== undefined) {
      body.opacity = this.opacityValue;
    }
    return { json: { Area: body }, columns };
  }

  private clone(): Area {
    const next = new Area(this.high, this.x, this.low);
    next.colorValue = this.colorValue;
    next.labelValue = this.labelValue;
    next.opacityValue = this.opacityValue;
    return next;
  }
}

export class Cells extends Mark {
  private columns: number;
  private values: Float64Array | undefined;
  private rgbValue: [number, number, number][] | undefined;
  private classesValue: string[] | undefined;
  private extentsValue: { x: [number, number]; y: [number, number] } | undefined;
  private colormapValue: ColormapJSON | undefined;
  private reduceValue: Reducer | undefined;
  private smoothValue = false;

  private constructor(columns: number) {
    super();
    this.columns = columns;
  }

  static matrix(columns: number, values: SeriesLike): Cells {
    if (columns < 1) {
      throw new TypeError("Cells.matrix requires a positive column count");
    }
    const series = toFloat64(values);
    if (series.length % columns !== 0) {
      throw new TypeError("Cells.matrix requires columns to divide the value count evenly");
    }
    const cells = new Cells(columns);
    cells.values = series;
    return cells;
  }

  static rgb(columns: number, pixels: Iterable<[number, number, number]>): Cells {
    if (columns < 1) {
      throw new TypeError("Cells.rgb requires a positive column count");
    }
    const list = [...pixels];
    if (list.length % columns !== 0) {
      throw new TypeError("Cells.rgb requires columns to divide the pixel count evenly");
    }
    const cells = new Cells(columns);
    cells.rgbValue = list;
    return cells;
  }

  static classes(columns: number, labels: Iterable<string>): Cells {
    if (columns < 1) {
      throw new TypeError("Cells.classes requires a positive column count");
    }
    const list = [...labels];
    if (list.length % columns !== 0) {
      throw new TypeError("Cells.classes requires columns to divide the label count evenly");
    }
    const cells = new Cells(columns);
    cells.classesValue = list;
    return cells;
  }

  extents(x: [number, number], y: [number, number]): Cells {
    if (![...x, ...y].every(Number.isFinite) || x[0] === x[1] || y[0] === y[1]) {
      throw new TypeError("Cells.extents requires finite, non-empty bounds");
    }
    const next = this.clone();
    next.extentsValue = { x, y };
    return next;
  }

  colormap(map: ColormapJSON): Cells {
    const next = this.clone();
    next.colormapValue = map;
    return next;
  }

  reduce(reducer: Reducer): Cells {
    const next = this.clone();
    next.reduceValue = reducer;
    return next;
  }

  smooth(): Cells {
    const next = this.clone();
    next.smoothValue = true;
    return next;
  }

  toLayer(columnStart: number): LayerPayload {
    const columns: Float64Array[] = [];
    const body: Record<string, unknown> = {
      columns: this.columns,
      values: this.values ? { col: columnStart } : [],
      extents: this.extentsValue
        ? [this.extentsValue.x, this.extentsValue.y]
        : null,
      colormap: this.colormapValue ?? Colormap.VIRIDIS,
    };
    if (this.values) {
      columns.push(this.values);
    }
    if (this.rgbValue) {
      body.rgb = this.rgbValue;
    }
    if (this.classesValue) {
      body.classes = this.classesValue;
    }
    if (this.reduceValue && this.reduceValue !== "Mean") {
      body.reduce = this.reduceValue;
    }
    if (this.smoothValue) {
      body.smooth = true;
    }
    return { json: { Cells: body }, columns };
  }

  private clone(): Cells {
    const next = new Cells(this.columns);
    next.values = this.values;
    next.rgbValue = this.rgbValue;
    next.classesValue = this.classesValue;
    next.extentsValue = this.extentsValue;
    next.colormapValue = this.colormapValue;
    next.reduceValue = this.reduceValue;
    next.smoothValue = this.smoothValue;
    return next;
  }
}

export class Range extends Mark {
  private placement:
    | { kind: "numeric"; x?: Float64Array }
    | { kind: "bands"; categories: string[] };
  private low: Float64Array;
  private high: Float64Array;
  private bodyLow: Float64Array | undefined;
  private bodyHigh: Float64Array | undefined;
  private markerValue: Float64Array | undefined;
  private colorValue: Color | null = null;
  private labelValue: string | null = null;
  private colorByValue: string[] | undefined;

  private constructor(
    placement: Range["placement"],
    low: Float64Array,
    high: Float64Array,
  ) {
    super();
    this.placement = placement;
    this.low = low;
    this.high = high;
  }

  static xy(x: SeriesLike, low: SeriesLike, high: SeriesLike): Range {
    const xs = toFloat64(x);
    const lo = toFloat64(low);
    const hi = toFloat64(high);
    if (xs.length !== lo.length || xs.length !== hi.length) {
      throw new TypeError("Range.xy requires series of equal length");
    }
    return new Range({ kind: "numeric", x: xs }, lo, hi);
  }

  static y(low: SeriesLike, high: SeriesLike): Range {
    const lo = toFloat64(low);
    const hi = toFloat64(high);
    if (lo.length !== hi.length) {
      throw new TypeError("Range.y requires series of equal length");
    }
    return new Range({ kind: "numeric" }, lo, hi);
  }

  static over(categories: Iterable<string>, low: SeriesLike, high: SeriesLike): Range {
    const names = [...categories];
    const lo = toFloat64(low);
    const hi = toFloat64(high);
    if (names.length !== lo.length || names.length !== hi.length) {
      throw new TypeError("Range.over requires one category per interval");
    }
    return new Range({ kind: "bands", categories: names }, lo, hi);
  }

  body(low: SeriesLike, high: SeriesLike): Range {
    const lo = toFloat64(low);
    const hi = toFloat64(high);
    if (lo.length !== this.low.length || hi.length !== this.low.length) {
      throw new TypeError("Range.body requires series matching the range length");
    }
    const next = this.clone();
    next.bodyLow = lo;
    next.bodyHigh = hi;
    return next;
  }

  marker(values: SeriesLike): Range {
    const series = toFloat64(values);
    if (series.length !== this.low.length) {
      throw new TypeError("Range.marker requires a series matching the range length");
    }
    const next = this.clone();
    next.markerValue = series;
    return next;
  }

  color(color: string | Color): Range {
    const next = this.clone();
    next.colorValue = canonicalizeColor(color);
    return next;
  }

  label(label: string): Range {
    const next = this.clone();
    next.labelValue = label;
    return next;
  }

  colorBy(categories: Iterable<string>): Range {
    const next = this.clone();
    next.colorByValue = [...categories];
    return next;
  }

  toLayer(columnStart: number): LayerPayload {
    const columns: Float64Array[] = [];
    let index = columnStart;
    let placement: unknown;
    if (this.placement.kind === "bands") {
      placement = { Bands: this.placement.categories };
    } else if (this.placement.x) {
      placement = { Numeric: { col: index } };
      columns.push(this.placement.x);
      index += 1;
    } else {
      placement = { Numeric: null };
    }
    const low = { col: index };
    columns.push(this.low);
    index += 1;
    const high = { col: index };
    columns.push(this.high);
    index += 1;
    const body: Record<string, unknown> = {
      placement,
      low,
      high,
      color: colorToJSON(this.colorValue),
      label: this.labelValue,
    };
    if (this.bodyLow && this.bodyHigh) {
      body.body = [{ col: index }, { col: index + 1 }];
      columns.push(this.bodyLow, this.bodyHigh);
      index += 2;
    }
    if (this.markerValue) {
      body.marker = { col: index };
      columns.push(this.markerValue);
    }
    if (this.colorByValue) {
      body.color_by = this.colorByValue;
    }
    return { json: { Range: body }, columns };
  }

  private clone(): Range {
    const next = new Range(this.placement, this.low, this.high);
    next.bodyLow = this.bodyLow;
    next.bodyHigh = this.bodyHigh;
    next.markerValue = this.markerValue;
    next.colorValue = this.colorValue;
    next.labelValue = this.labelValue;
    next.colorByValue = this.colorByValue;
    return next;
  }
}
