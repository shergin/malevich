//! End-to-end tests: fixtures piped through the real `kaz` binary, comparing exact
//! plot strings and the stream wiring — framing, flags, `-o`/`-O`, and the tally
//! are all under test at once (no assert_cmd; just `std::process::Command`).

use std::io::Write;
use std::process::{Command, Output, Stdio};

/// Runs `kaz ARGS`, feeding `input` on stdin, and returns the completed output.
fn run(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_kaz"))
        .args(args)
        // Deterministic detection regardless of the CI environment.
        .env_remove("NO_COLOR")
        .env_remove("CLICOLOR_FORCE")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn kaz");
    // A child that rejects its arguments exits without reading stdin, so this
    // write can race a closed pipe — that is the scenario under test, not a
    // harness failure.
    let write = child
        .stdin
        .take()
        .expect("stdin")
        .write_all(input.as_bytes());
    if let Err(error) = write {
        assert_eq!(
            error.kind(),
            std::io::ErrorKind::BrokenPipe,
            "write stdin: {error}"
        );
    }
    child.wait_with_output().expect("wait for kaz")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("utf-8 stdout")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("utf-8 stderr")
}

#[test]
fn out_of_range_numeric_selectors_are_usage_errors() {
    for (chart, flag) in [("line", "--cols"), ("scatter", "--by")] {
        let out = run(&[chart, flag, "999"], "1 2\n3 4\n");
        assert_eq!(out.status.code(), Some(2));
        assert!(
            stderr(&out).contains("column index 999 is out of range"),
            "{}",
            stderr(&out)
        );
    }
}

#[test]
fn histograms_handle_extreme_finite_endpoints_without_panicking() {
    let values = "-1.7976931348623157e308\n1.7976931348623157e308\n";
    for extra in [&[][..], &["--emit-code"][..]] {
        let mut args = vec!["hist", "--color", "never"];
        args.extend_from_slice(extra);
        let out = run(&args, values);
        assert!(out.status.success(), "{}", stderr(&out));
        assert!(!stderr(&out).contains("panicked"), "{}", stderr(&out));

        let mut args = vec!["hist", "--bins", "2", "--color", "never"];
        args.extend_from_slice(extra);
        let out = run(&args, values);
        assert!(out.status.success(), "{}", stderr(&out));
        assert!(!stderr(&out).contains("panicked"), "{}", stderr(&out));
    }

    let out = run(&["hist", "--bins", "1"], values);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("histogram extent cannot be represented"));
    assert!(!stderr(&out).contains("panicked"), "{}", stderr(&out));
}

// --- golden plots (plot forced onto stdout with `-o -`, plain and fixed-size) ---

#[test]
fn line_matches_golden() {
    let out = run(
        &[
            "line",
            "-o",
            "-",
            "--color",
            "never",
            "--charset",
            "braille",
            "-w",
            "40",
            "-h",
            "8",
        ],
        "1\n4\n2\n8\n5\n",
    );
    assert!(out.status.success());
    assert_eq!(stdout(&out), include_str!("golden/line.txt"));
}

#[test]
fn scatter_matches_golden() {
    let out = run(
        &[
            "scatter",
            "-o",
            "-",
            "--color",
            "never",
            "--charset",
            "braille",
            "-w",
            "40",
            "-h",
            "10",
        ],
        "1 2\n2 1\n3 5\n4 3\n5 4\n",
    );
    assert!(out.status.success());
    assert_eq!(stdout(&out), include_str!("golden/scatter.txt"));
}

#[test]
fn bar_matches_golden() {
    let out = run(
        &[
            "bar",
            "-o",
            "-",
            "--color",
            "never",
            "--charset",
            "braille",
            "-w",
            "30",
            "-h",
            "8",
        ],
        "a 3\nb 7\nc 5\n",
    );
    assert!(out.status.success());
    assert_eq!(stdout(&out), include_str!("golden/bar.txt"));
}

#[test]
fn hist_matches_golden() {
    let input: String = (1..=50).map(|n| format!("{n}\n")).collect();
    let out = run(
        &[
            "hist",
            "-o",
            "-",
            "--color",
            "never",
            "--charset",
            "braille",
            "-w",
            "40",
            "-h",
            "10",
        ],
        &input,
    );
    assert!(out.status.success());
    assert_eq!(stdout(&out), include_str!("golden/hist.txt"));
}

#[test]
fn count_matches_golden() {
    let out = run(
        &[
            "count",
            "-o",
            "-",
            "--color",
            "never",
            "--charset",
            "braille",
            "-w",
            "30",
            "-h",
            "8",
        ],
        "cat\ndog\ncat\nbird\ncat\ndog\n",
    );
    assert!(out.status.success());
    assert_eq!(stdout(&out), include_str!("golden/count.txt"));
}

