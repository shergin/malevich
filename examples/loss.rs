//! A real training log: per-step minibatch loss of topos's makemore bigram
//! model (see examples/data/README.md), its rolling mean, and the corpus's known
//! bigram limit as a target rule. No synthetic data — this training actually ran.

use malevich::{Dash, Frame, Line, Plot, Rule};

fn main() {
    let (steps, losses): (Vec<f64>, Vec<f64>) = include_str!("data/topos_loss.csv")
        .lines()
        .filter_map(|line| {
            let (step, loss) = line.split_once(',')?;
            let step: f64 = step.parse().ok()?;
            let loss: f64 = loss.parse().ok()?;
            Some((step, loss))
        })
        .unzip();
    let smoothed = malevich::stat::Window::new(25).mean(&losses);

    let plot = Plot::new()
        .layer(Line::xy(&steps[..], &losses[..]).label("minibatch"))
        .layer(
            Line::xy(&steps[..], &smoothed[..])
                .label("rolling mean")
                .glow(),
        )
        .layer(Rule::h(2.45).label("bigram limit").dash(Dash::Dashed))
        .title("topos: bigram training on 32k names")
        .x_label("step")
        .y_label("loss");
    println!("{}", plot.render_best(&Frame::plain(76, 19)));
}
