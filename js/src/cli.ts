#!/usr/bin/env node
/**
 * `npx malevich` — a tour of the JS rim, or a stdin plotter.
 *
 *   npx malevich              showcase
 *   npx malevich line         numbers on stdin
 *   npx malevich hist
 *   npx malevich bar          category<TAB>value
 */

import { readFileSync } from "node:fs";
import {
  Color,
  Frame,
  Line,
  Plot,
  Rule,
  Text,
  bar,
  boxPlot,
  hist,
  line,
} from "./index.js";

const command = process.argv[2] ?? "showcase";

if (command === "-h" || command === "--help") {
  process.stdout.write(`malevich — terminal plotting

Usage:
  npx malevich              colored tour sized to this terminal
  npx malevich line         plot stdin numbers as a line
  npx malevich hist         plot stdin numbers as a histogram
  npx malevich bar          plot stdin as category<TAB>value bars
`);
  process.exit(0);
}

if (command === "showcase") {
  showcase();
  process.exit(0);
}

const input = readStdin();
const frame = Frame.detect();

if (command === "line") {
  const numbers = parseNumbers(input);
  if (numbers.length === 0) {
    fail("malevich line: expected numbers on stdin");
  }
  process.stdout.write(`${line(numbers).render(frame)}\n`);
} else if (command === "hist") {
  const numbers = parseNumbers(input);
  if (numbers.length === 0) {
    fail("malevich hist: expected numbers on stdin");
  }
  process.stdout.write(`${hist(numbers).render(frame)}\n`);
} else if (command === "bar") {
  const rows = parseBars(input);
  if (rows.categories.length === 0) {
    fail("malevich bar: expected lines of `category<TAB>value`");
  }
  process.stdout.write(`${bar(rows.categories, rows.values).render(frame)}\n`);
} else {
  fail(`unknown command '${command}' (try --help)`);
}

function showcase(): void {
  const frame = Frame.detect();
  const steps = Float64Array.from({ length: 80 }, (_, i) => i);
  const train = Float64Array.from(
    steps,
    (s) => 3.8 * Math.exp(-0.035 * s) + 0.32 + 0.05 * Math.sin(s * 0.7),
  );
  const val = Float64Array.from(
    steps,
    (s) => 4.0 * Math.exp(-0.03 * s) + 0.55 + 0.08 * Math.cos(s * 0.35),
  );
  const show = (plot: Plot, size = frame): void => {
    process.stdout.write(`${plot.render(size)}\n\n`);
  };
  show(
    new Plot()
      .layer(Line.xy(steps, train).label("train").color(Color.Cyan))
      .layer(Line.xy(steps, val).label("val").color(Color.Yellow))
      .layer(Rule.h(0.5).label("target").dash("Dotted"))
      .layer(Text.at(50, 2, "converging"))
      .title("npx malevich")
      .xLabel("step")
      .yLabel("loss"),
  );
  show(hist(train).title("train, as a histogram"));
  show(bar(["mon", "tue", "wed", "thu", "fri"], [3, 7, 4.5, 8, 6]).title("week"));
  show(boxPlot(["train", "val"], [train, val]).title("the same series, as boxes"));
}

function readStdin(): string {
  try {
    return readFileSync(0, "utf8");
  } catch {
    return "";
  }
}

function parseNumbers(text: string): number[] {
  const values: number[] = [];
  for (const token of text.split(/[\s,;]+/)) {
    if (!token) {
      continue;
    }
    const value = Number(token);
    if (Number.isFinite(value)) {
      values.push(value);
    }
  }
  return values;
}

function parseBars(text: string): { categories: string[]; values: number[] } {
  const categories: string[] = [];
  const values: number[] = [];
  for (const lineText of text.split("\n")) {
    const trimmed = lineText.trim();
    if (!trimmed) {
      continue;
    }
    const [category, raw] = trimmed.split(/\t|,/);
    const value = Number(raw);
    if (category && Number.isFinite(value)) {
      categories.push(category);
      values.push(value);
    }
  }
  return { categories, values };
}

function fail(message: string): never {
  process.stderr.write(`${message}\n`);
  process.exit(1);
}
