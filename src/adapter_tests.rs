use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use ratatui_core::widgets::Widget;

use crate::{Line, Plot};

#[test]
fn the_widget_draws_into_a_buffer_with_styles() {
    let plot = Plot::new()
        .layer(Line::y(&[1.0, 5.0, 2.0][..]).label("a"))
        .layer(Line::y(&[2.0, 1.0, 4.0][..]).label("b"))
        .title("w");
    let area = Rect::new(0, 0, 30, 10);
    let mut buffer = Buffer::empty(area);
    Widget::render(plot.widget(), area, &mut buffer);

    let content: String = (0..10)
        .map(|y| {
            (0..30)
                .map(|x| buffer[(x, y)].symbol().to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(content.contains('\u{2502}'), "missing axis: {content}");
    assert!(content.contains('w'), "missing title: {content}");
    // Palette colors arrived as styles, not escapes.
    let styled =
        (0..30).any(|x| (0..10).any(|y| buffer[(x, y)].fg != ratatui_core::style::Color::Reset));
    assert!(styled, "no colored cells");
}

#[test]
fn rendering_clips_to_the_area() {
    let plot = Plot::new().layer(Line::y(&[1.0, 2.0][..]));
    let area = Rect::new(2, 1, 10, 5);
    let mut buffer = Buffer::empty(Rect::new(0, 0, 20, 10));
    Widget::render(plot.widget(), area, &mut buffer);
    for x in 0..20u16 {
        assert_eq!(buffer[(x, 0)].symbol(), " ", "wrote outside the area");
    }
}

#[test]
fn heatmap_half_blocks_map_both_colors_into_ratatui_styles() {
    let values: Vec<f64> = (0..128).map(f64::from).collect();
    let plot = crate::heatmap(1, &values);
    let area = Rect::new(0, 0, 24, 8);
    let mut buffer = Buffer::empty(area);
    Widget::render(plot.widget(), area, &mut buffer);

    let paired = (0..area.width).any(|x| {
        (0..area.height).any(|y| {
            let cell = &buffer[(x, y)];
            cell.symbol() == "\u{2580}"
                && cell.fg != ratatui_core::style::Color::Reset
                && cell.bg != ratatui_core::style::Color::Reset
        })
    });
    assert!(paired, "no independently styled heatmap half-block");
}

use ratatui_core::widgets::StatefulWidget;

use crate::{Mouse, MouseButton, PlotState, Points};

fn stateful_plot() -> Plot<'static> {
    Plot::new()
        .layer(Line::xy(&[0.0, 10.0][..], &[0.0, 10.0][..]))
        .x_domain(0.0, 10.0)
        .y_domain(0.0, 10.0)
}

fn render_stateful(plot: &Plot<'_>, area: Rect, state: &mut PlotState) -> Buffer {
    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 30));
    StatefulWidget::render(plot.widget(), area, &mut buffer, state);
    buffer
}

#[test]
fn a_stateful_render_hit_tests_through_the_area_offset() {
    let plot = stateful_plot();
    let area = Rect::new(5, 3, 50, 18);
    let mut state = PlotState::default();
    assert_eq!(state.data_at(10, 10), None, "no render yet");
    render_stateful(&plot, area, &mut state);

    let rect = state.plot_area().expect("a plot panel exists");
    assert!(
        rect.x >= area.x && rect.y >= area.y,
        "panel inside the area"
    );
    let (x, y) = state
        .data_at(rect.x + rect.width / 2, rect.y + rect.height / 2)
        .expect("center of the panel maps to data");
    assert!((0.0..=10.0).contains(&x), "x in domain: {x}");
    assert!((0.0..=10.0).contains(&y), "y in domain: {y}");
    assert_eq!(state.data_at(0, 0), None, "outside the panel");
}

#[test]
fn hovering_tracks_the_cursor_only_inside_the_panel() {
    let plot = stateful_plot();
    let mut state = PlotState::default();
    render_stateful(&plot, Rect::new(0, 0, 50, 18), &mut state);
    let rect = state.plot_area().unwrap();

    let inside = (rect.x + 2, rect.y + 2);
    assert!(state.on_mouse(Mouse::Moved {
        column: inside.0,
        row: inside.1
    }));
    assert_eq!(state.cursor(), Some(inside));
    assert!(state.cursor_data().is_some());
    assert!(
        !state.on_mouse(Mouse::Moved {
            column: inside.0,
            row: inside.1
        }),
        "no change"
    );
    assert!(
        state.on_mouse(Mouse::Moved { column: 0, row: 0 }),
        "leaving clears"
    );
    assert_eq!(state.cursor(), None);
}

