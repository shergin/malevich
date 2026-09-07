import { asPair, type Panel } from "../engine.js";
import { Mapping, Viewport as View } from "../mapping.js";
import type { ViewportWindows } from "../plot.js";

/** Wheel and keyboard zoom steps: one notch in, its exact inverse out. */
export const ZOOM_IN = 0.8;
export const ZOOM_OUT = 1.25;
/** The keyboard pan step, as a fraction of the visible window. */
export const PAN_STEP = 0.1;
/** One horizontal scroll notch's pan, small because trackpads emit many. */
export const SCROLL_PAN = 0.03;

export type MouseButton = "left" | "right" | "middle";

/**
 * Backend-neutral mouse input, in terminal cell coordinates (0-based).
 * Mapping a host's event onto this is one short switch — printed for Ink
 * SGR in `parseMouse`. The widget never reads the terminal.
 */
export type Mouse =
  | { kind: "moved"; column: number; row: number }
  | { kind: "down"; button: MouseButton; column: number; row: number }
  | { kind: "drag"; button: MouseButton; column: number; row: number }
  | { kind: "up"; button: MouseButton; column: number; row: number }
  | { kind: "scrollUp"; column: number; row: number }
  | { kind: "scrollDown"; column: number; row: number }
  | { kind: "scrollLeft"; column: number; row: number }
  | { kind: "scrollRight"; column: number; row: number };

type Drag =
  | { kind: "pan"; last: [number, number] }
  | { kind: "select"; anchor: [number, number]; current: [number, number] };

type Hover =
  | { kind: "cell"; column: number; row: number }
  | { kind: "column"; column: number };

/**
 * Interaction controller for one plot pane — the Ink analog of the ratatui
 * `PlotState`. Mutable, like `ListState`: the host keeps one instance, the
 * widget captures the last render's mapping, and `onMouse` runs the default
 * gesture grammar. The widget never reads input.
 */
export class PlotState {
  private area: Panel = { column: 0, row: 0, width: 0, height: 0 };
  private mappingValue: Mapping | undefined;
  private viewX: [number, number] | undefined;
  private viewY: [number, number] | undefined;
  private hoverValue: Hover | undefined;
  private dragValue: Drag | undefined;

  /** The mapping cached by the last stateful render. */
  mapping(): Mapping | undefined {
    return this.mappingValue;
  }

  /**
   * Hit-test a terminal cell against the last stateful render. `undefined`
   * before the first render or outside the plot rectangle.
   */
  dataAt(column: number, row: number): [number, number] | undefined {
    const mapping = this.mappingValue;
    if (!mapping) {
      return undefined;
    }
    const localColumn = column - this.area.column;
    const localRow = row - this.area.row;
    if (localColumn < 0 || localRow < 0) {
      return undefined;
    }
    return mapping.dataAt(localColumn, localRow);
  }

  /** The plot rectangle of the last stateful render, in terminal coordinates. */
  plotArea(): Panel | undefined {
    const panel = this.mappingValue?.plotArea;
    if (!panel) {
      return undefined;
    }
    return {
      column: this.area.column + panel.column,
      row: this.area.row + panel.row,
      width: panel.width,
      height: panel.height,
    };
  }

  /**
   * The hover cursor in terminal coordinates, when it is over the plot
   * rectangle. A mirrored x-only hover (`hoverX`) answers `undefined` — it
   * has no honest row.
   */
  cursor(): [number, number] | undefined {
    if (this.hoverValue?.kind !== "cell") {
      return undefined;
    }
    return [this.hoverValue.column, this.hoverValue.row];
  }

  /** The data coordinates under the hover cursor. */
  cursorData(): [number, number] | undefined {
    const cursor = this.cursor();
    if (!cursor) {
      return undefined;
    }
    return this.dataAt(cursor[0], cursor[1]);
  }

  /**
   * Places an x-only hover from a data x — the mirrored crosshair of a
   * linked pane. A non-finite x, or one outside the visible window, clears
   * it. Returns whether anything changed.
   */
  hoverX(x: number): boolean {
    const column = Number.isFinite(x)
      ? (this.mappingValue?.columnAt(x) ?? Number.NaN)
      : Number.NaN;
    const hover =
      Number.isFinite(column)
        ? ({
            kind: "column",
            column: this.area.column + column,
          } satisfies Hover)
        : undefined;
    const changed = !sameHover(hover, this.hoverValue);
    this.hoverValue = hover;
    return changed;
  }

  /** The viewport the next stateful render applies. */
  viewport(): ViewportWindows {
    return { x: this.viewX, y: this.viewY };
  }

  /** Replaces the viewport — the escape hatch for a host's own gestures. */
  setViewport(view: ViewportWindows): void {
    this.viewX = asPair(view.x);
    this.viewY = asPair(view.y);
  }

  /** Back to the automatic view on both axes. */
  resetView(): void {
    this.viewX = undefined;
    this.viewY = undefined;
  }

  /** Zooms in around the center of the visible x window. */
  zoomIn(): boolean {
    return this.zoomCenter(ZOOM_IN);
  }

