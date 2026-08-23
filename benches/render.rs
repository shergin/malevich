//! Regression fence for end-to-end rendering: preset construction, resolve, layout,
//! rasterization, and encoding of a 10k-point line into an 80×20 frame.

use criterion::{Criterion, criterion_group, criterion_main};
use malevich::{Frame, Plot, Points};
use std::hint::black_box;

fn line_render(c: &mut Criterion) {
    let values: Vec<f64> = (0..10_000)
        .map(|i| (i as f64 * 0.01).sin() * (i as f64).sqrt())
        .collect();
    let frame = Frame::plain(80, 20);
    c.bench_function("render/line_10k_80x20", |b| {
        b.iter(|| black_box(malevich::line(black_box(&values[..])).render(&frame)));
    });
}

fn scatter_render(c: &mut Criterion) {
    let n = 1_000_000;
    let x: Vec<f64> = (0..n).map(|i| (i as f64 * 0.417).sin()).collect();
    let y: Vec<f64> = (0..n).map(|i| (i as f64 * 0.731).cos()).collect();
    let frame = Frame::plain(80, 20);
    c.bench_function("render/scatter_1m_80x20", |b| {
        b.iter(|| {
            black_box(malevich::scatter(black_box(&x[..]), black_box(&y[..])).render(&frame))
        });
    });
}

fn categorical_render(c: &mut Criterion) {
    let values: Vec<f64> = (0..100_000)
        .map(|index| (index as f64 * 0.001).sin())
        .collect();
    let frame = Frame::plain(80, 20);
    let mut group = c.benchmark_group("render/color_by_100k");

    for distinct in [5, 100, 100_000] {
        let categories = (0..values.len()).map(|index| format!("g{}", index % distinct));
        let plot = Plot::new().layer(Points::y(&values[..]).color_by(categories));
        group.bench_function(format!("{distinct}_categories"), |b| {
            b.iter(|| black_box(plot.render(&frame)));
        });
    }
    group.finish();
}

fn ansi_encoding(c: &mut Criterion) {
    use malevich::{Charset, Color, ColorMode, render::Surface};
    let mut surface = Surface::new(200, 60, Charset::Braille);
    for i in 0..(200 * 2) {
        let color = if i % 3 == 0 { Color::Red } else { Color::Cyan };
        surface.line((i as f64, 0.0), ((400 - i) as f64, 239.0), color);
    }
    c.bench_function("render/encode_ansi_200x60", |b| {
        b.iter(|| black_box(surface.encode(black_box(ColorMode::Ansi16))));
    });
}

fn ten_million_points(c: &mut Criterion) {
    let n = 10_000_000;
    let y: Vec<f64> = (0..n)
        .map(|i| (i as f64 * 0.0002).sin() * (i as f64 * 0.000013).cos() * 8.0)
        .collect();
    let frame = Frame::plain(80, 20);
    // The headline fence: end to end, ingestion through ANSI-free encoding, with
    // the automatically inserted M4 doing the heavy lifting.
    c.bench_function("render/line_10m_80x20", |b| {
        b.iter(|| black_box(malevich::line(black_box(&y[..])).render(&frame)));
    });
    let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
    c.bench_function("stat/m4_10m_160cols", |b| {
        b.iter(|| {
            black_box(malevich::stat::m4(
                black_box(&x[..]),
                black_box(&y[..]),
                160,
            ))
        });
    });
}

fn histogram_binning(c: &mut Criterion) {
    let values: Vec<f64> = (0..1_000_000)
        .map(|i| {
            let i = i as f64;
            ((i * 0.731).sin() + (i * 1.13).sin()) * 4.0 + 20.0
        })
        .collect();
    c.bench_function("stat/bins_auto_1m", |b| {
        b.iter(|| black_box(malevich::stat::Bins::auto(black_box(&values[..]), 60)));
    });
}

fn least_squares_fit(c: &mut Criterion) {
    let x: Vec<f64> = (0..1_000_000).map(|i| i as f64 * 0.001).collect();
    let y: Vec<f64> = x
        .iter()
        .map(|v| 0.8 * v + 4.0 + (v * 7.31).sin() * 2.0)
        .collect();
    c.bench_function("stat/fit_1m", |b| {
        b.iter(|| black_box(malevich::stat::Fit::xy(black_box(&x), black_box(&y))));
    });
}

fn heatmap_render(c: &mut Criterion) {
    let grid: Vec<f64> = (0..64 * 48).map(|i| (i as f64 * 0.37).sin()).collect();
    let frame = Frame::plain(80, 24);
    c.bench_function("render/heatmap_64x48_80x24", |b| {
        b.iter(|| black_box(malevich::heatmap(64, black_box(&grid[..])).render(&frame)));
    });
}

fn pathological_layout(c: &mut Criterion) {
    // The chrome-shedding fence: every frame size from degenerate to normal.
    let values: Vec<f64> = (0..500).map(|i| (i as f64 * 0.1).sin()).collect();
    let plot = malevich::line(&values[..]).title("layout");
    c.bench_function("render/layout_sweep_0_to_40", |b| {
        b.iter(|| {
            for width in 0..40 {
                for height in 0..12 {
                    black_box(plot.render(&Frame::plain(width, height)));
                }
            }
        });
    });
}

fn plot_clone(c: &mut Criterion) {
    // The D5 gate: adopt cow_vec for the layer list only if cloning retained plots
    // is a measurable cost. Twelve owned layers, five thousand points each.
    let layers: Vec<Vec<f64>> = (0..12)
        .map(|l| {
            (0..5_000)
                .map(|i| ((i + l * 7) as f64 * 0.01).sin())
                .collect()
        })
        .collect();
    let mut plot = malevich::Plot::new();
    for layer in &layers {
        plot = plot.layer(malevich::Line::y(layer.clone()));
    }
    let plot = plot.into_owned();
    c.bench_function("plot/clone_12x5k_owned", |b| {
        b.iter(|| black_box(plot.clone()));
    });
}

fn streaming_frame(c: &mut Criterion) {
    // One live frame: snapshot a full ring, build the plot, render — the 60 fps
    // budget is 16 ms; this must be far under it.
    let ring = malevich::stream::Ring::new(512);
    for i in 0..512 {
        ring.push((i as f64 * 0.1).sin());
    }
    let frame = Frame::plain(100, 20);
    c.bench_function("stream/frame_512_100x20", |b| {
        b.iter(|| {
            let snapshot = ring.snapshot();
            black_box(malevich::line(&snapshot[..]).render(&frame))
        });
    });
}

fn kde_density(c: &mut Criterion) {
    let values: Vec<f64> = (0..1_000_000)
        .map(|i| {
            let i = i as f64;
            ((i * 0.731).sin() + (i * 1.13).sin()) * 4.0
        })
        .collect();
    c.bench_function("stat/kde_1m_512", |b| {
        b.iter(|| black_box(malevich::stat::kde(black_box(&values[..]), 512)));
    });
}

criterion_group!(
    benches,
    plot_clone,
    streaming_frame,
    kde_density,
    line_render,
    scatter_render,
    categorical_render,
    ansi_encoding,
    ten_million_points,
    histogram_binning,
    least_squares_fit,
    heatmap_render,
    pathological_layout
);
criterion_main!(benches);
