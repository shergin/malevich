//! A ridgeline of gradient distributions over training — the TensorBoard
//! histogram dashboard, and the terminal's honest answer to "draw me a 3D
//! surface": rows rendered back to front at fixed elevation, each a lifted
//! KDE drawn in the corners style so nearer rows overwrite the cells they
//! cross. No camera, no projection machinery — a painter's algorithm over
//! marks that already existed. The story in the shape: gradients start wide
//! and drift, then sharpen toward zero as training converges.

use malevich::stat::kde;
use malevich::{Frame, Line, LineStyle, Plot};

fn main() {
    let mut state = 7u64;
    let mut uniform = || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (state >> 33) as f64 / (1u64 << 31) as f64
    };

    // Eight epochs of "gradients": rough gaussians whose spread collapses and
    // whose center drifts to zero as training settles.
    let epochs: Vec<Vec<f64>> = (0..8)
        .map(|epoch| {
            let progress = epoch as f64 / 7.0;
            let sigma = 1.1 - 0.85 * progress;
            let center = 0.8 * (1.0 - progress);
            (0..600)
                .map(|_| {
                    let rough = uniform() + uniform() + uniform() - 1.5;
                    center + sigma * rough
                })
                .collect()
        })
        .collect();

    // Painter's algorithm: the oldest epoch is the farthest row, drawn first
    // at the highest lift; each nearer row overwrites what it crosses.
    let mut plot = Plot::new().title("gradient distribution by epoch");
    for (epoch, gradients) in epochs.iter().enumerate().rev() {
        let lift = (7 - epoch) as f64 * 0.55;
        let (xs, density) = kde(gradients, 200).expect("finite sample");
        let lifted: Vec<f64> = density.iter().map(|d| lift + d * 1.6).collect();
        plot = plot.layer(Line::xy(xs, lifted).style(LineStyle::Corners));
    }
    println!("{}", plot.render(&Frame::plain(64, 24)));
}
