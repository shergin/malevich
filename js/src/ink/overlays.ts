import type { Color } from "../color.js";
import { asPair, asPanel, type Panel } from "../engine.js";
import type { Plot } from "../plot.js";
import { Raster, type RasterCell } from "../raster.js";
import type { PlotState } from "./PlotState.js";

export type OverlayOptions = {
  crosshair?: boolean;
  readout?: boolean;
  snap?: boolean;
};

export type Snapped = {
  label: string | undefined;
  x: number;
  value: number | undefined;
};

/**
 * Draws the interaction chrome over a raster: selection band, crosshair,
 * snap highlights, readout. Overlays live in the cells only — the plot
 * value and its rendering stay byte-identical with or without them.
 */
export function applyOverlays(
  raster: Raster,
  state: PlotState,
  plot: Plot,
  options: OverlayOptions = {},
  light = false,
): Raster {
  const rect = state.plotArea();
  if (!rect) {
    return raster;
  }
  const cells = raster.cells.map((cell) => ({ ...cell }));
  const tint: Color = light ? "White" : "BrightBlack";
  const highlight: Color = light ? "Black" : "White";
  const origin = widgetOrigin(state, rect, raster);

  const selection = state.selection();
  if (selection) {
    const x0 = Math.min(selection.anchor[0], selection.current[0]);
    const x1 = Math.max(selection.anchor[0], selection.current[0]);
    const y0 = Math.min(selection.anchor[1], selection.current[1]);
    const y1 = Math.max(selection.anchor[1], selection.current[1]);
    for (let y = y0; y <= y1; y++) {
      for (let x = x0; x <= x1; x++) {
        setBackground(cells, raster.width, local(origin, x, y), tint);
      }
    }
  }

  const hover = hoverInRect(state, rect);
  if ((options.crosshair ?? true) && hover) {
    if (hover.row !== undefined) {
      for (let x = rect.column; x < rect.column + rect.width; x++) {
        tintIfUnset(cells, raster.width, local(origin, x, hover.row), tint);
      }
    }
    for (let y = rect.row; y < rect.row + rect.height; y++) {
      tintIfUnset(cells, raster.width, local(origin, hover.column, y), tint);
    }
  }

  if (!hover) {
    return new Raster(raster.width, raster.height, cells);
  }

  const snapped = (options.snap ?? true) ? snapTargets(plot, state) : [];
  const mapping = state.mapping();
  for (const snap of snapped) {
    if (snap.value === undefined || !mapping) {
      continue;
    }
    const cell = asPair(mapping.cellAt(snap.x, snap.value));
    if (!cell) {
      continue;
    }
    const column = origin.column + cell[0];
    const row = origin.row + cell[1];
    if (
      column < rect.column + rect.width &&
      row < rect.row + rect.height
    ) {
      setBackground(cells, raster.width, local(origin, column, row), highlight);
    }
  }

  if (options.readout ?? true) {
    const text = readoutLine(state, snapped, Math.max(0, rect.width - 2));
    if (text) {
      const width = displayWidth(text);
      writeString(
        cells,
        raster.width,
        local(origin, rect.column + rect.width - width - 1, rect.row),
        text,
        highlight,
      );
    }
  }

  return new Raster(raster.width, raster.height, cells);
}

export function snapTargets(plot: Plot, state: PlotState): Snapped[] {
  const cursorX = state.hoverDataX();
  const mapping = state.mapping();
  if (cursorX === undefined || !mapping) {
    return [];
  }
  const window = asPair(mapping.xDomain);
  if (!window) {
    return [];
  }
  const columns = plot.columnBuffers();
  const snapped: Snapped[] = [];
  for (const layer of plot.layers) {
    const body = layerBody(layer);
    if (!body) {
      continue;
    }
    const y = asSeries(body.y, columns);
    if (!y) {
      continue;
    }
    const x = asSeries(body.x, columns);
    const index = nearestVisible(x, y.length, cursorX, window);
    if (index === undefined) {
      continue;
    }
    const value = y[index];
    snapped.push({
      label: typeof body.label === "string" ? body.label : undefined,
      x: x ? (x[index] as number) : index,
      value: value !== undefined && Number.isFinite(value) ? value : undefined,
    });
  }
  return snapped;
}

export function displayWidth(text: string): number {
  let width = 0;
  for (const char of text) {
    const code = char.codePointAt(0) ?? 0;
    if (code < 32) {
      continue;
    }
    width += isWide(code) ? 2 : 1;
  }
  return width;
}

function readoutLine(state: PlotState, snapped: Snapped[], budget: number): string | undefined {
  const cursorX = state.hoverDataX();
  const mapping = state.mapping();
  if (cursorX === undefined || !mapping) {
    return undefined;
  }
  const xPart =
    snapped.length === 1 ? mapping.formatX(snapped[0]!.x) : mapping.formatX(cursorX);
  const parts = [xPart];
  if (snapped.length === 0) {
    const cursor = state.cursorData();
    if (cursor) {
      parts.push(mapping.formatY(cursor[1]));
    }
  }
  for (const snap of snapped) {
    const value =
      snap.value === undefined ? "—" : mapping.formatY(snap.value);
    parts.push(snap.label ? `${snap.label}: ${value}` : value);
  }
  for (;;) {
    const text = parts.join(" · ");
    if (displayWidth(text) <= budget) {
      return text;
    }
    if (parts.length <= 2) {
      return undefined;
    }
    parts.pop();
  }
}

