import { Color, canonicalizeColor, colorToJSON } from "./color.js";
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

export class Bars extends Mark {
  private categories: string[];
  private values: Float64Array;
  private colorValue: Color | null = null;
  private labelValue: string | null = null;
  private colorByValue: string[] | undefined;

  private constructor(categories: string[], values: Float64Array) {
    super();
    this.categories = categories;
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
    return new Bars(names, series);
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

  toLayer(columnStart: number): LayerPayload {
    const body: Record<string, unknown> = {
      placement: { Bands: this.categories },
      values: { col: columnStart },
      color: colorToJSON(this.colorValue),
      label: this.labelValue,
    };
    if (this.colorByValue) {
      body.color_by = this.colorByValue;
    }
    return { json: { Bars: body }, columns: [this.values] };
  }

  private clone(): Bars {
    const next = new Bars(this.categories, this.values);
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
