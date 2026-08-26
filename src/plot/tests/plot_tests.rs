use super::Plot;
use crate::mark::Line;
use crate::plot::Frame;

const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<Plot<'static>>();

#[test]
fn the_line_preset_equals_its_grammar_expansion() {
    let values = [1.0, 5.0, 2.0, 8.0];
    let frame = Frame::plain(40, 10);
    let preset = crate::line(&values[..]).render(&frame);
    let grammar = Plot::new().layer(Line::y(&values[..])).render(&frame);
    assert_eq!(preset, grammar);
}

#[test]
fn fallible_render_rejects_hostile_frame_geometry() {
    let plot = crate::line(&[1.0, 2.0, 3.0][..]);
    let enormous_width = Frame::plain(usize::MAX, 0);
    assert!(matches!(
        plot.try_render(&enormous_width),
        Err(crate::Error::DimensionTooLarge { .. })
    ));
    assert_eq!(plot.render(&enormous_width), "");

    let enormous_height = Frame::plain(0, usize::MAX);
    assert!(matches!(
        plot.try_render(&enormous_height),
        Err(crate::Error::DimensionTooLarge { .. })
    ));
    assert_eq!(plot.render(&enormous_height), "");
}

#[test]
fn extreme_finite_time_domains_render_without_panicking() {
    let plot = Plot::new()
        .layer(Line::xy(&[-f64::MAX, f64::MAX][..], &[1.0, 2.0][..]))
        .time_x();
    assert!(plot.validate().is_ok());
    let _ = plot.render(&Frame::plain(40, 10));
}

#[test]
fn the_scatter_preset_equals_its_grammar_expansion() {
    let x = [1.0, 2.0, 3.0];
    let y = [2.0, 1.0, 3.0];
    let frame = Frame::plain(40, 10);
    let preset = crate::scatter(&x[..], &y[..]).render(&frame);
    let grammar = Plot::new()
        .layer(crate::mark::Points::xy(&x[..], &y[..]))
        .render(&frame);
    assert_eq!(preset, grammar);
}

const PARABOLA: &str = r"10 ┤⠁                          ⠈
   │
   │  ⠁                      ⠈
 5 ┤    ⠄                  ⠠
   │      ⢀              ⡀
   │        ⢀          ⡀
 0 ┤          ⠠ ⢀  ⡀ ⠄
   └┬───────────┬────────────┬──
    0           3            6";

#[test]
fn scatter_dots_stay_unconnected_in_the_snapshot() {
    let x: Vec<f64> = (0..14).map(|i| i as f64 * 0.5).collect();
    let y: Vec<f64> = x.iter().map(|v| (v - 3.25) * (v - 3.25)).collect();
    let text = crate::scatter(&x[..], &y[..]).render(&Frame::plain(32, 9));
    assert_eq!(text, PARABOLA);
}

#[test]
fn the_bar_preset_equals_its_grammar_expansion() {
    let frame = Frame::plain(40, 10);
    let preset = crate::bar(["a", "b"], &[1.0, 2.0][..]).render(&frame);
    let grammar = Plot::new()
        .layer(crate::mark::Bars::new(["a", "b"], &[1.0, 2.0][..]))
        .render(&frame);
    assert_eq!(preset, grammar);
}

#[test]
fn large_lines_downsample_pixel_exactly_against_the_raw_raster() {
    // The oracle is the *raw* raster — every point drawn, M4 disabled. Mapped-space
    // M4 buckets by the rendered column, so the reduction is bit-identical to it, not
    // merely close. Cover an index line and an xy line at several frame sizes.
    let index: Vec<f64> = (0..50_000)
        .map(|i| (i as f64 * 0.002).sin() * (i as f64 * 0.0003).cos() * 5.0)
        .collect();
    let xy_x: Vec<f64> = (0..200_000).map(|i| i as f64 * 0.3).collect();
    let xy_y: Vec<f64> = (0..200_000).map(|i| (i as f64 * 0.001).sin()).collect();
    for (width, height) in [(70, 15), (133, 24), (40, 10)] {
        let frame = Frame::plain(width, height);
        let index_plot = Plot::new().layer(Line::y(&index[..])).title("t");
        assert_eq!(
            index_plot.rasterize_with(&frame, true).to_plain(),
            index_plot.rasterize_with(&frame, false).to_plain(),
            "index line at {width}x{height} is not pixel-exact"
        );
        let xy_plot = Plot::new().layer(Line::xy(&xy_x[..], &xy_y[..]));
        assert_eq!(
            xy_plot.rasterize_with(&frame, true).to_plain(),
            xy_plot.rasterize_with(&frame, false).to_plain(),
            "xy line at {width}x{height} is not pixel-exact"
        );
    }
}

#[test]
fn extreme_domain_downsampling_uses_the_safe_map() {
    let n = 2_000;
    let x: Vec<f64> = (0..n)
        .map(|index| crate::numeric::lerp(-f64::MAX, f64::MAX, index as f64 / (n - 1) as f64))
        .collect();
    let y: Vec<f64> = (0..n).map(|index| (index as f64 * 0.03).sin()).collect();
    let frame = Frame::plain(40, 10);
    let plot = Plot::new().layer(Line::xy(&x[..], &y[..]));

    assert_eq!(
        plot.rasterize_with(&frame, true).to_plain(),
        plot.rasterize_with(&frame, false).to_plain(),
        "an overflowing domain span must keep the scaled mapping path"
    );
}

#[test]
fn a_gap_inside_a_raster_column_stays_a_break() {
    // Many points per column with a NaN between a jump from low to high: the raw
    // render breaks the line there, and the downsampled one must too (COR-03).
    let n = 20_000;
    let mut x = Vec::with_capacity(n);
    let mut y = Vec::with_capacity(n);
    for i in 0..n {
        x.push(i as f64);
        y.push(if i == n / 2 {
            f64::NAN
        } else if i < n / 2 {
            -4.0
        } else {
            4.0
        });
    }
    let frame = Frame::plain(40, 11);
    let plot = Plot::new().layer(Line::xy(&x[..], &y[..]));
    assert_eq!(
        plot.rasterize_with(&frame, true).to_plain(),
        plot.rasterize_with(&frame, false).to_plain(),
        "M4 must reproduce the raw raster's gap, not bridge it"
    );
}

