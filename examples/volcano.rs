//! A volcano plot from the grammar — no preset: significance classes through
//! `color_by`, fold-change and p-value thresholds as `Rule`s. Genes classify by
//! effect size and significance; the palette pins grey to "not significant".

use malevich::scale::Palette;
use malevich::{Color, Dash, Frame, Plot, Points, Rule};

fn main() {
    // Deterministic synthetic differential expression: most genes near zero
    // effect, a regulated tail on both sides.
    let n = 900usize;
    let noise = |i: usize, seed: f64| {
        let hash = (i as f64 * 12.9898 + seed * 78.233).sin() * 43758.5453;
        (hash - hash.floor()) * 2.0 - 1.0
    };
    let mut fold = Vec::with_capacity(n);
    let mut significance = Vec::with_capacity(n);
    for i in 0..n {
        let spread = if i % 7 == 0 { 2.6 } else { 0.7 };
        let log2fc = noise(i, 1.0) * spread;
        let driven = (log2fc.abs() * 1.6 - 0.4 + noise(i, 7.0) * 1.2).max(0.02);
        fold.push(log2fc);
        significance.push(driven); // already -log10 p
    }
    // Partition so "n.s." appears first: category order is first appearance,
    // and the palette below assigns grey to it.
    let class = |fc: f64, p: f64| {
        if p < 2.0 || fc.abs() < 1.0 {
            "n.s."
        } else if fc > 0.0 {
            "up"
        } else {
            "down"
        }
    };
    let mut x = Vec::new();
    let mut y = Vec::new();
    let mut classes = Vec::new();
    for wanted in ["n.s.", "down", "up"] {
        for (&fc, &p) in fold.iter().zip(&significance) {
            if class(fc, p) == wanted {
                x.push(fc);
                y.push(p);
                classes.push(wanted);
            }
        }
    }
    let plot = Plot::new()
        .layer(Points::xy(&x[..], &y[..]).color_by(classes))
        .palette(Palette::new(&[
            Color::BrightBlack,      // n.s. — recedes
            Color::Rgb(0, 114, 178), // down — blue
            Color::Rgb(213, 94, 0),  // up — vermillion
        ]))
        .layer(Rule::v(-1.0).dash(Dash::Dashed))
        .layer(Rule::v(1.0).dash(Dash::Dashed))
        .layer(Rule::h(2.0).dash(Dash::Dashed))
        .title("differential expression (synthetic)")
        .x_label("log2 fold change")
        .y_label("-log10 p");
    println!("{}", plot.render_best(&Frame::plain(72, 22)));
}
