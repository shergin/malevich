//! Train a small MLP with topos and watch it learn through malevich: the two
//! interleaved moons, the loss curve with its EWMA smoothing on a log axis,
//! and the learned decision regions as categorical cells with the training
//! points on top — the scikit-learn playground panel, produced by an actual
//! network rather than a synthetic model.
//!
//! topos records the network once as a tape; training is a pure data
//! transform of caller-owned parameters, and a second grid-shaped expression
//! of the same parameters rasterizes the decision function in one forward
//! pass. Run with: `cargo run -p learn --release`

use std::f32::consts::PI;

use malevich::stat::ewma;
use malevich::{Cells, Frame, Line, LineStyle, Plot, PointStyle, Points};
use topos::{Activation, Mlp, Module, Shape, Tape, Tensor, init};

/// How many points each half-moon holds.
const MOON_LEN: usize = 120;

/// The resolution of the decision-region grid.
const GRID_COLUMNS: usize = 96;
const GRID_ROWS: usize = 64;

/// The data window both charts share: the moons with a margin.
const X_SPAN: (f32, f32) = (-1.5, 2.5);
const Y_SPAN: (f32, f32) = (-1.0, 1.5);

/// How many full-batch gradient descent steps the training takes.
const STEP_COUNT: usize = 9000;

/// One half-moon of noisy points: the upper arch, or the interleaved lower.
fn moon(flipped: bool, noise: &[f32]) -> (Vec<f32>, Vec<f32>) {
    let mut moon_x = Vec::with_capacity(MOON_LEN);
    let mut moon_y = Vec::with_capacity(MOON_LEN);
    for index in 0..MOON_LEN {
        let angle = PI * index as f32 / (MOON_LEN - 1) as f32;
        let (x, y) = if flipped {
            (1.0 - angle.cos(), 0.5 - angle.sin())
        } else {
            (angle.cos(), angle.sin())
        };
        moon_x.push(x + noise[index * 2]);
        moon_y.push(y + noise[index * 2 + 1]);
    }
    (moon_x, moon_y)
}

fn main() {
    let frame = Frame::detect();

    let noise: Tensor<f32> = init::normal(5, 0.1)(&Shape::new([2 * MOON_LEN, 2]));
    let noise = noise.to_vec();
    let (upper_x, upper_y) = moon(false, &noise[..2 * MOON_LEN]);
    let (lower_x, lower_y) = moon(true, &noise[2 * MOON_LEN..]);

    // The training batch: interleaved coordinates, upper moon +1, lower -1.
    let mut features = Vec::with_capacity(4 * MOON_LEN);
    for (x, y) in upper_x
        .iter()
        .zip(&upper_y)
        .chain(lower_x.iter().zip(&lower_y))
    {
        features.push(*x);
        features.push(*y);
    }
    let mut targets = vec![1.0_f32; MOON_LEN];
    targets.extend(vec![-1.0; MOON_LEN]);

    let tape: Tape<f32> = Tape::new();
    let mlp = Mlp::new(&tape, &[2, 16, 16, 1], Activation::Tanh, init::xavier(7));
    let input = tape.input(Tensor::new([2 * MOON_LEN, 2], features));
    let expected = tape.input(Tensor::new([2 * MOON_LEN, 1], targets));
    let error = mlp.express(input) - expected;
    let loss = (error * error).sum();

    // The rasterizing twin: the same parameters over the grid's cell centers.
    let mut centers = Vec::with_capacity(2 * GRID_COLUMNS * GRID_ROWS);
    for row in 0..GRID_ROWS {
        for column in 0..GRID_COLUMNS {
            let fx = (column as f32 + 0.5) / GRID_COLUMNS as f32;
            let fy = (row as f32 + 0.5) / GRID_ROWS as f32;
            centers.push(X_SPAN.0 + fx * (X_SPAN.1 - X_SPAN.0));
            centers.push(Y_SPAN.0 + fy * (Y_SPAN.1 - Y_SPAN.0));
        }
    }
    let grid_input = tape.input(Tensor::new([GRID_COLUMNS * GRID_ROWS, 2], centers));
    let surface = mlp.express(grid_input);

    let (loss, surface) = (loss.symbol(), surface.symbol());
    let network = tape.into_network();
    let mut parameters = network.parameters();

    let learning_rate = Tensor::new([], [0.0004]);
    let mut losses = Vec::with_capacity(STEP_COUNT);
    for _ in 0..STEP_COUNT {
        let run = network.forward(&parameters, []);
        losses.push(f64::from(run.of(loss).scalar()));
        let gradients = run.backward(loss).parameters(&parameters);
        parameters = parameters.step(&gradients, |parameter, gradient| {
            parameter.clone() - gradient.clone() * learning_rate.broadcast_like(gradient)
        });
    }

    // The loss curve and its smoothing, on a log axis: the training-diary
    // chart, with the raw curve as the ghost behind the EWMA.
    let smooth = ewma(&losses, 0.95);
    println!(
        "{}",
        Plot::new()
            .layer(Line::y(&losses[..]).label("loss"))
            .layer(
                Line::y(&smooth[..])
                    .style(LineStyle::Corners)
                    .label("ewma 0.95")
                    .glow(),
            )
            .log_y()
            .x_label("step")
            .title("two moons, sum of squared errors")
            .render_best(&frame)
    );

    // The learned decision regions as categorical cells — sign of the
    // network's output per grid cell — with the training points on top.
    let regions: Vec<&str> = network
        .forward(&parameters, [])
        .of(surface)
        .to_vec()
        .iter()
        .map(|&score| {
            if score > 0.0 {
                "upper moon"
            } else {
                "lower moon"
            }
        })
        .collect();
    println!(
        "{}",
        Plot::new()
            .layer(Cells::classes(GRID_COLUMNS, regions).extents(
                (f64::from(X_SPAN.0), f64::from(X_SPAN.1)),
                (f64::from(Y_SPAN.0), f64::from(Y_SPAN.1)),
            ))
            .layer(Points::xy(&upper_x[..], &upper_y[..]).style(PointStyle::Cross))
            .layer(Points::xy(&lower_x[..], &lower_y[..]).style(PointStyle::Circle))
            .title("the learned decision regions")
            .render_best(&frame)
    );
}
