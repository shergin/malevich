use super::*;

#[test]
fn whitespace_runs_are_one_separator() {
    let table = frame("1   2\t3\n4 5 6\n", None, false);
    assert_eq!(table.header, None);
    assert_eq!(
        table.rows,
        vec![
            vec!["1".to_string(), "2".into(), "3".into()],
            vec!["4".into(), "5".into(), "6".into()],
        ]
    );
}

#[test]
fn bare_numbers_per_line_are_single_field_rows() {
    let table = frame("1\n4\n2\n", None, false);
    assert_eq!(table.rows, vec![vec!["1"], vec!["4"], vec!["2"]]);
}

#[test]
fn a_fixed_delimiter_preserves_empty_fields() {
    let table = frame("a,,b\n1,2,3\n", Some(','), false);
    assert_eq!(
        table.rows,
        vec![
            vec!["a".to_string(), "".into(), "b".into()],
            vec!["1".into(), "2".into(), "3".into()],
        ]
    );
}

#[test]
fn blank_lines_are_skipped_everywhere() {
    let table = frame("\n1 2\n\n  \n3 4\n", None, false);
    assert_eq!(table.rows, vec![vec!["1", "2"], vec!["3", "4"]]);
}

#[test]
fn a_header_consumes_the_first_nonblank_row() {
    let table = frame("\nstep loss\n0 4\n1 2\n", None, true);
    assert_eq!(table.header, Some(vec!["step".to_string(), "loss".into()]));
    assert_eq!(table.rows, vec![vec!["0", "4"], vec!["1", "2"]]);
}

#[test]
fn no_header_flag_leaves_the_first_row_as_data() {
    let table = frame("step loss\n0 4\n", None, false);
    assert_eq!(table.header, None);
    assert_eq!(table.rows.len(), 2);
}

#[test]
fn literal_delimiters_preserve_boundaries_for_ascii_unicode_and_nul() {
    for separator in [',', '|', '\u{1f9ea}', '\0'] {
        let text = format!("left{separator}{separator}right\n{separator}\n");
        let table = frame(&text, Some(separator), false);
        assert_eq!(table.rows[0], ["left", "", "right"]);
        assert_eq!(table.rows[1], ["", ""]);
    }
}

#[test]
fn crlf_and_lf_inputs_frame_identically() {
    assert_eq!(
        frame("name,value\r\na,1\r\nb,2\r\n", Some(','), true),
        frame("name,value\na,1\nb,2\n", Some(','), true)
    );
}

#[test]
fn schema_width_includes_both_the_header_and_ragged_rows() {
    let header_is_wider = frame("a b c\n1 2\n", None, true);
    assert_eq!(header_is_wider.width(), 3);

    let row_is_wider = frame("a b\n1 2 3\n", None, true);
    assert_eq!(row_is_wider.width(), 3);
}

#[test]
fn numeric_selectors_are_checked_against_the_schema() {
    let table = frame("a b c\n1 2\n3 4 5\n", None, true);
    assert_eq!(column_index(&table, "2"), Ok(2));
    assert!(column_index(&table, "3").unwrap_err().contains("0..=2"));
    assert!(
        column_index(&Table::default(), "0")
            .unwrap_err()
            .contains("no columns")
    );
    assert!(
        column_index(&table, "999999999999999999999999")
            .unwrap_err()
            .contains("too large")
    );
}

#[test]
fn selecting_a_valid_column_preserves_ragged_rows_as_missing_fields() {
    let table = frame("1 2\n3\n", None, false);
    let selected = select(&table, &["1".to_owned()]).unwrap();
    assert_eq!(selected.rows, vec![vec!["2"], vec![""]]);
    assert!(select(&table, &["2".to_owned()]).is_err());
}
