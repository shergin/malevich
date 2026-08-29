use crate::render::{Charset, Color};
use crate::scale::Scale;
use crate::{
    Bars, Cells, Document, DocumentKind, Frame, Grid, Line, LineStyle, Plot, PointStyle, Points,
    Rule, Text,
};

const V1_PLOT: &str = include_str!("../tests/fixtures/serde/v1/plot.json");
const V1_GRID: &str = include_str!("../tests/fixtures/serde/v1/grid.json");
const LEGACY_PLOT_1_14: &str = include_str!("../tests/fixtures/serde/legacy/plot-1.14.json");

fn frame() -> Frame {
    Frame {
        charset: Charset::Braille,
        ..Frame::plain(64, 18)
    }
}

fn fixture_plot() -> Plot<'static> {
    Plot::new()
        .layer(
            Line::y(vec![1.0, f64::NAN, 3.0])
                .color(Color::Cyan)
                .label("series"),
        )
        .title("v1 plot")
        .x_label("step")
        .y_label("value")
        .y_domain(0.0, 4.0)
}

fn fixture_grid() -> Grid<'static> {
    Grid::new(2).with(Plot::new().title("pane"))
}

#[test]
fn v1_documents_match_their_golden_wire_fixtures() {
    let plot = Document::plot(fixture_plot()).unwrap();
    assert_eq!(plot.version(), 1);
    assert_eq!(plot.kind(), DocumentKind::Plot);
    assert!(plot.as_plot().is_some());
    assert!(plot.as_grid().is_none());
    assert_eq!(
        serde_json::to_value(&plot).unwrap(),
        serde_json::from_str::<serde_json::Value>(V1_PLOT).unwrap()
    );

    let grid = Document::grid(fixture_grid()).unwrap();
    assert_eq!(grid.kind(), DocumentKind::Grid);
    assert!(grid.as_grid().is_some());
    assert_eq!(
        serde_json::to_value(&grid).unwrap(),
        serde_json::from_str::<serde_json::Value>(V1_GRID).unwrap()
    );
}

#[test]
fn every_supported_document_fixture_decodes_validates_and_renders() {
    let plot: Document = serde_json::from_str(V1_PLOT).unwrap();
    assert!(plot.validate().is_ok());
    assert_eq!(
        plot.try_render(&frame()).unwrap(),
        fixture_plot().try_render(&frame()).unwrap()
    );

    let grid: Document = serde_json::from_str(V1_GRID).unwrap();
    assert!(grid.validate().is_ok());
    assert!(!grid.render(&frame()).is_empty());
}

#[test]
fn legacy_raw_plot_payloads_remain_decodable() {
    let decoded: Plot = serde_json::from_str(LEGACY_PLOT_1_14).unwrap();
    assert!(decoded.validate().is_ok());
    assert_eq!(decoded.render(&frame()), fixture_plot().render(&frame()));
}

#[test]
fn documents_reject_unknown_versions_and_invalid_specs() {
    let mut future: serde_json::Value = serde_json::from_str(V1_PLOT).unwrap();
    future["version"] = 2.into();
    let error = serde_json::from_value::<Document>(future).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("unsupported malevich document version")
    );

    let mut invalid: serde_json::Value = serde_json::from_str(V1_GRID).unwrap();
    invalid["spec"]["columns"] = 0.into();
    let error = serde_json::from_value::<Document>(invalid).unwrap_err();
    assert!(error.to_string().contains("Grid columns is empty"));
}

#[test]
fn documents_accept_additive_fields_and_default_omitted_plot_fields() {
    let mut value: serde_json::Value = serde_json::from_str(V1_PLOT).unwrap();
    value["future_envelope_field"] = true.into();
    value["spec"]["future_plot_field"] = "ignored".into();
    value["spec"].as_object_mut().unwrap().remove("colorbar");
    let document: Document = serde_json::from_value(value).unwrap();
    assert!(document.validate().is_ok());
    assert!(!document.render(&frame()).is_empty());
}

#[test]
fn documents_take_ownership_of_borrowed_series() {
    let document = {
        let values = [1.0, 2.0, 3.0];
        Document::try_from(crate::line(&values[..])).unwrap()
    };
    assert!(!document.render(&frame()).is_empty());
}

