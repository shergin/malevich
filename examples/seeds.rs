//! Training curves across random seeds: five noisy runs pooled into
//! per-step quantiles with `stat::binned` — the p10–p90 band as an Area, the
//! median as a line, and its `stat::ewma` smoothing on top, on a log y axis.
//! The two idioms every experiment tracker draws (the seed band and the
//! smoothed curve), composed from the reducer vocabulary and one scan.

use malevich::stat::{Bins, Reducer, binned, ewma};
use malevich::{Area, Frame, Line, LineStyle, Plot};

fn main() {
    let steps_per_run = 400usize;
    let mut state = 3u64;
    let mut uniform = || {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (state >> 33) as f64 / (1u64 << 31) as f64
    };

    // Five seeds: shared decay, per-seed floor and noise.
    let (mut steps, mut losses) = (Vec::new(), Vec::new());
    for seed in 0..5 {
        let floor = 0.30 + 0.05 * seed as f64;
        for step in 0..steps_per_run {
            let decay = 2.4 * (-(step as f64) / 90.0).exp();
            let noise = 1.0 + 0.55 * (uniform() - 0.5);
            steps.push(step as f64);
            losses.push((floor + decay) * noise);
        }
    }

    let bins = Bins::new(0.0, 8.0, 50);
    let p10 = binned(&steps, &losses, &bins, Reducer::Percentile(0.1));
    let p50 = binned(&steps, &losses, &bins, Reducer::Median);
    let p90 = binned(&steps, &losses, &bins, Reducer::Percentile(0.9));
    let smooth = ewma(&p50, 0.8);
    let centers: Vec<f64> = (0..50).map(|bin| 4.0 + 8.0 * bin as f64).collect();

    let plot = Plot::new()
        .layer(Area::between(&centers[..], &p10[..], &p90[..]).label("p10-p90"))
        .layer(Line::xy(&centers[..], &p50[..]).label("median"))
        // Corners glyphs keep the smoothed curve visible over the band fill
        // in cell output; subpixel lines would drown in it.
        .layer(
            Line::xy(&centers[..], &smooth[..])
                .style(LineStyle::Corners)
                .label("ewma"),
        )
        .log_y()
        .x_label("step")
        .title("loss across 5 seeds");
    println!("{}", plot.render(&Frame::plain(64, 22)));
}
