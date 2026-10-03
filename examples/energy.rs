//! Stacked areas via the Stack stat: each layer sits on the sum of the ones below.
//! In color modes the bands read by palette; plain output shows the envelope.
//! Synthetic data.

use malevich::{Area, Frame, Plot};
include!("support/svg_card.rs");

fn main() {
    let x: Vec<f64> = (0..80).map(f64::from).collect();
    let solar: Vec<f64> = x.iter().map(|v| 3.0 + (v * 0.2).sin() + v * 0.02).collect();
    let wind: Vec<f64> = x
        .iter()
        .map(|v| 2.0 + (v * 0.13).cos().abs() * 1.5)
        .collect();
    let hydro: Vec<f64> = x.iter().map(|v| 1.0 + (v * 0.07).sin().abs()).collect();

    let bands = malevich::stat::stack(&[&solar, &wind, &hydro]);
    let mut plot = Plot::new().title("energy mix, stacked (synthetic)");
    for ((low, high), label) in bands.iter().zip(["solar", "wind", "hydro"]) {
        plot = plot.layer(Area::between(&x[..], &low[..], &high[..]).label(label));
    }
    let frame = Frame::plain(64, 16);
    if svg_card(&plot, &frame) {
        return;
    }
    println!("{}", plot.render_best(&frame));
}
