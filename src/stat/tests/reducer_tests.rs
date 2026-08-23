use super::Reducer;
use super::quantiles;

#[test]
fn every_reducer_answers_the_hand_checked_set() {
    let values = [3.0, 1.0, 4.0, 1.0, 5.0];
    assert_eq!(Reducer::Count.reduce(&values), 5.0);
    assert_eq!(Reducer::Sum.reduce(&values), 14.0);
    assert_eq!(Reducer::Mean.reduce(&values), 2.8);
    assert_eq!(Reducer::Median.reduce(&values), 3.0);
    assert_eq!(Reducer::Min.reduce(&values), 1.0);
    assert_eq!(Reducer::Max.reduce(&values), 5.0);
    // Type-7 at 0.25 over sorted [1,1,3,4,5]: position 1.0 → exactly 1.
    assert_eq!(Reducer::Percentile(0.25).reduce(&values), 1.0);
    // At 0.75: position 3.0 → exactly 4.
    assert_eq!(Reducer::Percentile(0.75).reduce(&values), 4.0);
}

#[test]
fn percentiles_match_the_box_plot_quartiles() {
    let values: Vec<f64> = (1..=11).map(f64::from).collect();
    let stats = crate::stat::BoxStats::of(&values).unwrap();
    assert_eq!(Reducer::Percentile(0.25).reduce(&values), stats.q1);
    assert_eq!(Reducer::Median.reduce(&values), stats.median);
    assert_eq!(Reducer::Percentile(0.75).reduce(&values), stats.q3);
}

#[test]
fn median_of_opposite_finite_extremes_stays_finite() {
    assert_eq!(Reducer::Median.reduce(&[-f64::MAX, f64::MAX]), 0.0);
}

#[test]
fn mean_of_opposite_finite_extremes_stays_inside_their_hull() {
    assert_eq!(Reducer::Mean.reduce(&[-f64::MAX, f64::MAX]), 0.0);
    assert_eq!(Reducer::Mean.reduce(&[f64::MAX, -f64::MAX]), 0.0);
}

#[test]
fn gaps_are_excluded_and_the_empty_set_answers_honestly() {
    let gappy = [1.0, f64::NAN, 3.0, f64::INFINITY];
    assert_eq!(Reducer::Count.reduce(&gappy), 2.0);
    assert_eq!(Reducer::Mean.reduce(&gappy), 2.0);

    assert_eq!(Reducer::Count.reduce(&[]), 0.0);
    assert_eq!(Reducer::Sum.reduce(&[]), 0.0);
    assert!(Reducer::Mean.reduce(&[]).is_nan());
    assert!(Reducer::Median.reduce(&[]).is_nan());
    assert!(Reducer::Min.reduce(&[]).is_nan());
    assert!(Reducer::Max.reduce(&[]).is_nan());
    assert!(Reducer::Percentile(0.9).reduce(&[]).is_nan());
}

#[test]
#[should_panic(expected = "position in [0, 1]")]
fn a_percentile_outside_the_unit_interval_is_misuse() {
    let _ = Reducer::Percentile(1.5).reduce(&[1.0]);
}

#[test]
fn multi_quantiles_sort_once_and_agree_with_the_reducer() {
    let values = [9.0, 2.0, 7.0, 4.0, 6.0, 1.0, 8.0];
    let positions = [0.0, 0.25, 0.5, 0.9, 1.0];
    let batch = quantiles(&values, &positions);
    for (&position, &result) in positions.iter().zip(&batch) {
        assert_eq!(result, Reducer::Percentile(position).reduce(&values));
    }
    assert!(quantiles(&[], &positions).iter().all(|q| q.is_nan()));
}

#[test]
fn the_unified_vocabulary_reaches_windows_and_groups() {
    use crate::stat::{Agg, Window};

    // A rolling p100 is the rolling max — two doors, one vocabulary.
    let values = [1.0, 5.0, 2.0, 8.0, 3.0];
    let window = Window::new(3);
    assert_eq!(
        window.reduce(&values, Reducer::Percentile(1.0)),
        window.max(&values)
    );
    assert_eq!(window.median(&values), [1.0, 3.0, 2.0, 5.0, 3.0]);

    let (keys, p95) =
        Agg::by(["a", "a", "b"], &[1.0, 3.0, 10.0][..]).reduce(Reducer::Percentile(0.5));
    assert_eq!(keys, ["a", "b"]);
    assert_eq!(p95, [2.0, 10.0]);
}
