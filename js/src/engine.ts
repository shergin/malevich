import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

export type PackedRaster = {
  width: number;
  height: number;
  glyphs: string;
  fg: Uint8Array;
  bg: Uint8Array;
  columns: Uint8Array;
};

export type Panel = {
  column: number;
  row: number;
  width: number;
  height: number;
};

export type JsMapping = {
  dataAt(column: number, row: number): [number, number] | undefined;
  cellAt(x: number, y: number): [number, number] | undefined;
  columnAt(x: number): number | undefined;
  formatX(value: number): string;
  formatY(value: number): string;
  readonly xDomain: [number, number];
  readonly yDomain: [number, number];
  readonly plotArea: Panel | undefined;
  readonly xCategories: string[] | undefined;
  readonly yCategories: string[] | undefined;
  viewport(): JsViewport;
  free?(): void;
};

export type JsViewport = {
  zoomX(factor: number, anchor: number): JsViewport;
  zoomY(factor: number, anchor: number): JsViewport;
  panX(fraction: number): JsViewport;
  panY(fraction: number): JsViewport;
  reset(): JsViewport;
  resetX(): JsViewport;
  resetY(): JsViewport;
  withX(low: number, high: number): JsViewport;
  withY(low: number, high: number): JsViewport;
  clampX(low: number, high: number): JsViewport;
  clampY(low: number, high: number): JsViewport;
  tail(latest: number, width: number): JsViewport;
  readonly isAuto: boolean;
  readonly x: [number, number] | undefined;
  readonly y: [number, number] | undefined;
  free?(): void;
};

type Native = {
  render_document(document: string, frame: string): string;
  render_columns(document: string, frame: string, columns: Float64Array[]): string;
  raster_columns(
    document: string,
    frame: string,
    columns: Float64Array[],
  ): PackedRaster;
  mapping_columns(
    document: string,
    frame: string,
    columns: Float64Array[],
  ): JsMapping;
  expand_preset(name: string, options: string, columns: Float64Array[]): string;
  render_pixels_columns(
    document: string,
    frame: string,
    columns: Float64Array[],
    protocol: string,
    cell_width: number,
    cell_height: number,
  ): string;
  JsViewport: { auto(): JsViewport };
};

const require = createRequire(import.meta.url);

function load(): Native {
  const here = dirname(fileURLToPath(import.meta.url));
  const candidates = [
    join(here, "../generated/malevich_js.js"),
    join(here, "generated/malevich_js.js"),
  ];
  let last: unknown;
  for (const path of candidates) {
    try {
      return require(path) as Native;
    } catch (error) {
      last = error;
    }
  }
  throw new Error(
    `malevich wasm module not found; run npm run build:wasm in js/. Last error: ${last}`,
  );
}

let cached: Native | undefined;

export function engine(): Native {
  cached ??= load();
  return cached;
}

/** Load the wasm module. Sync on Node/Bun/Deno; call once at startup in bundlers. */
export function init(): Promise<void> {
  engine();
  return Promise.resolve();
}

export const engineVersion = "1.21.0";

/** A viewport with both axes automatic — seed from a mapping before transforming. */
export function viewportAuto(): JsViewport {
  return engine().JsViewport.auto();
}

/** Coerce a wasm pair (`[lo, hi]` or `undefined`) into a tuple. */
export function asPair(value: unknown): [number, number] | undefined {
  if (!Array.isArray(value) || value.length < 2) {
    return undefined;
  }
  const low = Number(value[0]);
  const high = Number(value[1]);
  if (!Number.isFinite(low) || !Number.isFinite(high)) {
    return undefined;
  }
  return [low, high];
}

export function asPanel(value: unknown): Panel | undefined {
  if (value == null || typeof value !== "object") {
    return undefined;
  }
  const panel = value as Record<string, unknown>;
  const column = Number(panel.column);
  const row = Number(panel.row);
  const width = Number(panel.width);
  const height = Number(panel.height);
  if (![column, row, width, height].every(Number.isFinite)) {
    return undefined;
  }
  return { column, row, width, height };
}

export function drop(value: { free?(): void } | undefined): void {
  try {
    value?.free?.();
  } catch {
    // Already dropped, or the runtime has no destructor.
  }
}
