<!-- GENERATED FILE — do not edit. Every byte of this file is produced by
examples/regen_docs.rs from the gallery examples; edit those instead and
run `cargo run --example regen_docs`. -->

# Gallery

The showcase and the system test in one artifact. This whole file is
generated from the examples (unlike README.md, which splices marked
blocks); regenerate with `cargo run --example regen_docs` — CI fails when
it is stale. Every example renders a fixed deterministic frame, so output
is deterministic.

## sine

Function sampling: curves drawn from `f(x)`, one sample per subpixel column.
Source: [examples/sine.rs](examples/sine.rs)

```text
                        sin(x) and 0.6 cos(x/2)
 1.0 ┤     ⢀⠔⠊⠉⠑⢄                           ⢀⠔⠉⠉⠒⢄
     │    ⡠⠃     ⠑⢄                        ⡔⠁     ⠣⡀
     │⠤⠤⢄⣰⡁       ⠈⢆                     ⢀⠜        ⠘⡄              ⢀⣀⡠⠤⠤
 0.5 ┤  ⢰⠁⠈⠉⠒⠢⣄    ⠘⡄                    ⡸          ⢸          ⣠⠔⠒⠉⠁
     │ ⢀⠎      ⠉⠒⢄⡀ ⠸⡀                  ⡰⠁           ⢣     ⢀⠤⠒⠉
     │⢀⠎          ⠈⠱⢄⠱⡀                ⢠⠃             ⢣ ⢀⡠⠊⠁
 0.0 ┤⠎              ⠉⢳⢄              ⢠⠃             ⢀⡨⢖⠁              ⡰
     │                 ⢣⠉⠦⡀          ⢠⠃            ⣀⠔⠁ ⠈⢆             ⡰⠁
     │                  ⢣ ⠈⠑⠢⣀      ⢀⠎         ⢀⡠⠔⠊     ⠈⢆           ⡰⠁
-0.5 ┤                   ⡇    ⠙⠒⠤⣀⡀ ⡎      ⣀⡠⠤⠒⠁         ⠈⡆          ⡇
     │                   ⠘⡄       ⠈⡹⠑⠒⠒⠒⠒⠉⠉               ⠱⡀       ⢀⠎
     │                    ⠈⢆     ⢀⠜                        ⠑⡄     ⡠⠊
-1.0 ┤                      ⠑⠤⣀⡠⠔⠁                          ⠈⠢⢄⣀⡠⠊
     └┬─────────┬──────────┬─────────┬─────────┬──────────┬─────────┬───
      0         2          4         6         8         10        12
```

## loss

A real training log: poorgrad's bigram model on 32k names — per-step loss, rolling mean, and the known bigram limit as a rule.
Source: [examples/loss.rs](examples/loss.rs)

```text
                   poorgrad: bigram training on 32k names
               ── minibatch  ── rolling mean  ── bigram limit
  3.3 ┤⡇
  3.2 ┤⣧
      │⢿
  3.1 ┤⢸⡄
  3.0 ┤⢸⡇
l     │⢸⢸
o 2.9 ┤ ⡏⡆
s 2.8 ┤ ⣧⡇
s     │ ⢹⣼
  2.7 ┤ ⠈⣷⣧
  2.6 ┤   ⢿⣿⣶⡀⢀   ⢀ ⡄ ⡄
      │    ⡟⡟⣿⣿⣦⣷⣶⣾⣾⣇⣦⣧⡀⣤⣠⣄⢰ ⣀⣠⡄⢠⣤⣀⡆⡄⣇⣀⢠⡀⡀⢀⡄⢀⡀  ⣄⣠⡇⣀⢠  ⣀⡀ ⡄ ⡀ ⢀⣠⣀⣰⣀⢸⡀  ⢀ ⡆ ⡄
  2.5 ┤⣀⣀⣀⣀⣀⣀⣋⣁⣛⣻⣹⣻⣏⣟⣿⣿⣿⣿⣿⣿⣟⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣾⣷⣿⣿⣿⣿⣿⣾⣿⣿⣿⣿⣿⣿⣷⣾⣿⣿⣾⣧⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣶⣿⣾⣾⣿⣿⣿
  2.4 ┤          ⠈   ⠈ ⠁⠋⠁⠃⠃⠉ ⠏⠛⠸⠈⠁⠘⠃⠏⠛⠛⠈⠈⠹⠹⠉⢸⠃⠇⠋⠇⠘⠋⡏⠋⠏⠿⠟⠻⠘⡇⠙⠙⠇⠏⠸ ⠏⠋⠻⠻⢻⡿⠻⠸⡏⠃
      └┬──────┬─────┬──────┬──────┬──────┬──────┬──────┬──────┬─────┬──────┬
       0     100   200    300    400    500    600    700    800   900  1000
                                       step
```

## languages

Categorical bars from a zero baseline, with eighth-block precision at the top.
Source: [examples/languages.rs](examples/languages.rs)

```text
            admired languages, % (synthetic)
   │  ████████
60 ┤  ████████                      ███████
   │  ████████            ███████   ███████
   │  ████████            ███████   ███████
40 ┤  ████████  ▆▆▆▆▆▆▆▆  ███████   ███████
   │  ████████  ████████  ███████   ███████
   │  ████████  ████████  ███████   ███████
20 ┤  ████████  ████████  ███████   ███████
   │  ████████  ████████  ███████   ███████   ▁▁▁▁▁▁▁
   │  ████████  ████████  ███████   ███████   ███████
 0 ┤  ████████  ████████  ███████   ███████   ███████
   └────────────────────────────────────────────────────
        rust       go      python   typescri…   zig
```

## clusters

Palmer penguins through one color_by channel: categories take palette colors, name themselves in the legend, and cycle marker shapes in colorless output.
Source: [examples/clusters.rs](examples/clusters.rs)

```text
                        penguin bills by species
                   •• Adelie  ++ Chinstrap  xx Gentoo
  21 ┤          ⡀        ⠄⡀   ⢀  ⠄       ⠁
     │                ⡀   ⠠⠄     ⠠                    +    +
  20 ┤                 ⠄⠄⠄  ⠠ ⠌ ⠂       ⠁       + + ++  ++
     │           ⡀  ⢄⢀⢀⠁  ⠂     ⡤    ⠁  +      + + +  ++      +
  19 ┤       ⠐   ⠠ ⠁⢀ ⠰⡀⠁⢂⡀⡔⠄⡦⠆ ⠐ ⠈⠂   +⠠+      + +++++
d    │         ⠠   ⠂⠠⠃⠐⠍  ⠁⠄⠐⠡⡁⠇⠐⠠⠂⠂     ++ +   ++ ++
e 18 ┤         ⠁⠁⠂⠈⠆⠡⠈⠄⢁⠑⡠⠠⠇⠈⠂⡁⠈  ⡀+ ⠁ +  +       ++  +            +
p 17 ┤        ⢀ ⠄ ⢁⡰⡀ ⢀⣂ ⣌⠈⠄⢀⢄⠁  +   x  ++ +  +  + x  x       x       x
t    │            ⠈⠄⠄⢈⠂⢀  ⠐   +  + +   x  ++++   x  x
h 16 ┤       ⠂   ⢀⠂⠐   ⠄                  x   xxxx xxx       x
     │    ⠠                          x x xxxxx xxxxx   x x x
  15 ┤                         x   x x xxxx xxxxxxxx
     │                            xx x xxxxxxxxx  x x
  14 ┤                        x  x xx xxx  xx
  13 ┤                          x xx  x  xx
     └┬──────────┬──────────┬──────────┬─────────┬──────────┬──────────┬
     30         35         40         45        50         55         60
                                bill length, mm
```

