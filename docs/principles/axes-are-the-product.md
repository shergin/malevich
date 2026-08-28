# The axes are the product

The differentiators live exactly where everyone else got bored. Ticks are
computed, labels are exact, and if the axes are wrong nothing else matters.

## Why

Terminal plotting libraries compete on marks and lose on axes. The typical
axis is an afterthought: ticks at arithmetic intervals that land on ugly
values, labels printed through a float formatter that emits
`0.30000000000000004`, a y-axis whose labels shift width per row, no answer
for log scales or calendar time. The chart body can be beautiful and the
chart is still unreadable, because the axes are how a reader attaches
numbers to ink.

Label lies are the quiet version of the same failure. A label that rounds
differently than its neighbor, or shows a value the tick does not sit on,
transfers wrong numbers into someone's head with full confidence.

## The idea

Treat the boring parts as the product. Tick placement is the extended
Wilkinson algorithm (Talbot, Lin, Hanrahan 2010) — candidate steps scored
for simplicity, coverage, density, and legibility — the placement the
visualization literature settled on, not a heuristic that happens to work at
one size.

Labels are exact decimals: an integer mantissa times a power of ten,
formatted so every label parses back to exactly its value. Float artifacts
are structurally impossible, not filtered out. One fraction width and one SI
prefix per axis (`2.5M`, `100µ`), so columns of labels align and the reader
carries one unit, not one per row. Log axes get superscript decades;
calendar axes get multi-scale labels that say `14:05` or `Aug 2` or `2027`
as the span demands; band axes fit category labels to their bands.

The same care extends to layout: labels measured in display cells (CJK
included), gutters computed, collisions resolved by shedding furniture
rather than overlapping ink.

## Consequences

- Ticks are never supplied as strings; there is no API for hand-placed
  ticks, because computed placement is a feature, not a limitation.
- Every label round-trips: parse it and you have the tick's exact value.
- Axis quality is testable — placement and formatting are pure functions
  with snapshot coverage, not visual judgment calls.
- Log, time, and band scales are first-class axis types, not label
  formatters bolted onto a linear scale.
- The formatter is shared: colorbars, legends, and disclosed quantization
  notes speak the same exact decimals as the axes.

## Not this

- `format!("{}", 0.1 + 0.2)` anywhere near a label.
- Tick counts that ignore available space, or labels that overlap.
- Accepting caller-supplied tick strings as the escape hatch for bad
  placement.
- A second, cheaper formatter for "small" charts.

See [The full draw is the oracle](full-draw-oracle.md) for the ink half of
honesty, and [Vision](../vision.md) rule 3.

## Witness

Ticks stepping by 0.2 — a value with no exact binary form, the classic
float-artifact trap — spliced here by the doc generator. Every label is an
exact decimal, sharing one fraction width per axis:

<!-- generated:witness_axes -->
```text
             every label an exact decimal
0.6 ┤     ⢀⠔⠉⠉⠉⠑⠤⡀                           ⡠⠒⠉⠉⠒⠒⡄
    │    ⡔⠁      ⠈⢆                        ⡠⠊      ⠈⠑⢄
    │  ⡠⠊          ⠑⢄                     ⡜          ⠈
0.4 ┤ ⡜             ⠈⢆                  ⢀⠎
    │⠜                ⠣⡀               ⡰⠁
0.2 ┤                  ⠱⡀            ⢀⠔⠁
    │                   ⠈⢆          ⢀⠎
    │                    ⠈⠢⣀      ⢀⠔⠁
0.0 ┤                       ⠣⢄⣀⣀⡠⠔⠊
    └┬───────┬───────┬────────┬───────┬───────┬───────┬
     0      10      20       30      40      50      60
```
<!-- /generated -->

## Spelled today

`scale::Ticks` is extended-Wilkinson placement, with `Ticks::log10` decades
and `Ticks::time` calendar ticks over unix seconds; `scale::Scale` is the
axis specification (`Linear | Log | Time | Bands`). Exact-decimal formatting
and per-axis SI prefixes live in the tick formatter; label measurement is
display-cell aware via `unicode-width`. This section may rot; the rest must
not.
