//! A raincloud plot from the grammar, no preset: for each species, the
//! kernel density as a half violin on the right (`Area::horizontal` from the
//! band center outward), the five-number summary as a `Range` box just left
//! of center, and every measurement as a jittered point on the left — the
//! rain. `stat::jitter` spreads the points with a van der Corput sequence,
//! so the strip fills evenly and renders the same every time. Palmer
//! penguin flippers.

use malevich::stat::{BoxStats, jitter, kde};
use malevich::{Area, Frame, Plot, Points, Range, Scale};
include!("support/svg_card.rs");

fn main() {
    let names = ["Adelie", "Chinstrap", "Gentoo"];
    let mut groups = [Vec::new(), Vec::new(), Vec::new()];
    for line in include_str!("data/penguins.csv").lines().skip(1) {
        let mut parts = line.split(',');
        let species = parts.next().unwrap_or_default();
        let flipper: Option<f64> = parts.nth(2).and_then(|v| v.parse().ok());
        if let (Some(index), Some(flipper)) = (names.iter().position(|n| *n == species), flipper) {
            groups[index].push(flipper);
        }
    }

    let mut plot = Plot::new()
        .x_scale(Scale::bands(names))
        .title("flipper length by species: cloud, box, and rain")
        .y_label("mm");
    let mut box_low = Vec::new();
    let mut box_high = Vec::new();
    let mut box_q1 = Vec::new();
    let mut box_q3 = Vec::new();
    let mut box_median = Vec::new();
    for (index, group) in groups.iter().enumerate() {
        let center = index as f64;
        // The cloud: a half violin, scaled so every species peaks the same.
        let (positions, density) = kde(group, 128).expect("finite sample");
        let peak = density.iter().copied().fold(f64::MIN_POSITIVE, f64::max);
        let inner = vec![center + 0.05; positions.len()];
        let outer: Vec<f64> = density
            .iter()
            .map(|d| center + 0.05 + d / peak * 0.35)
            .collect();
        plot = plot.layer(Area::horizontal(positions, inner, outer));
        // The rain: every measurement, jittered in a strip left of center.
        let strip = jitter(&vec![center - 0.28; group.len()], 0.2);
        plot = plot.layer(Points::xy(strip, group.clone()));
        // The box, between the two.
        let stats = BoxStats::of(group).expect("finite sample");
        box_low.push(stats.whisker_low);
        box_high.push(stats.whisker_high);
        box_q1.push(stats.q1);
        box_q3.push(stats.q3);
        box_median.push(stats.median);
    }
    let box_x: Vec<f64> = (0..names.len()).map(|i| i as f64 - 0.08).collect();
    plot = plot.layer(
        Range::xy(box_x, box_low, box_high)
            .body(box_q1, box_q3)
            .marker(box_median),
    );
    let frame = Frame::plain(66, 22);
    if svg_card(&plot, &frame) {
        return;
    }
    println!("{}", plot.render_best(&frame));
}
