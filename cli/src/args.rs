//! Argument model and the lexopt parser.
//!
//! Every flag names an existing malevich concept — a [`Frame`](malevich::Frame)
//! field, a preset argument, a scale option, or plot furniture (D-C11). Parsing is
//! flag-uniform: the subcommand only selects the chart mapping and the help text,
//! so one loop handles every option regardless of which chart follows.

use std::path::PathBuf;

use lexopt::prelude::*;
use malevich::Charset;
use malevich::scale::Colormap;
use malevich::stat::Reducer;

const MAX_FRAME_DIMENSION: usize = 4096;
const MAX_FRAME_CELLS: usize = 4 * 1024 * 1024;
const MAX_BINS: usize = 1_000_000;
const MAX_WINDOW: usize = 1_000_000;
const MAX_FPS: usize = 1_000;

/// The chart subcommand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Line,
    Scatter,
    Bar,
    Hist,
    Count,
    Density,
    Box,
    Ecdf,
    Violin,
    Hist2d,
    Heatmap,
}

impl Command {
    /// Resolves a subcommand name or its one-letter alias.
    fn parse(name: &str) -> Option<Command> {
        Some(match name {
            "line" | "l" => Command::Line,
            "scatter" | "s" => Command::Scatter,
            "bar" | "b" => Command::Bar,
            "hist" => Command::Hist,
            "count" | "c" => Command::Count,
            "density" | "d" => Command::Density,
            "box" => Command::Box,
            "ecdf" => Command::Ecdf,
            "violin" => Command::Violin,
            "hist2d" => Command::Hist2d,
            "heatmap" => Command::Heatmap,
            _ => return None,
        })
    }

    /// Whether `--time-x` applies: only charts with a numeric x drawn from an
    /// input column.
    pub fn has_time_axis(self) -> bool {
        matches!(self, Command::Line | Command::Scatter | Command::Hist2d)
    }

    /// The canonical name, for messages.
    pub fn name(self) -> &'static str {
        match self {
            Command::Line => "line",
            Command::Scatter => "scatter",
            Command::Bar => "bar",
            Command::Hist => "hist",
            Command::Count => "count",
            Command::Density => "density",
            Command::Box => "box",
            Command::Ecdf => "ecdf",
            Command::Violin => "violin",
            Command::Hist2d => "hist2d",
            Command::Heatmap => "heatmap",
        }
    }
}

/// Where the plot is written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output {
    /// The default: plot on stderr, stdout free for data.
    Stderr,
    /// `-o -`: plot on stdout (disables `-O`).
    Stdout,
    /// `-o FILE`: a plain frame written to a file.
    File(PathBuf),
}

/// How columns map onto axes (D-C6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fmt {
    /// Each column a y-series; x is the row index.
    Y,
    /// First column x, second column y (a single series).
    Xy,
    /// First column x, every remaining column a y-series sharing it.
    Xyy,
    /// Columns pair up: `(x0,y0)`, `(x1,y1)`, … — each pair its own series.
    Xyxy,
    /// First column y, second column x (YouPlot compatibility).
    Yx,
}

impl Fmt {
    fn parse(name: &str) -> Option<Fmt> {
        Some(match name {
            "y" => Fmt::Y,
            "xy" => Fmt::Xy,
            "xyy" => Fmt::Xyy,
            "xyxy" => Fmt::Xyxy,
            "yx" => Fmt::Yx,
            _ => return None,
        })
    }
}

/// The `--color` escape hatch. `Auto` and the two overrides ride the NO_COLOR /
/// CLICOLOR_FORCE precedence malevich already documents (see `output`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ColorChoice {
    #[default]
    Auto,
    Always,
    Never,
}

/// The `--pixels` ladder (D-C10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PixelsChoice {
    /// Real image when the destination is a terminal that speaks a protocol.
    #[default]
    Auto,
    /// Attempt pixels even from a pipe (falls back to cells when undetected).
    Always,
    /// Never pixels; always cell output.
    Never,
}

/// The `--charset` override. `Auto` keeps the detected tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CharsetChoice {
    #[default]
    Auto,
    Fixed(Charset),
}

