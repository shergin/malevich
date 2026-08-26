//! Chrome: the furniture around the plot area — title, legend, axes, labels.

use crate::plot::layout::{Colorbar, Layout};
use crate::render::{Color, Surface, display_width, fit_width_with};

/// Draws all furniture. Marks draw after, into the disjoint plot area.
pub(crate) fn draw(
    surface: &mut Surface,
    layout: &Layout<'_>,
    title: Option<&str>,
    axis_labels: (Option<&str>, Option<&str>),
    layers: &[crate::plot::resolve::ResolvedLayer<'_>],
) {
    let Layout {
        frame_width,
        px,
        py,
        ascii,
        title_rows,
        legend_rows,
        axis_rows,
        plot_top,
        plot_rows,
        gutter,
        label_width,
        plot_cols,
        ..
    } = *layout;
    let y_ticks = &layout.y_ticks;
    let x_ticks = &layout.x_ticks;
    let y_scale = &layout.y_scale;
    let glyphs = layout.charset.chrome();
    let x_scale = &layout.x_scale;
    // Chrome first, marks last: marks own the plot area, chrome owns the rest.
    if title_rows == 1
        && let Some(title) = title
    {
        let title = fit_width_with(title, frame_width, glyphs.ellipsis);
        let len = display_width(&title) as i64;
        let start = ((frame_width as i64 - len) / 2).max(0);
        surface.text(start, 0, &title, Color::Default);
    }

    if legend_rows == 1 {
        let mut entry_count = 0usize;
        let mut total = 0usize;
        for layer in layers {
            layer.for_each_legend_entry(ascii, |swatch, _, label| {
                entry_count = entry_count.saturating_add(1);
                total = total.saturating_add(
                    display_width(swatch)
                        .saturating_add(1)
                        .saturating_add(display_width(label)),
                );
            });
        }
        total = total.saturating_add(2usize.saturating_mul(entry_count.saturating_sub(1)));

        let total = i64::try_from(total).unwrap_or(i64::MAX);
        let frame_width = i64::try_from(frame_width).unwrap_or(i64::MAX);
        let mut column = (frame_width.saturating_sub(total) / 2).max(0);
        let row = title_rows as i64;
        let mut index = 0usize;
        for layer in layers {
            layer.for_each_legend_entry(ascii, |swatch, color, label| {
                if index > 0 {
                    column = column.saturating_add(2);
                }
                surface.text(column, row, swatch, color);
                let swatch_width = i64::try_from(display_width(swatch)).unwrap_or(i64::MAX);
                column = column.saturating_add(swatch_width.saturating_add(1));
                surface.text(column, row, label, Color::Default);
                let label_width = i64::try_from(display_width(label)).unwrap_or(i64::MAX);
                column = column.saturating_add(label_width);
                index = index.saturating_add(1);
            });
        }
    }

    if gutter >= 1 {
        let axis_column = (gutter - 1) as i64;
        for row in 0..plot_rows {
            surface.text(
                axis_column,
                (plot_top + row) as i64,
                glyphs.y_axis,
                Color::Default,
            );
        }
        if label_width > 0 {
            let mut used = vec![false; plot_rows];
            for tick in y_ticks {
                let sub = y_scale.map(tick.value);
                if !sub.is_finite() {
                    continue;
                }
                let row = (sub.round() as usize) / py;
                if row >= plot_rows || used[row] {
                    continue;
                }
                used[row] = true;
                let cell_row = (plot_top + row) as i64;
                let start =
                    (layout.y_label_cols + label_width) as i64 - display_width(&tick.label) as i64;
                surface.text(start, cell_row, &tick.label, Color::Default);
                surface.text(axis_column, cell_row, glyphs.y_tick, Color::Default);
            }
        }
    }

    if axis_rows >= 1 {
        let axis_row = (plot_top + plot_rows) as i64;
        if gutter >= 1 {
            surface.text((gutter - 1) as i64, axis_row, glyphs.corner, Color::Default);
        }
        for col in 0..plot_cols {
            surface.text(
                (gutter + col) as i64,
                axis_row,
                glyphs.x_axis,
                Color::Default,
            );
        }
        if let Some(ticks) = x_ticks {
            for tick in ticks {
                let column = (x_scale.map(tick.value).round() as usize) / px;
                surface.text(
                    (gutter + column) as i64,
                    axis_row,
                    glyphs.x_tick,
                    Color::Default,
                );
                let len = display_width(&tick.label) as i64;
                let center = (gutter + column) as i64;
                let start = (center - len / 2).clamp(0, (frame_width as i64 - len).max(0));
                surface.text(start, axis_row + 1, &tick.label, Color::Default);
            }
        }
        if axis_rows == 2
            && let (Some(band), Some(categories)) = (&layout.band, layout.categories)
        {
            let budget = ((band.step() / px as f64).round() as usize).max(2) - 1;
            for (index, category) in categories.iter().enumerate() {
                let label = fit_width_with(category, budget, glyphs.ellipsis);
                let len = display_width(&label) as i64;
                let center = gutter as i64 + (band.center(index) / px as f64).round() as i64;
                let start = (center - len / 2).clamp(0, (frame_width as i64 - len).max(0));
                surface.text(start, axis_row + 1, &label, Color::Default);
            }
        }
    }

    // Axis titles: x centered on its own bottom row, y written vertically along
    // the left edge, centered on the plot rows.
    if layout.x_label_rows == 1
        && let Some(label) = axis_labels.0
    {
        let label = fit_width_with(label, layout.plot_cols.max(1), glyphs.ellipsis);
        let len = display_width(&label) as i64;
        let center = (gutter + layout.plot_cols / 2) as i64;
        let start = (center - len / 2).clamp(0, (frame_width as i64 - len).max(0));
        let row = (layout.plot_top + plot_rows + axis_rows) as i64;
        surface.text(start, row, &label, Color::Default);
    }
    if layout.y_label_cols == 2
        && let Some(label) = axis_labels.1
    {
        let glyphs: Vec<char> = label.chars().take(plot_rows).collect();
        let start = layout.plot_top + (plot_rows.saturating_sub(glyphs.len())) / 2;
        let mut buffer = [0u8; 4];
        for (offset, glyph) in glyphs.into_iter().enumerate() {
            surface.text(
                0,
                (start + offset) as i64,
                glyph.encode_utf8(&mut buffer),
                Color::Default,
            );
        }
    }

    // The colorbar: a colormap gradient down the right edge, with value labels.
    if let Some(bar) = &layout.colorbar {
        draw_colorbar(surface, bar, plot_top, plot_rows);
    }
}

/// Draws the colorbar: a one-column colormap gradient (shade ramp plus color, like a
/// Cells layer, so it reads in plain text too) spanning the plot rows, top = high,
/// with the value labels beside it at their fractional heights.
fn draw_colorbar(surface: &mut Surface, bar: &Colorbar, plot_top: usize, plot_rows: usize) {
    const RAMP: [char; 4] = ['\u{2591}', '\u{2592}', '\u{2593}', '\u{2588}'];
    let mut buffer = [0u8; 4];
    let column = bar.column as i64;
    for offset in 0..plot_rows {
        let position = if plot_rows > 1 {
            1.0 - offset as f64 / (plot_rows - 1) as f64
        } else {
            1.0
        };
        let glyph = RAMP[((position * 4.0) as usize).min(3)];
        surface.text(
            column,
            (plot_top + offset) as i64,
            glyph.encode_utf8(&mut buffer),
            bar.colormap.color(position),
        );
    }
    // A log ramp spaces its decade ticks logarithmically down the strip.
    let log = bar.colormap.is_log() && bar.low > 0.0 && bar.high > 0.0;
    let (low, high) = if log {
        (bar.low.log10(), bar.high.log10())
    } else {
        (bar.low, bar.high)
    };
    let spread = high - low;
    for tick in bar.ticks.iter() {
        let value = if log { tick.value.log10() } else { tick.value };
        let fraction = if spread > 0.0 {
            (value - low) / spread
        } else {
            0.5
        };
        let offset = ((1.0 - fraction) * plot_rows.saturating_sub(1) as f64).round() as i64;
        surface.text(
            column + 2,
            plot_top as i64 + offset,
            &tick.label,
            Color::Default,
        );
    }
}
