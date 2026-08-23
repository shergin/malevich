//! Subcommand → [`Plot`]. Zero rendering logic: every chart is a preset or the
//! exact public grammar composition the preset is proven equal to (D-C3).

use malevich::{Line, Plot, Points};

use crate::args::{Args, Command};
use crate::input::Table;
use crate::series::{self, Series};

/// A built plot plus the count of fields that would not parse.
pub struct Built {
    pub plot: Plot<'static>,
    pub unparsed: usize,
}

/// Builds the plot for `args` over `table`. With `categories` (`--by`), the
/// scatter takes them as its `color_by` channel.
pub fn build(args: &Args, table: &Table, categories: Option<&[String]>) -> malevich::Result<Built> {
    let (plot, unparsed) = match (args.command, categories) {
        (Command::Scatter, Some(groups)) => Ok(scatter_by(args, table, groups)),
        (command, _) => plain_build(command, args, table),
    }?;
    Ok(Built {
        plot: furniture(plot, args),
        unparsed,
    })
}

fn plain_build(
    command: Command,
    args: &Args,
    table: &Table,
) -> malevich::Result<(Plot<'static>, usize)> {
    match command {
        Command::Line => Ok(value_plot(args, table, Kind::Line)),
        Command::Scatter => Ok(value_plot(args, table, Kind::Scatter)),
        Command::Hist => hist_plot(table, args.bins),
        Command::Bar => Ok(bar_plot(table)),
        Command::Count => Ok(count_plot(table)),
        Command::Density => Ok(distribution(table, malevich::density)),
        Command::Ecdf => Ok(distribution(table, malevich::ecdf)),
        Command::Box => Ok(box_plot(table)),
        Command::Violin => Ok(violin_plot(table)),
        Command::Hist2d => Ok(hist2d_plot(args, table)),
        Command::Heatmap => Ok(heatmap_plot(args, table)),
    }
}

/// Scatter grouped by `--by`: the first two remaining columns as x and y, the
/// extracted column as the categorical color channel.
fn scatter_by(args: &Args, table: &Table, categories: &[String]) -> (Plot<'static>, usize) {
    let (x, y, unparsed) = series::xy(table, args.time_x);
    let groups: Vec<String> = categories.to_vec();
    let plot = Plot::new().layer(Points::xy(x, y).color_by(groups));
    (plot, unparsed)
}

#[derive(Clone, Copy)]
enum Kind {
    Line,
    Scatter,
}

/// Line and scatter: one layer per series, following `--fmt`.
fn value_plot(args: &Args, table: &Table, kind: Kind) -> (Plot<'static>, usize) {
    let fmt = series::resolve_fmt(table, args.fmt);
    let data = series::dataset(table, fmt, args.time_x);
    let mut plot = Plot::new();
    for series in data.series {
        plot = layer(plot, series, kind);
    }
    (plot, data.unparsed)
}

/// Adds one series as a line or a scatter layer, labeled when named.
fn layer(plot: Plot<'static>, series: Series, kind: Kind) -> Plot<'static> {
    let Series { x, y, label } = series;
    match (kind, x) {
        (Kind::Line, Some(x)) => plot.layer(named(Line::xy(x, y), label, Line::label)),
        (Kind::Line, None) => plot.layer(named(Line::y(y), label, Line::label)),
        (Kind::Scatter, Some(x)) => plot.layer(named(Points::xy(x, y), label, Points::label)),
        (Kind::Scatter, None) => plot.layer(named(Points::y(y), label, Points::label)),
    }
}

/// Applies a label to a mark when one is present, via the mark's own setter.
fn named<M>(mark: M, label: Option<String>, set: impl FnOnce(M, String) -> M) -> M {
    match label {
        Some(text) => set(mark, text),
        None => mark,
    }
}

/// Histogram: pool every numeric field, then bin. Auto by default; with `--bins N`
/// the exact documented expansion of `hist` — checked uniform bins +
/// `Bars::spans`.
fn hist_plot(table: &Table, bins: Option<usize>) -> malevich::Result<(Plot<'static>, usize)> {
    let (values, unparsed) = series::flatten(table);
    let plot = match bins {
        None => malevich::hist(values),
        Some(count) => binned(&values, count)?,
    };
    Ok((plot, unparsed))
}

/// The `--bins N` expansion: `count` equal-width bins over the finite data range,
/// counted into a `Bars::spans` layer — exactly what `hist` does, minus the
/// automatic bin-count choice.
fn binned(values: &[f64], count: usize) -> malevich::Result<Plot<'static>> {
    use malevich::Bars;
    use malevich::stat::Bins;

    let Some(histogram) = Bins::try_uniform(values, count)? else {
        return Ok(Plot::new());
    };
    let counts: Vec<f64> = histogram.counts().iter().map(|&c| c as f64).collect();
    Ok(Plot::new().layer(Bars::spans(histogram.start(), histogram.width(), counts)))
}

