use super::{auc, roc};

#[test]
fn the_step_construction_matches_the_hand_computed_curve() {
    let scores = [0.9, 0.8, 0.7, 0.6];
    let labels = [true, false, true, false];
    let (fpr, tpr) = roc(&scores, &labels);
    assert_eq!(fpr, [0.0, 0.0, 0.5, 0.5, 1.0]);
    assert_eq!(tpr, [0.0, 0.5, 0.5, 1.0, 1.0]);
    assert!((auc(&fpr, &tpr) - 0.75).abs() < 1e-12);
}

#[test]
fn perfect_and_random_rankings_bound_the_area() {
    let (fpr, tpr) = roc(&[0.9, 0.8, 0.2, 0.1], &[true, true, false, false]);
    assert!((auc(&fpr, &tpr) - 1.0).abs() < 1e-12);

    // All scores tied: one grouped step straight up the diagonal.
    let (fpr, tpr) = roc(&[0.5, 0.5, 0.5, 0.5], &[true, false, true, false]);
    assert_eq!(fpr, [0.0, 1.0]);
    assert_eq!(tpr, [0.0, 1.0]);
    assert!((auc(&fpr, &tpr) - 0.5).abs() < 1e-12);
}

#[test]
fn missing_classes_and_gaps_stay_honest() {
    let (fpr, tpr) = roc(&[0.9, 0.8], &[true, true]);
    assert!(
        fpr.is_empty() && tpr.is_empty(),
        "one-class data has no curve"
    );

    // A non-finite score drops with its label rather than skewing a rate.
    let (fpr, tpr) = roc(&[0.9, f64::NAN, 0.1], &[true, true, false]);
    assert_eq!(fpr, [0.0, 0.0, 1.0]);
    assert_eq!(tpr, [0.0, 1.0, 1.0]);

    // A gap inside a polyline contributes no area; no segment at all is NaN.
    assert!((auc(&[0.0, 0.5, f64::NAN, 1.0], &[0.0, 1.0, 1.0, 1.0]) - 0.25).abs() < 1e-12);
    assert!(auc(&[0.5], &[1.0]).is_nan());
    assert!(auc(&[f64::NAN, 1.0], &[0.0, f64::NAN]).is_nan());
}

#[test]
#[should_panic(expected = "equal length")]
fn ragged_channels_panic() {
    roc(&[0.5], &[true, false]);
}
