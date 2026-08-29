use crate::plot::Frame;
use crate::{Bars, Line, Plot};

fn fixed_plot() -> Plot<'static> {
    Plot::new()
        .layer(Line::xy(&[0.0, 10.0][..], &[0.0, 10.0][..]))
        .x_domain(0.0, 10.0)
        .y_domain(0.0, 10.0)
}

#[test]
fn cells_and_data_round_trip_inside_the_plot_rectangle() {
    let plot = fixed_plot();
    let frame = Frame::plain(60, 20);
    let mapping = plot.mapping(&frame);
    let panel = mapping.plot_area().expect("a 60x20 frame draws a panel");
    let (left, top, columns, rows) = (panel.column, panel.row, panel.width, panel.height);
    assert!(
        columns > 10 && rows > 5,
        "panel too small: {columns}x{rows}"
    );

    for value in [0.0, 2.5, 5.0, 9.9] {
        let (column, row) = mapping.cell_at(value, value).expect("in-domain point");
        assert!((left..left + columns).contains(&column));
        assert!((top..top + rows).contains(&row));
        let (x, y) = mapping.data_at(column, row).expect("cell inside the panel");
        let (span_lo, span_hi) = mapping.x_span_at(column).expect("column inside the panel");
        assert!(
            span_lo <= value && value <= span_hi,
            "cell span [{span_lo}, {span_hi}] must cover {value}"
        );
        let cell_span = 10.0 / columns as f64;
        assert!((x - value).abs() <= cell_span, "x {x} too far from {value}");
        let row_span = 10.0 / rows as f64;
        assert!((y - value).abs() <= row_span, "y {y} too far from {value}");
    }
}

#[test]
fn queries_outside_the_plot_rectangle_are_none() {
    let plot = fixed_plot();
    let frame = Frame::plain(60, 20);
    let mapping = plot.mapping(&frame);
    let panel = mapping.plot_area().unwrap();
    let (left, top, columns, rows) = (panel.column, panel.row, panel.width, panel.height);
    assert_eq!(mapping.data_at(left.saturating_sub(1), top), None);
    assert_eq!(mapping.data_at(left, top + rows), None);
    assert_eq!(mapping.data_at(left + columns, top), None);
    assert_eq!(mapping.cell_at(11.0, 5.0), None, "beyond the domain");
    assert_eq!(mapping.cell_at(f64::NAN, 5.0), None);
    assert_eq!(mapping.x_span_at(left + columns), None);
}

#[test]
fn a_degenerate_frame_yields_an_empty_mapping() {
    let plot = fixed_plot();
    for frame in [Frame::plain(0, 0), Frame::plain(1, 1), Frame::plain(0, 10)] {
        let mapping = plot.mapping(&frame);
        if let Some(panel) = mapping.plot_area() {
            assert!(panel.width > 0 && panel.height > 0);
        } else {
            assert_eq!(mapping.data_at(0, 0), None);
            assert_eq!(mapping.cell_at(5.0, 5.0), None);
        }
    }
}

#[test]
fn the_resolved_domains_cover_the_manual_ones_exactly() {
    let plot = fixed_plot();
    let mapping = plot.mapping(&Frame::plain(60, 20));
    assert_eq!(mapping.x_domain(), (0.0, 10.0));
    assert_eq!(mapping.y_domain(), (0.0, 10.0));
}

#[test]
fn band_axes_answer_in_band_index_space_and_format_labels() {
    let plot = Plot::new().layer(Bars::new(["mon", "tue", "wed"], &[1.0, 3.0, 2.0][..]));
    let mapping = plot.mapping(&Frame::plain(50, 15));
    assert_eq!(
        mapping.x_categories().map(<[String]>::len),
        Some(3),
        "three named bands"
    );
    assert_eq!(
        mapping.x_categories().map(|c| c[0].as_str()),
        Some("mon"),
        "the labels themselves are exposed"
    );
    assert_eq!(mapping.y_categories(), None);
    assert_eq!(mapping.format_x(0.0), "mon");
    assert_eq!(mapping.format_x(1.4), "tue", "rounds to the nearest band");
    assert_eq!(mapping.format_x(9.0), "wed", "clamps to the last band");
    let (x, _) = {
        let panel = mapping.plot_area().unwrap();
        mapping
            .data_at(panel.column + panel.width / 2, panel.row)
            .unwrap()
    };
    assert!((0.0..=2.0).contains(&x), "band index in range, got {x}");
}

#[test]
fn linear_formatting_is_exact_decimal_at_cell_resolution() {
    let plot = Plot::new()
        .layer(Line::xy(&[0.0, 1.0][..], &[0.0, 1.0][..]))
        .x_domain(0.0, 1.0)
        .y_domain(0.0, 1.0);
    let mapping = plot.mapping(&Frame::plain(60, 20));
    let formatted = mapping.format_x(0.3);
    assert!(
        formatted
            .parse::<f64>()
            .is_ok_and(|v| (v - 0.3).abs() < 0.05),
        "parses near its value: {formatted}"
    );
    assert!(
        !formatted.contains("0.30000000000000004"),
        "no float artifacts: {formatted}"
    );
    assert!(formatted.len() <= 6, "cell resolution only: {formatted}");
    assert_eq!(mapping.format_x(f64::NAN), "NaN");
}

#[test]
fn time_formatting_reads_as_a_calendar_instant() {
    // Ten days in June 2024: day resolution, so the label carries the year.
    let start = 1_717_200_000.0;
    let end = start + 10.0 * 86_400.0;
    let stamps = [start, end];
    let plot = Plot::new()
        .layer(Line::xy(&stamps[..], &[0.0, 1.0][..]))
        .time_x()
        .x_domain(start, end);
    let mapping = plot.mapping(&Frame::plain(70, 20));
    let label = mapping.format_x(start + 86_400.0);
    assert!(label.contains("2024"), "calendar label with year: {label}");
    assert!(label.contains("Jun"), "calendar label with month: {label}");
}

#[test]
fn log_axes_map_and_format_in_value_space() {
    let plot = Plot::new()
        .layer(Line::xy(&[1.0, 1000.0][..], &[1.0, 1000.0][..]))
        .log_y()
        .y_domain(1.0, 1000.0);
    let mapping = plot.mapping(&Frame::plain(60, 24));
    let panel = mapping.plot_area().unwrap();
    let (left, top, columns, rows) = (panel.column, panel.row, panel.width, panel.height);
    let (_, y_top) = mapping.data_at(left + columns / 2, top).unwrap();
    let (_, y_bottom) = mapping.data_at(left + columns / 2, top + rows - 1).unwrap();
    assert!(y_top > y_bottom, "y grows upward: {y_top} vs {y_bottom}");
    assert!(y_bottom > 0.0, "log values stay positive");
    let (column, row) = mapping.cell_at(500.0, 100.0).expect("in-domain log point");
    let (_, y) = mapping.data_at(column, row).unwrap();
    assert!((y.log10() - 2.0).abs() < 0.2, "decade-space roundtrip: {y}");
    let _ = column;
}
