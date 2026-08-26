//! Layout: everything geometric, computed once — scales, ticks, gutters, offsets.

use crate::plot::frame::Frame;
use crate::plot::resolve::{ResolvedLayer, extent, union};
use crate::render::{Charset, display_width};
use crate::scale::{Band, Colormap, Linear, Scale, Ticks};

/// A colorbar: the colormap strip drawn down the right edge, legending a Cells
/// layer's value range.
pub(crate) struct Colorbar {
    pub colormap: Colormap,
    pub low: f64,
    pub high: f64,
    pub ticks: Ticks,
    /// The cell column the gradient strip occupies.
    pub column: usize,
}

/// The colormap, value range, ticks, and reserved column count (gap, gradient, gap,
/// labels) for the first Cells layer with a finite value range.
fn cells_colorbar(
    layers: &[ResolvedLayer<'_>],
    plot_rows: usize,
) -> Option<(Colormap, f64, f64, Ticks, usize)> {
    // An rgb grid has no value scale to legend, so only value cells qualify.
    let (values, colormap) = layers.iter().find_map(|layer| match layer {
        ResolvedLayer::Cells {
            values,
            colormap,
            rgb: None,
            classes: None,
            ..
        } => Some((*values, colormap.clone())),
        _ => None,
    })?;
    // The colorbar must label the same value range the cells were colored
    // by — for a centered map, the symmetric span around its midpoint; for a
    // log map, the positive values with decade ticks.
    let (low, high) = if colormap.is_log() {
        crate::plot::resolve::extent_positive(values)?
    } else {
        extent(values)?
    };
    let (low, high) = colormap.display_domain(low, high);
    let target = (plot_rows / 2).clamp(2, 5);
    let ticks = if colormap.is_log() && low > 0.0 && high > 0.0 {
        Ticks::log10(low, high, target)
    } else {
        Ticks::linear(low, high, target)
    };
    let label_width = ticks
        .iter()
        .map(|tick| display_width(&tick.label))
        .max()
        .unwrap_or(1);
    Some((colormap, low, high, ticks, 3 + label_width))
}

/// Manual axis overrides: `(x, y)`, each `Some((min, max))` when fixed.
pub(crate) type Domains = (Option<(f64, f64)>, Option<(f64, f64)>);

#[derive(Debug, Clone, Copy)]
pub(crate) enum Map {
    Linear(Linear),
    Log(Linear),
}

impl Map {
    pub(crate) fn build(domain: (f64, f64), range: (f64, f64), log: bool) -> Map {
        if log {
            Map::Log(Linear::new((domain.0.log10(), domain.1.log10()), range))
        } else {
            Map::Linear(Linear::new(domain, range))
        }
    }

    pub(crate) fn map(&self, value: f64) -> f64 {
        match self {
            Map::Linear(linear) => linear.map(value),
            Map::Log(linear) => linear.map(value.log10()),
        }
    }

    pub(crate) fn unmap(&self, value: f64) -> f64 {
        match self {
            Map::Linear(linear) => linear.unmap(value),
            Map::Log(linear) => 10.0f64.powf(linear.unmap(value)),
        }
    }
}

/// The resolved geometry of one render: where everything goes and how data maps
/// onto it. Computed once per render, read by chrome and mark drawing.
pub(crate) struct Layout<'p> {
    pub frame_width: usize,
    pub px: usize,
    pub py: usize,
    pub ascii: bool,
    pub charset: Charset,
    pub title_rows: usize,
    pub legend_rows: usize,
    pub axis_rows: usize,
    pub x_label_rows: usize,
    pub y_label_cols: usize,
    pub plot_top: usize,
    pub plot_rows: usize,
    pub gutter: usize,
    pub label_width: usize,
    pub plot_cols: usize,
    pub plot_sub_w: usize,
    pub plot_sub_h: usize,
    pub x_offset: f64,
    pub y_offset: f64,
    pub x_scale: Map,
    pub y_scale: Map,
    pub y_ticks: Ticks,
    pub x_ticks: Option<Ticks>,
    pub band: Option<Band>,
    pub y_band: Option<Band>,
    pub categories: Option<&'p [String]>,
    pub colorbar: Option<Colorbar>,
}