/// Density and ecdf: pool every numeric field, then the matching preset.
fn distribution(table: &Table, preset: fn(Vec<f64>) -> Plot<'static>) -> (Plot<'static>, usize) {
    let (values, unparsed) = series::flatten(table);
    (preset(values), unparsed)
}

/// Box plots: each column a group (header names, else positions).
fn box_plot(table: &Table) -> (Plot<'static>, usize) {
    let (categories, groups, unparsed) = series::groups(table);
    (malevich::box_plot(categories, groups), unparsed)
}

/// Violin plots: same column-as-group shape as box.
fn violin_plot(table: &Table) -> (Plot<'static>, usize) {
    let (categories, groups, unparsed) = series::groups(table);
    (malevich::violin(categories, groups), unparsed)
}

/// 2D histogram: the first two columns as x and y (x is time under `--time-x`).
fn hist2d_plot(args: &Args, table: &Table) -> (Plot<'static>, usize) {
    let (x, y, unparsed) = series::xy(table, args.time_x);
    let plot = match &args.colormap {
        Some(map) => {
            let options = malevich::Histogram2dOptions::default().colormap(map.clone());
            malevich::hist2d_with(x, y, options).expect("a parsed colormap is valid")
        }
        None => malevich::hist2d(x, y),
    };
    (plot, unparsed)
}

/// Heatmap: the rows as a row-major grid (first line on top).
fn heatmap_plot(args: &Args, table: &Table) -> (Plot<'static>, usize) {
    let (columns, values, unparsed) = series::matrix(table);
    if columns == 0 {
        return (Plot::new(), unparsed);
    }
    let plot = match &args.colormap {
        Some(map) => {
            let options = malevich::HeatmapOptions::new().colormap(map.clone());
            malevich::heatmap_with(columns, values, options).expect("a parsed colormap is valid")
        }
        None => malevich::heatmap(columns, values),
    };
    (plot, unparsed)
}

/// Bar: `label value` rows straight into the `bar` preset.
fn bar_plot(table: &Table) -> (Plot<'static>, usize) {
    let (labels, values, unparsed) = series::labeled_values(table);
    (malevich::bar(labels, values), unparsed)
}

/// Count: value frequencies (CLI-side) rendered as bars. Categories are never
/// "unparseable" — every string is a valid label — so the tally is zero.
fn count_plot(table: &Table) -> (Plot<'static>, usize) {
    let (labels, values): (Vec<String>, Vec<f64>) = series::counts(table).into_iter().unzip();
    (malevich::bar(labels, values), 0)
}

/// Applies the shared furniture flags: title, axis labels, limits, log scales.
fn furniture(mut plot: Plot<'static>, args: &Args) -> Plot<'static> {
    if let Some(title) = &args.title {
        plot = plot.title(title);
    }
    if let Some(xlabel) = &args.xlabel {
        plot = plot.x_label(xlabel);
    }
    if let Some(ylabel) = &args.ylabel {
        plot = plot.y_label(ylabel);
    }
    if let Some((lo, hi)) = args.xlim {
        plot = plot.x_domain(lo, hi);
    }
    if let Some((lo, hi)) = args.ylim {
        plot = plot.y_domain(lo, hi);
    }
    // Parsing rejects --time-x on charts without a time axis, so no gate here.
    if args.time_x {
        plot = plot.time_x();
    }
    if args.log_x {
        plot = plot.log_x();
    }
    if args.log_y {
        plot = plot.log_y();
    }
    plot
}
