//! Stacked and grouped bars, composed from the grammar — never a preset: a
//! `base` channel stacks each layer on the running total of the ones below
//! (the low half of `stat::stack`), and positioned bars (`Bars::at`) sit side
//! by side within their bands. In color modes the stack's segments read by
//! palette; plain output shows the envelope. Synthetic data.

use malevich::{Bars, Frame, Plot, Scale};

fn main() {
    let quarters = ["Q1", "Q2", "Q3", "Q4"];
    let platform = [4.2, 4.8, 5.1, 5.9];
    let services = [2.1, 2.4, 2.9, 3.3];
    let hardware = [1.4, 1.2, 1.1, 0.9];

    // Stacked: each layer rises from the running total of the ones below it.
    let bands = malevich::stat::stack(&[&platform, &services, &hardware]);
    let segments = [
        (&platform[..], "platform"),
        (&services[..], "services"),
        (&hardware[..], "hardware"),
    ];
    let mut stacked = Plot::new().title("revenue by segment, stacked ($B, synthetic)");
    for ((low, _), (values, label)) in bands.iter().zip(segments) {
        stacked = stacked.layer(Bars::new(quarters, values).base(&low[..]).label(label));
    }
    println!("{}", stacked.render_best(&Frame::plain(56, 16)));
    println!();

    // Grouped: one positioned layer per year, offset around the band centers.
    let last_year = [3.6, 4.1, 4.4, 5.0];
    let left: Vec<f64> = (0..quarters.len()).map(|i| i as f64 - 0.2).collect();
    let right: Vec<f64> = (0..quarters.len()).map(|i| i as f64 + 0.2).collect();
    let grouped = Plot::new()
        .x_scale(Scale::bands(quarters))
        .layer(Bars::at(&left[..], 0.32, &last_year[..]).label("2025"))
        .layer(Bars::at(&right[..], 0.32, &platform[..]).label("2026"))
        .title("platform revenue, year over year ($B, synthetic)");
    println!("{}", grouped.render_best(&Frame::plain(60, 14)));
}