impl CharsetChoice {
    fn parse(name: &str) -> Option<CharsetChoice> {
        Some(match name {
            "auto" => CharsetChoice::Auto,
            "ascii" => CharsetChoice::Fixed(Charset::Ascii),
            "half" => CharsetChoice::Fixed(Charset::HalfBlocks),
            "quad" => CharsetChoice::Fixed(Charset::Quadrants),
            "sextant" => CharsetChoice::Fixed(Charset::Sextants),
            "octant" => CharsetChoice::Fixed(Charset::Octants),
            "braille" => CharsetChoice::Fixed(Charset::Braille),
            _ => return None,
        })
    }
}

/// A fully parsed invocation of one chart subcommand.
#[derive(Debug, Clone)]
pub struct Args {
    pub command: Command,
    /// A positional input file, or stdin when absent.
    pub input: Option<PathBuf>,
    pub output: Output,
    pub passthrough: bool,
    pub delimiter: Option<char>,
    pub header: bool,
    pub fmt: Option<Fmt>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub title: Option<String>,
    pub xlabel: Option<String>,
    pub ylabel: Option<String>,
    pub xlim: Option<(f64, f64)>,
    pub ylim: Option<(f64, f64)>,
    pub log_x: bool,
    pub log_y: bool,
    pub time_x: bool,
    /// Explicit histogram bin count (`--bins`); auto when absent.
    pub bins: Option<usize>,
    /// Heatmap/hist2d colormap (`--colormap`, centered by `--midpoint`,
    /// logarithmic via `--log-color`); the default map when absent.
    pub colormap: Option<Colormap>,
    /// Heatmap band labels across the columns (`--labels-x`).
    pub labels_x: Option<Vec<String>>,
    /// Heatmap band labels down the rows, top to bottom (`--labels-y`).
    pub labels_y: Option<Vec<String>>,
    /// Heatmap bucket reduction (`--reduce`); the mean box filter when absent.
    pub reduce: Option<Reducer>,
    /// Column projection (`--cols`): selectors (header names or 0-based
    /// indices) applied to the framed table before any chart reads it.
    pub cols: Option<Vec<String>>,
    /// Grouping column for scatter (`--by`): a header name or 0-based index
    /// whose values become the `color_by` categories.
    pub by: Option<String>,
    /// Print the equivalent malevich Rust program instead of the plot
    /// (`--emit-code`).
    pub emit_code: bool,
    pub color: ColorChoice,
    pub charset: CharsetChoice,
    pub pixels: PixelsChoice,
    pub quiet: bool,
    /// Live streaming mode (`--live`): read stdin forever, repaint in place.
    pub live: bool,
    /// Sliding-window length (`--window`); the frame width when absent.
    pub window: Option<usize>,
    /// Repaint throttle in frames per second (`--fps`); 10 when absent.
    pub fps: Option<usize>,
    /// Plot the per-sample delta of a monotonic counter (`--rate`).
    pub rate: bool,
}

/// What a parse resolved to: run a chart, or a meta action that prints and exits.
#[derive(Debug, Clone)]
pub enum Outcome {
    Run(Box<Args>),
    /// `--help`: top-level when no subcommand was seen, else that subcommand's page.
    Help(Option<Command>),
    Version,
}

/// A usage error. Printed as `kaz: {0}` to stderr; exit code 2.
#[derive(Debug)]
pub struct Fail(pub String);

impl std::fmt::Display for Fail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<lexopt::Error> for Fail {
    fn from(error: lexopt::Error) -> Fail {
        Fail(error.to_string())
    }
}

/// Parses the process arguments into an [`Outcome`].
pub fn parse() -> Result<Outcome, Fail> {
    parse_from(lexopt::Parser::from_env())
}

