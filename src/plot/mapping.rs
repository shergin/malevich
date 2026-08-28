//! `Mapping`: the resolved geometry of one render, as a queryable value.

use super::layout::{Layout, Map};
use crate::scale::Scale;

/// How a rendered plot maps cells onto data: the plot rectangle and the
/// resolved scales, computed by the same layout pass rendering uses.
///
/// Obtained purely from [`Plot::mapping`](crate::Plot::mapping) — or cached by
/// the ratatui `PlotState` after a stateful render. A mapping answers the
/// questions interactive hosts ask: which data coordinates live under a cell
/// ([`Mapping::data_at`]), which cell shows a data point ([`Mapping::cell_at`]),
/// what window the axes resolved to ([`Mapping::x_domain`]), and how to write a
/// value the way the axis itself would ([`Mapping::format_x`]).
///
/// Coordinates follow the conventions marks already use: band axes answer in
/// band-index space (0 is the first band; on y, the top band), time axes in
/// unix seconds, log axes in data values. Queries outside the plot rectangle —
/// or on a layout so small the plot shed to nothing — return `None`.
///
/// A mapping is a plain value (`Clone + Send + Sync`) describing one
/// `(plot, frame)` pair; render again after either changes and query the new
/// mapping. It is derived state, deliberately not serializable.
#[derive(Debug, Clone)]
pub struct Mapping {
    /// Plot rectangle: leftmost cell column, topmost cell row, then size.
    left: usize,
    top: usize,
    columns: usize,
    rows: usize,
    /// Subpixels per cell for the charset the layout was computed at.
    px: usize,
    py: usize,
    x: Map,
    y: Map,
    x_domain: (f64, f64),
    y_domain: (f64, f64),
    x_kind: AxisKind,
    y_kind: AxisKind,
    x_categories: Option<Vec<String>>,
    y_categories: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AxisKind {
    Linear,
    Log,
    Time,
    Bands,
}

impl Mapping {
    pub(crate) fn new(layout: &Layout<'_>, x_spec: &Scale, y_spec: &Scale) -> Mapping {
        let x_kind = if layout.band.is_some() {
            AxisKind::Bands
        } else {
            AxisKind::of(x_spec)
        };
        let y_kind = if layout.y_band.is_some() {
            AxisKind::Bands
        } else {
            AxisKind::of(y_spec)
        };
        let y_categories = match y_spec {
            Scale::Bands(categories) if layout.y_band.is_some() => Some(categories.clone()),
            _ => None,
        };
        Mapping {
            left: layout.gutter,
            top: layout.plot_top,
            columns: layout.plot_cols,
            rows: layout.plot_rows,
            px: layout.px.max(1),
            py: layout.py.max(1),
            x: layout.x_scale,
            y: layout.y_scale,
            x_domain: layout.x_domain,
            y_domain: layout.y_domain,
            x_kind,
            y_kind,
            x_categories: layout.categories.map(<[String]>::to_vec),
            y_categories,
        }
    }

    /// A mapping with an empty plot rectangle; every positional query is `None`.
    pub(crate) fn empty() -> Mapping {
        Mapping {
            left: 0,
            top: 0,
            columns: 0,
            rows: 0,
            px: 1,
            py: 1,
            x: Map::build((0.0, 1.0), (0.0, 1.0), false),
            y: Map::build((0.0, 1.0), (1.0, 0.0), false),
            x_domain: (0.0, 1.0),
            y_domain: (0.0, 1.0),
            x_kind: AxisKind::Linear,
            y_kind: AxisKind::Linear,
            x_categories: None,
            y_categories: None,
        }
    }

    /// The plot rectangle as `(column, row, width, height)` in frame cells —
    /// the data panel only, chrome excluded — or `None` when the frame was too
    /// small to draw one.
    pub fn plot_area(&self) -> Option<(usize, usize, usize, usize)> {
        (self.columns > 0 && self.rows > 0).then_some((
            self.left,
            self.top,
            self.columns,
            self.rows,
        ))
    }

    /// The data coordinates at the center of the frame cell `(column, row)`,
    /// or `None` outside the plot rectangle.
    ///
    /// One cell spans an interval of data, not a point — [`Mapping::x_span_at`]
    /// discloses how much. Band axes answer in fractional band-index space;
    /// snap with `round()` and clamp to the band count.
    pub fn data_at(&self, column: usize, row: usize) -> Option<(f64, f64)> {
        if self.columns == 0 || self.rows == 0 {
            return None;
        }
        let inside = (self.left..self.left + self.columns).contains(&column)
            && (self.top..self.top + self.rows).contains(&row);
        if !inside {
            return None;
        }
        let sub_x = ((column - self.left) * self.px) as f64 + (self.px as f64 - 1.0) / 2.0;
        let sub_y = ((row - self.top) * self.py) as f64 + (self.py as f64 - 1.0) / 2.0;
        Some((self.x.unmap(sub_x), self.y.unmap(sub_y)))
    }

    /// The frame cell `(column, row)` where the data point `(x, y)` draws, or
    /// `None` when it falls outside the plot rectangle (or is not finite).
    pub fn cell_at(&self, x: f64, y: f64) -> Option<(usize, usize)> {
        if self.columns == 0 || self.rows == 0 {
            return None;
        }
        let sub_x = self.x.map(x).round();
        let sub_y = self.y.map(y).round();
        if !(sub_x.is_finite() && sub_y.is_finite()) || sub_x < 0.0 || sub_y < 0.0 {
            return None;
        }
        let column = self.left + sub_x as usize / self.px;
        let row = self.top + sub_y as usize / self.py;
        (column < self.left + self.columns && row < self.top + self.rows).then_some((column, row))
    }

