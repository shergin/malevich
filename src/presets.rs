//! Presets: chart types as plain functions over the grammar.
//!
//! Every preset is a composition of marks, scales, and furniture — nothing a preset
//! does is beyond reach of the grammar, and each returns the [`Plot`] for refinement.

use crate::data::IntoSeries;
use crate::mark::{Area, Bars, Cells, Line, Points, Range};
use crate::plot::Plot;
use crate::scale::Colormap;

/// Configuration for [`hist_with`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct HistogramOptions {
    /// Maximum number of automatically selected bins.
    pub max_bins: usize,
}

impl HistogramOptions {
    /// Uses at most `max_bins` automatically selected bins.
    pub const fn new(max_bins: usize) -> HistogramOptions {
        HistogramOptions { max_bins }
    }
}

impl Default for HistogramOptions {
    fn default() -> HistogramOptions {
        HistogramOptions { max_bins: 60 }
    }
}

/// Configuration for [`hist2d_with`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct Histogram2dOptions {
    /// Number of cells along the x axis.
    pub columns: usize,
    /// Number of cells along the y axis.
    pub rows: usize,
    /// Colors used for finite, non-empty cells.
    pub colormap: Colormap,
    /// Whether to reserve and draw the value colorbar.
    pub colorbar: bool,
}

impl Histogram2dOptions {
    /// Uses a `columns` by `rows` grid with the default colormap and colorbar.
    pub const fn new(columns: usize, rows: usize) -> Histogram2dOptions {
        Histogram2dOptions {
            columns,
            rows,
            colormap: Colormap::DEFAULT,
            colorbar: true,
        }
    }

    /// Replaces the default colormap.
    #[must_use]
    pub fn colormap(mut self, colormap: Colormap) -> Histogram2dOptions {
        self.colormap = colormap;
        self
    }

    /// Shows or suppresses the value colorbar.
    #[must_use]
    pub const fn colorbar(mut self, visible: bool) -> Histogram2dOptions {
        self.colorbar = visible;
        self
    }
}

impl Default for Histogram2dOptions {
    fn default() -> Histogram2dOptions {
        Histogram2dOptions::new(48, 32)
    }
}

/// Configuration for [`heatmap_with`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct HeatmapOptions {
    /// Colors used for finite cells.
    pub colormap: Colormap,
    /// Whether to reserve and draw the value colorbar.
    pub colorbar: bool,
}

impl HeatmapOptions {
    /// Uses the default colormap with a colorbar.
    pub const fn new() -> HeatmapOptions {
        HeatmapOptions {
            colormap: Colormap::DEFAULT,
            colorbar: true,
        }
    }

    /// Replaces the default colormap.
    #[must_use]
    pub fn colormap(mut self, colormap: Colormap) -> HeatmapOptions {
        self.colormap = colormap;
        self
    }

    /// Shows or suppresses the value colorbar.
    #[must_use]
    pub const fn colorbar(mut self, visible: bool) -> HeatmapOptions {
        self.colorbar = visible;
        self
    }
}

impl Default for HeatmapOptions {
    fn default() -> HeatmapOptions {
        HeatmapOptions::new()
    }
}

/// Configuration for [`trend_with`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct TrendOptions {
    /// Confidence-band half-width, as a multiplier on the standard error of
    /// the mean response; `None` draws no band. `1.96` approximates a 95%
    /// band for large samples.
    pub band: Option<f64>,
    /// Positions at which the band edges are evaluated (the edges curve —
    /// they flare away from the mean).
    pub band_samples: usize,
}

impl TrendOptions {
    /// No band, 64 band samples once one is requested.
    pub const fn new() -> TrendOptions {
        TrendOptions {
            band: None,
            band_samples: 64,
        }
    }

    /// Draws the confidence band at `multiplier` standard errors.
    #[must_use]
    pub const fn band(mut self, multiplier: f64) -> TrendOptions {
        self.band = Some(multiplier);
        self
    }
}

impl Default for TrendOptions {
    fn default() -> TrendOptions {
        TrendOptions::new()
    }
}

/// Configuration for [`ecdf_with`].
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub struct EcdfOptions {
    /// Simultaneous confidence level α for the
    /// Dvoretzky–Kiefer–Wolfowitz band; `None` draws no band. `0.05` gives
    /// the textbook 95% band.
    pub band_alpha: Option<f64>,
}

impl EcdfOptions {
    /// No band.
    pub const fn new() -> EcdfOptions {
        EcdfOptions { band_alpha: None }
    }

    /// Draws the DKW confidence band at level `alpha` (in `(0, 1)`).
    #[must_use]
    pub const fn band(mut self, alpha: f64) -> EcdfOptions {
        self.band_alpha = Some(alpha);
        self
    }
}

impl Default for EcdfOptions {
    fn default() -> EcdfOptions {
        EcdfOptions::new()
    }
}

/// Configuration for [`density_with`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct DensityOptions {
    /// Number of positions at which to evaluate the KDE.
    pub samples: usize,
}

impl DensityOptions {
    /// Evaluates the density at `samples` positions.
    pub const fn new(samples: usize) -> DensityOptions {
        DensityOptions { samples }
    }
}

impl Default for DensityOptions {
    fn default() -> DensityOptions {
        DensityOptions { samples: 256 }
    }
}

