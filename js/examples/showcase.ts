/**
 * A colored tour of the JS rim, rendered for *your* terminal.
 *
 * Analog of `cargo run --example showcase`. Uses Frame.detect(): charts size
 * themselves to the terminal width, use color when the terminal has any, and
 * degrade to plain text when piped. Pixel side-by-side is Phase 4 — this tour
 * is cells, which is the honest first look.
 *
 *   cd js && npm run build && npm run showcase
 */

import {
  Area,
  Color,
  Frame,
  Grid,
  Line,
  Plot,
  Points,
  Rule,
  Text,
  bar,
  boxPlot,
  density,
  describe,
  ecdf,
  heatmap,
  hist,
  hist2d,
  line,
  scatter,
  stairs,
  trend,
  violin,
} from "../dist/index.js";

const frame = Frame.detect();

function show(plot: Plot, size: Frame = frame): void {
  console.log(`${plot.render(size)}\n`);
}

const steps = Float64Array.from({ length: 120 }, (_, i) => i);
const train = Float64Array.from(
  steps,
  (s) => 3.8 * Math.exp(-0.035 * s) + 0.32 + 0.05 * Math.sin(s * 0.7),
);
const val = Float64Array.from(
  steps,
  (s) => 4.0 * Math.exp(-0.03 * s) + 0.55 + 0.08 * Math.cos(s * 0.35),
);

show(
  new Plot()
    .layer(Line.xy(steps, train).label("train"))
    .layer(Line.xy(steps, val).label("val"))
    .layer(Rule.h(0.5).label("target"))
    .layer(Text.at(60, 2, "< converging"))
    .title("loss with annotations (synthetic)")
    .xLabel("step")
    .yLabel("loss"),
);

show(
  describe(["train", "val"], [train, val]).title("the loss curves, described (synthetic)"),
  frame.with({ height: 8 }),
);

const vermilion = Color.rgb(227, 66, 52);
const sky = Color.rgb(108, 153, 212);
const trainSmooth = ewma(train, 0.9);
const valSmooth = ewma(val, 0.9);
show(
  new Plot()
    .layer(Area.xy(steps, trainSmooth).color(vermilion).opacity(0.13))
    .layer(Line.xy(steps, trainSmooth).label("train").color(vermilion).glow())
    .layer(Line.xy(steps, valSmooth).label("val").color(sky).dash("Dotted"))
    .layer(Rule.h(0.5).label("target").dash("Dashed"))
    .title("glow over a wash, dashed annotations (synthetic)")
    .xLabel("step")
    .yLabel("loss"),
);

show(
  new Plot()
    .layer(Line.y(train).style("Corners").label("train"))
    .title("the corners style")
    .xLabel("step"),
);

let state = 0x9e3779b97f4a7c15n;
const unit = (): number => {
  state ^= state << 13n;
  state ^= state >> 7n;
  state ^= state << 17n;
  state &= 0xffff_ffff_ffff_ffffn;
  return Number(state >> 11n) / 2 ** 53;
};
const cloudX: number[] = [];
const cloudY: number[] = [];
for (let index = 0; index < 15_000; index++) {
  const center = index % 3 === 0 ? ([2.4, 1.2] as const) : ([1.0, 0.8] as const);
  const sample = (spread: number) => {
    let sum = 0;
    for (let k = 0; k < 6; k++) sum += unit();
    return (sum / 6) * spread - spread / 2;
  };
  cloudX.push(center[0] + sample(1.6));
  cloudY.push(center[1] + sample(1.1));
}
show(
  new Plot()
    .layer(
      Points.xy(cloudX, cloudY)
        .color(Color.rgb(86, 178, 163))
        .opacity(0.18)
        .density(),
    )
    .title("15,000 points as accumulated ink (synthetic)"),
);

const stamps = Float64Array.from({ length: 36 }, (_, i) => {
  const year = 2024 + Math.floor(i / 12);
  const month = i % 12;
  return Date.UTC(year, month, 1) / 1000;
});
const level = Float64Array.from(
  { length: 36 },
  (_, i) => 400 + i * 0.2 + Math.sin((i % 12) * 0.52) * 3,
);
show(
  new Plot()
    .layer(Line.xy(stamps, level))
    .title("a monthly series on a calendar axis (synthetic)")
    .timeX(),
);

const raw = Float64Array.from(
  { length: 120 },
  (_, i) => 3 * Math.exp(-0.03 * i) + 0.4 + ((i * 7) % 13) * 0.06,
);
show(
  new Plot()
    .layer(Line.y(raw).label("raw"))
    .layer(Line.y(rollingMean(raw, 9)).label("rolling mean"))
    .title("smoothing (synthetic)"),
);

