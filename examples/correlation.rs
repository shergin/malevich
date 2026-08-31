//! A correlation matrix, annotated — from the grammar, no preset: signed data
//! on a diverging colormap centered at zero, feature names on band axes, and
//! every coefficient printed in its cell. The annotations keep the field they
//! land on — a glyph over a filled cell takes the cell's color as its
//! background — and each one picks dark or light ink from the luminance
//! underneath, so the numbers read at both ends of the ramp. In plain output
//! the digits replace the shades: the numbers are the values, so nothing is
//! lost in a pipe.

use malevich::scale::Colormap;
use malevich::{Align, Cells, Color, Frame, Plot, Scale, Text};

fn main() {
    let features = ["age", "len", "dep", "mass", "veg", "kcal", "spd", "alt"];
    let n = features.len();
    let grid: Vec<f64> = (0..n * n)
        .map(|i| {
            let (row, column) = ((i / n) as f64, (i % n) as f64);
            if row == column {
                1.0
            } else {
                // Symmetric, decaying with distance, alternating in sign — the
                // shape of a real feature-correlation matrix.
                ((row - column).abs() * -0.35).exp() * ((row + column) * 0.55).cos()
            }
        })
        .collect();

    let colormap = Colormap::RED_BLUE.centered_at(0.0);
    let mut plot = Plot::new()
        .layer(Cells::matrix(n, &grid[..]).colormap(colormap.clone()))
        .x_scale(Scale::bands(features))
        .y_scale(Scale::bands(features))
        .title("feature correlation (synthetic)");
    for (index, &coefficient) in grid.iter().enumerate() {
        let (column, row) = (index % n, index / n);
        // Ink by the luminance under it: dark on the pale middle of the
        // ramp, light on the saturated ends.
        let ink = match colormap.color(colormap.position_in(coefficient, -1.0, 1.0)) {
            Color::Rgb(r, g, b) if u16::from(r) + u16::from(g) + u16::from(b) > 384 => {
                Color::Rgb(32, 32, 32)
            }
            _ => Color::Rgb(235, 235, 230),
        };
        plot = plot.layer(
            Text::at(column as f64, row as f64, format!("{coefficient:+.2}"))
                .align(Align::Center)
                .color(ink),
        );
    }
    println!("{}", plot.render(&Frame::plain(56, 11)));
}