  /** Zooms out around the center of the visible x window. */
  zoomOut(): boolean {
    return this.zoomCenter(ZOOM_OUT);
  }

  /** Pans left by a tenth of the visible x window. */
  panLeft(): boolean {
    return this.panStep(-PAN_STEP);
  }

  /** Pans right by a tenth of the visible x window. */
  panRight(): boolean {
    return this.panStep(PAN_STEP);
  }

  /**
   * Feeds one mouse input through the default gesture grammar. Returns
   * whether any state changed — `false` means the host can skip a redraw.
   */
  onMouse(mouse: Mouse): boolean {
    switch (mouse.kind) {
      case "moved":
        return this.hoverAt(mouse.column, mouse.row);
      case "down":
        if (mouse.button === "left") {
          const hovered = this.hoverAt(mouse.column, mouse.row);
          if (this.inside(mouse.column, mouse.row)) {
            this.dragValue = { kind: "pan", last: [mouse.column, mouse.row] };
            return true;
          }
          return hovered;
        }
        if (mouse.button === "right") {
          if (this.inside(mouse.column, mouse.row)) {
            this.dragValue = {
              kind: "select",
              anchor: [mouse.column, mouse.row],
              current: [mouse.column, mouse.row],
            };
            return true;
          }
          return false;
        }
        return false;
      case "drag":
        if (mouse.button === "left") {
          const hovered = this.hoverAt(mouse.column, mouse.row);
          if (this.dragValue?.kind !== "pan") {
            return hovered;
          }
          const last = this.dragValue.last;
          this.dragValue = { kind: "pan", last: [mouse.column, mouse.row] };
          return this.panBy(last, [mouse.column, mouse.row]) || hovered;
        }
        if (mouse.button === "right") {
          const hovered = this.hoverAt(mouse.column, mouse.row);
          if (this.dragValue?.kind !== "select") {
            return hovered;
          }
          const rect = this.plotArea();
          if (!rect) {
            return hovered;
          }
          this.dragValue = {
            kind: "select",
            anchor: this.dragValue.anchor,
            current: clampInto(rect, mouse.column, mouse.row),
          };
          return true;
        }
        return false;
      case "up":
        if (mouse.button === "left") {
          const panning = this.dragValue?.kind === "pan";
          if (panning) {
            this.dragValue = undefined;
          }
          return panning;
        }
        if (mouse.button === "right") {
          if (this.dragValue?.kind !== "select") {
            return false;
          }
          const { anchor, current } = this.dragValue;
          this.dragValue = undefined;
          this.applySelection(anchor, current);
          return true;
        }
        return false;
      case "scrollUp":
        return this.wheel(ZOOM_IN, mouse.column, mouse.row);
      case "scrollDown":
        return this.wheel(ZOOM_OUT, mouse.column, mouse.row);
      case "scrollLeft":
        return this.scrollPan(-SCROLL_PAN, mouse.column, mouse.row);
      case "scrollRight":
        return this.scrollPan(SCROLL_PAN, mouse.column, mouse.row);
    }
  }

  /**
   * Called by the stateful widget after each render: caches the mapping of
   * what is on screen and the widget's rectangle in terminal coordinates.
   */
  capture(mapping: Mapping, area: Panel): void {
    if (this.mappingValue && this.mappingValue !== mapping) {
      this.mappingValue.dispose();
    }
    this.mappingValue = mapping;
    this.area = area;
  }

  /** The data x under the hover — a real cursor's or a mirrored one's. */
  hoverDataX(): number | undefined {
    const rect = this.plotArea();
    const hover = this.hoverValue;
    if (!rect || !hover) {
      return undefined;
    }
    return this.dataAt(hover.column, rect.row)?.[0];
  }

  /** A rubber-band selection in progress, in terminal coordinates. */
  selection(): { anchor: [number, number]; current: [number, number] } | undefined {
    if (this.dragValue?.kind !== "select") {
      return undefined;
    }
    return { anchor: this.dragValue.anchor, current: this.dragValue.current };
  }

  /** The hover, including a mirrored x-only column. */
  hover(): Hover | undefined {
    return this.hoverValue;
  }

  private hoverAt(column: number, row: number): boolean {
    const hover = this.inside(column, row)
      ? ({ kind: "cell", column, row } satisfies Hover)
      : undefined;
    const changed = !sameHover(hover, this.hoverValue);
    this.hoverValue = hover;
    return changed;
  }

  private inside(column: number, row: number): boolean {
    const rect = this.plotArea();
    if (!rect) {
      return false;
    }
    return (
      column >= rect.column &&
      column < rect.column + rect.width &&
      row >= rect.row &&
      row < rect.row + rect.height
    );
  }

  private wheel(factor: number, column: number, row: number): boolean {
    if (!this.inside(column, row)) {
      return false;
    }
    const data = this.dataAt(column, row);
    if (!data || this.xIsBands()) {
      return false;
    }
    return this.commitX((seeded) => seeded.zoomX(factor, data[0]));
  }