## volcano

A volcano plot from the grammar, no preset: significance classes via color_by, thresholds as Rules, grey pinned to the insignificant mass.
Source: [examples/volcano.rs](examples/volcano.rs)

```text
                  differential expression (synthetic)
                        •• n.s.  ++ down  xx up
  5 ┤                      ⡇                     ⢸               x
    │                      ⡇                     ⢸
    │                      ⡇                     ⢸             x xx
  4 ┤       + + +          ⡇                     ⢸           x xx
-   │          +           ⡇                     ⢸           x
l   │    +      ++ +       ⡇                     ⢸        xx  x x x
o 3 ┤            +   +     ⡇                     ⢸       xx
g   │        ++    +     + ⡇                     ⢸            x
1   │        +     + +     ⡇                     ⢸  x     x x xx x
0   │       +       +    + ⡇                   ⢀ ⢸   xx   x    x
  2 ┤⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠩⠉⠉⠉⡏⠉⡉⠩⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⢉⡍⠉⠉⢹⠉⠭⠉⢉⠉⠋⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉⠉
p   │                ⠂     ⡇⠐ ⢐⣪⠃⡄         ⠠⠦⠚⠆ ⠂⢸  ⡀   ⠠  ⠠
    │               ⠂    ⠠⠃⡇  ⠈⡫⠶⣷⣤⡄     ⢀⣰⢋⣳⢫⣄  ⢸ ⠂   ⠂⢀
  1 ┤                ⠐  ⠂  ⡇  ⠐⡒⢔⣧⢬⡺⢲⡀ ⠠⡮⠍⢶⣎⡨⣕   ⢸ ⠂
    │                   ⠠ ⡀⡇   ⢦⠅⣧⡫⠕⡭⠼⠬⡺⡛⣾⢶⢄⣻⢣⠁  ⢸   ⠠
    │                     ⠂⡇   ⣣⢝⣝⣃⣽⡁⢗⣳⠿⠾⡳⣏⢠⢊⡟⠂  ⢸
  0 ┤                     ⠐⡇ ⢀⢀⣓⣴⣰⣬⣜⣟⣹⣻⣺⣿⣭⣯⣣⣰⣢⡄⡀ ⢸
    └┬──────────┬──────────┬──────────┬──────────┬──────────┬──────────┬
    -3         -2         -1          0          1          2          3
                              log2 fold change
```

## manhattan

A Manhattan plot from the grammar, no preset: chromosomes alternate two shades as unlabeled layers, the genome-wide threshold is a labeled Rule.
Source: [examples/manhattan.rs](examples/manhattan.rs)

```text
                        association scan (synthetic)
                               ── genome-wide
  10 ┤                    ⠈
     │                    ⠈                       ⠘
     │                    ⢐                       ⠒⡀          ⠈
-  8 ┤                    ⢡⡄                      ⠱⡀          ⢸
l    │                    ⡐⣂                      ⠤           ⠠
o    │                    ⠂⡖                      ⠰⡂          ⢀⡀
g  6 ┤                   ⠘⡏⠴⡀                     ⠔           ⡒⠃
1    │⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠴⠥⠦⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⢤⠤⢬⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⣧⣧⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤⠤
0    │                  ⢀⡼ ⠈⠅                    ⢠⠃⢀         ⠠⡄⢅
   4 ┤                  ⢄⠁⡀⠨⠢⡄                   ⡘⠃⡐⡂        ⢨⠃⠨
p    │⢀⡀⢀⢀⡀ ⣀⣀ ⢀ ⡀⣀⡀⡀ ⢀⡀⢡⢎ ⠈⠖⡢⡀ ⢀⢀⡀⣀⣀⣀⢀ ⡀ ⡀⡀⢀⡀⣀⢀⡀⢨ ⠨⠡  ⣀⣀⡀ ⡀⡀⡠ ⢰ ⡀  ⣀⡀⡀
   2 ┤ ⡳⡳⡓⢒⠝⡷⠳⣾⠒⢞⠆⠓⣐⢇⢾⢧⢆⡨⠂  ⢴⣈⣇⢦⣑⠆⡡⡃⢖⣩⢁⡸⢞⢳⢚⠟⡕⣉⠏⢕⠈⡡ ⠠⠋⡭⡰⡬⣸⣲⡹⣜⠮⣋ ⢀⢡⡔⡫⡐⣢⢆⠅
     │⣋⠬⣆⢀⡍⡣⣒⠬⢞⡼⢂⢵⣲⡢⢣⠕⢾⠙⡲   ⠐⠫⢚⢧⢍⠷⡭⡜⡱⠳⢭⡡⣷⢐⡔⢭⣅⡌⢒⠫⡿⠇  ⠵⣽⠘⢡⢢⢍⡆⡕⣈⠡ ⠈⡩⠹⣳⣰⠽⡗⠄
     │⢡⣡⠤⡥⣪⢴⢚⢤⡀⢏⣁⡀⠉⡚⡸⠫⣔⢛⡅    ⢨⠬⠦⡐⠜⠱⢬⢢⠥⡚⠲⠔⡌⢠⠦⢵⠝⠔⠣⢹⡀  ⠌⠁⡖⢷⡂⢓⡕⠜⠘⢃  ⣖⣑⢦⣘⢱⡉
   0 ┤⢮⠙⣰⡹⠬⠭⠉⢦⠌⢄⡫⢩⢫⠳⠫⠆⡢⠤     ⠐⢐⡱⡉⡬⠩⡢⠠⡀⣮⡛⠠⠭⡗⡰⠄⡬⠝⠼⠂   ⠙⠃⠼⡐⠧⠣⢪⠭⢧⠓  ⠪⠮⠌⡧⠔⢸⠄
     └┬─────────┬─────────┬─────────┬────────┬─────────┬─────────┬─────────┬
      0        200       400       600      800      1000      1200     1400
                                 genomic position
```

## candles

Candlesticks from the grammar, no preset: Range whiskers and bodies with up/down days split by color_by.
Source: [examples/candles.rs](examples/candles.rs)

