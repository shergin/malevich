//! Bollinger bands from the grammar, no preset: a centered rolling mean and
//! a rolling sample deviation — two reductions in the one `Reducer`
//! vocabulary — give the band edges `mean ± 2σ` as `Line`s, the mean as a
//! dashed one, and the series itself on top. The edges are lines rather than
//! an `Area` fill because a solid fill would swallow the price in a
//! monochrome cell grid. The window is anchored in the middle so the band
//! sits on the data instead of trailing it. Synthetic prices.

use malevich::mark::Dash;
use malevich::stat::{Reducer, Window, WindowAnchor};
use malevich::{Color, Frame, Line, Plot};
include!("support/svg_card.rs");

fn main() {
    // A random-walk price, deterministic.
    let mut price = 100.0f64;
    let mut state = 0x2545_F491_4F6C_DD1Du64;
    let prices: Vec<f64> = (0..160)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let step = ((state >> 11) as f64 / (1u64 << 53) as f64 - 0.5) * 3.0;
            price = (price + step).max(1.0);
            price
        })
        .collect();
    let x: Vec<f64> = (0..prices.len()).map(|i| i as f64).collect();

    let window = Window::new(20).anchor(WindowAnchor::Middle);
    let mean = window.mean(&prices);
    let deviation = window.reduce(&prices, Reducer::Deviation);
    let lower: Vec<f64> = mean
        .iter()
        .zip(&deviation)
        .map(|(m, s)| m - 2.0 * s)
        .collect();
    let upper: Vec<f64> = mean
        .iter()
        .zip(&deviation)
        .map(|(m, s)| m + 2.0 * s)
        .collect();

    let plot = Plot::new()
        .layer(Line::xy(&x[..], &upper[..]).label("±2σ"))
        .layer(Line::xy(&x[..], &lower[..]).color(Color::Default))
        .layer(
            Line::xy(&x[..], &mean[..])
                .dash(Dash::Dashed)
                .label("20-day mean"),
        )
        .layer(Line::xy(&x[..], &prices[..]).label("price"))
        .title("Bollinger bands: a centered window's mean ± 2σ (synthetic)")
        .x_label("day");
    let frame = Frame::plain(72, 20);
    if svg_card(&plot, &frame) {
        return;
    }
    println!("{}", plot.render_best(&frame));
}
