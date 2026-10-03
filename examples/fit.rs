//! A least-squares trend through noisy measurements: the `trend_with` preset
//! draws the scatter, the fitted line, and a 95% confidence band around the
//! mean response; the same `stat::Fit` accumulator reports R² for the title —
//! and, being a mergeable monoid, fits streams and parallel chunks alike.

use malevich::stat::Fit;
use malevich::{Frame, TrendOptions};
include!("support/svg_card.rs");

fn main() {
    let noise = |i: usize, seed: f64| {
        let hash = (i as f64 * 12.9898 + seed * 78.233).sin() * 43758.5453;
        (hash - hash.floor()) * 2.0 - 1.0
    };
    let n = 60usize;
    let x: Vec<f64> = (0..n)
        .map(|i| i as f64 * 0.5 + noise(i, 2.0) * 0.2)
        .collect();
    let y: Vec<f64> = x
        .iter()
        .enumerate()
        .map(|(i, &v)| 0.8 * v + 4.0 + noise(i, 9.0) * 3.0)
        .collect();

    let fit = Fit::xy(&x, &y);
    let chart = malevich::trend_with(&x[..], &y[..], TrendOptions::new().band(1.96))
        .expect("a positive band multiplier is valid")
        .title(format!(
            "y = {:.2}x + {:.2}   R² = {:.2}",
            fit.slope().unwrap(),
            fit.intercept().unwrap(),
            fit.r_squared().unwrap()
        ))
        .x_label("dose")
        .y_label("response");
    let frame = Frame::plain(72, 20);
    if svg_card(&chart, &frame) {
        return;
    }
    println!("{}", chart.render_best(&frame));
}