const n = 10_000_000;
const wave = new Float64Array(n);
for (let i = 0; i < n; i++) {
  wave[i] = Math.sin(i * 0.0002) * Math.cos(i * 0.000013) * 8;
}
show(line(wave).title("10,000,000 points through M4"));

show(
  bar(
    ["rust", "go", "python", "typescript", "zig"],
    [68, 41, 55, 62, 12],
  ).title("admired languages, % (synthetic)"),
);

const samples = Float64Array.from({ length: 4000 }, (_, i) => {
  return (Math.sin(i * 0.731) + Math.sin(i * 1.13) + Math.sin(i * 2.71)) * 2 + 10;
});
show(hist(samples).title("histogram, automatic bins"));

const adelie = Float64Array.from({ length: 80 }, (_, i) => 170 + ((i * 17) % 23) * 0.4 + unit());
const chinstrap = Float64Array.from({ length: 60 }, (_, i) => 185 + ((i * 13) % 19) * 0.5 + unit());
const gentoo = Float64Array.from({ length: 90 }, (_, i) => 205 + ((i * 11) % 29) * 0.45 + unit());
show(
  boxPlot(["Adelie", "Chinstrap", "Gentoo"], [adelie, chinstrap, gentoo]).title(
    "flipper length by species (synthetic)",
  ),
);
show(
  violin(["Adelie", "Chinstrap", "Gentoo"], [adelie, chinstrap, gentoo]).title(
    "the same groups as violins",
  ),
);

show(density(samples).title("gaussian KDE"));
show(ecdf(samples).title("empirical CDF"));

const xs = Float64Array.from({ length: 80 }, (_, i) => i * 0.1);
const ys = Float64Array.from(xs, (x) => 0.4 * x + 1.2 + Math.sin(x * 3) * 0.3);
show(trend(xs, ys).title("least-squares trend with a band"));

const fieldSize = 24;
const field = Float64Array.from({ length: fieldSize * fieldSize }, (_, i) => {
  const fx = (i % fieldSize) / 4;
  const fy = Math.floor(i / fieldSize) / 4;
  return (fx - 3) ** 2 * 0.4 + (fy - 2.6) ** 2 * 0.7 + Math.sin(fx * 1.7) * Math.cos(fy * 1.3) * 0.8;
});
show(heatmap(fieldSize, field).colorbar().title("a loss landscape (synthetic)"));

const blobX: number[] = [];
const blobY: number[] = [];
for (let i = 0; i < 2000; i++) {
  blobX.push(unit() * 4 + (i % 2) * 1.2);
  blobY.push(unit() * 3 + (i % 3) * 0.4);
}
show(hist2d(blobX, blobY).title("2D histogram"));

const species = ["Adelie", "Chinstrap", "Gentoo"];
const scatterX: number[] = [];
const scatterY: number[] = [];
const group: string[] = [];
for (let i = 0; i < 180; i++) {
  const kind = species[i % 3]!;
  group.push(kind);
  scatterX.push(32 + (i % 3) * 4 + unit() * 3);
  scatterY.push(170 + (i % 3) * 18 + unit() * 8);
}
show(
  new Plot()
    .layer(Points.xy(scatterX, scatterY).colorBy(group))
    .title("bill depth vs flipper, colored by species (synthetic)")
    .xLabel("bill depth")
    .yLabel("flipper"),
);

show(
  stairs(Float64Array.from({ length: 24 }, (_, i) => ((i * 5) % 11) + Math.sin(i))).title(
    "stairs",
  ),
);

const sine = Float64Array.from({ length: 80 }, (_, i) => Math.sin(i / 6));
const cosine = Float64Array.from({ length: 80 }, (_, i) => Math.cos(i / 6));
show(
  new Grid(2)
    .with(line(sine).title("sin"))
    .with(line(cosine).title("cos"))
    .with(scatter(sine, cosine).title("phase"))
    .with(hist(sine).title("sin, binned")),
  frame.with({ height: 18 }),
);

function ewma(values: Float64Array, alpha: number): Float64Array {
  const out = new Float64Array(values.length);
  let mean = 0;
  let started = false;
  for (let i = 0; i < values.length; i++) {
    const value = values[i]!;
    if (!Number.isFinite(value)) {
      out[i] = Number.NaN;
      continue;
    }
    mean = started ? alpha * value + (1 - alpha) * mean : value;
    started = true;
    out[i] = mean;
  }
  return out;
}

function rollingMean(values: Float64Array, window: number): Float64Array {
  const out = new Float64Array(values.length);
  for (let i = 0; i < values.length; i++) {
    let sum = 0;
    let count = 0;
    const start = Math.max(0, i - window + 1);
    for (let j = start; j <= i; j++) {
      const value = values[j]!;
      if (Number.isFinite(value)) {
        sum += value;
        count += 1;
      }
    }
    out[i] = count === 0 ? Number.NaN : sum / count;
  }
  return out;
}