#[test]
fn the_wheel_zooms_x_around_the_cursor_and_reset_returns_to_auto() {
    let plot = stateful_plot();
    let mut state = PlotState::default();
    render_stateful(&plot, Rect::new(0, 0, 50, 18), &mut state);
    let rect = state.plot_area().unwrap();
    let at = (rect.x + rect.width / 2, rect.y + rect.height / 2);
    let (anchor, _) = state.data_at(at.0, at.1).unwrap();

    assert!(state.on_mouse(Mouse::ScrollUp {
        column: at.0,
        row: at.1
    }));
    let (lo, hi) = state.viewport().x().expect("the wheel fixed x");
    assert!(hi - lo < 10.0, "narrower than the full domain");
    assert!(lo <= anchor && anchor <= hi, "anchor stays visible");
    assert_eq!(state.viewport().y(), None, "y stays automatic");

    state.reset_view();
    assert!(state.viewport().is_auto());
    assert!(
        !state.on_mouse(Mouse::ScrollUp { column: 0, row: 0 }),
        "outside is ignored"
    );
}

#[test]
fn a_left_drag_pans_the_view() {
    let plot = stateful_plot();
    let mut state = PlotState::default();
    render_stateful(&plot, Rect::new(0, 0, 50, 18), &mut state);
    let rect = state.plot_area().unwrap();
    let start = (rect.x + rect.width / 2, rect.y + rect.height / 2);

    assert!(state.on_mouse(Mouse::Down {
        button: MouseButton::Left,
        column: start.0,
        row: start.1
    }));
    assert!(state.on_mouse(Mouse::Drag {
        button: MouseButton::Left,
        column: start.0 + 5,
        row: start.1
    }));
    let (lo, hi) = state.viewport().x().expect("panning fixed x");
    assert!(
        lo < 0.0 && hi < 10.0,
        "dragging right slides the window left: ({lo}, {hi})"
    );
    assert!((hi - lo - 10.0).abs() < 1e-9, "the span is preserved");
    assert!(state.on_mouse(Mouse::Up {
        button: MouseButton::Left,
        column: start.0 + 5,
        row: start.1
    }));
}

#[test]
fn a_right_drag_zooms_to_the_selection_and_a_click_does_not() {
    let plot = stateful_plot();
    let mut state = PlotState::default();
    render_stateful(&plot, Rect::new(0, 0, 50, 18), &mut state);
    let rect = state.plot_area().unwrap();
    let a = (rect.x + 2, rect.y + 2);
    let b = (rect.x + rect.width - 3, rect.y + rect.height - 3);

    assert!(state.on_mouse(Mouse::Down {
        button: MouseButton::Right,
        column: a.0,
        row: a.1
    }));
    assert!(state.on_mouse(Mouse::Drag {
        button: MouseButton::Right,
        column: b.0,
        row: b.1
    }));
    assert!(state.on_mouse(Mouse::Up {
        button: MouseButton::Right,
        column: b.0,
        row: b.1
    }));
    let x = state.viewport().x().expect("selection fixed x");
    let y = state.viewport().y().expect("selection fixed y");
    assert!(x.0 < x.1 && y.0 < y.1);
    assert!(
        x.1 - x.0 < 10.0 && y.1 - y.0 < 10.0,
        "narrower than the domain"
    );

    state.reset_view();
    assert!(state.on_mouse(Mouse::Down {
        button: MouseButton::Right,
        column: a.0,
        row: a.1
    }));
    assert!(state.on_mouse(Mouse::Up {
        button: MouseButton::Right,
        column: a.0 + 1,
        row: a.1
    }));
    assert!(
        state.viewport().is_auto(),
        "a click-sized selection is discarded"
    );
}