function layerBody(layer: Record<string, unknown>): {
  x: unknown;
  y: unknown;
  label: unknown;
} | undefined {
  const line = layer.Line as Record<string, unknown> | undefined;
  const points = layer.Points as Record<string, unknown> | undefined;
  const body = line ?? points;
  if (!body) {
    return undefined;
  }
  return { x: body.x, y: body.y, label: body.label };
}

function asSeries(field: unknown, columns: Float64Array[]): Float64Array | undefined {
  if (field == null) {
    return undefined;
  }
  if (Array.isArray(field)) {
    const out = new Float64Array(field.length);
    for (let i = 0; i < field.length; i++) {
      const value = field[i];
      out[i] = typeof value === "number" ? value : Number.NaN;
    }
    return out;
  }
  if (typeof field === "object" && "col" in field) {
    return columns[(field as { col: number }).col];
  }
  return undefined;
}

function nearestVisible(
  x: Float64Array | undefined,
  len: number,
  target: number,
  window: [number, number],
): number | undefined {
  if (x) {
    let best: number | undefined;
    let nearest = Infinity;
    for (let index = 0; index < x.length; index++) {
      const value = x[index]!;
      if (!Number.isFinite(value) || value < window[0] || value > window[1]) {
        continue;
      }
      const distance = Math.abs(value - target);
      if (distance < nearest) {
        best = index;
        nearest = distance;
      }
    }
    return best;
  }
  let index = Math.round(target);
  if (index < window[0]) {
    index = Math.ceil(target);
  } else if (index > window[1]) {
    index = Math.floor(target);
  }
  if (index < Math.max(window[0], 0) || index > window[1]) {
    return undefined;
  }
  return index < len ? index : undefined;
}

function hoverInRect(
  state: PlotState,
  rect: Panel,
): { column: number; row: number | undefined } | undefined {
  const hover = state.hover();
  if (!hover) {
    return undefined;
  }
  if (hover.kind === "cell") {
    if (
      hover.column >= rect.column &&
      hover.column < rect.column + rect.width &&
      hover.row >= rect.row &&
      hover.row < rect.row + rect.height
    ) {
      return { column: hover.column, row: hover.row };
    }
    return undefined;
  }
  if (hover.column >= rect.column && hover.column < rect.column + rect.width) {
    return { column: hover.column, row: undefined };
  }
  return undefined;
}

function widgetOrigin(state: PlotState, rect: Panel, raster: Raster): Panel {
  const mapping = state.mapping();
  const panel = mapping ? asPanel(mapping.plotArea) : undefined;
  if (!panel) {
    return { column: 0, row: 0, width: raster.width, height: raster.height };
  }
  return {
    column: rect.column - panel.column,
    row: rect.row - panel.row,
    width: raster.width,
    height: raster.height,
  };
}

function local(origin: Panel, column: number, row: number): [number, number] {
  return [column - origin.column, row - origin.row];
}

function cellAt(
  cells: RasterCell[],
  width: number,
  at: [number, number],
): RasterCell | undefined {
  const [column, row] = at;
  if (column < 0 || row < 0) {
    return undefined;
  }
  return cells[row * width + column];
}

function tintIfUnset(
  cells: RasterCell[],
  width: number,
  at: [number, number],
  tint: Color,
): void {
  const cell = cellAt(cells, width, at);
  if (cell && cell.background === "Default") {
    cell.background = tint;
  }
}

function setBackground(
  cells: RasterCell[],
  width: number,
  at: [number, number],
  color: Color,
): void {
  const cell = cellAt(cells, width, at);
  if (cell) {
    cell.background = color;
  }
}

function writeString(
  cells: RasterCell[],
  width: number,
  at: [number, number],
  text: string,
  foreground: Color,
): void {
  let column = at[0];
  const row = at[1];
  for (const char of text) {
    const cell = cellAt(cells, width, [column, row]);
    if (cell) {
      cell.glyph = char;
      cell.foreground = foreground;
      cell.columns = 1;
    }
    column += displayWidth(char) || 1;
  }
}

function isWide(code: number): boolean {
  return (
    code >= 0x1100 &&
    (code <= 0x115f ||
      code === 0x2329 ||
      code === 0x232a ||
      (code >= 0x2e80 && code <= 0xa4cf && code !== 0x303f) ||
      (code >= 0xac00 && code <= 0xd7a3) ||
      (code >= 0xf900 && code <= 0xfaff) ||
      (code >= 0xfe10 && code <= 0xfe19) ||
      (code >= 0xfe30 && code <= 0xfe6f) ||
      (code >= 0xff00 && code <= 0xff60) ||
      (code >= 0xffe0 && code <= 0xffe6) ||
      (code >= 0x1f300 && code <= 0x1f64f) ||
      (code >= 0x1f900 && code <= 0x1f9ff) ||
      (code >= 0x20000 && code <= 0x2fffd) ||
      (code >= 0x30000 && code <= 0x3fffd))
  );
}