/// Configuration for [`violin_with`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct ViolinOptions {
    /// Number of KDE positions along each violin.
    pub samples: usize,
}

impl ViolinOptions {
    /// Evaluates each violin at `samples` positions.
    pub const fn new(samples: usize) -> ViolinOptions {
        ViolinOptions { samples }
    }
}

impl Default for ViolinOptions {
    fn default() -> ViolinOptions {
        ViolinOptions { samples: 128 }
    }
}

/// How [`contour_with`] chooses iso-line values.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum ContourLevels {
    /// Choose nice interior levels using the tick algorithm and this target count.
    Automatic(usize),
    /// Trace these exact values, sorted and deduplicated before use.
    Explicit(Vec<f64>),
}

/// Configuration for [`contour_with`].
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub struct ContourOptions {
    /// Automatic or explicit iso-line values.
    pub levels: ContourLevels,
    /// Colors interpolated over the grid's finite value extent.
    pub colormap: Colormap,
}

impl ContourOptions {
    /// Chooses nice levels near `target` using the tick algorithm.
    pub const fn automatic(target: usize) -> ContourOptions {
        ContourOptions {
            levels: ContourLevels::Automatic(target),
            colormap: Colormap::DEFAULT,
        }
    }

    /// Traces the supplied levels after sorting and deduplicating them.
    pub fn explicit(levels: impl IntoIterator<Item = f64>) -> ContourOptions {
        ContourOptions {
            levels: ContourLevels::Explicit(levels.into_iter().collect()),
            colormap: Colormap::DEFAULT,
        }
    }

    /// Replaces the default colormap.
    #[must_use]
    pub fn colormap(mut self, colormap: Colormap) -> ContourOptions {
        self.colormap = colormap;
        self
    }
}

impl Default for ContourOptions {
    fn default() -> ContourOptions {
        ContourOptions::automatic(7)
    }
}

fn check_count(
    count: usize,
    minimum: usize,
    what: &'static str,
    minimum_detail: &'static str,
) -> crate::Result<()> {
    if count < minimum {
        return Err(crate::Error::InvalidParameter {
            detail: minimum_detail,
        });
    }
    if count > crate::stat::MAX_STAT_ELEMENTS {
        return Err(crate::Error::DimensionTooLarge {
            what,
            requested: count,
            limit: crate::stat::MAX_STAT_ELEMENTS,
        });
    }
    Ok(())
}

fn check_colormap(colormap: &Colormap) -> crate::Result<()> {
    colormap.validate()
}

fn check_contour_coordinates(
    value_count: usize,
    columns: usize,
    level_count: usize,
) -> crate::Result<()> {
    let rows = value_count / columns;
    let estimated = columns
        .saturating_sub(1)
        .checked_mul(rows.saturating_sub(1))
        .and_then(|blocks| blocks.checked_mul(level_count))
        // Marching squares emits at most two three-entry segments per block.
        .and_then(|crossings| crossings.checked_mul(6))
        .unwrap_or(usize::MAX);
    if estimated > crate::stat::MAX_STAT_ELEMENTS {
        return Err(crate::Error::DimensionTooLarge {
            what: "contour coordinate count",
            requested: estimated,
            limit: crate::stat::MAX_STAT_ELEMENTS,
        });
    }
    Ok(())
}

/// A line chart of `values` plotted against their indices.
///
/// ```
/// let chart = malevich::line(&[1.0, 4.0, 2.0, 8.0][..]);
/// println!("{}", chart.render(&malevich::Frame::plain(40, 10)));
/// ```
pub fn line<'a>(values: impl IntoSeries<'a>) -> Plot<'a> {
    Plot::new().layer(Line::y(values))
}

/// A scatter chart of the points `(x[i], y[i])`.
///
/// ```
/// let chart = malevich::scatter(&[1.0, 2.0, 3.0][..], &[2.0, 1.0, 3.0][..]);
/// println!("{}", chart.render(&malevich::Frame::plain(40, 10)));
/// ```
pub fn scatter<'a>(x: impl IntoSeries<'a>, y: impl IntoSeries<'a>) -> Plot<'a> {
    Plot::new().layer(Points::xy(x, y))
}

/// A bar chart: one labeled bar per category, rising from zero.
///
/// ```
/// let chart = malevich::bar(["a", "b", "c"], &[3.0, 7.0, 5.0][..]);
/// println!("{}", chart.render(&malevich::Frame::plain(40, 10)));
/// ```
pub fn bar<'a>(
    categories: impl IntoIterator<Item = impl Into<String>>,
    values: impl IntoSeries<'a>,
) -> Plot<'a> {
    Plot::new().layer(Bars::new(categories, values))
}

/// A histogram: `values` binned automatically (Sturges/Freedman–Diaconis, nice
/// decimal edges) and drawn as contiguous bars from zero.
///
/// ```
/// let samples = [1.0, 2.0, 2.5, 2.7, 3.0, 3.1, 3.2, 4.0, 5.5];
/// println!("{}", malevich::hist(&samples[..]).render(&malevich::Frame::plain(40, 10)));
/// ```
pub fn hist<'a>(values: impl IntoSeries<'a>) -> Plot<'a> {
    hist_with(values, HistogramOptions::default()).expect("default histogram options are valid")
}

