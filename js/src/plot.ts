import { inspect } from "node:util";
import { ScaleJSON, scaleToJSON } from "./color.js";
import { asPair, engine } from "./engine.js";
import { wrapWasm } from "./error.js";
import { Frame } from "./frame.js";
import { Mark } from "./mark.js";
import { Raster, unpackRaster } from "./raster.js";

/** An axis window pair. `undefined` on an axis means automatic. */
export type Viewport = {
  x?: [number, number];
  y?: [number, number];
};

export type PlotSpec = {
  layers: Record<string, unknown>[];
  title: string | null;
  x: ScaleJSON;
  y: ScaleJSON;
  x_label: string | null;
  y_label: string | null;
  x_domain: [number, number] | null;
  y_domain: [number, number] | null;
  colorbar: boolean;
  palette?: unknown;
};

export type DocumentJSON = {
  version: 1;
  kind: "plot" | "grid";
  spec: PlotSpec | GridSpec;
};

export type GridSpec = {
  columns: number;
  plots: PlotSpec[];
};

function emptySpec(): PlotSpec {
  return {
    layers: [],
    title: null,
    x: "Auto",
    y: "Auto",
    x_label: null,
    y_label: null,
    x_domain: null,
    y_domain: null,
    colorbar: false,
  };
}

export class Plot {
  private spec: PlotSpec;
  private columns: Float64Array[];

  constructor() {
    this.spec = emptySpec();
    this.columns = [];
  }

  static new(): Plot {
    return new Plot();
  }

  static fromDocument(document: DocumentJSON, columns: Float64Array[] = []): Plot {
    if (document.kind !== "plot") {
      throw new TypeError("Plot.fromDocument requires a plot document");
    }
    const plot = new Plot();
    plot.spec = document.spec as PlotSpec;
    plot.columns = columns;
    return plot;
  }

  layer(mark: Mark): Plot {
    const next = this.clone();
    const { json, columns } = mark.toLayer(next.columns.length);
    next.columns = [...next.columns, ...columns];
    next.spec = { ...next.spec, layers: [...next.spec.layers, json] };
    return next;
  }

  title(title: string): Plot {
    return this.withSpec({ title });
  }

  xLabel(label: string): Plot {
    return this.withSpec({ x_label: label });
  }

  yLabel(label: string): Plot {
    return this.withSpec({ y_label: label });
  }

  xDomain(min: number, max: number): Plot {
    if (!Number.isFinite(min) || !Number.isFinite(max)) {
      throw new TypeError("Plot.xDomain requires finite bounds");
    }
    return this.withSpec({ x_domain: [Math.min(min, max), Math.max(min, max)] });
  }

  yDomain(min: number, max: number): Plot {
    if (!Number.isFinite(min) || !Number.isFinite(max)) {
      throw new TypeError("Plot.yDomain requires finite bounds");
    }
    return this.withSpec({ y_domain: [Math.min(min, max), Math.max(min, max)] });
  }

  xScale(scale: "Auto" | "Linear" | "Log" | "Time" | { bands: string[] }): Plot {
    return this.withSpec({ x: scaleToJSON(scale) });
  }

  yScale(scale: "Auto" | "Linear" | "Log" | "Time" | { bands: string[] }): Plot {
    return this.withSpec({ y: scaleToJSON(scale) });
  }

  timeX(): Plot {
    return this.xScale("Time");
  }

  logX(): Plot {
    return this.xScale("Log");
  }

  logY(): Plot {
    return this.yScale("Log");
  }

  colorbar(): Plot {
    return this.withSpec({ colorbar: true });
  }

  viewport(view: Viewport): Plot {
    let next: Plot = this;
    const x = asPair(view.x);
    const y = asPair(view.y);
    if (x) {
      next = next.xDomain(x[0], x[1]);
    }
    if (y) {
      next = next.yDomain(y[0], y[1]);
    }
    return next;
  }

  render(frame: Frame): string {
    try {
      return engine().render_columns(
        JSON.stringify(this.toJSON()),
        JSON.stringify(frame.toJSON()),
        this.columns,
      );
    } catch (error) {
      wrapWasm(error);
    }
  }