#[test]
fn a_full_spec_round_trips_to_an_identical_render() {
    let plot = Plot::new()
        .layer(
            Line::xy(
                &[0.0, 1.0, 2.0, 3.0, 4.0][..],
                &[1.0, f64::NAN, 4.0, 2.0, 5.0][..],
            )
            .label("with a gap")
            .color(Color::Rgb(200, 40, 90))
            .style(LineStyle::Corners),
        )
        .layer(
            Points::xy(&[0.5, 2.5][..], &[3.0, 1.0][..])
                .style(PointStyle::Cross)
                .label("crosses"),
        )
        .layer(Rule::h(2.5))
        .layer(Text::at(1.0, 4.5, "note"))
        .title("round trip")
        .x_label("x")
        .y_label("y")
        .y_domain(0.0, 6.0);

    let encoded = serde_json::to_string(&plot).expect("serializes");
    let decoded: Plot = serde_json::from_str(&encoded).expect("deserializes");
    assert_eq!(plot.render(&frame()), decoded.render(&frame()));
}

#[test]
fn point_styles_round_trip_and_old_payloads_default_to_dots() {
    let points = Points::y(vec![1.0, 2.0]).style(PointStyle::Plus);
    let encoded = serde_json::to_string(&points).unwrap();
    let decoded: Points<'static> = serde_json::from_str(&encoded).unwrap();
    assert_eq!(decoded.style, PointStyle::Plus);

    let legacy = r#"{"x":null,"y":[1.0],"color":null,"label":null}"#;
    let decoded: Points<'static> = serde_json::from_str(legacy).unwrap();
    assert_eq!(decoded.style, PointStyle::Dot);
}

#[test]
fn gaps_survive_json_as_nulls() {
    let plot = Plot::new().layer(Line::y(&[1.0, f64::NAN, 3.0][..]));
    let encoded = serde_json::to_string(&plot).expect("serializes");
    assert!(encoded.contains("[1.0,null,3.0]"), "{encoded}");
    let decoded: Plot = serde_json::from_str(&encoded).expect("deserializes");
    assert_eq!(plot.render(&frame()), decoded.render(&frame()));
}

#[test]
fn bands_cells_and_log_scales_round_trip_as_valid_specs() {
    let plots = [
        Plot::new()
            .layer(Bars::new(["a", "b", "c"], &[3.0, 7.0, 5.0][..]))
            .x_scale(Scale::bands(["a", "b", "c"])),
        Plot::new()
            .layer(Cells::matrix(2, &[1.0, 2.0, 3.0, 4.0][..]).extents((1.0, 100.0), (1.0, 1000.0)))
            .x_scale(Scale::Log)
            .y_scale(Scale::Log),
        Plot::new().layer(Cells::rgb(2, vec![(0u8, 0, 0), (255, 255, 255)])),
        Plot::new().layer(Cells::classes(2, ["a", "b", "b", "a"])),
        Plot::new()
            .layer(Cells::matrix(2, &[1.0, 2.0, 3.0, 4.0][..]).reduce(crate::stat::Reducer::Max)),
    ];
    for plot in plots {
        assert!(plot.validate().is_ok());
        let encoded = serde_json::to_string(&plot).expect("serializes");
        let decoded: Plot = serde_json::from_str(&encoded).expect("deserializes");
        assert!(decoded.validate().is_ok());
        assert_eq!(plot.render(&frame()), decoded.render(&frame()));
    }

    // Value grids encode exactly as before the rgb and classes channels existed.
    let matrix = Plot::new().layer(Cells::matrix(1, &[1.0][..]));
    let encoded = serde_json::to_string(&matrix).expect("serializes");
    assert!(!encoded.contains("rgb"), "spurious field: {encoded}");
    assert!(!encoded.contains("classes"), "spurious field: {encoded}");
    assert!(!encoded.contains("reduce"), "spurious field: {encoded}");

    // A non-default reducer is carried; an out-of-range deserialized
    // percentile fails validation instead of panicking at render.
    let max = Plot::new().layer(Cells::matrix(1, &[1.0][..]).reduce(crate::stat::Reducer::Max));
    let encoded = serde_json::to_string(&max).expect("serializes");
    assert!(encoded.contains("\"reduce\":\"Max\""), "{encoded}");
    let decoded: Plot = serde_json::from_str(&encoded).expect("deserializes");
    assert!(decoded.validate().is_ok());
    let hostile: Plot = serde_json::from_str(
        r#"{"layers":[{"Cells":{"columns":1,"values":[1.0],"extents":null,"colormap":{"stops":[[0,0,0],[255,255,255]]},"reduce":{"Percentile":7.0}}}],"title":null,"x":"Linear","y":"Linear","x_label":null,"y_label":null,"x_domain":null,"y_domain":null}"#,
    )
    .expect("deserializes");
    assert!(matches!(
        hostile.validate(),
        Err(crate::Error::InvalidParameter { .. })
    ));
}