  private scrollPan(fraction: number, column: number, row: number): boolean {
    if (!this.inside(column, row) || this.xIsBands()) {
      return false;
    }
    return this.commitX((seeded) => seeded.panX(fraction));
  }

  private panStep(fraction: number): boolean {
    if (!this.mappingValue || this.xIsBands()) {
      return false;
    }
    return this.commitX((seeded) => seeded.panX(fraction));
  }

  private zoomCenter(factor: number): boolean {
    const mapping = this.mappingValue;
    const panel = mapping?.plotArea;
    if (!mapping || !panel || this.xIsBands()) {
      return false;
    }
    const anchor = mapping.dataAt(
      panel.column + Math.floor(panel.width / 2),
      panel.row + Math.floor(panel.height / 2),
    );
    if (!anchor) {
      return false;
    }
    return this.commitX((seeded) => seeded.zoomX(factor, anchor[0]));
  }

  private panBy(from: [number, number], to: [number, number]): boolean {
    const mapping = this.mappingValue;
    const panel = mapping?.plotArea;
    if (!mapping || !panel) {
      return false;
    }
    const dx = to[0] - from[0];
    const dy = to[1] - from[1];
    if (dx === 0 && dy === 0) {
      return false;
    }
    const columns = Math.max(panel.width, 1);
    const rows = Math.max(panel.height, 1);
    return this.commitBoth((seeded) =>
      seeded.panX(-dx / columns).panY(dy / rows),
    );
  }

  private applySelection(anchor: [number, number], current: [number, number]): void {
    const mapping = this.mappingValue;
    if (!mapping) {
      return;
    }
    if (Math.abs(anchor[0] - current[0]) < 2 || Math.abs(anchor[1] - current[1]) < 2) {
      return;
    }
    const a = this.dataAt(anchor[0], anchor[1]);
    const c = this.dataAt(current[0], current[1]);
    if (!a || !c) {
      return;
    }
    let view = mapping.viewport();
    if (!mapping.xCategories) {
      view = take(view, view.withX(a[0], c[0]));
    }
    if (!mapping.yCategories) {
      view = take(view, view.withY(a[1], c[1]));
    }
    this.viewX = view.x;
    this.viewY = view.y;
    view.dispose();
  }

  private commitX(transform: (seeded: View) => View): boolean {
    const mapping = this.mappingValue;
    if (!mapping) {
      return false;
    }
    let seeded = mapping.viewport();
    if (this.viewX) {
      seeded = take(seeded, seeded.withX(this.viewX[0], this.viewX[1]));
    }
    seeded = this.viewY
      ? take(seeded, seeded.withY(this.viewY[0], this.viewY[1]))
      : take(seeded, seeded.resetY());
    const next = transform(seeded);
    if (next !== seeded) {
      seeded.dispose();
    }
    this.viewX = next.x;
    this.viewY = next.y;
    next.dispose();
    return true;
  }

  private commitBoth(transform: (seeded: View) => View): boolean {
    const mapping = this.mappingValue;
    if (!mapping) {
      return false;
    }
    let seeded = mapping.viewport();
    if (this.viewX) {
      seeded = take(seeded, seeded.withX(this.viewX[0], this.viewX[1]));
    }
    if (this.viewY) {
      seeded = take(seeded, seeded.withY(this.viewY[0], this.viewY[1]));
    }
    const next = transform(seeded);
    if (next !== seeded) {
      seeded.dispose();
    }
    this.viewX = next.x;
    this.viewY = next.y;
    next.dispose();
    return true;
  }

  private xIsBands(): boolean {
    return this.mappingValue?.xCategories != null;
  }
}

/**
 * Share an x window and mirror the active pane's cursor into the passive
 * one. Zoom and pan in either pane and both move; each pane keeps its own y.
 */
export function linkX(active: PlotState, passive: PlotState): void {
  const window = active.viewport().x;
  const view = passive.viewport();
  passive.setViewport(window ? { x: window, y: view.y } : { y: view.y });
  const x = active.hoverDataX();
  passive.hoverX(x ?? Number.NaN);
}

function take<T extends { dispose(): void }>(previous: T, next: T): T {
  if (previous !== next) {
    previous.dispose();
  }
  return next;
}

function sameHover(a: Hover | undefined, b: Hover | undefined): boolean {
  if (a === b) {
    return true;
  }
  if (!a || !b || a.kind !== b.kind) {
    return false;
  }
  if (a.kind === "cell" && b.kind === "cell") {
    return a.column === b.column && a.row === b.row;
  }
  if (a.kind === "column" && b.kind === "column") {
    return a.column === b.column;
  }
  return false;
}

function clampInto(rect: Panel, column: number, row: number): [number, number] {
  const right = Math.max(rect.column, rect.column + rect.width - 1);
  const bottom = Math.max(rect.row, rect.row + rect.height - 1);
  return [
    Math.min(Math.max(column, rect.column), right),
    Math.min(Math.max(row, rect.row), bottom),
  ];
}
