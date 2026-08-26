//! Hand-written help — a product surface, not a generated afterthought (D-C8).
//!
//! Every page carries curated, runnable pipeline examples and is snapshot-tested
//! so it cannot rot. `-h` is height, not help; help is `--help` only.

use crate::args::Command;

/// The help text for a topic: the top-level page, or one subcommand's page.
pub fn text(topic: Option<Command>) -> &'static str {
    match topic {
        None => TOP,
        Some(Command::Line) => LINE,
        Some(Command::Scatter) => SCATTER,
        Some(Command::Bar) => BAR,
        Some(Command::Hist) => HIST,
        Some(Command::Count) => COUNT,
        Some(Command::Density) => DENSITY,
        Some(Command::Box) => BOX,
        Some(Command::Ecdf) => ECDF,
        Some(Command::Violin) => VIOLIN,
        Some(Command::Hist2d) => HIST2D,
        Some(Command::Heatmap) => HEATMAP,
    }
}

const TOP: &str = "\
kaz — pipe data to an honest terminal plot

Usage:
  <data> | kaz <chart> [options]
  kaz <chart> [FILE] [options]

Charts:
  line     l   line chart, one line per series     y | xy | xyy | xyxy | yx
  scatter  s   scatter plot                        xy | xyy
  bar      b   one bar per label                   label value
  hist         histogram, automatic bins           columns of numbers
  count    c   value frequencies as bars           one column of labels
  density  d   kernel density estimate             columns of numbers
  ecdf         empirical cumulative distribution   columns of numbers
  box          box plot per column                 columns are groups
  violin       violin plot per column              columns are groups
  hist2d       2D histogram (density grid)         xy
  heatmap      shade a row-major matrix            rows of numbers

The plot goes to stderr, so stdout stays the data channel; -O echoes the input
through, letting the plot sit in the middle of a pipeline:

  cat loss.tsv | kaz line -t training
  cat data.tsv | kaz line -O | next-tool
  awk '{print $5}' access.log | kaz hist
  cut -f2 species.tsv | kaz count

Options:
  -o TARGET      plot destination: stderr (default), - for stdout, or a FILE
  -O             pass input through to stdout (mid-pipeline mode)
  -d CHAR        field separator (default: any run of whitespace)
  -H             first row is a header; its names label the series
  --fmt FMT      column mapping: y | xy | xyy | xyxy | yx
  --cols LIST    select and reorder columns first: comma-separated header names
                 (with -H) or 0-based indices, e.g. --cols time,loss or --cols 2,0
  --by COL       scatter: color points by this column's categories (name or index)
  --emit-code    print the equivalent malevich Rust program, data inlined,
                 instead of the plot — the pipe-to-program bridge
  -w N, -h N     frame width and height in cells (0..4096; default: detected)
  -t TITLE       plot title
  --xlabel TEXT  x-axis title
  --ylabel TEXT  y-axis title
  --xlim A,B     fix the x range to [A, B]
  --ylim A,B     fix the y range to [A, B]
  --log-x        log-scale the x axis
  --log-y        log-scale the y axis
  --time-x       read the x column as time (unix seconds or ISO 8601)
  --bins N       histogram bin count (hist; 1..1000000; default: automatic)
  --colormap M   heatmap/hist2d colors: viridis (default) | magma | cividis |
                 greys | red-blue | purple-orange
  --midpoint V   center the colormap on value V (for signed data; heatmap/hist2d)
  --log-color    logarithmic colormap: equal color steps per decade, zeros blank
  --labels-x A,B band labels across heatmap columns (comma-separated)
  --labels-y A,B band labels down heatmap rows, top to bottom
  --reduce R     dense-heatmap bucket summary: mean (default) | max | min | median
  --color WHEN   auto (default) | always | never
  --charset SET  auto (default) | ascii | half | quad | sextant | braille | octant
  --pixels WHEN  auto (default) | always | never   — sixel/kitty/iTerm2 image panel
  -q             suppress the unparsed-values tally
  --live         stream stdin, repainting a line in place (see below)
  --window N     live sliding-window length (1..1000000; default: frame width)
  --fps N        live repaint throttle (1..1000; default: 10)
  --rate         live: plot the per-sample delta of a monotonic counter
  --version      print version
  --help         this help; per chart: kaz <chart> --help

