use super::Colormap;
use crate::render::Color;

#[test]
fn endpoints_hit_the_terminal_stops() {
    let map = Colormap::DEFAULT;
    assert_eq!(map.color(0.0), Color::Rgb(68, 1, 84));
    assert_eq!(map.color(1.0), Color::Rgb(253, 231, 37));
}

#[test]
fn out_of_range_and_gap_positions_clamp() {
    let map = Colormap::DEFAULT;
    assert_eq!(map.color(-5.0), map.color(0.0));
    assert_eq!(map.color(5.0), map.color(1.0));
    assert_eq!(map.color(f64::NAN), map.color(0.0));
}

#[test]
fn midpoints_interpolate_between_stops() {
    let map = Colormap::new(&[(0, 0, 0), (100, 200, 50)]);
    assert_eq!(map.color(0.5), Color::Rgb(50, 100, 25));
}

#[test]
fn runtime_stops_move_into_an_owned_colormap() {
    let stops = vec![(3, 5, 8), (13, 21, 34), (55, 89, 144)];
    let allocation = stops.as_ptr();
    let map = Colormap::try_from_stops(stops).unwrap();

    assert_eq!(map.stops(), [(3, 5, 8), (13, 21, 34), (55, 89, 144)]);
    assert_eq!(map.stops().as_ptr(), allocation, "the vector was copied");
    assert_eq!(map.color(1.0), Color::Rgb(55, 89, 144));
}

#[test]
fn a_runtime_colormap_requires_two_stops() {
    for stops in [Vec::new(), vec![(1, 2, 3)]] {
        assert!(matches!(
            Colormap::try_from_stops(stops),
            Err(crate::Error::EmptyDimension {
                what: "Colormap stops"
            })
        ));
    }
}

#[test]
fn every_canonical_name_resolves_and_unknown_names_do_not() {
    for name in Colormap::NAMES {
        let map = Colormap::named(name).expect("canonical name failed to resolve");
        assert!(map.stops().len() >= 2, "{name} has too few stops");
        assert!(map.midpoint().is_none(), "{name} came back pre-anchored");
    }
    assert_eq!(Colormap::named("grays"), Colormap::named("greys"));
    assert_eq!(Colormap::named("jet"), None, "no rainbow maps");
    assert_eq!(Colormap::named("VIRIDIS"), None, "names are lowercase");
}

#[test]
fn the_default_map_is_viridis() {
    assert_eq!(Colormap::DEFAULT, Colormap::VIRIDIS);
    assert_eq!(Colormap::default(), Colormap::VIRIDIS);
}

#[test]
fn a_centered_map_pins_its_midpoint_to_the_ramp_middle() {
    let map = Colormap::RED_BLUE.centered_at(0.0);
    // Data on [-1, 0.5]: the larger side (1.0) sets the symmetric span [-1, 1].
    assert_eq!(map.position_in(0.0, -1.0, 0.5), 0.5);
    assert_eq!(map.position_in(-1.0, -1.0, 0.5), 0.0);
    assert_eq!(map.position_in(0.5, -1.0, 0.5), 0.75);
    // Equal magnitudes on either side get equal distances from the middle.
    let below = map.position_in(-0.3, -1.0, 0.5);
    let above = map.position_in(0.3, -1.0, 0.5);
    assert!((0.5 - below - (above - 0.5)).abs() < 1e-12);
}

#[test]
fn a_linear_map_spans_the_observed_range() {
    let map = Colormap::VIRIDIS;
    assert_eq!(map.position_in(2.0, 2.0, 6.0), 0.0);
    assert_eq!(map.position_in(6.0, 2.0, 6.0), 1.0);
    assert_eq!(map.position_in(4.0, 2.0, 6.0), 0.5);
    // Out-of-range and NaN degrade exactly like Colormap::color.
    assert_eq!(map.position_in(9.0, 2.0, 6.0), 1.0);
    assert_eq!(map.position_in(f64::NAN, 2.0, 6.0), 0.0);

    assert_eq!(map.position_in(-f64::MAX, -f64::MAX, f64::MAX), 0.0);
    assert_eq!(map.position_in(0.0, -f64::MAX, f64::MAX), 0.5);
    assert_eq!(map.position_in(f64::MAX, -f64::MAX, f64::MAX), 1.0);
}

#[test]
fn a_degenerate_range_centers_on_the_midpoint() {
    let map = Colormap::RED_BLUE.centered_at(1.0);
    assert_eq!(map.position_in(1.0, 1.0, 1.0), 0.5);
}

#[test]
#[should_panic(expected = "finite midpoint")]
fn centering_on_a_non_finite_value_is_misuse() {
    let _ = Colormap::RED_BLUE.centered_at(f64::NAN);
}

#[test]
fn a_deserialized_non_finite_midpoint_degrades_and_fails_validation() {
    // Unreachable through the constructors; only deserialization can build it.
    let map = Colormap {
        stops: Colormap::RED_BLUE.stops().to_vec().into(),
        midpoint: Some(super::Midpoint(f64::NAN)),
    };
    assert!(!map.midpoint_is_valid());
    // Rendering paths degrade to the linear mapping instead of spreading NaN.
    assert_eq!(map.position_in(3.0, 2.0, 6.0), 0.25);
}

/// The curation criterion: every named map must stay distinguishable after the
/// honest quantizers, not just in truecolor. (The plain tier reads the ramp
/// position, not the color, so it is monotonic by construction.)
#[test]
fn named_maps_survive_the_color_ladder_distinguishably() {
    use crate::render::color::{rgb_to_16, rgb_to_256};

    let rgb = |color: Color| match color {
        Color::Rgb(r, g, b) => (r, g, b),
        other => panic!("named maps interpolate to concrete RGB, got {other:?}"),
    };
    for name in Colormap::NAMES {
        let map = Colormap::named(name).unwrap();
        let low = rgb(map.color(0.0));
        let mid = rgb(map.color(0.5));
        let high = rgb(map.color(1.0));
        for (quantize, tier) in [
            (rgb_to_256 as fn(u8, u8, u8) -> u8, "256"),
            (rgb_to_16, "16"),
        ] {
            let low = quantize(low.0, low.1, low.2);
            let high = quantize(high.0, high.1, high.2);
            let mid = quantize(mid.0, mid.1, mid.2);
            assert_ne!(low, high, "{name}: ends collapse at {tier} colors");
            // Diverging maps must also keep both ends apart from the neutral
            // middle, or the sign of the data disappears.
            if map == Colormap::RED_BLUE || map == Colormap::PURPLE_ORANGE {
                assert_ne!(
                    low, mid,
                    "{name}: low end collapses into the middle at {tier}"
                );
                assert_ne!(
                    high, mid,
                    "{name}: high end collapses into the middle at {tier}"
                );
            }
        }
    }
}
