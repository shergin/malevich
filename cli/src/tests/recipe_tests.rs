use super::{Chart, PrepareError, ValueMark, prepare};
use crate::args::{Args, Outcome, parse_from};
use crate::input;

fn args(arguments: &[&str]) -> Args {
    match parse_from(lexopt::Parser::from_args(arguments.iter().copied())) {
        Ok(Outcome::Run(args)) => *args,
        other => panic!("expected a run, got {other:?}"),
    }
}

#[test]
fn projection_and_column_roles_are_resolved_in_the_recipe() {
    let args = args(&["line", "-H", "--cols", "step,loss,score", "--fmt", "xyy"]);
    let table = input::frame(
        "step score loss ignored\n1 0.8 4 x\n2 0.9 3 y\n",
        None,
        true,
    );
    let recipe = prepare(&args, table).unwrap();

    let Chart::Value { mark, series } = recipe.chart else {
        panic!("expected value data")
    };
    assert_eq!(mark, ValueMark::Line);
    assert_eq!(series.len(), 2);
    assert_eq!(series[0].x.as_deref(), Some(&[1.0, 2.0][..]));
    assert_eq!(series[0].y, [4.0, 3.0]);
    assert_eq!(series[0].label.as_deref(), Some("loss"));
    assert_eq!(series[1].y, [0.8, 0.9]);
    assert_eq!(series[1].label.as_deref(), Some("score"));
}

#[test]
fn grouping_is_extracted_before_scatter_parsing() {
    let args = args(&["scatter", "-H", "--by", "kind"]);
    let table = input::frame("x kind y\n1 a 2\n3 b 4\n", None, true);
    let recipe = prepare(&args, table).unwrap();

    let Chart::ScatterBy { x, y, groups } = recipe.chart else {
        panic!("expected grouped scatter data")
    };
    assert_eq!(x, [1.0, 3.0]);
    assert_eq!(y, [2.0, 4.0]);
    assert_eq!(groups, ["a", "b"]);
}

#[test]
fn automatic_histogram_geometry_is_prepared_once() {
    let args = args(&["hist"]);
    let table = input::frame("1\n2\n2\n3\nbad\n", None, false);
    let recipe = prepare(&args, table).unwrap();

    let Chart::Histogram { counts, .. } = recipe.chart else {
        panic!("expected histogram geometry")
    };
    assert_eq!(counts.iter().sum::<f64>(), 4.0);
    assert_eq!(recipe.unparsed, 1);
}

#[test]
fn selector_failures_remain_input_errors() {
    let args = args(&["line", "--cols", "4"]);
    let table = input::frame("1 2\n", None, false);
    let error = prepare(&args, table).unwrap_err();

    assert!(matches!(error, PrepareError::Input(_)));
    assert!(error.to_string().contains("column index 4"));
}