#[test]
fn header_labels_the_series() {
    let out = run(
        &[
            "line",
            "-H",
            "-o",
            "-",
            "--color",
            "never",
            "--charset",
            "ascii",
            "-w",
            "44",
            "-h",
            "9",
        ],
        "step a b\n0 4 1\n1 2 3\n2 1 5\n",
    );
    assert!(out.status.success());
    assert_eq!(stdout(&out), include_str!("golden/line_header.txt"));
    // The legend proves the header names reached the series.
    assert!(stdout(&out).contains("a"));
    assert!(stdout(&out).contains("b"));
}

// --- the statistical set (M-C2) ---

/// Runs a chart with plain, fixed-size output forced onto stdout, and returns it.
fn plain(cmd: &[&str], width: &str, height: &str, input: &str) -> String {
    let mut args = cmd.to_vec();
    args.extend([
        "-o",
        "-",
        "--color",
        "never",
        "--charset",
        "braille",
        "-w",
        width,
        "-h",
        height,
    ]);
    let out = run(&args, input);
    assert!(out.status.success(), "chart {cmd:?} exited non-zero");
    stdout(&out)
}

#[test]
fn ecdf_matches_golden() {
    assert_eq!(
        plain(&["ecdf"], "40", "8", "3\n1\n4\n1\n5\n9\n2\n6\n"),
        include_str!("golden/ecdf.txt")
    );
}

#[test]
fn density_matches_golden() {
    assert_eq!(
        plain(
            &["density"],
            "44",
            "9",
            "1.0\n2.0\n2.0\n3.0\n3.0\n3.0\n4.0\n4.0\n5.0\n"
        ),
        include_str!("golden/density.txt")
    );
}

#[test]
fn box_matches_golden() {
    assert_eq!(
        plain(
            &["box", "-H"],
            "40",
            "12",
            "ctl trt\n1 2\n2 4\n3 5\n4 6\n9 7\n"
        ),
        include_str!("golden/box.txt")
    );
}

#[test]
fn heatmap_matches_golden() {
    assert_eq!(
        plain(&["heatmap"], "32", "9", "1 2 3\n4 5 6\n7 8 9\n"),
        include_str!("golden/heatmap.txt")
    );
}

#[test]
fn hist_bins_matches_golden() {
    let input: String = (1..=50).map(|n| format!("{n}\n")).collect();
    assert_eq!(
        plain(&["hist", "--bins", "5"], "44", "8", &input),
        include_str!("golden/hist_bins.txt")
    );
}

#[test]
fn time_x_gives_calendar_ticks() {
    let out = plain(
        &["line", "--time-x", "--fmt", "xy"],
        "52",
        "9",
        "2021-01-01 10\n2021-02-01 14\n2021-03-01 9\n2021-04-01 18\n",
    );
    assert_eq!(out, include_str!("golden/line_time.txt"));
    // Calendar-aligned labels prove the timestamps parsed and drove a time scale.
    assert!(out.contains("2021"));
    assert!(out.contains("Feb"));
}

// --- live mode (M-C3) ---

#[test]
fn live_streams_to_a_final_frame_with_cursor_discipline() {
    // A finite stream: stdin closes, the reader hits EOF, the loop draws a final
    // frame and exits. High fps keeps it quick.
    let out = run(
        &[
            "line",
            "--live",
            "--window",
            "8",
            "--fps",
            "120",
            "-t",
            "live",
            "--color",
            "never",
            "--charset",
            "ascii",
            "-w",
            "36",
            "-h",
            "7",
        ],
        "1\n4\n2\n8\n5\n3\n7\n6\n",
    );
    assert!(out.status.success());
    let plot = stderr(&out);
    assert!(
        plot.contains("live"),
        "the title is drawn in the final frame"
    );
    // Cursor hidden while repainting, and restored on the way out.
    assert!(plot.contains('\u{1b}'), "repaint uses escape sequences");
    assert!(
        plot.starts_with("\u{1b}[?25l"),
        "cursor hidden at the start"
    );
    assert!(
        plot.trim_end().ends_with("\u{1b}[?25h"),
        "cursor restored at the end"
    );
}

#[test]
fn live_rejects_non_line_charts() {
    let out = run(&["hist", "--live"], "1\n2\n3\n");
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("--live"));
}

// --- stream wiring (D-C4) ---

#[test]
fn plot_defaults_to_stderr_leaving_stdout_empty() {
    let out = run(
        &[
            "line",
            "--color",
            "never",
            "--charset",
            "ascii",
            "-w",
            "30",
            "-h",
            "6",
        ],
        "1\n3\n2\n5\n",
    );
    assert!(out.status.success());
    assert_eq!(stdout(&out), "", "no -O: stdout stays empty");
    assert!(!stderr(&out).is_empty(), "the plot goes to stderr");
}

