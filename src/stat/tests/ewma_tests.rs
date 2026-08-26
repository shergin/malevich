use super::ewma;

#[test]
fn matches_the_hand_computed_debiased_average() {
    let smoothed = ewma(&[1.0, 2.0], 0.5);
    assert!(
        (smoothed[0] - 1.0).abs() < 1e-12,
        "debiasing keeps the first value"
    );
    // state = 0.5 * 0.5 + 0.5 * 2 = 1.25, debiased by 1 - 0.25.
    assert!((smoothed[1] - 1.25 / 0.75).abs() < 1e-12);
}

#[test]
fn identity_constants_and_gaps() {
    assert_eq!(ewma(&[3.0, 1.0, 4.0], 0.0), [3.0, 1.0, 4.0]);
    let flat = ewma(&[7.0; 50], 0.97);
    assert!(flat.iter().all(|&value| (value - 7.0).abs() < 1e-9));

    // A gap stays a gap and does not disturb the state around it.
    let with_gap = ewma(&[1.0, f64::NAN, 1.0], 0.5);
    assert!((with_gap[0] - 1.0).abs() < 1e-12);
    assert!(with_gap[1].is_nan());
    assert!((with_gap[2] - 1.0).abs() < 1e-12);
}

#[test]
fn heavy_smoothing_lags_a_step_change() {
    let mut values = vec![0.0; 20];
    values.extend(vec![1.0; 20]);
    let smoothed = ewma(&values, 0.9);
    assert!(
        smoothed[20] < 0.2,
        "the first post-step output barely moves"
    );
    assert!(smoothed[39] > 0.7, "the average closes on the new level");
}

#[test]
#[should_panic(expected = "smoothing factor")]
fn an_out_of_range_factor_is_misuse() {
    ewma(&[1.0], 1.0);
}
