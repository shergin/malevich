//! Step charts: `stairs` holds values flat between indices; `ecdf` climbs a
//! distribution from zero to one.

use malevich::{Frame, Grid};
include!("support/svg_card.rs");

fn main() {
    let requests = [12.0, 12.0, 19.0, 14.0, 23.0, 23.0, 31.0, 26.0, 18.0, 18.0];
    let samples: Vec<f64> = (0..300)
        .map(|i| {
            let i = i as f64;
            ((i * 0.731).sin() + (i * 1.13).sin()) * 2.5 + 6.0
        })
        .collect();
    let stairs = malevich::stairs(&requests[..]).title("requests per window");
    let ecdf = malevich::ecdf(&samples[..]).title("latency ecdf");
    let frame = Frame::plain(76, 13);
    if svg_grid(&[&stairs, &ecdf], 2, &frame) {
        return;
    }
    let grid = Grid::new(2).with(stairs).with(ecdf);
    println!("{}", grid.render(&frame));
}