```text
                       daily candles (synthetic)
                             ┃┃ up  ┃┃ down
  107.5 ┤         ⠠⡤
        │     ⠠⢤⣄⡤⡤⡧⢤⠄ ⢀⣀
  105.0 ┤      ⢸⣾⣶⣷⣷⣾⣶⣀⣀⡗⢲⠂
        │    ⠠⣼⣿⣿⠿⡿⣿⣿⣿⣤⣿⣿⣿⣿⠂
  102.5 ┤⠠⢤⠄⠒⣖⣿⣿⣿⠒⠓⡧⠼⢿⠿⣿⣟⢻⣿⣶⡤
p       │⡏⢸⢲⣶⣿⡟⢻  ⠐⠓ ⠉⠓⠓⠈⢹⣿⣿⡇⢀⣀⡀
r 100.0 ┤⣿⣿⣿⣿⣿⡷⠚⠂        ⠈⢹⣿⣿⡧⢸⢹⠁
i       │⣇⣸⣹⠉⡏⠁           ⠉⠉⡯⠯⢹⣿⣿⣀                    ⠐⢲⠂  ⢀⣀⣀⡀
c       │  ⠉⠉⠉             ⠒⠓⠐⠚⢻⣿⣷⡆ ⢹⠓⡖        ⠒⡞⡯⢤⠄ ⣬⣯⣼⣤⡒⡖ ⢸⢸
e  97.5 ┤                      ⠚⣿⣿⡏⢀⣸⣀⣇⣀       ⣤⣧⣧⣼⣤⠒⣿⣿⣿⣿⣿⣿⣷⣾⣾⣶⡖
        │                       ⣉⣿⣿⣿⣿⣿⣿⣧⣤⢶⠂⢀⣀ ⢤⣿⣿⣿⣿⣿⣷⣿⣟⢻⠚⠦⠿⠛⢹⣿⣿⣇⡀
   95.0 ┤                        ⠿⡿⢿⢿⠉⣿⣿⣿⣼⣭⣯⣏⣹⣿⣿⡟⢓⣸⣻⠛⡟⠓⠚⠂  ⠠⠼⢼⣿⣿⡗
        │                        ⠠⠷⠚⠾⠖⠛⣿⣿⣿⣿⣿⣿⣿⡽⠟⠓  ⠉⠓⠓       ⠚⠿⣿⣷⣶⠂
   92.5 ┤                             ⠈⠹⢿⢿⠿⠯⡏⠁                 ⡿⣿⣿⣤
        │                               ⢸⠼⠄⠈⠉                 ⠉⠩⢧⣸⡀
   90.0 ┤                              ⠈⠉⠁
        └┬───────────┬────────────┬───────────┬────────────┬───────────┬
         0          10           20          30           40          50
```

## fit

Least squares as a stat: scatter, trend line, and a 95% confidence band from one mergeable Fit accumulator — slope, intercept, and R² included.
Source: [examples/fit.rs](examples/fit.rs)

```text
                      y = 0.82x + 3.87   R² = 0.95
  30 ┤                                                              ⠐
     │                                                         ⡈⣀⣠ ⣀⠤⠔⠊⠁
  25 ┤                                                    ⠂⢀⣠⣤⣶⣿⡿⠟⡉⠈   ⠄
     │                                                ⢂⣈⣥⣶⣿⠿⠟⠫⠉  ⡀
r    │                                          ⠠ ⣀⣮⣶⣾⠿⠛⠋⠉   ⠄
e 20 ┤                                     ⠁ ⣐⣤⣶⡾⠿⠛⠉⠁
s    │                                ⠂ ⣐⣤⣴⡾⠟⠛⠉⠃ ⠐
p    │                             ⣀⣤⣴⡾⠟⠟⠉⠁
o 15 ┤                       ⢀⣄⣦⣴⡾⠿⠛⠉⠁
n    │                ⠂ ⢀⣀⣤⣷⡾⠟⠛⠉     ⠁
s 10 ┤           ⡀ ⢈⣨⣤⣶⡾⡿⠛⢉⠁⡀⡀
e    │        ⣀⣠⣴⣶⣿⠿⠛⠉⠁  ⠂
     │⠄  ⣀⣤⣴⣾⣿⠿⠛⠋⠉⠈
   5 ┤⣶⣾⣿⠿⠟⠋⠉⠁
     │⠛⠋   ⠁
   0 ┤
     └┬──────────┬──────────┬──────────┬──────────┬─────────┬──────────┬
      0          5         10         15         20        25         30
                                     dose
```

## qq

A Q–Q plot from the grammar, no preset: matched type-7 quantiles of two samples against the identity line — the heavy tail peels off it.
Source: [examples/qq.rs](examples/qq.rs)

```text
                Q–Q: heavy-tailed vs normal-ish
                   ── identity  •• quantiles
   6 ┤                                         ⠠           ⢀⣀⠤⠒⠉
h    │                                       ⢀         ⣀⠤⠔⠊⠁
e  4 ┤                                     ⢀⠂     ⢀⡠⠔⠒⠉
a    │                                   ⢀⡠  ⢀⡠⠤⠒⠉⠁
v    │                                 ⡠⠠⣂⠤⠒⠊⠁
y  2 ┤                             ⢀⣀⡴⠝⠊⠉
-    │                         ⣀⡤⠴⠚⠉
t  0 ┤                    ⢀⣀⠴⠚⠋⠁
a    │                ⣀⡠⠖⠋⠉
i -2 ┤           ⢀⡠⠔⢒⠽⠃
l    │      ⢀⣀⠤⠒⠉⡁⠄⠊⠁
e    │  ⣀⠤⠔⠊⠁  ⠈
d -4 ┤⠒⠉     ⡀⠊
     │
  -6 ┤     ⠈
     └┬──────────┬───────────┬──────────┬───────────┬──────────┬
     -4         -2           0          2           4          6
                         normal-ish quantiles
```

## waveform

Ten million points through the auto-inserted M4 aggregation — pixel-identical to drawing every point, in tens of milliseconds.
Source: [examples/waveform.rs](examples/waveform.rs)

```text
                           10,000,000 points
 7.5 ┤⣇⢸⡄⣧⢸⡆⣾ ⡇⢸ ⣧⢸⡆⣷⢰⡇⣼ ⡇⢸ ⣷⢰⡇⣾⢠⡇⣸ ⡇⢸⡆⣾⢠⡇⣸ ⡇⢸ ⣇⢸⡇⣸ ⡇⢸ ⣇⢸⡀⣧⢰⡇⢸ ⡇⢸⡀⣧⢸⡆⣷ ⡇
     │⣿⢸⡇⣿⢸⡇⣿⢰⡇⣸ ⣿⢸⡇⣿⢸⡇⣿ ⡇⢸⡆⣿⢸⡇⣿⢸⡇⣿⡄⣷⢸⡇⣿⢸⡇⣿⣸⣷⣾⡇⣿⢸⡇⣿⢸⣧⣾⣦⣿⢸⡇⣿⢸⡇⣾⢠⣿⣸⡇⣿⢸⡇⣿⢸⡇
 5.0 ┤⣿⣿⣷⣿⢸⡇⣿⢸⣧⣿⣶⣿⣼⡇⣿⢸⣇⣿⣾⣿⣼⡇⣿⢸⡇⣿⣾⣿⣿⣇⣿⢸⡇⣿⢸⣧⣿⣿⣿⣿⡇⣿⢸⡇⣿⢸⣿⣿⣿⣿⣾⡇⣿⢸⡇⣿⢸⣿⣿⣿⣿⢸⡇⣿⢸⣇
     │⣿⣿⣿⣿⣼⣧⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣾⣧⣿⣿⣿⣿⣿⣿⣾⣇⣿⢸⣿⣿⣿⣿⣿⣧⣿⢸⡇⣿⣸⣿⣿⣿⣿⣿⡇⣿⢸⣧⣿⣿⣿⣿⣿⣿⣾⣧⣿⣾⣿
 2.5 ┤⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣾⣿⣿⣿⣿⣿⣿⣿⣿⣷⣿⣿⣿⣿⣿⣿⣿⣧⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
     │⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
 0.0 ┤⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
     │⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
-2.5 ┤⣿⣿⣿⣿⣿⡿⣿⣿⣿⣿⣿⣿⣿⡿⣿⢿⣿⣿⣿⣿⣿⣿⣿⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
     │⣿⣿⡿⣿⢹⡇⣿⣿⣿⣿⣿⣿⢹⡇⣿⢸⡏⣿⣿⣿⣿⣿⣿⢸⡏⣿⢻⣿⣿⣿⣿⣿⡟⣿⢻⣿⣿⣿⣿⣿⣿⣿⣿⡿⣿⣿⣿⣿⣿⣿⣿⡿⣿⢹⣿⣿⣿⣿⣿⡿⣿⢻⡇⣿⣿⣿
-5.0 ┤⣿⢸⡇⣿⢸⡇⣿⢿⣿⣿⣿⣿⢸⡇⣿⢸⡇⣿⣿⣿⣿⡿⣿⢸⡇⣿⢸⡇⣿⣿⣿⢿⡇⣿⢸⡇⣿⢹⡿⣿⡟⣿⢸⡇⣿⢹⡿⣿⡟⣿⢹⡇⣿⢸⡟⣿⣿⣿⢹⡇⣿⢸⡇⣿⢻⣿
     │⡟⢸⠇⣿⢸⡇⣿⢸⡟⣿⠿⣿⢸⡇⣿⢸⡇⣿⠻⣿⢻⡇⣿⢸⡇⣿⠸⡇⢻⡏⣿⢸⡇⣿⢸⡇⢿⠈⡇⢸⡇⣿⢸⡇⣿⢸⡇⢸ ⡿⢸⡇⣿⢸⡇⣿⢸⡏⢸⠇⣿⢸⡇⣿⢸⡏
-7.5 ┤⠇⢸ ⡿⠸⡇⢻⠈⡇⢸ ⡇⢸⠁⣿⠈⡇⢸ ⡇⢸⠁⡟⢸⠃⢿ ⡇⢸⠁⡏⢸⠃⡿⢸⠇⢸ ⡇⢸⠃⡿⢸⠇⢿⠸⡇⢸ ⡇⢸⠇⢿⠸⡇⢻⠈⡇⢸ ⡟⠸⡇⢻⠈⡇
     └┬────────────┬────────────┬────────────┬────────────┬────────────┬
      0          2.0M         4.0M         6.0M         8.0M       10.0M
```

