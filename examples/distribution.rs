//! Palmer penguin body mass (CC0): a real bimodal-ish distribution through the
//! automatic binning — Gentoos are simply heavier.

use malevich::Frame;
include!("support/svg_card.rs");

fn main() {
    let mass: Vec<f64> = include_str!("data/penguins.csv")
        .lines()
        .skip(1)
        .filter_map(|line| line.split(',').nth(4)?.parse().ok())
        .collect();
    let chart = malevich::hist(&mass[..])
        .title("penguin body mass")
        .x_label("grams");
    let frame = Frame::plain(64, 15);
    if svg_card(&chart, &frame) {
        return;
    }
    println!("{}", chart.render_best(&frame));
}
