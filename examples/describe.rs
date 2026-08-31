//! The first look is sometimes a table: the same flippers the box plot draws,
//! as the numbers — count, mean, sd, min, quartiles, max per species. A table
//! is text on band scales, not a widget: rows ride the y band axis, columns
//! the x band axis, and every column is formatted like a tiny axis — uniform
//! decimals, padded to the column's width so numbers meet at the decimal
//! point, centered under its header by the header's own rule.

use malevich::Frame;

fn main() {
    let (species, groups) = penguin_flippers();
    let refs: Vec<&[f64]> = groups.iter().map(Vec::as_slice).collect();
    // Three rows plus title, axis line, and headers — the tight-table height,
    // so every species lands on a consecutive line.
    let chart = malevich::describe(species, refs).title("flipper length by species (mm)");
    println!("{}", chart.render(&Frame::plain(76, 6)));
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
