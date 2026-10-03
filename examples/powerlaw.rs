//! Log-log axes: a power law renders as a straight line, with decade ticks
//! (`10²`-style) on both axes. Values at or below zero would become gaps — a log
//! axis cannot place them honestly.

use malevich::{Frame, Line, Plot};
include!("support/svg_card.rs");

fn main() {
    let plot = Plot::new()
        .layer(Line::function(1.0..100_000.0, |x| 0.5 * x.powf(1.5)).label("0.5 x^1.5"))
        .layer(Line::function(1.0..100_000.0, |x| 20.0 * x.sqrt()).label("20 sqrt x"))
        .title("power laws on log-log axes")
        .log_x()
        .log_y();
    let frame = Frame::plain(64, 16);
    if svg_card(&plot, &frame) {
        return;
    }
    println!("{}", plot.render_best(&frame));
}