#[test]
fn several_gaps_inside_one_raster_column_stay_disconnected() {
    // An isolated high point sits between two gaps in the same target column.
    // Collapsing either gap invents a visible vertical connection.
    let n = 20_000;
    let x: Vec<f64> = (0..n).map(|index| index as f64).collect();
    let mut y = vec![-4.0; n];
    y[n / 2] = f64::NAN;
    y[n / 2 + 1] = 4.0;
    y[n / 2 + 2] = f64::NAN;

    let frame = Frame::plain(40, 11);
    let plot = Plot::new().layer(Line::xy(&x[..], &y[..]));
    assert_eq!(
        plot.rasterize_with(&frame, true).to_plain(),
        plot.rasterize_with(&frame, false).to_plain(),
        "M4 must preserve every break inside a target column"
    );
}

#[test]
fn category_transitions_survive_line_downsampling() {
    // The category switch and numerical gap both land inside densely populated
    // raster columns. Identity is topology: neither may become a connecting line.
    let n = 20_000;
    let x: Vec<f64> = (0..n).map(|index| index as f64).collect();
    let mut y: Vec<f64> = (0..n).map(|index| (index as f64 * 0.003).sin()).collect();
    y[n / 2 + 3] = f64::NAN;
    let categories: Vec<&str> = (0..n)
        .map(|index| if index < n / 2 { "before" } else { "after" })
        .collect();
    let plot = Plot::new().layer(Line::xy(&x[..], &y[..]).color_by(categories));
    let mut frame = Frame::plain(50, 12);
    frame.color = crate::ColorMode::TrueColor;

    assert_eq!(
        plot.rasterize_with(&frame, true).encode(frame.color),
        plot.rasterize_with(&frame, false).encode(frame.color),
        "category-aware M4 must reproduce the raw colored path"
    );
}

#[test]
fn labeled_layers_grow_a_legend_row() {
    let plot = Plot::new()
        .layer(Line::y(&[1.0, 2.0][..]).label("first"))
        .layer(Line::y(&[2.0, 1.0][..]).label("second"));
    let text = plot.render(&Frame::plain(40, 10));
    assert!(
        text.contains("\u{2500}\u{2500} first  \u{2500}\u{2500} second"),
        "missing legend: {text}"
    );
    // Shed before anything else when the frame is short.
    let short = plot.render(&Frame::plain(40, 7));
    assert!(!short.contains("first"), "legend not shed: {short}");
}

const BARS_WITH_TREND: &str = r"           bars with a trend line
7.5 ┤                             ▃▃▃▃▃▃▃
    │                             ███████
    │           ▁▁▁▁▁▁▁        ⢀⡠⠔███████
5.0 ┤           ███████⠒⠢⠤⠤⠤⠤⠔⠊⠁  ███████
    │         ⣀⠔███████  ▇▇▇▇▇▇▇  ███████
2.5 ┤  ▆▆▆▆▆▆▆  ███████  ███████  ███████
    │  ███████  ███████  ███████  ███████
    │  ███████  ███████  ███████  ███████
0.0 ┤  ███████  ███████  ███████  ███████
    └───────────────────────────────────────
          q1       q2       q3       q4";

#[test]
fn bars_share_scales_with_a_line_overlay_in_the_snapshot() {
    let text = Plot::new()
        .layer(crate::mark::Bars::new(
            ["q1", "q2", "q3", "q4"],
            &[3.0, 5.0, 4.0, 7.0][..],
        ))
        .layer(Line::y(&[2.5, 4.8, 4.4, 6.5][..]))
        .title("bars with a trend line")
        .render(&Frame::plain(44, 12));
    assert_eq!(text, BARS_WITH_TREND);
}

const NEGATIVE_BARS: &str = r" 7.5 ┤            ▄▄▄▄
     │            ████            ▄▄▄▄
 5.0 ┤            ████ ▁▁▁▁       ████      ▅▅▅▅
     │            ████ ████       ████      ████
     │ ▅▅▅▅       ████ ████       ████      ████
 2.5 ┤ ████       ████ ████       ████ ▇▇▇▇ ████
     │ ████       ████ ████       ████ ████ ████
 0.0 ┤ ████  ████ ████ ████ ████  ████ ████ ████
     │       ████           ▔▔▔▔
-2.5 ┤       ▔▔▔▔
     └────────────────────────────────────────────
         a     b    c    d    e     f    g    h";

#[test]
fn negative_bars_hang_below_the_baseline_in_the_snapshot() {
    let text = crate::bar(
        ["a", "b", "c", "d", "e", "f", "g", "h"],
        &[3.0, -2.0, 7.0, 4.5, -1.2, 6.0, 2.2, 5.0][..],
    )
    .render(&Frame::plain(50, 12));
    assert_eq!(text, NEGATIVE_BARS);
}

#[test]
fn log_axes_straighten_exponentials_and_drop_nonpositives() {
    let steps: Vec<f64> = (0..40).map(f64::from).collect();
    let decay: Vec<f64> = steps.iter().map(|s| 100.0 * (-0.3 * s).exp()).collect();
    let text = Plot::new()
        .layer(Line::xy(&steps[..], &decay[..]))
        .log_y()
        .render(&Frame::plain(40, 10));
    assert!(
        text.contains("10\u{2077}") || text.contains("10\u{207B}"),
        "no decade labels: {text}"
    );

    let with_zeroes = Plot::new()
        .layer(Line::y(&[1.0, 0.0, -5.0, 100.0][..]))
        .log_y()
        .render(&Frame::plain(40, 10));
    assert!(!with_zeroes.is_empty());
}

#[test]
fn the_hist_preset_equals_its_grammar_expansion() {
    let samples: Vec<f64> = (0..500).map(|i| ((i * 37) % 100) as f64 / 10.0).collect();
    let frame = Frame::plain(50, 12);
    let preset = crate::hist(&samples[..]).render(&frame);
    let bins = crate::stat::Bins::auto(&samples, 60).unwrap();
    let counts: Vec<f64> = bins.counts().iter().map(|&c| c as f64).collect();
    let grammar = Plot::new()
        .layer(crate::mark::Bars::spans(
            bins.start(),
            bins.width(),
            &counts[..],
        ))
        .render(&frame);
    assert_eq!(preset, grammar);
}

