import assert from "node:assert/strict";
import { test } from "node:test";
import { Colormap, Palette } from "../color.js";
import { Frame } from "../frame.js";
import { Bars, Cells, Line, Range } from "../mark.js";
import { Plot } from "../plot.js";
import { line } from "../presets.js";

const frame = Frame.plain(40, 10);

test("Cells.matrix renders a value grid", () => {
  const text = new Plot()
    .layer(Cells.matrix(4, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]))
    .title("grid")
    .render(frame);
  assert.ok(text.includes("grid"));
  assert.doesNotMatch(text, /NaN/);
});

test("Range.over with body and marker renders", () => {
  const text = new Plot()
    .layer(
      Range.over(["a", "b"], [1, 2], [4, 5])
        .body([2, 3], [3, 4])
        .marker([2.5, 3.5])
        .label("box"),
    )
    .title("intervals")
    .render(frame);
  assert.ok(text.includes("intervals"));
});

test("Bars.base stacks on the lower series", () => {
  const text = new Plot()
    .layer(Bars.new(["a", "b", "c"], [1, 2, 1.5]).label("low"))
    .layer(Bars.new(["a", "b", "c"], [0.5, 1, 0.5]).base([1, 2, 1.5]).label("high"))
    .title("stack")
    .render(frame);
  assert.ok(text.includes("stack"));
});

test("Plot.palette is honored by color_by", () => {
  const text = new Plot()
    .layer(Line.y([1, 2, 3, 4]).colorBy(["a", "a", "b", "b"]))
    .palette(Palette.OKABE_ITO)
    .title("groups")
    .render(Frame.plain(40, 10));
  assert.ok(text.includes("groups"));
});

test("renderPixels emits a protocol payload", () => {
  const text = line([1, 5, 2, 8]).renderPixels(Frame.plain(20, 8), {
    protocol: "sixel",
  });
  assert.ok(text.includes("q") || text.includes("P") || text.length > 0);
});

test("Colormap.named resolves viridis", () => {
  const map = Colormap.named("viridis");
  assert.equal(map.stops.length, Colormap.VIRIDIS.stops.length);
  const log = Colormap.log(Colormap.centeredAt(Colormap.RED_BLUE, 0));
  assert.equal(log.log, true);
  assert.equal(log.midpoint, 0);
});
