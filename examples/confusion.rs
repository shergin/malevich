//! A confusion matrix from the grammar, no preset: a Cells matrix on Bands
//! axes — the class names label both rows and columns — with per-cell counts
//! as Text marks. Row 0 is the top band, so the chart reads like the printed
//! matrix: true classes down, predictions across.

use malevich::scale::Colormap;
use malevich::{Cells, Frame, Plot, Scale, Text};

fn main() {
    let classes = ["cat", "dog", "bird"];
    let counts = [
        38.0, 2.0, 0.0, //
        3.0, 33.0, 4.0, //
        1.0, 5.0, 34.0, //
    ];
    let mut plot = Plot::new()
        .layer(Cells::matrix(classes.len(), &counts[..]).colormap(Colormap::GREYS))
        .x_scale(Scale::bands(classes))
        .y_scale(Scale::bands(classes))
        .x_label("predicted")
        .y_label("true")
        .title("validation confusion");
    for (index, &count) in counts.iter().enumerate() {
        let (column, row) = (index % classes.len(), index / classes.len());
        plot = plot.layer(Text::at(column as f64, row as f64, format!("{count:.0}")));
    }
    println!("{}", plot.render(&Frame::plain(46, 16)));
}
