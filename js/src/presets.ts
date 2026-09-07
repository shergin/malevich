import { engine } from "./engine.js";
import { wrapWasm } from "./error.js";
import { Bars, Line, Points } from "./mark.js";
import { Plot } from "./plot.js";
import { type SeriesLike, toFloat64 } from "./series.js";

export function line(values: SeriesLike): Plot {
  return new Plot().layer(Line.y(values));
}

export function scatter(x: SeriesLike, y: SeriesLike): Plot {
  return new Plot().layer(Points.xy(x, y));
}

export function bar(categories: Iterable<string>, values: SeriesLike): Plot {
  return new Plot().layer(Bars.new(categories, values));
}

export function stairs(values: SeriesLike): Plot {
  return fromPreset("stairs", {}, [toFloat64(values)]);
}

export function hist(values: SeriesLike): Plot {
  return histWith(values, { maxBins: 60 });
}

export function histWith(values: SeriesLike, options: { maxBins?: number } = {}): Plot {
  return fromPreset("hist", { max_bins: options.maxBins ?? 60 }, [toFloat64(values)]);
}

export function ecdf(values: SeriesLike): Plot {
  return fromPreset("ecdf", {}, [toFloat64(values)]);
}

export function density(values: SeriesLike): Plot {
  return fromPreset("density", {}, [toFloat64(values)]);
}

export function trend(x: SeriesLike, y: SeriesLike): Plot {
  return fromPreset("trend", {}, [toFloat64(x), toFloat64(y)]);
}

export function hist2d(x: SeriesLike, y: SeriesLike): Plot {
  return fromPreset("hist2d", {}, [toFloat64(x), toFloat64(y)]);
}

export function heatmap(columns: number, values: SeriesLike): Plot {
  return fromPreset("heatmap", { columns }, [toFloat64(values)]);
}

export function boxPlot(categories: Iterable<string>, groups: SeriesLike[]): Plot {
  return fromPreset(
    "box_plot",
    { categories: [...categories] },
    groups.map(toFloat64),
  );
}

export function violin(categories: Iterable<string>, groups: SeriesLike[]): Plot {
  return fromPreset(
    "violin",
    { categories: [...categories] },
    groups.map(toFloat64),
  );
}

export function describe(names: Iterable<string>, groups: SeriesLike[]): Plot {
  return fromPreset("describe", { names: [...names] }, groups.map(toFloat64));
}

export function table(
  rows: Iterable<string>,
  columns: Iterable<string>,
  values: SeriesLike,
): Plot {
  return tableWith(rows, columns, values);
}

export function tableWith(
  rows: Iterable<string>,
  columns: Iterable<string>,
  values: SeriesLike,
  options: { colormap?: import("./color.js").ColormapJSON } = {},
): Plot {
  return fromPreset(
    "table",
    {
      rows: [...rows],
      columns: [...columns],
      colormap: options.colormap ?? null,
    },
    [toFloat64(values)],
  );
}

export function errorBars(x: SeriesLike, y: SeriesLike, error: SeriesLike): Plot {
  return fromPreset("error_bars", {}, [toFloat64(x), toFloat64(y), toFloat64(error)]);
}

function fromPreset(
  name: string,
  options: unknown,
  columns: Float64Array[],
): Plot {
  try {
    const json = engine().expand_preset(name, JSON.stringify(options), columns);
    return Plot.fromDocument(JSON.parse(json) as Parameters<typeof Plot.fromDocument>[0]);
  } catch (error) {
    wrapWasm(error);
  }
}
