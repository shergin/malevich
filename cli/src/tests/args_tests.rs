use super::*;

/// Parses an argument vector (without the binary name), as the process would.
fn parse(args: &[&str]) -> Result<Outcome, Fail> {
    parse_from(lexopt::Parser::from_args(args.iter().copied()))
}

/// Unwraps to the `Args` of a successful run, or panics with the failure.
fn run(args: &[&str]) -> Args {
    match parse(args) {
        Ok(Outcome::Run(args)) => *args,
        Ok(other) => panic!("expected a run, got {other:?}"),
        Err(fail) => panic!("expected a run, got error: {fail}"),
    }
}

#[test]
fn subcommands_and_their_aliases_resolve() {
    assert_eq!(run(&["line"]).command, Command::Line);
    assert_eq!(run(&["l"]).command, Command::Line);
    assert_eq!(run(&["scatter"]).command, Command::Scatter);
    assert_eq!(run(&["s"]).command, Command::Scatter);
    assert_eq!(run(&["bar"]).command, Command::Bar);
    assert_eq!(run(&["b"]).command, Command::Bar);
    assert_eq!(run(&["hist"]).command, Command::Hist);
    assert_eq!(run(&["count"]).command, Command::Count);
    assert_eq!(run(&["c"]).command, Command::Count);
    assert_eq!(run(&["density"]).command, Command::Density);
    assert_eq!(run(&["d"]).command, Command::Density);
    assert_eq!(run(&["box"]).command, Command::Box);
    assert_eq!(run(&["ecdf"]).command, Command::Ecdf);
    assert_eq!(run(&["violin"]).command, Command::Violin);
    assert_eq!(run(&["hist2d"]).command, Command::Hist2d);
    assert_eq!(run(&["heatmap"]).command, Command::Heatmap);
}

#[test]
fn bins_takes_a_positive_count() {
    assert_eq!(run(&["hist", "--bins", "20"]).bins, Some(20));
    assert!(parse(&["hist", "--bins", "0"]).is_err());
    assert!(parse(&["hist", "--bins", "-3"]).is_err());
    assert!(parse(&["hist", "--bins", "1000001"]).is_err());
}

#[test]
fn colormap_resolves_names_and_centers_on_the_midpoint() {
    use malevich::scale::Colormap;

    assert_eq!(run(&["heatmap"]).colormap, None);
    assert_eq!(
        run(&["heatmap", "--colormap", "magma"]).colormap,
        Some(Colormap::MAGMA)
    );
    assert_eq!(
        run(&["hist2d", "--colormap", "red-blue", "--midpoint", "0"]).colormap,
        Some(Colormap::RED_BLUE.centered_at(0.0))
    );
    // A bare --midpoint centers the default map.
    assert_eq!(
        run(&["heatmap", "--midpoint", "1"]).colormap,
        Some(Colormap::DEFAULT.centered_at(1.0))
    );

    assert!(
        parse(&["heatmap", "--colormap", "jet"]).is_err(),
        "no rainbow maps"
    );
    assert!(parse(&["heatmap", "--midpoint", "nan"]).is_err());
    assert!(parse(&["heatmap", "--midpoint", "much"]).is_err());
    // The flags mean nothing without a gridded chart.
    assert!(parse(&["line", "--colormap", "magma"]).is_err());
    assert!(parse(&["line", "--midpoint", "0"]).is_err());
}

#[test]
fn cols_by_and_emit_code_parse_with_their_boundaries() {
    assert_eq!(
        run(&["line", "--cols", "time, loss"]).cols,
        Some(vec!["time".to_string(), "loss".to_string()])
    );
    assert!(
        parse(&["line", "--cols", "a,,b"]).is_err(),
        "empty selector"
    );
    assert_eq!(
        run(&["scatter", "--by", "species"]).by.as_deref(),
        Some("species")
    );
    assert!(run(&["hist", "--emit-code"]).emit_code);

    // --by is scatter's; nothing survives into --live.
    assert!(parse(&["line", "--by", "species"]).is_err());
    assert!(parse(&["line", "--live", "--cols", "0"]).is_err());
    assert!(parse(&["line", "--live", "--emit-code"]).is_err());
}

#[test]
fn time_x_is_a_flag() {
    assert!(run(&["line", "--time-x"]).time_x);
    assert!(!run(&["line"]).time_x);
}

#[test]
fn live_flags_parse() {
    let args = run(&["line", "--live", "--window", "50", "--fps", "20", "--rate"]);
    assert!(args.live);
    assert_eq!(args.window, Some(50));
    assert_eq!(args.fps, Some(20));
    assert!(args.rate);
}