#[test]
fn the_crosshair_tints_the_cursor_row_and_column_and_can_be_disabled() {
    let plot = stateful_plot();
    let area = Rect::new(0, 0, 50, 18);
    let mut state = PlotState::default();
    render_stateful(&plot, area, &mut state);
    let rect = state.plot_area().unwrap();
    let at = (rect.x + rect.width / 2, rect.y + rect.height / 2);
    state.on_mouse(Mouse::Moved {
        column: at.0,
        row: at.1,
    });

    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 30));
    StatefulWidget::render(plot.widget(), area, &mut buffer, &mut state);
    assert_ne!(
        buffer[(rect.x, at.1)].bg,
        ratatui_core::style::Color::Reset,
        "row tinted"
    );
    assert_ne!(
        buffer[(at.0, rect.y)].bg,
        ratatui_core::style::Color::Reset,
        "column tinted"
    );

    let mut bare = Buffer::empty(Rect::new(0, 0, 80, 30));
    StatefulWidget::render(
        plot.widget().crosshair(false).readout(false),
        area,
        &mut bare,
        &mut state,
    );
    assert_eq!(
        bare[(rect.x, at.1)].bg,
        ratatui_core::style::Color::Reset,
        "suppressed"
    );
}

#[test]
fn the_readout_writes_axis_formatted_coordinates() {
    let plot = stateful_plot();
    let area = Rect::new(0, 0, 60, 20);
    let mut state = PlotState::default();
    render_stateful(&plot, area, &mut state);
    let rect = state.plot_area().unwrap();
    let at = (rect.x + rect.width / 2, rect.y + rect.height / 2);
    state.on_mouse(Mouse::Moved {
        column: at.0,
        row: at.1,
    });

    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 30));
    StatefulWidget::render(
        plot.widget().crosshair(false),
        area,
        &mut buffer,
        &mut state,
    );
    let top_row: String = (rect.x..rect.right())
        .map(|x| buffer[(x, rect.y)].symbol().to_string())
        .collect();
    assert!(
        top_row.contains('·'),
        "readout separator present: {top_row:?}"
    );
}

#[test]
fn an_automatic_viewport_renders_exactly_like_the_stateless_widget() {
    let plot = stateful_plot();
    let area = Rect::new(0, 0, 50, 18);
    let mut stateless = Buffer::empty(Rect::new(0, 0, 80, 30));
    Widget::render(plot.widget(), area, &mut stateless);

    let mut state = PlotState::default();
    let stateful = render_stateful(&plot, area, &mut state);
    assert_eq!(stateless, stateful, "no cursor, no overlays, same cells");
}

fn hover_center(plot: &Plot<'_>, area: Rect, state: &mut PlotState) -> (Rect, (u16, u16)) {
    render_stateful(plot, area, state);
    let rect = state.plot_area().unwrap();
    let at = (rect.x + rect.width / 2, rect.y + rect.height / 2);
    state.on_mouse(Mouse::Moved {
        column: at.0,
        row: at.1,
    });
    (rect, at)
}

fn top_row(buffer: &Buffer, rect: Rect) -> String {
    (rect.x..rect.right())
        .map(|x| buffer[(x, rect.y)].symbol().to_string())
        .collect()
}

#[test]
fn the_readout_snaps_to_the_labeled_series() {
    let x: Vec<f64> = (0..=10).map(f64::from).collect();
    let y = x.clone();
    let plot = Plot::new()
        .layer(Line::xy(&x[..], &y[..]).label("loss"))
        .x_domain(0.0, 10.0)
        .y_domain(0.0, 10.0);
    let area = Rect::new(0, 0, 60, 20);
    let mut state = PlotState::default();
    let (rect, _) = hover_center(&plot, area, &mut state);

    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 30));
    StatefulWidget::render(
        plot.widget().crosshair(false),
        area,
        &mut buffer,
        &mut state,
    );
    let row = top_row(&buffer, rect);
    assert!(
        row.contains("loss:"),
        "snapped label in the readout: {row:?}"
    );

    // The snapped datum's cell wears the highlight.
    let (cursor_x, _) = state.cursor_data().unwrap();
    let snap_x = cursor_x.round();
    let mapping = state.mapping().unwrap();
    let (column, prow) = mapping.cell_at(snap_x, snap_x).unwrap();
    let cell = &buffer[(area.x + column as u16, area.y + prow as u16)];
    assert_ne!(cell.bg, ratatui_core::style::Color::Reset, "snap highlight");
}

