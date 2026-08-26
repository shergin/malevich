use super::Band;

#[test]
fn bands_partition_the_range_evenly() {
    let band = Band::new(4, (0.0, 100.0));
    assert_eq!(band.count(), 4);
    let step = band.step();
    for index in 0..3 {
        let gap = band.position(index + 1) - band.position(index);
        assert!((gap - step).abs() < 1e-9);
    }
    assert!(band.bandwidth() < step);
    assert!(band.position(0) > 0.0);
    assert!(band.position(3) + band.bandwidth() < 100.0);
}

#[test]
fn centers_sit_inside_their_bands() {
    let band = Band::new(3, (0.0, 30.0));
    for index in 0..3 {
        let center = band.center(index);
        assert!(center > band.position(index));
        assert!(center < band.position(index) + band.bandwidth());
    }
}

#[test]
fn a_single_band_fills_most_of_the_range() {
    let band = Band::new(1, (0.0, 10.0));
    assert!(band.bandwidth() > 5.0);
    assert!(band.bandwidth() < 10.0);
}

#[test]
fn zero_bands_do_not_divide_by_zero() {
    let band = Band::new(0, (0.0, 10.0));
    assert_eq!(band.bandwidth(), 0.0);
    assert_eq!(band.count(), 0);
}

#[test]
fn index_at_finds_bands_and_skips_padding() {
    let band = Band::new(3, (0.0, 30.0));
    for index in 0..3 {
        assert_eq!(band.index_at(band.center(index)), Some(index));
        assert_eq!(band.index_at(band.position(index)), Some(index));
    }
    // The padding between adjacent bands, and the margins outside the run,
    // belong to no band.
    assert_eq!(band.index_at(band.position(1) - band.step() * 0.1), None);
    assert_eq!(band.index_at(band.position(0) - 0.5), None);
    assert_eq!(
        band.index_at(band.position(2) + band.bandwidth() + 0.1),
        None
    );
    assert_eq!(band.index_at(-5.0), None);
    assert_eq!(band.index_at(35.0), None);
}

#[test]
fn index_at_survives_degenerate_bands() {
    assert_eq!(Band::new(0, (0.0, 10.0)).index_at(5.0), None);
    assert_eq!(Band::new(2, (5.0, 5.0)).index_at(5.0), None);
}
