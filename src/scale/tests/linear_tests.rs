use super::Linear;

#[test]
fn map_and_unmap_are_inverses_in_both_range_directions() {
    for range in [(20.0, 80.0), (80.0, 20.0)] {
        let scale = Linear::new((-5.0, 15.0), range);
        for value in [-5.0, 0.0, 7.5, 15.0] {
            let round_trip = scale.unmap(scale.map(value));
            assert!((round_trip - value).abs() < 1e-12);
        }
    }
}

#[test]
fn degenerate_maps_are_stable_and_preserve_gaps() {
    let domain = Linear::new((4.0, 4.0), (0.0, 10.0));
    assert_eq!(domain.map(100.0), 5.0);
    assert!(domain.map(f64::NAN).is_nan());

    let range = Linear::new((2.0, 8.0), (5.0, 5.0));
    assert_eq!(range.unmap(100.0), 5.0);
    assert!(range.unmap(f64::NAN).is_nan());
}

#[test]
fn extreme_finite_endpoints_map_without_non_finite_intermediates() {
    let scale = Linear::new((-f64::MAX, f64::MAX), (0.0, 1.0));
    assert!(scale.finite_affine().is_none());
    assert_eq!(scale.map(-f64::MAX), 0.0);
    assert_eq!(scale.map(0.0), 0.5);
    assert_eq!(scale.map(f64::MAX), 1.0);
    assert_eq!(scale.unmap(0.0), -f64::MAX);
    assert_eq!(scale.unmap(0.5), 0.0);
    assert_eq!(scale.unmap(1.0), f64::MAX);

    let degenerate = Linear::new((1.0, 1.0), (-f64::MAX, f64::MAX));
    assert_eq!(degenerate.map(1.0), 0.0);
}

#[test]
fn ordinary_scales_expose_their_prechecked_affine_map() {
    assert_eq!(
        Linear::new((-5.0, 15.0), (20.0, 80.0)).finite_affine(),
        Some((-5.0, 20.0, 20.0, 60.0))
    );
    assert!(
        Linear::new((1.0, 1.0), (0.0, 10.0))
            .finite_affine()
            .is_none()
    );
}