Note: -h is height, not help. Help is --help only.
Auto charset uses quadrants in UTF-8 and ASCII otherwise; set MALEVICH_CHARSET
or --charset to opt into a dense tier supported by your font.

Live mode (line only):
  --live reads stdin forever, one value per line, and repaints a sliding window
  in place — the final frame stays in your scrollback, and Ctrl-C restores the
  cursor. Feed it a live source:

    ping -i.2 host | grep -oE 'time=[0-9.]+' | tr -d 'time=' | kaz line --live
    vmstat 1 | awk 'NR>2{print $1}' | kaz line --live -t runnable
    while :; do cat /proc/loadavg | cut -d' ' -f1; sleep 1; done | kaz line --live

  IMPORTANT: if the plot seems frozen, the producer is buffering — pipes hold
  output until a block fills. Unbuffer at the source:
    stdbuf -oL producer | kaz line --live       # force line buffering
    grep --line-buffered ...                     # grep's own flag
    awk '{print; fflush()}'                      # flush every line
  --rate turns a monotonic counter (bytes, packets) into a per-interval rate.

Data is parsed as fields, not CSV — no quoting, no embedded delimiters. For real
CSV, pre-shape upstream:
  xsv select 2,5 data.csv | kaz line
  mlr --c2p cut -f x,y data.csv | kaz line -H
";

const LINE: &str = "\
kaz line — line chart, one line per series

Usage:
  <data> | kaz line [options]
  kaz line FILE [options]

Columns (--fmt, default by column count):
  y      each column is a line over its row index      (default: 1 column)
  xy     first column x, second column y
  xyy    first column x, every remaining column a line  (default: 2+ columns)
  xyxy   columns pair up: (x0,y0) (x1,y1) ...
  yx     first column y, second column x                (YouPlot compatibility)

Examples:
  cat loss.tsv | kaz line -t training
  cat data.tsv | kaz line -O | next-tool        # plot on stderr, data flows on
  paste xs ys | kaz line --fmt xy
  kaz line metrics.tsv -H                        # header names label the lines
  seq 1 100 | awk '{print $1, sqrt($1)}' | kaz line
  ping -i.2 host | grep -oE 'time=[0-9.]+' | tr -d 'time=' | kaz line --live

Live streaming is line-only; see `kaz --help` for --live, --window, --fps,
--rate, and the producer-buffering note.

Shared options: kaz --help
";

const SCATTER: &str = "\
kaz scatter — scatter plot

Usage:
  <data> | kaz scatter [options]
  kaz scatter FILE [options]

Columns (--fmt):
  xy     first column x, second column y            (default: 2 columns)
  xyy    first column x, every remaining column a series
  y      a single column against its row index

Grouping: --by COL pulls one column out as categories — each group gets a
palette color and a legend entry (portable marker shapes when piped), and the
remaining columns are x and y. Name the column (with -H) or give its 0-based
index.

Examples:
  paste heights weights | kaz scatter -t growth
  awk '{print $3, $4}' points.tsv | kaz scatter
  kaz scatter samples.tsv -H --xlabel dose --ylabel response
  kaz scatter penguins.tsv -H --by species

Shared options: kaz --help
";

const BAR: &str = "\
kaz bar — one bar per label, rising from zero

Usage:
  <data> | kaz bar [options]
  kaz bar FILE [options]

Input: `label value` per row — the first field names the bar, the second is its
height. Rows with no value leave a gap.

Examples:
  printf 'a 3\\nb 7\\nc 5\\n' | kaz bar
  awk '{print $1, $2}' totals.tsv | kaz bar -t revenue
  kaz bar sales.tsv -H

For value frequencies (counting bare labels), use `kaz count`.

Shared options: kaz --help
";

const HIST: &str = "\
kaz hist — histogram

Usage:
  <data> | kaz hist [options]
  kaz hist FILE [options]

Input: every numeric field is pooled into one distribution. Bins are sized
automatically (Sturges / Freedman-Diaconis) with nice decimal edges, or fixed
with --bins N.

