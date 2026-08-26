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
    let grid = Grid::new(2)
        .with(pane(Reducer::Mean, "mean-reduced"))
        .with(pane(Reducer::Max, "max-reduced"));
    println!("{}", grid.render(&Frame::plain(76, 22)));
}
