//! The same penguin flippers as `boxes`, drawn as mirrored kernel densities —
//! Gentoo's separation is a shape, not just a summary.

use malevich::Frame;
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
    let refs: Vec<&[f64]> = groups.iter().map(Vec::as_slice).collect();
    let chart = malevich::violin(names, refs)
        .title("flipper length by species, as densities")
        .y_label("mm");
    let frame = Frame::plain(60, 16);
    if svg_card(&chart, &frame) {
        return;
    }
    println!("{}", chart.render_best(&frame));
}
