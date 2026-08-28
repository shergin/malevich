//! The view layer: pure functions from catalog data to `malevich::Plot`s.
//!
//! Nothing here reads state or owns a terminal — every function takes data and
//! options in and returns owned plots, so views render identically in the TUI, in
//! the headless `--render` mode, and under test.
//!
//! This file doubles as a tour of malevich's model. The core idea: a `Plot` is a
//! *retained description* — layers of marks over shared scales plus furniture —
//! and rendering is a separate, pure step (`plot.render(&frame)` for strings,
//! `plot.widget()` for ratatui). Because plots are plain values, these functions
//! can build and return them with no knowledge of where they will be drawn or at
//! what size; the frame's dimensions only matter at render time.

use malevich::{Area, Cells, Color, Dash, Line, LineStyle, Plot, Points, Rule};

use crate::data::{Catalog, Kind, Series, align, extent, step_series};

/// The app's screens, in tab order.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// Every series at a glance — small multiples.
    Overview,
    /// One series large, with transforms and recession shading.
    Series,
    /// How the series' changes distribute: histogram plus decade box plots.
    Distribution,
    /// Period changes as a year × period heatmap.
    Seasonality,
    /// Cross-series classics: the Phillips curve and the yield spread.
    Relations,
}

impl View {
    pub const ALL: [View; 5] = [
        View::Overview,
        View::Series,
        View::Distribution,
        View::Seasonality,
        View::Relations,
    ];

    pub fn title(self) -> &'static str {
        match self {
            View::Overview => "overview",
            View::Series => "series",
            View::Distribution => "distribution",
            View::Seasonality => "seasonality",
            View::Relations => "relations",
        }
    }

    pub fn next(self) -> View {
        let index = Self::ALL.iter().position(|v| *v == self).unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }

    pub fn previous(self) -> View {
        let index = Self::ALL.iter().position(|v| *v == self).unwrap_or(0);
        Self::ALL[(index + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

/// What the series view draws: the level as reported, its change over one year, or
/// the level on a logarithmic axis.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Transform {
    Level,
    YearOverYear,
    Log,
}

impl Transform {
    pub fn next(self) -> Transform {
        match self {
            Transform::Level => Transform::YearOverYear,
            Transform::YearOverYear => Transform::Log,
            Transform::Log => Transform::Level,
        }
    }

    pub fn label(self, kind: Kind) -> &'static str {
        match (self, kind) {
            (Transform::Level, _) => "level",
            (Transform::YearOverYear, Kind::Index) => "year-over-year %",
            (Transform::YearOverYear, Kind::Rate) => "1-year change, points",
            (Transform::Log, _) => "level (log axis)",
        }
    }
}

/// One small-multiple line chart per series, for the overview grid.
///
/// malevich notes: `Line::xy` takes paired x/y columns — here unix-second dates
/// against values — and `.time_x()` declares the x axis a calendar axis, so ticks
/// land on real boundaries (Januaries, decades) with labels like `1980`, not raw
/// second counts. Missing observations are `NaN` in the data and render as visible
/// line breaks, never interpolated across. If a series were huge, the pipeline
/// would auto-reduce it (M4) to the raster with no visual change — chart building
/// stays size-oblivious.
pub fn overview_charts(catalog: &Catalog) -> Vec<Plot<'static>> {
    catalog
        .series
        .iter()
        .map(|series| {
            let latest = series
                .latest()
                .map(|v| format!("{v:.1}"))
                .unwrap_or_default();
            Plot::new()
                .layer(Line::xy(series.dates.clone(), series.values.clone()).color(Color::Cyan))
                .title(format!("{}  {latest}", series.id))
                .time_x()
        })
        .collect()
}

