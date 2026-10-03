//! A 2D histogram: point density on a grid, empty bins left blank. The bell-shaped
//! clusters come from sums of incommensurate sines (a poor man's central limit).

use malevich::Frame;
include!("support/svg_card.rs");

fn main() {
    let bell = |i: f64, seed: f64| -> f64 {
        ((i * 0.97 + seed).sin() + (i * 1.31 + seed * 2.0).sin() + (i * 2.63 + seed * 3.0).sin())
            / 3.0
    };
    let n = 6000;
    let x: Vec<f64> = (0..n)
        .map(|i| {
            let i = i as f64;
            if i as i64 % 2 == 0 {
                3.0 + bell(i, 1.0) * 1.8
            } else {
                7.0 + bell(i, 4.0) * 1.2
            }
        })
        .collect();
    let y: Vec<f64> = (0..n)
        .map(|i| {
            let i = i as f64;
            if i as i64 % 2 == 0 {
                3.0 + bell(i, 7.0) * 1.4
            } else {
                6.5 + bell(i, 9.0) * 1.7
            }
        })
        .collect();
    let chart = malevich::hist2d(&x[..], &y[..]).title("two clusters, binned (synthetic)");
    let frame = Frame::plain(60, 17);
    if svg_card(&chart, &frame) {
        return;
    }
    println!("{}", chart.render_best(&frame));
}
