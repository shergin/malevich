//! A decision boundary from the grammar, no preset: a `Cells::classes` grid of
//! model predictions colors the feature plane by class, with the training
//! points on top — the scikit-learn classifier panel, in a terminal. In plain
//! output each region keeps its own shade and the legend swatches carry the
//! same glyphs, so the boundary survives a pipe.

use malevich::{Cells, Frame, Plot, PointStyle, Points};

/// Three deterministic training blobs (a tiny LCG stands in for a dataset —
/// loading files is the host's job).
fn blobs() -> (Vec<f64>, Vec<f64>, Vec<&'static str>) {
    let centers = [
        (-1.6, -1.0, "adelie"),
        (1.7, -0.6, "gentoo"),
        (0.1, 1.6, "chinstrap"),
    ];
    let mut state = 9u64;
    let mut noise = || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((state >> 33) as f64 / (1u64 << 31) as f64) - 1.0
    };
    let (mut x, mut y, mut class) = (Vec::new(), Vec::new(), Vec::new());
    for &(cx, cy, label) in &centers {
        for _ in 0..14 {
            x.push(cx + noise() * 0.9);
            y.push(cy + noise() * 0.9);
            class.push(label);
        }
    }
    (x, y, class)
}

fn main() {
    let (x, y, class) = blobs();

    // 5-nearest-neighbor predictions over the feature plane.
    let n = 96usize;
    let (lo, hi) = (-3.2, 3.2);
    let mut regions = Vec::with_capacity(n * n);
    for row in 0..n {
        for column in 0..n {
            let px = lo + (hi - lo) * (column as f64 + 0.5) / n as f64;
            let py = lo + (hi - lo) * (row as f64 + 0.5) / n as f64;
            let mut nearest: Vec<(f64, &str)> = x
                .iter()
                .zip(&y)
                .zip(&class)
                .map(|((&sx, &sy), &label)| ((sx - px).powi(2) + (sy - py).powi(2), label))
                .collect();
            nearest.sort_by(|a, b| a.0.total_cmp(&b.0));
            let mut votes: Vec<(&str, usize)> = Vec::new();
            for &(_, label) in nearest.iter().take(5) {
                match votes.iter_mut().find(|(seen, _)| *seen == label) {
                    Some((_, count)) => *count += 1,
                    None => votes.push((label, 1)),
                }
            }
            regions.push(votes.iter().max_by_key(|(_, count)| *count).unwrap().0);
        }
    }

    // The regions already say which class is where; the training points only
    // need to be visible on top of the fill, so they draw as one plain layer.
    let plot = Plot::new()
        .layer(Cells::classes(n, regions).extents((lo, hi), (lo, hi)))
        .layer(Points::xy(&x[..], &y[..]).style(PointStyle::Cross))
        .title("5-NN decision regions");
    println!("{}", plot.render(&Frame::plain(60, 22)));
}
