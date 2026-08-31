//! The complete first look: the same data as shape and as numbers. A box plot
//! summarizes the penguin flippers visually; the describe table below it is
//! the identical statistics — the very same type-7 quartiles — as text. No
//! figure API is involved: a plot renders to a `String`, so a chart with its
//! stat table is two renders printed in order, each at its own natural
//! height. `Grid` is for equal panes; unequal panes are just `println!`.

use malevich::Frame;

fn main() {
    let (species, groups) = penguin_flippers();
    let refs: Vec<&[f64]> = groups.iter().map(Vec::as_slice).collect();
    let chart =
        malevich::box_plot(species.clone(), refs.clone()).title("flipper length by species (mm)");
    println!("{}", chart.render_best(&Frame::portable(72, 13)));
    // The table sits directly below at the tight-table height (rows + 2,
    // untitled), sharing the frame width so the two read as one figure.
    let table = malevich::describe(species, refs);
    println!("{}", table.render_best(&Frame::portable(72, 5)));
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
