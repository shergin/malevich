//! Resolution: marks materialized into drawable columns with resolved colors.

use std::borrow::Cow;

use crate::mark::{LineStyle, Mark, Orientation, Placement, PointStyle, RangePlacement, Source};
use crate::plot::layout::Map;
use crate::render::Color;
use crate::scale::Colormap;

/// A resolved coordinate channel, either backed by values or by the implicit
/// `0, 1, 2, ...` indices used when a mark omits x coordinates.
///
/// Keeping indices symbolic avoids allocating an otherwise redundant `Vec<f64>`
/// for every unreduced line, point, area, and numeric range layer.
pub(crate) enum Coordinates<'p> {
    Values(Cow<'p, [f64]>),
    Indices(usize),
}

impl Coordinates<'_> {
    pub(crate) fn iter(&self) -> CoordinatesIter<'_> {
        match self {
            Coordinates::Values(values) => CoordinatesIter::Values(values.iter()),
            Coordinates::Indices(len) => CoordinatesIter::Indices(0..*len),
        }
    }

    fn extent(&self) -> Option<(f64, f64)> {
        match self {
            Coordinates::Values(values) => extent(values),
            Coordinates::Indices(0) => None,
            Coordinates::Indices(len) => Some((0.0, (*len - 1) as f64)),
        }
    }

    fn extent_positive(&self) -> Option<(f64, f64)> {
        match self {
            Coordinates::Values(values) => extent_positive(values),
            Coordinates::Indices(0 | 1) => None,
            Coordinates::Indices(len) => Some((1.0, (*len - 1) as f64)),
        }
    }
}

pub(crate) enum CoordinatesIter<'a> {
    Values(std::slice::Iter<'a, f64>),
    Indices(std::ops::Range<usize>),
}

impl Iterator for CoordinatesIter<'_> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            CoordinatesIter::Values(values) => values.next().copied(),
            CoordinatesIter::Indices(indices) => indices.next().map(|index| index as f64),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            CoordinatesIter::Values(values) => values.size_hint(),
            CoordinatesIter::Indices(indices) => indices.size_hint(),
        }
    }
}

impl ExactSizeIterator for CoordinatesIter<'_> {}

/// How large line layers are reduced before rasterizing.
#[derive(Clone, Copy)]
pub(crate) enum Reduce {
    /// Draw every point — the raw raster, and the oracle it is checked against.
    None,
    /// Collapse each line to the two endpoints of the extent needed by each axis.
    /// The flags select positive-only summaries for log scales.
    Extent { x_positive: bool, y_positive: bool },
    /// Pixel-exact M4 bucketed by the rendered column, using the resolved layout's
    /// scale and subpixel width.
    Mapped { map: Map, columns: usize },
}

/// How a resolved series layer draws its columns.
pub(crate) enum Kind {
    Line(LineStyle),
    Points(PointStyle),
}

/// A resolved interval body: the open/close pair, borrowed or masked-owned.
pub(crate) type Body<'p> = (Cow<'p, [f64]>, Cow<'p, [f64]>);

/// One layer, resolved to drawable data.
pub(crate) enum ResolvedLayer<'p> {
    Series {
        x: Coordinates<'p>,
        y: Cow<'p, [f64]>,
        color: Color,
        kind: Kind,
        label: Option<&'p str>,
    },
    Bars {
        placement: &'p Placement,
        values: Cow<'p, [f64]>,
        color: Color,
        label: Option<&'p str>,
    },
    Area {
        x: Coordinates<'p>,
        low: Option<&'p [f64]>,
        high: &'p [f64],
        horizontal: bool,
        color: Color,
        label: Option<&'p str>,
    },
    Cells {
        columns: usize,
        values: &'p [f64],
        extents: Option<((f64, f64), (f64, f64))>,
        colormap: Colormap,
    },
    Range {
        x: Coordinates<'p>,
        categories: Option<&'p [String]>,
        low: Cow<'p, [f64]>,
        high: Cow<'p, [f64]>,
        body: Option<Body<'p>>,
        marker: Option<Cow<'p, [f64]>>,
        color: Color,
        label: Option<&'p str>,
    },
    Rule {
        orientation: Orientation,
        color: Color,
        label: Option<&'p str>,
    },
    Text {
        x: f64,
        y: f64,
        text: &'p str,
        color: Color,
    },
}