## distribution

Penguin body mass through automatic binning: a real, lumpy distribution.
Source: [examples/distribution.rs](examples/distribution.rs)

```text
                       penguin body mass
100 ┤               ▃▃▃▃▃▃▃
    │               ███████
 75 ┤               ███████
    │               ███████
    │       ▇▇▇▇▇▇▇▇███████▅▅▅▅▅▅▅
 50 ┤       ██████████████████████▆▆▆▆▆▆▆▆
    │       ██████████████████████████████
    │       ██████████████████████████████▇▇▇▇▇▇▇▃▃▃▃▃▃▃
 25 ┤       ████████████████████████████████████████████
    │▂▂▂▂▂▂▂████████████████████████████████████████████
  0 ┤███████████████████████████████████████████████████▅▅▅▅▅▅▅▅
    └┬──────┬──────┬───────┬──────┬──────┬───────┬──────┬──────┬
   2500   3000   3500    4000   4500   5000    5500   6000  6500
                                grams
```

## powerlaw

Log-log axes: power laws render straight, with decade ticks on both axes.
Source: [examples/powerlaw.rs](examples/powerlaw.rs)

```text
                   power laws on log-log axes
                   ── 0.5 x^1.5  ── 20 sqrt x
    │                                                      ⣀⣠⠴⠒⠋
    │                                                 ⢀⡠⠤⠒⠋⠁
10⁶ ┤                                            ⢀⡠⠤⠒⠉⠁
    │                                       ⣀⡠⠔⠊⠉⠁
    │                                   ⣀⠤⠒⠉
10⁴ ┤                              ⣀⠤⠔⠊⠉               ⢀⣀⣀⣀⡤⠤⠤⠤⠒
    │                         ⣀⠤⠔⠊⠉      ⣀⣀⡠⠤⠤⠤⠔⠒⠒⠒⠒⠉⠉⠉⠁
    │                    ⣀⣤⣔⣊⣉⠤⠤⠤⠔⠒⠒⠒⠊⠉⠉⠉
10² ┤      ⣀⣀⣀⡠⠤⠤⠤⠔⢒⣒⠶⠛⠋⠉⠉
    │⠒⠒⠉⠉⠉⠉   ⢀⣀⠤⠒⠊⠁
    │    ⢀⣀⠤⠒⠊⠁
  1 ┤⣀⠤⠒⠊⠁
    └┬──────────────────────┬───────────────────────┬───────────
     1                     10²                     10⁴
```

## energy

Stacked areas via the Stack stat: each layer sits on the sum of the ones below.
Source: [examples/energy.rs](examples/energy.rs)

```text
                energy mix, stacked (synthetic)
                  ▄▄ solar  ▄▄ wind  ▄▄ hydro
   │                                                 ⢀⣠⣶⣿⣿⣿⣦
10 ┤                                               ⢀⣴⣿⣿⣿⣿⣿⣿⣿
   │  ⣀⣠⣤⣤⣄⡀             ⢀⣀⣀⣀⣀⡀⢀⣤⣶⣾⣿⣶⣶⣤⣤⣄⣀        ⣠⣾⣿⣿⣿⣿⣿⣿⣿⣿
 8 ┤⣰⣾⣿⣿⣿⣿⣿⣿⣷⣴⣶⣶⣶⣶⣶⣶⣶⣶⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣶⣤⣤⣤⣤⣶⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
   │⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
 6 ┤⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
   │⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
 4 ┤⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
   │⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
 2 ┤⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
   │⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
 0 ┤⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿
   └┬──────┬───────┬──────┬───────┬──────┬──────┬───────┬──────┬
    0     10      20     30      40     50     60      70     80
```

## annotated

Annotations: a Rule for the target line, a Text note at data coordinates.
Source: [examples/annotated.rs](examples/annotated.rs)

```text
                 annotated loss (synthetic)
                     ── loss  ── target
  │⠑⢄
4 ┤ ⠈⠢⡀
  │   ⠑⠢⡀
3 ┤     ⠈⠢⣀
  │        ⠑⠢⣀
2 ┤           ⠉⠢⢄⣀            < converging
  │               ⠉⠒⠢⢄⣀
1 ┤                    ⠉⠉⠑⠒⠤⠤⢄⣀⣀⡀
  │⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣀⣈⣉⣉⣉⣑⣒⣒⣒⣒⣒⣤⣤⣤⣤⣤⣤⣤⣤⣤⣤⣀⣀⣀⣀⣀⣀⣀⣀
0 ┤
  └┬───────┬───────┬───────┬───────┬───────┬───────┬───────┬
   0      10      20      30      40      50      60      70
```

## correlation

Signed data on a diverging colormap centered at zero: correlation and anti-correlation read as opposite colors, and the colorbar spans symmetrically.
Source: [examples/correlation.rs](examples/correlation.rs)

```text
        correlation matrix (synthetic)
8 ┤▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓████  █ 1.0
  │▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓█████████████  █
6 ┤▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓█████████▓▓▓▓  █
  │▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓█████████▓▓▓▓  ▓ 0.5
  │▒▒▒▒▒▒▒▒▒▒▒▒▒░░░░░████▓▓▓▓▓▓▓▓▓▓▓▓▓  ▓
4 ┤▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓  ▓ 0.0
  │▓▓▓▓▒▒▒▒▒▓▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓  ▒
  │▓▓▓▓▒▒▒▒▒████░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓  ▒
2 ┤█████████▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  ░ -0.5
  │█████████▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  ░
0 ┤█████████▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  ░ -1.0
  └┬──────────┬──────────┬──────────┬──
  0.0        2.5        5.0        7.5
```