#[test]
fn live_is_line_only() {
    assert!(parse(&["hist", "--live"]).is_err());
    assert!(parse(&["scatter", "--live"]).is_err());
    assert!(run(&["line", "--live"]).live);
}

#[test]
fn live_rejects_incompatible_destinations() {
    assert!(parse(&["line", "--live", "-O"]).is_err());
    assert!(parse(&["line", "--live", "-o", "out.txt"]).is_err());
    // Plot on stdout is fine for live.
    assert!(run(&["line", "--live", "-o", "-"]).live);
}

#[test]
fn window_fps_and_rate_require_live() {
    assert!(parse(&["line", "--window", "50"]).is_err());
    assert!(parse(&["line", "--fps", "20"]).is_err());
    assert!(parse(&["line", "--rate"]).is_err());
}

#[test]
fn window_and_fps_reject_zero() {
    assert!(parse(&["line", "--live", "--window", "0"]).is_err());
    assert!(parse(&["line", "--live", "--fps", "0"]).is_err());
    assert!(parse(&["line", "--live", "--window", "1000001"]).is_err());
    assert!(parse(&["line", "--live", "--fps", "1001"]).is_err());
}

#[test]
fn flags_the_chart_would_ignore_are_rejected() {
    assert!(parse(&["line", "--bins", "5"]).is_err());
    assert!(parse(&["bar", "--time-x"]).is_err());
    assert!(parse(&["hist", "--fmt", "xy"]).is_err());
    // The valid homes still parse.
    assert!(run(&["hist", "--bins", "5"]).bins.is_some());
    assert!(run(&["hist2d", "--time-x"]).time_x);
    assert!(run(&["scatter", "--fmt", "xy"]).fmt.is_some());
}

#[test]
fn live_rejects_flags_the_sliding_window_ignores() {
    assert!(parse(&["line", "--live", "--time-x"]).is_err());
    assert!(parse(&["line", "--live", "--xlim", "0,10"]).is_err());
    assert!(parse(&["line", "--live", "--log-x"]).is_err());
    assert!(parse(&["line", "--live", "--fmt", "xy"]).is_err());
    assert!(parse(&["line", "--live", "-H"]).is_err());
    assert!(parse(&["line", "--live", "--pixels", "always"]).is_err());
    // The y side still applies to a live plot.
    assert!(run(&["line", "--live", "--ylim", "0,10", "--log-y"]).log_y);
}

#[test]
fn no_subcommand_is_the_top_level_help() {
    assert!(matches!(parse(&[]), Ok(Outcome::Help(None))));
}

#[test]
fn help_carries_the_subcommand_it_followed() {
    assert!(matches!(
        parse(&["bar", "--help"]),
        Ok(Outcome::Help(Some(Command::Bar)))
    ));
    // Before any subcommand, --help is the top-level page.
    assert!(matches!(parse(&["--help"]), Ok(Outcome::Help(None))));
}

#[test]
fn version_short_and_long() {
    assert!(matches!(parse(&["-V"]), Ok(Outcome::Version)));
    assert!(matches!(
        parse(&["line", "--version"]),
        Ok(Outcome::Version)
    ));
}

#[test]
fn unknown_subcommand_fails() {
    assert!(parse(&["bogus"]).is_err());
}

#[test]
fn a_positional_after_the_chart_is_the_input_file() {
    let args = run(&["line", "data.tsv"]);
    assert_eq!(
        args.input.as_deref(),
        Some(std::path::Path::new("data.tsv"))
    );
}

#[test]
fn a_second_positional_is_rejected() {
    assert!(parse(&["line", "a.tsv", "b.tsv"]).is_err());
}

#[test]
fn output_target_maps_dash_to_stdout_and_a_name_to_a_file() {
    assert_eq!(run(&["line", "-o", "-"]).output, Output::Stdout);
    assert_eq!(run(&["line"]).output, Output::Stderr);
    assert_eq!(
        run(&["line", "-o", "out.txt"]).output,
        Output::File("out.txt".into())
    );
}

#[test]
fn passthrough_to_stdout_conflicts_with_plot_on_stdout() {
    assert!(parse(&["line", "-O", "-o", "-"]).is_err());
    // -O with the default stderr plot is fine.
    assert!(run(&["line", "-O"]).passthrough);
}