/// A histogram with a caller-selected automatic bin cap.
///
/// # Errors
///
/// Returns an error when `options.max_bins` is zero, exceeds the defensive
/// statistics limit, or cannot represent the complete finite data span.
pub fn hist_with<'a>(
    values: impl IntoSeries<'a>,
    options: HistogramOptions,
) -> crate::Result<Plot<'a>> {
    check_count(
        options.max_bins,
        1,
        "histogram bin cap",
        "histogram max_bins must be at least one",
    )?;
    let series = values.into_series();
    Ok(
        match crate::stat::Bins::try_auto(series.as_slice(), options.max_bins)? {
            Some(bins) => {
                let counts: Vec<f64> = bins.counts().iter().map(|&count| count as f64).collect();
                Plot::new().layer(Bars::spans(bins.start(), bins.width(), counts))
            }
            None => Plot::new(),
        },
    )
}

/// A step chart: `values` held flat between indices — counters, rates, states.
///
/// ```
/// println!("{}", malevich::stairs(&[1.0, 3.0, 2.0][..]).render(&malevich::Frame::plain(40, 8)));
/// ```
pub fn stairs<'a>(values: impl IntoSeries<'a>) -> Plot<'a> {
    let series = values.into_series();
    let mut x = Vec::with_capacity(series.len() * 2);
    let mut y = Vec::with_capacity(series.len() * 2);
    for (index, value) in series.iter().enumerate() {
        if index > 0 {
            x.push(index as f64);
            y.push(y.last().copied().unwrap_or(value));
        }
        x.push(index as f64);
        y.push(value);
    }
    Plot::new().layer(Line::xy(x, y))
}

/// An ECDF chart: the fraction of `values` at or below each value, as a step line
/// from 0 to 1.
///
/// ```
/// let samples = [3.0, 1.0, 4.0, 1.0, 5.0];
/// println!("{}", malevich::ecdf(&samples[..]).render(&malevich::Frame::plain(40, 8)));
/// ```
pub fn ecdf<'a>(values: impl IntoSeries<'a>) -> Plot<'a> {
    ecdf_with(values, EcdfOptions::default()).expect("default ecdf options are valid")
}

/// An empirical CDF with an optional Dvoretzky–Kiefer–Wolfowitz confidence
/// band: the finite-sample envelope `F̂ ± √(ln(2/α)/2n)`, clipped to `[0, 1]`,
/// stepped exactly like the curve and drawn through the existing band mark.
///
/// # Errors
///
/// Returns an error when the band level is outside `(0, 1)`.
pub fn ecdf_with<'a>(values: impl IntoSeries<'a>, options: EcdfOptions) -> crate::Result<Plot<'a>> {
    if let Some(alpha) = options.band_alpha
        && !(alpha > 0.0 && alpha < 1.0)
    {
        return Err(crate::Error::InvalidParameter {
            detail: "an ecdf band level must be strictly between 0 and 1",
        });
    }
    let series = values.into_series();
    let (sorted, fractions) = crate::stat::ecdf(series.as_slice());
    let count = sorted.len();
    let mut x = Vec::with_capacity(count * 2);
    let mut y = Vec::with_capacity(count * 2);
    let mut previous = 0.0f64;
    for (value, fraction) in sorted.into_iter().zip(fractions) {
        x.push(value);
        y.push(previous);
        x.push(value);
        y.push(fraction);
        previous = fraction;
    }
    let mut plot = Plot::new();
    if let (Some(alpha), true) = (options.band_alpha, count > 0) {
        let epsilon = ((2.0 / alpha).ln() / (2.0 * count as f64)).sqrt();
        let low: Vec<f64> = y.iter().map(|f| (f - epsilon).max(0.0)).collect();
        let high: Vec<f64> = y.iter().map(|f| (f + epsilon).min(1.0)).collect();
        plot = plot.layer(Area::between(x.clone(), low, high));
    }
    Ok(plot.layer(Line::xy(x, y)))
}

/// A heatmap of a row-major grid, `columns` wide: two vertical color samples per
/// cell (an averaged shade-ramp glyph in plain output), with a
/// [colorbar](crate::Plot::colorbar) legending the value range. Row 0 is the bottom
/// row.
///
/// ```
/// let grid = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0];
/// println!("{}", malevich::heatmap(3, &grid[..]).render(&malevich::Frame::plain(30, 8)));
/// ```
///
/// # Panics
///
/// Panics if `columns` is zero or does not divide the value count evenly. Use
/// [`heatmap_with`] for a checked boundary.
pub fn heatmap<'a>(columns: usize, values: impl IntoSeries<'a>) -> Plot<'a> {
    heatmap_with(columns, values, HeatmapOptions::default())
        .expect("default heatmap options are valid")
}

/// A heatmap with a caller-selected color presentation — a named or custom
/// [`Colormap`], optionally [centered](Colormap::centered_at) for signed data.
///
/// ```
/// use malevich::scale::Colormap;
/// let correlations = [1.0, -0.4, -0.4, 1.0];
/// let options = malevich::HeatmapOptions::new().colormap(Colormap::RED_BLUE.centered_at(0.0));
/// let chart = malevich::heatmap_with(2, &correlations[..], options).unwrap();
/// ```
///
/// # Errors
///
/// Returns an error for a zero column count, a non-rectangular value grid, or an
/// invalid colormap.
pub fn heatmap_with<'a>(
    columns: usize,
    values: impl IntoSeries<'a>,
    options: HeatmapOptions,
) -> crate::Result<Plot<'a>> {
    check_colormap(&options.colormap)?;
    let plot = Plot::new().layer(Cells::try_matrix(columns, values)?.colormap(options.colormap));
    Ok(if options.colorbar {
        plot.colorbar()
    } else {
        plot
    })
}

