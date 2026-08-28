# The full draw is the oracle

The truth is drawing every point. Anything faster must reproduce its pixels
exactly, and anything the raster cannot say honestly is said another way —
never silently.

## Why

Ten million points do not fit through a naive draw loop in interactive time,
so every plotting library reduces. Most sample: take every nth point, or one
per bucket, and hope. Sampling is a lie with good posture — the rendered
chart is *a* chart of the data, not *the* chart, and the difference is
exactly the points that matter: the dropout spike in a loss curve, the
outage gap in a latency series, the one saturated cell in an attention map.
A reduction chosen for speed silently decides what the analyst gets to see.

The same failure has quieter forms. Interpolating across missing data
invents readings that were never taken. Smearing out-of-range points onto
the border fabricates extremes. Rounding tick labels manufactures values.
Each is a small lie the reader cannot detect, which is what makes it a lie.

## The idea

Name the oracle: the raster produced by drawing every point. Every
optimization is proven against it, pixel for pixel — not benchmarked as
"visually close," but asserted equal.

For lines, M4 makes this achievable: keeping the first, last, minimum, and
maximum point of every raster column reproduces that column's pixels exactly
(Jugel et al., PVLDB 2014). The pipeline buckets by the column each point
renders into, so the reduction is a theorem about the raster, not an
approximation of it. Gaps are path topology: finite runs summarize
independently, so a break inside a column stays a break.

For grids denser than the raster, sampling is replaced by ownership: every
screen bucket owns the cells whose centers fall inside it and shows a
declared reduction over all of them — the mean box filter by default, `Max`
when the sparse spikes are the point. The choice is the caller's and the
default is disclosed; nothing is dropped because a sampler happened to step
over it.

What the raster cannot represent, the chart says out loud: `NaN` renders as
a visible gap, never interpolated; out-of-range data clips rather than
smears; quantization is disclosed rather than implied.

## Consequences

- Performance claims and honesty claims are the same claim: the fast path
  is the exact path, so there is no fidelity knob to trade away.
- The oracle test is a permanent fixture: raw raster versus reduced raster,
  asserted equal at several frame sizes. A reduction change that breaks it
  is a different chart, not an optimization.
- Extremes always survive. A spike one sample wide renders at every zoom
  level, because min and max per column are kept by construction.
- A gap in the data is a gap in the chart, at every reduction level.
- Reductions that are *not* pixel-exact — LTTB, smoothing — exist as
  explicit stats the caller applies, never as silent defaults.

## Not this

- Sampling, striding, or "one point per bucket" as an automatic reduction.
- Interpolating across `NaN`, or dropping it silently.
- A fast path that is "indistinguishable at normal sizes."
- Treating the oracle test as a benchmark. It is an equality, not a budget.

See [The axes are the product](axes-are-the-product.md) for the label half of
honesty, and [Vision](../vision.md) rule 3.

## Witness

One hundred thousand points with three one-sample spikes, reduced through
the auto-inserted M4 and spliced here by the doc generator. The spikes
survive because per-column extremes are kept by construction; the
byte-equality against the raw raster is asserted in the crate's oracle test:

<!-- generated:witness_oracle -->
```text
             100,000 points, three one-sample spikes
 8 ┤           ⡇                   ⡇                   ⡇
   │           ⡇                   ⡇                   ⡇
   │           ⡇                  ⢀⡇                   ⡇
 4 ┤ ⢀⣀        ⡇                  ⢸⡇⢀⡀                 ⣧    ⣀
   │⣰⠋⠈⢧     ⣀⡀⡇            ⡼⠉⢧   ⢸⣷⠋⠙⣆     ⣀          ⣿  ⢀⡞⠉⢧
 0 ┤⠁  ⠘⣆  ⢀⡞⠁⠹⡇    ⣰⠋⢳⡀   ⡼⠁ ⠈⢧  ⣸⠃  ⠸⡄   ⡼⠉⠳⡄    ⡼⠉⠳⡄⣿  ⡞  ⠈⢧  ⣠
   │    ⠸⣄⢀⡞   ⠹⡄  ⣰⠃  ⢳⡀ ⣰⠃   ⠈⠳⠴⠃    ⢹⡀ ⡼⠁  ⢳⡀  ⣸⠁  ⠹⣿ ⡼⠁   ⠈⠳⠴⠃
   │     ⠈⠉     ⢳⡀⣰⠃    ⠳⠴⠃             ⠙⠚⠁    ⢳⡀⣰⠃    ⠙⠚⠁
-4 ┤             ⠉⠁                             ⠉⠁
   └┬───────────┬───────────┬────────────┬───────────┬───────────┬
    0          20k         40k          60k         80k       100k
```
<!-- /generated -->

## Spelled today

`stat::m4` is the public reduction; resolution auto-inserts the
column-mapped form for line layers past four points per column.
`Cells::reduce` chooses the grid reduction through the shared
`stat::Reducer` vocabulary. The oracle test is
`large_lines_downsample_pixel_exactly_against_the_raw_raster` in
`src/plot/tests/plot_tests.rs`, toggling the reduction against the raw
raster at several frame sizes. `stat::lttb` and `stat::ewma` are the
explicit, opt-in inexact transforms. This section may rot; the rest must
not.
