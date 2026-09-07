import assert from "node:assert/strict";
import { test } from "node:test";
import { Theme } from "../color.js";
import { Frame } from "../frame.js";
import { Line } from "../mark.js";
import { Plot } from "../plot.js";
import { bar } from "../presets.js";
import { applyOverlays } from "../ink/overlays.js";
import { parseMouse } from "../ink/mouse.js";
import { PlotState, linkX } from "../ink/PlotState.js";

function statefulPlot(): Plot {
  return new Plot()
    .layer(Line.xy([0, 10], [0, 10]))
    .xDomain(0, 10)
    .yDomain(0, 10);
}

function renderStateful(
  plot: Plot,
  state: PlotState,
  width = 50,
  height = 18,
  origin = { column: 0, row: 0 },
) {
  const frame = new Frame({
    width,
    height,
    charset: "Quadrants",
    color: "TrueColor",
    theme: Theme.DARK,
  });
  const drawn = plot.viewport(state.viewport());
  const raster = drawn.raster(frame);
  state.capture(drawn.mapping(frame), {
    column: origin.column,
    row: origin.row,
    width,
    height,
  });
  return { frame, raster };
}

test("a stateful render hit-tests through the area offset", () => {
  const plot = statefulPlot();
  const state = new PlotState();
  assert.equal(state.dataAt(10, 10), undefined);
  renderStateful(plot, state, 50, 18, { column: 5, row: 3 });
  const rect = state.plotArea();
  assert.ok(rect);
  assert.ok(rect.column >= 5 && rect.row >= 3);
  const data = state.dataAt(
    rect.column + Math.floor(rect.width / 2),
    rect.row + Math.floor(rect.height / 2),
  );
  assert.ok(data);
  assert.ok(data[0] >= 0 && data[0] <= 10, `x in domain: ${data[0]}`);
  assert.ok(data[1] >= 0 && data[1] <= 10, `y in domain: ${data[1]}`);
  assert.equal(state.dataAt(0, 0), undefined);
});

test("hovering tracks the cursor only inside the panel", () => {
  const plot = statefulPlot();
  const state = new PlotState();
  renderStateful(plot, state);
  const rect = state.plotArea()!;
  const inside = { column: rect.column + 2, row: rect.row + 2 };

  assert.equal(state.onMouse({ kind: "moved", ...inside }), true);
  assert.deepEqual(state.cursor(), [inside.column, inside.row]);
  assert.ok(state.cursorData());
  assert.equal(state.onMouse({ kind: "moved", ...inside }), false, "no change");
  assert.equal(state.onMouse({ kind: "moved", column: 0, row: 0 }), true, "leaving clears");
  assert.equal(state.cursor(), undefined);
});

test("the wheel zooms x around the cursor and reset returns to auto", () => {
  const plot = statefulPlot();
  const state = new PlotState();
  renderStateful(plot, state);
  const rect = state.plotArea()!;
  const at = {
    column: rect.column + Math.floor(rect.width / 2),
    row: rect.row + Math.floor(rect.height / 2),
  };
  const anchor = state.dataAt(at.column, at.row)![0];

  assert.equal(state.onMouse({ kind: "scrollUp", ...at }), true);
  const x = state.viewport().x;
  assert.ok(x, "the wheel fixed x");
  assert.ok(x[1] - x[0] < 10, "narrower than the full domain");
  assert.ok(x[0] <= anchor && anchor <= x[1], "anchor stays visible");
  assert.equal(state.viewport().y, undefined, "y stays automatic");

  state.resetView();
  assert.equal(state.viewport().x, undefined);
  assert.equal(state.viewport().y, undefined);
  assert.equal(state.onMouse({ kind: "scrollUp", column: 0, row: 0 }), false);
});