#[test]
fn a_grid_of_plots_round_trips() {
    let grid = Grid::new(2)
        .with(crate::line(&[1.0, 3.0, 2.0][..]).title("a"))
        .with(crate::line(&[2.0, 1.0, 3.0][..]).title("b"));
    let encoded = serde_json::to_string(&grid).expect("serializes");
    let decoded: Grid = serde_json::from_str(&encoded).expect("deserializes");
    assert_eq!(grid.render(&frame()), decoded.render(&frame()));
}

#[test]
fn color_by_and_the_palette_round_trip_and_stay_out_of_legacy_encodings() {
    use crate::mark::Points;
    use crate::scale::Palette;

    let plot = Plot::new()
        .layer(Points::y(&[1.0, 2.0, 3.0][..]).color_by(["a", "b", "a"]))
        .palette(Palette::new(&[Color::Red, Color::Blue]));
    let encoded = serde_json::to_string(&plot).expect("serializes");
    assert!(encoded.contains("color_by") && encoded.contains("palette"));
    let decoded: Plot = serde_json::from_str(&encoded).expect("deserializes");
    assert_eq!(plot.render(&frame()), decoded.render(&frame()));

    // A plot without the channel encodes exactly as before this field existed,
    // and a ragged deserialized channel is caught at the validation boundary.
    let legacy = Plot::new().layer(Points::y(&[1.0, 2.0][..]));
    let legacy_encoded = serde_json::to_string(&legacy).expect("serializes");
    assert!(!legacy_encoded.contains("color_by") && !legacy_encoded.contains("palette"));

    let ragged: Plot = serde_json::from_str(
        r#"{"layers":[{"Points":{"x":null,"y":[1.0,2.0,3.0],"color":null,"label":null,"style":"Dot","color_by":["a"]}}],"title":null,"x":"Linear","y":"Linear","x_label":null,"y_label":null,"x_domain":null,"y_domain":null}"#,
    )
    .expect("ragged color_by deserializes");
    assert!(matches!(
        ragged.validate(),
        Err(crate::Error::UnequalChannels { .. })
    ));
}

