//! An ROC curve from the grammar, no preset: `stat::roc` sweeps the
//! thresholds, `stat::auc` puts the area in the title, and the chance
//! diagonal is just another labeled line. The curve's distance from the
//! diagonal is the classifier; everything else is furniture.

use malevich::{Dash, Frame, Line, Plot, stat};

fn main() {
    // A deterministic classifier in miniature: positive scores center higher
    // than negatives with real overlap (an LCG stands in for a model).
    let mut state = 5u64;
    let mut uniform = || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (state >> 33) as f64 / (1u64 << 31) as f64
    };
    let (mut scores, mut labels) = (Vec::new(), Vec::new());
    for _ in 0..300 {
        // Rough gaussians from averaged uniforms; separation ~1 sigma.
        let noise = uniform() + uniform() + uniform() - 1.5;
        let positive = uniform() < 0.5;
        scores.push(if positive { 0.55 + noise } else { noise });
        labels.push(positive);
    }

    let (fpr, tpr) = stat::roc(&scores, &labels);
    let area = stat::auc(&fpr, &tpr);
    let plot = Plot::new()
        .layer(
            Line::xy(&[0.0, 1.0][..], &[0.0, 1.0][..])
                .label("chance")
                .dash(Dash::Dotted),
        )
        .layer(Line::xy(&fpr[..], &tpr[..]).label("model"))
        .x_label("false positive rate")
        .y_label("true positive rate")
        .title(format!("ROC, AUC {area:.3}"));
    println!("{}", plot.render(&Frame::plain(58, 22)));
}