## confusion

A confusion matrix from the grammar, no preset: a Cells matrix on Bands axes — class names label rows and columns, counts sit on the cells as Text, and row 0 is the top band so the chart reads in matrix order.
Source: [examples/confusion.rs](examples/confusion.rs)

```text
             validation confusion
       │
       │   █████████  ░░░░░░░░░   ░░░░░░░░░
   cat ┤   ████38███  ░░░░░2░░░   ░░░░0░░░░
       │   █████████  ░░░░░░░░░   ░░░░░░░░░
t      │   ░░░░░░░░░  █████████   ░░░░░░░░░
r      │   ░░░░░░░░░  █████████   ░░░░░░░░░
u  dog ┤   ░░░░3░░░░  █████33██   ░░░░4░░░░
e      │
       │   ░░░░░░░░░  ░░░░░░░░░   █████████
  bird ┤   ░░░░1░░░░  ░░░░░5░░░   ████34███
       │   ░░░░░░░░░  ░░░░░░░░░   █████████
       │
       └──────────────────────────────────────
              cat         dog       bird
                       predicted
```

## attention

An attention map: token labels on both axes, a logarithmic colormap so weights spanning decades stay distinguishable, and the causal mask's zeros rendered as honest gaps — with decade ticks on the colorbar.
Source: [examples/attention.rs](examples/attention.rs)

```text
                    attention, layer 7 head 3
        │  █████                                            █
    The ┤  █████                                            █
        │  █████  █████                                     █
  robot ┤  █████  █████                                     █ 10⁻¹
        │  █████  █████                                     ▓
q   ate ┤  ▓▓▓▓▓  █████ █████                               ▓
u       │  ▓▓▓▓▓  █████ █████                               ▓
e       │  ▓▓▓▓▓  ▓▓▓▓▓ █████  █████                        ▓
r   the ┤  ▓▓▓▓▓  ▓▓▓▓▓ █████  █████                        ▒
y       │  ▒▒▒▒▒  ▓▓▓▓▓ ▓▓▓▓▓  █████  █████                 ▒ 10⁻³
    red ┤  ▒▒▒▒▒  ▓▓▓▓▓ ▓▓▓▓▓  █████  █████                 ▒
        │  ░░░░░  ▒▒▒▒▒ ▓▓▓▓▓  ▓▓▓▓▓  █████  █████          ▒
  apple ┤  ░░░░░  ▒▒▒▒▒ ▓▓▓▓▓  ▓▓▓▓▓  █████  █████          ░
        │  ░░░░░  █████ ▒▒▒▒▒  ▒▒▒▒▒  ▓▓▓▓▓  █████  █████   ░
      . ┤  ░░░░░  █████ ▒▒▒▒▒  ▒▒▒▒▒  ▓▓▓▓▓  █████  █████   ░
        │                                                   ░ 10⁻⁵
        └──────────────────────────────────────────────────
            The   robot   ate    the    red  apple    .
                                 key
```

## filters

Convolution filters as images: a Gabor bank with color opponency through Cells::rgb — direct colors, no colormap, and a luma shade ramp when the output is a plain pipe.
Source: [examples/filters.rs](examples/filters.rs)

```text
           0°                      57°                      120°
20 ┤                     20 ┤                     20 ┤
   │▓▓▒▒▓▓▒▒▓▓▒▒▓▓▒▒▓▓      │▓▒▒▓▓▓▓▓▓▒▒▓▓▓▓▓▓▒      │▒▓▓▓▓▓▓▒▒▒▓▓▓▓▓▓▒▒
15 ┤▓▓▒▒▓▓▒▒▓▓▒▒▓▓▒▒▓▓   15 ┤▓▓▓▓▓▓▓▒▓▓▓▓▓▒▓▓▓▓   15 ┤▓▓▓▓▓▓▓▓▓▓▒▓▒▓▓▓▓▓
   │▓▓▒▒▓▓▒▒██▒▒▓▓▒▒▓▓      │▓▒▒▓▓██▓▒░▒▓▓▓▓▓▒▒      │▒▓▓▓▓▓▒▒░▒▓█▓▓▓▒▒▒
10 ┤▓▓▒▒▓▓░▒██▒░▓▓▒▒▓▓   10 ┤▓▓▓▓▓▒▓▓▓█▓▓▒▒▓▓▓▓   10 ┤▓▓▓▓▓▒▓▓█▓▓▒▒▓▓▓▓▓
 5 ┤▓▓▒▒▓▓▒▒██▒▒▓▓▒▒▓▓    5 ┤▓▒▒▓▓▓▓▓▒▒▒▓█▓▓▓▒▒    5 ┤▒▓▓▓▓▓▓▒▒▒▓▓▓▓▓▒▒▒
   │▓▓▒▒▓▓▒▒▓▓▒▒▓▓▒▒▓▓      │▓▓▓▓▓▒▒▒▓▓▓▓▓▒▒▒▓▓      │▓▓▒▒▓▓▓▓▓▓▒▒▒▓▓▓▓▓
 0 ┤▓▓▒▒▓▓▒▒▓▓▒▒▓▓▒▒▓▓    0 ┤▓▒▒▓▓▓▓▓▒▒▒▓▓▓▓▓▒▒    0 ┤▒▓▓▓▓▓▓▒▒▓▓▓▓▓▓▒▒▒
   └┬──────────────────┬    └┬──────────────────┬    └┬──────────────────┬
    0                 20     0                 20     0                 20

        29° rgb                  92° rgb                  149° rgb
20 ┤                     20 ┤                     20 ┤
   │▓▒▓▓▓▒▒▓▓▓▒▒▓▓▓▒▒▓      │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓      │▓▓▒▓▓▓▒▒▓▓▓▒▒▓▓▓▒▒
15 ┤▓▓▒▓▓▓▓▒▓▓▓▓▒▓▓▓▓▒   15 ┤▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓   15 ┤▒▓▓▓▓▒▓▓▓▓▒▓▓▓▓▒▓▓
   │▓▓▓▒▒▓▓▓▒▒▓▓▓▒▒▓▓▒      │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓      │▓▓▓▒▒▓▓▓▒▒▓▓▓▒▒▓▓▓
10 ┤▒▓▓▓▓▒▓█▓▒▒▓▓▓▒▒▓▓   10 ┤▓▓▓▓▓▓▓▓▒▓▓▓▓▓▓▓▓▓   10 ┤▓▓▒▓▓▓▓▒▒▓▓▓▒▓▓▓▓▒
 5 ┤▓▒▒▓▓▓▒▒▓▓▓▒▒▓▓▒▒▓    5 ┤▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▒▒    5 ┤▒▒▓▓▓▒▒▓▓▓▒▒▓▓▓▒▒▓
   │▓▓▒▒▓▓▓▒▒▓▓▓▒▒▓▓▓▒      │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓      │▓▓▓▒▒▓▓▓▒▒▓▓▓▒▒▓▓▓
 0 ┤▒▓▓▓▒▒▓▓▓▒▓▓▓▒▒▓▓▓    0 ┤▒▒▒▒▒▒▒▒▒▒▒▓▒▒▒▒▒▒    0 ┤▓▓▒▒▓▓▓▒▒▓▓▓▒▒▓▓▓▒
   └┬──────────────────┬    └┬──────────────────┬    └┬──────────────────┬
    0                 20     0                 20     0                 20
```

## boundary