impl ResolvedLayer<'_> {
    /// The finite x extent this layer contributes to the shared domain.
    /// Bars contribute none — their axis is the band scale.
    pub(crate) fn x_extent(&self) -> Option<(f64, f64)> {
        match self {
            ResolvedLayer::Series { x, .. } => x.extent(),
            ResolvedLayer::Bars {
                placement: Placement::Spans { start, width },
                values,
                ..
            } => Some((*start, width.mul_add(values.len() as f64, *start))),
            ResolvedLayer::Bars { .. } => None,
            ResolvedLayer::Area {
                x,
                low,
                high,
                horizontal,
                ..
            } => {
                if *horizontal {
                    union([low.and_then(extent), extent(high)].into_iter())
                } else {
                    x.extent()
                }
            }
            ResolvedLayer::Rule {
                orientation: Orientation::Vertical(x),
                ..
            } => Some((*x, *x)),
            ResolvedLayer::Rule { .. } => None,
            ResolvedLayer::Text { x, .. } => Some((*x, *x)),
            ResolvedLayer::Cells {
                columns, extents, ..
            } => Some(match extents {
                Some((x, _)) => *x,
                None => (0.0, *columns as f64),
            }),
            ResolvedLayer::Range { x, categories, .. } => {
                if categories.is_some() {
                    None
                } else {
                    x.extent()
                }
            }
        }
    }

    /// The finite y extent this layer contributes to the shared domain.
    pub(crate) fn y_extent(&self) -> Option<(f64, f64)> {
        match self {
            ResolvedLayer::Series { y, .. } => extent(y),
            ResolvedLayer::Bars { values, .. } => extent(values),
            ResolvedLayer::Area {
                x,
                low,
                high,
                horizontal,
                ..
            } => {
                if *horizontal {
                    x.extent()
                } else {
                    let highs = extent(high);
                    let lows = match low {
                        Some(low) => extent(low),
                        // A baseline fill keeps zero in view, like bars.
                        None => Some((0.0, 0.0)),
                    };
                    union([highs, lows].into_iter())
                }
            }
            ResolvedLayer::Rule {
                orientation: Orientation::Horizontal(y),
                ..
            } => Some((*y, *y)),
            ResolvedLayer::Rule { .. } => None,
            ResolvedLayer::Text { y, .. } => Some((*y, *y)),
            ResolvedLayer::Cells {
                columns,
                values,
                extents,
                ..
            } => Some(match extents {
                Some((_, y)) => *y,
                None => (0.0, (values.len() / (*columns).max(1)) as f64),
            }),
            ResolvedLayer::Range {
                low,
                high,
                body,
                marker,
                ..
            } => union(
                [
                    extent(low),
                    extent(high),
                    // The body can reach past the whiskers; every encoded coordinate
                    // must fit the scale, or it renders clipped.
                    body.as_ref().and_then(|(lo, _)| extent(lo)),
                    body.as_ref().and_then(|(_, hi)| extent(hi)),
                    marker.as_deref().and_then(extent),
                ]
                .into_iter(),
            ),
        }
    }

    /// [`ResolvedLayer::x_extent`] over strictly positive values (log axes).
    pub(crate) fn x_extent_positive(&self) -> Option<(f64, f64)> {
        match self {
            ResolvedLayer::Series { x, .. } => x.extent_positive(),
            ResolvedLayer::Area {
                x,
                low,
                high,
                horizontal,
                ..
            } => {
                if *horizontal {
                    union([low.and_then(extent_positive), extent_positive(high)].into_iter())
                } else {
                    x.extent_positive()
                }
            }
            ResolvedLayer::Rule {
                orientation: Orientation::Vertical(x),
                ..
            } if *x > 0.0 => Some((*x, *x)),
            ResolvedLayer::Text { x, .. } if *x > 0.0 => Some((*x, *x)),
            ResolvedLayer::Cells { .. } => self.x_extent().filter(|(lo, _)| *lo > 0.0),
            ResolvedLayer::Range {
                x,
                categories: None,
                ..
            } => x.extent_positive(),
            _ => None,
        }
    }

    /// [`ResolvedLayer::y_extent`] over strictly positive values (log axes).
    pub(crate) fn y_extent_positive(&self) -> Option<(f64, f64)> {
        match self {
            ResolvedLayer::Series { y, .. } => extent_positive(y),
            ResolvedLayer::Bars { values, .. } => extent_positive(values),
            ResolvedLayer::Area {
                x,
                low,
                high,
                horizontal,
                ..
            } => {
                if *horizontal {
                    x.extent_positive()
                } else {
                    union([extent_positive(high), low.and_then(extent_positive)].into_iter())
                }
            }
            ResolvedLayer::Rule {
                orientation: Orientation::Horizontal(y),
                ..
            } if *y > 0.0 => Some((*y, *y)),
            ResolvedLayer::Text { y, .. } if *y > 0.0 => Some((*y, *y)),
            ResolvedLayer::Cells { .. } => self.y_extent().filter(|(lo, _)| *lo > 0.0),
            ResolvedLayer::Range { low, high, .. } => {
                union([extent_positive(low), extent_positive(high)].into_iter())
            }
            _ => None,
        }
    }

    /// The legend entry of this layer, if labeled: swatch text, color, label.
    pub(crate) fn legend_entry(&self, ascii: bool) -> Option<(&'static str, Color, &str)> {
        let (swatch, color, label) = match self {
            ResolvedLayer::Series {
                color, kind, label, ..
            } => {
                let swatch = match (kind, ascii) {
                    (Kind::Line(_), false) => "\u{2500}\u{2500}",
                    (Kind::Line(_), true) => "--",
                    (Kind::Points(PointStyle::Dot), false) => "\u{2022}\u{2022}",
                    (Kind::Points(PointStyle::Dot), true) => "..",
                    (Kind::Points(PointStyle::Plus), _) => "++",
                    (Kind::Points(PointStyle::Cross), _) => "xx",
                    (Kind::Points(PointStyle::Asterisk), _) => "**",
                    (Kind::Points(PointStyle::Circle), _) => "oo",
                };
                (swatch, *color, *label)
            }
            ResolvedLayer::Bars { color, label, .. } => {
                let swatch = if ascii { "##" } else { "\u{2588}\u{2588}" };
                (swatch, *color, *label)
            }
            ResolvedLayer::Area { color, label, .. } => {
                let swatch = if ascii { "##" } else { "\u{2584}\u{2584}" };
                (swatch, *color, *label)
            }
            ResolvedLayer::Rule { color, label, .. } => {
                let swatch = if ascii { "--" } else { "\u{2500}\u{2500}" };
                (swatch, *color, *label)
            }
            ResolvedLayer::Range { color, label, .. } => {
                let swatch = if ascii { "||" } else { "\u{2503}\u{2503}" };
                (swatch, *color, *label)
            }
            ResolvedLayer::Text { .. } | ResolvedLayer::Cells { .. } => return None,
        };
        label.map(|label| (swatch, color, label))
    }
}

