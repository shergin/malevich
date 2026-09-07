import assert from "node:assert/strict";
import { test } from "node:test";
import { toFloat64 } from "../series.js";

test("null and undefined become NaN gaps", () => {
  const series = toFloat64([1, null, 3, undefined]);
  assert.equal(series.length, 4);
  assert.equal(series[0], 1);
  assert.ok(Number.isNaN(series[1]));
  assert.equal(series[2], 3);
  assert.ok(Number.isNaN(series[3]));
});

test("Float64Array is kept by reference", () => {
  const values = new Float64Array([1, 2, 3]);
  assert.equal(toFloat64(values), values);
});

test("Float32Array converts once", () => {
  const series = toFloat64(new Float32Array([0.5, 1.5]));
  assert.equal(series.length, 2);
  assert.equal(series[0], 0.5);
  assert.equal(series[1], 1.5);
});
