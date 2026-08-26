//! Resolution: marks materialized into drawable columns with resolved colors.

use std::borrow::Cow;

use crate::mark::{
    Categories, LineStyle, Mark, Orientation, Placement, PointStyle, RangePlacement, Source,
};
use crate::plot::layout::Map;
use crate::render::Color;
use crate::scale::{Colormap, Palette};

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

/// A resolved constant or data-bound color channel.
///
/// Categorical channels keep one compact identity vector beside the values.
/// The palette remains symbolic until drawing, so no per-datum color array and
/// no per-category copy of the numeric channels is needed.
pub(crate) enum ColorChannel<'p> {
    Fixed {
        color: Color,
        label: Option<&'p str>,
    },
    Categories {
        labels: &'p [String],
        ids: Cow<'p, [usize]>,
        palette: &'p Palette,
        cycle_markers: bool,
    },
}

impl<'p> ColorChannel<'p> {
    fn fixed(color: Color, label: Option<&'p str>) -> Self {
        Self::Fixed { color, label }
    }

    fn categories(
        categories: &'p Categories,
        ids: Cow<'p, [usize]>,
        palette: &'p Palette,
        cycle_markers: bool,
    ) -> Self {
        Self::Categories {
            labels: categories.labels(),
            ids,
            palette,
            cycle_markers,
        }
    }

    pub(crate) fn color(&self, index: usize) -> Color {
        match self {
            ColorChannel::Fixed { color, .. } => *color,
            ColorChannel::Categories { ids, palette, .. } => ids
                .get(index)
                .map_or(Color::Default, |&category| palette.color(category)),
        }
    }

    /// The palette color of one category, independent of any datum index.
    pub(crate) fn category_color(&self, category: usize) -> Color {
        match self {
            ColorChannel::Fixed { color, .. } => *color,
            ColorChannel::Categories { palette, .. } => palette.color(category),
        }
    }

    /// Category identity is line topology as well as paint: unequal adjacent
    /// identities must never be connected, even when palette colors wrap.
    pub(crate) fn category(&self, index: usize) -> Option<usize> {
        match self {
            ColorChannel::Fixed { .. } => None,
            ColorChannel::Categories { ids, .. } => ids.get(index).copied(),
        }
    }

    pub(crate) fn point_style(&self, index: usize, base: PointStyle) -> PointStyle {
        let category = self.category(index);
        self.point_style_for_category(category, base)
    }

    fn point_style_for_category(&self, category: Option<usize>, base: PointStyle) -> PointStyle {
        match self {
            ColorChannel::Categories {
                cycle_markers: true,
                ..
            } if base == PointStyle::Dot => category
                .map(|category| MARKER_CYCLE[category % MARKER_CYCLE.len()])
                .unwrap_or(base),
            _ => base,
        }
    }

    fn has_legend(&self) -> bool {
        match self {
            ColorChannel::Fixed { label, .. } => label.is_some(),
            ColorChannel::Categories { labels, .. } => !labels.is_empty(),
        }
    }

    fn for_each_legend_entry<'a>(
        &'a self,
        mut swatch: impl FnMut(Option<usize>) -> &'static str,
        visit: &mut impl FnMut(&'static str, Color, &'a str),
    ) {
        match self {
            ColorChannel::Fixed {
                color,
                label: Some(label),
            } => visit(swatch(None), *color, label),
            ColorChannel::Categories {
                labels, palette, ..
            } => {
                for (category, label) in labels.iter().enumerate() {
                    visit(swatch(Some(category)), palette.color(category), label);
                }
            }
            ColorChannel::Fixed { label: None, .. } => {}
        }
    }
}

/// A resolved interval body: the open/close pair, borrowed or materialized.
pub(crate) type Body<'p> = (Cow<'p, [f64]>, Cow<'p, [f64]>);

