use super::Bins;

#[test]
fn values_land_in_their_bins_and_the_last_edge_is_inclusive() {
    let mut bins = Bins::new(0.0, 1.0, 4);
    for value in [0.0, 0.5, 1.0, 2.5, 3.9, 4.0] {
        bins.add(value);
    }
    assert_eq!(bins.counts(), [2, 1, 1, 2]);
}

#[test]
fn out_of_range_and_gap_values_are_ignored() {
    let mut bins = Bins::new(0.0, 1.0, 2);
    for value in [-0.1, 2.1, f64::NAN, 0.5] {
        bins.add(value);
    }
    assert_eq!(bins.counts(), [1, 0]);
}

#[test]
fn merged_chunks_equal_one_sequential_pass() {
    let values: Vec<f64> = (0..5_000).map(|i| ((i * 37) % 100) as f64 / 10.0).collect();
    let mut sequential = Bins::new(0.0, 1.0, 10);
    for &value in &values {
        sequential.add(value);
    }
    let mut merged = Bins::new(0.0, 1.0, 10);
    for chunk in values.chunks(613) {
        let mut partial = Bins::new(0.0, 1.0, 10);
        for &value in chunk {
            partial.add(value);
        }
        merged.merge(&partial);
    }
    assert_eq!(sequential, merged);
}

#[test]
fn auto_bins_cover_the_data_with_nice_edges() {
    let values: Vec<f64> = (0..1_000)
        .map(|i| ((i * 61) % 997) as f64 / 100.0)
        .collect();
    let bins = Bins::auto(&values, 60).unwrap();
    assert!(bins.start() <= 0.0);
    assert!(bins.end() >= 9.96);
    assert_eq!(bins.counts().iter().sum::<u64>(), 1_000);
    // Nice-decimal width: a short exact decimal.
    let width = format!("{}", bins.width());
    assert!(width.len() <= 5, "width {width} is not a nice decimal");
}

#[test]
fn constant_data_gets_one_bin() {
    let bins = Bins::auto(&[7.0; 42], 60).unwrap();
    assert_eq!(bins.counts(), [42]);

    let extreme = Bins::auto(&[f64::MAX; 2], 60).unwrap();
    assert_eq!(extreme.counts(), [2]);
    assert!(extreme.start().is_finite() && extreme.width().is_finite());
    assert!(extreme.end().is_finite() && extreme.start() < extreme.end());
}

#[test]
fn no_finite_data_means_no_bins() {
    assert!(Bins::auto(&[f64::NAN], 60).is_none());
    assert!(Bins::auto(&[], 60).is_none());
}

#[test]
fn auto_never_drops_finite_values_and_respects_the_cap() {
    for offset in [0.0, 1e6, 1e12] {
        for span in [1e-3, 1.0, 1e6] {
            let values: Vec<f64> = (0..101).map(|i| offset + span * i as f64 / 100.0).collect();
            for limit in [1usize, 2, 3, 7, 60] {
                let bins = super::Bins::auto(&values, limit).expect("finite data bins");
                let sum: u64 = bins.counts().iter().sum();
                assert_eq!(
                    sum, 101,
                    "offset {offset} span {span} limit {limit} dropped data"
                );
                assert!(bins.counts().len() <= limit.max(1), "exceeded the cap");
            }
        }
    }
}

#[test]
fn auto_bins_cover_opposite_finite_extremes_without_panicking() {
    let bins = Bins::try_auto(&[-f64::MAX, f64::MAX], 60).unwrap().unwrap();
    assert_eq!(bins.counts().iter().sum::<u64>(), 2);
    assert!(bins.start().is_finite());
    assert!(bins.width().is_finite() && bins.width() > 0.0);
    assert!(bins.end().is_finite() && bins.end() >= f64::MAX);

    assert!(matches!(
        Bins::try_auto(&[-f64::MAX, f64::MAX], 1),
        Err(crate::Error::InvalidParameter { .. })
    ));
}

#[test]
fn bins2_of_constant_data_keeps_a_drawable_extent() {
    let grid = super::bins2(&[3.0, 3.0, 3.0], &[7.0, 7.0, 7.0], 8, 8).expect("finite pairs");
    assert!(grid.x.0 < grid.x.1, "x extent must be drawable");
    assert!(grid.y.0 < grid.y.1, "y extent must be drawable");
    assert_eq!(grid.counts.iter().sum::<f64>(), 3.0);
}

#[test]
fn bins2_distinguishes_opposite_finite_extremes() {
    let grid = super::try_bins2(&[-f64::MAX, f64::MAX], &[0.0, 0.0], 2, 1)
        .unwrap()
        .unwrap();
    assert_eq!(grid.counts, [1.0, 1.0]);
    assert_eq!(grid.x, (-f64::MAX, f64::MAX));
}

#[test]
fn caller_selected_histogram_geometry_is_bounded() {
    assert!(matches!(
        Bins::try_new(0.0, 1.0, usize::MAX),
        Err(crate::Error::DimensionTooLarge { .. })
    ));
    assert!(matches!(
        super::try_bins2(&[1.0], &[1.0], usize::MAX, 2),
        Err(crate::Error::DimensionTooLarge { .. })
    ));
}
