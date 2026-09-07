import assert from "node:assert/strict";
import { test } from "node:test";
import React from "react";
import { renderToString } from "ink";
import { line } from "../index.js";
import { PlotWidget } from "../ink/PlotWidget.js";
import { PlotColumn } from "../ink/PlotColumn.js";
import { Text } from "ink";

test("PlotWidget paints a line into Ink output", () => {
  const output = renderToString(
    React.createElement(PlotWidget, {
      plot: line([1, 5, 2, 8]).title("w"),
      width: 40,
      height: 10,
      charset: "Braille",
    }),
  );
  assert.ok(output.includes("w"));
  assert.ok(output.length > 20);
});

test("PlotColumn injects origin from siblings above", () => {
  const output = renderToString(
    React.createElement(
      PlotColumn,
      null,
      React.createElement(Text, null, "header"),
      React.createElement(PlotWidget, {
        plot: line([1, 2, 3]).title("below"),
        width: 24,
        height: 8,
        charset: "Braille",
      }),
    ),
  );
  assert.ok(output.includes("header"));
  assert.ok(output.includes("below"));
});