/// One layer, resolved to drawable data.
pub(crate) enum ResolvedLayer<'p> {
    Series {
        x: Coordinates<'p>,
        y: Cow<'p, [f64]>,
        color: ColorChannel<'p>,
        kind: Kind,
    },
    Bars {
        placement: &'p Placement,
        values: Cow<'p, [f64]>,
        color: ColorChannel<'p>,
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
        rgb: Option<&'p [(u8, u8, u8)]>,
        classes: Option<ColorChannel<'p>>,
        reduce: crate::stat::Reducer,
    },
    Range {
        x: Coordinates<'p>,
        bands: Option<&'p [String]>,
        low: Cow<'p, [f64]>,
        high: Cow<'p, [f64]>,
        body: Option<Body<'p>>,
        marker: Option<Cow<'p, [f64]>>,
        color: ColorChannel<'p>,
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
            ResolvedLayer::Range { x, bands, .. } => {
                if bands.is_some() {
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
                rgb,
                classes,
                ..
            } => Some(match extents {
                Some((_, y)) => *y,
                None => {
                    let count = match (classes, rgb) {
                        (Some(ColorChannel::Categories { ids, .. }), _) => ids.len(),
                        (_, Some(pixels)) => pixels.len(),
                        _ => values.len(),
                    };
                    (0.0, (count / (*columns).max(1)) as f64)
                }
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
            ResolvedLayer::Range { x, bands: None, .. } => x.extent_positive(),
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

    pub(crate) fn has_legend(&self) -> bool {
        match self {
            ResolvedLayer::Series { color, .. }
            | ResolvedLayer::Bars { color, .. }
            | ResolvedLayer::Range { color, .. } => color.has_legend(),
            ResolvedLayer::Area { label, .. } | ResolvedLayer::Rule { label, .. } => {
                label.is_some()
            }
            ResolvedLayer::Cells {
                classes: Some(channel),
                ..
            } => channel.has_legend(),
            ResolvedLayer::Text { .. } | ResolvedLayer::Cells { .. } => false,
        }
    }

    /// Visits every legend entry represented by this layer. A constant channel
    /// contributes at most one; a categorical channel contributes its stable
    /// first-appearance label table without manufacturing drawable layers.
    pub(crate) fn for_each_legend_entry<'a>(
        &'a self,
        ascii: bool,
        mut visit: impl FnMut(&'static str, Color, &'a str),
    ) {
        match self {
            ResolvedLayer::Series { color, kind, .. } => color.for_each_legend_entry(
                |category| series_swatch(kind, color, category, ascii),
                &mut visit,
            ),
            ResolvedLayer::Bars { color, .. } => color.for_each_legend_entry(
                |_| {
                    if ascii { "##" } else { "\u{2588}\u{2588}" }
                },
                &mut visit,
            ),
            ResolvedLayer::Range { color, .. } => color.for_each_legend_entry(
                |_| {
                    if ascii { "||" } else { "\u{2503}\u{2503}" }
                },
                &mut visit,
            ),
            ResolvedLayer::Area {
                color,
                label: Some(label),
                ..
            } => visit(if ascii { "##" } else { "\u{2584}\u{2584}" }, *color, label),
            ResolvedLayer::Rule {
                color,
                label: Some(label),
                ..
            } => visit(if ascii { "--" } else { "\u{2500}\u{2500}" }, *color, label),
            ResolvedLayer::Cells {
                classes: Some(channel),
                ..
            } => channel.for_each_legend_entry(class_swatch, &mut visit),
            ResolvedLayer::Area { label: None, .. }
            | ResolvedLayer::Rule { label: None, .. }
            | ResolvedLayer::Text { .. }
            | ResolvedLayer::Cells { .. } => {}
        }
    }
}

/// Legend swatches for class cells mirror the shade each class paints in the
/// grid, so plain output can match regions to names without color.
pub(crate) const CLASS_SWATCHES: [&str; 4] = [
    "\u{2591}\u{2591}",
    "\u{2592}\u{2592}",
    "\u{2593}\u{2593}",
    "\u{2588}\u{2588}",
];

fn class_swatch(category: Option<usize>) -> &'static str {
    CLASS_SWATCHES[category.unwrap_or(0) % CLASS_SWATCHES.len()]
}

fn series_swatch(
    kind: &Kind,
    color: &ColorChannel<'_>,
    category: Option<usize>,
    ascii: bool,
) -> &'static str {
    match kind {
        Kind::Line(_) if ascii => "--",
        Kind::Line(_) => "\u{2500}\u{2500}",
        Kind::Points(style) => match color.point_style_for_category(category, *style) {
            PointStyle::Dot if ascii => "..",
            PointStyle::Dot => "\u{2022}\u{2022}",
            PointStyle::Plus => "++",
            PointStyle::Cross => "xx",
            PointStyle::Asterisk => "**",
            PointStyle::Circle => "oo",
        },
    }
}