/// Materializes every layer into drawable columns plus a resolved color.
/// Functions are sampled here, once per subpixel column of the frame width.
/// A `color_by` mark expands into one layer per category, colored from the
/// categorical palette and labeled for the legend; `cycle_markers` (plain
/// output) additionally cycles default point markers so the categories stay
/// separable without color.
pub(crate) fn resolve<'p>(
    marks: &'p [Mark<'_>],
    sample_width: usize,
    palette: &[Color; 6],
    categorical: &crate::scale::Palette,
    cycle_markers: bool,
    reduce: Reduce,
) -> Vec<ResolvedLayer<'p>> {
    // Annotations (rules, text) draw in the default foreground and do not
    // consume palette slots; a single data layer draws in the default too.
    let data_layers = marks
        .iter()
        .filter(|mark| !matches!(mark, Mark::Rule(_) | Mark::Text(_)))
        .count();
    let single = data_layers == 1;
    let mut palette_index = 0usize;
    marks
        .iter()
        .flat_map(|mark| {
            if let Some(layers) = expand_color_by(mark, categorical, cycle_markers, reduce) {
                return layers;
            }
            let mut assigned = |explicit: Option<Color>| {
                let index = palette_index;
                palette_index += 1;
                explicit.unwrap_or(if single {
                    Color::Default
                } else {
                    palette[index % palette.len()]
                })
            };
            vec![match mark {
                Mark::Line(line) => {
                    let color = assigned(line.color);
                    match &line.source {
                        Source::Points { x, y } => {
                            // The aggregate-to-raster pipeline: past four points per
                            // raster column, M4 reduces the series to what the column
                            // can show. Mapped M4 buckets by the rendered column, so
                            // the reduction is pixel-exact; non-monotonic x declines.
                            let downsampled = reduced(
                                x.as_ref().map(|series| series.as_slice()),
                                y.as_slice(),
                                reduce,
                            );
                            match downsampled {
                                Some((dx, dy)) => ResolvedLayer::Series {
                                    x: Coordinates::Values(Cow::Owned(dx)),
                                    y: Cow::Owned(dy),
                                    color,
                                    kind: Kind::Line(line.style),
                                    label: line.label.as_deref(),
                                },
                                None => ResolvedLayer::Series {
                                    x: coordinates(x.as_ref(), y.len()),
                                    y: Cow::Borrowed(y.as_slice()),
                                    color,
                                    kind: Kind::Line(line.style),
                                    label: line.label.as_deref(),
                                },
                            }
                        }
                        Source::Function { domain, function } => {
                            let samples = sample_width.max(2);
                            let x: Vec<f64> = (0..samples)
                                .map(|index| {
                                    crate::numeric::lerp(
                                        domain.0,
                                        domain.1,
                                        index as f64 / (samples - 1) as f64,
                                    )
                                })
                                .collect();
                            let y: Vec<f64> = x.iter().map(|&value| function(value)).collect();
                            ResolvedLayer::Series {
                                x: Coordinates::Values(Cow::Owned(x)),
                                y: Cow::Owned(y),
                                color,
                                kind: Kind::Line(line.style),
                                label: line.label.as_deref(),
                            }
                        }
                    }
                }
                Mark::Points(points) => ResolvedLayer::Series {
                    x: coordinates(points.x.as_ref(), points.y.len()),
                    y: Cow::Borrowed(points.y.as_slice()),
                    color: assigned(points.color),
                    kind: Kind::Points(points.style),
                    label: points.label.as_deref(),
                },
                Mark::Bars(bars) => ResolvedLayer::Bars {
                    placement: &bars.placement,
                    values: Cow::Borrowed(bars.values.as_slice()),
                    color: assigned(bars.color),
                    label: bars.label.as_deref(),
                },
                Mark::Area(area) => ResolvedLayer::Area {
                    x: coordinates(area.x.as_ref(), area.high.len()),
                    low: area.low.as_ref().map(|series| series.as_slice()),
                    high: area.high.as_slice(),
                    horizontal: area.horizontal,
                    color: assigned(area.color),
                    label: area.label.as_deref(),
                },
                Mark::Cells(cells) => ResolvedLayer::Cells {
                    columns: cells.columns,
                    values: cells.values.as_slice(),
                    extents: cells.extents,
                    colormap: cells.colormap.clone(),
                },
                Mark::Range(range) => {
                    let (x, categories) = match &range.placement {
                        RangePlacement::Numeric(x) => {
                            (coordinates(x.as_ref(), range.low.len()), None)
                        }
                        RangePlacement::Bands(categories) => (
                            Coordinates::Indices(categories.len()),
                            Some(categories.as_slice()),
                        ),
                    };
                    ResolvedLayer::Range {
                        x,
                        categories,
                        low: Cow::Borrowed(range.low.as_slice()),
                        high: Cow::Borrowed(range.high.as_slice()),
                        body: range.body.as_ref().map(|(low, high)| {
                            (
                                Cow::Borrowed(low.as_slice()),
                                Cow::Borrowed(high.as_slice()),
                            )
                        }),
                        marker: range.marker.as_ref().map(|m| Cow::Borrowed(m.as_slice())),
                        color: assigned(range.color),
                        label: range.label.as_deref(),
                    }
                }
                Mark::Rule(rule) => ResolvedLayer::Rule {
                    orientation: rule.orientation,
                    color: rule.color.unwrap_or(Color::Default),
                    label: rule.label.as_deref(),
                },
                Mark::Text(text) => ResolvedLayer::Text {
                    x: text.x,
                    y: text.y,
                    text: &text.text,
                    color: text.color.unwrap_or(Color::Default),
                },
            }]
        })
        .collect()
}