/// A 2D histogram: point density on a uniform grid over the data's extent, with a
/// [colorbar](crate::Plot::colorbar) legending the counts.
///
/// ```
/// let x = [1.0, 1.1, 5.0, 5.1, 5.2];
/// let y = [2.0, 2.1, 8.0, 8.1, 7.9];
/// println!("{}", malevich::hist2d(&x[..], &y[..]).render(&malevich::Frame::plain(40, 12)));
/// ```
pub fn hist2d<'a>(x: impl IntoSeries<'a>, y: impl IntoSeries<'a>) -> Plot<'a> {
    hist2d_with(x, y, Histogram2dOptions::default())
        .expect("default 2D histogram options and equal channels are required")
}

/// A 2D histogram with caller-selected grid geometry and color presentation.
///
/// # Errors
///
/// Returns an error for unequal channels, an empty or oversized grid, or an
/// invalid colormap.
pub fn hist2d_with<'a>(
    x: impl IntoSeries<'a>,
    y: impl IntoSeries<'a>,
    options: Histogram2dOptions,
) -> crate::Result<Plot<'a>> {
    check_colormap(&options.colormap)?;
    let xs = x.into_series();
    let ys = y.into_series();
    Ok(
        match crate::stat::try_bins2(xs.as_slice(), ys.as_slice(), options.columns, options.rows)? {
            Some(grid) => {
                // Empty bins are gaps, not the faintest shade — blank space must mean
                // "no data", never "a little data".
                let counts: Vec<f64> = grid
                    .counts
                    .into_iter()
                    .map(|count| if count == 0.0 { f64::NAN } else { count })
                    .collect();
                let cells = Cells::try_matrix(grid.columns, counts)?
                    .try_extents(grid.x, grid.y)?
                    .colormap(options.colormap);
                let plot = Plot::new().layer(cells);
                if options.colorbar {
                    plot.colorbar()
                } else {
                    plot
                }
            }
            None => Plot::new(),
        },
    )
}

/// Contour lines of a row-major grid (row 0 at the bottom), like
/// [`heatmap`](crate::heatmap) but tracing iso-lines instead of shading.
///
/// Levels are chosen by the tick algorithm — nice decimals inside the data's
/// range — each traced by marching squares, colored along the default colormap,
/// and labeled with its value in the legend.
///
/// ```
/// let values: Vec<f64> = (0..64).map(|i| ((i % 8) * (i / 8)) as f64).collect();
/// println!("{}", malevich::contour(8, &values[..]).render(&malevich::Frame::plain(40, 12)));
/// ```
///
/// # Panics
///
/// Panics if `columns` is zero, does not divide the number of values, or the
/// default level set could exceed the defensive contour output budget. Use
/// [`contour_with`] for a checked boundary.
pub fn contour<'a>(columns: usize, values: impl IntoSeries<'a>) -> Plot<'a> {
    contour_with(columns, values, ContourOptions::default())
        .expect("contour requires a rectangular grid and valid default options")
}

/// Contour lines with caller-selected levels and colormap.
///
/// Explicit levels outside the grid's finite range are omitted because they
/// cannot cross a cell. Automatic levels use the same nice-decimal tick engine as
/// the axes.
///
/// # Errors
///
/// Returns an error for a non-rectangular grid, fewer than two requested
/// automatic ticks, empty/non-finite explicit levels, an oversized level set, or
/// an invalid colormap.
pub fn contour_with<'a>(
    columns: usize,
    values: impl IntoSeries<'a>,
    options: ContourOptions,
) -> crate::Result<Plot<'a>> {
    use crate::scale::Ticks;

    let series = values.into_series();
    if columns == 0 {
        return Err(crate::Error::EmptyDimension {
            what: "contour columns",
        });
    }
    if !series.len().is_multiple_of(columns) {
        return Err(crate::Error::NonRectangular {
            mark: "contour",
            shape: (series.len(), columns),
        });
    }
    check_colormap(&options.colormap)?;
    let level_selection = match options.levels {
        ContourLevels::Automatic(target) => {
            check_count(
                target,
                2,
                "contour automatic level target",
                "contour automatic target must be at least two",
            )?;
            check_contour_coordinates(series.len(), columns, target)?;
            ContourLevels::Automatic(target)
        }
        ContourLevels::Explicit(mut levels) => {
            check_count(
                levels.len(),
                1,
                "contour explicit level count",
                "contour explicit levels must not be empty",
            )?;
            check_contour_coordinates(series.len(), columns, levels.len())?;
            if levels.iter().any(|level| !level.is_finite()) {
                return Err(crate::Error::InvalidParameter {
                    detail: "contour explicit levels must be finite",
                });
            }
            levels.sort_by(f64::total_cmp);
            levels.dedup();
            ContourLevels::Explicit(levels)
        }
    };
    let mut extent: Option<(f64, f64)> = None;
    for &value in series.as_slice() {
        if value.is_finite() {
            let (low, high) = extent.get_or_insert((value, value));
            *low = low.min(value);
            *high = high.max(value);
        }
    }
    let Some((min, max)) = extent.filter(|(low, high)| low < high) else {
        return Ok(Plot::new());
    };
    let levels: Vec<(f64, String)> = match level_selection {
        ContourLevels::Automatic(target) => Ticks::linear(min, max, target)
            .iter()
            .filter(|tick| tick.value > min && tick.value < max)
            .map(|tick| (tick.value, tick.label.clone()))
            .collect(),
        ContourLevels::Explicit(levels) => levels
            .into_iter()
            .filter(|level| *level > min && *level < max)
            .map(|level| (level, level.to_string()))
            .collect(),
    };
    let values: Vec<f64> = levels.iter().map(|(level, _)| *level).collect();
    check_contour_coordinates(series.len(), columns, values.len())?;
    let mut plot = Plot::new();
    for ((level, label), line) in
        levels
            .into_iter()
            .zip(crate::stat::contours(series.as_slice(), columns, &values))
    {
        plot = plot.layer(
            Line::xy(line.x, line.y).label(label).color(
                options
                    .colormap
                    .color(options.colormap.position_in(level, min, max)),
            ),
        );
    }
    Ok(plot)
}

