//! Palmer penguins (CC0 — see examples/data/README.md): bill dimensions separate
//! the species into visible clusters. One layer, one `color_by` channel — the
//! categories take palette colors, name themselves in the legend, and cycle
//! marker shapes in colorless output.

use malevich::{Frame, Plot, Points};
include!("support/svg_card.rs");

fn main() {
    let mut length = Vec::new();
    let mut depth = Vec::new();
    let mut species = Vec::new();
    for line in include_str!("data/penguins.csv").lines().skip(1) {
        let mut parts = line.split(',');
        let name = parts.next().unwrap_or_default();
        let bill: f64 = parts
            .next()
            .and_then(|v| v.parse().ok())
            .unwrap_or(f64::NAN);
        let bill_depth: f64 = parts
            .next()
            .and_then(|v| v.parse().ok())
            .unwrap_or(f64::NAN);
        length.push(bill);
        depth.push(bill_depth);
        species.push(name);
    }
    let plot = Plot::new()
        .layer(Points::xy(&length[..], &depth[..]).color_by(species))
        .title("penguin bills by species")
        .x_label("bill length, mm")
        .y_label("depth");
    let frame = Frame::plain(72, 20);
    if svg_card(&plot, &frame) {
        return;
    }
    println!("{}", plot.render_best(&frame));
}