/// The marker shapes colorless output cycles through per category, so groups
/// separate without color. Explicit styles are never overridden.
const MARKER_CYCLE: [PointStyle; 5] = [
    PointStyle::Dot,
    PointStyle::Plus,
    PointStyle::Cross,
    PointStyle::Asterisk,
    PointStyle::Circle,
];

/// NaN-masks `values` down to the elements of one category — the mask is the
/// gap convention, so masked-out elements draw nothing, honestly.
fn masked(values: &[f64], indices: &[usize], category: usize) -> Vec<f64> {
    values
        .iter()
        .zip(indices)
        .map(|(value, index)| if *index == category { *value } else { f64::NAN })
        .collect()
}

/// Applies the layer reduction to one line series, returning owned reduced
/// coordinates when a reduction applies.
fn reduced(x: Option<&[f64]>, y: &[f64], reduce: Reduce) -> Option<(Vec<f64>, Vec<f64>)> {
    match reduce {
        Reduce::None => None,
        Reduce::Extent {
            x_positive,
            y_positive,
        } => line_extent(x, y, x_positive, y_positive),
        Reduce::Mapped { map, columns } if y.len() > 4 * columns.max(1) => {
            crate::stat::m4_mapped(x, y, columns, |value| map.map(value))
        }
        Reduce::Mapped { .. } => None,
    }
}