/// A vector field: one arrow per point, from `(x[i], y[i])` along `(u[i], v[i])`.
///
/// Arrows are drawn in data coordinates — a shaft to the displaced tip with two
/// barbs — so they scale with the axes like any other mark. Points with a
/// non-finite component are skipped.
///
/// ```
/// let (x, y) = ([0.0, 1.0, 2.0], [0.0, 0.5, 0.2]);
/// let (u, v) = ([0.4, 0.3, -0.2], [0.2, -0.3, 0.4]);
/// let chart = malevich::quiver(&x[..], &y[..], &u[..], &v[..]);
/// println!("{}", chart.render(&malevich::Frame::plain(40, 12)));
/// ```
///
/// # Panics
///
/// Panics if the four series have different lengths.
pub fn quiver<'a>(
    x: impl IntoSeries<'a>,
    y: impl IntoSeries<'a>,
    u: impl IntoSeries<'a>,
    v: impl IntoSeries<'a>,
) -> Plot<'a> {
    let (x, y) = (x.into_series(), y.into_series());
    let (u, v) = (u.into_series(), v.into_series());
    assert!(
        x.len() == y.len() && y.len() == u.len() && u.len() == v.len(),
        "quiver requires series of equal length"
    );
    let mut xs = Vec::with_capacity(x.len() * 9);
    let mut ys = Vec::with_capacity(x.len() * 9);
    let mut segment = |from: (f64, f64), to: (f64, f64)| {
        xs.extend([from.0, to.0, f64::NAN]);
        ys.extend([from.1, to.1, f64::NAN]);
    };
    for index in 0..x.len() {
        let (x, y) = (x.as_slice()[index], y.as_slice()[index]);
        let (u, v) = (u.as_slice()[index], v.as_slice()[index]);
        if !(x.is_finite() && y.is_finite() && u.is_finite() && v.is_finite()) {
            continue;
        }
        let tip = (x + u, y + v);
        segment((x, y), tip);
        // Barbs point back from the tip, 30° off the shaft, 30% of its length.
        let angle = v.atan2(u);
        let reach = 0.3 * u.hypot(v);
        for barb in [-1.0, 1.0] {
            let theta = angle + barb * (std::f64::consts::PI * 5.0 / 6.0);
            segment(
                tip,
                (tip.0 + reach * theta.cos(), tip.1 + reach * theta.sin()),
            );
        }
    }
    Plot::new().layer(Line::xy(xs, ys))
}

/// Box plots: one five-number box per category (type-7 quartiles, Tukey whiskers),
/// with outliers as dots.
///
/// ```
/// let a = [1.0, 2.0, 3.0, 4.0, 9.0];
/// let b = [2.0, 4.0, 5.0, 6.0, 7.0];
/// let chart = malevich::box_plot(["a", "b"], [&a[..], &b[..]]);
/// println!("{}", chart.render(&malevich::Frame::plain(40, 12)));
/// ```
///
/// # Panics
///
/// Panics if the number of categories differs from the number of groups.
pub fn box_plot<'a>(
    categories: impl IntoIterator<Item = impl Into<String>>,
    groups: impl IntoIterator<Item = impl IntoSeries<'a>>,
) -> Plot<'a> {
    let categories: Vec<String> = categories.into_iter().map(Into::into).collect();
    let stats: Vec<Option<crate::stat::BoxStats>> = groups
        .into_iter()
        .map(|group| crate::stat::BoxStats::of(group.into_series().as_slice()))
        .collect();
    assert_eq!(
        categories.len(),
        stats.len(),
        "box_plot requires one category per group"
    );
    let pick = |f: &dyn Fn(&crate::stat::BoxStats) -> f64| -> Vec<f64> {
        stats
            .iter()
            .map(|s| s.as_ref().map_or(f64::NAN, f))
            .collect()
    };
    let mut outlier_x = Vec::new();
    let mut outlier_y = Vec::new();
    for (index, stat) in stats.iter().enumerate() {
        if let Some(stat) = stat {
            for &outlier in &stat.outliers {
                outlier_x.push(index as f64);
                outlier_y.push(outlier);
            }
        }
    }
    let plot = Plot::new().layer(
        Range::over(
            categories,
            pick(&|s| s.whisker_low),
            pick(&|s| s.whisker_high),
        )
        .body(pick(&|s| s.q1), pick(&|s| s.q3))
        .marker(pick(&|s| s.median)),
    );
    if outlier_x.is_empty() {
        plot
    } else {
        plot.layer(Points::xy(outlier_x, outlier_y))
    }
}

