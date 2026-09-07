const RED_BLUE = {
  stops: [
    [202, 0, 32],
    [244, 165, 130],
    [247, 247, 247],
    [146, 197, 222],
    [5, 113, 176],
  ],
  midpoint: 0,
};

const VERMILION = { Rgb: [227, 66, 52] };
const SKY = { Rgb: [108, 153, 212] };

function ewma(values, alpha) {
  const out = new Float64Array(values.length);
  let mean = 0;
  let started = false;
  for (let i = 0; i < values.length; i++) {
    const value = values[i];
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

function lcg(seed) {
  let state = BigInt(seed);
  return () => {
    state ^= state << 13n;
    state ^= state >> 7n;
    state ^= state << 17n;
    state &= 0xffff_ffff_ffff_ffffn;
    return Number(state >> 11n) / 2 ** 53;
  };
}

function plot(spec, columns = []) {
  return {
    document: { version: 1, kind: "plot", spec },
    columns,
  };
}

function loss() {
  const steps = Float64Array.from({ length: 120 }, (_, i) => i);
  const train = Float64Array.from(
    steps,
    (s) => 3.8 * Math.exp(-0.035 * s) + 0.32 + 0.05 * Math.sin(s * 0.7),
  );
  const val = Float64Array.from(
    steps,
    (s) => 4.0 * Math.exp(-0.03 * s) + 0.55 + 0.08 * Math.cos(s * 0.35),
  );
  const trainSmooth = ewma(train, 0.9);
  const valSmooth = ewma(val, 0.9);
  return plot(
    {
      layers: [
        {
          Area: {
            x: { col: 0 },
            low: null,
            high: { col: 1 },
            horizontal: false,
            color: VERMILION,
            label: null,
            opacity: 0.13,
          },
        },
        {
          Line: {
            x: { col: 0 },
            y: { col: 1 },
            color: VERMILION,
            label: "train",
            style: "Pixels",
            glow: true,
          },
        },
        {
          Line: {
            x: { col: 0 },
            y: { col: 2 },
            color: SKY,
            label: "val",
            style: "Pixels",
            dash: "Dotted",
          },
        },
        {
          Rule: {
            orientation: { Horizontal: 0.5 },
            color: null,
            label: "target",
            dash: "Dashed",
          },
        },
      ],
      title: "loss (synthetic)",
      x: "Auto",
      y: "Auto",
      x_label: "step",
      y_label: "loss",
      x_domain: null,
      y_domain: null,
      colorbar: false,
    },
    [steps, trainSmooth, valSmooth],
  );
}

function correlation() {
  const features = ["age", "len", "dep", "mass", "veg", "kcal", "spd", "alt"];
  const n = features.length;
  const grid = Float64Array.from({ length: n * n }, (_, i) => {
    const row = Math.floor(i / n);
    const column = i % n;
    if (row === column) {
      return 1;
    }
    return Math.exp(Math.abs(row - column) * -0.35) * Math.cos((row + column) * 0.55);
  });
  const layers = [
    {
      Cells: {
        columns: n,
        values: { col: 0 },
        extents: null,
        colormap: RED_BLUE,
      },
    },
  ];
  for (let index = 0; index < grid.length; index++) {
    const coefficient = grid[index];
    const column = index % n;
    const row = Math.floor(index / n);
    const ink =
      Math.abs(coefficient) > 0.45 ? { Rgb: [235, 235, 230] } : { Rgb: [32, 32, 32] };
    layers.push({
      Text: {
        x: column,
        y: row,
        text: `${coefficient >= 0 ? "+" : ""}${coefficient.toFixed(2)}`,
        color: ink,
        align: "Center",
      },
    });
  }
  return plot(
    {
      layers,
      title: "feature correlation (synthetic)",
      x: { Bands: features },
      y: { Bands: features },
      x_label: null,
      y_label: null,
      x_domain: null,
      y_domain: null,
      colorbar: false,
    },
    [grid],
  );
}

function violins(wasm) {
  const { adelie, chinstrap, gentoo } = penguins();
  return titled(
    wasm.expand_preset(
      "violin",
      JSON.stringify({ categories: ["Adelie", "Chinstrap", "Gentoo"] }),
      [adelie, chinstrap, gentoo],
    ),
    "flipper length by species (synthetic)",
  );
}

function landscape(wasm) {
  const size = 24;
  const field = Float64Array.from({ length: size * size }, (_, i) => {
    const x = (i % size) / 4;
    const y = Math.floor(i / size) / 4;
    return (x - 3) ** 2 * 0.4 + (y - 2.6) ** 2 * 0.7 + Math.sin(x * 1.7) * Math.cos(y * 1.3) * 0.8;
  });
  const json = wasm.expand_preset("heatmap", JSON.stringify({ columns: size }), [field]);
  const document = JSON.parse(json);
  document.spec.title = "a loss landscape (synthetic)";
  document.spec.colorbar = true;
  return { document, columns: [] };
}

function titled(json, title) {
  const document = JSON.parse(json);
  document.spec.title = title;
  return { document, columns: [] };
}

function penguins() {
  const unit = lcg(0x9e3779b97f4a7c15n);
  return {
    adelie: Float64Array.from(
      { length: 80 },
      (_, i) => 170 + ((i * 17) % 23) * 0.4 + unit(),
    ),
    chinstrap: Float64Array.from(
      { length: 60 },
      (_, i) => 185 + ((i * 13) % 19) * 0.5 + unit(),
    ),
    gentoo: Float64Array.from(
      { length: 90 },
      (_, i) => 205 + ((i * 11) % 29) * 0.45 + unit(),
    ),
  };
}

function languages() {
  return plot(
    {
      layers: [
        {
          Bars: {
            placement: { Bands: ["rust", "go", "python", "typescript", "zig"] },
            values: [68, 41, 55, 62, 12],
            color: null,
            label: null,
          },
        },
      ],
      title: "admired languages, % (synthetic)",
      x: "Auto",
      y: "Auto",
      x_label: null,
      y_label: null,
      x_domain: null,
      y_domain: null,
      colorbar: false,
    },
    [],
  );
}

function boxPlots(wasm) {
  const { adelie, chinstrap, gentoo } = penguins();
  return titled(
    wasm.expand_preset(
      "box_plot",
      JSON.stringify({ categories: ["Adelie", "Chinstrap", "Gentoo"] }),
      [adelie, chinstrap, gentoo],
    ),
    "flipper length, boxes (synthetic)",
  );
}

function scatter() {
  const unit = lcg(0xcafef00ddeadbeefn);
  const species = ["Adelie", "Chinstrap", "Gentoo"];
  const n = 180;
  const x = new Float64Array(n);
  const y = new Float64Array(n);
  const group = [];
  for (let i = 0; i < n; i++) {
    const kind = species[i % 3];
    group.push(kind);
    x[i] = 32 + (i % 3) * 4 + unit() * 3;
    y[i] = 170 + (i % 3) * 18 + unit() * 8;
  }
  return plot(
    {
      layers: [
        {
          Points: {
            x: { col: 0 },
            y: { col: 1 },
            color: null,
            label: null,
            style: "Dot",
            color_by: group,
          },
        },
      ],
      title: "bill depth vs flipper (synthetic)",
      x: "Auto",
      y: "Auto",
      x_label: "bill depth",
      y_label: "flipper",
      x_domain: null,
      y_domain: null,
      colorbar: false,
      palette: {
        colors: [
          { Rgb: [230, 159, 0] },
          { Rgb: [86, 180, 233] },
          { Rgb: [0, 158, 115] },
        ],
      },
    },
    [x, y],
  );
}

function density2d(wasm) {
  const unit = lcg(0x123456789abcdef0n);
  const x = Float64Array.from({ length: 2000 }, (_, i) => unit() * 4 + (i % 2) * 1.2);
  const y = Float64Array.from({ length: 2000 }, (_, i) => unit() * 3 + (i % 3) * 0.4);
  return titled(
    wasm.expand_preset("hist2d", "{}", [x, y]),
    "2D histogram (synthetic)",
  );
}

export function waveDocument(n, domain) {
  return {
    version: 1,
    kind: "plot",
    spec: {
      layers: [
        {
          Line: {
            x: null,
            y: { col: 0 },
            color: { Rgb: [227, 66, 52] },
            label: null,
            style: "Pixels",
            glow: true,
          },
        },
      ],
      title: `${n.toLocaleString("en-US")} points through M4`,
      x: "Auto",
      y: "Auto",
      x_label: "index",
      y_label: null,
      x_domain: domain,
      y_domain: null,
      colorbar: false,
    },
  };
}

export const CHARTS = [
  {
    kicker: "Fig. 1",
    caption:
      "Fig. 1. Training loss as two series, a wash, and a dashed target.",
    width: 72,
    height: 16,
    rust: `let plot = Plot::new()
    .layer(Area::xy(&steps, &train).color(vermilion).opacity(0.13))
    .layer(Line::xy(&steps, &train).label("train").color(vermilion).glow())
    .layer(Line::xy(&steps, &val).label("val").color(sky).dash(Dash::Dotted))
    .layer(Rule::h(0.5).label("target").dash(Dash::Dashed))
    .title("loss (synthetic)")
    .x_label("step")
    .y_label("loss");`,
    typescript: `new Plot()
  .layer(Area.xy(steps, train).color(vermilion).opacity(0.13))
  .layer(Line.xy(steps, train).label("train").color(vermilion).glow())
  .layer(Line.xy(steps, val).label("val").color(sky).dash("Dotted"))
  .layer(Rule.h(0.5).label("target").dash("Dashed"))
  .title("loss (synthetic)")
  .xLabel("step")
  .yLabel("loss")`,
    build: () => loss(),
  },
  {
    kicker: "Fig. 2",
    caption:
      "Fig. 2. Feature correlation: band axes, a diverging map centered at zero, coefficients in the cells.",
    width: 72,
    height: 12,
    rust: `let colormap = Colormap::RED_BLUE.centered_at(0.0);
let mut plot = Plot::new()
    .layer(Cells::matrix(n, &grid).colormap(colormap.clone()))
    .x_scale(Scale::bands(features))
    .y_scale(Scale::bands(features))
    .title("feature correlation (synthetic)");
for (index, &c) in grid.iter().enumerate() {
    plot = plot.layer(Text::at(col, row, format!("{c:+.2}")).align(Align::Center));
}`,
    typescript: `const colormap = Colormap.centeredAt(Colormap.RED_BLUE, 0);
let plot = new Plot()
  .layer(Cells.matrix(n, grid).colormap(colormap))
  .xScale({ bands: features })
  .yScale({ bands: features })
  .title("feature correlation (synthetic)");
for (const [i, c] of grid.entries()) {
  plot = plot.layer(Text.at(i % n, Math.floor(i / n), format(c)).align("Center"));
}`,
    build: () => correlation(),
  },
  {
    kicker: "Fig. 3",
    caption:
      "Fig. 3. Flipper length as violins — a KDE per species. Pixels keep the shoulder; ascii keeps the shape.",
    width: 64,
    height: 16,
    rust: `violin(
    ["Adelie", "Chinstrap", "Gentoo"],
    [adelie, chinstrap, gentoo],
)
.title("flipper length by species (synthetic)")`,
    typescript: `violin(
  ["Adelie", "Chinstrap", "Gentoo"],
  [adelie, chinstrap, gentoo],
).title("flipper length by species (synthetic)")`,
    build: (wasm) => violins(wasm),
  },
  {
    kicker: "Fig. 4",
    caption:
      "Fig. 4. A loss landscape as a heatmap. Ascii is a shade grid; the pixel panel interpolates the field.",
    width: 56,
    height: 18,
    rust: `heatmap(24, &field)
    .colorbar()
    .title("a loss landscape (synthetic)")`,
    typescript: `heatmap(24, field)
  .colorbar()
  .title("a loss landscape (synthetic)")`,
    build: (wasm) => landscape(wasm),
  },
  {
    kicker: "Fig. 5",
    caption: "Fig. 5. Categorical bars from a zero baseline.",
    width: 64,
    height: 14,
    rust: `bar(
    ["rust", "go", "python", "typescript", "zig"],
    &[68.0, 41.0, 55.0, 62.0, 12.0][..],
)
.title("admired languages, % (synthetic)")`,
    typescript: `bar(
  ["rust", "go", "python", "typescript", "zig"],
  [68, 41, 55, 62, 12],
).title("admired languages, % (synthetic)")`,
    build: () => languages(),
  },
  {
    kicker: "Fig. 6",
    caption: "Fig. 6. The same three groups as box plots: type-7 quartiles, Tukey whiskers.",
    width: 64,
    height: 16,
    rust: `box_plot(
    ["Adelie", "Chinstrap", "Gentoo"],
    [adelie, chinstrap, gentoo],
)
.title("flipper length, boxes (synthetic)")`,
    typescript: `boxPlot(
  ["Adelie", "Chinstrap", "Gentoo"],
  [adelie, chinstrap, gentoo],
).title("flipper length, boxes (synthetic)")`,
    build: (wasm) => boxPlots(wasm),
  },
  {
    kicker: "Fig. 7",
    caption: "Fig. 7. A scatter with a color_by channel. Okabe–Ito colors; the groups stay separable.",
    width: 64,
    height: 16,
    rust: `Plot::new()
    .layer(Points::xy(&x, &y).color_by(&group))
    .title("bill depth vs flipper (synthetic)")
    .x_label("bill depth")
    .y_label("flipper")`,
    typescript: `new Plot()
  .layer(Points.xy(x, y).colorBy(group))
  .title("bill depth vs flipper (synthetic)")
  .xLabel("bill depth")
  .yLabel("flipper")`,
    build: () => scatter(),
  },
  {
    kicker: "Fig. 8",
    caption: "Fig. 8. A 2D histogram: two thousand points reduced onto the raster.",
    width: 56,
    height: 16,
    rust: `hist2d(&x, &y).title("2D histogram (synthetic)")`,
    typescript: `hist2d(x, y).title("2D histogram (synthetic)")`,
    build: (wasm) => density2d(wasm),
  },
  {
    kicker: "Fig. 9",
    interactive: true,
    caption:
      "Fig. 9. A long line through M4. Wheel zooms at the cursor; drag pans. The series does not change — only the window does.",
    width: 72,
    height: 16,
    rust: `line(&wave[..]).title("1,000,000 points through M4")
// a zoom is a domain window; M4 re-aggregates to the columns.`,
    typescript: `line(wave).title("1,000,000 points through M4")
// Plot.viewport({ x: [lo, hi] }) — M4 walks the visible window.`,
    build: () => ({ document: waveDocument(1_000_000, null), columns: [] }),
  },
];
