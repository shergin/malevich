//! A latency density that respects zero: request times cannot be negative,
//! and an unbounded kernel estimate leaks mass below zero anyway — the dashed
//! curve. `KdeOptions::bounds(Some(0.0), None)` reflects the kernels at the
//! bound, so the solid estimate keeps every gram of probability on the right
//! side of the `Rule` at zero and the pile-up near it stays sharp. Synthetic
//! latencies.

use malevich::mark::Dash;
use malevich::stat::{KdeOptions, kde, kde_with};
use malevich::{Frame, Line, Plot, Rule};
include!("support/svg_card.rs");

fn main() {
    // An exponential pile against zero (mean 6 ms), deterministic.
    let latencies: Vec<f64> = (0..600)
        .map(|i| {
            let u = ((i * 7919) % 600) as f64 / 600.0 + 0.0008;
            -u.ln() * 6.0
        })
        .collect();

    let (xs, leaky) = kde(&latencies, 240).expect("finite sample");
    let bounded = KdeOptions::new().bounds(Some(0.0), None);
    let (bxs, honest) = kde_with(&latencies, 240, bounded)
        .expect("valid options")
        .expect("finite sample");

    let plot = Plot::new()
        .layer(Rule::v(0.0).dash(Dash::Dotted))
        .layer(Line::xy(xs, leaky).dash(Dash::Dashed).label("unbounded"))
        .layer(Line::xy(bxs, honest).label("bounded at 0"))
        .title("request latency, density (synthetic)")
        .x_label("ms");
    let frame = Frame::plain(66, 16);
    if svg_card(&plot, &frame) {
        return;
    }
    println!("{}", plot.render_best(&frame));
}
