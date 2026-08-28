//! A Manhattan plot from the grammar — no preset: association points along a
//! genomic axis, chromosomes alternating two shades (unlabeled layers keep the
//! legend away — the position is the identity), and the genome-wide
//! significance threshold as a `Rule` with a label.

use malevich::{Color, Dash, Frame, Plot, Points, Rule};

fn main() {
    let noise = |i: usize, seed: f64| {
        let hash = (i as f64 * 12.9898 + seed * 78.233).sin() * 43758.5453;
        (hash - hash.floor()) * 2.0 - 1.0
    };
    // 12 chromosomes of shrinking size; a handful of loci carry real signal.
    let sizes = [180, 160, 150, 130, 120, 110, 95, 85, 75, 70, 60, 55];
    let hits = [2usize, 6, 9];
    let (mut even_x, mut even_y) = (Vec::new(), Vec::new());
    let (mut odd_x, mut odd_y) = (Vec::new(), Vec::new());
    let mut position = 0usize;
    for (chromosome, &size) in sizes.iter().enumerate() {
        for i in 0..size {
            let mut p = noise(position + i, 1.0).abs() * 2.8;
            if hits.contains(&chromosome) {
                // A peak near the middle of the chromosome.
                let center = (i as f64 - size as f64 / 2.0).abs() / size as f64;
                let lift = (0.5 - center).max(0.0) * 2.0;
                p += lift * (6.5 + noise(position + i, 5.0) * 1.5) * lift;
            }
            let x = (position + i) as f64;
            if chromosome % 2 == 0 {
                even_x.push(x);
                even_y.push(p);
            } else {
                odd_x.push(x);
                odd_y.push(p);
            }
        }
        position += size;
    }
    let plot = Plot::new()
        .layer(Points::xy(&even_x[..], &even_y[..]).color(Color::Rgb(0, 114, 178)))
        .layer(Points::xy(&odd_x[..], &odd_y[..]).color(Color::Rgb(86, 180, 233)))
        .layer(Rule::h(5.0).label("genome-wide").dash(Dash::Dashed))
        .title("association scan (synthetic)")
        .x_label("genomic position")
        .y_label("-log10 p");
    println!("{}", plot.render_best(&Frame::plain(76, 20)));
}
