import {
  asPair,
  asPanel,
  drop,
  engine,
  type JsMapping,
  type JsViewport,
  type Panel,
} from "./engine.js";
const registry = new FinalizationRegistry<{ free?(): void }>((value) => {
  drop(value);
});

/**
 * Resolved geometry of one render, as a JS value. Hit-testing, axis-formatted
 * labels, the plot rectangle, a viewport seed. The wasm handle is private;
 * you never call `free()`.
 */
export class Mapping {
  readonly #inner: JsMapping;

  private constructor(inner: JsMapping) {
    this.#inner = inner;
    registry.register(this, inner, this);
  }

  /** @internal */
  static fromWasm(inner: JsMapping): Mapping {
    return new Mapping(inner);
  }

  dataAt(column: number, row: number): [number, number] | undefined {
    return asPair(this.#inner.dataAt(column, row));
  }

  cellAt(x: number, y: number): [number, number] | undefined {
    return asPair(this.#inner.cellAt(x, y));
  }

  columnAt(x: number): number | undefined {
    const column = this.#inner.columnAt(x);
    return typeof column === "number" && Number.isFinite(column) ? column : undefined;
  }

  formatX(value: number): string {
    return this.#inner.formatX(value);
  }

  formatY(value: number): string {
    return this.#inner.formatY(value);
  }

  get xDomain(): [number, number] {
    return asPair(this.#inner.xDomain) ?? [0, 1];
  }

  get yDomain(): [number, number] {
    return asPair(this.#inner.yDomain) ?? [0, 1];
  }

  get plotArea(): Panel | undefined {
    return asPanel(this.#inner.plotArea);
  }

  get xCategories(): string[] | undefined {
    const value = this.#inner.xCategories;
    return Array.isArray(value) ? value : undefined;
  }

  get yCategories(): string[] | undefined {
    const value = this.#inner.yCategories;
    return Array.isArray(value) ? value : undefined;
  }

  /** Seeds a viewport from this mapping's resolved domains (and log flags). */
  viewport(): Viewport {
    return Viewport.fromWasm(this.#inner.viewport());
  }

  /** @internal */
  dispose(): void {
    registry.unregister(this);
    drop(this.#inner);
  }
}

/**
 * An axis window pair with zoom/pan arithmetic. Windows are plain data;
 * transforms go through the engine so log axes stay in decade space.
 */
export class Viewport {
  readonly #inner: JsViewport;

  private constructor(inner: JsViewport) {
    this.#inner = inner;
    registry.register(this, inner, this);
  }

  /** @internal */
  static fromWasm(inner: JsViewport): Viewport {
    return new Viewport(inner);
  }

  static auto(): Viewport {
    return Viewport.fromWasm(engine().JsViewport.auto());
  }

  get x(): [number, number] | undefined {
    return asPair(this.#inner.x);
  }

  get y(): [number, number] | undefined {
    return asPair(this.#inner.y);
  }

  get isAuto(): boolean {
    return Boolean(this.#inner.isAuto);
  }

  /** The window pair `Plot.viewport` accepts. */
  windows(): { x?: [number, number]; y?: [number, number] } {
    return { x: this.x, y: this.y };
  }

  zoomX(factor: number, anchor: number): Viewport {
    return Viewport.fromWasm(this.#inner.zoomX(factor, anchor));
  }

  zoomY(factor: number, anchor: number): Viewport {
    return Viewport.fromWasm(this.#inner.zoomY(factor, anchor));
  }

  panX(fraction: number): Viewport {
    return Viewport.fromWasm(this.#inner.panX(fraction));
  }

  panY(fraction: number): Viewport {
    return Viewport.fromWasm(this.#inner.panY(fraction));
  }

  withX(low: number, high: number): Viewport {
    return Viewport.fromWasm(this.#inner.withX(low, high));
  }

  withY(low: number, high: number): Viewport {
    return Viewport.fromWasm(this.#inner.withY(low, high));
  }

  reset(): Viewport {
    return Viewport.fromWasm(this.#inner.reset());
  }

  resetX(): Viewport {
    return Viewport.fromWasm(this.#inner.resetX());
  }

  resetY(): Viewport {
    return Viewport.fromWasm(this.#inner.resetY());
  }

  clampX(low: number, high: number): Viewport {
    return Viewport.fromWasm(this.#inner.clampX(low, high));
  }

  clampY(low: number, high: number): Viewport {
    return Viewport.fromWasm(this.#inner.clampY(low, high));
  }

  tail(latest: number, width: number): Viewport {
    return Viewport.fromWasm(this.#inner.tail(latest, width));
  }

  /** @internal */
  dispose(): void {
    registry.unregister(this);
    drop(this.#inner);
  }
}
