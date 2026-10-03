//! Candlesticks from the grammar — no preset: `Range` whiskers carry high/low,
//! its body carries open/close, and `color_by` splits up-days from down-days.
//! Categories take palette colors in first-appearance order; the walk below
//! opens upward, so green leads.

use malevich::scale::Palette;
use malevich::{Color, Frame, Plot, Range};
include!("support/svg_card.rs");

fn main() {
    let days = 46usize;
    let noise = |i: usize, seed: f64| {
        let hash = (i as f64 * 12.9898 + seed * 78.233).sin() * 43758.5453;
        (hash - hash.floor()) * 2.0 - 1.0
    };
    let mut price = 100.0f64;
    let (mut t, mut low, mut high, mut open, mut close, mut day) = (
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    for i in 0..days {
        let drift = if i == 0 {
            0.8
        } else {
            noise(i, 3.0) * 2.2 + 0.1
        };
        let opened = price;
        let closed = opened + drift;
        let wick = 0.4 + noise(i, 11.0).abs() * 1.4;
        t.push(i as f64);
        open.push(opened);
        close.push(closed);
        low.push(opened.min(closed) - wick);
        high.push(opened.max(closed) + wick);
        day.push(if closed >= opened { "up" } else { "down" });
        price = closed;
    }
    let plot = Plot::new()
        .layer(
            Range::xy(&t[..], &low[..], &high[..])
                .body(&open[..], &close[..])
                .color_by(day),
        )
        .palette(Palette::new(&[
            Color::Rgb(0, 158, 115), // up — bluish green
            Color::Rgb(213, 94, 0),  // down — vermillion
        ]))
        .title("daily candles (synthetic)")
        .y_label("price");
    let frame = Frame::plain(72, 20);
    if svg_card(&plot, &frame) {
        return;
    }
    println!("{}", plot.render_best(&frame));
}
