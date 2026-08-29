//! Mark drawing: resolved layers rasterized onto a canvas through the layout.
//!
//! Generic over [`Canvas`] and monomorphized per target: the same mark code draws
//! glyph cells and (under the `pixel` feature) device pixels. Anything
//! precision-dependent — bar fills, marker crossbars, Cells patches — goes through
//! the canvas's mid-level ops, so each target renders it at its own fidelity.

use crate::mark::{Dash, LineStyle, PointStyle};
use crate::mark::{Orientation, Placement};
use crate::plot::layout::{Layout, Map};
use crate::plot::resolve::{
    ColorChannel, Coordinates, Kind, ResolvedLayer, extent, extent_positive,
};
use crate::render::{Canvas, Charset, Color, PlotRect, PointShape};
use crate::scale::{Band, Colormap};
use crate::stat::{Reducer, ReducerState};

/// Draws every resolved layer, in order, through the shared scales.
pub(crate) fn layers<C: Canvas>(
    surface: &mut C,
    layout: &Layout<'_>,
    layers: &[ResolvedLayer<'_>],
) {
    let Layout {
        px,
        py,
        gutter,
        plot_top,
        plot_rows,
        plot_cols,
        plot_sub_w,
        plot_sub_h,
        x_offset,
        y_offset,
        ..
    } = *layout;
    let x_scale = &layout.x_scale;
    let y_scale = &layout.y_scale;
    let band = &layout.band;
    let rect = PlotRect {
        gutter,
        top: plot_top,
        columns: plot_cols,
        rows: plot_rows,
    };
    // Confine every mark to the plot rectangle: ink that maps outside the data
    // region (out-of-domain points, overshooting bars, distant finite values) is
    // clipped here rather than escaping into the axes, gutter, or a neighbor in a
    // grid. Chrome has already been drawn, unclipped.
    surface.set_clip(
        x_offset as i64,
        y_offset as i64,
        x_offset as i64 + plot_sub_w as i64,
        y_offset as i64 + plot_sub_h as i64,
    );
    for layer in layers {
        match layer {
            ResolvedLayer::Series {
                x,
                y,
                color,
                kind:
                    Kind::Line {
                        style: LineStyle::Corners,
                        ..
                    },
                ..
            } => {
                draw_corners(surface, layout, x, y, color);
            }
            ResolvedLayer::Series {
                x, y, color, kind, ..
            } => {
                draw_series(
                    surface,
                    kind,
                    x,
                    y,
                    color,
                    x_scale,
                    y_scale,
                    (x_offset, y_offset),
                );
            }
            ResolvedLayer::Area {
                x,
                low,
                high,
                horizontal,
                color,
                opacity,
                ..
            } => {
                surface.set_opacity(*opacity);
                draw_area(
                    surface,
                    x,
                    *low,
                    high,
                    *horizontal,
                    *color,
                    x_scale,
                    y_scale,
                    (x_offset, y_offset),
                    (plot_sub_w, plot_sub_h),
                );
                surface.set_opacity(1.0);
            }
            ResolvedLayer::Cells {
                columns,
                values,
                extents,
                colormap,
                rgb,
                classes,
                reduce,
                smooth,
            } => {
                draw_cells(
                    surface,
                    *columns,
                    (values, *rgb, classes.as_ref()),
                    *extents,
                    colormap.clone(),
                    *reduce,
                    *smooth,
                    x_scale,
                    y_scale,
                    (band.as_ref(), layout.y_band.as_ref()),
                    rect,
                    (px, py),
                );
            }
            ResolvedLayer::Range {
                x,
                low,
                high,
                body,
                marker,
                color,
                ..
            } => {
                let half_width = match &band {
                    Some(band) => band.bandwidth() * 0.3,
                    None => px as f64,
                };
                draw_ranges(
                    surface,
                    x,
                    low,
                    high,
                    body.as_ref().map(|(lo, hi)| (lo.as_ref(), hi.as_ref())),
                    marker.as_deref(),
                    color,
                    x_scale,
                    y_scale,
                    (x_offset, y_offset),
                    half_width,
                );
            }
            ResolvedLayer::Rule {
                orientation,
                color,
                dash,
                ..
            } => match orientation {
                Orientation::Horizontal(y) => {
                    let sy = y_offset + y_scale.map(*y);
                    dash_segment(
                        surface,
                        (x_offset, sy),
                        (x_offset + (plot_sub_w - 1) as f64, sy),
                        *color,
                        *dash,
                        &mut 0.0,
                    );
                }
                Orientation::Vertical(x) => {
                    let sx = x_offset + x_scale.map(*x);
                    dash_segment(
                        surface,
                        (sx, y_offset),
                        (sx, y_offset + (plot_sub_h - 1) as f64),
                        *color,
                        *dash,
                        &mut 0.0,
                    );
                }
            },
            ResolvedLayer::Text { x, y, text, color } => {
                let sx = x_offset + x_scale.map(*x);
                let sy = y_offset + y_scale.map(*y);
                if sx.is_finite() && sy.is_finite() {
                    surface.note(sx, sy, (px as f64, py as f64), text, *color);
                }
            }
            ResolvedLayer::Bars {
                placement,
                values,
                base,
                color,
            } => match placement {
                Placement::Bands(_) => {
                    if let Some(band) = &band {
                        draw_bars(
                            surface,
                            &|index| {
                                (
                                    band.position(index),
                                    band.position(index) + band.bandwidth(),
                                )
                            },
                            y_scale,
                            values,
                            *base,
                            color,
                            rect,
                        );
                    }
                }
                Placement::Spans { start, width } => {
                    draw_bars(
                        surface,
                        &|index| {
                            let left = x_scale.map(width.mul_add(index as f64, *start));
                            let right = x_scale.map(width.mul_add((index + 1) as f64, *start));
                            (left, right)
                        },
                        y_scale,
                        values,
                        *base,
                        color,
                        rect,
                    );
                }
                Placement::At { x, width } => {
                    let positions = x.as_slice();
                    let half = width / 2.0;
                    draw_bars(
                        surface,
                        &|index| {
                            let center = positions.get(index).copied().unwrap_or(f64::NAN);
                            (x_scale.map(center - half), x_scale.map(center + half))
                        },
                        y_scale,
                        values,
                        *base,
                        color,
                        rect,
                    );
                }
            },
        }
    }
    surface.clear_clip();
}

/// Draws one segment in the layer's stroke pattern, advancing `phase` by
/// the segment's length so the pattern flows through polyline joints. The
/// unit scales with the target's vertical sampling density, keeping the
/// rhythm proportional across braille cells and retina pixels.
pub(crate) fn dash_segment<C: Canvas>(
    surface: &mut C,
    from: (f64, f64),
    to: (f64, f64),
    color: Color,
    dash: Dash,
    phase: &mut f64,
) {
    if dash == Dash::Solid {
        surface.line(from, to, color);
        return;
    }
    let unit = (surface.patch_density().1 as f64 / 8.0).max(1.0);
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let length = (dx * dx + dy * dy).sqrt();
    if !length.is_finite() {
        return;
    }
    let at = |distance: f64| {
        if length > 0.0 {
            let t = distance / length;
            (from.0 + dx * t, from.1 + dy * t)
        } else {
            from
        }
    };
    match dash {
        Dash::Solid => unreachable!("solid returned above"),
        Dash::Dashed => {
            let (on, period) = (3.0 * unit, 5.5 * unit);
            let mut position = 0.0;
            while position < length {
                let offset = (*phase + position).rem_euclid(period);
                if offset < on {
                    let run = (length - position).min(on - offset);
                    surface.line(at(position), at(position + run), color);
                    position += run;
                } else {
                    position += period - offset;
                }
            }
        }
        Dash::Dotted => {
            let period = 3.0 * unit;
            let mut position = (period - phase.rem_euclid(period)).rem_euclid(period);
            while position <= length {
                let dot = at(position);
                surface.line(dot, dot, color);
                position += period;
            }
        }
    }
    *phase += length;
}

#[allow(clippy::too_many_arguments)]
fn draw_series<C: Canvas>(
    surface: &mut C,
    kind: &Kind,
    x: &Coordinates<'_>,
    y: &[f64],
    color: &ColorChannel<'_>,
    x_scale: &Map,
    y_scale: &Map,
    offset: (f64, f64),
) {
    match kind {
        Kind::Line {
            style: LineStyle::Corners,
            ..
        } => {
            unreachable!("corners are drawn by draw_corners");
        }
        Kind::Line {
            style: LineStyle::Pixels,
            glow,
            dash,
        } => {
            let mut previous: Option<((f64, f64), Option<usize>)> = None;
            let mut phase = 0.0;
            for (index, (xv, &yv)) in x.iter().zip(y.iter()).enumerate() {
                if !xv.is_finite() || !yv.is_finite() {
                    previous = None;
                    continue;
                }
                let position = (offset.0 + x_scale.map(xv), offset.1 + y_scale.map(yv));
                // A finite value can still map to nothing — a non-positive value on
                // a log axis. That is a gap, not a point to connect across.
                if !position.0.is_finite() || !position.1.is_finite() {
                    previous = None;
                    continue;
                }
                let category = color.category(index);
                let ink = color.color(index);
                match previous {
                    Some((from, previous_category)) if previous_category == category => {
                        if *glow {
                            surface.glow(from, position, ink);
                        }
                        dash_segment(surface, from, position, ink, *dash, &mut phase);
                    }
                    // A zero-length stroke: the round cap makes a gap-isolated
                    // point (or the first point of a NaN-jointed segment, as
                    // the contour preset emits) a stroke-weight dot — the
                    // marker pen would bead every joint on pixel targets.
                    _ => surface.line(position, position, ink),
                }
                previous = Some((position, category));
            }
        }
        Kind::Points {
            style,
            opacity,
            density,
        } => {
            surface.set_opacity(*opacity);
            surface.set_accumulate(*density);
            for (index, (xv, &yv)) in x.iter().zip(y.iter()).enumerate() {
                if xv.is_finite() && yv.is_finite() {
                    surface.point(
                        offset.0 + x_scale.map(xv),
                        offset.1 + y_scale.map(yv),
                        point_shape(color.point_style(index, *style)),
                        color.color(index),
                    );
                }
            }
            surface.set_accumulate(false);
            surface.set_opacity(1.0);
        }
    }
}

fn point_shape(style: PointStyle) -> PointShape {
    match style {
        PointStyle::Dot => PointShape::Dot,
        PointStyle::Plus => PointShape::Plus,
        PointStyle::Cross => PointShape::Cross,
        PointStyle::Asterisk => PointShape::Asterisk,
        PointStyle::Circle => PointShape::Circle,
    }
}

/// Draws one line layer in the asciichart style: one box-drawing glyph per cell
/// column — `─` when flat, `╭╮╰╯` elbows joined by `│` runs when the line moves.
/// The polyline is sampled at each column's center; gaps skip columns.
///
/// Cell-glyph art by construction: pixel targets substitute [`LineStyle::Pixels`]
/// before drawing, so this only ever runs on a glyph canvas.
fn draw_corners<C: Canvas>(
    surface: &mut C,
    layout: &Layout<'_>,
    x: &Coordinates<'_>,
    y: &[f64],
    color: &ColorChannel<'_>,
) {
    let Layout {
        px,
        py,
        gutter,
        plot_top,
        plot_rows,
        plot_cols,
        x_offset,
        y_offset,
        ..
    } = *layout;
    let ascii = layout.charset == Charset::Ascii;
    let (flat, vertical, down_out, down_in, up_out, up_in) = if ascii {
        ("-", "|", "+", "+", "+", "+")
    } else {
        // Falling: leave right-down `╮`, arrive down-right `╰`.
        // Rising: leave right-up `╯`… drawn from the left: `╰` opens up-right.
        (
            "\u{2500}", "\u{2502}", "\u{256E}", "\u{2570}", "\u{256F}", "\u{256D}",
        )
    };

    // The line's row at each cell column, sampled at the column center.
    let mut rows: Vec<Option<(i64, Option<usize>, Color)>> = vec![None; plot_cols];
    let mut previous: Option<(f64, f64, Option<usize>)> = None;
    for (index, (xv, &yv)) in x.iter().zip(y).enumerate() {
        if !xv.is_finite() || !yv.is_finite() {
            previous = None;
            continue;
        }
        let sx = x_offset + layout.x_scale.map(xv);
        let sy = y_offset + layout.y_scale.map(yv);
        if !sx.is_finite() || !sy.is_finite() {
            previous = None;
            continue;
        }
        let category = color.category(index);
        let ink = color.color(index);
        if let Some((px_, py_, previous_category)) = previous
            && previous_category == category
        {
            let (from, to) = if px_ <= sx { (px_, sx) } else { (sx, px_) };
            let span = sx - px_;
            let first = (from / px as f64 - 0.5).ceil().max(0.0) as usize;
            let last = ((to / px as f64 - 0.5).floor() as usize).min(plot_cols.saturating_sub(1));
            if first <= last {
                for (offset, slot) in rows[first..=last].iter_mut().enumerate() {
                    let center = ((first + offset) as f64 + 0.5) * px as f64;
                    let t = if span.abs() < f64::EPSILON {
                        0.0
                    } else {
                        ((center - px_) / span).clamp(0.0, 1.0)
                    };
                    let sub_y = py_ + (sy - py_) * t;
                    let row = (sub_y / py as f64).floor() as i64 - plot_top as i64;
                    if (0..plot_rows as i64).contains(&row) {
                        *slot = Some((row + plot_top as i64, category, ink));
                    }
                }
            }
        } else {
            let column = (sx / px as f64 - 0.5).round() as i64;
            if (0..plot_cols as i64).contains(&column) {
                let row = (sy / py as f64).floor() as i64;
                rows[column as usize] = Some((row, category, ink));
            }
        }
        previous = Some((sx, sy, category));
    }

    for column in 0..plot_cols {
        let Some((row, category, ink)) = rows[column] else {
            continue;
        };
        let cell = (gutter + column) as i64;
        let next = if column + 1 < plot_cols {
            rows[column + 1]
        } else {
            None
        };
        match next {
            Some((next_row, next_category, _)) if next_category == category && next_row == row => {
                surface.text(cell, row, flat, ink);
            }
            Some((next_row, next_category, _)) if next_category == category && next_row > row => {
                // The line falls: leave rightward-down, fill, arrive.
                surface.text(cell, row, down_out, ink);
                for between in row + 1..next_row {
                    surface.text(cell, between, vertical, ink);
                }
                surface.text(cell, next_row, down_in, ink);
            }
            Some((next_row, next_category, _)) if next_category == category => {
                // The line rises: leave rightward-up, fill, arrive.
                surface.text(cell, row, up_out, ink);
                for between in next_row + 1..row {
                    surface.text(cell, between, vertical, ink);
                }
                surface.text(cell, next_row, up_in, ink);
            }
            Some(_) | None => {
                surface.text(cell, row, flat, ink);
            }
        }
    }
}

/// Draws one bars layer: per bar, the canvas fills the span from the baseline —
/// zero, or the bar's own base — to the value end at its own precision
/// (eighth-block ramps on cells, exact rectangles on pixels).
fn draw_bars<C: Canvas>(
    surface: &mut C,
    span: &dyn Fn(usize) -> (f64, f64),
    y_scale: &Map,
    values: &[f64],
    base: Option<&[f64]>,
    color: &ColorChannel<'_>,
    rect: PlotRect,
) {
    let zero = y_scale.map(0.0);
    for (index, &value) in values.iter().enumerate() {
        if !value.is_finite() || value == 0.0 {
            continue;
        }
        // A gap (`NaN`) in the base skips the bar, like a gap in the values.
        let start = match base {
            Some(base) => match base.get(index) {
                Some(&start) if start.is_finite() => start,
                _ => continue,
            },
            None => 0.0,
        };
        let (left_sub, right_sub) = span(index);
        if !left_sub.is_finite() || !right_sub.is_finite() {
            continue;
        }
        let baseline = if start == 0.0 {
            zero
        } else {
            y_scale.map(start)
        };
        surface.bar(
            (left_sub, right_sub),
            y_scale.map(start + value),
            baseline,
            value > 0.0,
            rect,
            color.color(index),
        );
    }
}

/// Draws one range layer: per interval, a thin capped whisker from `low` to
/// `high`, an optional thick body (filled with vertical subpixel runs, like areas),
/// and an optional marker crossbar drawn through the canvas so it stays visible
/// over the fill on every target.
#[allow(clippy::too_many_arguments)]
fn draw_ranges<C: Canvas>(
    surface: &mut C,
    x: &Coordinates<'_>,
    low: &[f64],
    high: &[f64],
    body: Option<(&[f64], &[f64])>,
    marker: Option<&[f64]>,
    color: &ColorChannel<'_>,
    x_scale: &Map,
    y_scale: &Map,
    offset: (f64, f64),
    half_width: f64,
) {
    let cap = (half_width * 0.6).max(1.0);
    // Bound to the shortest required channel: constructors keep these equal, but a
    // deserialized range can arrive ragged, and rendering must not index past an end.
    for (index, ((xv, &lv), &hv)) in x.iter().zip(low).zip(high).enumerate() {
        if !xv.is_finite() || !lv.is_finite() || !hv.is_finite() {
            continue;
        }
        let sx = offset.0 + x_scale.map(xv);
        let sl = offset.1 + y_scale.map(lv);
        let sh = offset.1 + y_scale.map(hv);
        let ink = color.color(index);
        // The whisker and its caps.
        surface.line((sx, sl), (sx, sh), ink);
        surface.line((sx - cap, sl), (sx + cap, sl), ink);
        surface.line((sx - cap, sh), (sx + cap, sh), ink);
        // The body: vertical subpixel runs across the width.
        if let Some((body_low, body_high)) = body {
            let (Some(&bl), Some(&bh)) = (body_low.get(index), body_high.get(index)) else {
                continue;
            };
            if bl.is_finite() && bh.is_finite() {
                let sbl = offset.1 + y_scale.map(bl);
                let sbh = offset.1 + y_scale.map(bh);
                let from = (sx - half_width).round() as i64;
                let to = (sx + half_width).round() as i64;
                for column in from..=to {
                    surface.line((column as f64, sbl), (column as f64, sbh), ink);
                }
            }
        }
        // The marker crossbar: the canvas keeps it readable over the fill.
        if let Some(marker) = marker {
            let Some(&mv) = marker.get(index) else {
                continue;
            };
            if mv.is_finite() {
                let sy = offset.1 + y_scale.map(mv);
                surface.marker(sx, half_width, sy, ink);
            }
        }
    }
}

/// Draws one cells layer: every patch of the canvas's sampling grid owns the
/// grid cells whose centers fall inside it (adjacent patches partition the
/// centers), and shows the layer's reduction over them — the mean box filter
/// by default, [`Cells::reduce`](crate::Cells::reduce) to choose — so a grid
/// denser than the raster is summarized bucket-exactly, never sampled. A patch
/// owning no center shows the single cell containing its center. Gaps stay
/// blank.
///
/// An rgb grid reduces by per-channel mean and shows its luma as the plain
/// shade. A classes grid reduces to the modal class (ties to the lowest id)
/// and keeps a stable shade per class, mirrored by its legend swatches.
///
/// `CellChannels` is the mark's one populated grid: values, pixels, or classes.
///
/// On a band axis the grid index is the band containing the patch — cell k fills
/// band k exactly, top-down on y, and the padding between bands stays blank — so
/// a labeled matrix reads as discrete categories, like the labels say.
#[allow(clippy::too_many_arguments)]
fn draw_cells<C: Canvas>(
    surface: &mut C,
    columns: usize,
    channels: CellChannels<'_, '_>,
    extents: Option<((f64, f64), (f64, f64))>,
    colormap: Colormap,
    reduce: Reducer,
    smooth: bool,
    x_scale: &Map,
    y_scale: &Map,
    bands: (Option<&Band>, Option<&Band>),
    rect: PlotRect,
    density: (usize, usize),
) {
    let (px, py) = density;
    let (values, rgb, classes) = channels;
    let (x_band, y_band) = bands;
    let count = match (classes, rgb) {
        (Some(ColorChannel::Categories { ids, .. }), _) => ids.len(),
        (_, Some(pixels)) => pixels.len(),
        _ => values.len(),
    };
    let rows = count / columns.max(1);
    if rows == 0 {
        return;
    }
    // A log ramp positions by decade over the positive values; everything at
    // or below zero is a gap and must not stretch the ramp. An rgb or classes
    // grid has no value scale at all.
    let range = match (rgb, classes) {
        (Some(_), _) | (_, Some(_)) => None,
        (None, None) => {
            let observed = if colormap.is_log() {
                extent_positive(values)
            } else {
                extent(values)
            };
            let Some(observed) = observed else {
                return;
            };
            Some(observed)
        }
    };
    let ((x0, x1), (y0, y1)) = extents.unwrap_or(((0.0, columns as f64), (0.0, rows as f64)));
    let (samples_x, samples_y) = surface.patch_density();
    if samples_x == 0 || samples_y == 0 {
        return;
    }
    let units_x = rect.columns * samples_x;
    let units_y = rect.rows * samples_y;

    // The center-partition ranges, one per patch column and per patch row.
    // Band axes map patches to bands directly and never reduce.
    let column_ranges: Option<Vec<(usize, usize)>> = x_band.is_none().then(|| {
        (0..units_x)
            .map(|unit| {
                let left = unit as f64 * px as f64 / samples_x as f64;
                let right = (unit + 1) as f64 * px as f64 / samples_x as f64;
                cell_range(x_scale, (left, right), (x0, x1), columns)
            })
            .collect()
    });
    let row_ranges: Option<Vec<(usize, usize)>> = y_band.is_none().then(|| {
        (0..units_y)
            .map(|unit| {
                let top = unit as f64 * py as f64 / samples_y as f64;
                let bottom = (unit + 1) as f64 * py as f64 / samples_y as f64;
                cell_range(y_scale, (top, bottom), (y0, y1), rows)
            })
            .collect()
    });
    // Modal-class scratch, reused across patches.
    let class_count = match classes {
        Some(ColorChannel::Categories { labels, .. }) => labels.len(),
        _ => 0,
    };
    let mut votes = vec![0u32; class_count];
    let mut touched: Vec<usize> = Vec::new();

    for unit_row in 0..units_y {
        for unit_col in 0..units_x {
            // The data position at this patch's center, via the shared scales'
            // subpixel geometry — the fallback when the patch owns no center.
            let sub_x = (unit_col as f64 + 0.5) * px as f64 / samples_x as f64;
            let sub_y = (unit_row as f64 + 0.5) * py as f64 / samples_y as f64;
            let sample = (|| {
                let (c0, c1) = match (x_band, &column_ranges) {
                    (Some(band), _) => {
                        let column = band.index_at(sub_x).filter(|&index| index < columns)?;
                        (column, column + 1)
                    }
                    (None, Some(ranges)) => {
                        let (start, end) = ranges[unit_col];
                        if start < end {
                            (start, end)
                        } else {
                            let fx = position_on(x_scale, sub_x, x0, x1)?;
                            let column =
                                (crate::numeric::inverse_lerp(x0, x1, fx) * columns as f64).floor();
                            if !(0.0..columns as f64).contains(&column) {
                                return None;
                            }
                            let column = column as usize;
                            (column, column + 1)
                        }
                    }
                    (None, None) => unreachable!("a bandless axis precomputes its ranges"),
                };
                let (r0, r1) = match (y_band, &row_ranges) {
                    (Some(band), _) => {
                        let row = band.index_at(sub_y).filter(|&index| index < rows)?;
                        (row, row + 1)
                    }
                    (None, Some(ranges)) => {
                        let (start, end) = ranges[unit_row];
                        if start < end {
                            (start, end)
                        } else {
                            let fy = position_on(y_scale, sub_y, y0, y1)?;
                            let row =
                                (crate::numeric::inverse_lerp(y0, y1, fy) * rows as f64).floor();
                            if !(0.0..rows as f64).contains(&row) {
                                return None;
                            }
                            let row = row as usize;
                            (row, row + 1)
                        }
                    }
                    (None, None) => unreachable!("a bandless axis precomputes its ranges"),
                };
                if let Some(channel) = classes {
                    // The modal class of the covered cells, ties to the lowest
                    // id, painted with its stable shade and palette color.
                    touched.clear();
                    for row in r0..r1 {
                        for column in c0..c1 {
                            if let Some(id) = channel.category(row * columns + column)
                                && id < votes.len()
                            {
                                if votes[id] == 0 {
                                    touched.push(id);
                                }
                                votes[id] += 1;
                            }
                        }
                    }
                    let mut winner: Option<(u32, usize)> = None;
                    for &id in &touched {
                        let better = winner.is_none_or(|(count, best)| {
                            votes[id] > count || (votes[id] == count && id < best)
                        });
                        if better {
                            winner = Some((votes[id], id));
                        }
                    }
                    for &id in &touched {
                        votes[id] = 0;
                    }
                    let (_, id) = winner?;
                    let intensity = ((id % 4) as f64 + 0.5) / 4.0;
                    return Some((intensity, channel.category_color(id)));
                }
                if let Some(pixels) = rgb {
                    // The box filter: per-channel mean of the covered pixels.
                    let (mut r, mut g, mut b, mut n) = (0u64, 0u64, 0u64, 0u64);
                    for row in r0..r1 {
                        for column in c0..c1 {
                            if let Some(&(pr, pg, pb)) = pixels.get(row * columns + column) {
                                r += u64::from(pr);
                                g += u64::from(pg);
                                b += u64::from(pb);
                                n += 1;
                            }
                        }
                    }
                    if n == 0 {
                        return None;
                    }
                    let (r, g, b) = (
                        ((r + n / 2) / n) as u8,
                        ((g + n / 2) / n) as u8,
                        ((b + n / 2) / n) as u8,
                    );
                    return Some((luma(r, g, b), Color::Rgb(r, g, b)));
                }
                let (low, high) = range?;
                if smooth && x_band.is_none() && y_band.is_none() && c1 - c0 <= 1 && r1 - r0 <= 1 {
                    // Inside one bucket: bilinear over the four surrounding
                    // bucket centers turns the upsampled grid into a
                    // continuous field. A non-finite neighbor falls back to
                    // the nearest bucket, keeping gaps honest.
                    let fx =
                        crate::numeric::inverse_lerp(x0, x1, position_on(x_scale, sub_x, x0, x1)?)
                            * columns as f64
                            - 0.5;
                    let fy =
                        crate::numeric::inverse_lerp(y0, y1, position_on(y_scale, sub_y, y0, y1)?)
                            * rows as f64
                            - 0.5;
                    let cx = fx.floor().clamp(0.0, (columns - 1) as f64);
                    let cy = fy.floor().clamp(0.0, (rows - 1) as f64);
                    let (tx, ty) = ((fx - cx).clamp(0.0, 1.0), (fy - cy).clamp(0.0, 1.0));
                    let (cx, cy) = (cx as usize, cy as usize);
                    let at = |column: usize, row: usize| -> Option<f64> {
                        values
                            .get(row.min(rows - 1) * columns + column.min(columns - 1))
                            .copied()
                            .filter(|value| value.is_finite())
                    };
                    let corners = (
                        at(cx, cy),
                        at(cx + 1, cy),
                        at(cx, cy + 1),
                        at(cx + 1, cy + 1),
                    );
                    if let (Some(v00), Some(v10), Some(v01), Some(v11)) = corners {
                        let value = v00 * (1.0 - tx) * (1.0 - ty)
                            + v10 * tx * (1.0 - ty)
                            + v01 * (1.0 - tx) * ty
                            + v11 * tx * ty;
                        let position = colormap.position_in(value, low, high);
                        if position.is_finite() {
                            return Some((position, colormap.color(position)));
                        }
                        return None;
                    }
                }
                let mut state = ReducerState::new(reduce);
                for row in r0..r1 {
                    for column in c0..c1 {
                        if let Some(&value) = values.get(row * columns + column) {
                            state.add(value);
                        }
                    }
                }
                let value = state.finish();
                if !value.is_finite() {
                    return None;
                }
                let position = colormap.position_in(value, low, high);
                if !position.is_finite() {
                    return None;
                }
                Some((position, colormap.color(position)))
            })();
            surface.patch(unit_col, unit_row, rect, sample);
        }
    }
}

/// The half-open range of grid cells whose centers a patch owns on one axis:
/// the patch's subpixel edges inverted through the scale into fractional cell
/// indices, then the centers strictly between them. Monotone maps make
/// adjacent patches partition the centers; `(0, 0)` means the patch owns none
/// and falls back to sampling the cell at its center.
fn cell_range(scale: &Map, edges: (f64, f64), extent: (f64, f64), count: usize) -> (usize, usize) {
    let index_at = |sub: f64| -> f64 {
        crate::numeric::inverse_lerp(extent.0, extent.1, scale.unmap(sub)) * count as f64
    };
    let (a, b) = (index_at(edges.0), index_at(edges.1));
    let (first, last) = if a <= b { (a, b) } else { (b, a) };
    let start = (first - 0.5).ceil().max(0.0);
    let end = (last - 0.5).ceil().clamp(0.0, count as f64);
    if !(start.is_finite() && end.is_finite()) || start >= end {
        return (0, 0);
    }
    (start as usize, end as usize)
}

/// The one populated grid channel of a cells layer, as borrowed slices.
type CellChannels<'a, 'p> = (
    &'a [f64],
    Option<&'a [(u8, u8, u8)]>,
    Option<&'a ColorChannel<'p>>,
);

/// Rec. 709 luma of a gamma-encoded pixel, normalized to `[0, 1]` — the shade
/// an rgb cell shows where color is unavailable.
fn luma(r: u8, g: u8, b: u8) -> f64 {
    (0.2126 * f64::from(r) + 0.7152 * f64::from(g) + 0.0722 * f64::from(b)) / 255.0
}

/// Inverts a scale at a subpixel position, returning the data value if it lands
/// inside `[lo, hi]`.
fn position_on(scale: &Map, sub: f64, lo: f64, hi: f64) -> Option<f64> {
    if lo == hi {
        return None;
    }
    let value = scale.unmap(sub);
    let t = crate::numeric::inverse_lerp(lo, hi, value);
    if !(0.0..1.0).contains(&t) {
        return None;
    }
    Some(value)
}

/// Draws one area layer: for every subpixel column a segment covers, a vertical
/// run between its interpolated low and high edges — solid in every charset, with
/// subpixel edge precision.
#[allow(clippy::too_many_arguments)]
fn draw_area<C: Canvas>(
    surface: &mut C,
    channel: &Coordinates<'_>,
    low: Option<&[f64]>,
    high: &[f64],
    horizontal: bool,
    color: Color,
    x_scale: &Map,
    y_scale: &Map,
    offset: (f64, f64),
    bounds: (usize, usize),
) {
    // In the vertical (default) orientation the channel is x and fills run in y;
    // horizontally the channel is y and fills run in x. `place` restores raster
    // coordinates from (main, cross).
    let place = |main: f64, cross: f64| -> (f64, f64) {
        if horizontal {
            (cross, main)
        } else {
            (main, cross)
        }
    };
    // The main axis runs along x for vertical areas, y for horizontal ones; its
    // frame-absolute subpixel extent bounds the fill loop so distant points
    // cannot spin it. `main` carries the plot offset, so the bounds must too —
    // clamping to the plot-relative width alone cut every fill short of the
    // plot's far edge by exactly the gutter.
    let main_offset = (if horizontal { offset.1 } else { offset.0 }).round() as i64;
    let main_limit = main_offset + if horizontal { bounds.1 } else { bounds.0 } as i64;
    let mut previous: Option<(f64, f64, f64)> = None;
    // Constructors keep channel and edges equal length; a deserialized area may not,
    // so bound to the shortest and read the optional low edge by index.
    for (index, (cv, &hv)) in channel.iter().zip(high).enumerate() {
        let lv = match low {
            Some(low) => match low.get(index) {
                Some(&value) => value,
                None => {
                    previous = None;
                    continue;
                }
            },
            None => 0.0,
        };
        if !cv.is_finite() || !hv.is_finite() || !lv.is_finite() {
            previous = None;
            continue;
        }
        let (main, cross_low, cross_high) = if horizontal {
            (
                offset.1 + y_scale.map(cv),
                offset.0 + x_scale.map(lv),
                offset.0 + x_scale.map(hv),
            )
        } else {
            (
                offset.0 + x_scale.map(cv),
                offset.1 + y_scale.map(lv),
                offset.1 + y_scale.map(hv),
            )
        };
        match previous {
            Some((pm, pl, ph)) => {
                let (from, to) = if pm <= main { (pm, main) } else { (main, pm) };
                let span = main - pm;
                let lo = (from.round() as i64).max(main_offset);
                let hi = (to.round() as i64).min(main_limit - 1);
                for step in lo..=hi {
                    let t = if span.abs() < f64::EPSILON {
                        0.0
                    } else {
                        ((step as f64 - pm) / span).clamp(0.0, 1.0)
                    };
                    let step_low = pl + (cross_low - pl) * t;
                    let step_high = ph + (cross_high - ph) * t;
                    surface.line(
                        place(step as f64, step_low),
                        place(step as f64, step_high),
                        color,
                    );
                }
            }
            None => surface.line(place(main, cross_low), place(main, cross_high), color),
        }
        previous = Some((main, cross_low, cross_high));
    }
}

#[cfg(test)]
mod tests {
    use super::{Map, cell_range, position_on};

    #[test]
    fn cell_ranges_partition_the_centers() {
        // Every cell center must be owned by exactly one patch, on linear and
        // log scales alike — the bucket-exactness invariant.
        for (scale, extent) in [
            (Map::build((0.0, 100.0), (0.0, 239.0), false), (0.0, 100.0)),
            (Map::build((1.0, 1000.0), (0.0, 239.0), true), (1.0, 1000.0)),
            // The raster flip: y scales run backwards.
            (Map::build((0.0, 100.0), (239.0, 0.0), false), (0.0, 100.0)),
        ] {
            let mut owned = vec![0usize; 1000];
            for unit in 0..240 {
                let (start, end) =
                    cell_range(&scale, (unit as f64, (unit + 1) as f64), extent, 1000);
                for slot in &mut owned[start..end] {
                    *slot += 1;
                }
            }
            // The last patch edge (240) sits past the range end (239), so the
            // sweep covers every center; none may be double-counted.
            assert!(
                owned.iter().all(|&count| count == 1),
                "centers owned {:?}",
                owned
                    .iter()
                    .enumerate()
                    .filter(|(_, c)| **c != 1)
                    .take(5)
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn upscaled_patches_own_no_centers() {
        // 4 cells across 240 subpixels: almost every patch owns none and
        // falls back to sampling; exactly 4 own one center each.
        let scale = Map::build((0.0, 4.0), (0.0, 239.0), false);
        let mut owners = 0;
        for unit in 0..240 {
            let (start, end) = cell_range(&scale, (unit as f64, (unit + 1) as f64), (0.0, 4.0), 4);
            owners += end - start;
        }
        assert_eq!(owners, 4);
    }

    #[test]
    fn cells_invert_log_scales_in_logarithmic_data_space() {
        let log = Map::build((1.0, 1000.0), (0.0, 3.0), true);
        let sampled = position_on(&log, 1.0, 1.0, 1000.0).expect("inside extent");
        assert!((sampled - 10.0).abs() < 1e-12, "sampled {sampled}");

        // Linear interpolation in data space would sample 334 here. The true
        // inverse keeps the first linearly-spaced heatmap cell wide on a log axis.
        let column = ((sampled - 1.0) / 999.0 * 3.0).floor() as usize;
        assert_eq!(column, 0);

        let reversed = Map::build((1.0, 1000.0), (3.0, 0.0), true);
        let sampled = position_on(&reversed, 2.0, 1.0, 1000.0).expect("inside extent");
        assert!((sampled - 10.0).abs() < 1e-12, "sampled {sampled}");
    }
}
