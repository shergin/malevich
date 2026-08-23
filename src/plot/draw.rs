//! Mark drawing: resolved layers rasterized onto a canvas through the layout.
//!
//! Generic over [`Canvas`] and monomorphized per target: the same mark code draws
//! glyph cells and (under the `pixel` feature) device pixels. Anything
//! precision-dependent — bar fills, marker crossbars, Cells patches — goes through
//! the canvas's mid-level ops, so each target renders it at its own fidelity.

use crate::mark::{LineStyle, PointStyle};
use crate::mark::{Orientation, Placement};
use crate::plot::layout::{Layout, Map};
use crate::plot::resolve::{ColorChannel, Coordinates, Kind, ResolvedLayer, extent};
use crate::render::{Canvas, Charset, Color, PlotRect, PointShape};
use crate::scale::Colormap;

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
                kind: Kind::Line(LineStyle::Corners),
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
                ..
            } => {
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
            }
            ResolvedLayer::Cells {
                columns,
                values,
                extents,
                colormap,
            } => {
                draw_cells(
                    surface,
                    *columns,
                    values,
                    *extents,
                    colormap.clone(),
                    x_scale,
                    y_scale,
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
                orientation, color, ..
            } => match orientation {
                Orientation::Horizontal(y) => {
                    let sy = y_offset + y_scale.map(*y);
                    surface.line(
                        (x_offset, sy),
                        (x_offset + (plot_sub_w - 1) as f64, sy),
                        *color,
                    );
                }
                Orientation::Vertical(x) => {
                    let sx = x_offset + x_scale.map(*x);
                    surface.line(
                        (sx, y_offset),
                        (sx, y_offset + (plot_sub_h - 1) as f64),
                        *color,
                    );
                }
            },
            ResolvedLayer::Text { x, y, text, color } => {
                let sx = x_offset + x_scale.map(*x);
                let sy = y_offset + y_scale.map(*y);
                if sx.is_finite() && sy.is_finite() {
                    surface.text(
                        (sx / px as f64).round() as i64,
                        (sy / py as f64).round() as i64,
                        text,
                        *color,
                    );
                }
            }
            ResolvedLayer::Bars {
                placement,
                values,
                color,
                ..
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
                        color,
                        rect,
                    );
                }
            },
        }
    }
    surface.clear_clip();
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
        Kind::Line(LineStyle::Corners) => {
            unreachable!("corners are drawn by draw_corners");
        }
        Kind::Line(LineStyle::Pixels) => {
            let mut previous: Option<((f64, f64), Option<usize>)> = None;
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
                        surface.line(from, position, ink);
                    }
                    _ => surface.dot(position.0, position.1, ink),
                }
                previous = Some((position, category));
            }
        }
        Kind::Points(style) => {
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

/// Draws one bars layer: per bar, the canvas fills the span from the zero baseline
/// to the value end at its own precision (eighth-block ramps on cells, exact
/// rectangles on pixels).
fn draw_bars<C: Canvas>(
    surface: &mut C,
    span: &dyn Fn(usize) -> (f64, f64),
    y_scale: &Map,
    values: &[f64],
    color: &ColorChannel<'_>,
    rect: PlotRect,
) {
    let baseline = y_scale.map(0.0);
    for (index, &value) in values.iter().enumerate() {
        if !value.is_finite() || value == 0.0 {
            continue;
        }
        let (left_sub, right_sub) = span(index);
        if !left_sub.is_finite() || !right_sub.is_finite() {
            continue;
        }
        surface.bar(
            (left_sub, right_sub),
            y_scale.map(value),
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

/// Draws one cells layer: for every patch of the canvas's sampling grid inside the
/// plot area, the nearest grid sample fills colored by the colormap — one cell per
/// patch on glyph targets, one pixel per patch on pixel targets. Gaps stay blank.
#[allow(clippy::too_many_arguments)]
fn draw_cells<C: Canvas>(
    surface: &mut C,
    columns: usize,
    values: &[f64],
    extents: Option<((f64, f64), (f64, f64))>,
    colormap: Colormap,
    x_scale: &Map,
    y_scale: &Map,
    rect: PlotRect,
    density: (usize, usize),
) {
    let (px, py) = density;
    let rows = values.len() / columns.max(1);
    if rows == 0 {
        return;
    }
    let Some((low, high)) = extent(values) else {
        return;
    };
    let ((x0, x1), (y0, y1)) = extents.unwrap_or(((0.0, columns as f64), (0.0, rows as f64)));
    let (samples_x, samples_y) = surface.patch_density();
    if samples_x == 0 || samples_y == 0 {
        return;
    }
    let units_x = rect.columns * samples_x;
    let units_y = rect.rows * samples_y;

    for unit_row in 0..units_y {
        for unit_col in 0..units_x {
            // The data position at this patch's center, via the shared scales'
            // subpixel geometry.
            let sub_x = (unit_col as f64 + 0.5) * px as f64 / samples_x as f64;
            let sub_y = (unit_row as f64 + 0.5) * py as f64 / samples_y as f64;
            let sample = (|| {
                let fx = position_on(x_scale, sub_x, x0, x1)?;
                let fy = position_on(y_scale, sub_y, y0, y1)?;
                let column = (crate::numeric::inverse_lerp(x0, x1, fx) * columns as f64).floor();
                let row = (crate::numeric::inverse_lerp(y0, y1, fy) * rows as f64).floor();
                if column < 0.0 || row < 0.0 {
                    return None;
                }
                let (column, row) = (column as usize, row as usize);
                if column >= columns || row >= rows {
                    return None;
                }
                let value = values[row * columns + column];
                if !value.is_finite() {
                    return None;
                }
                let position = colormap.position_in(value, low, high);
                Some((position, colormap.color(position)))
            })();
            surface.patch(unit_col, unit_row, rect, sample);
        }
    }
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
    // subpixel extent bounds the fill loop so distant points cannot spin it.
    let main_limit = if horizontal { bounds.1 } else { bounds.0 } as i64;
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
                let lo = (from.round() as i64).max(0);
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
    use super::{Map, position_on};

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