/// Expands a `color_by` mark into one resolved layer per category. Returns
/// `None` for marks without the channel (including a function-backed line,
/// which has no per-point categories).
fn expand_color_by<'p>(
    mark: &'p Mark<'_>,
    categorical: &crate::scale::Palette,
    cycle_markers: bool,
    reduce: Reduce,
) -> Option<Vec<ResolvedLayer<'p>>> {
    match mark {
        Mark::Points(points) => {
            let categories = points.color_by.as_ref()?;
            Some(
                categories
                    .labels()
                    .iter()
                    .enumerate()
                    .map(|(category, name)| ResolvedLayer::Series {
                        x: coordinates(points.x.as_ref(), points.y.len()),
                        y: Cow::Owned(masked(points.y.as_slice(), categories.ids(), category)),
                        color: categorical.color(category),
                        kind: Kind::Points(if cycle_markers && points.style == PointStyle::Dot {
                            MARKER_CYCLE[category % MARKER_CYCLE.len()]
                        } else {
                            points.style
                        }),
                        label: Some(name),
                    })
                    .collect(),
            )
        }
        Mark::Line(line) => {
            let categories = line.color_by.as_ref()?;
            let Source::Points { x, y } = &line.source else {
                return None;
            };
            let x_slice = x.as_ref().map(|series| series.as_slice());
            Some(
                categories
                    .labels()
                    .iter()
                    .enumerate()
                    .map(|(category, name)| {
                        let masked_y = masked(y.as_slice(), categories.ids(), category);
                        match reduced(x_slice, &masked_y, reduce) {
                            Some((dx, dy)) => ResolvedLayer::Series {
                                x: Coordinates::Values(Cow::Owned(dx)),
                                y: Cow::Owned(dy),
                                color: categorical.color(category),
                                kind: Kind::Line(line.style),
                                label: Some(name),
                            },
                            None => ResolvedLayer::Series {
                                x: coordinates(x.as_ref(), y.len()),
                                y: Cow::Owned(masked_y),
                                color: categorical.color(category),
                                kind: Kind::Line(line.style),
                                label: Some(name),
                            },
                        }
                    })
                    .collect(),
            )
        }
        Mark::Bars(bars) => {
            let categories = bars.color_by.as_ref()?;
            Some(
                categories
                    .labels()
                    .iter()
                    .enumerate()
                    .map(|(category, name)| ResolvedLayer::Bars {
                        placement: &bars.placement,
                        values: Cow::Owned(masked(
                            bars.values.as_slice(),
                            categories.ids(),
                            category,
                        )),
                        color: categorical.color(category),
                        label: Some(name),
                    })
                    .collect(),
            )
        }
        Mark::Range(range) => {
            let categories = range.color_by.as_ref()?;
            Some(
                categories
                    .labels()
                    .iter()
                    .enumerate()
                    .map(|(category, name)| {
                        let (x, band_categories) = match &range.placement {
                            RangePlacement::Numeric(x) => {
                                (coordinates(x.as_ref(), range.low.len()), None)
                            }
                            RangePlacement::Bands(bands) => {
                                (Coordinates::Indices(bands.len()), Some(bands.as_slice()))
                            }
                        };
                        ResolvedLayer::Range {
                            x,
                            categories: band_categories,
                            low: Cow::Owned(masked(
                                range.low.as_slice(),
                                categories.ids(),
                                category,
                            )),
                            high: Cow::Owned(masked(
                                range.high.as_slice(),
                                categories.ids(),
                                category,
                            )),
                            body: range.body.as_ref().map(|(low, high)| {
                                (
                                    Cow::Owned(masked(low.as_slice(), categories.ids(), category)),
                                    Cow::Owned(masked(high.as_slice(), categories.ids(), category)),
                                )
                            }),
                            marker: range.marker.as_ref().map(|marker| {
                                Cow::Owned(masked(marker.as_slice(), categories.ids(), category))
                            }),
                            color: categorical.color(category),
                            label: Some(name),
                        }
                    })
                    .collect(),
            )
        }
        _ => None,
    }
}

