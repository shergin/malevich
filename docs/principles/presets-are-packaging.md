# Presets are packaging

A preset is a name for a grammar expansion, proven byte-identical to it. The
front door and the grammar are the same library.

## Why

A chart library that ships `hist()` as its own code path forks the grammar.
The preset accumulates private options, the expansion drifts from the name,
and soon there are two ways to draw a histogram that disagree in the corners —
one for beginners, one for people who read the source. The catalog then grows
by accretion: every requested chart becomes a new function with its own
defaults, and the library's real vocabulary stops being learnable, because
knowing the grammar no longer predicts what the presets do.

The other failure is the opposite: no front door at all. A grammar-only
library makes the first plot a lesson, and the first look at data should not
require one.

## The idea

Every preset is a plain function that composes public grammar — marks, stats,
scales — into a named chart type, and a test asserts its rendered output is
byte-identical to the explicit composition. The preset can therefore never do
anything the grammar cannot; it is packaging, not different math.

Presets are the front door; the grammar is discovered, not required. `line()`
is the first call; `Plot::new().layer(Line::y(...))` is the same call with
the lid off, and graduating from one to the other changes nothing about the
output. Grouped scatters, volcano plots, Manhattan plots, and candlesticks
never become presets — they are a few grammar lines each, and the gallery
shows the lines.

Configuration follows the same discipline. A `_with` variant takes an options
value and returns a typed error for invalid data or options; its default
options must reproduce the plain preset exactly. No option exists that only a
preset can reach.

## Consequences

- Adding a preset costs one function and one equality test, never a render
  path.
- A preset's behavior is documented by its expansion; the test keeps the
  documentation true.
- Users graduate continuously: preset, preset plus builder calls, full
  grammar — with no cliff, because there is nothing behind the preset to
  learn.
- A requested chart type that the grammar cannot spell is a grammar
  question, not a preset request. See
  [What earns a concept](what-earns-a-concept.md).
- The gallery can honestly label charts "from the grammar, no preset" — the
  strongest evidence the vocabulary suffices.

## Not this

- A preset with a private mark, a private stat, or a hidden default the
  grammar cannot express.
- Options objects that grow per chart type into a config kitchen sink.
- A chart-type zoo: one exported function per paper figure.
- "Close enough" equality. The test is byte equality of rendered output, not
  visual similarity.

See [What earns a concept](what-earns-a-concept.md) for what may grow the
grammar instead, and [Vision](../vision.md) rule 2.

## Witness

The `hist` preset and its expansion, rendered by the same program that
splices this file; the example asserts the two strings are equal before
printing one of them:

<!-- generated:witness_packaging -->
```text
hist(&samples) == Bins::auto + Bars::spans, byte for byte:
90 ┤                    ▅▅▅▅▅▂▂▂▂▂
   │                    ██████████
60 ┤               ▂▂▂▂▂██████████▂▂▂▂▂
   │          ▂▂▂▂▂████████████████████▄▄▄▄▄
   │          ██████████████████████████████
30 ┤          ██████████████████████████████
   │     ▁▁▁▁▁██████████████████████████████▂▂▂▂▂
 0 ┤     ████████████████████████████████████████
   └┬─────────┬─────────┬─────────┬─────────┬─────────┬
    0         2         4         6         8        10
```
<!-- /generated -->

## Spelled today

The presets are re-exported at the crate root: `line`, `scatter`, `bar`,
`hist`, `stairs`, `ecdf`, `heatmap`, `hist2d`, `density`, `box_plot`,
`violin`, `error_bars`, `trend`, `contour`, `quiver`, and their `_with`
twins. The equality tests are the
`the_*_preset_equals_its_grammar_expansion` family in
`src/plot/tests/plot_tests.rs`. `witness_packaging` is the spliced example
above. This section may rot; the rest must not.
