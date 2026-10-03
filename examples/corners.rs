//! The asciichart homage: `LineStyle::Corners` draws one box-drawing glyph per
//! column — the instantly-legible low-fi style that kroitor/asciichart made famous
//! (credited in ACKNOWLEDGEMENTS.md), here with real axes underneath it.

use malevich::{Frame, Line, LineStyle, Plot};
include!("support/svg_card.rs");

fn main() {
    let values: Vec<f64> = (0..60)
        .map(|i| 15.0 * (i as f64 * std::f64::consts::PI / 30.0).sin())
        .collect();
    let chart = Plot::new()
        .layer(Line::y(&values[..]).style(LineStyle::Corners))
        .title("the corners style");
    let frame = Frame::portable(70, 16);
    if svg_card(&chart, &frame) {
        return;
    }
    println!("{}", chart.render(&frame));
}