/// Materializes every layer into drawable columns plus a resolved color.
/// Functions are sampled here, once per subpixel column of the frame width.
/// A `color_by` mark remains one layer with integer category identity beside its
/// values; `cycle_markers` (plain output) cycles default point markers so the
/// categories stay separable without color.
pub(crate) fn resolve<'p>(
    marks: &'p [Mark<'_>],
    sample_width: usize,
    palette: &[Color; 6],
    categorical: &'p Palette,
    cycle_markers: bool,
    reduce: Reduce,
) -> Vec<ResolvedLayer<'p>> {
    // Annotations (rules, text) draw in the default foreground and do not
    // consume palette slots; a single data layer draws in the default too.
    let data_layers = marks
        .iter()
        .filter(|mark| !matches!(mark, Mark::Rule(_) | Mark::Text(_)))
        .count();
    let mut colors = ColorResolver {
        layer_palette: *palette,
        categorical,
        cycle_markers,
        single_data_layer: data_layers == 1,
        layer_index: 0,
    };

    marks
        .iter()
        .map(|mark| match mark {
            Mark::Line(line) => match &line.source {
                Source::Points { x, y } => {
                    let x_slice = x.as_ref().map(|series| series.as_slice());
                    if let Some(categories) = &line.color_by {
                        match reduced_categories(x_slice, y.as_slice(), categories.ids(), reduce) {
                            Some((dx, dy, ids)) => ResolvedLayer::Series {
                                x: Coordinates::Values(Cow::Owned(dx)),
                                y: Cow::Owned(dy),
                                color: colors.categories(categories, Cow::Owned(ids)),
                                kind: Kind::Line(line.style),
                            },
                            None => ResolvedLayer::Series {
                                x: coordinates(x.as_ref(), y.len()),
                                y: Cow::Borrowed(y.as_slice()),
                                color: colors
                                    .categories(categories, Cow::Borrowed(categories.ids())),
                                kind: Kind::Line(line.style),
                            },
                        }
                    } else {
                        // The aggregate-to-raster pipeline: past four points per
                        // raster column, M4 reduces the series to what the column
                        // can show. Mapped M4 buckets by the rendered column, so
                        // the reduction is pixel-exact; non-monotonic x declines.
                        let color = colors.fixed(line.color, line.label.as_deref());
                        match reduced(x_slice, y.as_slice(), reduce) {
                            Some((dx, dy)) => ResolvedLayer::Series {
                                x: Coordinates::Values(Cow::Owned(dx)),
                                y: Cow::Owned(dy),
                                color,
                                kind: Kind::Line(line.style),
                            },
                            None => ResolvedLayer::Series {
                                x: coordinates(x.as_ref(), y.len()),
                                y: Cow::Borrowed(y.as_slice()),
                                color,
                                kind: Kind::Line(line.style),
                            },
                        }
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
                        color: colors.fixed(line.color, line.label.as_deref()),
                        kind: Kind::Line(line.style),
                    }
                }
            },
            Mark::Points(points) => ResolvedLayer::Series {
                x: coordinates(points.x.as_ref(), points.y.len()),
                y: Cow::Borrowed(points.y.as_slice()),
                color: colors.channel(
                    points.color_by.as_ref(),
                    points.color,
                    points.label.as_deref(),
                ),
                kind: Kind::Points(points.style),
            },
            Mark::Bars(bars) => ResolvedLayer::Bars {
                placement: &bars.placement,
                values: Cow::Borrowed(bars.values.as_slice()),
                color: colors.channel(bars.color_by.as_ref(), bars.color, bars.label.as_deref()),
            },
            Mark::Area(area) => ResolvedLayer::Area {
                x: coordinates(area.x.as_ref(), area.high.len()),
                low: area.low.as_ref().map(|series| series.as_slice()),
                high: area.high.as_slice(),
                horizontal: area.horizontal,
                color: colors.assigned(area.color),
                label: area.label.as_deref(),
            },
            Mark::Cells(cells) => ResolvedLayer::Cells {
                columns: cells.columns,
                values: cells.values.as_slice(),
                extents: cells.extents,
                colormap: cells.colormap.clone(),
                rgb: cells.rgb.as_deref(),
                classes: cells.classes.as_ref().map(|categories| {
                    colors.categories(categories, Cow::Borrowed(categories.ids()))
                }),
                reduce: cells.reduce,
            },
            Mark::Range(range) => {
                let (x, bands) = match &range.placement {
                    RangePlacement::Numeric(x) => (coordinates(x.as_ref(), range.low.len()), None),
                    RangePlacement::Bands(categories) => (
                        Coordinates::Indices(categories.len()),
                        Some(categories.as_slice()),
                    ),
                };
                ResolvedLayer::Range {
                    x,
                    bands,
                    low: Cow::Borrowed(range.low.as_slice()),
                    high: Cow::Borrowed(range.high.as_slice()),
                    body: range.body.as_ref().map(|(low, high)| {
                        (
                            Cow::Borrowed(low.as_slice()),
                            Cow::Borrowed(high.as_slice()),
                        )
                    }),
                    marker: range.marker.as_ref().map(|m| Cow::Borrowed(m.as_slice())),
                    color: colors.channel(
                        range.color_by.as_ref(),
                        range.color,
                        range.label.as_deref(),
                    ),
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
        })
        .collect()
}

struct ColorResolver<'p> {
    layer_palette: [Color; 6],
    categorical: &'p Palette,
    cycle_markers: bool,
    single_data_layer: bool,
    layer_index: usize,
}