#[test]
fn span_bars_sit_contiguously_on_a_numeric_axis() {
    let text = Plot::new()
        .layer(crate::mark::Bars::spans(0.0, 1.0, &[2.0, 5.0, 3.0][..]))
        .render(&Frame::plain(40, 10));
    // A numeric axis (ticks, not category labels) under contiguous bars.
    assert!(text.contains('\u{252C}'), "missing numeric ticks: {text}");
    assert!(text.contains('\u{2588}'), "missing bar fills: {text}");
}

#[test]
fn every_charset_renders_with_its_own_glyphs() {
    use crate::Charset;
    let plot = Plot::new()
        .layer(Line::y(&[1.0, 4.0, 2.0, 5.0][..]))
        .layer(crate::mark::Area::y(&[0.5, 2.0, 1.0, 2.5][..]))
        .title("t");
    for (charset, witness) in [
        (
            Charset::HalfBlocks,
            &['\u{2580}', '\u{2584}', '\u{2588}'][..],
        ),
        (
            Charset::Quadrants,
            &['\u{2596}', '\u{2599}', '\u{2588}', '\u{259F}', '\u{2584}'][..],
        ),
        (Charset::Braille, &['\u{28FF}', '\u{2801}', '\u{28C0}'][..]),
    ] {
        let mut frame = Frame::plain(24, 8);
        frame.charset = charset;
        let text = plot.render(&frame);
        assert_eq!(text, plot.render(&frame), "nondeterministic in {charset:?}");
        assert!(
            text.chars().any(|c| {
                let cp = c as u32;
                (0x2580..=0x28FF).contains(&cp)
            }),
            "{charset:?} drew no block/braille glyphs: {text}"
        );
        let _ = witness;
    }
    let mut ascii = Frame::plain(24, 8);
    ascii.charset = Charset::Ascii;
    let text = plot.render(&ascii);
    assert!(text.is_ascii(), "ASCII output leaked non-ASCII: {text}");
}

#[test]
fn axis_titles_render_on_both_axes() {
    let plot = Plot::new()
        .layer(Line::y(&[1.0, 2.0][..]))
        .x_label("step")
        .y_label("loss");
    let text = plot.render(&Frame::plain(40, 12));
    assert!(text.contains("step"), "missing x label: {text}");
    for letter in ["l", "o", "s"] {
        assert!(text.contains(letter), "missing y label letters: {text}");
    }
    // Both shed cleanly when there is no room.
    let _ = plot.render(&Frame::plain(10, 3));
}

#[test]
fn rendering_is_deterministic() {
    let plot = Plot::new().layer(Line::y(&[1.0, 5.0, 2.0, 8.0][..]));
    let frame = Frame::plain(40, 10);
    assert_eq!(plot.render(&frame), plot.render(&frame));
}

#[test]
fn no_frame_size_panics() {
    let plot = Plot::new()
        .layer(Line::y(&[1.0, f64::NAN, 2.0, 8.0][..]))
        .title("robustness");
    for width in 0..=42 {
        for height in 0..=8 {
            let _ = plot.render(&Frame::plain(width, height));
        }
    }
}

#[test]
fn empty_plots_render_bare_chrome() {
    let text = Plot::new().render(&Frame::plain(30, 8));
    assert!(text.contains('\u{2502}'), "missing y axis: {text}");
    assert!(text.contains('\u{2500}'), "missing x axis: {text}");
}

// Golden snapshots. Flush-left so the expected charts stay readable in this file;
// regenerate by rendering with the same frames and eyeballing the diff.

const SPIKY: &str = r"8 ┤                         ⡠⠊
  │                       ⡠⠊
  │       ⣀⠤⠒⠤⣀        ⢀⠔⠉
4 ┤    ⡠⠔⠊     ⠉⠒⠤⣀  ⢀⠔⠁
  │⢀⡠⠔⠉            ⠉⠒⠁
0 ┤⠁
  └┬────────┬───────┬────────┬
   0        1       2        3";

#[test]
fn a_small_line_chart_matches_its_snapshot() {
    let text = crate::line(&[1.0, 5.0, 2.0, 8.0][..]).render(&Frame::plain(30, 8));
    assert_eq!(text, SPIKY);
}

const SIMPLE_BARS: &str = r"            bars
5 ┤         ██████
  │         ██████  ▁▁▁▁▁▁
  │  ▁▁▁▁▁▁ ██████  ██████
  │  ██████ ██████  ██████
0 ┤  ██████ ██████  ██████
  └─────────────────────────
        a      b       c";

#[test]
fn a_small_bar_chart_matches_its_snapshot() {
    let text = crate::bar(["a", "b", "c"], &[2.0, 5.0, 3.0][..])
        .title("bars")
        .render(&Frame::plain(28, 8));
    assert_eq!(text, SIMPLE_BARS);
}

const GAPPY: &str = r"5 ┤                      ⢀⠤⠊
  │       ⢀⡠⠒⠁         ⡠⠔⠁
  │    ⢀⡠⠔⠁           ⠈
  │⣀⠤⠒⠊⠁
0 ┤
  └┬───────────┬───────────┬
  0.0         2.5        5.0";

#[test]
fn a_gap_breaks_the_line_in_the_snapshot() {
    let gappy = [1.0, 2.0, 4.0, f64::NAN, 3.0, 5.0];
    let text = crate::line(&gappy[..]).render(&Frame::plain(28, 7));
    assert_eq!(text, GAPPY);
}

const SINE: &str = r"           sin
1 ┤      ⢀⠤⠔⠚⠉⠉⠉⠓⠢⠤⡀
  │    ⡠⠜⠁         ⠈⠣⢄
  │  ⡤⠊               ⠑⢤
0 ┤⡠⠊                   ⠑⢄
  └┬────────────────────┬─
   0                    3";

#[test]
fn a_sampled_function_matches_its_snapshot() {
    let plot = Plot::new()
        .layer(Line::function(0.0..std::f64::consts::PI, f64::sin))
        .title("sin");
    assert_eq!(plot.render(&Frame::plain(26, 7)), SINE);
}

const SMALL_HEATMAP: &str = r"8 ┤█████████████████  █ 30
  │█████████████████  █
  │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓  ▓ 20
4 ┤▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  ▒ 10
  │▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  ░
0 ┤░░░░░░░░░░░░░░░░░  ░ 0
  └┬────────────────┬
   0                4";

#[test]
fn a_small_heatmap_matches_its_plain_snapshot() {
    let values: Vec<f64> = (0..32).map(f64::from).collect();
    let text = crate::heatmap(4, values).render(&Frame::plain(26, 8));
    assert_eq!(text, SMALL_HEATMAP);
}