test("a left drag pans the view and stacked drags compound", () => {
  const plot = statefulPlot();
  const state = new PlotState();
  renderStateful(plot, state);
  const rect = state.plotArea()!;
  const start = {
    column: rect.column + Math.floor(rect.width / 2),
    row: rect.row + Math.floor(rect.height / 2),
  };
  const perCell = 10 / rect.width;

  assert.equal(
    state.onMouse({ kind: "down", button: "left", ...start }),
    true,
  );
  assert.equal(
    state.onMouse({
      kind: "drag",
      button: "left",
      column: start.column + 3,
      row: start.row,
    }),
    true,
  );
  assert.equal(
    state.onMouse({
      kind: "drag",
      button: "left",
      column: start.column + 6,
      row: start.row,
    }),
    true,
  );
  const x = state.viewport().x!;
  const expected = -6 * perCell;
  assert.ok(
    Math.abs(x[0] - expected) < 1e-9,
    `six cells of drag compound: lo=${x[0]}, expected ${expected}`,
  );
  assert.ok(Math.abs(x[1] - x[0] - 10) < 1e-9, "the span is preserved");
});

test("a right drag zooms to the selection and a click does not", () => {
  const plot = statefulPlot();
  const state = new PlotState();
  renderStateful(plot, state);
  const rect = state.plotArea()!;
  const a = { column: rect.column + 2, row: rect.row + 2 };
  const b = {
    column: rect.column + rect.width - 3,
    row: rect.row + rect.height - 3,
  };

  assert.equal(state.onMouse({ kind: "down", button: "right", ...a }), true);
  assert.equal(state.onMouse({ kind: "drag", button: "right", ...b }), true);
  assert.equal(state.onMouse({ kind: "up", button: "right", ...b }), true);
  const x = state.viewport().x!;
  const y = state.viewport().y!;
  assert.ok(x[0] < x[1] && y[0] < y[1]);
  assert.ok(x[1] - x[0] < 10 && y[1] - y[0] < 10, "narrower than the domain");

  state.resetView();
  assert.equal(state.onMouse({ kind: "down", button: "right", ...a }), true);
  assert.equal(
    state.onMouse({
      kind: "up",
      button: "right",
      column: a.column + 1,
      row: a.row,
    }),
    true,
  );
  assert.equal(state.viewport().x, undefined, "a click-sized selection is discarded");
  assert.equal(state.viewport().y, undefined);
});

test("keyboard zoom, pan, and reset", () => {
  const plot = statefulPlot();
  const state = new PlotState();
  renderStateful(plot, state);
  assert.equal(state.zoomIn(), true);
  const zoomed = state.viewport().x!;
  assert.ok(zoomed[1] - zoomed[0] < 10);
  assert.equal(state.viewport().y, undefined);
  assert.equal(state.panRight(), true);
  const panned = state.viewport().x!;
  assert.ok(panned[0] > zoomed[0]);
  state.resetView();
  assert.equal(state.viewport().x, undefined);
});

test("hoverX mirrors a linked pane without inventing a row", () => {
  const plot = statefulPlot();
  const active = new PlotState();
  const passive = new PlotState();
  renderStateful(plot, active);
  renderStateful(plot, passive);
  const rect = active.plotArea()!;
  active.onMouse({
    kind: "moved",
    column: rect.column + 4,
    row: rect.row + 4,
  });
  linkX(active, passive);
  assert.equal(passive.cursor(), undefined, "mirrored hover has no row");
  assert.ok(passive.hoverDataX() !== undefined);
  const window = active.viewport().x;
  assert.deepEqual(passive.viewport().x, window);
});

test("a bands axis does not zoom", () => {
  const plot = bar(["a", "b", "c"], [3, 7, 5]);
  const state = new PlotState();
  renderStateful(plot, state);
  const rect = state.plotArea()!;
  assert.equal(
    state.onMouse({
      kind: "scrollUp",
      column: rect.column + 2,
      row: rect.row + 2,
    }),
    false,
  );
  assert.equal(state.viewport().x, undefined);
});