#[test]
fn delimiter_is_a_single_character() {
    assert_eq!(run(&["line", "-d", ","]).delimiter, Some(','));
    assert!(parse(&["line", "-d", ",,"]).is_err());
    assert!(parse(&["line", "-d", ""]).is_err());
}

#[test]
fn fmt_parses_and_rejects_junk() {
    assert_eq!(run(&["line", "--fmt", "xyy"]).fmt, Some(Fmt::Xyy));
    assert_eq!(run(&["line", "--fmt", "yx"]).fmt, Some(Fmt::Yx));
    assert!(parse(&["line", "--fmt", "zz"]).is_err());
}

#[test]
fn limits_parse_a_pair_and_reject_the_rest() {
    assert_eq!(run(&["line", "--xlim", "0,10"]).xlim, Some((0.0, 10.0)));
    assert_eq!(run(&["line", "--ylim", "-1.5,2"]).ylim, Some((-1.5, 2.0)));
    assert!(parse(&["line", "--xlim", "0"]).is_err());
    assert!(parse(&["line", "--xlim", "a,b"]).is_err());
    assert!(parse(&["line", "--xlim", "0,inf"]).is_err());
}

#[test]
fn size_flags_take_numbers_and_h_is_height() {
    let args = run(&["line", "-w", "100", "-h", "20"]);
    assert_eq!(args.width, Some(100));
    assert_eq!(args.height, Some(20));
    assert!(parse(&["line", "-w", "4097"]).is_err());
    assert!(parse(&["line", "-h", "4097"]).is_err());
    assert!(parse(&["line", "-w", "4096", "-h", "4096"]).is_err());
}

#[test]
fn color_charset_and_pixels_ladders() {
    assert_eq!(
        run(&["line", "--color", "always"]).color,
        ColorChoice::Always
    );
    assert_eq!(run(&["line", "--color", "never"]).color, ColorChoice::Never);
    assert!(parse(&["line", "--color", "sometimes"]).is_err());

    assert_eq!(
        run(&["line", "--charset", "octant"]).charset,
        CharsetChoice::Fixed(malevich::Charset::Octants)
    );
    assert_eq!(
        run(&["line", "--charset", "auto"]).charset,
        CharsetChoice::Auto
    );
    assert!(parse(&["line", "--charset", "crayon"]).is_err());

    assert_eq!(
        run(&["line", "--pixels", "never"]).pixels,
        PixelsChoice::Never
    );
    assert!(parse(&["line", "--pixels", "maybe"]).is_err());
}

#[test]
fn header_log_and_quiet_are_flags() {
    let args = run(&["line", "-H", "--log-x", "--log-y", "-q"]);
    assert!(args.header);
    assert!(args.log_x);
    assert!(args.log_y);
    assert!(args.quiet);
}

#[test]
fn title_and_axis_labels() {
    let args = run(&[
        "line", "-t", "loss", "--xlabel", "step", "--ylabel", "value",
    ]);
    assert_eq!(args.title.as_deref(), Some("loss"));
    assert_eq!(args.xlabel.as_deref(), Some("step"));
    assert_eq!(args.ylabel.as_deref(), Some("value"));
}

#[test]
fn heatmap_band_and_reduction_flags_parse() {
    let args = run(&[
        "heatmap",
        "--labels-x",
        "cat, dog",
        "--labels-y",
        "p,q",
        "--reduce",
        "max",
        "--log-color",
    ]);
    assert_eq!(
        args.labels_x.as_deref(),
        Some(&["cat".to_string(), "dog".to_string()][..])
    );
    assert_eq!(
        args.labels_y.as_deref(),
        Some(&["p".to_string(), "q".to_string()][..])
    );
    assert!(matches!(args.reduce, Some(malevich::stat::Reducer::Max)));
    assert!(args.colormap.as_ref().is_some_and(|map| map.is_log()));
}

#[test]
fn heatmap_flags_stay_off_other_charts_and_conflicts_fail() {
    assert!(parse(&["line", "--labels-x", "a"]).is_err());
    assert!(parse(&["hist2d", "--reduce", "max"]).is_err());
    assert!(parse(&["line", "--log-color"]).is_err());
    assert!(parse(&["heatmap", "--log-color", "--midpoint", "0"]).is_err());
    assert!(parse(&["heatmap", "--reduce", "p95"]).is_err());
    assert!(parse(&["heatmap", "--labels-x", "a,,b"]).is_err());
    // hist2d keeps --log-color: its counts span decades too.
    assert!(
        run(&["hist2d", "--log-color"])
            .colormap
            .as_ref()
            .is_some_and(|map| map.is_log())
    );
}
