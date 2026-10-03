//! Small multiples: a Grid pastes independently rendered plots side by side.
//! Shared axes are a composition — fix them with `y_domain` — not a mode.

use malevich::{Frame, Grid};
include!("support/svg_card.rs");

fn main() {
    let a: Vec<f64> = (0..50).map(|i| (i as f64 * 0.2).sin() * 3.0).collect();
    let b: Vec<f64> = (0..50).map(|i| (i as f64 * 0.13).cos() * 5.0).collect();
    let alpha = malevich::line(&a[..]).title("alpha").y_domain(-6.0, 6.0);
    let beta = malevich::line(&b[..]).title("beta").y_domain(-6.0, 6.0);
    let alpha_dist = malevich::hist(&a[..]).title("alpha dist");
    let beta_dist = malevich::hist(&b[..]).title("beta dist");
    let frame = Frame::plain(76, 22);
    if svg_grid(&[&alpha, &beta, &alpha_dist, &beta_dist], 2, &frame) {
        return;
    }
    let grid = Grid::new(2)
        .with(alpha)
        .with(beta)
        .with(alpha_dist)
        .with(beta_dist);
    println!("{}", grid.render(&frame));
}
