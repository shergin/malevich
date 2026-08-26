use super::*;
use malevich::Frame;

#[test]
fn first_number_takes_the_first_finite_field() {
    assert_eq!(first_number("42", None), Some(42.0));
    assert_eq!(first_number("  3.5  extra", None), Some(3.5));
    // Leading non-numeric fields are skipped to reach the number.
    assert_eq!(first_number("time= 0.128", None), Some(0.128));
    assert_eq!(first_number("no numbers here", None), None);
}

#[test]
fn first_number_honors_the_delimiter() {
    assert_eq!(first_number("a,7,b", Some(',')), Some(7.0));
    assert_eq!(first_number("x,y,z", Some(',')), None);
}

#[test]
fn first_number_rejects_non_finite() {
    assert_eq!(first_number("inf", None), None);
    assert_eq!(first_number("nan", None), None);
}

#[test]
fn the_live_plot_carries_the_window_and_furniture() {
    let args = args_with(|a| {
        a.title = Some("ping".into());
        a.ylim = Some((0.0, 10.0));
    });
    let plot = plot(vec![1.0, 3.0, 2.0, 8.0], &args);
    let text = plot.render(&Frame::plain(40, 8));
    assert!(text.contains("ping"), "title is drawn");
    // The fixed y range means the top axis label is 10.
    assert!(text.contains("10"));
}

#[test]
fn the_live_plot_is_a_single_line_over_indices() {
    // No x furniture leaks in: even with --time-x set on the args, the sliding
    // window plots against its index, not a mangled time axis.
    let args = args_with(|a| a.time_x = true);
    let plot = plot(vec![0.0, 1.0, 2.0], &args);
    let text = plot.render(&Frame::plain(30, 6));
    assert!(!text.contains("1970"), "indices are not read as unix time");
}

/// A default `Args` for `line` under `--live`, mutated by `f`.
fn args_with(f: impl FnOnce(&mut crate::args::Args)) -> crate::args::Args {
    let mut args = crate::args::Args {
        command: crate::args::Command::Line,
        input: None,
        output: crate::args::Output::Stderr,
        passthrough: false,
        delimiter: None,
        header: false,
        fmt: None,
        width: None,
        height: None,
        title: None,
        xlabel: None,
        ylabel: None,
        xlim: None,
        ylim: None,
        log_x: false,
        log_y: false,
        time_x: false,
        bins: None,
        colormap: None,
        labels_x: None,
        labels_y: None,
        reduce: None,
        cols: None,
        by: None,
        emit_code: false,
        color: crate::args::ColorChoice::Auto,
        charset: crate::args::CharsetChoice::Auto,
        pixels: crate::args::PixelsChoice::Auto,
        quiet: false,
        live: true,
        window: None,
        fps: None,
        rate: false,
    };
    f(&mut args);
    args
}
