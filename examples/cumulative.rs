//! A cumulative histogram in percent: the share of requests served within a
//! latency, read straight off the axis. `HistogramOptions::normalization`
//! rescales the same bins — counts, probability, percent, or density — and
//! `cumulative` accumulates them, so the last bar reaches 100 % and the `Rule`
//! at 95 shows where the tail begins. Synthetic latencies.

use malevich::mark::Dash;
use malevich::stat::Normalization;
use malevich::{Frame, HistogramOptions, Rule, hist_with};
include!("support/svg_card.rs");

fn main() {
    // An exponential pile against zero (mean 6 ms), deterministic.
    let latencies: Vec<f64> = (0..600)
        .map(|i| {
            let u = ((i * 7919) % 600) as f64 / 600.0 + 0.0008;
            -u.ln() * 6.0
        })
        .collect();

    let share = HistogramOptions::new(24)
        .normalization(Normalization::Percent)
        .cumulative(true);
    let plot = hist_with(&latencies[..], share)
        .expect("valid options")
        .layer(Rule::h(95.0).dash(Dash::Dotted).label("p95"))
        .title("requests served within a latency (synthetic)")
        .x_label("ms");
    let frame = Frame::plain(66, 14);
    if svg_card(&plot, &frame) {
        return;
    }
    println!("{}", plot.render_best(&frame));
}