#[test]
fn passthrough_echoes_input_on_stdout_with_plot_on_stderr() {
    let input = "1\n4\n2\n8\n5\n";
    let out = run(
        &[
            "line",
            "-O",
            "--color",
            "never",
            "--charset",
            "ascii",
            "-w",
            "40",
            "-h",
            "8",
        ],
        input,
    );
    assert!(out.status.success());
    // Mid-pipeline: stdout carries the data verbatim, stderr carries the plot.
    assert_eq!(stdout(&out), input);
    assert!(stderr(&out).contains('*') || !stderr(&out).is_empty());
}

#[test]
fn passthrough_with_plot_on_stdout_is_rejected() {
    let out = run(&["line", "-O", "-o", "-"], "1\n2\n3\n");
    assert!(!out.status.success());
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("-O"));
}

// --- the unparsed tally (D-C6) ---

#[test]
fn unparsed_fields_are_tallied_on_stderr() {
    let out = run(
        &[
            "line",
            "-o",
            "-",
            "--color",
            "never",
            "--charset",
            "ascii",
            "-w",
            "30",
            "-h",
            "6",
        ],
        "1\n2\noops\n4\nbad\n",
    );
    assert!(out.status.success());
    assert_eq!(stderr(&out), "2 values could not be parsed\n");
}

#[test]
fn one_unparsed_field_is_singular() {
    let out = run(
        &[
            "line",
            "-o",
            "-",
            "--color",
            "never",
            "--charset",
            "ascii",
            "-w",
            "30",
            "-h",
            "6",
        ],
        "1\n2\nnope\n4\n",
    );
    assert_eq!(stderr(&out), "1 value could not be parsed\n");
}

#[test]
fn quiet_suppresses_the_tally() {
    let out = run(
        &[
            "line",
            "-q",
            "-o",
            "-",
            "--color",
            "never",
            "--charset",
            "ascii",
            "-w",
            "30",
            "-h",
            "6",
        ],
        "1\n2\noops\n4\n",
    );
    assert_eq!(stderr(&out), "");
}

// --- color keyed to the destination (D-C4) ---

#[test]
fn color_never_emits_no_escapes_and_always_forces_them() {
    let input = "step a b\n0 4 1\n1 2 3\n2 1 5\n";
    let base = [
        "line",
        "-H",
        "-o",
        "-",
        "--charset",
        "ascii",
        "-w",
        "40",
        "-h",
        "8",
    ];

    let never: Vec<&str> = base.iter().copied().chain(["--color", "never"]).collect();
    let out = run(&never, input);
    assert!(!stdout(&out).contains('\x1b'), "never: no escapes");

    let always: Vec<&str> = base.iter().copied().chain(["--color", "always"]).collect();
    let out = run(&always, input);
    // Forced even though stdout is a pipe — the escape hatch overrides the tty gate.
    assert!(stdout(&out).contains('\x1b'), "always: escapes present");
}

// --- errors and meta ---

#[test]
fn unknown_subcommand_is_a_usage_error() {
    let out = run(&["bogus"], "");
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("unknown subcommand"));
}

#[test]
fn excessive_resource_flags_are_usage_errors() {
    for args in [
        vec!["line", "-w", "4097"],
        vec!["line", "-w", "4096", "-h", "4096"],
        vec!["hist", "--bins", "1000001"],
        vec!["line", "--live", "--window", "1000001"],
        vec!["line", "--live", "--fps", "1001"],
    ] {
        let out = run(&args, "1\n");
        assert_eq!(out.status.code(), Some(2), "{args:?}: {}", stderr(&out));
        assert!(stderr(&out).contains("must"), "{args:?}: {}", stderr(&out));
    }
}

/// Every chart the CLI ships, for the help coverage tests.
const CHARTS: [&str; 11] = [
    "line", "scatter", "bar", "hist", "count", "density", "ecdf", "box", "violin", "hist2d",
    "heatmap",
];

#[test]
fn help_goes_to_stdout_and_names_every_chart() {
    let out = run(&["--help"], "");
    assert!(out.status.success());
    let text = stdout(&out);
    for chart in CHARTS {
        assert!(text.contains(chart), "top-level help lists `{chart}`");
    }
}

#[test]
fn per_chart_help_is_specific() {
    for chart in CHARTS {
        let out = run(&[chart, "--help"], "");
        assert!(out.status.success());
        let text = stdout(&out);
        assert!(
            text.contains(&format!("kaz {chart}")),
            "`{chart}` help names itself"
        );
        assert!(text.contains("Shared options: kaz --help"));
    }
}

#[test]
fn version_prints() {
    let out = run(&["--version"], "");
    assert!(out.status.success());
    assert!(stdout(&out).starts_with("kaz "));
}