#[cfg(feature = "evcxr")]
const HTML_GRID: &str = r#"    a &lt; b &amp; c
3 ┤     <span style="color:#00cdcd">⢀⠔⠊⠑⠢⢄⣀</span>
  │  <span style="color:#00cdcd">⢀⡠⠊⠁      ⠉⠒⠤</span>
1 ┤<span style="color:#00cdcd">⡠⠔⠁</span>
  └┬─────────────┬
   0             2"#;

#[cfg(feature = "evcxr")]
const DARK_HTML: &str = r##"<pre style="margin:0;padding:12px 16px;border:0;border-radius:8px;box-sizing:border-box;display:inline-block;max-width:100%;overflow-x:auto;white-space:pre;font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:13px;line-height:1.1;font-variant-ligatures:none;font-feature-settings:"liga" 0,"calt" 0;background-color:#0d1117;color:#e6edf3">    a &lt; b &amp; c
3 ┤     <span style="color:#00cdcd">⢀⠔⠊⠑⠢⢄⣀</span>
  │  <span style="color:#00cdcd">⢀⡠⠊⠁      ⠉⠒⠤</span>
1 ┤<span style="color:#00cdcd">⡠⠔⠁</span>
  └┬─────────────┬
   0             2</pre>"##;

#[cfg(feature = "evcxr")]
const LIGHT_HTML: &str = r##"<pre style="margin:0;padding:12px 16px;border:0;border-radius:8px;box-sizing:border-box;display:inline-block;max-width:100%;overflow-x:auto;white-space:pre;font-family:ui-monospace,SFMono-Regular,Menlo,Monaco,Consolas,monospace;font-size:13px;line-height:1.1;font-variant-ligatures:none;font-feature-settings:"liga" 0,"calt" 0;background-color:#ffffff;color:#1f2328">    a &lt; b &amp; c
3 ┤     <span style="color:#00cdcd">⢀⠔⠊⠑⠢⢄⣀</span>
  │  <span style="color:#00cdcd">⢀⡠⠊⠁      ⠉⠒⠤</span>
1 ┤<span style="color:#00cdcd">⡠⠔⠁</span>
  └┬─────────────┬
   0             2</pre>"##;

#[cfg(feature = "evcxr")]
fn html_snapshot_plot() -> Plot<'static> {
    Plot::new()
        .layer(Line::y(vec![1.0, 3.0, 2.0]).color(crate::Color::Cyan))
        .title("a < b & c")
}

#[cfg(feature = "evcxr")]
#[test]
fn the_html_cell_grid_matches_its_snapshot() {
    let plot = html_snapshot_plot();
    assert_eq!(
        plot.rasterize(&Frame::plain(18, 6)).encode_html(),
        HTML_GRID
    );
}

#[cfg(feature = "evcxr")]
#[test]
fn html_cards_match_their_dark_and_light_snapshots() {
    let plot = html_snapshot_plot();
    let dark = Frame::plain(18, 6);
    assert_eq!(plot.to_html(&dark), DARK_HTML);

    let light = Frame {
        theme: crate::Theme::LIGHT,
        ..dark
    };
    assert_eq!(plot.to_html(&light), LIGHT_HTML);
}

#[test]
fn a_well_formed_plot_validates_and_try_renders() {
    let plot = crate::scatter(&[1.0, 2.0, 3.0][..], &[3.0, 1.0, 2.0][..]).title("ok");
    assert!(plot.validate().is_ok());
    assert!(plot.try_render(&Frame::plain(40, 10)).is_ok());
}

#[test]
fn a_log_axis_with_a_non_positive_domain_is_rejected() {
    let plot = crate::line(&[1.0, 10.0, 100.0][..])
        .y_domain(-1.0, 100.0)
        .log_y();
    assert!(matches!(
        plot.validate(),
        Err(crate::Error::IncompatibleScale { .. })
    ));
    // render still succeeds — it clamps rather than fails.
    assert!(!plot.render(&Frame::plain(40, 10)).is_empty());
    assert!(plot.try_render(&Frame::plain(40, 10)).is_err());
}

#[test]
fn validation_reaches_into_every_layer() {
    // A ragged range built by round-tripping through into_owned keeps its lengths,
    // so a valid multi-layer plot validates; the layer walk visits each mark.
    let plot = Plot::new()
        .layer(Line::xy(&[0.0, 1.0][..], &[2.0, 3.0][..]))
        .layer(crate::mark::Bars::new(["a", "b"], &[1.0, 2.0][..]));
    assert!(plot.validate().is_ok());
}

#[test]
fn an_explicit_scale_is_not_overridden_by_a_categorical_layer() {
    // Default (Auto) infers the categorical axis for bars.
    let auto = crate::bar(["a", "b"], &[1.0, 2.0][..]);
    assert!(auto.validate().is_ok());

    // An explicit numeric x scale with a bars layer is a conflict, not an override.
    let forced = crate::bar(["a", "b"], &[1.0, 2.0][..]).log_x();
    assert!(matches!(
        forced.validate(),
        Err(crate::Error::IncompatibleScale { .. })
    ));
    // Render stays lenient (it honors the scale rather than panicking).
    let _ = forced.render(&Frame::plain(40, 10));
}

#[test]
fn disagreeing_categorical_layers_are_rejected() {
    let plot = Plot::new()
        .layer(crate::mark::Bars::new(["a", "b"], &[1.0, 2.0][..]))
        .layer(crate::mark::Bars::new(["x", "y"], &[3.0, 4.0][..]));
    assert!(matches!(
        plot.validate(),
        Err(crate::Error::IncompatibleScale { .. })
    ));
}

#[test]
fn an_explicit_scale_without_categorical_layers_validates() {
    let plot = crate::line(&[1.0, 10.0, 100.0][..]).log_y();
    assert!(plot.validate().is_ok());
}

#[test]
fn zero_baseline_marks_are_rejected_on_log_axes() {
    let bars = crate::bar(["a", "b"], &[1.0, 10.0][..]).log_y();
    assert!(matches!(
        bars.validate(),
        Err(crate::Error::IncompatibleScale { .. })
    ));

    let baseline = Plot::new()
        .layer(crate::Area::xy([1.0, 10.0], [2.0, 20.0]))
        .log_y();
    assert!(matches!(
        baseline.validate(),
        Err(crate::Error::IncompatibleScale { .. })
    ));

    let band = Plot::new()
        .layer(crate::Area::between([1.0, 10.0], [2.0, 3.0], [4.0, 30.0]))
        .log_y();
    assert!(band.validate().is_ok());
}