pub(crate) fn parse_from(mut parser: lexopt::Parser) -> Result<Outcome, Fail> {
    let mut command: Option<Command> = None;
    let mut input: Option<PathBuf> = None;
    let mut output = Output::Stderr;
    let mut passthrough = false;
    let mut delimiter = None;
    let mut header = false;
    let mut fmt = None;
    let mut width = None;
    let mut height = None;
    let mut title = None;
    let mut xlabel = None;
    let mut ylabel = None;
    let mut xlim = None;
    let mut ylim = None;
    let mut log_x = false;
    let mut log_y = false;
    let mut time_x = false;
    let mut bins = None;
    let mut colormap = None;
    let mut midpoint = None;
    let mut log_color = false;
    let mut labels_x = None;
    let mut labels_y = None;
    let mut reduce = None;
    let mut cols = None;
    let mut by = None;
    let mut emit_code = false;
    let mut color = ColorChoice::Auto;
    let mut charset = CharsetChoice::Auto;
    let mut pixels = PixelsChoice::Auto;
    let mut quiet = false;
    let mut live = false;
    let mut window = None;
    let mut fps = None;
    let mut rate = false;

    while let Some(arg) = parser.next()? {
        match arg {
            // `--help` is long-only: `-h` is reserved for height (D: the flag budget
            // is one screen, and height earns the short form more than help does).
            Long("help") => return Ok(Outcome::Help(command)),
            Short('V') | Long("version") => return Ok(Outcome::Version),
            Value(value) => {
                if command.is_none() {
                    let name = value.to_string_lossy();
                    command = Some(Command::parse(&name).ok_or_else(|| {
                        Fail(format!("unknown subcommand `{name}` (try `kaz --help`)"))
                    })?);
                } else if input.is_none() {
                    input = Some(PathBuf::from(value));
                } else {
                    return Err(Fail(format!(
                        "unexpected extra argument `{}`",
                        value.to_string_lossy()
                    )));
                }
            }
            Short('o') | Long("output") => {
                let value = parser.value()?.string()?;
                output = if value == "-" {
                    Output::Stdout
                } else {
                    Output::File(PathBuf::from(value))
                };
            }
            Short('O') => passthrough = true,
            Short('d') | Long("delimiter") => {
                let value = parser.value()?.string()?;
                let mut chars = value.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => delimiter = Some(c),
                    _ => {
                        return Err(Fail(format!(
                            "-d takes a single character, got `{value}` \
                             (whitespace is the default; real CSV: pipe through `xsv`/`mlr`)"
                        )));
                    }
                }
            }
            Short('H') | Long("header") => header = true,
            Long("fmt") => {
                let value = parser.value()?.string()?;
                fmt = Some(Fmt::parse(&value).ok_or_else(|| {
                    Fail(format!("--fmt is one of y|xy|xyy|xyxy|yx, got `{value}`"))
                })?);
            }
            Short('w') | Long("width") => {
                width = Some(parse_bounded(
                    "--width",
                    &parser.value()?.string()?,
                    0,
                    MAX_FRAME_DIMENSION,
                )?)
            }
            Short('h') | Long("height") => {
                height = Some(parse_bounded(
                    "--height",
                    &parser.value()?.string()?,
                    0,
                    MAX_FRAME_DIMENSION,
                )?)
            }
            Short('t') | Long("title") => title = Some(parser.value()?.string()?),
            Long("xlabel") => xlabel = Some(parser.value()?.string()?),
            Long("ylabel") => ylabel = Some(parser.value()?.string()?),
            Long("xlim") => xlim = Some(parse_pair("--xlim", &parser.value()?.string()?)?),
            Long("ylim") => ylim = Some(parse_pair("--ylim", &parser.value()?.string()?)?),
            Long("log-x") => log_x = true,
            Long("log-y") => log_y = true,
            Long("time-x") => time_x = true,
            Long("bins") => {
                bins = Some(parse_bounded(
                    "--bins",
                    &parser.value()?.string()?,
                    1,
                    MAX_BINS,
                )?);
            }
            Long("cols") => {
                let value = parser.value()?.string()?;
                let selectors: Vec<String> = value
                    .split(',')
                    .map(|selector| selector.trim().to_owned())
                    .collect();
                if selectors.iter().any(String::is_empty) {
                    return Err(Fail(format!(
                        "--cols wants comma-separated column names or 0-based indices, got `{value}`"
                    )));
                }
                cols = Some(selectors);
            }
            Long("by") => by = Some(parser.value()?.string()?),
            Long("emit-code") => emit_code = true,
            Long("colormap") => {
                let value = parser.value()?.string()?;
                colormap = Some(Colormap::named(&value).ok_or_else(|| {
                    Fail(format!(
                        "--colormap is one of {}, got `{value}`",
                        Colormap::NAMES.join("|")
                    ))
                })?);
            }
            Long("log-color") => log_color = true,
            Long("labels-x") => {
                labels_x = Some(parse_labels("--labels-x", &parser.value()?.string()?)?)
            }
            Long("labels-y") => {
                labels_y = Some(parse_labels("--labels-y", &parser.value()?.string()?)?)
            }
            Long("reduce") => {
                let value = parser.value()?.string()?;
                reduce = Some(match value.as_str() {
                    "mean" => Reducer::Mean,
                    "max" => Reducer::Max,
                    "min" => Reducer::Min,
                    "median" => Reducer::Median,
                    _ => {
                        return Err(Fail(format!(
                            "--reduce is mean|max|min|median, got `{value}`"
                        )));
                    }
                });
            }
            Long("midpoint") => {
                let value = parser.value()?.string()?;
                let center: f64 = value
                    .parse()
                    .ok()
                    .filter(|center: &f64| center.is_finite())
                    .ok_or_else(|| {
                        Fail(format!("--midpoint needs a finite number, got `{value}`"))
                    })?;
                midpoint = Some(center);
            }
            Long("color") => {
                let value = parser.value()?.string()?;
                color = match value.as_str() {
                    "auto" => ColorChoice::Auto,
                    "always" => ColorChoice::Always,
                    "never" => ColorChoice::Never,
                    _ => return Err(Fail(format!("--color is auto|always|never, got `{value}`"))),
                };
            }
            Long("charset") => {
                let value = parser.value()?.string()?;
                charset = CharsetChoice::parse(&value).ok_or_else(|| {
                    Fail(format!(
                        "--charset is auto|ascii|half|quad|sextant|braille|octant, got `{value}`"
                    ))
                })?;
            }
            Long("pixels") => {
                let value = parser.value()?.string()?;
                pixels = match value.as_str() {
                    "auto" => PixelsChoice::Auto,
                    "always" => PixelsChoice::Always,
                    "never" => PixelsChoice::Never,
                    _ => {
                        return Err(Fail(format!(
                            "--pixels is auto|always|never, got `{value}`"
                        )));
                    }
                };
            }
            Short('q') | Long("quiet") => quiet = true,
            Long("live") => live = true,
            Long("window") => {
                window = Some(parse_bounded(
                    "--window",
                    &parser.value()?.string()?,
                    1,
                    MAX_WINDOW,
                )?);
            }
            Long("fps") => {
                fps = Some(parse_bounded(
                    "--fps",
                    &parser.value()?.string()?,
                    1,
                    MAX_FPS,
                )?);
            }
            Long("rate") => rate = true,
            _ => return Err(Fail(arg.unexpected().to_string())),
        }
    }

    let Some(command) = command else {
        // No subcommand: `kaz` alone shows the top-level page.
        return Ok(Outcome::Help(None));
    };

    // `-o -` puts the plot on stdout; there is no data channel left to pass through.
    if passthrough && output == Output::Stdout {
        return Err(Fail(
            "-O passes input to stdout, but -o - already sends the plot there".into(),
        ));
    }

    if live {
        if command != Command::Line {
            return Err(Fail(format!(
                "--live streams a single line; `{}` is not supported",
                command.name()
            )));
        }
        if passthrough {
            return Err(Fail("-O is not supported with --live".into()));
        }
        if matches!(output, Output::File(_)) {
            return Err(Fail(
                "--live repaints a terminal; -o FILE is not supported".into(),
            ));
        }
        // The x axis is the sliding window itself, and input is one value per
        // line; flags that shape a data x axis or reframe columns would
        // silently do nothing — reject them like the stray live flags below.
        if time_x || xlim.is_some() || log_x || fmt.is_some() || header {
            return Err(Fail(
                "--live plots a sliding window of single values; \
                 --time-x/--xlim/--log-x/--fmt/-H do not apply"
                    .into(),
            ));
        }
        if pixels == PixelsChoice::Always {
            return Err(Fail(
                "--live repaints cells in place; --pixels always is not supported".into(),
            ));
        }
    } else if window.is_some() || fps.is_some() || rate {
        return Err(Fail("--window/--fps/--rate only apply with --live".into()));
    }

    // A flag the chosen chart would silently ignore is a lie, not a no-op.
    if bins.is_some() && command != Command::Hist {
        return Err(Fail(format!(
            "--bins only applies to hist, not `{}`",
            command.name()
        )));
    }
    if fmt.is_some() && !matches!(command, Command::Line | Command::Scatter) {
        return Err(Fail(format!(
            "--fmt only applies to line and scatter, not `{}`",
            command.name()
        )));
    }
    if time_x && !command.has_time_axis() {
        return Err(Fail(format!(
            "--time-x only applies to line, scatter, and hist2d, not `{}`",
            command.name()
        )));
    }
    if let (Some(width), Some(height)) = (width, height)
        && width
            .checked_mul(height)
            .is_none_or(|cells| cells > MAX_FRAME_CELLS)
    {
        return Err(Fail(format!(
            "--width × --height must not exceed {MAX_FRAME_CELLS} cells"
        )));
    }
    if (colormap.is_some() || midpoint.is_some() || log_color)
        && !matches!(command, Command::Heatmap | Command::Hist2d)
    {
        return Err(Fail(format!(
            "--colormap, --midpoint, and --log-color only apply to heatmap and hist2d, not `{}`",
            command.name()
        )));
    }
    if log_color && midpoint.is_some() {
        return Err(Fail(
            "--log-color and --midpoint are mutually exclusive: a ramp is centered or logarithmic, not both".into(),
        ));
    }
    if (labels_x.is_some() || labels_y.is_some()) && command != Command::Heatmap {
        return Err(Fail(format!(
            "--labels-x and --labels-y only apply to heatmap, not `{}`",
            command.name()
        )));
    }
    if reduce.is_some() && command != Command::Heatmap {
        return Err(Fail(format!(
            "--reduce only applies to heatmap, not `{}` (hist2d already aggregates its bins)",
            command.name()
        )));
    }
    if by.is_some() && command != Command::Scatter {
        return Err(Fail(format!(
            "--by only applies to scatter, not `{}`",
            command.name()
        )));
    }
    if live && (cols.is_some() || by.is_some() || emit_code) {
        return Err(Fail(
            "--cols, --by, and --emit-code do not apply to --live".into(),
        ));
    }
    // One resolved value for the chart builders: a bare --midpoint centers the
    // default map, a bare --log-color makes it logarithmic.
    let colormap = match (colormap, midpoint, log_color) {
        (map, Some(center), _) => Some(map.unwrap_or_default().centered_at(center)),
        (map, None, true) => Some(map.unwrap_or_default().log()),
        (map, None, false) => map,
    };

    Ok(Outcome::Run(Box::new(Args {
        command,
        input,
        output,
        passthrough,
        delimiter,
        header,
        fmt,
        width,
        height,
        title,
        xlabel,
        ylabel,
        xlim,
        ylim,
        log_x,
        log_y,
        time_x,
        bins,
        colormap,
        labels_x,
        labels_y,
        reduce,
        cols,
        by,
        emit_code,
        color,
        charset,
        pixels,
        quiet,
        live,
        window,
        fps,
        rate,
    })))
}

