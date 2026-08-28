//! A Q–Q plot from the grammar — no preset: matched quantiles of two samples
//! as a scatter (`stat::quantiles` sorts once per sample), the identity as a
//! plain line. Points on the line mean the distributions agree; the bowed tail
//! shows the second sample's heavier right side.

use malevich::stat::quantiles;
use malevich::{Dash, Frame, Line, Plot, Points};

fn main() {
    let noise = |i: usize, seed: f64| {
        let hash = (i as f64 * 12.9898 + seed * 78.233).sin() * 43758.5453;
        (hash - hash.floor()) * 2.0 - 1.0
    };
    // Two samples: near-normal (sum of uniforms) vs the same with a heavy
    // right tail.
    let normalish: Vec<f64> = (0..400)
        .map(|i| (0..6).map(|k| noise(i * 6 + k, 1.0)).sum::<f64>())
        .collect();
    let heavy: Vec<f64> = (0..400)
        .map(|i| {
            let base = (0..6).map(|k| noise(i * 6 + k, 9.0)).sum::<f64>();
            if noise(i, 17.0) > 0.6 {
                base * 2.5
            } else {
                base
            }
        })
        .collect();

    let positions: Vec<f64> = (1..100).map(|p| p as f64 / 100.0).collect();
    let qx = quantiles(&normalish, &positions);
    let qy = quantiles(&heavy, &positions);

    let span = (-4.0, 6.0);
    let plot = Plot::new()
        .layer(
            Line::xy(vec![span.0, span.1], vec![span.0, span.1])
                .label("identity")
                .dash(Dash::Dotted),
        )
        .layer(Points::xy(qx, qy).label("quantiles"))
        .title("Q\u{2013}Q: heavy-tailed vs normal-ish")
        .x_label("normal-ish quantiles")
        .y_label("heavy-tailed");
    println!("{}", plot.render_best(&Frame::plain(64, 20)));
}