#[test]
fn cells_require_matching_bands_and_positive_log_extents() {
    let values = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let bands = Plot::new()
        .layer(crate::Cells::matrix(3, values))
        .x_scale(crate::Scale::bands(["a", "b"]));
    assert!(matches!(
        bands.validate(),
        Err(crate::Error::IncompatibleScale { .. })
    ));
    let matched = Plot::new()
        .layer(crate::Cells::matrix(3, values))
        .x_scale(crate::Scale::bands(["a", "b", "c"]));
    assert!(matched.validate().is_ok());

    let implicit_zero = Plot::new().layer(crate::Cells::matrix(3, values)).log_x();
    assert!(matches!(
        implicit_zero.validate(),
        Err(crate::Error::IncompatibleScale { .. })
    ));

    let log = Plot::new()
        .layer(crate::Cells::matrix(3, values).extents((1.0, 1000.0), (1.0, 100.0)))
        .log_x()
        .log_y();
    assert!(log.validate().is_ok());
    assert!(!log.try_render(&Frame::plain(40, 10)).unwrap().is_empty());
}

#[test]
fn numeric_span_bars_do_not_silently_use_a_band_scale() {
    let plot = Plot::new()
        .layer(crate::Bars::spans(0.0, 1.0, [1.0, 2.0]))
        .x_scale(crate::Scale::bands(["a", "b"]));
    assert!(matches!(
        plot.validate(),
        Err(crate::Error::IncompatibleScale { .. })
    ));
}

#[test]
fn an_empty_explicit_band_scale_is_invalid() {
    let plot = crate::line(&[1.0, 2.0][..]).x_scale(crate::Scale::Bands(Vec::new()));
    assert!(matches!(
        plot.validate(),
        Err(crate::Error::EmptyDimension { .. })
    ));
}

#[test]
fn a_colorbar_legends_the_cells_value_range() {
    let grid: Vec<f64> = (0..24).map(|i| i as f64).collect();
    let with = crate::heatmap(6, &grid[..]).render(&Frame::plain(34, 8));
    // The value extent (0..23) is labeled with nice ticks beside the strip...
    assert!(
        with.contains("20") && with.contains("10"),
        "no value labels:\n{with}"
    );
    // ...and the strip itself shows the shade ramp so it reads in plain text.
    assert!(
        with.contains('\u{2588}') && with.contains('\u{2591}'),
        "no gradient"
    );
}

#[test]
fn color_by_matches_explicit_category_layers() {
    use crate::mark::Points;
    use crate::scale::Palette;

    // The compact channel remains visually equivalent to writing one explicit,
    // palette-colored, labeled layer per category by hand.
    let x = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let y = [2.0, 1.0, 3.0, 2.5, 0.5, 1.5];
    let species = ["a", "b", "a", "c", "b", "a"];
    let mut colored = Frame::plain(44, 12);
    colored.color = crate::ColorMode::TrueColor;

    let channel = Plot::new()
        .layer(Points::xy(&x[..], &y[..]).color_by(species))
        .render(&colored);

    let mask = |keep: &str| -> Vec<f64> {
        y.iter()
            .zip(species)
            .map(|(v, s)| if s == keep { *v } else { f64::NAN })
            .collect()
    };
    let by_hand = Plot::new()
        .layer(
            Points::xy(&x[..], mask("a"))
                .color(Palette::OKABE_ITO.colors()[0])
                .label("a"),
        )
        .layer(
            Points::xy(&x[..], mask("b"))
                .color(Palette::OKABE_ITO.colors()[1])
                .label("b"),
        )
        .layer(
            Points::xy(&x[..], mask("c"))
                .color(Palette::OKABE_ITO.colors()[2])
                .label("c"),
        )
        .render(&colored);
    assert_eq!(channel, by_hand);
}

#[test]
fn color_by_legends_categories_in_first_appearance_order() {
    use crate::mark::Points;

    let y = [1.0, 2.0, 3.0, 4.0];
    let text = Plot::new()
        .layer(Points::y(&y[..]).color_by(["gentoo", "adelie", "gentoo", "chinstrap"]))
        .render(&Frame::plain(50, 12));
    let gentoo = text.find("gentoo").expect("first category missing");
    let adelie = text.find("adelie").expect("second category missing");
    let chinstrap = text.find("chinstrap").expect("third category missing");
    assert!(
        gentoo < adelie && adelie < chinstrap,
        "legend order broke:\n{text}"
    );
}

#[test]
fn plain_output_cycles_markers_so_categories_stay_separable() {
    use crate::mark::Points;

    let x = [1.0, 2.0, 3.0];
    let y = [1.0, 2.0, 3.0];
    let plot = Plot::new().layer(Points::xy(&x[..], &y[..]).color_by(["a", "b", "c"]));

    // Colorless output: the second and third categories take whole-cell
    // markers; the legend swatches differ per category.
    let plain = plot.render(&Frame::plain(40, 10));
    assert!(
        plain.contains('+') && plain.contains('x'),
        "markers did not cycle in plain output:\n{plain}"
    );

    // Colored output: color separates the categories; the default dot stays.
    let mut colored = Frame::plain(40, 10);
    colored.color = crate::ColorMode::TrueColor;
    let ansi = plot.render(&colored);
    assert!(
        !ansi.contains('+') && !ansi.contains('x'),
        "markers cycled despite color being available:\n{ansi}"
    );
}

#[test]
fn more_categories_than_palette_colors_wrap_without_loss() {
    use crate::mark::Points;

    let y: Vec<f64> = (0..9).map(f64::from).collect();
    let names: Vec<String> = (0..9).map(|i| format!("g{i}")).collect();
    let text = Plot::new()
        .layer(Points::y(&y[..]).color_by(names))
        .render(&Frame::plain(70, 14));
    for i in 0..9 {
        assert!(
            text.contains(&format!("g{i}")),
            "category g{i} vanished:\n{text}"
        );
    }
}

