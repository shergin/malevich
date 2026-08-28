# Degradation is the contract

Every terminal gets the best chart it can carry, and no terminal gets a
failure. The ladders are declared, the descent is honest, and the bottom rung
always works.

## Why

A terminal is a hostile rendering target that lies about itself. `$TERM`
names a protocol, not the installed font; a truecolor terminal may be piped
into a file; tmux sits between the process and the screen and eats escapes;
`TERM=dumb` is still someone's workflow. A library that assumes the best
case corrupts output exactly where the user cannot see why — mojibake in a
CI log, escape bytes in a saved file, a probe sequence echoed into someone's
shell.

The opposite instinct — require capabilities, error out below them — is
worse. The first look at data happens in SSH sessions, pipelines, and CI
logs precisely because that is where the numbers are born. A plotting
library that fails on modest terminals fails its actual audience.

## The idea

Every capability is a declared ladder, and rendering walks each ladder down
to a rung that cannot fail. Charsets: octants, sextants, braille, quadrants,
half blocks, ASCII. Color: truecolor, 256, 16, plain — quantized honestly
downhill, with marker-shape cycling carrying category identity where color
cannot. Output: real pixels where the terminal speaks a graphics protocol,
cells everywhere else. Piped output is clean plain text.

Detection is two tiers with different licenses. Sniffing reads the
environment — free, instant, wrong only by omission, so it may run anywhere.
Probing writes escape bytes and reads the reply — ground truth, but licensed
only where escapes are safe: the destination is a tty, nothing sits between,
the terminal is not declared dumb. An unanswered probe is not evidence; it
degrades to the sniff answer. And detection can only widen the safe choice,
never gate correctness: the conservative default must already be right.

Space degrades the same way. When the frame shrinks, furniture sheds before
data — legend, then titles, then tick density — because a small chart of the
real numbers beats a complete frame around nothing. The data region is the
last thing standing.

The ladder's bottom is a guarantee, not an apology: plain ASCII, no color,
any width, `TERM=dumb` — still a correct chart.

## Consequences

- No render path can fail on terminal grounds; there is no "unsupported
  terminal" error anywhere in the library.
- Dense charsets are explicit opt-ins, because no environment variable can
  prove font coverage; the automatic choice is the conservative rung.
- Every downgrade preserves meaning: categories stay separable without
  color, extremes stay visible without subpixels, gaps stay gaps in ASCII.
- A probe that would be unsafe is not attempted, so redirected output can
  never contain interrogation escapes.
- The same plot value renders at every rung; testing the ladder is
  rendering one spec across frames.

## Not this

- Failing, warning, or rendering nothing on an old terminal.
- Probing through tmux, into a pipe, or on `TERM=dumb`.
- A feature that exists only at the top rung with no cell fallback.
- Shedding data to preserve furniture.
- Guessing font coverage from the terminal's name.

See [The frame is run state](frame-is-run-state.md) for why detection
constructs values instead of living in render, and [Vision](../vision.md)
rule 4.

## Witness

One curve at every rung of the charset ladder, spliced from the gallery's
`charsets` example — the same plot value, degrading from octants to ASCII:

