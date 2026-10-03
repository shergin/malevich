//! Error bars: a Range interval around each measured point.

use malevich::Frame;
include!("support/svg_card.rs");

fn main() {
    let x: Vec<f64> = (1..=8).map(|i| i as f64).collect();
    let y: Vec<f64> = x.iter().map(|v| 3.0 + (v * 0.8).sin() * 2.0).collect();
    let error: Vec<f64> = x
        .iter()
        .map(|v| 0.3 + (v * 1.7).cos().abs() * 0.5)
        .collect();
    let chart = malevich::error_bars(&x[..], &y[..], &error[..])
        .title("measurements with uncertainty (synthetic)");
    let frame = Frame::plain(52, 13);
    if svg_card(&chart, &frame) {
        return;
    }
    println!("{}", chart.render_best(&frame));
}
