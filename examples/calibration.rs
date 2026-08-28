//! A reliability diagram from the grammar, no preset: predicted confidence
//! binned with `stat::binned` and a Mean reducer over the 0/1 outcomes —
//! accuracy per confidence bin — against the diagonal of perfect calibration.
//! The model below is overconfident, the classic failure: its curve sags
//! under the diagonal at the high-confidence end.

use malevich::stat::{Bins, Reducer, binned};
use malevich::{Dash, Frame, Line, Plot, PointStyle, Points};

fn main() {
    // A deterministic overconfident classifier: predictions cluster near the
    // extremes, but the true hit rate is closer to the middle than claimed.
    let mut state = 11u64;
    let mut uniform = || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (state >> 33) as f64 / (1u64 << 31) as f64
    };
    let (mut confidence, mut correct) = (Vec::new(), Vec::new());
    for _ in 0..4000 {
        let claimed = uniform();
        // The real accuracy pulls the claim 40% of the way back to a coin flip.
        let actual = 0.5 + (claimed - 0.5) * 0.6;
        confidence.push(claimed);
        correct.push(if uniform() < actual { 1.0 } else { 0.0 });
    }

    let bins = Bins::new(0.0, 0.1, 10);
    let accuracy = binned(&confidence, &correct, &bins, Reducer::Mean);
    let centers: Vec<f64> = (0..10).map(|bin| 0.05 + 0.1 * bin as f64).collect();

    let plot = Plot::new()
        .layer(
            Line::xy(&[0.0, 1.0][..], &[0.0, 1.0][..])
                .label("perfect")
                .dash(Dash::Dotted),
        )
        .layer(Line::xy(&centers[..], &accuracy[..]).label("model"))
        .layer(Points::xy(&centers[..], &accuracy[..]).style(PointStyle::Circle))
        .x_label("claimed confidence")
        .y_label("observed accuracy")
        .title("reliability");
    println!("{}", plot.render(&Frame::plain(58, 22)));
}
