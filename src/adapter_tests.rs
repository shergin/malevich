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

use crate::{Mouse, MouseButton, PlotState};

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