A decision boundary from the grammar, no preset: Cells::classes colors the feature plane by predicted class through the categorical palette, each region keeps a stable shade with matching legend swatches, and the training points sit on top.
Source: [examples/boundary.rs](examples/boundary.rs)

```text
                   5-NN decision regions
             ░░ adelie  ▒▒ gentoo  ▓▓ chinstrap
 3 ┤▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
   │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
   │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
 2 ┤▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
   │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓x▓x▓x▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
   │▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓x▓▓xx▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒▒
 1 ┤░░░░▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓x▓xx▓xxx▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒
   │░░░░░░░░░▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
   │░░░░░░░░░░░░▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
 0 ┤░░░░░░░░░░░░░░░░░▒▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
   │░░░░░░░░░░░░░░░░░░░░▒▒▒▓▓▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
-1 ┤░░░░░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒x▒▒▒xx▒▒▒▒▒▒▒▒▒▒▒▒▒▒
   │░░░░░░░xxx░x░x░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒xxxxx▒x▒▒▒▒▒▒▒▒▒▒▒▒▒▒
   │░░░░░░░░░░░x░xx░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒x▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
-2 ┤░░░░░░░x░░░x░xx░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
   │░░░░░░░░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
   │░░░░░░░░░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
-3 ┤░░░░░░░░░░░░░░░░░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒
   └─┬────────┬────────┬────────┬───────┬────────┬────────┬─
    -3       -2       -1        0       1        2        3
```

## density2d

A 2D histogram: point density on a grid, empty bins honestly blank.
Source: [examples/density2d.rs](examples/density2d.rs)

```text
              two clusters, binned (synthetic)
8 ┤                                    ░░░░░░░░         █ 60
  │                                  ░░░░░░▒▒░▒░░░░     █
7 ┤                                  ░░░▒▒▓▒▓▓▓▓▓░░░    █
  │                                   ░░░▒▓▓▓▓▒▓▓▒▒░░░  █
6 ┤                                    ░░░▒▒▓▒▒▓▒▒▒░░░  ▓
  │                                       ░░░▒▒▒▒░░░░░  ▓ 40
5 ┤                                         ░░░░▒░░░░░  ▓
  │                     ░░░░░░░                 ░░░     ▒
4 ┤                ░░░░░░▒░░░░░                         ▒
  │           ░░░░▒▒▒▓▒▒▒░░▒░                           ▒ 20
3 ┤       ░░▒▒▒▓▓▓▓▓▓▓▒▒░▒░                             ░
  │    ░░░▒▒▓██▓▓▓▒▒▒░░                                 ░
  │ ░░░░▒▒▒▒▒▒▒▒░░░                                     ░
2 ┤ ░░░░▒░░░░░░                                         ░ 0
  └┬──────┬──────┬──────┬──────┬───────┬──────┬──────┬─
   1      2      3      4      5       6      7      8
```

## contour

The MATLAB peaks function as iso-lines: marching squares, tick-chosen levels, a labeled legend.
Source: [examples/contour.rs](examples/contour.rs)

```text
                           the peaks function
           ── -6  ── -4  ── -2  ── 0  ── 2  ── 4  ── 6  ── 8
40 ┤                         ⣀⡠⠤⠒⠒⠒⠉⠉⠉⠉⠉⠉⠒⠒⠒⠤⠤⣀
   │⣀                     ⢠⠔⠉ ⢀⣀⠔⠒⠒⠉⢉⣉⣉⡉⠉⠒⠒⠢⢄  ⠉⠒⢄
   │ ⠉⠑⠒⠤⠤⠤⣀             ⡔⠁  ⡔⠁ ⢠⠒⠉⠉⠁  ⠈⠉⠉⠒⢄ ⠉⠢⡀  ⠑⡄
35 ┤        ⠉⠉⠒⠤⠤⣀       ⢇  ⢸  ⠠⡃    ⠶⠶    ⢀⠇  ⢸   ⠘⡄
   │              ⠉⠉⠒⠒⠤⣀  ⠑⠢⣀⠉⠒⢄⡈⠑⠒⠢⠤⠤⠤⠤⠤⠒⠊⢁⣀⡠⠔⠉    ⡇
30 ┤                    ⠉⠒⠢⢄⡀⠑⠒⠤⣈⣉⠉⠒⠒⠒⠒⢒⣒⣉⣉⣁⣀⣀⣀     ⠱⡀
   │               ⢀⠤⠤⠤⠤⢄⣀  ⠈⠑⢄   ⠉⠉⠉⠉⠉⠁       ⠉⡆    ⠈⠒⠢⡀
25 ┤              ⡰⠁      ⠱⡀  ⠈⡆        ⣀⠤⡀    ⢠⠃       ⠈⠢⡀
   │             ⢸        ⡰⠁ ⢀⠔⠁        ⠈⠒⠁    ⡎          ⢇
   │              ⢇⣀   ⣀⡠⠒⠁ ⡤⠊ ⣀⠔⠒⠒⠒⢄          ⡇          ⢸
20 ┤                ⠉⠉⠉ ⢀⡠⠒⠉⢀⠔⠊      ⢱         ⠑⢄       ⢀⠔⠃
   │                 ⢀⡠⠒⠁  ⡜⠁       ⣀⠎ ⢀⣀⠤⠔⠒⠤⠤⣀  ⠉⠑⠢⠤⠤⠔⠒⠉
15 ┤             ⣀⡠⠔⠊⠁    ⠘⠤⣀⣀⣀⣀⠤⠔⠒⣉⡠⠔⣊⡡⠤⠤⠤⠤⠤⢄⡀⠉⠒⠤⢄⡀
   │       ⣀⡠⠤⠒⠊⠉             ⣀⠤⠒⢊⣉⠤⢒⣉⠤⠔⠒⠒⠒⠒⠤⢄⠈⠑⠤⡀ ⠈⠉⠒⠤⣀
   │⣀⠤⠔⠒⠒⠊⠉              ⢀⡠⠔⠒⠉ ⡠⠒⠁⡠⠒⠁ ⡠⠔⠒⠒⠢⡄  ⠑⡄ ⠈⢢     ⠉⠉⠒⠤⢄⡀
10 ┤                   ⡠⠒⠁    ⢸   ⢇  ⠈⠢⠤⠤⠤⠒⠁  ⡤⠃  ⡸          ⠈⠑⠢⠤⣀
   │                 ⡠⠊       ⠈⠦⣀ ⠈⠑⠤⠤⠤⠤⠤⠤⠤⠔⠊⠉ ⣀⡠⠊                ⠉⠢⢄⣀
 5 ┤                 ⡇          ⠈⠑⠒⠢⠤⠤⣀⣀⣀⣀⡠⠤⠒⠒⠊                       ⠑⢆
   │                 ⢇
 0 ┤                 ⠈⢆
   └┬──────┬───────┬──────┬───────┬──────┬───────┬──────┬───────┬──────┬
    0      5      10     15      20     25      30     35      40     45
```

## quiver

A vector field: spiral flow into a sink, one arrow per grid point, drawn in data coordinates.
Source: [examples/quiver.rs](examples/quiver.rs)

