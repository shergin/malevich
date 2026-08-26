//! A spectrogram: time × frequency power as dense Cells with extents in
//! seconds and hertz, a log frequency axis, and a log colormap — power spans
//! decades, and both logs are one builder call each. No FFT lives in this
//! crate (signal processing is the host's job); the example synthesizes the
//! time–frequency energy of a scene analytically: an exponential chirp (a
//! straight ridge on a log frequency axis), a steady 440 Hz tone, and one
//! broadband click.

use malevich::scale::Colormap;
use malevich::{Cells, Frame, Plot};

fn main() {
    let (columns, rows) = (360usize, 240usize);
    let (t0, t1) = (0.0f64, 4.0);
    let (f0, f1) = (60.0f64, 8000.0);

    let power: Vec<f64> = (0..columns * rows)
        .map(|index| {
            let t = t0 + (t1 - t0) * ((index % columns) as f64 + 0.5) / columns as f64;
            let f = f0 + (f1 - f0) * ((index / columns) as f64 + 0.5) / rows as f64;
            let log_f = f.ln();

            // The chirp sweeps 100 Hz to 4 kHz exponentially over 4 seconds.
            let chirp_f = 100.0 * (4000.0f64 / 100.0).powf(t / 4.0);
            let chirp = (-((log_f - chirp_f.ln()) / 0.09).powi(2)).exp();
            let tone = 0.5 * (-((log_f - 440.0f64.ln()) / 0.05).powi(2)).exp();
            let click = 0.8 * (-((t - 2.6) / 0.015).powi(2)).exp();
            1e-4 + chirp + tone + click
        })
        .collect();

    let plot = Plot::new()
        .layer(
            Cells::matrix(columns, &power[..])
                .extents((t0, t1), (f0, f1))
                .colormap(Colormap::MAGMA.log()),
        )
        .log_y()
        .x_label("s")
        .y_label("Hz")
        .colorbar()
        .title("spectrogram");
    println!("{}", plot.render(&Frame::plain(66, 22)));
}