impl<'p> ColorResolver<'p> {
    fn assigned(&mut self, explicit: Option<Color>) -> Color {
        let index = self.layer_index;
        self.layer_index += 1;
        explicit.unwrap_or(if self.single_data_layer {
            Color::Default
        } else {
            self.layer_palette[index % self.layer_palette.len()]
        })
    }

    fn fixed(&mut self, explicit: Option<Color>, label: Option<&'p str>) -> ColorChannel<'p> {
        ColorChannel::fixed(self.assigned(explicit), label)
    }

    fn categories(&self, categories: &'p Categories, ids: Cow<'p, [usize]>) -> ColorChannel<'p> {
        ColorChannel::categories(categories, ids, self.categorical, self.cycle_markers)
    }

    fn channel(
        &mut self,
        categories: Option<&'p Categories>,
        explicit: Option<Color>,
        label: Option<&'p str>,
    ) -> ColorChannel<'p> {
        match categories {
            Some(categories) => self.categories(categories, Cow::Borrowed(categories.ids())),
            None => self.fixed(explicit, label),
        }
    }
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
            mapped_m4(x, y, map, columns)
        }
        Reduce::Mapped { .. } => None,
    }
}

/// Reduces a categorical line without converting membership into numeric gaps.
/// The extent probe deliberately borrows the raw channels: it avoids fabricating
/// category identities for independent x/y summaries, and remains linear.
fn reduced_categories(
    x: Option<&[f64]>,
    y: &[f64],
    categories: &[usize],
    reduce: Reduce,
) -> Option<(Vec<f64>, Vec<f64>, Vec<usize>)> {
    match reduce {
        Reduce::Mapped { map, columns } if y.len() > 4 * columns.max(1) => {
            mapped_m4_categories(x, y, categories, map, columns)
        }
        Reduce::None | Reduce::Extent { .. } | Reduce::Mapped { .. } => None,
    }
}

