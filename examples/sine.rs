//! Function sampling: curves drawn from `f(x)`, one sample per subpixel column.

use malevich::{Frame, Line, Plot};
include!("support/svg_card.rs");

fn main() {
    let plot = Plot::new()
        .layer(Line::function(0.0..12.6, f64::sin))
        .layer(Line::function(0.0..12.6, |x| (x * 0.5).cos() * 0.6))
        .title("sin(x) and 0.6 cos(x/2)");
    let frame = Frame::plain(72, 16);
    if svg_card(&plot, &frame) {
        return;
    }
    println!("{}", plot.render_best(&frame));
}