```text
                        spiral flow into a sink
   2.0 ┤                                             ⠠⣤⡤⠤⠈⢟⠶⡒
       │                                   ⢀⡀ ⢤⣖⣂⡀⠹⡛⠥⢄⣘⠌⠑⠢⢌⣤⢌⣑⠢⣀
   1.5 ┤      ⡰   ⡰   ⢠⠂  ⡠⠂  ⣀⠔ ⢀⡠⣴⣀⡠⠤⢶⡧⠔⠚⠻⠋⠑⠒⠚⠂⠈⠉⢵⡶⠄ ⠹⡳⣒ ⠸⠙⠢⡀ ⠉⠂
       │     ⢠⠃  ⡰⠁  ⡰⠁⢀⢀⠎ ⣰⣤⡊ ⠼⠮⠁⠈⠉⠁   ⢀⡀ ⠠⣴⣂⣀⠈⠻⠣⠤⣈⠊⠑⠢⢄⣠⣄⣑⠢⣻⢕⠚⠒⢄
   1.0 ┤   ⡀ ⡎⡸⡄⡰⢁⡜⢸⣔⠥⡰⠟⠋⠁⡰⠉⢀ ⡠⠊⢠⣠⠔⠊⠼⠔⠒⠊⠙⠋⠉⠉⠁  ⠉⠡⣤⡄ ⠉⠟⢅ ⠘⠈⠢⢄⠈⢡⣑⢄ ⠁
       │   ⠱⣸⢤⢧⠱⠋⢡⢣⠈⢀⡰⠁⡄⢰⣜⠄⡄⠸⠚ ⡠⠉  ⡠ ⢀⣀⡠⠄⢴⠦⠤⠄⠙⠓⠢⠄⠁⠈⠑⠢⠠⣄⡉⠢⢸⢳⠒⠑⠼⠑⡍⠑⠄
   0.5 ┤    ⡅⢸⣸ ⢣⣜⢼ ⠸⠓⢱⠁⠈⢀⢸  ⢠⣔⠁ ⠸⠚  ⠉⠁       ⢤⡄  ⠟⢅  ⠉⢆   ⣕⡄ ⡜⡞⢤
       │    ⠘⠗⢹  ⡁⣷⢀ ⠰⣼⡆ ⠈⠗⡇ ⠈ ⢰  ⢀⡔  ⠰⠔⠂ ⠈⠑⠂  ⠈⠂  ⣄⡑ ⠰⢳⠓ ⠘⠘⡌⠂ ⢸⣌⠂
y      │     ⠱⣸⡟ ⠘⠗⡇  ⢁⡇⡀ ⢠⣧  ⠘⠟  ⠈        ⢠⠄  ⠺⡁  ⠘⡄  ⠈⣦⡀ ⠰⢷⡢ ⠊⢳⡑
   0.0 ┤      ⢜⣧⡠ ⠺⣵⠆ ⠈⢫   ⠱⡀  ⢘⡤  ⠘⠂   ⠁   ⠁⡀  ⣵⡄  ⢺⠂ ⠈⢸⢁  ⢸⢵⡄ ⣸⡗⢄
       │      ⠠⡙⡅ ⠠⡘⡄⡄ ⢤⢧⠆ ⢌⠙  ⠠⡀  ⠠⢄⡀  ⠔⠆  ⠜⠁  ⠇ ⡀ ⢸⢴⡀ ⠸⡟⠆ ⠁⢿⢈  ⣇⢴⡄
  -0.5 ┤       ⠓⡼⡜ ⠘⢝   ⠱⣀  ⢑⣴  ⠘⠓       ⢀⣀  ⡤⡆ ⢀⠝⠃  ⡇⠁⡀⢀⢇⢤⡆ ⡗⡝⢣ ⡏⡇⢘
       │      ⠐⢄⣘⢄⡖⢄⠤⢧⡇⠢⣈⠙⠂⠢⢄⡀⢀⠐⠢⢤⣄⠐⠒⠲⠗ ⠒⠉⠁ ⠊  ⣀⠊ ⡤⡆⠘⠐⡝⠇⠘⢀⠎⠁⡀⢣⢃⣠⢆⢳⠓⡏⢆
  -1.0 ┤      ⢀ ⠑⢍⢃⡀⠑⠢⡀⡄ ⢑⣴⣀ ⠘⠛⢂⣀  ⢀⣀⣀⣠⣄⣀⠤⠴⡖⡠⠔⠋⠃⡠⠊ ⠁⣀⠎⢀⣠⣴⠎⢒⠝⡇⡜⢁⠎⠘⡎⡸ ⠈
       │       ⠑⠤⡤⢕⣯⠢⢍⠙⠋⠑⠢⢄⡠⡉⠒⢢⣦⡀⠉⠩⠟⠂ ⠈⠁   ⢀⣀⡀⢀⡲⡖ ⡨⠛⠏ ⡰⠁⠁⢀⠎  ⢀⠎  ⢠⠃
  -1.5 ┤      ⠠⣀ ⠈⠢⣄⡆ ⠭⢮⣆ ⠐⠾⢗⣀⡀⠠⡤⠤⢄⣠⣦⡤⠔⢚⠷⠒⠊⠉⠟⠊⠁ ⠔⠉  ⠠⠊  ⠠⠃   ⠎   ⠎
       │        ⠉⠢⢍⡑⠛⡑⠢⢄⡐⡍⠑⢒⣬⣆⠈⠩⠽⠓ ⠈⠁
  -2.0 ┤           ⠬⠶⣵⡀⠒⠚⠛⠂
       └┬─────────┬──────────┬──────────┬─────────┬──────────┬─────────┬
       -3        -2         -1          0         1          2         3
                                        x
```

## boxes

Box plots: type-7 quartiles, Tukey whiskers, outliers — one Range mark with          body and marker channels per category.
Source: [examples/boxes.rs](examples/boxes.rs)

```text
                 flipper length by species
  230 ┤                                        ▀▀▜▀▀
      │                                          ▐
  220 ┤                                       ███████▌
      │                                       ━━━━━━━━
  210 ┤        ▄▄▄▄▄           ▀▀▜▀▀          ▀▀▀▜▀▀▀▘
m     │          ▌               ▐             ▄▄▟▄▄
m 200 ┤          ▌           ▐███████▌
      │      ▗▄▄▄▙▄▄▄        ▐━━━━━━━━
  190 ┤      ━━━━━━━━━       ▝▀▀▀▜▀▀▀▘
      │      ▝▀▀▀▛▀▀▀            ▐
  180 ┤          ▌               ▐
      │        ▄▄▙▄▄           ▀▀▀▀▀
  170 ┤          ▘
      └─────────────────────────────────────────────────────
              Adelie         Chinstrap        Gentoo
```

## violins

The same flippers as mirrored kernel densities — separation as a shape, not a summary.
Source: [examples/violins.rs](examples/violins.rs)

```text
          flipper length by species, as densities
  240 ┤                                          ⢠
  230 ┤                                        ⢀⣴⣿⣷⣄
      │                                       ⢀⣼⣿⣿⣿⣿⣀
  220 ┤                          ⣤          ⣠⣶⣿⣿⣿⣿⣿⣿⣿⣷⣄
      │          ⡇              ⢠⣿⡄         ⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠇
m 210 ┤          ⣿             ⢀⣿⣿⣿⡀         ⠛⠿⣿⣿⣿⣿⣿⠿⠟⠁
m 200 ┤        ⢀⣼⣿⣆         ⢀⣠⣶⣿⣿⣿⣿⣿⣶⣄⡀         ⠈⢿⠋
      │      ⣠⣾⣿⣿⣿⣿⣿⣦⣄     ⢰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡆         ⠸
  190 ┤    ⢰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⠄    ⠙⢿⣿⣿⣿⣿⣿⣿⣿⡿⠋
      │     ⠙⠿⣿⣿⣿⣿⣿⣿⣿⠟⠁       ⠉⠻⣿⣿⣿⠟⠉
  180 ┤       ⠈⠻⣿⣿⡿⠛            ⠸⣿⠇
      │         ⠘⣿⠁              ⣿
  170 ┤          ⡀               ⠉
      └─────────────────────────────────────────────────────
              Adelie         Chinstrap        Gentoo
```

