# Changelog — kaz (malevich-cli)

Versioned on its own semver (flags, not marks). Built entirely on the public
malevich API.

## Unreleased

- Heatmaps grow the ML face of the core's new machinery: `--labels-x` and
  `--labels-y` put band labels on both axes (rows read top to bottom, like
  the printed matrix — confusion matrices and attention maps), `--log-color`
  makes the colormap logarithmic for values spanning decades (mutually
  exclusive with `--midpoint`; zeros render as gaps), and `--reduce
  mean|max|min|median` chooses how a matrix denser than the terminal
  summarizes each screen bucket (`max` keeps sparse spikes visible).
  `--emit-code` covers all of it, compile-tested as always.

## 0.2.1 — 2026-08-24

- Numeric `--cols` and `--by` selectors now fail when they are outside the input
  schema instead of silently producing empty data; oversized indices get a distinct
  error, and diagnostics include the valid range.
- `--time-x` rejects impossible civil dates and times, including invalid leap days,
  out-of-range clock fields, and offsets that cross the supported timestamp range.
- Rendering and `--emit-code` now consume the same prepared recipe and shared parsed
  channels. This removes duplicate chart interpretation, keeps emitted programs in
  lockstep with rendered plots, and avoids copying a shared x column for every series.
- Checked library preset failures propagate as normal `kaz` errors instead of
  reaching convenience assertions.

## 0.2.0 — 2026-08-15

- `--emit-code` prints the equivalent malevich Rust program instead of the
  plot — same presets and grammar calls, the parsed data inlined as literals,
  a copy-pasteable bridge from an exploratory pipe to a real program. Every
  emission shape is compile-tested in CI.
- `--cols LIST` selects and reorders columns before any chart reads them, by
  header name (with `-H`) or 0-based index; unknown names fail with the
  available header listed.
- `--by COL` on `scatter` pulls one column out as categories: each group takes
  a palette color and a legend entry through the library's `color_by` channel,
  with marker shapes keeping groups apart in piped plain output.
- `--colormap` picks the colors for `heatmap` and `hist2d` from the named set
  (`viridis`, `magma`, `cividis`, `greys`, `red-blue`, `purple-orange`), and
  `--midpoint V` centers the map on a data value — signed grids (correlations,
  differences) get opposite signs in opposite colors and a symmetric colorbar.

## 0.1.0 — 2026-08-04

First release: the full surface of the CLI plan (private/CLI.md), M-C1 through
M-C3 in one cut.

- Charts: `line` (`l`), `scatter` (`s`), `bar` (`b`), `hist`, `count` (`c`),
  `density` (`d`), `ecdf`, `box`, `violin`, `hist2d`, `heatmap`. `box`/`violin`
  read each column as a group (`-H` names them); `hist2d` reads `x y` pairs;
  `heatmap` reads rows of a matrix (first line on top).
- Stdin framing: whitespace-run default, single-char `-d`, `-H` header row,
  `--fmt y|xy|xyy|xyxy|yx` column mapping with per-column-count defaults.
- Stream wiring: plot on stderr by default, `-o -` to stdout, `-o FILE`, and
  `-O` passthrough — input echoed to stdout line by line as it is read, so a
  downstream consumer starts before the upstream finishes and the plot can sit
  mid-pipeline. Color keyed to the destination stream; an explicit
  `--color always` beats an inherited `NO_COLOR`.
- Ladders: `--color`, `--charset`, `--pixels` (sixel/kitty/iTerm2 from a pipe).
- Furniture: `-t`, `--xlabel`/`--ylabel`, `--xlim`/`--ylim`, `--log-x`/`--log-y`,
  `-w`/`-h`. An unparsed-field tally on stderr, silenced with `-q`. A flag the
  chosen chart would silently ignore is a usage error, not a no-op.
- `--time-x`: unix epoch (seconds, or milliseconds past ~1e11) or an ISO-8601
  subset (`YYYY-MM-DD[ T HH:MM[:SS[.fff]]][Z|±HH:MM]`), parsed in-house, no
  chrono. Applies to `line`, `scatter`, `hist2d`.
- `--bins N`: fix the histogram bin count — the exact `stat::Bins::new` +
  `Bars::spans` expansion `hist` uses, minus the automatic count.
- `--live` (line only): read stdin forever, one value per line, repaint a
  sliding window in place (cursor up, erase down — no alt-screen, so the final
  frame stays in scrollback; cursor hidden while repainting, restored on EOF,
  SIGINT, or a closed pipe). `--window N` sizes the window (default: the frame
  width), `--fps N` throttles repaints (default 10), `--rate` plots the
  per-sample delta of a monotonic counter. `--help` documents the
  producer-buffering footgun (`stdbuf -oL`, `grep --line-buffered`, `fflush`).
- Packaging: hand-written shell completions (bash, zsh, fish) under
  `completions/` and a man page at `man/kaz.1`, both guarded against subcommand
  drift by a test. A Homebrew tap (`shergin/homebrew-tap`) whose formula —
  canonically `homebrew/kaz.rb` — builds from source and installs the binary,
  completions, and man page (`brew install shergin/tap/kaz`). Prebuilt bottles
  (release CI) are the remaining reach.
