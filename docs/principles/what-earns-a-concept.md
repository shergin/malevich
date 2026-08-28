# What earns a concept

A concept sits in the vocabulary iff real charts demand it and no composition
of the rest can draw it. Everything else is a preset or an example.

## Why

Chart libraries die of vocabulary. Each requested chart lands as a new type,
each type grows its own options, and after a few years the library is a
catalog of a hundred near-duplicates — `barh` beside `bar`, `Heatmap` beside
`Image` beside `Matrix` — none composable, all subtly inconsistent, every
one a permanent maintenance obligation. The user-facing cost is worse than
the maintenance cost: a vocabulary that large cannot be learned, only
searched.

The opposite failure is purism: refusing a concept the grammar genuinely
cannot express, so users hand-roll the hard part — and the hand-rolled
versions get the statistics wrong, which is how a plotting ecosystem ends up
with a hundred incorrect quartile implementations.

## The idea

One membership test, two clauses, both required:

- Real charts demand it — charts people actually draw, not charts a
  completeness argument names.
- No composition of the existing marks, stats, and scales reproduces its
  rendered output.

The eight marks pass: a violin is composable from `Area` once the KDE
exists, but the KDE itself is not composable from marks — so the stat earns
a seat and the violin stays a preset. `Cells` earns one mark, not three:
value grids, rgb images, and categorical regions are one geometry with three
color readings. A heatmap is not a mark at all; it is `Cells` under a
colormap, and the preset says so.

The same test shapes the statistics layer. One `Reducer` vocabulary serves
bins, groups, and windows, so "rolling p95" needs zero new concepts — but
the layer refuses a fake uniform algebra: online accumulators, reducers, and
batch transforms keep distinct execution contracts, because pretending every
statistic merges is how libraries ship wrong parallel medians.

A new concept must also pay for itself *across* features: the band scale
earned its seat by serving bar charts, confusion matrices, and attention
maps with one mechanism. A concept that serves one chart is that chart's
implementation detail, not vocabulary.

## Consequences

- The mark family is closed and declared complete; adding a mark is a
  design event judged by the test, never a convenience.
- Feature requests get answered with compositions first. "From the grammar,
  no preset" in the gallery is the test passing in public.
- Options do not multiply: an option must be a mark channel, a stat
  parameter, a scale option, or a theme entry, or it does not ship.
- Statistical correctness centralizes: one type-7 quantile implementation
  serves the box plot, the reducers, and the Q–Q plot.
- Removing is a contribution. A concept whose charts the grammar learns to
  compose is retired at the next major version.

## Not this

- A mark per chart type, or a `Heatmap` mark beside `Cells`.
- "We might need it" as clause one, or "it would be elegant" as clause two.
- A uniform `Stat` trait that pretends every statistic streams and merges.
- Opening the mark enum so dependents can register marks. New geometry
  lands in the crate, under this test.

See [Presets are packaging](presets-are-packaging.md) for where refused
concepts go instead, and [Vision](../vision.md) rule 2.

## Spelled today

`mark::Mark` is the closed enum over `Line`, `Points`, `Bars`, `Area`,
`Cells`, `Range`, `Rule`, `Text`; its docs declare the family complete.
`stat::Reducer` is the shared aggregation vocabulary; the execution
contracts are documented per type in `stat` (accumulators with merge laws,
reducers, batch transforms). `scale::Scale` is the closed axis
specification. The refusals are recorded in the README's "What it will not
be." This section may rot; the rest must not.
