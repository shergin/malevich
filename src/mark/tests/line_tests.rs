use super::{Line, Source};
use crate::render::Color;

const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<Line<'static>>();

#[test]
#[should_panic(expected = "equal length")]
fn paired_series_of_unequal_lengths_panic() {
    Line::xy(&[1.0, 2.0][..], &[1.0][..]);
}

#[test]
#[should_panic(expected = "non-empty domain")]
fn an_empty_function_domain_panics() {
    #[allow(clippy::reversed_empty_ranges)]
    Line::function(5.0..5.0, f64::sin);
}

#[test]
fn explicit_colors_stick() {
    let line = Line::y(&[1.0, 2.0][..]).color(Color::Red);
    assert_eq!(line.color, Some(Color::Red));
}

#[test]
fn into_owned_detaches_from_borrowed_storage() {
    let values = vec![1.0, 2.0];
    let line = Line::y(values.as_slice()).into_owned();
    let Source::Points { y, .. } = &line.source else {
        panic!("expected points");
    };
    assert_ne!(y.as_slice().as_ptr(), values.as_ptr());
    assert_eq!(y.as_slice(), values.as_slice());
}

#[test]
fn debug_stays_curated() {
    let line = Line::y(&[1.0, 2.0, 3.0][..]);
    let debug = format!("{line:?}");
    assert!(debug.contains("points: 3"), "unexpected debug: {debug}");
    assert!(!debug.contains("1.0"), "debug dumps data: {debug}");
}

#[test]
fn glow_is_off_by_default_and_survives_into_owned() {
    let line = Line::y(&[1.0, 2.0][..]);
    assert!(!line.glow);
    let glowing = Line::y(&[1.0, 2.0][..]).glow().into_owned();
    assert!(glowing.glow);
}

#[test]
fn grade_maps_points_through_a_colormap_and_guards_its_channels() {
    use crate::data::IntoSeries as _;
    use crate::scale::Colormap;
    let line = Line::y(&[1.0, 2.0, 3.0][..]).grade(&[0.0, 5.0, 10.0][..], Colormap::VIRIDIS);
    assert!(line.grade.is_some());
    // Length mismatch is rejected.
    let mut bad = Line::y(&[1.0, 2.0][..]);
    bad.grade = Some(((&[1.0][..]).into_series(), Colormap::VIRIDIS));
    assert!(bad.validate().is_err());
    // Grading and categorical color are conflicting channels.
    let mut both = Line::y(&[1.0, 2.0][..]).color_by(["a", "b"]);
    both.grade = Some(((&[1.0, 2.0][..]).into_series(), Colormap::VIRIDIS));
    assert!(both.validate().is_err());
}
