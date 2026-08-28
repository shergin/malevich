//! The witness for docs/principles/presets-are-packaging.md, spliced by
//! `regen_docs`: the `hist` preset and its explicit grammar expansion render
//! byte-identically, asserted here before printing.

use malevich::{Frame, Plot, mark::Bars, stat::Bins};

fn main() {
    let samples: Vec<f64> = (0..400)
        .map(|i| 5.0 + 2.0 * (i as f64 * 0.7).sin() + 1.3 * (i as f64 * 1.3).sin())
        .collect();
    let frame = Frame::plain(55, 10);

    let preset = malevich::hist(&samples[..]).render(&frame);

    let bins = Bins::auto(&samples, 60).expect("finite samples bin");
    let counts: Vec<f64> = bins.counts().iter().map(|&count| count as f64).collect();
    let grammar = Plot::new()
        .layer(Bars::spans(bins.start(), bins.width(), &counts[..]))
        .render(&frame);

    assert_eq!(preset, grammar, "the preset must equal its expansion");
    println!("hist(&samples) == Bins::auto + Bars::spans, byte for byte:");
    println!("{preset}");
}