/// The main chart: one series under a transform, optionally with the NBER
/// recessions marked, in the chosen line style. The fed funds rate draws as steps —
/// an administered rate holds until the next decision, and the chart should say so.
///
/// malevich notes on the two kinds of "transform" here:
/// - Year-over-year is a *data* transform: we compute a new series and plot that.
/// - Log is a *scale* transform: the data stays untouched and `.log_y()` changes
///   the axis itself — decade ticks (`10¹`, `10²`), and non-positive values become
///   honest gaps rather than lies.
///
/// Layers draw in insertion order, which is why the recession ribbon is layered
/// before the data line: later layers draw over earlier ones.
pub fn series_chart(
    series: &Series,
    transform: Transform,
    style: LineStyle,
    recessions: Option<&[(f64, f64)]>,
) -> Plot<'static> {
    let (x, y) = match transform {
        Transform::Level | Transform::Log => (series.dates.clone(), series.values.clone()),
        Transform::YearOverYear => (series.dates.clone(), series.year_over_year()),
    };
    // Steps are plain data, not a special mark: doubling interior points makes the
    // polyline hold each value flat until the next observation (see `step_series`).
    let stepped = series.id == "FEDFUNDS" && transform != Transform::YearOverYear;
    let (x, y) = if stepped { step_series(&x, &y) } else { (x, y) };

    let mut plot = Plot::new()
        .title(format!("{} ({})", series.title, series.id))
        .time_x()
        .y_label(match transform {
            Transform::YearOverYear => match series.kind {
                Kind::Index => "percent",
                Kind::Rate => "points",
            },
            _ => series.unit,
        });

    // Recession shading is skipped on a log axis: the ribbon strip must extend
    // below the data, and a log axis cannot go to or past zero honestly.
    if transform != Transform::Log
        && let Some(recessions) = recessions
    {
        plot = recession_ribbon(plot, recessions, &x, &y);
    }

    // Inflation charts get the Fed's 2% target as a reference rule: `Rule::h` is a
    // horizontal annotation line spanning the plot at y = 2, and giving any layer a
    // `.label(...)` is what makes the legend appear. `.dash(...)` sets the stroke
    // pattern, so the target reads as annotation, never as data.
    if series.id == "CPIAUCSL" && transform == Transform::YearOverYear {
        plot = plot.layer(
            Rule::h(2.0)
                .label("2% target")
                .color(Color::Yellow)
                .dash(Dash::Dashed),
        );
    }

    // `.style(...)` switches how the same polyline rasterizes: subpixel braille
    // dots (`Pixels`, the default) or whole-cell `╭╮╰╯` elbows (`Corners`, the
    // classic asciichart look) — the data and scales are identical either way.
    // `.glow()` marks the primary series with a soft halo on pixel targets;
    // glyph rendering ignores it, so the cell view is unchanged.
    plot = plot.layer(Line::xy(x, y).style(style).color(Color::Cyan).glow());
    if transform == Transform::Log {
        plot = plot.log_y();
    }
    plot
}

/// Adds the NBER recessions as a ribbon in a strip reserved *below* the data — a
/// full-height band would fill every subpixel and swallow the line (terminals have
/// no translucency). Assumes a linear y axis.
///
/// malevich notes: two features compose here. `.y_domain(lo - strip, hi)` fixes
/// the axis wider than the data (matplotlib's `ylim`), carving out space the data
/// never enters; each recession is then an `Area::between(x, low, high)` — a
/// filled band between two edges — living only inside that carved strip. Marks
/// clip to the plot rectangle, so a period reaching past the visible range simply
/// clips honestly.
fn recession_ribbon(
    mut plot: Plot<'static>,
    recessions: &[(f64, f64)],
    x: &[f64],
    y: &[f64],
) -> Plot<'static> {
    let (lo, hi) = extent(y);
    if x.is_empty() || !lo.is_finite() || hi <= lo {
        return plot;
    }
    let strip = (hi - lo) * 0.06;
    plot = plot.y_domain(lo - strip, hi);
    let (first, last) = (x[0], *x.last().unwrap_or(&0.0));
    for &(start, end) in recessions {
        if end >= first && start <= last {
            plot = plot.layer(
                Area::between([start, end], [lo - strip, lo - strip], [lo, lo]).color(Color::Red),
            );
        }
    }
    plot
}

/// The distribution view: how the series' period changes distribute, and how its
/// level moved by decade. Returns `(histogram, decade box plots)`.
///
/// malevich notes: these are *presets* — one-line chart constructors that are pure
/// compositions of the same grammar (`hist` runs Sturges/Freedman–Diaconis binning
/// with nice decimal edges and draws `Bars`; `box_plot` computes type-7 quartiles
/// with Tukey whiskers and draws `Range` marks on a categorical axis, outliers as
/// dots). A preset returns an ordinary `Plot`, so `.title()`/`.x_label()` chain on
/// afterwards — presets are starting points, not dead ends.
pub fn distribution_charts(series: &Series) -> (Plot<'static>, Plot<'static>) {
    let changes = series.period_changes();
    let change_unit = match series.kind {
        Kind::Index => "% change per period",
        Kind::Rate => "points change per period",
    };
    let histogram = malevich::hist(changes)
        .title(format!("{}: distribution of changes", series.id))
        .x_label(change_unit);

    let (decades, groups) = series.by_decade();
    let boxes = malevich::box_plot(decades, groups)
        .title(format!("{}: level by decade", series.id))
        .y_label(series.unit);
    (histogram, boxes)
}