/// Error bars: points with symmetric `error` intervals around each `y`.
///
/// ```
/// let x = [1.0, 2.0, 3.0];
/// let y = [4.0, 6.0, 5.0];
/// let e = [0.5, 1.0, 0.4];
/// println!("{}", malevich::error_bars(&x[..], &y[..], &e[..]).render(&malevich::Frame::plain(40, 10)));
/// ```
///
/// # Panics
///
/// Panics if the series have different lengths.
pub fn error_bars<'a>(
    x: impl IntoSeries<'a>,
    y: impl IntoSeries<'a>,
    error: impl IntoSeries<'a>,
) -> Plot<'a> {
    let x = x.into_series();
    let y = y.into_series();
    let error = error.into_series();
    assert!(
        x.len() == y.len() && y.len() == error.len(),
        "error_bars requires series of equal length"
    );
    let low: Vec<f64> = y.iter().zip(error.iter()).map(|(y, e)| y - e).collect();
    let high: Vec<f64> = y.iter().zip(error.iter()).map(|(y, e)| y + e).collect();
    let xs = x.as_slice().to_vec();
    Plot::new()
        .layer(Range::xy(xs.clone(), low, high))
        .layer(Points::xy(xs, y.as_slice().to_vec()))
}

/// Error bars with asymmetric intervals: each point reaches down by
/// `minus[i]` and up by `plus[i]` — the two-sided deviations of matplotlib's
/// 2×N `yerr`. For absolute interval bounds, use
/// [`Range::xy`](crate::Range::xy) directly.
///
/// ```
/// let x = [1.0, 2.0, 3.0];
/// let y = [4.0, 6.0, 5.0];
/// let minus = [0.5, 1.0, 0.4];
/// let plus = [1.5, 0.3, 0.9];
/// let chart = malevich::error_bars_asymmetric(&x[..], &y[..], &minus[..], &plus[..]);
/// println!("{}", chart.render(&malevich::Frame::plain(40, 10)));
/// ```
///
/// # Panics
///
/// Panics if the series have different lengths.
pub fn error_bars_asymmetric<'a>(
    x: impl IntoSeries<'a>,
    y: impl IntoSeries<'a>,
    minus: impl IntoSeries<'a>,
    plus: impl IntoSeries<'a>,
) -> Plot<'a> {
    let x = x.into_series();
    let y = y.into_series();
    let minus = minus.into_series();
    let plus = plus.into_series();
    assert!(
        x.len() == y.len() && y.len() == minus.len() && minus.len() == plus.len(),
        "error_bars_asymmetric requires series of equal length"
    );
    let low: Vec<f64> = y.iter().zip(minus.iter()).map(|(y, e)| y - e).collect();
    let high: Vec<f64> = y.iter().zip(plus.iter()).map(|(y, e)| y + e).collect();
    let xs = x.as_slice().to_vec();
    Plot::new()
        .layer(Range::xy(xs.clone(), low, high))
        .layer(Points::xy(xs, y.as_slice().to_vec()))
}

/// A scatter with its least-squares trend line ([`Fit`](crate::stat::Fit):
/// slope, intercept, and R² are one call away on the same accumulator).
/// Degenerate data (fewer than two distinct x) draws the points alone.
///
/// ```
/// let x = [1.0, 2.0, 3.0, 4.0, 5.0];
/// let y = [1.2, 1.9, 3.2, 3.8, 5.1];
/// println!("{}", malevich::trend(&x[..], &y[..]).render(&malevich::Frame::plain(40, 10)));
/// ```
///
/// # Panics
///
/// Panics if the two series have different lengths.
pub fn trend<'a>(x: impl IntoSeries<'a>, y: impl IntoSeries<'a>) -> Plot<'a> {
    trend_with(x, y, TrendOptions::default()).expect("default trend options are valid")
}

