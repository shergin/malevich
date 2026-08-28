//! The witness for docs/principles/full-draw-oracle.md, spliced by
//! `regen_docs`: one hundred thousand points with three one-sample spikes,
//! reduced by the auto-inserted M4. The spikes survive because per-column
//! extremes are kept by construction; the byte-equality against the raw
//! raster is asserted by the crate's oracle test.

use malevich::{Frame, Line, Plot};

fn main() {
    let mut y: Vec<f64> = (0..100_000)
        .map(|i| (i as f64 * 0.0005).sin() * 2.0 + (i as f64 * 0.00013).cos())
        .collect();
    for index in [17_777, 50_000, 83_333] {
        y[index] = 8.0;
    }
    let chart = Plot::new()
        .layer(Line::y(&y[..]))
        .title("100,000 points, three one-sample spikes");
    println!("{}", chart.render(&Frame::plain(66, 12)));
}
