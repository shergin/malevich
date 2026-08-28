//! Millions of points under a live zoom. Run with
//! `cargo run --release --example zoom --features ratatui -- 10000000`
//! (the count is optional; two million by default). Wheel-zoom into any spike,
//! left-drag to pan, right-drag a rectangle to zoom to it, `r` resets, `q`
//! quits.
//!
//! The point: zooming is nothing but a domain window, so every frame re-runs
//! M4 aggregation over the full series into the visible window — the drawn
//! line is pixel-identical to plotting every point, at any zoom, at
//! interactive rates.

use std::io::stdout;

use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton as Ct,
    MouseEvent, MouseEventKind as Kind,
};
use crossterm::execute;
use malevich::{Charset, Color, Line, Mouse, MouseButton, Plot, PlotState};

fn mouse(event: MouseEvent) -> Option<Mouse> {
    let button = |b: Ct| match b {
        Ct::Left => MouseButton::Left,
        Ct::Right => MouseButton::Right,
        Ct::Middle => MouseButton::Middle,
    };
    let (column, row) = (event.column, event.row);
    Some(match event.kind {
        Kind::Moved => Mouse::Moved { column, row },
        Kind::Down(b) => Mouse::Down {
            button: button(b),
            column,
            row,
        },
        Kind::Drag(b) => Mouse::Drag {
            button: button(b),
            column,
            row,
        },
        Kind::Up(b) => Mouse::Up {
            button: button(b),
            column,
            row,
        },
        Kind::ScrollUp => Mouse::ScrollUp { column, row },
        Kind::ScrollDown => Mouse::ScrollDown { column, row },
        _ => return None,
    })
}

/// A deterministic composite signal: slow waves, fast ripple, LCG noise, and
/// rare narrow spikes that only a raster-exact reduction keeps visible.
fn signal(n: usize) -> (Vec<f64>, Vec<f64>) {
    let mut x = Vec::with_capacity(n);
    let mut y = Vec::with_capacity(n);
    let mut lcg: u64 = 0x2545F491_4F6CDD1D;
    for i in 0..n {
        let t = i as f64 / n as f64;
        lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let noise = (lcg >> 33) as f64 / f64::from(1u32 << 31) - 1.0;
        let spike = if lcg % 1_000_003 < 2 { 6.0 } else { 0.0 };
        x.push(i as f64);
        y.push(
            (t * 12.0 * std::f64::consts::TAU).sin() * 2.0
                + (t * 397.0 * std::f64::consts::TAU).sin() * 0.6
                + noise * 0.25
                + spike,
        );
    }
    (x, y)
}

fn main() -> std::io::Result<()> {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(2_000_000);
    let (x, y) = signal(n);

    let mut terminal = ratatui::init();
    execute!(stdout(), EnableMouseCapture)?;
    let mut state = PlotState::default();
    let result = loop {
        terminal.draw(|frame| {
            let window = state
                .viewport()
                .x()
                .map_or_else(|| "all".to_string(), |(lo, hi)| {
                    format!("{:.0}..{:.0}", lo, hi)
                });
            let plot = Plot::new()
                .layer(Line::xy(&x[..], &y[..]).color(Color::Cyan))
                .title(format!(
                    "{n} points, showing {window} — wheel zoom · drag pan · right-drag zoom · r reset · q quit"
                ));
            frame.render_stateful_widget(
                plot.widget().charset(Charset::Braille),
                frame.area(),
                &mut state,
            );
        })?;
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                KeyCode::Char('r') => state.reset_view(),
                KeyCode::Char('+') | KeyCode::Char('=') => {
                    state.zoom_in();
                }
                KeyCode::Char('-') => {
                    state.zoom_out();
                }
                _ => {}
            },
            Event::Mouse(raw) => {
                if let Some(input) = mouse(raw) {
                    state.on_mouse(input);
                }
            }
            _ => {}
        }
    };
    execute!(stdout(), DisableMouseCapture)?;
    ratatui::restore();
    result
}
