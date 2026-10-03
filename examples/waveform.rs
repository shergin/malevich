//! Ten million points, one fused pass: the automatically inserted M4 aggregation
//! keeps first/last/min/max per raster column — provably pixel-identical to drawing
//! every point, in a few tens of milliseconds. The x axis shows the shared SI
//! prefix (`2.5M`) that large axes pick automatically.

use malevich::Frame;
include!("support/svg_card.rs");

fn main() {
    let n = 10_000_000;
    let y: Vec<f64> = (0..n)
        .map(|i| {
            let i = i as f64;
            (i * 0.0002).sin() * (i * 0.000013).cos() * 8.0
        })
        .collect();
    let chart = malevich::line(&y[..]).title("10,000,000 points");
    let frame = Frame::plain(72, 16);
    if svg_card(&chart, &frame) {
        return;
    }
    println!("{}", chart.render_best(&frame));
}