/// Compact x and y summaries for the extent each axis actually consumes.
/// Layout reads the channels independently; `x = None` means the implicit indices
/// `0, 1, 2, …`.
fn line_extent(
    x: Option<&[f64]>,
    y: &[f64],
    x_positive: bool,
    y_positive: bool,
) -> Option<(Vec<f64>, Vec<f64>)> {
    let len = y.len();
    let y = selected_extent_summary(y, y_positive)?;
    let x = match x {
        Some(values) => selected_extent_summary(values, x_positive)?,
        None if x_positive && len > 1 => vec![1.0, (len - 1) as f64],
        None if !x_positive && len > 0 => vec![0.0, (len - 1) as f64],
        None => return None,
    };
    Some((x, y))
}

fn selected_extent_summary(values: &[f64], positive: bool) -> Option<Vec<f64>> {
    if !positive {
        // The common linear-axis path stays branch-free and vectorizable. NaNs
        // disappear through `f64::min`/`max`; infinities trigger the exact slow
        // path so they cannot hide the outermost finite value.
        let (mut min, mut max) = (f64::INFINITY, f64::NEG_INFINITY);
        for &value in values {
            min = min.min(value);
            max = max.max(value);
        }
        if min.is_finite() && max.is_finite() {
            return Some(vec![min, max]);
        }
        return extent(values).map(|(min, max)| vec![min, max]);
    }

    let (mut min, mut max) = (f64::INFINITY, f64::NEG_INFINITY);
    for &value in values {
        if value.is_finite() && value > 0.0 {
            min = min.min(value);
            max = max.max(value);
        }
    }
    (min <= max).then(|| vec![min, max])
}