## measurements

Error bars: a Range interval around each measured point.
Source: [examples/measurements.rs](examples/measurements.rs)

```text
     measurements with uncertainty (synthetic)
6 ┤           ⠐⡖
5 ┤            ⡇
  │     ⠈⡏     ⡇    ⠈⡏⠁
4 ┤     ⠐⠓    ⠈⠉    ⠠⠧⠄   ⢀⣀⡀                      ⢤
  │                        ⢸                       ⢸
3 ┤                        ⢸                 ⣀⡀    ⠼
2 ┤                       ⠐⠚⠂   ⠐⢲⠂          ⢸
  │                              ⢸     ⢹⠁    ⢸
1 ┤                             ⠐⠚⠂    ⢸     ⠉⠁
0 ┤                                    ⠚⠂
  └┬───────────┬───────────┬───────────┬───────────┬
   0           2           4           6           8
```

## timeseries

The Keeling curve: monthly CO2 at Mauna Loa since 1958 (NOAA), on a calendar axis.
Source: [examples/timeseries.rs](examples/timeseries.rs)

```text
                    atmospheric CO2 at Mauna Loa (NOAA)
  440 ┤
      │                                                                 ⢀⣠⣾⡎
  420 ┤                                                              ⢀⣤⣴⡾⠏⠁
      │                                                          ⢀⣠⣴⡾⠟⠋⠁
  400 ┤                                                       ⢠⣠⣴⡿⠟⠋⠁
      │                                                   ⢀⣤⣴⣾⡟⠛⠉
p     │                                              ⢀⣀⣴⣴⡾⠟⠏⠁
p 380 ┤                                          ⡀⣀⣤⣾⡿⠟⠋⠁
m     │                                     ⣀⣀⣤⣶⢿⠿⠛⠉
  360 ┤                               ⣀⣠⣴⣶⣶⣿⠿⠛⠛⠁
      │                         ⢀⣀⣤⣴⣶⡿⠻⠛⠙⠉⠉
  340 ┤                    ⣀⣀⣤⣶⣿⡿⠻⠛⠉
      │             ⡀⣀⣤⣦⣶⣶⠿⠻⠛⠉⠈
  320 ┤  ⡀⡀⣀⣄⣄⣄⣤⣦⣶⢿⠿⠻⠛⠉⠈⠈
      │⢲⢿⠿⠻⠻⠛⠙⠙⠉⠈
      └──┬─────────┬─────────┬─────────┬─────────┬─────────┬─────────┬──────
       1960      1970      1980      1990      2000      2010      2020
```

## multiples

Small multiples: a Grid of independent plots, axes shared by fixing          domains explicitly.
Source: [examples/multiples.rs](examples/multiples.rs)

```text
                alpha                                 beta
 5 ┤                                   5 ┤⠤⣀⡀                          ⣀⡠⠤⠄
   │    ⣀⣀⣀                 ⣀⣀⣀          │  ⠈⠒⢄                     ⢀⠤⠊
   │ ⣀⠔⠉   ⠑⠢⡀           ⢀⠔⠊   ⠉⠢⣀       │     ⠑⠢⡀                ⢀⠤⠃
 0 ┤⠜        ⠈⠢⡀       ⢀⠔⠊        ⠑⡄   0 ┤       ⠑⢢              ⡠⠃
   │           ⠈⠢⣀   ⣀⠔⠊           ⠈⠁    │         ⠑⢄          ⡠⠊
   │              ⠉⠉⠉                    │           ⠑⠤⡀    ⣀⡠⠊
-5 ┤                                  -5 ┤             ⠈⠉⠒⠒⠉
   └┬─────┬──────┬─────┬──────┬─────┬    └┬─────┬──────┬─────┬──────┬─────┬
    0    10     20    30     40    50     0    10     20    30     40    50

             alpha dist                             beta dist
20 ┤                                  10 ┤▃▃▃▃▃                      ██████
   │                           ██████    │█████▁▁▁▁▁▁           ▁▁▁▁▁██████
   │                           ██████    │███████████     ▄▄▄▄▄▄███████████
10 ┤                           ██████  5 ┤███████████▇▇▇▇▇█████████████████
   │█████           ▅▅▅▅▅▅███████████    │█████████████████████████████████
   │█████████████████████████████████    │█████████████████████████████████
 0 ┤█████████████████████████████████  0 ┤█████████████████████████████████
   └──┬─────────────┬─────────────┬──    └──┬─────────────┬─────────────┬──
    -2.5           0.0           2.5       -5             0             5
```

## corners

The asciichart homage: box-drawing corners, one glyph per column — with real axes underneath.
Source: [examples/corners.rs](examples/corners.rs)

```text
                          the corners style
 15 ┤              ╭───────────╮
    │            ╭─╯           ╰──╮
 10 ┤          ╭─╯                ╰─╮
    │        ╭─╯                    ╰╮
  5 ┤      ╭─╯                       ╰─╮
    │     ─╯                           ╰─╮
  0 ┤                                    ╰╮
    │                                     ╰─╮
 -5 ┤                                       ╰─╮
    │                                         ╰─╮                  ╭──
-10 ┤                                           ╰─╮              ╭─╯
    │                                             ╰──╮       ╭───╯
-15 ┤                                                ╰───────╯
    └┬──────────┬─────────┬──────────┬──────────┬──────────┬─────────┬
     0         10        20         30         40         50        60
```

## steps

Step charts: stairs hold values flat between indices; an ECDF climbs a distribution from zero to one.
Source: [examples/steps.rs](examples/steps.rs)

```text
         requests per window                      latency ecdf
30 ┤                     ⢸⠉⠉⠉⢹        1.00 ┤                           ⣀⡤⠞⠉
   │                     ⢸   ⢸             │                       ⢀⣠⠴⠚⠁
25 ┤                     ⢸   ⠘⠒⠒⠒⡆    0.75 ┤                    ⣀⣰⠚⠉
   │              ⢀⣀⣀⣀⣀⣀⣀⣸       ⡇         │                  ⢀⡞⠁
   │              ⢸              ⡇         │                 ⡴⠋
20 ┤       ⡤⠤⠤⠤⡄  ⢸              ⡇    0.50 ┤               ⢠⠞⠁
   │       ⡇   ⡇  ⢸              ⠉⠉⠉⠉      │             ⣠⠖⠋
15 ┤       ⡇   ⡇  ⢸                   0.25 ┤         ⢀⣠⠴⠋⠁
   │⣀⣀⣀⣀⣀⣀⣀⡇   ⠉⠉⠉⠉                        │      ⢀⣀⡴⠋
10 ┤                                  0.00 ┤   ⡤⠴⠚⠉
   └┬──────────┬─────────┬──────────┬      └┬─────────────┬─────────────┬──
    0          3         6          9       0             5            10
```

## charsets

The charset ladder: one curve at every subpixel density — solid blocks (octants, sextants, quadrants, half blocks), braille dots, and plain ASCII.
Source: [examples/charsets.rs](examples/charsets.rs)

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