#[test]
fn a_centered_colormap_round_trips_and_legacy_maps_stay_linear() {
    use crate::scale::Colormap;

    let centered = Colormap::RED_BLUE.centered_at(0.0);
    let encoded = serde_json::to_string(&centered).expect("serializes");
    assert!(encoded.contains("midpoint"), "midpoint missing: {encoded}");
    let decoded: Colormap = serde_json::from_str(&encoded).expect("deserializes");
    assert_eq!(decoded, centered);

    // A map without a midpoint serializes exactly as it always has, and the
    // legacy encoding decodes to the linear behavior.
    let linear = serde_json::to_string(&Colormap::GREYS).expect("serializes");
    assert!(!linear.contains("midpoint"), "spurious field: {linear}");
    let legacy: Colormap =
        serde_json::from_str(r#"{"stops":[[0,0,0],[255,255,255]]}"#).expect("deserializes");
    assert_eq!(legacy.midpoint(), None);
    assert_eq!(legacy.position_in(1.0, 0.0, 4.0), 0.25);

    // The log flag rides the same additive contract: present when set, absent
    // otherwise, defaulting to linear in legacy payloads (checked above), and
    // an adversarial centered-and-log payload fails validation, not rendering.
    let log = Colormap::MAGMA.log();
    let encoded = serde_json::to_string(&log).expect("serializes");
    assert!(encoded.contains("\"log\":true"), "log missing: {encoded}");
    let decoded: Colormap = serde_json::from_str(&encoded).expect("deserializes");
    assert_eq!(decoded, log);
    assert!(!linear.contains("log"), "spurious log field: {linear}");
    let contradictory: Colormap =
        serde_json::from_str(r#"{"stops":[[0,0,0],[255,255,255]],"midpoint":0.0,"log":true}"#)
            .expect("deserializes");
    assert!(matches!(
        contradictory.validate(),
        Err(crate::Error::InvalidParameter { .. })
    ));
}

#[test]
fn malformed_payloads_render_without_panicking() {
    // Deserialization can produce states the constructors forbid; rendering must
    // shed them, never panic (COR-04).
    let colormap: crate::scale::Colormap =
        serde_json::from_str(r#"{"stops":[]}"#).expect("empty colormap deserializes");
    assert_eq!(colormap.color(0.5), Color::Default);

    let grid: Grid = serde_json::from_str(
        r#"{"columns":0,"plots":[{"layers":[],"title":null,"x":"Linear","y":"Linear","x_label":null,"y_label":null,"x_domain":null,"y_domain":null}]}"#,
    )
    .expect("zero-column grid deserializes");
    assert!(matches!(
        grid.validate(),
        Err(crate::Error::EmptyDimension { .. })
    ));
    assert!(grid.try_render(&frame()).is_err());
    assert_eq!(grid.render(&frame()), "");

    // A Range with ragged x/low/high/marker channels inside a plot.
    let ragged = r#"{"layers":[{"Range":{"placement":{"Numeric":[0.0,1.0,2.0]},"low":[0.0],"high":[5.0,6.0],"body":null,"marker":[1.0,2.0,3.0,4.0],"color":null,"label":null}}],"title":null,"x":"Linear","y":"Linear","x_label":null,"y_label":null,"x_domain":null,"y_domain":null}"#;
    let plot: Plot = serde_json::from_str(ragged).expect("ragged range deserializes");
    let _ = plot.render(&frame());

    // Large lines take the M4 path, whose tolerance must match the small line
    // renderer instead of indexing the shorter channel.
    let ragged_line: Plot = serde_json::from_value(serde_json::json!({
        "layers": [{
            "Line": {
                "x": [0.0],
                "y": vec![1.0; 1_000],
                "color": null,
                "label": null,
                "style": "Pixels"
            }
        }]
    }))
    .expect("ragged large line deserializes");
    assert!(matches!(
        ragged_line.validate(),
        Err(crate::Error::UnequalChannels { .. })
    ));
    let _ = ragged_line.render(&Frame::plain(20, 8));
}

#[test]
fn validate_rejects_the_malformed_payloads_render_tolerates() {
    // The strict boundary reports what the lenient renderer sheds.
    let ragged: Plot = serde_json::from_str(
        r#"{"layers":[{"Cells":{"columns":0,"values":[1.0,2.0,3.0],"extents":null,"colormap":{"stops":[[0,0,0],[255,255,255]]}}}],"title":null,"x":"Linear","y":"Linear","x_label":null,"y_label":null,"x_domain":null,"y_domain":null}"#,
    )
    .expect("zero-column cells deserializes");
    assert!(matches!(
        ragged.validate(),
        Err(crate::Error::EmptyDimension { .. })
    ));
    assert!(ragged.try_render(&frame()).is_err());
    // Rendering the same spec still does not panic.
    let _ = ragged.render(&frame());

    let degenerate: Plot = serde_json::from_str(
        r#"{"layers":[{"Cells":{"columns":1,"values":[1.0],"extents":[[2.0,2.0],[0.0,1.0]],"colormap":{"stops":[[0,0,0],[255,255,255]]}}}],"title":null,"x":"Linear","y":"Linear","x_label":null,"y_label":null,"x_domain":null,"y_domain":null}"#,
    )
    .expect("degenerate Cells extents deserialize");
    assert!(matches!(
        degenerate.validate(),
        Err(crate::Error::InvalidParameter { .. })
    ));

    // A banded y axis deserializes and validates, but a Cells grid whose row
    // count disagrees with the bands is a conflict, not a silent stretch.
    let categorical_y: Plot = serde_json::from_str(
        r#"{"layers":[{"Cells":{"columns":1,"values":[1.0],"extents":null,"colormap":{"stops":[[0,0,0],[255,255,255]]}}}],"title":null,"x":"Linear","y":{"Bands":["a","b"]},"x_label":null,"y_label":null,"x_domain":null,"y_domain":null}"#,
    )
    .expect("banded y scale deserializes");
    assert!(matches!(
        categorical_y.validate(),
        Err(crate::Error::IncompatibleScale { .. })
    ));
}

#[test]
fn strict_validation_reuses_constructor_invariants() {
    let valid = Plot::new().layer(Bars::spans(0.0, 1.0, [1.0]));
    let mut raw = serde_json::to_value(valid).expect("plot serializes");
    *raw.pointer_mut("/layers/0/Bars/placement/Spans/width")
        .expect("span width in wire representation") = serde_json::json!(0.0);

    let invalid: Plot = serde_json::from_value(raw.clone()).expect("raw Plot stays decodable");
    assert!(matches!(
        invalid.validate(),
        Err(crate::Error::InvalidParameter { .. })
    ));
    let _ = invalid.render(&frame());

    let envelope = serde_json::json!({
        "version": Document::VERSION,
        "kind": "plot",
        "spec": raw
    });
    let error = serde_json::from_value::<Document>(envelope)
        .expect_err("a strict Document rejects invalid Bars geometry");
    assert!(
        error.to_string().contains("finite positive width"),
        "{error}"
    );

    let mut reversed = serde_json::to_value(Plot::new()).unwrap();
    reversed["x_domain"] = serde_json::json!([2.0, 1.0]);
    let reversed: Plot = serde_json::from_value(reversed).unwrap();
    assert!(matches!(
        reversed.validate(),
        Err(crate::Error::InvalidParameter { .. })
    ));

    let mut empty_palette =
        serde_json::to_value(Plot::new().palette(crate::scale::Palette::default())).unwrap();
    empty_palette["palette"]["colors"] = serde_json::json!([]);
    let empty_palette: Plot = serde_json::from_value(empty_palette).unwrap();
    assert!(matches!(
        empty_palette.validate(),
        Err(crate::Error::EmptyDimension { .. })
    ));
}

#[test]
fn a_function_line_refuses_to_serialize() {
    let plot = Plot::new().layer(Line::function(0.0..10.0, f64::sin));
    let error = serde_json::to_string(&plot).expect_err("closures have no data form");
    assert!(
        error.to_string().contains("sample it into points"),
        "{error}"
    );

    let document = Document::plot(Plot::new().layer(Line::function(0.0..10.0, f64::sin))).unwrap();
    let error = serde_json::to_string(&document).expect_err("the envelope cannot encode a closure");
    assert!(error.to_string().contains("sample it into points"));
}

#[test]
fn positioned_and_based_bars_round_trip_and_stay_off_the_old_wire() {
    let stacked = Plot::new()
        .layer(Bars::new(["a", "b"], &[1.0, 2.0][..]))
        .layer(Bars::new(["a", "b"], &[2.0, 1.0][..]).base(&[1.0, 2.0][..]));
    let grouped = Plot::new()
        .x_scale(Scale::bands(["a", "b"]))
        .layer(Bars::at(&[-0.2, 0.8][..], 0.35, &[1.0, 2.0][..]))
        .layer(Bars::at(&[0.2, 1.2][..], 0.35, &[2.0, 1.0][..]));
    for plot in [stacked, grouped] {
        assert!(plot.validate().is_ok());
        let encoded = serde_json::to_string(&plot).expect("serializes");
        let decoded: Plot = serde_json::from_str(&encoded).expect("deserializes");
        assert!(decoded.validate().is_ok());
        assert_eq!(plot.render(&frame()), decoded.render(&frame()));
    }

    // Zero-based band bars encode exactly as before the base channel existed.
    let plain = Plot::new().layer(Bars::new(["a"], &[1.0][..]));
    let encoded = serde_json::to_string(&plain).expect("serializes");
    assert!(!encoded.contains("base"), "spurious field: {encoded}");

    // A ragged decoded base fails validation instead of failing at render.
    let based = Plot::new().layer(Bars::new(["a", "b"], &[1.0, 2.0][..]).base(&[0.0, 1.0][..]));
    let mut raw: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&based).unwrap()).unwrap();
    *raw.pointer_mut("/layers/0/Bars/base")
        .expect("the base field is on the wire") = serde_json::json!([0.0]);
    let decoded: Plot = serde_json::from_value(raw).expect("decodes structurally");
    assert!(
        decoded.validate().is_err(),
        "a ragged base must fail validation"
    );
}
