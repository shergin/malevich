//! Events per calendar month on a time axis. `stat::calendar_bins` counts
//! per bucket — months of their true length, empty ones kept — and
//! `Bars::intervals` draws each bucket between its own edges, so February is
//! narrower than March and a quiet month shows as a gap in the bars, not
//! in the axis. Synthetic commit timestamps.

use malevich::stat::{TimeUnit, calendar_bins};
use malevich::{Bars, Frame, Plot, Scale};
include!("support/svg_card.rs");

fn main() {
    // Commits over fourteen months, bursty, with a silent August.
    let start = 1_735_689_600.0; // 2025-01-01 00:00 UTC
    let stamps: Vec<f64> = (0..900)
        .map(|i| {
            let day = ((i * 7919) % 425) as f64;
            let hour = ((i * 104_729) % 24) as f64;
            start + day * 86_400.0 + hour * 3_600.0
        })
        .filter(|t| !(*t >= start + 212.0 * 86_400.0 && *t < start + 243.0 * 86_400.0))
        .collect();

    let bins = calendar_bins(&stamps, TimeUnit::Month).expect("finite timestamps");
    let counts: Vec<f64> = bins.counts().iter().map(|&count| count as f64).collect();
    let plot = Plot::new()
        .layer(Bars::intervals(bins.starts(), bins.ends(), counts))
        .y_scale(Scale::Integer)
        .time_x()
        .title("commits per month (synthetic)");
    let frame = Frame::plain(72, 14);
    if svg_card(&plot, &frame) {
        return;
    }
    println!("{}", plot.render_best(&frame));
}
