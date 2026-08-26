//! First-layer convolution filters as images: a bank of oriented Gabor filters
//! with color opponency, the pattern AlexNet's first layer famously learns —
//! synthesized here, because decoding weight files is the host's job. Each pane
//! is a `Cells::rgb` grid: direct colors, no colormap, quantized honestly down
//! the color ladder; in a plain pipe each pixel shows its luma on the shade
//! ramp, so the orientations stay readable without color.

use malevich::{Cells, Frame, Grid, Plot};

/// One 18×18 Gabor patch at `theta`, with per-channel phase offsets — zero
/// offsets give a grayscale edge detector, nonzero give the red/green and
/// blue/orange opponency the trained filters show.
fn gabor(theta: f64, phases: (f64, f64, f64)) -> Vec<(u8, u8, u8)> {
    let n = 18usize;
    let (sigma, wavelength) = (0.36, 0.5);
    let mut pixels = Vec::with_capacity(n * n);
    for row in 0..n {
        for column in 0..n {
            let x = (column as f64 / (n - 1) as f64) * 2.0 - 1.0;
            let y = (row as f64 / (n - 1) as f64) * 2.0 - 1.0;
            let along = x * theta.cos() + y * theta.sin();
            let across = -x * theta.sin() + y * theta.cos();
            let envelope = (-(along * along + 0.6 * across * across) / (2.0 * sigma * sigma)).exp();
            let carrier = |phase: f64| {
                let wave = (std::f64::consts::TAU * along / wavelength + phase).cos();
                let level = 0.5 + 0.5 * wave * envelope;
                (level * 255.0).round() as u8
            };
            pixels.push((carrier(phases.0), carrier(phases.1), carrier(phases.2)));
        }
    }
    pixels
}

fn main() {
    let banks = [
        (0.0, (0.0, 0.0, 0.0), "0°"),
        (1.0, (0.0, 0.0, 0.0), "57°"),
        (2.1, (0.0, 0.0, 0.0), "120°"),
        (0.5, (0.0, 2.1, 4.2), "29° rgb"),
        (1.6, (0.0, 2.1, 4.2), "92° rgb"),
        (2.6, (0.0, 2.1, 4.2), "149° rgb"),
    ];
    let mut grid = Grid::new(3);
    for (theta, phases, title) in banks {
        grid = grid.with(
            Plot::new()
                .layer(Cells::rgb(18, gabor(theta, phases)))
                .title(title),
        );
    }
    println!("{}", grid.render(&Frame::plain(76, 24)));
}