#[test]
fn the_trend_preset_equals_its_grammar_expansion() {
    use crate::mark::{Area, Points};
    use crate::stat::Fit;

    let x = [1.0, 2.0, 3.0, 4.0, 5.0];
    let y = [1.2, 1.9, 3.2, 3.8, 5.1];
    let frame = Frame::plain(44, 12);

    let fit = Fit::xy(&x, &y);
    let expansion = Plot::new()
        .layer(Line::xy(
            vec![1.0, 5.0],
            vec![fit.predict(1.0).unwrap(), fit.predict(5.0).unwrap()],
        ))
        .layer(Points::xy(&x[..], &y[..]))
        .render(&frame);
    assert_eq!(crate::trend(&x[..], &y[..]).render(&frame), expansion);

    // With a band: the Area::between under line and points.
    let options = crate::TrendOptions::new().band(1.96);
    let samples = 64usize;
    let step = 4.0 / (samples - 1) as f64;
    let positions: Vec<f64> = (0..samples).map(|i| 1.0 + i as f64 * step).collect();
    let low: Vec<f64> = positions
        .iter()
        .map(|&at| fit.predict(at).unwrap() - 1.96 * fit.standard_error(at).unwrap())
        .collect();
    let high: Vec<f64> = positions
        .iter()
        .map(|&at| fit.predict(at).unwrap() + 1.96 * fit.standard_error(at).unwrap())
        .collect();
    let banded = Plot::new()
        .layer(Area::between(positions, low, high))
        .layer(Line::xy(
            vec![1.0, 5.0],
            vec![fit.predict(1.0).unwrap(), fit.predict(5.0).unwrap()],
        ))
        .layer(Points::xy(&x[..], &y[..]))
        .render(&frame);
    assert_eq!(
        crate::trend_with(&x[..], &y[..], options)
            .unwrap()
            .render(&frame),
        banded
    );
}

#[test]
fn degenerate_trend_data_draws_the_points_alone() {
    use crate::mark::Points;

    let frame = Frame::plain(40, 10);
    // No x spread: no line to draw, but the scatter still renders.
    let vertical = crate::trend(&[2.0, 2.0, 2.0][..], &[1.0, 2.0, 3.0][..]).render(&frame);
    let points_only = Plot::new()
        .layer(Points::xy(&[2.0, 2.0, 2.0][..], &[1.0, 2.0, 3.0][..]))
        .render(&frame);
    assert_eq!(vertical, points_only);

    let empty = crate::trend(&[][..] as &[f64], &[][..] as &[f64]).render(&frame);
    assert!(
        !empty.is_empty(),
        "an empty trend must still render a frame"
    );
}

#[test]
fn trend_with_rejects_meaningless_bands() {
    let x = [1.0, 2.0, 3.0];
    let y = [1.0, 2.0, 3.0];
    for multiplier in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(
            crate::trend_with(&x[..], &y[..], crate::TrendOptions::new().band(multiplier)).is_err(),
            "band multiplier {multiplier} should be rejected"
        );
    }
    let mut options = crate::TrendOptions::new().band(1.96);
    options.band_samples = 1;
    assert!(crate::trend_with(&x[..], &y[..], options).is_err());
}

#[test]
fn the_asymmetric_error_bars_preset_equals_its_grammar_expansion() {
    use crate::mark::{Points, Range};

    let x = [1.0, 2.0, 3.0];
    let y = [4.0, 6.0, 5.0];
    let minus = [0.5, 1.0, 0.4];
    let plus = [1.5, 0.3, 0.9];
    let frame = Frame::plain(40, 10);
    let expansion = Plot::new()
        .layer(Range::xy(
            &x[..],
            &[3.5, 5.0, 4.6][..],
            &[5.5, 6.3, 5.9][..],
        ))
        .layer(Points::xy(&x[..], &y[..]))
        .render(&frame);
    assert_eq!(
        crate::error_bars_asymmetric(&x[..], &y[..], &minus[..], &plus[..]).render(&frame),
        expansion
    );
}

#[test]
fn the_ecdf_band_is_the_dkw_envelope_and_rejects_bad_levels() {
    let values = [1.0, 2.0, 2.5, 3.0, 4.0, 4.5, 5.0, 6.0];
    let frame = Frame::plain(44, 12);
    let plain = crate::ecdf(&values[..]).render(&frame);
    let banded = crate::ecdf_with(&values[..], crate::EcdfOptions::new().band(0.05))
        .unwrap()
        .render(&frame);
    assert_ne!(plain, banded, "the band changed nothing");
    assert_eq!(
        crate::ecdf_with(&values[..], crate::EcdfOptions::default())
            .unwrap()
            .render(&frame),
        plain,
        "the default options must be the plain preset"
    );
    for alpha in [0.0, 1.0, -0.5, f64::NAN] {
        assert!(
            crate::ecdf_with(&values[..], crate::EcdfOptions::new().band(alpha)).is_err(),
            "alpha {alpha} should be rejected"
        );
    }
}

#[test]
fn the_heatmap_preset_equals_its_grammar_expansion() {
    use crate::mark::Cells;
    use crate::scale::Colormap;

    let values: Vec<f64> = (0..12).map(|i| (i as f64).sin()).collect();
    let frame = Frame::plain(30, 9);
    assert_eq!(
        crate::heatmap(4, &values[..]).render(&frame),
        Plot::new()
            .layer(Cells::matrix(4, &values[..]))
            .colorbar()
            .render(&frame),
    );
    let options = crate::HeatmapOptions::new()
        .colormap(Colormap::RED_BLUE.centered_at(0.0))
        .colorbar(false);
    assert_eq!(
        crate::heatmap_with(4, &values[..], options)
            .unwrap()
            .render(&frame),
        Plot::new()
            .layer(Cells::matrix(4, &values[..]).colormap(Colormap::RED_BLUE.centered_at(0.0)))
            .render(&frame),
    );
}

#[test]
fn a_centered_colormap_legends_the_symmetric_range() {
    use crate::mark::Cells;
    use crate::scale::Colormap;

    // Correlations on [-1, 0.5]: a linear bar labels the observed range, a
    // centered one widens to the symmetric [-1, 1] so the neutral middle sits
    // at the midpoint and the labels admit the widened span.
    let values = [-1.0, -0.5, 0.25, 0.5];
    let frame = Frame::plain(34, 8);
    let linear = Plot::new()
        .layer(Cells::matrix(2, &values[..]).colormap(Colormap::RED_BLUE))
        .colorbar()
        .render(&frame);
    let centered = Plot::new()
        .layer(Cells::matrix(2, &values[..]).colormap(Colormap::RED_BLUE.centered_at(0.0)))
        .colorbar()
        .render(&frame);

    assert_ne!(linear, centered, "centering changed nothing");
    assert!(
        centered.contains("-1"),
        "low end label missing:\n{centered}"
    );
    assert!(
        linear.contains("0.5") && !centered.contains("0.5"),
        "the centered bar should span [-1, 1], not the observed 0.5:\nlinear:\n{linear}\ncentered:\n{centered}"
    );
}

