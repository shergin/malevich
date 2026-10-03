# Degradation is the contract

Every terminal gets the best chart it can carry, and no terminal gets a
failure. The ladders are declared, the descent is honest, and the bottom rung
always works.

## Why

A terminal is a hostile place to draw, and it lies about itself. `$TERM`
names a protocol, not the font that was installed. A truecolor terminal may
be piped into a file. tmux sits between the process and the screen and eats
escapes. `TERM=dumb` is still someone's workflow. Assume the best case and
the output corrupts exactly where the user cannot see why. Mojibake in a CI
log. Escape bytes in a saved file. A probe sequence echoed into a shell.

The opposite instinct is worse. Require the capabilities, and error out below
them. The first look at data happens in SSH sessions, pipelines, and CI logs,
because that is where the numbers are born. A plotting library that fails on
a modest terminal fails its actual audience.

## The idea

Every capability is a declared ladder, and rendering walks each one down to a
rung that cannot fail. Charsets: octants, sextants, braille, quadrants, half
blocks, ASCII. Color: truecolor, 256, 16, plain — quantized honestly
downhill, with marker-shape cycling carrying category identity where color
cannot. Output: real pixels where the terminal speaks a graphics protocol,
cells everywhere else. Glyphs and escapes in a tty. A card of spans where the
host draws with HTML. Rectangles and text runs where it draws with SVG. A
pipe gets clean plain text.

Detection is two tiers, and they are not licensed the same way. Sniffing
reads the environment. It is free, instant, and wrong only by omission, so it
may run anywhere. Probing writes escape bytes and reads the reply. That is
ground truth, but only where escapes are safe: the destination is a tty,
nothing sits between, and the terminal is not declared dumb. An unanswered
probe is not evidence. It degrades to the sniff answer. Detection can only
widen the safe choice. It never gates correctness. The conservative default
must already be right.

Space degrades the same way. When the frame shrinks, furniture sheds before
data: the legend, then the titles, then tick density. A small chart of the
real numbers beats a complete frame around nothing. The data region is the
last thing standing.

The bottom of the ladder is a guarantee, not an apology. Plain ASCII, no
color, any width, `TERM=dumb`. Still a correct chart.

## Consequences

- No render path can fail on terminal grounds. There is no "unsupported
  terminal" error anywhere in the library.
- Dense charsets are explicit opt-ins. No environment variable can prove font
  coverage, so the automatic choice is the conservative rung.
- Every step down keeps the meaning. Categories stay separable without color.
  Extremes stay visible without subpixels. Gaps stay gaps in ASCII.
- A probe that would be unsafe is not attempted, so redirected output can
  never contain interrogation escapes.
- The same plot value renders at every rung. Testing the ladder is rendering
  one spec across frames.

## Not this

- Do not fail, warn, or render nothing on an old terminal.
- Do not probe through tmux, into a pipe, or on `TERM=dumb`.
- No feature that lives only on the top rung, with no cell fallback.
- Do not shed data to keep the furniture.
- Do not guess font coverage from the terminal's name.

See [The frame is run state](frame-is-run-state.md) for why detection
constructs values instead of living in render, and [Vision](../vision.md)
rule 4.

## Witness

One curve at every rung of the charset ladder, spliced from the gallery's
`charsets` example. The same plot value, from octants down to ASCII:

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
`Capabilities::detect_for`. `Raster::to_html` and `Raster::to_svg` are the
cell-card encoders, and with the `pixel` feature `Plot::to_svg_pixels`
encodes the hybrid render for an SVG host, degrading to the cell card.
Furniture shedding is collision-aware layout in resolve. This section may
rot; the rest must not.
