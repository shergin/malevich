//! Palmer penguins again (CC0): flipper length summarized per species — type-7
//! quartiles, Tukey whiskers, outliers as dots. Real measurements, real spread.

use malevich::Frame;
include!("support/svg_card.rs");

fn main() {
    let (categories, groups) = penguin_flippers();
    let refs: Vec<&[f64]> = groups.iter().map(Vec::as_slice).collect();
    let chart = malevich::box_plot(categories, refs)
        .title("flipper length by species")
        .y_label("mm");
    let frame = Frame::portable(60, 16);
    if svg_card(&chart, &frame) {
        return;
    }
    println!("{}", chart.render_best(&frame));
}

fn penguin_flippers() -> (Vec<&'static str>, [Vec<f64>; 3]) {
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
    (names.to_vec(), groups)
}
