//! Regression fences for the ratatui widget: what one interactive frame costs.
//!
//! Three rows. `dashboard_200x50` is a representative two-pane frame — a
//! legended two-series line chart beside a colorbarred heatmap — through the
//! stateless `Widget` path (rasterize + blit, no string encoding).
//! `zoom_10m_200x50` is the interactive headline: a ten-million-point line
//! rendered stateful at a fixed 1% zoom window, the cost of every frame while
//! a user pans or zooms — M4 re-aggregates the full series into the window
//! each time. `hover_snap_10m_200x50` adds a hover cursor to the same state,
//! pricing the snap readout's nearest scan over explicit x plus the overlay
//! drawing on top of the zoomed render.

use criterion::{Criterion, criterion_group, criterion_main};
use malevich::{Charset, Line, Mouse, Plot, PlotState, Viewport};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{StatefulWidget, Widget};
use std::hint::black_box;

fn dashboard(c: &mut Criterion) {
    let n = 100_000;
    let first: Vec<f64> = (0..n).map(|i| (i as f64 * 0.0004).sin()).collect();
    let second: Vec<f64> = (0..n).map(|i| (i as f64 * 0.0007).cos()).collect();
    let lines = Plot::new()
        .layer(Line::y(&first[..]).label("first"))
        .layer(Line::y(&second[..]).label("second"))
        .title("lines");
    let grid: Vec<f64> = (0..256 * 128)
        .map(|i| ((i % 256) as f64 * 0.13).sin() + ((i / 256) as f64 * 0.29).cos())
        .collect();
    let heat = malevich::heatmap(256, &grid[..]).colorbar().title("heat");

    let area = Rect::new(0, 0, 200, 50);
    let left = Rect::new(0, 0, 120, 50);
    let right = Rect::new(120, 0, 80, 50);
    let mut buffer = Buffer::empty(area);
    c.bench_function("widget/dashboard_200x50", |b| {
        b.iter(|| {
            Widget::render(lines.widget().charset(Charset::Braille), left, &mut buffer);
            Widget::render(heat.widget(), right, &mut buffer);
            black_box(&buffer);
        });
    });
}

fn zoomed_ten_million(c: &mut Criterion) {
    let n = 10_000_000;
    let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let y: Vec<f64> = (0..n)
        .map(|i| (i as f64 * 0.0002).sin() * (i as f64 * 0.000013).cos() * 8.0)
        .collect();
    let plot = Plot::new().layer(Line::xy(&x[..], &y[..]));
    let area = Rect::new(0, 0, 200, 50);
    let mut buffer = Buffer::empty(area);
    let mut state = PlotState::default();
    // The window a few wheel notches deep: one percent of the data, mid-series.
    state.set_viewport(Viewport::auto().with_x(4_950_000.0, 5_050_000.0));
    c.bench_function("widget/zoom_10m_200x50", |b| {
        b.iter(|| {
            StatefulWidget::render(
                plot.widget().charset(Charset::Braille),
                area,
                &mut buffer,
                &mut state,
            );
            black_box(&buffer);
        });
    });

    // The same zoomed frame with the cursor over the panel: the snap readout
    // scans the explicit x column for the nearest visible datum every frame.
    let rect = state.plot_area().expect("the panel exists");
    state.on_mouse(Mouse::Moved {
        column: rect.x + rect.width / 2,
        row: rect.y + rect.height / 2,
    });
    c.bench_function("widget/hover_snap_10m_200x50", |b| {
        b.iter(|| {
            StatefulWidget::render(
                plot.widget().charset(Charset::Braille),
                area,
                &mut buffer,
                &mut state,
            );
            black_box(&buffer);
        });
    });
}

criterion_group!(benches, dashboard, zoomed_ten_million);
criterion_main!(benches);
