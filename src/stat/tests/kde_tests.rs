use super::kde;

#[test]
fn the_density_integrates_to_about_one() {
    let values: Vec<f64> = (0..2000)
        .map(|i| {
            let i = i as f64;
            ((i * 0.731).sin() + (i * 1.13).sin() + (i * 2.71).sin()) * 2.0
        })
        .collect();
    let (positions, densities) = kde(&values, 512).unwrap();
    let step = positions[1] - positions[0];
    let integral: f64 = densities.iter().sum::<f64>() * step;
    assert!((integral - 1.0).abs() < 0.02, "integral {integral}");
}

#[test]
fn the_mode_sits_near_the_data_center() {
    let values: Vec<f64> = (0..3000)
        .map(|i| {
            let i = i as f64;
            5.0 + ((i * 0.97).sin() + (i * 1.31).sin() + (i * 2.63).sin()) / 3.0
        })
        .collect();
    let (positions, densities) = kde(&values, 256).unwrap();
    let peak = densities
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(b.1))
        .map(|(i, _)| positions[i])
        .unwrap();
    assert!((peak - 5.0).abs() < 0.5, "peak at {peak}");
}

#[test]
fn degenerate_and_empty_samples_behave() {
    assert!(kde(&[f64::NAN], 128).is_none());
    let (_, densities) = kde(&[7.0; 50], 128).unwrap();
    assert!(densities.iter().all(|d| d.is_finite()));
}

#[test]
fn a_degenerate_large_offset_sample_declines_without_panicking() {
    assert!(kde(&[1e20], 16).is_none());
    assert!(kde(&[1e20, 1e20, 1e20], 32).is_none());
}

#[test]
fn caller_selected_grid_is_bounded() {
    assert!(kde(&[1.0, 2.0], usize::MAX).is_none());
}

#[test]
fn gaps_do_not_change_the_finite_sample_density() {
    let finite = [1.0, 2.0, 3.0, 5.0, 8.0];
    let gappy = [
        1.0,
        f64::NAN,
        2.0,
        f64::INFINITY,
        3.0,
        f64::NEG_INFINITY,
        5.0,
        8.0,
    ];
    assert_eq!(kde(&gappy, 64), kde(&finite, 64));
}
