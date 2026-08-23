use super::program;
use crate::args::{Args, Outcome, parse_from};
use crate::input;
use crate::recipe;

/// A parsed `Args` for an argument vector, as the process would build it.
fn args(arguments: &[&str]) -> Args {
    match parse_from(lexopt::Parser::from_args(arguments.iter().copied())) {
        Ok(Outcome::Run(args)) => *args,
        other => panic!("expected a run, got {other:?}"),
    }
}

/// Emits the program for one invocation over inline input text.
fn emit(arguments: &[&str], text: &str) -> String {
    let arguments = args(arguments);
    let table = input::frame(text, arguments.delimiter, arguments.header);
    let recipe = recipe::prepare(&arguments, table).expect("chart inputs are representable");
    program(&recipe)
}

#[test]
fn a_line_program_inlines_the_data_and_furniture() {
    let code = emit(
        &["line", "-t", "loss", "--ylabel", "nats", "--log-y"],
        "1 4\n2 3\n3 2.5\n",
    );
    assert!(
        code.contains("vec![1.0, 4.0]") || code.contains("vec![1.0, 2.0, 3.0]"),
        "{code}"
    );
    assert!(code.contains(".title(\"loss\")"), "{code}");
    assert!(code.contains(".y_label(\"nats\")"), "{code}");
    assert!(code.contains(".log_y()"), "{code}");
    assert!(code.contains("Frame::detect()"), "{code}");
}

#[test]
fn gaps_emit_as_nan_expressions_not_invalid_literals() {
    let code = emit(&["line"], "1\nnot-a-number\n3\n");
    assert!(code.contains("f64::NAN"), "{code}");
    assert!(!code.contains("vec![1.0, NaN"), "{code}");
}

#[test]
fn a_grouped_scatter_program_uses_the_color_channel() {
    let code = emit(
        &["scatter", "-H", "--by", "species"],
        "len depth species\n1 2 a\n3 4 b\n",
    );
    assert!(code.contains(".color_by(groups)"), "{code}");
    assert!(code.contains("vec![\"a\", \"b\"]"), "{code}");
}

#[test]
fn a_centered_colormap_emits_its_named_constant() {
    let code = emit(
        &["heatmap", "--colormap", "red-blue", "--midpoint", "0"],
        "1 -1\n-0.5 0.5\n",
    );
    assert!(
        code.contains("Colormap::RED_BLUE.centered_at(0.0)"),
        "{code}"
    );
}

/// The discipline behind the flag: emitted programs must compile. One scratch
/// cargo project with a path dependency on this workspace's malevich builds
/// every emission shape as its own binary.
#[test]
fn emitted_programs_compile() {
    let cases: [(&[&str], &str); 8] = [
        (
            &["line", "--fmt", "xyy", "-t", "training"],
            "1 4 5\n2 3 4\n3 2.5 3.5\n",
        ),
        (
            &["scatter", "-H", "--by", "kind"],
            "x y kind\n1 2 a\n3 4 b\n5 6 a\n",
        ),
        (&["hist", "--bins", "4"], "1\n2\n2.5\nbad\n3\n"),
        (&["bar"], "mon 3\ntue 7\n"),
        (&["count"], "200\n404\n200\n"),
        (&["ecdf", "--xlim", "0,10"], "1\n2\n3\n"),
        (&["box"], "1 4\n2 5\n3 6\n"),
        (
            &[
                "heatmap",
                "--colormap",
                "red-blue",
                "--midpoint",
                "0",
                "-w",
                "40",
                "-h",
                "12",
            ],
            "1 -1\n-0.5 0.5\n",
        ),
    ];

    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let workspace = manifest_dir
        .parent()
        .expect("cli sits inside the workspace");
    let scratch = workspace.join("target/emit-check");
    let src = scratch.join("src/bin");
    std::fs::create_dir_all(&src).expect("scratch project dirs");
    std::fs::write(
        scratch.join("Cargo.toml"),
        format!(
            "[package]\nname = \"emit-check\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n\
             [dependencies]\nmalevich = {{ path = {:?} }}\n\n[workspace]\n",
            workspace
        ),
    )
    .expect("scratch manifest");
    for (index, (arguments, text)) in cases.iter().enumerate() {
        std::fs::write(src.join(format!("case{index}.rs")), emit(arguments, text))
            .expect("scratch source");
    }

    let output = std::process::Command::new(env!("CARGO"))
        .args(["build", "--offline", "--quiet"])
        .current_dir(&scratch)
        .env(
            "CARGO_TARGET_DIR",
            workspace.join("target/emit-check-target"),
        )
        .output()
        .expect("cargo runs");
    assert!(
        output.status.success(),
        "emitted code failed to compile:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