    /// The data interval one plot column covers — the x resolution a cell-level
    /// cursor honestly has — or `None` when `column` is outside the plot.
    pub fn x_span_at(&self, column: usize) -> Option<(f64, f64)> {
        if self.columns == 0 || !(self.left..self.left + self.columns).contains(&column) {
            return None;
        }
        let start = ((column - self.left) * self.px) as f64 - 0.5;
        let (a, b) = (self.x.unmap(start), self.x.unmap(start + self.px as f64));
        Some((a.min(b), a.max(b)))
    }

    /// The resolved x window: the manual domain if one was set, otherwise the
    /// automatic domain grown to its ticks. A bands axis answers in band-index
    /// space: `(0, count - 1)`.
    pub fn x_domain(&self) -> (f64, f64) {
        self.x_domain
    }

    /// The resolved y window; see [`Mapping::x_domain`].
    pub fn y_domain(&self) -> (f64, f64) {
        self.y_domain
    }

    /// The number of x bands when the x axis is categorical.
    pub fn x_bands(&self) -> Option<usize> {
        matches!(self.x_kind, AxisKind::Bands).then(|| {
            self.x_categories
                .as_ref()
                .map_or_else(|| self.x_domain.1 as usize + 1, Vec::len)
        })
    }

    /// The number of y bands when the y axis is categorical.
    pub fn y_bands(&self) -> Option<usize> {
        matches!(self.y_kind, AxisKind::Bands).then(|| {
            self.y_categories
                .as_ref()
                .map_or_else(|| self.y_domain.1 as usize + 1, Vec::len)
        })
    }

    /// Formats an x value the way the x axis would: exact decimals at the
    /// resolution one cell actually has (never `0.30000000000000004`, never
    /// false precision), calendar instants on a time axis, the category label
    /// on a bands axis.
    pub fn format_x(&self, value: f64) -> String {
        format_value(
            value,
            self.x_kind,
            self.x_domain,
            self.columns,
            self.x_categories.as_deref(),
        )
    }

    /// Formats a y value the way the y axis would; see [`Mapping::format_x`].
    pub fn format_y(&self, value: f64) -> String {
        format_value(
            value,
            self.y_kind,
            self.y_domain,
            self.rows,
            self.y_categories.as_deref(),
        )
    }

    /// A [`Viewport`](crate::Viewport) fixed to this mapping's resolved
    /// domains — "the view I am looking at", the natural seed for zoom and pan.
    /// Bands axes stay unfixed: a categorical axis has no continuous window.
    pub fn viewport(&self) -> crate::plot::Viewport {
        crate::plot::Viewport::seeded(
            (self.x_kind != AxisKind::Bands).then_some(self.x_domain),
            (self.y_kind != AxisKind::Bands).then_some(self.y_domain),
            self.x_kind == AxisKind::Log,
            self.y_kind == AxisKind::Log,
        )
    }
}

impl AxisKind {
    fn of(spec: &Scale) -> AxisKind {
        match spec {
            Scale::Log => AxisKind::Log,
            Scale::Time => AxisKind::Time,
            _ => AxisKind::Linear,
        }
    }
}

/// One value, formatted at the axis's honest resolution: the domain span over
/// the cell count decides how many decimals a readout can truthfully carry.
fn format_value(
    value: f64,
    kind: AxisKind,
    domain: (f64, f64),
    cells: usize,
    categories: Option<&[String]>,
) -> String {
    if !value.is_finite() {
        return value.to_string();
    }
    match kind {
        AxisKind::Bands => {
            let index = value.round();
            match categories {
                Some(categories) if !categories.is_empty() => {
                    let index = (index.max(0.0) as usize).min(categories.len() - 1);
                    categories[index].clone()
                }
                _ => index.to_string(),
            }
        }
        AxisKind::Time => {
            crate::scale::time::readout(value, (domain.1 - domain.0) / cells.max(1) as f64)
        }
        AxisKind::Linear => decimal_at(value, (domain.1 - domain.0) / cells.max(1) as f64),
        AxisKind::Log => {
            if value <= 0.0 || domain.0 <= 0.0 || domain.1 <= 0.0 {
                return value.to_string();
            }
            let per_cell = (domain.1.log10() - domain.0.log10()) / cells.max(1) as f64;
            decimal_at(value, value * std::f64::consts::LN_10 * per_cell)
        }
    }
}

/// Exact-decimal formatting of `value` rounded to the resolution `step`: the
/// fraction digits are just enough to distinguish neighboring cells.
fn decimal_at(value: f64, step: f64) -> String {
    let decimals = if step.is_finite() && step > 0.0 {
        (-step.log10()).ceil().clamp(0.0, 12.0) as i32
    } else {
        3
    };
    let scaled = value * 10f64.powi(decimals);
    if scaled.abs() >= 1e15 {
        // Beyond exact-integer range the decimal would lie; fall back.
        return format!("{value}");
    }
    crate::scale::format::decimal(scaled.round() as i128, -decimals)
}

#[cfg(test)]
#[path = "tests/mapping_tests.rs"]
mod tests;
