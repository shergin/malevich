//! A categorical bar chart: labeled bars from a zero baseline, eighth-block
//! precision at the top, labels truncated to their bands. Synthetic data.

use malevich::Frame;
include!("support/svg_card.rs");

fn main() {
    let chart = malevich::bar(
        ["rust", "go", "python", "typescript", "zig"],
        &[68.0, 41.0, 55.0, 62.0, 12.0][..],
    )
    .title("admired languages, % (synthetic)");
    let frame = Frame::plain(56, 14);
    if svg_card(&chart, &frame) {
        return;
    }
    println!("{}", chart.render_best(&frame));
}
