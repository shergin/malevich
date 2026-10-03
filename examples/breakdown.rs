//! Breakdown bars: how each region's electricity splits by source, as
//! horizontal 100 % stacks — `stat::stack_with` under `StackOffset::Normalize`
//! turns every row into bands that fill `[0, 1]`, one `Bars::base` layer per
//! source carries them sideways, and the x axis reads as a fraction. Each
//! segment wide enough to hold it carries its share as a centered `Text`
//! annotation, which keeps the segment's color as its background in a
//! terminal and stands in for the color in a plain pipe. A region with no
//! generation at all draws nothing rather than a full bar of its first
//! source. Synthetic data.

use malevich::stat::{StackOffset, StackOptions, stack_with};
use malevich::{Align, Bars, Frame, Plot, Scale, Text};
include!("support/svg_card.rs");

fn main() {
    let regions = ["north", "coast", "plains", "island", "outage"];
    let hydro = [42.0, 3.0, 8.0, 0.0, 0.0];
    let wind = [18.0, 30.0, 51.0, 7.0, 0.0];
    let solar = [5.0, 22.0, 13.0, 11.0, 0.0];
    let gas = [20.0, 45.0, 28.0, 82.0, 0.0];
    let sources = [
        (&hydro[..], "hydro"),
        (&wind[..], "wind"),
        (&solar[..], "solar"),
        (&gas[..], "gas"),
    ];
    let series: Vec<&[f64]> = sources.iter().map(|(values, _)| *values).collect();
    let bands = stack_with(&series, StackOptions::new().offset(StackOffset::Normalize));

    let mut plot = Plot::new()
        .y_scale(Scale::bands(regions))
        .title("electricity by source, share of each region (synthetic)")
        .x_label("share");
    for ((low, high), (_, label)) in bands.iter().zip(sources) {
        let lengths: Vec<f64> = high.iter().zip(low).map(|(h, l)| h - l).collect();
        plot = plot.layer(
            Bars::new(regions, lengths)
                .base(&low[..])
                .horizontal()
                .label(label),
        );
    }
    // The shares, on their segments: band k of the y axis is row k, and a
    // segment narrower than a tenth would not hold three glyphs.
    for (low, high) in &bands {
        for (row, (l, h)) in low.iter().zip(high).enumerate() {
            if h - l >= 0.1 {
                let share = format!("{:.0}%", (h - l) * 100.0);
                plot = plot.layer(Text::at((l + h) / 2.0, row as f64, share).align(Align::Center));
            }
        }
    }
    // Five bands in five plot rows: title, legend, axis, and x label make ten.
    let frame = Frame::plain(66, 10);
    if svg_card(&plot, &frame) {
        return;
    }
    println!("{}", plot.render_best(&frame));
}
