//! A colored tour of every mark and preset, rendered for *your* terminal.
//!
//! Uses `Frame::detect()`: charts size themselves to the terminal width, use color
//! when the terminal has any, and degrade to plain text when piped. With the
//! `pixel` feature in a terminal that speaks a pixel protocol, every chart
//! becomes a side-by-side comparison — cells on the left, the same plot as a
//! real image on the right. This example is deliberately not part of the
//! deterministic gallery — its output depends on where you run it, which is the
//! point. For the moving version of this tour, run `cargo run --example live`.

use malevich::scale::{Colormap, Palette};
use malevich::stat::{Bins, Reducer, binned, ewma};
use malevich::{
    Area, Cells, Color, Dash, Frame, Grid, Line, LineStyle, Plot, PointStyle, Points, Range, Rule,
    Scale, Text,
};

/// The tour's render: one chart per row, or a cells-versus-pixels comparison
/// when the terminal offers a pixel protocol.
trait Show {
    fn show(&self, frame: &Frame) -> String;
}

impl Show for Plot<'_> {
    #[cfg(feature = "pixel")]
    fn show(&self, frame: &Frame) -> String {
        use std::fmt::Write as _;
        match malevich::pixel::Graphics::detect() {
            Some(graphics) => {
                // Two panes on the same rows: print the cell pane, walk back to
                // its top row, and print the column-anchored pixel pane.
                let pane = Frame {
                    width: frame.width.saturating_sub(2) / 2,
                    ..*frame
                };
                let mut out = self.render(&pane);
                if pane.height > 1 {
                    let _ = write!(out, "\x1b[{}A", pane.height - 1);
                }
                out.push_str(&self.render_pixels_at(&pane, &graphics, pane.width + 2));
                out
            }
            None => self.render_best(frame),
        }
    }

    #[cfg(not(feature = "pixel"))]
    fn show(&self, frame: &Frame) -> String {
        self.render_best(frame)
    }
}