#[test]
fn a_colorbar_without_a_cells_layer_changes_nothing() {
    let frame = Frame::plain(40, 10);
    let bare = crate::line(&[1.0, 2.0, 3.0][..]);
    assert_eq!(
        bare.clone().colorbar().render(&frame),
        bare.render(&frame),
        "colorbar reserved space with nothing to show"
    );
}

/// Matrix reading order on a banded y axis: row 0 of the Cells grid is the top
/// band, row labels sit in the gutter at their band rows, and continuous marks
/// (the Text) position against band indices.
const BANDED_MATRIX: &str = "       │  ░░░░░  ▒▒▒▒▒\n       │  ░░░░░  ▒▒▒▒▒\n   top ┤  ░░░░░  ▒▒X▒▒\n       │  ▓▓▓▓▓  █████\n       │  ▓▓▓▓▓  █████\nbottom ┤  ▓▓▓▓▓  █████\n       │\n       └────────────────\n            a      b";

#[test]
fn banded_y_renders_matrix_order_with_row_labels() {
    let plot = Plot::new()
        .layer(crate::mark::Cells::matrix(2, &[0.0, 1.0, 2.0, 3.0][..]))
        .x_scale(crate::Scale::bands(["a", "b"]))
        .y_scale(crate::Scale::bands(["top", "bottom"]))
        .layer(crate::mark::Text::at(1.0, 0.0, "X"));
    assert!(plot.validate().is_ok());
    assert_eq!(plot.render(&Frame::plain(24, 9)), BANDED_MATRIX);
}

#[test]
fn band_axes_reject_what_they_cannot_encode() {
    let bars = Plot::new()
        .layer(crate::mark::Bars::new(["a", "b"], &[1.0, 2.0][..]))
        .y_scale(crate::Scale::bands(["p", "q"]));
    assert!(matches!(
        bars.validate(),
        Err(crate::Error::IncompatibleScale { .. })
    ));

    let extents = Plot::new()
        .layer(crate::mark::Cells::matrix(2, &[1.0, 2.0][..]).extents((0.0, 1.0), (0.0, 1.0)))
        .y_scale(crate::Scale::bands(["only"]));
    assert!(matches!(
        extents.validate(),
        Err(crate::Error::IncompatibleScale { .. })
    ));

    let mismatched_rows = Plot::new()
        .layer(crate::mark::Cells::matrix(2, &[1.0, 2.0, 3.0, 4.0][..]))
        .y_scale(crate::Scale::bands(["a", "b", "c"]));
    assert!(matches!(
        mismatched_rows.validate(),
        Err(crate::Error::IncompatibleScale { .. })
    ));

    let mismatched_columns = Plot::new()
        .layer(crate::mark::Cells::matrix(2, &[1.0, 2.0, 3.0, 4.0][..]))
        .x_scale(crate::Scale::bands(["a", "b", "c"]));
    assert!(matches!(
        mismatched_columns.validate(),
        Err(crate::Error::IncompatibleScale { .. })
    ));

    let empty = Plot::new()
        .layer(crate::mark::Line::y(&[1.0, 2.0][..]))
        .y_scale(crate::Scale::Bands(Vec::new()));
    assert!(matches!(
        empty.validate(),
        Err(crate::Error::EmptyDimension { .. })
    ));

    // The combination the restriction used to reject wholesale is now the
    // labeled-matrix contract: matching grids validate on both axes.
    let matched = Plot::new()
        .layer(crate::mark::Cells::matrix(2, &[1.0, 2.0, 3.0, 4.0][..]))
        .x_scale(crate::Scale::bands(["a", "b"]))
        .y_scale(crate::Scale::bands(["p", "q"]));
    assert!(matched.validate().is_ok());
}

/// A log colormap gives each decade its own shade instead of collapsing the
/// low rows into one, drops the zero row as a gap, and the colorbar labels
/// decades at logarithmic heights.
const LOG_DECADES: &str = "6 ┤████████████████  █ 10⁴\n  │████████████████  █\n  │▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓  ▓\n3 ┤▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒  ▒ 10²\n  │░░░░░░░░░░░░░░░░  ░\n0 ┤                  ░ 1\n  └┬───────────────┬\n   0               1";

#[test]
fn log_colormaps_spread_decades_and_gap_zero() {
    let values = [0.0, 1.0, 10.0, 100.0, 1000.0, 10000.0];
    let plot = Plot::new()
        .layer(
            crate::mark::Cells::matrix(1, &values[..])
                .colormap(crate::scale::Colormap::GREYS.log()),
        )
        .colorbar();
    assert!(plot.validate().is_ok());
    assert_eq!(plot.render(&Frame::plain(26, 8)), LOG_DECADES);
}

/// An rgb Cells grid draws direct colors; in plain output each pixel falls
/// back to its luma on the shade ramp — black `░`, white `█`.
const RGB_LUMA: &str =
    "3 ┤▓▓▓▓▓▓▓▓\n  │▒▒▒▒▒▒▒▒\n  │░░░░▓▓▓▓\n0 ┤░░░░████\n  └┬───────┬\n   0       2";

#[test]
fn rgb_cells_render_direct_colors_with_luma_fallback() {
    let pixels = [
        (0u8, 0, 0),
        (255, 255, 255),
        (255, 0, 0),
        (0, 0, 255),
        (0, 255, 0),
        (128, 128, 128),
    ];
    let plot = Plot::new().layer(crate::mark::Cells::rgb(2, &pixels[..]));
    assert!(plot.validate().is_ok());
    assert_eq!(plot.render(&Frame::plain(12, 6)), RGB_LUMA);

    let mut colored = Frame::plain(12, 6);
    colored.color = crate::ColorMode::TrueColor;
    let ansi = plot.render(&colored);
    assert!(ansi.contains("255;0;0"), "red pixel lost: {ansi:?}");
    assert!(ansi.contains("0;0;255"), "blue pixel lost: {ansi:?}");

    // No value scale: requesting a colorbar reserves nothing.
    let with_bar = Plot::new()
        .layer(crate::mark::Cells::rgb(2, &pixels[..]))
        .colorbar();
    assert_eq!(
        with_bar.render(&Frame::plain(12, 6)),
        plot.render(&Frame::plain(12, 6)),
        "an rgb grid has no value range for a colorbar to legend"
    );
}

