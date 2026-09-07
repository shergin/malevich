import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
import { engine } from "../engine.js";
import { Frame } from "../frame.js";
import { Plot } from "../plot.js";

type Golden = {
  name: string;
  document: { version: 1; kind: "plot"; spec: Record<string, unknown> };
  frame: {
    width: number;
    height: number;
    charset: string;
    color: string;
    theme: { palette: string[] };
  };
  expected: string;
};

const path = join(
  dirname(fileURLToPath(import.meta.url)),
  "../../../tests/fixtures/js/goldens.json",
);

test("wasm renders the shared rust goldens", () => {
  const goldens = JSON.parse(readFileSync(path, "utf8")) as Golden[];
  assert.ok(goldens.length >= 8, "fixture is populated");
  for (const golden of goldens) {
    const got = engine().render_document(
      JSON.stringify(golden.document),
      JSON.stringify(golden.frame),
    );
    assert.equal(got, golden.expected, golden.name);
  }
});

test("js builders match the line and stacked-bar goldens", () => {
  const goldens = JSON.parse(readFileSync(path, "utf8")) as Golden[];
  const lineGolden = goldens.find((g) => g.name === "line");
  assert.ok(lineGolden);
  const frame = Frame.plain(lineGolden.frame.width, lineGolden.frame.height);
  const fromDoc = Plot.fromDocument(lineGolden.document as never);
  assert.equal(fromDoc.render(frame), lineGolden.expected);
});
