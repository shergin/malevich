import type { PackedRaster } from "./engine.js";
import type { Color, NamedColor } from "./color.js";

const NAMED: NamedColor[] = [
  "Black",
  "Red",
  "Green",
  "Yellow",
  "Blue",
  "Magenta",
  "Cyan",
  "White",
  "BrightBlack",
  "BrightRed",
  "BrightGreen",
  "BrightYellow",
  "BrightBlue",
  "BrightMagenta",
  "BrightCyan",
  "BrightWhite",
];

export type RasterCell = {
  glyph: string;
  foreground: Color;
  background: Color;
  columns: number;
};

export class Raster {
  readonly width: number;
  readonly height: number;
  readonly cells: readonly RasterCell[];

  constructor(width: number, height: number, cells: RasterCell[]) {
    this.width = width;
    this.height = height;
    this.cells = cells;
  }

  isEmpty(): boolean {
    return this.cells.length === 0;
  }

  cell(column: number, row: number): RasterCell | undefined {
    if (column < 0 || row < 0 || column >= this.width || row >= this.height) {
      return undefined;
    }
    return this.cells[row * this.width + column];
  }

  /** Run-length rows for Ink / any cell host. Continuation cells are skipped. */
  rows(): { glyph: string; foreground: Color; background: Color }[][] {
    const rows: { glyph: string; foreground: Color; background: Color }[][] = [];
    for (let row = 0; row < this.height; row++) {
      const line: { glyph: string; foreground: Color; background: Color }[] = [];
      for (let column = 0; column < this.width; column++) {
        const cell = this.cells[row * this.width + column];
        if (cell.columns === 0) {
          continue;
        }
        line.push({
          glyph: cell.glyph,
          foreground: cell.foreground,
          background: cell.background,
        });
      }
      rows.push(line);
    }
    return rows;
  }
}

export function unpackRaster(packed: PackedRaster): Raster {
  const cells: RasterCell[] = [];
  const fg = packed.fg;
  const bg = packed.bg;
  const columns = packed.columns;
  const glyphs = [...packed.glyphs];
  for (let i = 0; i < glyphs.length; i++) {
    cells.push({
      glyph: glyphs[i] ?? " ",
      foreground: unpackColor(fg, i * 4),
      background: unpackColor(bg, i * 4),
      columns: columns[i] ?? 1,
    });
  }
  return new Raster(packed.width, packed.height, cells);
}

function unpackColor(bytes: Uint8Array, offset: number): Color {
  const tag = bytes[offset] ?? 0;
  if (tag === 1) {
    return NAMED[bytes[offset + 1] ?? 0] ?? "Default";
  }
  if (tag === 2) {
    return { ansi256: bytes[offset + 1] ?? 0 };
  }
  if (tag === 3) {
    return { rgb: [bytes[offset + 1] ?? 0, bytes[offset + 2] ?? 0, bytes[offset + 3] ?? 0] };
  }
  return "Default";
}
