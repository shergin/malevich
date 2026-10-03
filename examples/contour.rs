//! The MATLAB peaks function: three humps and dips traced as iso-lines by
//! marching squares — levels picked by the tick algorithm, colored along the
//! colormap, labeled in the legend.

use malevich::Frame;
include!("support/svg_card.rs");

fn main() {
    let (columns, rows) = (46, 46);
    let mut z = Vec::with_capacity(columns * rows);
    for r in 0..rows {
        for c in 0..columns {
            let x = -3.0 + 6.0 * c as f64 / (columns - 1) as f64;
            let y = -3.0 + 6.0 * r as f64 / (rows - 1) as f64;
            z.push(peaks(x, y));
        }
    }
    let chart = malevich::contour(columns, &z[..]).title("the peaks function");
    let frame = Frame::plain(72, 24);
    if svg_card(&chart, &frame) {
        return;
    }
    println!("{}", chart.render_best(&frame));
}

fn peaks(x: f64, y: f64) -> f64 {
    3.0 * (1.0 - x).powi(2) * (-x * x - (y + 1.0).powi(2)).exp()
        - 10.0 * (x / 5.0 - x.powi(3) - y.powi(5)) * (-x * x - y * y).exp()
        - (-(x + 1.0).powi(2) - y * y).exp() / 3.0
}