Examples:
  awk '{print $5}' access.log | kaz hist -t latency
  cut -f2 measurements.tsv | kaz hist --bins 30
  kaz hist samples.txt --xlim 0,100

Shared options: kaz --help
";

const DENSITY: &str = "\
kaz density — kernel density estimate

Usage:
  <data> | kaz density [options]
  kaz density FILE [options]

Input: every numeric field is pooled, then drawn as a smooth Gaussian-KDE
curve — a histogram without the bin-edge arbitrariness.

Examples:
  cut -f3 samples.tsv | kaz density -t weights
  awk '{print $5}' access.log | kaz density

Shared options: kaz --help
";

const ECDF: &str = "\
kaz ecdf — empirical cumulative distribution

Usage:
  <data> | kaz ecdf [options]
  kaz ecdf FILE [options]

Input: every numeric field is pooled, then drawn as a step from 0 to 1 — the
fraction of values at or below each point. Reads quantiles straight off the y
axis, no binning.

Examples:
  cut -f2 latencies.tsv | kaz ecdf -t latency
  kaz ecdf samples.txt

Shared options: kaz --help
";

const BOX: &str = "\
kaz box — a box plot per column

Usage:
  <data> | kaz box [options]
  kaz box FILE [options]

Input: each column is a group — five-number summary (median, quartiles, Tukey
whiskers) with outliers as dots. -H names the groups; otherwise they are
numbered.

Examples:
  paste control treated | kaz box -H
  awk '{print $2, $3, $4}' trials.tsv | kaz box

Shared options: kaz --help
";

const VIOLIN: &str = "\
kaz violin — a violin plot per column

Usage:
  <data> | kaz violin [options]
  kaz violin FILE [options]

Input: each column is a group, drawn as a mirrored density — the shape a box
plot summarizes. -H names the groups.

Examples:
  paste a b c | kaz violin -H
  kaz violin measurements.tsv -H -t distributions

Shared options: kaz --help
";

const HIST2D: &str = "\
kaz hist2d — 2D histogram (density grid)

Usage:
  <data> | kaz hist2d [options]
  kaz hist2d FILE [options]

Input: two columns of x, y points, binned onto a uniform grid and shaded by
count, with a colorbar. Empty cells stay blank — no data is never a little data.

Examples:
  awk '{print $1, $2}' points.tsv | kaz hist2d
  kaz hist2d samples.tsv --time-x
  kaz hist2d samples.tsv --colormap magma

Shared options: kaz --help
";

const HEATMAP: &str = "\
kaz heatmap — shade a row-major matrix

Usage:
  <data> | kaz heatmap [options]
  kaz heatmap FILE [options]

Input: each row of numbers is a row of the grid (first line on top), shaded by
value with a colorbar. Missing cells stay blank.

Pick colors with --colormap (viridis, magma, cividis, greys, red-blue,
purple-orange); for signed data, --midpoint V centers the map on V so the
colorbar spans symmetrically — the honest encoding for correlations and
differences; for data spanning decades (attention weights, spectral power),
--log-color gives every decade equal color steps and renders zeros as gaps.

Label the rows and columns with --labels-x/--labels-y (comma-separated band
names, rows top to bottom) — confusion matrices and attention maps. A matrix
denser than the terminal reduces honestly per screen bucket: the mean by
default, or --reduce max to keep sparse spikes visible.

Examples:
  kaz heatmap confusion.tsv --labels-x cat,dog --labels-y cat,dog
  awk '{print $2, $3, $4, $5}' grid.tsv | kaz heatmap
  kaz heatmap correlations.tsv --colormap red-blue --midpoint 0
  kaz heatmap attention.tsv --log-color --reduce max

Shared options: kaz --help
";

const COUNT: &str = "\
kaz count — value frequencies as bars

Usage:
  <data> | kaz count [options]
  kaz count FILE [options]

Input: the first field of each row is a category; kaz tallies how often each
appears and draws one bar per value, most frequent first. The strongest tool for
logs — no `sort | uniq -c` needed.

Examples:
  cut -f2 species.tsv | kaz count
  awk '{print $9}' access.log | kaz count -t status-codes
  git log --format='%an' | kaz count -t commits-by-author

Shared options: kaz --help
";