impl<'p> Layout<'p> {
    /// Computes the full geometry for `layers` in `frame`, at `density` subpixels
    /// per cell — the charset's density for glyph output, the cell size in device
    /// pixels for pixel output. Chrome geometry is in cells either way; only the
    /// scales' range resolution changes.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute(
        frame: &Frame,
        density: (usize, usize),
        layers: &[ResolvedLayer<'p>],
        has_title: bool,
        scales: (&'p Scale, &Scale),
        axis_labels: (Option<&str>, Option<&str>),
        domains: Domains,
        colorbar_requested: bool,
    ) -> Layout<'p> {
        let (x_spec, y_spec) = scales;
        let (has_x_label, has_y_label) = (axis_labels.0.is_some(), axis_labels.1.is_some());
        let (px, py) = density;
        // An explicit Bands spec wins; otherwise band layers imply the categories.
        let categories: Option<&[String]> = match x_spec {
            Scale::Bands(categories) if !categories.is_empty() => Some(categories.as_slice()),
            // Only an automatic axis infers categories from its layers; an explicitly
            // chosen numeric scale is honored, never silently overridden by a bars
            // layer (validation rejects that combination up front).
            Scale::Auto => layers.iter().find_map(|layer| match layer {
                ResolvedLayer::Bars {
                    placement: crate::mark::Placement::Bands(categories),
                    ..
                } if !categories.is_empty() => Some(categories.as_slice()),
                ResolvedLayer::Range {
                    bands: Some(categories),
                    ..
                } if !categories.is_empty() => Some(*categories),
                _ => None,
            }),
            _ => None,
        };
        let has_bars = layers
            .iter()
            .any(|layer| matches!(layer, ResolvedLayer::Bars { .. }));
        // The y axis takes bands only explicitly — no mark implies them, because
        // no bar-family mark places itself on y. Band 0 is the top band, so a
        // Cells matrix reads like the printed matrix.
        let y_categories: Option<&[String]> = match y_spec {
            Scale::Bands(categories) if !categories.is_empty() => Some(categories.as_slice()),
            _ => None,
        };

        let time_x = matches!(x_spec, Scale::Time) && categories.is_none();
        let log_x = matches!(x_spec, Scale::Log) && categories.is_none();
        let time_y = matches!(y_spec, Scale::Time);
        let log_y = matches!(y_spec, Scale::Log);
        // A log axis cannot place a non-positive bound; a manual domain that gives
        // one is clamped into the positive decade below its top rather than panicking.
        let clamp_log = |(lo, hi): (f64, f64)| -> (f64, f64) {
            if lo > 0.0 {
                (lo, hi.max(lo * 10.0))
            } else {
                let hi = if hi > 0.0 { hi } else { 100.0 };
                (hi / 1000.0, hi)
            }
        };
        let x_data = if let Some(fixed) = domains.0.filter(|_| categories.is_none()) {
            if log_x { clamp_log(fixed) } else { fixed }
        } else if log_x {
            union(layers.iter().map(ResolvedLayer::x_extent_positive)).unwrap_or((1.0, 100.0))
        } else {
            union(layers.iter().map(ResolvedLayer::x_extent)).unwrap_or((0.0, 1.0))
        };
        let mut y_data = if let Some(fixed) = domains.1.filter(|_| y_categories.is_none()) {
            if log_y { clamp_log(fixed) } else { fixed }
        } else if log_y {
            union(layers.iter().map(ResolvedLayer::y_extent_positive)).unwrap_or((1.0, 100.0))
        } else {
            union(layers.iter().map(ResolvedLayer::y_extent)).unwrap_or((0.0, 1.0))
        };
        if has_bars && !log_y && domains.1.is_none() {
            // Bar length is the encoding, so the baseline must be in view.
            y_data = (y_data.0.min(0.0), y_data.1.max(0.0));
        }

        // Vertical layout: title, legend, plot rows, then the x axis and its
        // labels — shed in priority order (legend first) when the frame is short.
        let ascii = frame.charset == Charset::Ascii;
        let title_rows = usize::from(has_title && frame.height >= 6);
        let has_legend = layers.iter().any(ResolvedLayer::has_legend);
        let legend_rows = usize::from(has_legend && frame.height >= 8);
        let chrome_top = title_rows + legend_rows;
        let axis_rows = match frame.height - chrome_top {
            0..=1 => 0,
            2..=3 => 1,
            _ => 2,
        };
        let x_label_rows = usize::from(
            has_x_label && axis_rows == 2 && frame.height - chrome_top - axis_rows >= 4,
        );
        let plot_rows = frame.height - chrome_top - axis_rows - x_label_rows;

        // Horizontal layout: the y-label gutter is measured, not fixed — and shed
        // entirely when it would eat the plot.
        let target = (plot_rows / 2).clamp(2, 8);
        let y_ticks = if let Some(categories) = y_categories {
            // Band labels ride the tick pipeline: each lands on its band center
            // through the y scale, and the collision shed below drops what a
            // short plot cannot fit. The budget keeps the gutter honest.
            Ticks::bands(
                categories,
                (frame.width / 3).max(1),
                frame.charset.chrome().ellipsis,
            )
        } else if time_y {
            Ticks::time(y_data.0, y_data.1, target)
        } else if log_y {
            Ticks::log10(y_data.0, y_data.1, target)
        } else {
            Ticks::linear(y_data.0, y_data.1, target)
        };
        let mut label_width = y_ticks
            .iter()
            .map(|tick| display_width(&tick.label))
            .max()
            .unwrap_or(0);
        let y_label_cols = usize::from(has_y_label && frame.width >= label_width + 12) * 2;
        let mut gutter = y_label_cols + label_width + 2;
        if gutter + 4 > frame.width {
            label_width = 0;
            gutter = usize::from(frame.width >= 2);
        }
        // Reserve right-edge columns for a colorbar when requested and a Cells layer
        // has a value range to show — shed it when the plot would be left too narrow.
        let available = frame.width - gutter;
        let (plot_cols, colorbar) = match colorbar_requested
            .then(|| cells_colorbar(layers, plot_rows))
            .flatten()
        {
            Some((colormap, low, high, ticks, reserved)) if available > reserved + 12 => {
                let plot_cols = available - reserved;
                let bar = Colorbar {
                    colormap,
                    low,
                    high,
                    ticks,
                    column: gutter + plot_cols + 1,
                };
                (plot_cols, Some(bar))
            }
            _ => (available, None),
        };

        // A manual domain is honored exactly; an automatic one grows to its ticks
        // so the axis spans whole round numbers.
        let y_fixed = domains.1.is_some() && y_categories.is_none();
        let x_fixed = domains.0.is_some() && categories.is_none();
        let y_domain = match y_categories {
            Some(categories) => (0.0, categories.len().saturating_sub(1) as f64),
            None if y_fixed => y_data,
            None => domain_with_ticks(y_data, &y_ticks),
        };
        let plot_sub_w = (plot_cols * px).max(1);
        let plot_sub_h = (plot_rows * py).max(1);

        // The x axis: a band scale when a bars layer is present, ticks otherwise.
        let band = categories.map(|c| Band::new(c.len(), (0.0, (plot_sub_w - 1) as f64)));
        // The y band scale runs top-down: raster row 0 is band 0.
        let y_band = y_categories.map(|c| Band::new(c.len(), (0.0, (plot_sub_h - 1) as f64)));
        let x_ticks = if band.is_none() && axis_rows == 2 {
            if time_x {
                fit_time_ticks(x_data, plot_cols, plot_sub_w, px, gutter, frame.width)
            } else if log_x {
                Some(Ticks::log10(
                    x_data.0,
                    x_data.1,
                    (plot_cols / 10).clamp(2, 8),
                ))
            } else {
                fit_x_ticks(x_data, plot_cols, plot_sub_w, px, gutter, frame.width)
            }
        } else {
            None
        };
        let x_domain = match (&band, &x_ticks) {
            (Some(band), _) => (0.0, (band.count() - 1) as f64),
            (None, Some(_)) if x_fixed => x_data,
            (None, Some(ticks)) => domain_with_ticks(x_data, ticks),
            (None, None) => x_data,
        };
        let x_range = match &band {
            Some(band) => (band.center(0), band.center(band.count() - 1)),
            None => (0.0, (plot_sub_w - 1) as f64),
        };
        let x_scale = Map::build(x_domain, x_range, log_x);
        let y_range = match &y_band {
            Some(band) => (band.center(0), band.center(band.count().saturating_sub(1))),
            None => ((plot_sub_h - 1) as f64, 0.0),
        };
        let y_scale = Map::build(y_domain, y_range, log_y);

        Layout {
            frame_width: frame.width,
            px,
            py,
            ascii,
            charset: frame.charset,
            title_rows,
            legend_rows,
            axis_rows,
            x_label_rows,
            y_label_cols,
            plot_top: chrome_top,
            plot_rows,
            gutter,
            label_width,
            plot_cols,
            plot_sub_w,
            plot_sub_h,
            x_offset: (gutter * px) as f64,
            y_offset: (chrome_top * py) as f64,
            x_scale,
            y_scale,
            y_ticks,
            x_ticks,
            band,
            y_band,
            categories,
            colorbar,
        }
    }
}

