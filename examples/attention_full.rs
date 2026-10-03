//! A full-context attention matrix in a terminal: 1024×1024 weights — a
//! million cells — reduced honestly onto a few thousand screen buckets. Every
//! bucket owns the cells whose centers fall inside it and shows a reduction
//! over all of them, never a sample. The same matrix renders twice: the mean
//! box filter fades the sparse long-range spikes into their buckets; max
//! reduction keeps every spike visible — the diagnostic that per-bucket
//! sampling would silently destroy.

use malevich::scale::Colormap;
use malevich::stat::Reducer;
use malevich::{Cells, Frame, Grid, Plot};
include!("support/svg_card.rs");

fn main() {
    let n = 1024usize;
    // A causal head at scale: geometric local decay along the diagonal, plus a
    // handful of strong long-range associations far off it.
    let mut weights = vec![0.0f64; n * n];
    for query in 0..n {
        for key in query.saturating_sub(48)..=query {
            weights[query * n + key] = (-0.35 * (query - key) as f64).exp();
        }
    }
    for spike in 1..24 {
        let query = (spike * 41) % n;
        let key = (spike * 17) % (query.max(2) - 1).max(1);
        weights[query * n + key] = 0.9;
    }

    // A linear ramp, deliberately: the honesty gap is starkest there — the
    // box filter dilutes an isolated 0.9 into a near-zero bucket mean, while
    // max keeps it at full brightness. (`Colormap::MAGMA.log()` would show
    // the decay tail instead; `attention` in this gallery does exactly that.)
    let pane = |reducer: Reducer, title: &str| {
        Plot::new()
            .layer(
                Cells::matrix(n, &weights[..])
                    .colormap(Colormap::MAGMA)
                    .reduce(reducer),
            )
            .title(title.to_string())
    };
    let mean = pane(Reducer::Mean, "mean-reduced");
    let max = pane(Reducer::Max, "max-reduced");
    let frame = Frame::plain(76, 22);
    if svg_grid(&[&mean, &max], 2, &frame) {
        return;
    }
    let grid = Grid::new(2).with(mean).with(max);
    println!("{}", grid.render(&frame));
}