test("crosshair tints unset backgrounds and readout writes coordinates", () => {
  const plot = statefulPlot();
  const state = new PlotState();
  const { raster } = renderStateful(plot, state, 60, 20);
  const rect = state.plotArea()!;
  const at = {
    column: rect.column + Math.floor(rect.width / 2),
    row: rect.row + Math.floor(rect.height / 2),
  };
  state.onMouse({ kind: "moved", ...at });
  const painted = applyOverlays(raster, state, plot);
  const columnTinted = [...Array(rect.height).keys()].some((i) => {
    const cell = painted.cell(at.column, rect.row + i);
    return cell?.background === "BrightBlack" || cell?.background === "White";
  });
  assert.ok(columnTinted, "crosshair column tinted");
  const top = [...Array(rect.width).keys()]
    .map((i) => painted.cell(rect.column + i, rect.row)?.glyph ?? " ")
    .join("");
  assert.ok(top.includes("·"), `readout separator present: ${JSON.stringify(top)}`);
});

test("snap lists the nearest datum, gaps as em dash", () => {
  const plot = new Plot()
    .layer(Line.xy([0, 1, 2, 3], [1, Number.NaN, 4, 8]).label("loss"))
    .xDomain(0, 3)
    .yDomain(0, 8);
  const state = new PlotState();
  const { raster } = renderStateful(plot, state, 40, 12);
  const rect = state.plotArea()!;
  let at = { column: rect.column + 2, row: rect.row + 2 };
  let nearest = Infinity;
  for (let column = rect.column; column < rect.column + rect.width; column++) {
    const data = state.dataAt(column, rect.row);
    if (!data) {
      continue;
    }
    const distance = Math.abs(data[0] - 1);
    if (distance < nearest) {
      nearest = distance;
      at = { column, row: rect.row + 2 };
    }
  }
  assert.equal(state.onMouse({ kind: "moved", ...at }), true);
  const painted = applyOverlays(raster, state, plot, { crosshair: false });
  const top = [...Array(painted.width).keys()]
    .map((i) => painted.cell(i, rect.row)?.glyph ?? "")
    .join("");
  assert.ok(top.includes("loss"), `snapped label: ${JSON.stringify(top)}`);
  assert.ok(top.includes("—"), `gap reads as em dash: ${JSON.stringify(top)}`);
});

test("mapping.viewport exposes window arithmetic", () => {
  const mapping = statefulPlot().mapping(new Frame({
    width: 50,
    height: 18,
    charset: "Quadrants",
    color: "TrueColor",
    theme: Theme.DARK,
  }));
  const view = mapping.viewport();
  assert.equal(view.isAuto, false, "a seeded mapping fixes both axes");
  const x = view.x as [number, number];
  const zoomed = view.zoomX(0.8, (x[0] + x[1]) / 2);
  const zx = zoomed.x as [number, number];
  assert.ok(zx[1] - zx[0] < x[1] - x[0]);
  assert.equal(view.resetY().y, undefined);
  assert.ok(view.withX(1, 4).x);
});

test("parseMouse reads SGR sequences", () => {
  assert.deepEqual(parseMouse("\u001b[<35;10;4M"), {
    kind: "moved",
    column: 9,
    row: 3,
  });
  assert.deepEqual(parseMouse("\u001b[<0;2;3M"), {
    kind: "down",
    button: "left",
    column: 1,
    row: 2,
  });
  assert.deepEqual(parseMouse("\u001b[<32;5;6M"), {
    kind: "drag",
    button: "left",
    column: 4,
    row: 5,
  });
  assert.deepEqual(parseMouse("\u001b[<2;8;9m"), {
    kind: "up",
    button: "right",
    column: 7,
    row: 8,
  });
  assert.deepEqual(parseMouse("\u001b[<64;3;3M"), {
    kind: "scrollUp",
    column: 2,
    row: 2,
  });
  assert.equal(parseMouse("a"), undefined);
});