#[test]
fn a_gap_at_the_snapped_position_reads_as_a_dash() {
    let mut y: Vec<f64> = (0..=10).map(f64::from).collect();
    y[5] = f64::NAN;
    let plot = Plot::new()
        .layer(Line::y(&y[..]).label("loss"))
        .x_domain(0.0, 10.0)
        .y_domain(0.0, 10.0);
    let mut state = PlotState::default();
    let (rect, at) = hover_center(&plot, Rect::new(0, 0, 60, 20), &mut state);
    let (cursor_x, _) = state.data_at(at.0, at.1).unwrap();
    assert_eq!(cursor_x.round(), 5.0, "the center cell hovers the gap");

    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 30));
    StatefulWidget::render(
        plot.widget().crosshair(false),
        Rect::new(0, 0, 60, 20),
        &mut buffer,
        &mut state,
    );
    let row = top_row(&buffer, rect);
    assert!(
        row.contains("loss: —"),
        "the gap is a dash, not a value: {row:?}"
    );
}

#[test]
fn snapping_disabled_returns_to_cursor_coordinates() {
    let x: Vec<f64> = (0..=10).map(f64::from).collect();
    let plot = Plot::new()
        .layer(Line::xy(&x[..], &x[..]).label("loss"))
        .x_domain(0.0, 10.0)
        .y_domain(0.0, 10.0);
    let mut state = PlotState::default();
    let (rect, _) = hover_center(&plot, Rect::new(0, 0, 60, 20), &mut state);

    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 30));
    StatefulWidget::render(
        plot.widget().crosshair(false).snap(false),
        Rect::new(0, 0, 60, 20),
        &mut buffer,
        &mut state,
    );
    let row = top_row(&buffer, rect);
    assert!(
        !row.contains("loss"),
        "no snapped label without snap: {row:?}"
    );
    assert!(row.contains('·'), "the coordinate readout remains: {row:?}");
}

#[test]
fn data_outside_the_zoomed_window_does_not_snap() {
    // All data lives in x 0..10; the viewport looks at 100..110.
    let x: Vec<f64> = (0..=10).map(f64::from).collect();
    let plot = Plot::new()
        .layer(Line::xy(&x[..], &x[..]).label("loss"))
        .y_domain(0.0, 10.0);
    let mut state = PlotState::default();
    state.set_viewport(crate::Viewport::auto().with_x(100.0, 110.0));
    let (rect, _) = hover_center(&plot, Rect::new(0, 0, 60, 20), &mut state);

    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 30));
    StatefulWidget::render(
        plot.widget().crosshair(false),
        Rect::new(0, 0, 60, 20),
        &mut buffer,
        &mut state,
    );
    let row = top_row(&buffer, rect);
    assert!(!row.contains("loss"), "nothing visible to snap to: {row:?}");
    assert!(row.contains('·'), "falls back to coordinates: {row:?}");
}

#[test]
fn every_snappable_series_joins_the_readout() {
    let x: Vec<f64> = (0..=10).map(f64::from).collect();
    let doubled: Vec<f64> = x.iter().map(|v| v / 2.0).collect();
    let plot = Plot::new()
        .layer(Line::xy(&x[..], &x[..]).label("a"))
        .layer(Points::xy(&x[..], &doubled[..]).label("b"))
        .layer(crate::Rule::h(3.0).label("rule"))
        .x_domain(0.0, 10.0)
        .y_domain(0.0, 10.0);
    let mut state = PlotState::default();
    let (rect, _) = hover_center(&plot, Rect::new(0, 0, 70, 22), &mut state);

    let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 30));
    StatefulWidget::render(
        plot.widget().crosshair(false),
        Rect::new(0, 0, 70, 22),
        &mut buffer,
        &mut state,
    );
    let row = top_row(&buffer, rect);
    assert!(row.contains("a:"), "the line snaps: {row:?}");
    assert!(row.contains("b:"), "the points snap: {row:?}");
    assert!(!row.contains("rule"), "a rule is not a series: {row:?}");
}

#[test]
fn keyboard_pan_shifts_the_window_and_ignores_bands() {
    let plot = stateful_plot();
    let mut state = PlotState::default();
    assert!(!state.pan_right(), "no mapping before the first render");
    render_stateful(&plot, Rect::new(0, 0, 50, 18), &mut state);

    assert!(state.pan_right());
    let (lo, hi) = state.viewport().x().expect("panning fixed x");
    assert!(
        (lo - 1.0).abs() < 1e-9 && (hi - 11.0).abs() < 1e-9,
        "a tenth right: ({lo}, {hi})"
    );
    assert!(state.pan_left());
    let (lo, hi) = state.viewport().x().expect("still fixed");
    assert!(
        (lo - 0.0).abs() < 1e-9 && (hi - 10.0).abs() < 1e-9,
        "and back: ({lo}, {hi})"
    );
    assert_eq!(state.viewport().y(), None, "y stays automatic");

    let bars = crate::bar(["a", "b"], &[1.0, 2.0][..]);
    let mut bands = PlotState::default();
    render_stateful(&bars, Rect::new(0, 0, 40, 12), &mut bands);
    assert!(!bands.pan_right(), "a bands axis never pans");
}