/// A scatter with its trend line and an optional confidence band around the
/// mean response, drawn through the existing band mark
/// ([`Area::between`](crate::Area::between)).
///
/// # Errors
///
/// Returns an error when the two series have different lengths, the band
/// multiplier is not finite and positive, or the band sample count is out of
/// range.
pub fn trend_with<'a>(
    x: impl IntoSeries<'a>,
    y: impl IntoSeries<'a>,
    options: TrendOptions,
) -> crate::Result<Plot<'a>> {
    if let Some(multiplier) = options.band {
        if !multiplier.is_finite() || multiplier <= 0.0 {
            return Err(crate::Error::InvalidParameter {
                detail: "a trend band multiplier must be finite and positive",
            });
        }
        check_count(
            options.band_samples,
            2,
            "trend band samples",
            "a trend band needs at least two samples",
        )?;
    }
    let x = x.into_series();
    let y = y.into_series();
    crate::mark::pair("trend: x and y", x.len(), y.len())?;
    let fit = crate::stat::Fit::xy(x.as_slice(), y.as_slice());
    let extent = x.iter().filter(|value| value.is_finite()).fold(
        None,
        |extent: Option<(f64, f64)>, value| {
            Some(match extent {
                Some((low, high)) => (low.min(value), high.max(value)),
                None => (value, value),
            })
        },
    );
    let mut plot = Plot::new();
    if let (Some((x0, x1)), Some(_)) = (extent, fit.slope()) {
        if let (Some(multiplier), true) = (options.band, x1 > x0) {
            let samples = options.band_samples;
            let positions: Vec<f64> = (0..samples)
                .map(|index| crate::numeric::lerp(x0, x1, index as f64 / (samples - 1) as f64))
                .collect();
            let band: Option<(Vec<f64>, Vec<f64>)> = positions
                .iter()
                .map(|&at| {
                    let center = fit.predict(at)?;
                    let error = fit.standard_error(at)?;
                    Some((center - multiplier * error, center + multiplier * error))
                })
                .collect();
            if let Some((low, high)) = band {
                plot = plot.layer(Area::between(positions, low, high));
            }
        }
        let fitted = [fit.predict(x0), fit.predict(x1)];
        if let [Some(y0), Some(y1)] = fitted {
            plot = plot.layer(Line::xy(vec![x0, x1], vec![y0, y1]));
        }
    }
    Ok(plot.layer(Points::xy(x.as_slice().to_vec(), y.as_slice().to_vec())))
}

/// A density chart: the Gaussian KDE of `values` as a smooth line.
///
/// ```
/// let samples = [1.0, 2.0, 2.5, 2.7, 3.0, 3.2, 4.0];
/// println!("{}", malevich::density(&samples[..]).render(&malevich::Frame::plain(40, 10)));
/// ```
pub fn density<'a>(values: impl IntoSeries<'a>) -> Plot<'a> {
    density_with(values, DensityOptions::default()).expect("default density options are valid")
}

/// A Gaussian KDE evaluated at a caller-selected number of positions.
///
/// # Errors
///
/// Returns an error when `options.samples` is below two or exceeds the
/// defensive statistics limit.
pub fn density_with<'a>(
    values: impl IntoSeries<'a>,
    options: DensityOptions,
) -> crate::Result<Plot<'a>> {
    check_count(
        options.samples,
        2,
        "density sample count",
        "density samples must be at least two",
    )?;
    let series = values.into_series();
    Ok(match crate::stat::kde(series.as_slice(), options.samples) {
        Some((positions, densities)) => Plot::new().layer(Line::xy(positions, densities)),
        None => Plot::new(),
    })
}

/// Violin plots: one mirrored density per category, each scaled to the same width.
///
/// ```
/// let a = [1.0, 2.0, 2.5, 3.0, 3.5];
/// let b = [4.0, 5.0, 5.5, 6.0, 8.0];
/// let chart = malevich::violin(["a", "b"], [&a[..], &b[..]]);
/// println!("{}", chart.render(&malevich::Frame::plain(44, 12)));
/// ```
///
/// # Panics
///
/// Panics if the number of categories differs from the number of groups or the
/// total default KDE output exceeds the defensive statistics limit. Use
/// [`violin_with`] for a checked boundary.
pub fn violin<'a>(
    categories: impl IntoIterator<Item = impl Into<String>>,
    groups: impl IntoIterator<Item = impl IntoSeries<'a>>,
) -> Plot<'a> {
    violin_with(categories, groups, ViolinOptions::default())
        .expect("violin requires one category per group and valid default options")
}