fn main() {
    let frame = Frame::detect();

    // Lines, legend, annotations: the training-loop story.
    let steps: Vec<f64> = (0..120).map(f64::from).collect();
    let train: Vec<f64> = steps
        .iter()
        .map(|s| 3.8 * (-0.035 * s).exp() + 0.32 + 0.05 * (s * 0.7).sin())
        .collect();
    let val: Vec<f64> = steps
        .iter()
        .map(|s| 4.0 * (-0.03 * s).exp() + 0.55 + 0.08 * (s * 0.35).cos())
        .collect();
    println!(
        "{}\n",
        Plot::new()
            .layer(Line::xy(&steps[..], &train[..]).label("train"))
            .layer(Line::xy(&steps[..], &val[..]).label("val"))
            .layer(Rule::h(0.5).label("target"))
            .layer(Text::at(60.0, 2.0, "< converging"))
            .title("loss with annotations (synthetic)")
            .x_label("step")
            .y_label("loss")
            .show(&frame)
    );

    // The effects corner: what the alpha canvas buys. Anti-aliased
    // strokes with a glow over a translucent wash, dashed and dotted
    // annotation strokes — pixel panes show the full treatment, cell
    // panes stay solid ink.
    let vermilion = Color::Rgb(227, 66, 52);
    let sky = Color::Rgb(108, 153, 212);
    let train_smooth = ewma(&train, 0.9);
    let val_smooth = ewma(&val, 0.9);
    println!(
        "{}\n",
        Plot::new()
            .layer(
                Area::xy(&steps[..], &train_smooth[..])
                    .color(vermilion)
                    .opacity(0.13)
            )
            .layer(
                Line::xy(&steps[..], &train_smooth[..])
                    .label("train")
                    .color(vermilion)
                    .glow()
            )
            .layer(
                Line::xy(&steps[..], &val_smooth[..])
                    .label("val")
                    .color(sky)
                    .dash(Dash::Dotted)
            )
            .layer(Rule::h(0.5).label("target").dash(Dash::Dashed))
            .title("glow over a wash, dashed annotations (synthetic)")
            .x_label("step")
            .y_label("loss")
            .show(&frame)
    );

    // A trajectory graded by progress: each point wears its position in
    // training through a colormap — cold start, hot finish.
    let turns: Vec<f64> = (0..500)
        .map(|i| i as f64 / 499.0 * 18.0)
        .collect();
    let spiral_x: Vec<f64> = turns
        .iter()
        .map(|&a| a.cos() * (0.15 + a * 0.05) + (a * 2.3).sin() * 0.04)
        .collect();
    let spiral_y: Vec<f64> = turns
        .iter()
        .map(|&a| a.sin() * (0.15 + a * 0.05) * 0.8 + (a * 1.9).cos() * 0.04)
        .collect();
    println!(
        "{}\n",
        Plot::new()
            .layer(
                Line::xy(&spiral_x[..], &spiral_y[..])
                    .grade(&turns[..], Colormap::VIRIDIS)
                    .label("optimizer path")
            )
            .title("a trajectory graded by step (synthetic)")
            .show(&frame)
    );

    // Overplotting as brightness: accumulated low-opacity markers turn a
    // fifteen-thousand-point cloud into a density field.
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    let mut unit = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 11) as f64 / (1u64 << 53) as f64
    };
    let mut cloud_x = Vec::new();
    let mut cloud_y = Vec::new();
    for index in 0..15_000 {
        // Two gaussian-ish blobs (sum of uniforms), one twice as heavy.
        let center = if index % 3 == 0 { (2.4, 1.2) } else { (1.0, 0.8) };
        let sample = |unit: &mut dyn FnMut() -> f64, spread: f64| {
            (0..6).map(|_| unit()).sum::<f64>() / 6.0 * spread - spread / 2.0
        };
        cloud_x.push(center.0 + sample(&mut unit, 1.6));
        cloud_y.push(center.1 + sample(&mut unit, 1.1));
    }
    println!(
        "{}\n",
        Plot::new()
            .layer(
                Points::xy(&cloud_x[..], &cloud_y[..])
                    .color(Color::Rgb(86, 178, 163))
                    .opacity(0.18)
                    .density()
            )
            .title("15,000 points as accumulated ink (synthetic)")
            .show(&frame)
    );

    // A smooth field under its own contours: bilinear cells read as a
    // continuous surface, marching squares traces the levels over it.
    let field_size = 24usize;
    let field: Vec<f64> = (0..field_size * field_size)
        .map(|i| {
            let (fx, fy) = (
                (i % field_size) as f64 / 4.0,
                (i / field_size) as f64 / 4.0,
            );
            (fx - 3.0).powi(2) * 0.4
                + (fy - 2.6).powi(2) * 0.7
                + ((fx * 1.7).sin() * (fy * 1.3).cos()) * 0.8
        })
        .collect();
    let mut landscape = Plot::new().layer(
        Cells::matrix(field_size, &field[..])
            .colormap(Colormap::MAGMA)
            .smooth(),
    );
    let levels: Vec<f64> = (1..7).map(|i| f64::from(i) * 1.6).collect();
    for line in malevich::stat::contours(&field, field_size, &levels) {
        landscape = landscape.layer(
            Line::xy(line.x.clone(), line.y.clone()).color(Color::Rgb(235, 235, 230)),
        );
    }
    println!(
        "{}\n",
        landscape
            .title("a smooth loss landscape with contours (synthetic)")
            .show(&frame)
    );

    // A calendar axis: unix seconds in, "Aug 2" out.
    let month_stamp = |year: i64, month: u64| -> f64 {
        let y = year - i64::from(month <= 2);
        let era = if y >= 0 { y } else { y - 399 } / 400;
        let yoe = (y - era * 400) as u64;
        let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        ((era * 146_097 + doe as i64 - 719_468) * 86_400) as f64
    };
    let stamps: Vec<f64> = (0..36)
        .map(|i| month_stamp(2024 + i / 12, (1 + i % 12) as u64))
        .collect();
    let level: Vec<f64> = (0..36)
        .map(|i| 400.0 + i as f64 * 0.2 + ((i % 12) as f64 * 0.52).sin() * 3.0)
        .collect();
    println!(
        "{}\n",
        Plot::new()
            .layer(Line::xy(&stamps[..], &level[..]))
            .title("a monthly series on a calendar axis (synthetic)")
            .time_x()
            .show(&frame)
    );

    // A rolling mean over its noisy source.
    let raw: Vec<f64> = (0..120)
        .map(|i| 3.0 * (-0.03 * i as f64).exp() + 0.4 + ((i * 7) % 13) as f64 * 0.06)
        .collect();
    let smooth = malevich::stat::Window::new(9).mean(&raw);
    println!(
        "{}\n",
        Plot::new()
            .layer(Line::y(&raw[..]).label("raw"))
            .layer(Line::y(&smooth[..]).label("rolling mean"))
            .title("smoothing (synthetic)")
            .show(&frame)
    );

    // Ten million points, downsampled pixel-exactly on the way in.
    let n = 10_000_000;
    let wave: Vec<f64> = (0..n)
        .map(|i| {
            let i = i as f64;
            (i * 0.0002).sin() * (i * 0.000013).cos() * 8.0
        })
        .collect();
    println!(
        "{}\n",
        malevich::line(&wave[..])
            .title("10,000,000 points through M4")
            .show(&frame)
    );

    // Bars and a histogram.
    println!(
        "{}\n",
        malevich::bar(
            ["rust", "go", "python", "typescript", "zig"],
            &[68.0, 41.0, 55.0, 62.0, 12.0][..],
        )
        .title("admired languages, % (synthetic)")
        .show(&frame)
    );
    let samples: Vec<f64> = (0..4000)
        .map(|i| {
            let i = i as f64;
            ((i * 0.731).sin() + (i * 1.13).sin() + (i * 2.71).sin()) * 2.0 + 10.0
        })
        .collect();
    println!(
        "{}\n",
        malevich::hist(&samples[..])
            .title("histogram, automatic bins")
            .show(&frame)
    );

    // Stacked areas.
    let x: Vec<f64> = (0..80).map(f64::from).collect();
    let solar: Vec<f64> = x.iter().map(|v| 3.0 + (v * 0.2).sin() + v * 0.02).collect();
    let wind: Vec<f64> = x
        .iter()
        .map(|v| 2.0 + (v * 0.13).cos().abs() * 1.5)
        .collect();
    let hydro: Vec<f64> = x.iter().map(|v| 1.0 + (v * 0.07).sin().abs()).collect();
    let bands = malevich::stat::stack(&[&solar, &wind, &hydro]);
    let mut stacked = Plot::new().title("energy mix, stacked (synthetic)");
    for ((low, high), label) in bands.iter().zip(["solar", "wind", "hydro"]) {
        stacked = stacked.layer(Area::between(&x[..], &low[..], &high[..]).label(label));
    }
    println!("{}\n", stacked.show(&frame));

    // A heatmap and a 2D histogram.
    let size = 8usize;
    let grid: Vec<f64> = (0..size * size)
        .map(|i| {
            let (row, column) = ((i / size) as f64, (i % size) as f64);
            if row == column {
                1.0
            } else {
                ((row - column).abs() * -0.35).exp() * ((row + column) * 0.55).cos()
            }
        })
        .collect();
    let correlation_options = malevich::HeatmapOptions::new()
        .colormap(malevich::scale::Colormap::RED_BLUE.centered_at(0.0));
    println!(
        "{}\n",
        malevich::heatmap_with(size, &grid[..], correlation_options)
            .expect("a named colormap is valid")
            .title("correlation matrix (synthetic)")
            .show(&frame)
    );
    let bell = |i: f64, seed: f64| -> f64 {
        ((i * 0.97 + seed).sin() + (i * 1.31 + seed * 2.0).sin() + (i * 2.63 + seed * 3.0).sin())
            / 3.0
    };
    let points = 6000;
    let cx: Vec<f64> = (0..points)
        .map(|i| {
            let i = i as f64;
            if i as i64 % 2 == 0 {
                3.0 + bell(i, 1.0) * 1.8
            } else {
                7.0 + bell(i, 4.0) * 1.2
            }
        })
        .collect();
    let cy: Vec<f64> = (0..points)
        .map(|i| {
            let i = i as f64;
            if i as i64 % 2 == 0 {
                3.0 + bell(i, 7.0) * 1.4
            } else {
                6.5 + bell(i, 9.0) * 1.7
            }
        })
        .collect();
    println!(
        "{}\n",
        malevich::hist2d(&cx[..], &cy[..])
            .title("2d density (synthetic)")
            .show(&frame)
    );

    // Contour lines: marching squares over a saddle between two humps.
    let (columns, rows) = (40, 30);
    let mut z = Vec::with_capacity(columns * rows);
    for r in 0..rows {
        for c in 0..columns {
            let x = c as f64 / (columns - 1) as f64 * 4.0 - 2.0;
            let y = r as f64 / (rows - 1) as f64 * 4.0 - 2.0;
            z.push(
                (-(x - 0.8).powi(2) - (y - 0.6).powi(2)).exp()
                    - 0.8 * (-(x + 0.8).powi(2) - (y + 0.6).powi(2)).exp(),
            );
        }
    }
    println!(
        "{}\n",
        malevich::contour(columns, &z[..])
            .title("contour lines (synthetic)")
            .show(&frame)
    );

    // A vector field: circular flow, one arrow per grid point.
    let mut fx = Vec::new();
    let mut fy = Vec::new();
    let mut fu = Vec::new();
    let mut fv = Vec::new();
    for row in 0..8 {
        for column in 0..11 {
            let px = -2.0 + 0.4 * column as f64;
            let py = -1.4 + 0.4 * row as f64;
            fx.push(px);
            fy.push(py);
            fu.push(-0.3 * py);
            fv.push(0.3 * px);
        }
    }
    println!(
        "{}\n",
        malevich::quiver(&fx[..], &fy[..], &fu[..], &fv[..])
            .title("vector field (synthetic)")
            .show(&frame)
    );

    // The asciichart-style corners line.
    let wave: Vec<f64> = (0..60)
        .map(|i| 15.0 * (i as f64 * std::f64::consts::PI / 30.0).sin())
        .collect();
    println!(
        "{}\n",
        Plot::new()
            .layer(Line::y(&wave[..]).style(LineStyle::Corners))
            .title("the corners style")
            .show(&frame)
    );

    // Small multiples.
    let alpha: Vec<f64> = (0..50).map(|i| (i as f64 * 0.2).sin() * 3.0).collect();
    let beta: Vec<f64> = (0..50).map(|i| (i as f64 * 0.13).cos() * 5.0).collect();
    println!(
        "{}\n",
        Grid::new(2)
            .with(
                malevich::line(&alpha[..])
                    .title("alpha")
                    .y_domain(-6.0, 6.0)
            )
            .with(malevich::line(&beta[..]).title("beta").y_domain(-6.0, 6.0))
            .render(&frame)
    );

    // Log-log axes, an ECDF, and a labeled scatter to close.
    println!(
        "{}\n",
        Plot::new()
            .layer(Line::function(1.0..100_000.0, |x| 0.5 * x.powf(1.5)).label("0.5 x^1.5"))
            .layer(Line::function(1.0..100_000.0, |x| 20.0 * x.sqrt()).label("20 sqrt x"))
            .title("power laws, log-log")
            .log_x()
            .log_y()
            .show(&frame)
    );
    println!(
        "{}\n",
        malevich::ecdf_with(&samples[..], malevich::EcdfOptions::new().band(0.05))
            .expect("a valid band level")
            .title("ecdf of the histogram sample, 95% DKW band")
            .show(&frame)
    );
    // A deterministic unit hash for the synthetic panels below.
    let noise = |i: usize, seed: f64| {
        let hash = (i as f64 * 12.9898 + seed * 78.233).sin() * 43758.5453;
        (hash - hash.floor()) * 2.0 - 1.0
    };
    // Least squares as a stat: the fitted line, a 95% confidence band around
    // the mean response, and R² from the same mergeable accumulator.
    let dose: Vec<f64> = (0..70).map(|i| i as f64 * 0.4).collect();
    let response: Vec<f64> = dose
        .iter()
        .enumerate()
        .map(|(i, &d)| 0.8 * d + 4.0 + noise(i, 9.0) * 2.4)
        .collect();
    let fit = malevich::stat::Fit::xy(&dose, &response);
    println!(
        "{}\n",
        malevich::trend_with(
            &dose[..],
            &response[..],
            malevich::TrendOptions::new().band(1.96),
        )
        .expect("a positive band multiplier is valid")
        .title(format!(
            "least squares: R\u{b2} = {:.2} (synthetic)",
            fit.r_squared().unwrap_or(f64::NAN)
        ))
        .show(&frame)
    );
    // A Q–Q plot from the grammar: matched type-7 quantiles of two samples
    // against the identity line — the heavy tail peels off it.
    let normalish: Vec<f64> = (0..400)
        .map(|i| (0..6).map(|k| noise(i * 6 + k, 1.0)).sum::<f64>())
        .collect();
    let heavy: Vec<f64> = (0..400)
        .map(|i| {
            let base = (0..6).map(|k| noise(i * 6 + k, 9.0)).sum::<f64>();
            if noise(i, 17.0) > 0.6 {
                base * 2.5
            } else {
                base
            }
        })
        .collect();
    let positions: Vec<f64> = (1..100).map(|p| p as f64 / 100.0).collect();
    let qx = malevich::stat::quantiles(&normalish, &positions);
    let qy = malevich::stat::quantiles(&heavy, &positions);
    println!(
        "{}\n",
        Plot::new()
            .layer(Line::xy(vec![-4.0, 6.0], vec![-4.0, 6.0]).label("identity"))
            .layer(Points::xy(qx, qy).label("quantiles"))
            .title("Q\u{2013}Q: heavy-tailed vs normal-ish (synthetic)")
            .show(&frame)
    );
    let blob = |count: usize, cx: f64, cy: f64, spread: f64| -> (Vec<f64>, Vec<f64>) {
        (0..count)
            .map(|i| {
                let i = i as f64;
                (
                    cx + spread * (i * 0.97).sin() * (i * 0.31).cos(),
                    cy + spread * 0.6 * (i * 1.13).cos() * (i * 0.47).sin(),
                )
            })
            .unzip()
    };
    // Two colonies through one color_by channel: palette colors, a categorical
    // legend, and marker shapes keeping the groups apart when piped.
    let (ax, ay) = blob(80, 3.0, 4.0, 1.6);
    let (bx, by) = blob(80, 7.5, 7.0, 1.9);
    let mut colony = vec!["colony a"; ax.len()];
    colony.extend(std::iter::repeat_n("colony b", bx.len()));
    let x: Vec<f64> = ax.into_iter().chain(bx).collect();
    let y: Vec<f64> = ay.into_iter().chain(by).collect();
    println!(
        "{}\n",
        Plot::new()
            .layer(Points::xy(&x[..], &y[..]).color_by(colony))
            .title("two colonies, one color_by channel (synthetic)")
            .show(&frame)
    );
    // A volcano plot from the grammar: significance classes through color_by,
    // thresholds as Rules, grey pinned to the insignificant mass.
    let class = |fc: f64, p: f64| {
        if p < 2.0 || fc.abs() < 1.0 {
            "n.s."
        } else if fc > 0.0 {
            "up"
        } else {
            "down"
        }
    };
    let genes: Vec<(f64, f64)> = (0..900)
        .map(|i| {
            let spread = if i % 7 == 0 { 2.6 } else { 0.7 };
            let log2fc = noise(i, 1.0) * spread;
            let lifted = (log2fc.abs() * 1.6 - 0.4 + noise(i, 7.0) * 1.2).max(0.02);
            (log2fc, lifted)
        })
        .collect();
    // Partitioned so "n.s." appears first: category order is first appearance,
    // and the palette below pins grey to it.
    let (mut vx, mut vy, mut classes) = (Vec::new(), Vec::new(), Vec::new());
    for wanted in ["n.s.", "down", "up"] {
        for &(fc, p) in &genes {
            if class(fc, p) == wanted {
                vx.push(fc);
                vy.push(p);
                classes.push(wanted);
            }
        }
    }
    println!(
        "{}\n",
        Plot::new()
            .layer(Points::xy(&vx[..], &vy[..]).color_by(classes))
            .palette(Palette::new(&[
                Color::BrightBlack,
                Color::Rgb(0, 114, 178),
                Color::Rgb(213, 94, 0),
            ]))
            .layer(Rule::v(-1.0))
            .layer(Rule::v(1.0))
            .layer(Rule::h(2.0))
            .title("volcano: differential expression (synthetic)")
            .show(&frame)
    );
    // A Manhattan plot from the grammar: chromosomes alternate two shades as
    // unlabeled layers, the genome-wide threshold is a labeled Rule.
    let sizes = [180, 160, 150, 130, 120, 110, 95, 85, 75, 70, 60, 55];
    let hits = [2usize, 6, 9];
    let (mut even_x, mut even_y) = (Vec::new(), Vec::new());
    let (mut odd_x, mut odd_y) = (Vec::new(), Vec::new());
    let mut position = 0usize;
    for (chromosome, &size) in sizes.iter().enumerate() {
        for i in 0..size {
            let mut p = noise(position + i, 1.0).abs() * 2.8;
            if hits.contains(&chromosome) {
                let center = (i as f64 - size as f64 / 2.0).abs() / size as f64;
                let lift = (0.5 - center).max(0.0) * 2.0;
                p += lift * (6.5 + noise(position + i, 5.0) * 1.5) * lift;
            }
            let at = (position + i) as f64;
            if chromosome % 2 == 0 {
                even_x.push(at);
                even_y.push(p);
            } else {
                odd_x.push(at);
                odd_y.push(p);
            }
        }
        position += size;
    }
    println!(
        "{}\n",
        Plot::new()
            .layer(Points::xy(&even_x[..], &even_y[..]).color(Color::Rgb(0, 114, 178)))
            .layer(Points::xy(&odd_x[..], &odd_y[..]).color(Color::Rgb(86, 180, 233)))
            .layer(Rule::h(5.0).label("genome-wide"))
            .title("manhattan: association scan (synthetic)")
            .show(&frame)
    );
    // Candlesticks from the grammar: Range whiskers and bodies, up/down days
    // split by the same categorical channel.
    let mut price = 100.0f64;
    let days = 42usize;
    let (mut t, mut low, mut high, mut open, mut close, mut day) = (
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    );
    for i in 0..days {
        let drift = if i == 0 {
            0.8
        } else {
            noise(i, 3.0) * 2.2 + 0.1
        };
        let (opened, closed) = (price, price + drift);
        let wick = 0.4 + noise(i, 11.0).abs() * 1.4;
        t.push(i as f64);
        open.push(opened);
        close.push(closed);
        low.push(opened.min(closed) - wick);
        high.push(opened.max(closed) + wick);
        day.push(if closed >= opened { "up" } else { "down" });
        price = closed;
    }
    println!(
        "{}\n",
        Plot::new()
            .layer(
                Range::xy(&t[..], &low[..], &high[..])
                    .body(&open[..], &close[..])
                    .color_by(day),
            )
            .palette(Palette::new(&[
                Color::Rgb(0, 158, 115),
                Color::Rgb(213, 94, 0),
            ]))
            .title("daily candles (synthetic)")
            .show(&frame)
    );

    // ── The ML corner: the charts training loops actually need. ──

    // A confusion matrix: a Cells grid on Bands axes reading in matrix order,
    // per-cell counts as Text.
    let labels = ["cat", "dog", "bird"];
    let counts = [38.0, 2.0, 0.0, 3.0, 33.0, 4.0, 1.0, 5.0, 34.0];
    let mut confusion = Plot::new()
        .layer(Cells::matrix(3, &counts[..]).colormap(Colormap::GREYS))
        .x_scale(Scale::bands(labels))
        .y_scale(Scale::bands(labels))
        .x_label("predicted")
        .y_label("true")
        .title("confusion matrix (synthetic)");
    for (i, &count) in counts.iter().enumerate() {
        confusion = confusion.layer(Text::at(
            (i % 3) as f64,
            (i / 3) as f64,
            format!("{count:.0}"),
        ));
    }
    println!("{}\n", confusion.show(&frame));

    // An attention head: token bands on both axes, a logarithmic colormap so
    // decades stay apart, and the causal mask's zeros as honest gaps.
    let tokens = ["The", "robot", "ate", "the", "red", "apple", "."];
    let width = tokens.len();
    let mut weights = vec![0.0f64; width * width];
    for query in 0..width {
        for key in 0..=query {
            weights[query * width + key] = (-1.9 * (query - key) as f64).exp();
        }
    }
    weights[6 * width + 1] = 0.35;
    for query in 0..width {
        let row = &mut weights[query * width..(query + 1) * width];
        let sum: f64 = row.iter().sum();
        for weight in row {
            *weight /= sum;
        }
    }
    println!(
        "{}\n",
        Plot::new()
            .layer(Cells::matrix(width, &weights[..]).colormap(Colormap::MAGMA.log()))
            .x_scale(Scale::bands(tokens))
            .y_scale(Scale::bands(tokens))
            .x_label("key")
            .y_label("query")
            .colorbar()
            .title("attention, log colormap (synthetic)")
            .show(&frame)
    );

    // A first-layer filter as an image: direct rgb cells, no colormap — luma
    // shades in cells, true color in the pixel pane.
    let side = 24usize;
    let filter: Vec<(u8, u8, u8)> = (0..side * side)
        .map(|i| {
            let x = (i % side) as f64 / (side - 1) as f64 * 2.0 - 1.0;
            let y = (i / side) as f64 / (side - 1) as f64 * 2.0 - 1.0;
            let along = x * 0.5f64.cos() + y * 0.5f64.sin();
            let envelope = (-(x * x + y * y) / 0.5).exp();
            let level = |phase: f64| {
                let wave = (std::f64::consts::TAU * along / 0.5 + phase).cos();
                ((0.5 + 0.5 * wave * envelope) * 255.0).round() as u8
            };
            (level(0.0), level(2.1), level(4.2))
        })
        .collect();
    println!(
        "{}\n",
        Plot::new()
            .layer(Cells::rgb(side, filter))
            .title("a learned filter as rgb cells (synthetic)")
            .show(&frame)
    );

    // Decision regions: class cells through the categorical palette with a
    // legend, the training scatter on top.
    let centers = [(-1.5, -0.9, "a"), (1.6, -0.5, "b"), (0.1, 1.5, "c")];
    let (mut tx, mut ty, mut tclass) = (Vec::new(), Vec::new(), Vec::new());
    for (blob_index, &(cx, cy, name)) in centers.iter().enumerate() {
        for i in 0..12 {
            tx.push(cx + noise(i, blob_index as f64 * 3.0 + 20.0) * 0.8);
            ty.push(cy + noise(i, blob_index as f64 * 3.0 + 23.0) * 0.8);
            tclass.push(name);
        }
    }
    let resolution = 72usize;
    let regions: Vec<&str> = (0..resolution * resolution)
        .map(|i| {
            let px = -3.0 + 6.0 * ((i % resolution) as f64 + 0.5) / resolution as f64;
            let py = -3.0 + 6.0 * ((i / resolution) as f64 + 0.5) / resolution as f64;
            tx.iter()
                .zip(&ty)
                .zip(&tclass)
                .map(|((&sx, &sy), &name)| ((sx - px).powi(2) + (sy - py).powi(2), name))
                .min_by(|a, b| a.0.total_cmp(&b.0))
                .map(|(_, name)| name)
                .unwrap_or("a")
        })
        .collect();
    println!(
        "{}\n",
        Plot::new()
            .layer(Cells::classes(resolution, regions).extents((-3.0, 3.0), (-3.0, 3.0)))
            .layer(Points::xy(&tx[..], &ty[..]).style(PointStyle::Cross))
            .title("1-NN decision regions (synthetic)")
            .show(&frame)
    );

    // A loss landscape with a momentum trajectory. The 220×220 grid is denser
    // than the raster, so every screen bucket is an honest mean, not a sample.
    let himmelblau = |x: f64, y: f64| (x * x + y - 11.0).powi(2) + (x + y * y - 7.0).powi(2);
    let resolution = 220usize;
    let surface: Vec<f64> = (0..resolution * resolution)
        .map(|i| {
            let x = -5.0 + 10.0 * ((i % resolution) as f64 + 0.5) / resolution as f64;
            let y = -5.0 + 10.0 * ((i / resolution) as f64 + 0.5) / resolution as f64;
            himmelblau(x, y)
        })
        .collect();
    let (mut px, mut py, mut vx, mut vy) = (-0.27f64, -4.6f64, 0.0f64, 0.0f64);
    let (mut path_x, mut path_y) = (vec![px], vec![py]);
    for _ in 0..48 {
        let gx = 4.0 * px * (px * px + py - 11.0) + 2.0 * (px + py * py - 7.0);
        let gy = 2.0 * (px * px + py - 11.0) + 4.0 * py * (px + py * py - 7.0);
        vx = 0.82 * vx - 8.0e-4 * gx;
        vy = 0.82 * vy - 8.0e-4 * gy;
        px += vx;
        py += vy;
        path_x.push(px);
        path_y.push(py);
    }
    println!(
        "{}\n",
        Plot::new()
            .layer(
                Cells::matrix(resolution, &surface[..])
                    .extents((-5.0, 5.0), (-5.0, 5.0))
                    .colormap(Colormap::VIRIDIS.log()),
            )
            .layer(Line::xy(&path_x[..], &path_y[..]))
            .layer(Points::xy(&path_x[..], &path_y[..]).style(PointStyle::Circle))
            .title("momentum on a loss landscape")
            .show(&frame)
    );

    // Training curves across seeds: pooled per-step quantiles as a band, the
    // median inside it, and its EWMA smoothing on top, on a log axis.
    let (mut run_steps, mut run_losses) = (Vec::new(), Vec::new());
    for seed in 0..5 {
        for step in 0..400 {
            let decay = 2.4 * (-(step as f64) / 90.0).exp();
            let floor = 0.30 + 0.05 * seed as f64;
            let wobble = 1.0 + 0.5 * noise(step, seed as f64 + 40.0);
            run_steps.push(step as f64);
            run_losses.push((floor + decay) * wobble);
        }
    }
    let bins = Bins::new(0.0, 8.0, 50);
    let p10 = binned(&run_steps, &run_losses, &bins, Reducer::Percentile(0.1));
    let p50 = binned(&run_steps, &run_losses, &bins, Reducer::Median);
    let p90 = binned(&run_steps, &run_losses, &bins, Reducer::Percentile(0.9));
    let smoothed = ewma(&p50, 0.8);
    let centers: Vec<f64> = (0..50).map(|bin| 4.0 + 8.0 * bin as f64).collect();
    println!(
        "{}\n",
        Plot::new()
            .layer(Area::between(&centers[..], &p10[..], &p90[..]).label("p10-p90"))
            .layer(Line::xy(&centers[..], &p50[..]).label("median"))
            .layer(
                Line::xy(&centers[..], &smoothed[..])
                    .style(LineStyle::Corners)
                    .label("ewma"),
            )
            .log_y()
            .x_label("step")
            .title("loss across 5 seeds")
            .show(&frame)
    );

    // A spectrogram to close: seconds × hertz through extents, a log frequency
    // axis, and a log colormap — the chirp reads as a straight ridge.
    let (columns, rows) = (240usize, 160usize);
    let power: Vec<f64> = (0..columns * rows)
        .map(|i| {
            let t = 4.0 * ((i % columns) as f64 + 0.5) / columns as f64;
            let f = 60.0 + 7940.0 * ((i / columns) as f64 + 0.5) / rows as f64;
            let chirp_f = 100.0 * (4000.0f64 / 100.0).powf(t / 4.0);
            let chirp = (-((f.ln() - chirp_f.ln()) / 0.09).powi(2)).exp();
            let tone = 0.5 * (-((f.ln() - 440.0f64.ln()) / 0.05).powi(2)).exp();
            let click = 0.8 * (-((t - 2.6) / 0.015).powi(2)).exp();
            1e-4 + chirp + tone + click
        })
        .collect();
    println!(
        "{}",
        Plot::new()
            .layer(
                Cells::matrix(columns, &power[..])
                    .extents((0.0, 4.0), (60.0, 8000.0))
                    .colormap(Colormap::MAGMA.log()),
            )
            .log_y()
            .x_label("s")
            .y_label("Hz")
            .colorbar()
            .title("spectrogram (synthetic)")
            .show(&frame)
    );
}
