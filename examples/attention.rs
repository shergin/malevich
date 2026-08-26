//! An attention map: one head's weights over a short sequence, token labels on
//! both axes through band scales, and a logarithmic colormap — attention spans
//! decades, and a linear ramp would collapse everything but the diagonal into
//! black. The causal mask's zeros have no logarithmic position and render as
//! honest gaps, so the masked triangle stays blank instead of faking a shade.

use malevich::scale::Colormap;
use malevich::{Cells, Frame, Plot, Scale};

fn main() {
    let tokens = ["The", "robot", "ate", "the", "red", "apple", "."];
    let n = tokens.len();
    // A causal head in miniature: each query attends to itself and its recent
    // past with geometrically decaying weight, plus one long-range association
    // — the period looks back at the subject. Rows are normalized like a
    // softmax; the masked future stays exactly zero.
    let mut weights = vec![0.0f64; n * n];
    for query in 0..n {
        for key in 0..=query {
            weights[query * n + key] = (-1.9 * (query - key) as f64).exp();
        }
    }
    weights[6 * n + 1] = 0.35;
    for query in 0..n {
        let row = &mut weights[query * n..(query + 1) * n];
        let sum: f64 = row.iter().sum();
        for weight in row {
            *weight /= sum;
        }
    }

    let plot = Plot::new()
        .layer(Cells::matrix(n, &weights[..]).colormap(Colormap::MAGMA.log()))
        .x_scale(Scale::bands(tokens))
        .y_scale(Scale::bands(tokens))
        .x_label("key")
        .y_label("query")
        .colorbar()
        .title("attention, layer 7 head 3");
    println!("{}", plot.render(&Frame::plain(66, 20)));
}
