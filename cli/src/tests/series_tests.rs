use super::*;
use crate::input::frame;

#[test]
fn default_fmt_follows_the_column_count() {
    assert_eq!(default_fmt(0), Fmt::Y);
    assert_eq!(default_fmt(1), Fmt::Y);
    assert_eq!(default_fmt(2), Fmt::Xyy);
    assert_eq!(default_fmt(5), Fmt::Xyy);
}

#[test]
fn fmt_y_makes_each_column_a_series_over_its_index() {
    let table = frame("1 10\n2 20\n3 30\n", None, false);
    let data = dataset(&table, Fmt::Y, false);
    assert_eq!(data.series.len(), 2);
    assert!(data.series[0].x.is_none());
    assert_eq!(data.y(&data.series[0]), [1.0, 2.0, 3.0]);
    assert_eq!(data.y(&data.series[1]), [10.0, 20.0, 30.0]);
}

#[test]
fn fmt_xy_pairs_the_first_two_columns() {
    let table = frame("1 2\n3 4\n", None, false);
    let data = dataset(&table, Fmt::Xy, false);
    assert_eq!(data.series.len(), 1);
    assert_eq!(data.x(&data.series[0]), Some(&[1.0, 3.0][..]));
    assert_eq!(data.y(&data.series[0]), [2.0, 4.0]);
}

#[test]
fn fmt_xyy_shares_the_first_column_as_x() {
    let table = frame("0 1 2\n1 3 4\n", None, false);
    let data = dataset(&table, Fmt::Xyy, false);
    assert_eq!(data.series.len(), 2);
    assert_eq!(data.x(&data.series[0]), Some(&[0.0, 1.0][..]));
    assert_eq!(data.y(&data.series[0]), [1.0, 3.0]);
    assert_eq!(data.x(&data.series[1]), Some(&[0.0, 1.0][..]));
    assert_eq!(data.y(&data.series[1]), [2.0, 4.0]);
    assert_eq!(
        data.series[0].x, data.series[1].x,
        "xyy series reference one shared x buffer"
    );
}

#[test]
fn fmt_xyxy_takes_columns_in_pairs() {
    let table = frame("0 1 10 100\n1 2 11 101\n", None, false);
    let data = dataset(&table, Fmt::Xyxy, false);
    assert_eq!(data.series.len(), 2);
    assert_eq!(data.x(&data.series[0]), Some(&[0.0, 1.0][..]));
    assert_eq!(data.y(&data.series[0]), [1.0, 2.0]);
    assert_eq!(data.x(&data.series[1]), Some(&[10.0, 11.0][..]));
    assert_eq!(data.y(&data.series[1]), [100.0, 101.0]);
}

#[test]
fn fmt_yx_swaps_for_youplot_compatibility() {
    let table = frame("2 0\n4 1\n", None, false);
    let data = dataset(&table, Fmt::Yx, false);
    assert_eq!(data.series.len(), 1);
    assert_eq!(data.x(&data.series[0]), Some(&[0.0, 1.0][..]));
    assert_eq!(data.y(&data.series[0]), [2.0, 4.0]);
}

#[test]
fn header_names_become_series_labels() {
    let table = frame("step loss acc\n0 4 0.1\n1 2 0.9\n", None, true);
    let data = dataset(&table, Fmt::Xyy, false);
    assert_eq!(data.series[0].label.as_deref(), Some("loss"));
    assert_eq!(data.series[1].label.as_deref(), Some("acc"));
}

#[test]
fn a_present_but_unparseable_field_is_a_counted_gap() {
    let table = frame("1\n2\noops\n4\n", None, false);
    let data = dataset(&table, Fmt::Y, false);
    assert_eq!(data.unparsed, 1);
    assert!(data.y(&data.series[0])[2].is_nan());
    assert_eq!(data.y(&data.series[0])[3], 4.0);
}

#[test]
fn a_short_row_is_a_gap_but_not_a_parse_failure() {
    let table = frame("1 2\n3\n5 6\n", None, false);
    let data = dataset(&table, Fmt::Y, false);
    assert_eq!(
        data.unparsed, 0,
        "a missing field is structural, not unparseable"
    );
    assert!(
        data.y(&data.series[1])[1].is_nan(),
        "the absent second field is a gap"
    );
}

