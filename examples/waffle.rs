//! The pie chart's honest form: a waffle. One hundred cells, each a
//! category, on a ten-by-ten grid with the axes off — `Cells::classes` over
//! the grammar, no preset. Shares round to whole cells, so the parts can be
//! counted, and the legend names them; a pie's angles cannot be counted at
//! all. Synthetic shares.

use malevich::{Cells, Frame, Plot};
include!("support/svg_card.rs");

fn main() {
    let shares = [("rust", 46), ("go", 27), ("python", 18), ("other", 9)];
    let classes: Vec<&str> = shares
        .iter()
        .flat_map(|(name, share)| std::iter::repeat_n(*name, *share))
        .collect();
    assert_eq!(classes.len(), 100);
    let plot = Plot::new()
        .layer(Cells::classes(10, classes))
        .axes(false)
        .title("language share, one cell per percent (synthetic)");
    let frame = Frame::plain(56, 14);
    if svg_card(&plot, &frame) {
        return;
    }
    println!("{}", plot.render_best(&frame));
}