  tryRender(frame: Frame): string {
    return this.render(frame);
  }

  raster(frame: Frame): Raster {
    try {
      return unpackRaster(
        engine().raster_columns(
          JSON.stringify(this.toJSON()),
          JSON.stringify(frame.toJSON()),
          this.columns,
        ),
      );
    } catch (error) {
      wrapWasm(error);
    }
  }

  mapping(frame: Frame) {
    try {
      return engine().mapping_columns(
        JSON.stringify(this.toJSON()),
        JSON.stringify(frame.toJSON()),
        this.columns,
      );
    } catch (error) {
      wrapWasm(error);
    }
  }

  toJSON(): DocumentJSON {
    return { version: 1, kind: "plot", spec: this.spec };
  }

  get layers(): readonly Record<string, unknown>[] {
    return this.spec.layers;
  }

  /** @internal */
  columnBuffers(): Float64Array[] {
    return this.columns;
  }

  [inspect.custom](): string {
    return this.render(Frame.detect());
  }

  toString(): string {
    return this.render(Frame.detect());
  }

  private withSpec(patch: Partial<PlotSpec>): Plot {
    const next = this.clone();
    next.spec = { ...next.spec, ...patch };
    return next;
  }

  private clone(): Plot {
    const next = new Plot();
    next.spec = {
      ...this.spec,
      layers: [...this.spec.layers],
      x_domain: this.spec.x_domain ? [...this.spec.x_domain] : null,
      y_domain: this.spec.y_domain ? [...this.spec.y_domain] : null,
    };
    next.columns = [...this.columns];
    return next;
  }
}

export class Grid {
  private readonly columnCount: number;
  private readonly plots: Plot[];

  constructor(columns: number) {
    if (columns < 1) {
      throw new TypeError("Grid.new requires at least one column");
    }
    this.columnCount = columns;
    this.plots = [];
  }

  static new(columns: number): Grid {
    return new Grid(columns);
  }

  with(plot: Plot): Grid {
    const next = new Grid(this.columnCount);
    next.plots.push(...this.plots, plot);
    return next;
  }

  render(frame: Frame): string {
    const document: DocumentJSON = {
      version: 1,
      kind: "grid",
      spec: {
        columns: this.columnCount,
        plots: this.plots.map((plot) => (plot.toJSON().spec as PlotSpec)),
      },
    };
    // Grid panes inline their series via a combined column table on each plot
    // independently: render each pane's document through the engine by going
    // through Document JSON. Each plot already carries its own columns; the
    // crate's Grid serde expects owned plots with inline series. Expand by
    // rendering each plot at the cell size would fork layout. Instead, serialize
    // each plot through expand-by-render: we send a grid whose plots have been
    // converted by rendering-roundtrip. Simpler path: concatenate is wrong.
    // The wasm Document decoder inlines `{col}` only with a *shared* column
    // table, so a grid of independently-columned plots cannot share one table
    // without rebasing indices.
    //
    // Rebase each plot's `{col: n}` against a concatenated column list.
    const columns: Float64Array[] = [];
    const specs: PlotSpec[] = [];
    for (const plot of this.plots) {
      const json = plot.toJSON();
      const spec = structuredClone(json.spec) as PlotSpec;
      const offset = columns.length;
      rebaseCols(spec, offset);
      columns.push(...plot.columnBuffers());
      specs.push(spec);
    }
    document.spec = { columns: this.columnCount, plots: specs };
    try {
      return engine().render_columns(
        JSON.stringify(document),
        JSON.stringify(frame.toJSON()),
        columns,
      );
    } catch (error) {
      wrapWasm(error);
    }
  }

  toString(): string {
    return this.render(Frame.detect());
  }
}

function rebaseCols(value: unknown, offset: number): void {
  if (Array.isArray(value)) {
    for (const item of value) {
      rebaseCols(item, offset);
    }
    return;
  }
  if (value && typeof value === "object") {
    const record = value as Record<string, unknown>;
    if (typeof record.col === "number" && Object.keys(record).length === 1) {
      record.col = record.col + offset;
      return;
    }
    for (const nested of Object.values(record)) {
      rebaseCols(nested, offset);
    }
  }
}