fn domain_with_ticks(data: (f64, f64), ticks: &Ticks) -> (f64, f64) {
    match (ticks.as_slice().first(), ticks.as_slice().last()) {
        (Some(first), Some(last)) => (data.0.min(first.value), data.1.max(last.value)),
        _ => data,
    }
}

/// Whether tick labels fit without collisions: centered under their ticks, clamped
/// to the frame, at least two cells apart.
fn labels_fit(
    ticks: &Ticks,
    domain: (f64, f64),
    plot_sub_w: usize,
    px: usize,
    gutter: usize,
    frame_width: usize,
) -> bool {
    let scale = Linear::new(domain, (0.0, (plot_sub_w - 1) as f64));
    let mut last_end: i64 = i64::MIN;
    for tick in ticks {
        let column = (scale.map(tick.value).round() as usize) / px;
        let len = display_width(&tick.label) as i64;
        let center = (gutter + column) as i64;
        let start = (center - len / 2).clamp(0, (frame_width as i64 - len).max(0));
        if start < last_end + 2 {
            return false;
        }
        last_end = start + len;
    }
    true
}

/// Chooses the densest calendar labeling that fits without collisions.
fn fit_time_ticks(
    data: (f64, f64),
    plot_cols: usize,
    plot_sub_w: usize,
    px: usize,
    gutter: usize,
    frame_width: usize,
) -> Option<Ticks> {
    let densest = (plot_cols / 8).clamp(2, 12);
    for target in (2..=densest).rev() {
        let ticks = Ticks::time(data.0, data.1, target);
        if ticks.is_empty() {
            continue;
        }
        if labels_fit(&ticks, data, plot_sub_w, px, gutter, frame_width) {
            return Some(ticks);
        }
    }
    None
}

/// Chooses the densest x labeling whose labels fit without collisions: centered
/// under their ticks, clamped to the frame, at least two cells apart.
fn fit_x_ticks(
    data: (f64, f64),
    plot_cols: usize,
    plot_sub_w: usize,
    px: usize,
    gutter: usize,
    frame_width: usize,
) -> Option<Ticks> {
    let densest = (plot_cols / 8).clamp(2, 12);
    for target in (2..=densest).rev() {
        let ticks = Ticks::linear(data.0, data.1, target);
        let domain = domain_with_ticks(data, &ticks);
        if labels_fit(&ticks, domain, plot_sub_w, px, gutter, frame_width) {
            return Some(ticks);
        }
    }
    None
}