pub(crate) fn coordinates<'p>(
    x: Option<&'p crate::data::Series<'_>>,
    len: usize,
) -> Coordinates<'p> {
    match x {
        Some(series) => Coordinates::Values(Cow::Borrowed(series.as_slice())),
        None => Coordinates::Indices(len),
    }
}

/// The finite `(min, max)` over strictly positive values, or `None` without any.
pub(crate) fn extent_positive(values: &[f64]) -> Option<(f64, f64)> {
    let mut extent: Option<(f64, f64)> = None;
    for &value in values
        .iter()
        .filter(|value| value.is_finite() && **value > 0.0)
    {
        extent = match extent {
            None => Some((value, value)),
            Some((min, max)) => Some((min.min(value), max.max(value))),
        };
    }
    extent
}

/// The finite `(min, max)` of a column, or `None` without finite values.
pub(crate) fn extent(values: &[f64]) -> Option<(f64, f64)> {
    let mut extent: Option<(f64, f64)> = None;
    for &value in values.iter().filter(|value| value.is_finite()) {
        extent = match extent {
            None => Some((value, value)),
            Some((min, max)) => Some((min.min(value), max.max(value))),
        };
    }
    extent
}

/// Unions the extents of several columns.
pub(crate) fn union(extents: impl Iterator<Item = Option<(f64, f64)>>) -> Option<(f64, f64)> {
    extents
        .flatten()
        .reduce(|(min_a, max_a), (min_b, max_b)| (min_a.min(min_b), max_a.max(max_b)))
}

#[cfg(test)]
mod tests {
    use super::{Coordinates, coordinates, extent, extent_positive, line_extent};

    #[test]
    fn implicit_coordinates_remain_symbolic() {
        let coordinates = coordinates(None, 4);
        assert!(matches!(coordinates, Coordinates::Indices(4)));
        assert_eq!(
            coordinates.iter().collect::<Vec<_>>(),
            vec![0.0, 1.0, 2.0, 3.0]
        );
        assert_eq!(coordinates.extent(), Some((0.0, 3.0)));
        assert_eq!(coordinates.extent_positive(), Some((1.0, 3.0)));
    }

    #[test]
    fn empty_and_zero_only_indices_have_no_positive_extent() {
        assert_eq!(Coordinates::Indices(0).extent(), None);
        assert_eq!(Coordinates::Indices(0).extent_positive(), None);
        assert_eq!(Coordinates::Indices(1).extent_positive(), None);
    }

    #[test]
    fn line_summaries_preserve_independent_and_log_extents() {
        let x = [-10.0, 1.0, 100.0];
        let y = [f64::NAN, 0.1, 10.0];
        let (finite_x, finite_y) =
            line_extent(Some(&x), &y, false, false).expect("each channel has finite data");
        assert_eq!(extent(&finite_x), Some((-10.0, 100.0)));
        assert_eq!(extent(&finite_y), Some((0.1, 10.0)));

        let (positive_x, positive_y) =
            line_extent(Some(&x), &y, true, true).expect("each channel has positive data");
        assert_eq!(extent_positive(&positive_x), Some((1.0, 100.0)));
        assert_eq!(extent_positive(&positive_y), Some((0.1, 10.0)));

        let (indices, _) =
            line_extent(None, &[2.0, 3.0, 4.0], false, false).expect("finite values");
        assert_eq!(extent(&indices), Some((0.0, 2.0)));
        let (indices, _) =
            line_extent(None, &[2.0, 3.0, 4.0], true, true).expect("positive values");
        assert_eq!(extent_positive(&indices), Some((1.0, 2.0)));

        let (_, finite_y) = line_extent(None, &[1.0, f64::INFINITY, 3.0], false, false)
            .expect("finite values survive an infinity");
        assert_eq!(extent(&finite_y), Some((1.0, 3.0)));
    }
}
