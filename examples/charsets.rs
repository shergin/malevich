//! The charset ladder: one curve at every subpixel density, from solid blocks to
//! braille dots down to plain ASCII. `Frame::detect` conservatively picks quadrants
//! in UTF-8 environments; here every tier is explicit so the trade-off is visible.
//! Sextants (Unicode 13), octants (Unicode 16), and braille need suitable font
//! coverage and may otherwise show as tofu.

use malevich::{Charset, Frame, Line, Plot};
include!("support/svg_card.rs");

fn main() {
    let x: Vec<f64> = (0..90).map(|i| i as f64 * 0.1).collect();
    let y: Vec<f64> = x.iter().map(|v| v.sin() * (v * 0.5).cos()).collect();
    let tiers = [
        (
            Charset::Octants,
            "Octants — 2x4 solid blocks (Unicode 16, densest ink)",
        ),
        (
            Charset::Sextants,
            "Sextants — 2x3 solid blocks (Unicode 13)",
        ),
        (
            Charset::Quadrants,
            "Quadrants — 2x2 solid blocks (the conservative UTF-8 default)",
        ),
        (Charset::HalfBlocks, "Half blocks — 1x2"),
        (Charset::Braille, "Braille — 2x4 dots (dense opt-in)"),
        (Charset::Ascii, "ASCII — 1x1, the guaranteed fallback"),
    ];
    let cards: Vec<_> = tiers
        .iter()
        .map(|(charset, _)| {
            let frame = Frame {
                charset: *charset,
                ..Frame::plain(60, 8)
            };
            let plot = Plot::new().layer(Line::xy(&x[..], &y[..]));
            (plot, frame)
        })
        .collect();
    let refs: Vec<_> = cards.iter().map(|(plot, frame)| (plot, *frame)).collect();
    if svg_card_frames(&refs) {
        return;
    }
    for ((_, label), (plot, frame)) in tiers.iter().zip(&cards) {
        println!("{label}");
        println!("{}\n", plot.render(frame));
    }
}
