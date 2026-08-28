//! A loss landscape with an optimizer trajectory — the gradient-descent
//! chart, composed from marks that all existed already: a dense Cells grid of
//! the surface on a logarithmic colormap (loss spans decades; a linear ramp
//! would flatten the basins), and the momentum path as a Line with its steps
//! as Points. The surface samples bilinearly on dense targets, the path is
//! graded dim-to-bright by step — time reads along the line. The four basins
//! of Himmelblau's function read as dark wells, and the trajectory
//! overshoots and curls into one of them.

use malevich::scale::Colormap;
use malevich::{Cells, Frame, Line, Plot, PointStyle, Points};

fn himmelblau(x: f64, y: f64) -> f64 {
    (x * x + y - 11.0).powi(2) + (x + y * y - 7.0).powi(2)
}

fn gradient(x: f64, y: f64) -> (f64, f64) {
    (
        4.0 * x * (x * x + y - 11.0) + 2.0 * (x + y * y - 7.0),
        2.0 * (x * x + y - 11.0) + 4.0 * y * (x + y * y - 7.0),
    )
}

fn main() {
    let (lo, hi) = (-5.0, 5.0);
    let n = 220usize;
    let surface: Vec<f64> = (0..n * n)
        .map(|index| {
            let x = lo + (hi - lo) * ((index % n) as f64 + 0.5) / n as f64;
            let y = lo + (hi - lo) * ((index / n) as f64 + 0.5) / n as f64;
            himmelblau(x, y)
        })
        .collect();

    // Gradient descent with momentum from a bad corner.
    let (mut x, mut y) = (-0.27, -4.6);
    let (mut vx, mut vy) = (0.0, 0.0);
    let (mut path_x, mut path_y) = (vec![x], vec![y]);
    for _ in 0..48 {
        let (gx, gy) = gradient(x, y);
        vx = 0.82 * vx - 8.0e-4 * gx;
        vy = 0.82 * vy - 8.0e-4 * gy;
        x += vx;
        y += vy;
        path_x.push(x);
        path_y.push(y);
    }

    let progress: Vec<f64> = (0..path_x.len()).map(|step| step as f64).collect();
    let plot = Plot::new()
        .layer(
            Cells::matrix(n, &surface[..])
                .extents((lo, hi), (lo, hi))
                .colormap(Colormap::VIRIDIS.log())
                .smooth(),
        )
        .layer(Line::xy(&path_x[..], &path_y[..]).grade(&progress[..], Colormap::GREYS))
        // Circles, not subpixel dots: glyph-drawn markers stay visible on top
        // of the filled surface in cell output (the line takes over in pixel
        // output, where it draws at device resolution).
        .layer(Points::xy(&path_x[..], &path_y[..]).style(PointStyle::Circle))
        .title("momentum on Himmelblau");
    println!("{}", plot.render(&Frame::plain(62, 24)));
}