/// Class cells paint one stable shade per category and legend the classes
/// with matching shade swatches, so regions stay separable without color.
const CLASS_REGIONS: &str = "     ░░ hot  ▒▒ cold\n2 ┤▒▒▒▒▒▒▒▒▒▒▒░░░░░░░░░░░\n  │▒▒▒▒▒▒▒▒▒▒▒░░░░░░░░░░░\n  │▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒\n  │░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒\n0 ┤░░░░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒\n  └┬─────────────────────┬\n   0                     2";

#[test]
fn class_cells_shade_regions_and_legend_them() {
    let plot = Plot::new().layer(crate::mark::Cells::classes(
        2,
        ["hot", "cold", "cold", "hot"],
    ));
    assert!(plot.validate().is_ok());
    assert_eq!(plot.render(&Frame::plain(26, 8)), CLASS_REGIONS);

    // No value scale: a requested colorbar reserves nothing.
    let with_bar = Plot::new()
        .layer(crate::mark::Cells::classes(2, ["a", "b", "b", "a"]))
        .colorbar();
    let without = Plot::new().layer(crate::mark::Cells::classes(2, ["a", "b", "b", "a"]));
    assert_eq!(
        with_bar.render(&Frame::plain(26, 8)),
        without.render(&Frame::plain(26, 8)),
    );
}

#[test]
fn dense_grids_reduce_bucket_exactly_instead_of_sampling() {
    // One row of 10,000 zeros with a single spike. Mean-reduced, the spike
    // dilutes into its bucket; max-reduced it must survive at full intensity.
    // Under the old per-patch sampling it would almost surely vanish.
    let mut values = vec![0.0f64; 10_000];
    values[6_173] = 1000.0;
    let frame = Frame::plain(44, 8);

    let max = Plot::new()
        .layer(crate::mark::Cells::matrix(10_000, &values[..]).reduce(crate::stat::Reducer::Max));
    assert!(
        max.render(&frame).contains('\u{2588}'),
        "a max-reduced spike must keep the ramp's top shade"
    );

    let mean = Plot::new().layer(crate::mark::Cells::matrix(10_000, &values[..]));
    assert!(
        !mean.render(&frame).contains('\u{2588}'),
        "the box filter dilutes an isolated spike into its bucket"
    );

    // Reducing a constant grid is exact for every reducer, so the choice must
    // not change a single glyph — a partition-independent invariant.
    let flat = vec![7.0f64; 10_000];
    let frame = Frame::plain(46, 8);
    let renders: Vec<String> = [
        crate::stat::Reducer::Mean,
        crate::stat::Reducer::Max,
        crate::stat::Reducer::Median,
    ]
    .into_iter()
    .map(|reducer| {
        Plot::new()
            .layer(crate::mark::Cells::matrix(10_000, &flat[..]).reduce(reducer))
            .render(&frame)
    })
    .collect();
    assert_eq!(renders[0], renders[1]);
    assert_eq!(renders[0], renders[2]);
}

/// Every escape in ANSI output must be a complete SGR sequence the encoder
/// wrote itself; any other control character is an injection leak.
fn assert_only_sgr_escapes(output: &str) {
    let mut chars = output.chars();
    while let Some(glyph) = chars.next() {
        if glyph == '\u{1b}' {
            assert_eq!(chars.next(), Some('['), "non-CSI escape in output");
            loop {
                let byte = chars.next().expect("unterminated escape sequence");
                if byte == 'm' {
                    break;
                }
                assert!(
                    byte.is_ascii_digit() || byte == ';',
                    "non-SGR escape byte {byte:?} in output"
                );
            }
        } else {
            assert!(
                glyph == '\n' || !glyph.is_control(),
                "control character {glyph:?} leaked into output"
            );
        }
    }
}

#[test]
fn hostile_labels_never_leak_control_bytes() {
    // An OSC title change, a clear-screen CSI, a C1 string terminator, DEL,
    // a CRLF, and a script tag — every slot that accepts caller text gets all
    // of them.
    let hostile = "\u{1b}]0;pwned\u{7}\u{1b}[2Jx\u{9c}\u{7f}\r\n<script>payload";

    // Two data layers so the palette assigns real colors and the ANSI encoder
    // emits SGR sequences around the hostile text.
    let numeric = Plot::new()
        .layer(Line::y(&[1.0, 5.0, 2.0][..]).label(hostile))
        .layer(Line::y(&[2.0, 1.0, 4.0][..]).label("clean"))
        .layer(crate::mark::Text::at(1.0, 3.0, hostile))
        .title(hostile)
        .x_label(hostile)
        .y_label(hostile);
    let bands = Plot::new()
        .layer(crate::mark::Bars::new([hostile, "ok"], &[3.0, 7.0][..]).color(crate::Color::Red))
        .title(hostile);
    let matrix = Plot::new()
        .layer(crate::mark::Cells::matrix(2, &[1.0, 2.0][..]))
        .x_scale(crate::Scale::bands([hostile, "ok"]))
        .y_scale(crate::Scale::bands([hostile]))
        .title(hostile);

    for plot in [&numeric, &bands, &matrix] {
        let plain = plot.render(&Frame::plain(48, 14));
        assert!(
            !plain.contains(|c: char| c != '\n' && c.is_control()),
            "control character leaked into plain output"
        );

        let mut colored = Frame::plain(48, 14);
        colored.color = crate::ColorMode::TrueColor;
        let ansi = plot.render(&colored);
        assert!(ansi.contains('\u{1b}'), "colored render exercised no SGR");
        assert_only_sgr_escapes(&ansi);

        // The printable remainder survives; only the control bytes vanish.
        assert!(plain.contains("payload"), "printable label text was lost");

        #[cfg(feature = "evcxr")]
        {
            let html = plot.to_html(&Frame::plain(48, 14));
            assert!(
                !html.contains(|c: char| c != '\n' && c.is_control()),
                "control character leaked into HTML output"
            );
            assert!(
                !html.contains("<script>"),
                "markup from a label survived HTML escaping"
            );
        }
    }
}