#[cfg(feature = "pixel")]
mod pixels {
    use super::*;
    use crate::pixel::{Graphics, Protocol};
    use crate::{Mouse, PlotState, clear_pixels, present_pixels};
    use ratatui_core::buffer::CellDiffOption;
    use ratatui_core::widgets::StatefulWidget;

    fn kitty() -> Graphics {
        Graphics::new(Protocol::Kitty)
    }

    fn render_pixel_frame(state: &mut PlotState, area: Rect) -> Buffer {
        let plot = stateful_plot();
        let mut buffer = Buffer::empty(Rect::new(0, 0, 80, 30));
        StatefulWidget::render(plot.widget().graphics(kitty()), area, &mut buffer, state);
        buffer
    }

    #[test]
    fn a_pixel_render_reserves_the_ground_and_stores_the_block() {
        let area = Rect::new(2, 1, 50, 18);
        let mut state = PlotState::default();
        let buffer = render_pixel_frame(&mut state, area);

        // First frame: fresh ground — real spaces, not skipped.
        let corner = &buffer[(area.x, area.y)];
        assert_eq!(corner.symbol(), " ");
        assert_eq!(corner.diff_option, CellDiffOption::None, "fresh ground");

        let (rect, block, _) = state.pixels.clone().expect("a pending block");
        assert_eq!(rect, area);
        assert!(block.contains("\x1b_G"), "kitty APC in the block");
        assert!(block.contains(",i="), "a stable image id rides the block");
        assert!(
            block.contains(",p=1,"),
            "a fixed placement id makes repaints replace, not stack"
        );
        assert!(block.contains("\r\n"), "raw-mode row separators");
        assert!(
            !block.replace("\r\n", "").contains('\n'),
            "no bare newlines remain"
        );

        // The mapping hit-tests the panel at cell resolution.
        let panel = state.plot_area().expect("a plot panel");
        assert!(
            state
                .data_at(panel.x + panel.width / 2, panel.y + panel.height / 2)
                .is_some()
        );

        // Second frame at the same rectangle: the ground is already clear,
        // cells skip so the diff leaves the image alone.
        let buffer = render_pixel_frame(&mut state, area);
        assert_eq!(
            buffer[(area.x, area.y)].diff_option,
            CellDiffOption::Skip,
            "steady-state frames skip"
        );
    }

    #[test]
    fn the_cursor_draws_interaction_chrome_into_the_image() {
        let area = Rect::new(0, 0, 50, 18);
        let mut state = PlotState::default();
        render_pixel_frame(&mut state, area);
        let (_, plain, _) = state.pixels.take().expect("block without a cursor");

        let panel = state.plot_area().unwrap();
        state.on_mouse(Mouse::Moved {
            column: panel.x + panel.width / 2,
            row: panel.y + panel.height / 2,
        });
        // Step past the pace window; an immediate re-render would reuse the
        // image already on screen.
        std::thread::sleep(std::time::Duration::from_millis(40));
        render_pixel_frame(&mut state, area);
        let (_, hovered, _) = state.pixels.take().expect("block with a cursor");
        assert_ne!(
            plain, hovered,
            "crosshair, markers, and readout in the image"
        );
    }