/// The seasonality view: period changes as a year × period heatmap (rows are
/// years, oldest at the bottom; columns run January → December), with a colorbar.
///
/// malevich notes: `Cells::matrix(columns, values)` takes a row-major grid with
/// row 0 at the *bottom*, so ascending years stack upward chronologically.
/// `.extents(x, y)` maps the grid from index space onto data coordinates — here
/// the y axis reads as real years. Values render as a shade ramp *and* a colormap
/// color (readable even with color stripped), `NaN` cells stay blank ("no data",
/// never "a little data"), and `.colorbar()` legends the value→color mapping in a
/// labeled strip on the right.
pub fn seasonality_chart(series: &Series, rows: usize) -> Plot<'static> {
    let (columns, grid, first_year, last_year) = series.seasonality(rows);
    let period = if columns == 4 {
        "quarter"
    } else {
        "month (Jan → Dec)"
    };
    let change = match series.kind {
        Kind::Index => "% change",
        Kind::Rate => "points change",
    };
    if grid.is_empty() {
        return Plot::new().title(format!("{}: no data", series.id));
    }
    Plot::new()
        .layer(Cells::matrix(columns, grid).extents(
            (0.0, columns as f64),
            (first_year as f64, last_year as f64 + 1.0),
        ))
        .colorbar()
        .title(format!("{}: {change} by {period} and year", series.id))
        .x_label(period)
}

/// The relations view: the Phillips curve (unemployment vs inflation, split at
/// 2000 to show the flattening) and the 10y − fed-funds yield spread whose
/// inversions precede recessions. Returns `(phillips, spread)`.
///
/// malevich notes: the Phillips chart shows multi-layer scatter — two `Points`
/// layers on shared scales, each `.label(...)`ed so the legend distinguishes the
/// eras, colors assigned from the theme palette in layer order. The spread chart
/// layers three things in draw order: the recession ribbon (bottom), a `Rule::h(0)`
/// marking inversion, and the spread line on top.
pub fn relations_charts(
    catalog: &Catalog,
    recessions: Option<&[(f64, f64)]>,
) -> (Plot<'static>, Plot<'static>) {
    let unrate = catalog.by_id("UNRATE").expect("vendored");
    let cpi = catalog.by_id("CPIAUCSL").expect("vendored");
    let gs10 = catalog.by_id("GS10").expect("vendored");
    let fedfunds = catalog.by_id("FEDFUNDS").expect("vendored");

    // Unemployment against CPI inflation on the same month, split into eras.
    let inflation = cpi.year_over_year();
    let cutoff = crate::data::parse_date("2000-01-01").expect("valid date");
    let mut phillips = Plot::new()
        .title("Phillips curve: unemployment vs inflation, monthly")
        .x_label("unemployment %")
        .y_label("CPI YoY %");
    let (dates, unemployment, inflation) =
        align(&unrate.dates, &unrate.values, &cpi.dates, &inflation);
    for (label, keep) in [
        ("1948-1999", true), // keep dates before the cutoff
        ("2000-now", false), // keep dates at or after it
    ] {
        let (u, i): (Vec<f64>, Vec<f64>) = dates
            .iter()
            .zip(unemployment.iter().zip(&inflation))
            .filter(|(date, _)| (**date < cutoff) == keep)
            .map(|(_, (&u, &i))| (u, i))
            .unzip();
        phillips = phillips.layer(Points::xy(u, i).label(label));
    }

    // The spread between the 10-year yield and the policy rate: inversions (below
    // zero) precede the shaded recessions — the classic predictor, in one chart.
    let (dates, ten_year, funds) =
        align(&gs10.dates, &gs10.values, &fedfunds.dates, &fedfunds.values);
    let spread: Vec<f64> = ten_year.iter().zip(&funds).map(|(a, b)| a - b).collect();
    let mut spread_plot = Plot::new()
        .title("Yield spread: 10-year minus fed funds")
        .time_x()
        .y_label("points");
    if let Some(recessions) = recessions {
        spread_plot = recession_ribbon(spread_plot, recessions, &dates, &spread);
    }
    spread_plot = spread_plot
        .layer(Rule::h(0.0).label("inversion").color(Color::Red))
        .layer(
            Line::xy(dates, spread)
                .label("10y - fed funds")
                .color(Color::Cyan),
        );
    (phillips, spread_plot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Catalog;

    #[test]
    fn the_recession_toggle_changes_the_series_chart() {
        let catalog = Catalog::load();
        let series = catalog.by_id("UNRATE").unwrap();
        let frame = malevich::Frame::plain(80, 20);
        let shaded = series_chart(
            series,
            Transform::Level,
            LineStyle::Pixels,
            Some(&catalog.recessions),
        )
        .render(&frame);
        let bare = series_chart(series, Transform::Level, LineStyle::Pixels, None).render(&frame);
        assert_ne!(shaded, bare, "toggling recessions must change the chart");
    }
}