<!-- generated:charsets -->
```text
Octants — 2x4 solid blocks (Unicode 16, densest ink)
 1 ┤     𜺠▂▂▂▂𜺣                  ▂▂▂▂▂
   │  𜺠𜴐𜴆𜺨    𜺫𜴆𜴧▂           ▂𜴧𜴁🮂     𜴄𜴜𜶀
   │▗𜴁𜺨           𜴄𜴆𜴧▂▂▂▂▂𜴧𜴐𜴁            𜴄▖
 0 ┤𜺨                                     𜺫𜴄𜶀            𜵑𜴧𜴀
   │                                         🮂𜴜▂𜺣   𜺠▂𜴧𜴁🮂
-1 ┤                                            𜺫🮂🮂🮂𜺨
   └┬─────┬─────┬─────┬─────┬──────┬─────┬─────┬─────┬─────┬
    0     1     2     3     4      5     6     7     8     9

Sextants — 2x3 solid blocks (Unicode 13)
 1 ┤      🬭🬭🬭🬭                   🬞🬭🬭🬭
   │  🬞🬖🬂🬂    🬂🬈🬋🬭           🬭🬖🬅🬂🬀   🬂🬂🬢🬭
   │🬞🬅🬀           🬂🬈🬢🬭🬭🬭🬭🬭🬖🬋🬂            🬈🬏
 0 ┤🬀                                     🬁🬂🬢            🬖🬋🬃
   │                                         🬂🬋🬭🬏   🬞🬭🬖🬅🬂
-1 ┤                                            🬁🬂🬂🬂🬀
   └┬─────┬─────┬─────┬─────┬──────┬─────┬─────┬─────┬─────┬
    0     1     2     3     4      5     6     7     8     9

Quadrants — 2x2 solid blocks (the conservative UTF-8 default)
 1 ┤      ▄▄▄▄                   ▗▄▄▄
   │  ▗▄▀▀    ▀▀▄▄           ▄▄▀▀▘   ▀▀▄▄
   │▗▀▘           ▀▀▄▄▄▄▄▄▄▞▀            ▚▖
 0 ┤▘                                     ▝▀▄           ▗▄▀▘
   │                                         ▀▚▄▄   ▄▄▞▀▘
-1 ┤                                             ▀▀▀
   └┬─────┬─────┬─────┬─────┬──────┬─────┬─────┬─────┬─────┬
    0     1     2     3     4      5     6     7     8     9

Half blocks — 1x2
 1 ┤      ▄▄▄▄                   ▄▄▄▄
   │  ▄▄█▀▀   ▀▀█▄           ▄▄▀▀▀   ▀▀▄▄
   │ █▀           ▀▀▄▄▄▄▄▄▄█▀            █▄
 0 ┤▀                                      ▀▄           ▄▄▀
   │                                         ▀█▄▄   ▄▄▀▀
-1 ┤                                             ▀▀▀
   └┬─────┬─────┬─────┬─────┬──────┬─────┬─────┬─────┬─────┬
    0     1     2     3     4      5     6     7     8     9

Braille — 2x4 dots (dense opt-in)
 1 ┤     ⢀⣀⣀⣀⣀⡀                  ⣀⣀⣀⣀⣀
   │  ⢀⠔⠒⠁    ⠈⠒⠤⣀           ⣀⠤⠊⠉     ⠑⠢⢄
   │⢠⠊⠁           ⠑⠒⠤⣀⣀⣀⣀⣀⠤⠔⠊            ⠑⡄
 0 ┤⠁                                     ⠈⠑⢄            ⡠⠤⠂
   │                                         ⠉⠢⣀⡀   ⢀⣀⠤⠊⠉
-1 ┤                                            ⠈⠉⠉⠉⠁
   └┬─────┬─────┬─────┬─────┬──────┬─────┬─────┬─────┬─────┬
    0     1     2     3     4      5     6     7     8     9

ASCII — 1x1, the guaranteed fallback
 1 +
   |   ***********            **********
   | **          *************          ***
 0 +*                                      ***          ***
   |                                          **********
-1 +
   ++-----+-----+-----+-----+------+-----+-----+-----+-----+
    0     1     2     3     4      5     6     7     8     9
```
<!-- /generated -->

## Spelled today

`render::Charset` and `plot::ColorMode` are the ladders; `Frame::detect`
sniffs (UTF-8 → quadrants, `TERM=dumb` or non-UTF-8 → ASCII) and
`MALEVICH_CHARSET` overrides. `pixel::Capabilities` holds the two-tier
answer with its `Source`; the probe preconditions live in
`Capabilities::detect_for`. Furniture shedding is collision-aware layout in
resolve. This section may rot; the rest must not.
