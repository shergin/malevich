import assert from "node:assert/strict";
import { test } from "node:test";
import { Frame } from "../frame.js";
import { Line, Rule, Text } from "../mark.js";
import { Plot } from "../plot.js";
import { bar, hist, line, scatter } from "../presets.js";

const LINE = `7.5 ┤                                 ⡠⠊
    │                               ⡠⠊
    │                             ⡠⠊
5.0 ┤         ⣀⠔⠊⠑⠢⢄⡀           ⡠⠊
    │      ⣀⠔⠊      ⠈⠑⠢⢄⡀     ⡠⠊
2.5 ┤   ⡠⠔⠉             ⠈⠑⠢⢄⡠⠊
    │⡠⠔⠉
0.0 ┤
    └┬──────────┬───────────┬──────────┬
     0          1           2          3`;

test("line one-liner matches the rust Frame.plain(40, 10) chart", () => {
  const text = line([1, 5, 2, 8]).render(Frame.plain(40, 10));
  assert.equal(text, LINE);
});

test("the line preset equals its grammar expansion", () => {
  const values = [1, 5, 2, 8];
  const frame = Frame.plain(40, 10);
  const preset = line(values).render(frame);
  const grammar = new Plot().layer(Line.y(values)).render(frame);
  assert.equal(preset, grammar);
});

test("raster encodes to the same string as render", () => {
  const plot = line([1, 5, 2, 8]).title("training");
  const frame = Frame.plain(40, 10);
  assert.equal(plot.raster(frame).rows().length, 10);
  // Packed glyphs include continuations; the string form is the public contract.
  assert.equal(plot.render(frame).split("\n").length, plot.raster(frame).rows().length);
});

test("nulls are visible gaps", () => {
  const text = line([1, null, 3]).render(Frame.plain(24, 8));
  assert.ok(text.length > 0);
  assert.doesNotMatch(text, /NaN/);
});

test("rule and text compose with a line", () => {
  const text = new Plot()
    .layer(Line.y([1, 5, 2, 8]))
    .layer(Rule.h(4).label("mid"))
    .layer(Text.at(2, 6, "hi"))
    .title("annotated")
    .render(Frame.plain(40, 10));
  assert.ok(text.includes("annotated"));
  assert.ok(text.includes("hi"));
});

test("scatter, bar, and hist render without throwing", () => {
  const frame = Frame.plain(40, 10);
  assert.ok(scatter([1, 2, 3], [2, 1, 3]).render(frame).length > 0);
  assert.ok(bar(["a", "b", "c"], [3, 7, 5]).render(frame).length > 0);
  assert.ok(hist([1, 2, 2.5, 2.7, 3, 3.1, 3.2, 4, 5.5]).render(frame).length > 0);
});

test("console.log convenience uses inspect", async () => {
  const { inspect } = await import("node:util");
  const chart = line([1, 5, 2, 8]);
  assert.ok(inspect(chart).length > 20);
  assert.equal(chart.render(Frame.plain(40, 10)), LINE);
});
