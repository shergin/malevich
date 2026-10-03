//! A vector field: spiral flow into a sink, one arrow per grid point, drawn in
//! data coordinates so the arrows scale with the axes.

use malevich::Frame;
include!("support/svg_card.rs");

fn main() {
    let mut x = Vec::new();
    let mut y = Vec::new();
    let mut u = Vec::new();
    let mut v = Vec::new();
    for row in 0..9 {
        for column in 0..13 {
            let px = -2.4 + 0.4 * column as f64;
            let py = -1.6 + 0.4 * row as f64;
            x.push(px);
            y.push(py);
            u.push(0.30 * -py - 0.10 * px);
            v.push(0.30 * px - 0.10 * py);
        }
    }
    let chart = malevich::quiver(&x[..], &y[..], &u[..], &v[..])
        .title("spiral flow into a sink")
        .x_label("x")
        .y_label("y");
    let frame = Frame::plain(72, 22);
    if svg_card(&chart, &frame) {
        return;
    }
    println!("{}", chart.render_best(&frame));
}