#[test]
fn non_finite_spellings_are_gaps_not_values() {
    let table = frame("1\ninf\nnan\n4\n", None, false);
    let data = dataset(&table, Fmt::Y, false);
    // `inf`/`nan` carry no position; they are counted as unparseable gaps.
    assert_eq!(data.unparsed, 2);
    assert!(data.y(&data.series[0])[1].is_nan());
    assert!(data.y(&data.series[0])[2].is_nan());
}

#[test]
fn flatten_pools_every_numeric_field() {
    let table = frame("1 2\n3 4\nbad 6\n", None, false);
    let (values, unparsed) = flatten(&table);
    assert_eq!(values, vec![1.0, 2.0, 3.0, 4.0, 6.0]);
    assert_eq!(unparsed, 1);
}

#[test]
fn labeled_values_take_label_then_height() {
    let table = frame("alpha 3\nbeta 7\ngamma\n", None, false);
    let (labels, values, unparsed) = labeled_values(&table);
    assert_eq!(labels, vec!["alpha", "beta", "gamma"]);
    assert_eq!(values[0], 3.0);
    assert_eq!(values[1], 7.0);
    assert!(values[2].is_nan(), "a bar with no value is a gap");
    assert_eq!(unparsed, 0, "the missing height is structural");
}

#[test]
fn counts_are_most_frequent_first_ties_by_label() {
    let table = frame("cat\ndog\ncat\nbird\ncat\ndog\n", None, false);
    assert_eq!(
        counts(&table),
        vec![
            ("cat".to_string(), 3.0),
            ("dog".into(), 2.0),
            ("bird".into(), 1.0)
        ]
    );

    let tied = frame("b\na\nb\na\nc\n", None, false);
    assert_eq!(
        counts(&tied),
        vec![("a".to_string(), 2.0), ("b".into(), 2.0), ("c".into(), 1.0)]
    );
}

#[test]
fn time_x_parses_the_x_column_as_timestamps() {
    let table = frame("2021-01-01 10\n2021-01-02 20\n", None, false);
    let data = dataset(&table, Fmt::Xy, true);
    // 2021-01-01 and 2021-01-02 as unix seconds, one day apart.
    assert_eq!(
        data.x(&data.series[0]),
        Some(&[1_609_459_200.0, 1_609_545_600.0][..])
    );
    assert_eq!(data.y(&data.series[0]), [10.0, 20.0]);
}

#[test]
fn time_x_leaves_the_y_column_numeric() {
    let table = frame("1609459200 5\n", None, false);
    let data = dataset(&table, Fmt::Xy, true);
    assert_eq!(data.x(&data.series[0]), Some(&[1_609_459_200.0][..]));
    assert_eq!(data.y(&data.series[0]), [5.0]);
}

#[test]
fn groups_name_columns_and_drop_gaps() {
    let table = frame("a b\n1 4\n2 5\nbad 6\n", None, true);
    let (categories, groups, unparsed) = groups(&table);
    assert_eq!(categories, vec!["a", "b"]);
    assert_eq!(
        groups[0],
        vec![1.0, 2.0],
        "the unparseable `bad` is dropped"
    );
    assert_eq!(groups[1], vec![4.0, 5.0, 6.0]);
    assert_eq!(unparsed, 1);
}

#[test]
fn groups_without_a_header_number_from_one() {
    let table = frame("1 4\n2 5\n", None, false);
    let (categories, _, _) = groups(&table);
    assert_eq!(categories, vec!["1", "2"]);
}

#[test]
fn xy_takes_the_first_two_columns() {
    let table = frame("1 2\n3 4\n", None, false);
    let (x, y, unparsed) = xy(&table, false);
    assert_eq!(x, vec![1.0, 3.0]);
    assert_eq!(y, vec![2.0, 4.0]);
    assert_eq!(unparsed, 0);
}

#[test]
fn matrix_flips_rows_so_the_first_line_is_on_top() {
    // Input rows: bottom line becomes heatmap row 0 (the bottom).
    let table = frame("1 2\n3 4\n", None, false);
    let (columns, values, unparsed) = matrix(&table);
    assert_eq!(columns, 2);
    // row 0 (bottom) = last input line `3 4`, then `1 2` above it.
    assert_eq!(values, vec![3.0, 4.0, 1.0, 2.0]);
    assert_eq!(unparsed, 0);
}