/// Parses a comma-separated label list for `--labels-x` / `--labels-y`.
fn parse_labels(flag: &str, value: &str) -> Result<Vec<String>, Fail> {
    let labels: Vec<String> = value
        .split(',')
        .map(|label| label.trim().to_owned())
        .collect();
    if labels.iter().any(String::is_empty) {
        return Err(Fail(format!(
            "{flag} wants comma-separated band labels, got `{value}`"
        )));
    }
    Ok(labels)
}

fn parse_bounded(flag: &str, value: &str, minimum: usize, maximum: usize) -> Result<usize, Fail> {
    let parsed = value
        .parse::<usize>()
        .map_err(|_| Fail(format!("{flag} needs a whole number, got `{value}`")))?;
    if !(minimum..=maximum).contains(&parsed) {
        return Err(Fail(format!(
            "{flag} must be between {minimum} and {maximum}, got {parsed}"
        )));
    }
    Ok(parsed)
}

/// Parses a `A,B` numeric pair for `--xlim` / `--ylim`.
fn parse_pair(flag: &str, value: &str) -> Result<(f64, f64), Fail> {
    let (a, b) = value
        .split_once(',')
        .ok_or_else(|| Fail(format!("{flag} takes two numbers as A,B, got `{value}`")))?;
    let parse = |part: &str| {
        part.trim()
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .ok_or_else(|| Fail(format!("{flag}: `{part}` is not a finite number")))
    };
    Ok((parse(a)?, parse(b)?))
}

#[cfg(test)]
#[path = "tests/args_tests.rs"]
mod tests;