    #[test]
    fn renders_inside_the_pace_window_reuse_the_screen() {
        let area = Rect::new(0, 0, 50, 18);
        let mut state = PlotState::default();
        render_pixel_frame(&mut state, area);
        assert!(state.pixels.is_some(), "the first render always encodes");
        let mut out: Vec<u8> = Vec::new();
        present_pixels(&mut out, &kitty(), &mut [&mut state]).unwrap();

        // A hover flood within the window: nothing new to encode or present —
        // the image already on screen stays, and the frames cost nothing.
        let panel = state.plot_area().unwrap();
        state.on_mouse(Mouse::Moved {
            column: panel.x + 2,
            row: panel.y + 2,
        });
        render_pixel_frame(&mut state, area);
        assert!(state.pixels.is_none(), "paced: reuse, not re-encode");

        // A changed viewport must never show stale, pace or no pace.
        state.on_mouse(Mouse::ScrollUp {
            column: panel.x + 2,
            row: panel.y + 2,
        });
        render_pixel_frame(&mut state, area);
        assert!(state.pixels.is_some(), "a zoom bypasses the pace");

        // Past the window, a cursor move renders again.
        let mut out: Vec<u8> = Vec::new();
        present_pixels(&mut out, &kitty(), &mut [&mut state]).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(40));
        state.on_mouse(Mouse::Moved {
            column: panel.x + 3,
            row: panel.y + 3,
        });
        render_pixel_frame(&mut state, area);
        assert!(state.pixels.is_some(), "the pace window expired");
    }

    #[test]
    fn present_consumes_the_blocks_in_one_synchronized_swap() {
        let mut state = PlotState::default();
        render_pixel_frame(&mut state, Rect::new(2, 1, 50, 18));
        let mut out: Vec<u8> = Vec::new();
        present_pixels(&mut out, &kitty(), &mut [&mut state]).unwrap();
        let text = String::from_utf8_lossy(&out);
        assert!(text.starts_with("\x1b[?2026h"), "synchronized begin");
        assert!(text.ends_with("\x1b[?2026l"), "synchronized end");
        assert!(
            !text.contains("a=d"),
            "no delete — id retransmission replaces atomically"
        );
        assert!(text.contains("\x1b[2;3H"), "absolute move to the area");
        assert!(state.pixels.is_none(), "the block is consumed");

        let mut quiet: Vec<u8> = Vec::new();
        present_pixels(&mut quiet, &kitty(), &mut [&mut state]).unwrap();
        assert!(quiet.is_empty(), "nothing pending, nothing written");
    }

    #[test]
    fn clear_pixels_retires_only_our_presented_images() {
        let area = Rect::new(0, 0, 50, 18);
        let mut presented = PlotState::default();
        render_pixel_frame(&mut presented, area);
        let mut out: Vec<u8> = Vec::new();
        present_pixels(&mut out, &kitty(), &mut [&mut presented]).unwrap();

        let mut goodbye: Vec<u8> = Vec::new();
        let mut untouched = PlotState::default();
        clear_pixels(
            &mut goodbye,
            &kitty(),
            &mut [&mut presented, &mut untouched],
        )
        .unwrap();
        let text = String::from_utf8_lossy(&goodbye);
        assert!(text.contains("a=d,d=I,i="), "delete by our id: {text:?}");
        assert_eq!(text.matches("a=d").count(), 1, "only the presented panel");
        assert!(presented.pixel_sent.is_none(), "presentation memory reset");

        let mut sixel: Vec<u8> = Vec::new();
        clear_pixels(
            &mut sixel,
            &Graphics::new(Protocol::Sixel),
            &mut [&mut presented],
        )
        .unwrap();
        assert!(sixel.is_empty(), "cells cover themselves");
    }

    #[test]
    fn an_unchanged_panel_is_never_retransmitted() {
        let area = Rect::new(0, 0, 50, 18);
        let mut state = PlotState::default();
        render_pixel_frame(&mut state, area);
        let mut out: Vec<u8> = Vec::new();
        present_pixels(&mut out, &kitty(), &mut [&mut state]).unwrap();

        // Past the pace window with nothing changed: the encode runs, matches
        // what is on screen, and queues nothing.
        std::thread::sleep(std::time::Duration::from_millis(40));
        render_pixel_frame(&mut state, area);
        assert!(state.pixels.is_none(), "identical content is suppressed");

        // A cursor changes the image, so it queues again.
        let panel = state.plot_area().unwrap();
        state.on_mouse(Mouse::Moved {
            column: panel.x + 2,
            row: panel.y + 2,
        });
        std::thread::sleep(std::time::Duration::from_millis(40));
        render_pixel_frame(&mut state, area);
        assert!(state.pixels.is_some(), "changed content queues");
    }

    #[test]
    fn invalidation_forces_fresh_ground() {
        let area = Rect::new(0, 0, 50, 18);
        let mut state = PlotState::default();
        render_pixel_frame(&mut state, area);
        render_pixel_frame(&mut state, area);
        state.invalidate_pixels();
        let buffer = render_pixel_frame(&mut state, area);
        assert_eq!(
            buffer[(area.x, area.y)].diff_option,
            CellDiffOption::None,
            "after invalidation the ground is repainted"
        );
    }
}
