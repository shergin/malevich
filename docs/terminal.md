# Terminals

How a chart meets a terminal: the charset and color ladders, what detection
reads, and the overrides. The design argument is
[Degradation is the contract](principles/degradation-is-the-contract.md);
this file is the mechanics.

## The charset ladder

A charset is a glyph tier the subpixel surface encodes through; glyph tables
are data, not code.

| charset | subpixels per cell | standing |
|---|---|---|
| `Octants` | 2×4 solid blocks | Unicode 16; densest ink, explicit opt-in |
| `Sextants` | 2×3 solid blocks | Unicode 13; explicit opt-in |
| `Braille` | 2×4 dots | dense opt-in; dots, not blocks |
| `Quadrants` | 2×2 solid blocks | the conservative UTF-8 default |
| `HalfBlocks` | 1×2 | the lowest Unicode rung |
| `Ascii` | 1×1 | the guaranteed fallback |

The dense tiers are explicit because no environment variable can prove the
configured font covers them — a terminal name is not a font. `Frame::detect`
therefore picks quadrants in any UTF-8 environment and ASCII otherwise;
octants, sextants, and braille are choices you make for fonts you know.

The gallery's `charsets` example renders one curve at every rung:
`cargo run --example charsets`.

## The color ladder

Four tiers, quantized honestly downhill: `TrueColor`, `Ansi256`, `Ansi16`,
`Plain`. Heatmap half-blocks carry independent upper and lower colors; plain
output retains an averaged shade. In colorless output, `color_by` categories
cycle portable marker shapes (`•`, `+`, `x`, `*`, `o`) so groups never
vanish in a pipe.

## What detection reads

`Frame::detect` sniffs; it never writes to the terminal. The rules, in
order:

- **Charset.** `MALEVICH_CHARSET` wins if set to a known name. `TERM=dumb`
  means ASCII. Then locale, POSIX precedence — `LC_ALL`, `LC_CTYPE`,
  `LANG`; a locale without `utf` means ASCII. Otherwise quadrants.
- **Color.** `NO_COLOR` (any value) means plain. Output that is not a
  terminal means plain, unless `CLICOLOR_FORCE` is set and not `0`.
  `TERM=dumb` means plain. `COLORTERM=truecolor` or `24bit` means
  truecolor; a `TERM` containing `256color` means 256; otherwise 16.
- **Size.** The terminal's reported cell size, with a fallback when there
  is none.
- **Theme.** `COLORFGBG` distinguishes dark from light backgrounds.

Piped output is therefore clean plain text by default — detection sees a
non-terminal and drops color, and the charset choice never emits anything a
file cannot hold.

## Overrides

- `MALEVICH_CHARSET` — `ascii`, `half`, `quad`, `sextants`, `octants`,
  `braille`, or `auto`.
- `NO_COLOR` — force plain output ([no-color.org](https://no-color.org)).
- `CLICOLOR_FORCE` — keep color when piping.
- An explicit `Frame` — the programmatic override that consults nothing.

## Text discipline

- CJK labels are measured in display cells and stay aligned.
- Combining marks are deliberately dropped at the cell grid.
- Control characters are dropped at the cell grid: a title, label, or
  category carrying escape bytes can never smuggle them into any encoder's
  output. The only escapes in ANSI output are the encoder's own SGR
  sequences; a regression test pins this contract.
- `NaN` is always a visible gap, never interpolated away.

## Small frames

When the frame shrinks, furniture sheds before data: legend, then titles,
then tick density. The data region is the last thing standing, and
`TERM=dumb` at any width still gets a correct chart.