fn mapped_m4(
    x: Option<&[f64]>,
    y: &[f64],
    map: Map,
    columns: usize,
) -> Option<(Vec<f64>, Vec<f64>)> {
    match map {
        Map::Linear(linear) => match linear.finite_affine() {
            Some((start, span, output_start, output_span)) => {
                crate::stat::m4_mapped(x, y, columns, move |value| {
                    output_start + (value - start) / span * output_span
                })
            }
            None => crate::stat::m4_mapped(x, y, columns, |value| linear.map(value)),
        },
        Map::Log(linear) => match linear.finite_affine() {
            Some((start, span, output_start, output_span)) => {
                crate::stat::m4_mapped(x, y, columns, move |value| {
                    output_start + (value.log10() - start) / span * output_span
                })
            }
            None => crate::stat::m4_mapped(x, y, columns, |value| linear.map(value.log10())),
        },
    }
}

fn mapped_m4_categories(
    x: Option<&[f64]>,
    y: &[f64],
    categories: &[usize],
    map: Map,
    columns: usize,
) -> Option<(Vec<f64>, Vec<f64>, Vec<usize>)> {
    match map {
        Map::Linear(linear) => match linear.finite_affine() {
            Some((start, span, output_start, output_span)) => {
                crate::stat::m4_mapped_categories(x, y, categories, columns, move |value| {
                    output_start + (value - start) / span * output_span
                })
            }
            None => crate::stat::m4_mapped_categories(x, y, categories, columns, |value| {
                linear.map(value)
            }),
        },
        Map::Log(linear) => match linear.finite_affine() {
            Some((start, span, output_start, output_span)) => {
                crate::stat::m4_mapped_categories(x, y, categories, columns, move |value| {
                    output_start + (value.log10() - start) / span * output_span
                })
            }
            None => crate::stat::m4_mapped_categories(x, y, categories, columns, |value| {
                linear.map(value.log10())
            }),
        },
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
    use std::borrow::Cow;

    use super::{
        ColorChannel, Coordinates, Reduce, ResolvedLayer, coordinates, extent, extent_positive,
        line_extent, resolve,
    };
    use crate::mark::{Mark, Points};
    use crate::render::Color;
    use crate::scale::Palette;

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

    #[test]
    fn unique_categories_remain_one_borrowed_numeric_layer() {
        let values: Vec<f64> = (0..10_000).map(f64::from).collect();
        let labels: Vec<String> = (0..values.len()).map(|index| format!("g{index}")).collect();
        let marks = vec![Mark::from(Points::y(&values[..]).color_by(labels))];
        let layer_palette = [Color::Red; 6];
        let categorical = Palette::default();

        let layers = resolve(
            &marks,
            80,
            &layer_palette,
            &categorical,
            false,
            Reduce::None,
        );
        assert_eq!(layers.len(), 1, "categories must not manufacture layers");
        let ResolvedLayer::Series { y, color, .. } = &layers[0] else {
            panic!("points resolve to a series")
        };
        assert!(matches!(y, Cow::Borrowed(_)));
        let ColorChannel::Categories { labels, ids, .. } = color else {
            panic!("the categorical channel stays explicit")
        };
        assert_eq!(labels.len(), values.len());
        assert_eq!(ids.len(), values.len());
    }
}
