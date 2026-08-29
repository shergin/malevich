use super::Bars;
use crate::render::Color;

const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<Bars<'static>>();

#[test]
#[should_panic(expected = "one category per value")]
fn mismatched_categories_and_values_panic() {
    Bars::new(["a", "b"], &[1.0][..]);
}

#[test]
#[should_panic(expected = "one position per value")]
fn mismatched_positions_and_values_panic() {
    Bars::at(&[0.0][..], 0.5, &[1.0, 2.0][..]);
}

#[test]
#[should_panic(expected = "finite positive width")]
fn a_zero_position_width_panics() {
    Bars::at(&[0.0][..], 0.0, &[1.0][..]);
}

#[test]
#[should_panic(expected = "finite positive width")]
fn a_non_finite_position_width_panics() {
    Bars::at(&[0.0][..], f64::NAN, &[1.0][..]);
}

#[test]
#[should_panic(expected = "one base per value")]
fn a_mismatched_base_panics() {
    let _ = Bars::new(["a", "b"], &[1.0, 2.0][..]).base(&[1.0][..]);
}

#[test]
fn a_gap_position_is_valid_and_disclosed_nowhere_else() {
    // NaN positions are gaps that skip their bar at draw time, not errors.
    let bars = Bars::at(&[0.0, f64::NAN, 2.0][..], 0.5, &[1.0, 2.0, 3.0][..]);
    assert!(bars.validate().is_ok());
}

#[test]
fn explicit_colors_stick() {
    let bars = Bars::new(["a"], &[1.0][..]).color(Color::Blue);
    assert_eq!(bars.color, Some(Color::Blue));
}

#[test]
fn debug_stays_curated() {
    let bars = Bars::new(["a", "b", "c"], &[1.0, 2.0, 3.0][..]);
    let debug = format!("{bars:?}");
    assert!(debug.contains("bars: 3"), "unexpected debug: {debug}");
    assert!(debug.contains("based: false"), "unexpected debug: {debug}");
    assert!(!debug.contains("1.0"), "debug dumps data: {debug}");

    let based = Bars::new(["a"], &[1.0][..]).base(&[2.0][..]);
    assert!(format!("{based:?}").contains("based: true"));
}
