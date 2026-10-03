//! One trading session on a calendar axis. Hour labels say `10:00`, never
//! which day — so the axis prints the day once, at the end of its title
//! row: the context note matplotlib's `ConciseDateFormatter` and Bokeh's
//! `context` put on a time axis, here an automatic layout rule. A synthetic
//! walk from 09:30 to 16:00 UTC on 2026-08-03.

use malevich::{Frame, Line, Plot, Rule};
include!("support/svg_card.rs");

fn main() {
    let open = 1_785_749_400.0; // 2026-08-03 09:30 UTC
    let stamps: Vec<f64> = (0..=390).map(|m| open + f64::from(m) * 60.0).collect();
    let mut price = 184.20;
    let prices: Vec<f64> = (0..=390)
        .map(|m| {
            let tick = ((m * 7919) % 97) as f64 / 97.0 - 0.5;
            price += tick * 0.35 + (f64::from(m) / 390.0 - 0.4) * 0.02;
            price
        })
        .collect();

    let plot = Plot::new()
        .layer(Rule::h(prices[0]).label("open"))
        .layer(Line::xy(&stamps[..], &prices[..]).label("last"))
        .time_x()
        .title("one session (synthetic)")
        .x_label("time (UTC)")
        .y_label("$");
    let frame = Frame::plain(72, 16);
    if svg_card(&plot, &frame) {
        return;
    }
    println!("{}", plot.render_best(&frame));
}
