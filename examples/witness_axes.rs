//! The witness for docs/principles/axes-are-the-product.md, spliced by
//! `regen_docs`: ticks stepping by 0.2 — a value with no exact binary form,
//! the classic float-artifact trap — labeled with exact decimals that parse
//! back to their values.

use malevich::{Frame, Line, Plot};

fn main() {
    let y: Vec<f64> = (0..60)
        .map(|i| 0.3 + 0.3 * (i as f64 * 0.15).sin())
        .collect();
    let chart = Plot::new()
        .layer(Line::y(&y[..]))
        .title("every label an exact decimal");
    println!("{}", chart.render(&Frame::plain(55, 12)));
}