/// Violin plots with a caller-selected KDE sample count.
///
/// # Errors
///
/// Returns an error for fewer than two samples, excessive total KDE output, or a
/// category/group length mismatch.
pub fn violin_with<'a>(
    categories: impl IntoIterator<Item = impl Into<String>>,
    groups: impl IntoIterator<Item = impl IntoSeries<'a>>,
    options: ViolinOptions,
) -> crate::Result<Plot<'a>> {
    check_count(
        options.samples,
        2,
        "violin sample count",
        "violin samples must be at least two",
    )?;
    let categories: Vec<String> = categories.into_iter().map(Into::into).collect();
    let groups: Vec<_> = groups
        .into_iter()
        .map(|group| group.into_series())
        .collect();
    if categories.len() != groups.len() {
        return Err(crate::Error::UnequalChannels {
            mark: "violin: categories and groups",
            lengths: (categories.len(), groups.len()),
        });
    }
    let requested = groups.len().saturating_mul(options.samples);
    if requested > crate::stat::MAX_STAT_ELEMENTS {
        return Err(crate::Error::DimensionTooLarge {
            what: "violin KDE sample count",
            requested,
            limit: crate::stat::MAX_STAT_ELEMENTS,
        });
    }
    let densities: Vec<Option<(Vec<f64>, Vec<f64>)>> = groups
        .iter()
        .map(|group| crate::stat::kde(group.as_slice(), options.samples))
        .collect();
    // The Bands spec declares the categorical axis; the violins themselves are
    // horizontal areas over the band centers.
    let mut plot = Plot::new().x_scale(crate::scale::Scale::bands(categories));
    for (index, density) in densities.into_iter().enumerate() {
        let Some((positions, values)) = density else {
            continue;
        };
        let peak = values.iter().copied().fold(f64::MIN_POSITIVE, f64::max);
        let center = index as f64;
        let half: Vec<f64> = values.iter().map(|v| v / peak * 0.35).collect();
        let left: Vec<f64> = half.iter().map(|w| center - w).collect();
        let right: Vec<f64> = half.iter().map(|w| center + w).collect();
        plot = plot.layer(Area::horizontal(positions, left, right));
    }
    Ok(plot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Color, Frame};

    #[test]
    fn a_custom_histogram_cap_matches_the_grammar() {
        let values: Vec<f64> = (0..200).map(|index| (index % 37) as f64).collect();
        let frame = Frame::plain(44, 10);
        let actual = hist_with(&values[..], HistogramOptions::new(4))
            .unwrap()
            .render(&frame);

        let bins = crate::stat::Bins::auto(&values, 4).unwrap();
        let counts: Vec<f64> = bins.counts().iter().map(|&count| count as f64).collect();
        let expected = Plot::new()
            .layer(Bars::spans(bins.start(), bins.width(), counts))
            .render(&frame);
        assert_eq!(actual, expected);
    }

    #[test]
    fn histograms_handle_the_full_finite_range_without_panicking() {
        let plot = hist([-f64::MAX, f64::MAX]);
        assert!(plot.validate().is_ok());
        let _ = plot.render(&Frame::plain(40, 10));

        assert!(matches!(
            hist_with([-f64::MAX, f64::MAX], HistogramOptions::new(1)),
            Err(crate::Error::InvalidParameter { .. })
        ));
    }

    #[test]
    fn configured_grids_and_kdes_reach_the_generated_marks() {
        let x = [0.0, 1.0, 2.0, 3.0];
        let y = [0.0, 1.0, 0.5, 1.5];
        let histogram = hist2d_with(
            &x[..],
            &y[..],
            Histogram2dOptions::new(3, 2).colorbar(false),
        )
        .unwrap();
        let debug = format!("{histogram:?}");
        assert!(debug.contains("columns: 3"), "{debug}");
        assert!(debug.contains("rows: 2"), "{debug}");

        let density = density_with(&y[..], DensityOptions::new(24)).unwrap();
        assert!(format!("{density:?}").contains("points: 24"));

        let violin = violin_with(["one"], [&y[..]], ViolinOptions::new(20)).unwrap();
        assert!(format!("{violin:?}").contains("points: 20"));
    }

    #[test]
    fn explicit_contours_are_sorted_deduplicated_and_colored() {
        let grid: Vec<f64> = (0..9).map(f64::from).collect();
        let grayscale = Colormap::try_from_stops(vec![(0, 0, 0), (255, 255, 255)]).unwrap();
        let plot = contour_with(
            3,
            &grid[..],
            ContourOptions::explicit([6.0, 2.0, 6.0]).colormap(grayscale),
        )
        .unwrap();
        let debug = format!("{plot:?}");
        assert_eq!(debug.matches("Line {").count(), 2, "{debug}");
        assert!(debug.contains(&format!("{:?}", Color::Rgb(63, 63, 63))));
        assert!(debug.contains(&format!("{:?}", Color::Rgb(191, 191, 191))));
    }

    #[test]
    fn invalid_preset_options_return_typed_errors() {
        assert!(matches!(
            hist_with([1.0], HistogramOptions::new(0)),
            Err(crate::Error::InvalidParameter { .. })
        ));
        assert!(matches!(
            hist2d_with([1.0], [1.0], Histogram2dOptions::new(0, 2)),
            Err(crate::Error::EmptyDimension { .. })
        ));
        assert!(matches!(
            heatmap_with(0, [1.0], HeatmapOptions::new()),
            Err(crate::Error::EmptyDimension { .. })
        ));
        assert!(matches!(
            heatmap_with(2, [1.0, 2.0, 3.0], HeatmapOptions::new()),
            Err(crate::Error::NonRectangular { .. })
        ));
        assert!(matches!(
            trend_with([1.0], [1.0, 2.0], TrendOptions::new()),
            Err(crate::Error::UnequalChannels { .. })
        ));
        assert!(matches!(
            density_with([1.0], DensityOptions::new(1)),
            Err(crate::Error::InvalidParameter { .. })
        ));
        assert!(matches!(
            violin_with(["a", "b"], [[1.0, 2.0]], ViolinOptions::default()),
            Err(crate::Error::UnequalChannels { .. })
        ));
        assert!(matches!(
            contour_with(
                2,
                [0.0, 1.0, 2.0, 3.0],
                ContourOptions::explicit([f64::NAN])
            ),
            Err(crate::Error::InvalidParameter { .. })
        ));
        assert!(matches!(
            contour_with(2, [1.0; 4], ContourOptions::explicit([])),
            Err(crate::Error::InvalidParameter { .. })
        ));
        assert!(matches!(
            contour_with(
                2,
                [0.0, 1.0, 2.0, 3.0],
                ContourOptions::automatic(crate::stat::MAX_STAT_ELEMENTS)
            ),
            Err(crate::Error::DimensionTooLarge { .. })
        ));
    }
}
