use super::Window;
use crate::stat::Reducer;

#[test]
fn trailing_means_smooth_without_a_warmup_gap() {
    let smoothed = Window::new(3).mean(&[3.0, 6.0, 9.0, 12.0]);
    assert_eq!(smoothed, [3.0, 4.5, 6.0, 9.0]);
}

#[test]
fn gaps_are_excluded_and_all_gap_windows_stay_gaps() {
    let smoothed = Window::new(2).mean(&[1.0, f64::NAN, 5.0]);
    assert_eq!(smoothed[0], 1.0);
    assert_eq!(smoothed[1], 1.0);
    assert_eq!(smoothed[2], 5.0);
    let gaps = Window::new(1).mean(&[f64::NAN]);
    assert!(gaps[0].is_nan());
}

#[test]
fn the_other_reducers_reduce() {
    let window = Window::new(2);
    assert_eq!(window.sum(&[1.0, 2.0, 3.0]), [1.0, 3.0, 5.0]);
    assert_eq!(window.min(&[3.0, 1.0, 2.0]), [3.0, 1.0, 1.0]);
    assert_eq!(window.max(&[1.0, 3.0, 2.0]), [1.0, 3.0, 3.0]);
}

#[test]
fn optimized_rolling_strategies_match_one_shot_reduction() {
    let values: Vec<f64> = (0..257)
        .map(|index| match index % 29 {
            0 => f64::NAN,
            1 => f64::INFINITY,
            _ => ((index * 37) % 101) as f64 - 50.0,
        })
        .collect();
    let reducers = [
        Reducer::Count,
        Reducer::Sum,
        Reducer::Mean,
        Reducer::Min,
        Reducer::Max,
        Reducer::Median,
        Reducer::Percentile(0.9),
    ];

    for size in [1, 2, 7, 64, 999] {
        let window = Window::new(size);
        for reducer in reducers {
            let actual = window.reduce(&values, reducer);
            let expected: Vec<f64> = (0..values.len())
                .map(|end| {
                    let start = (end + 1).saturating_sub(size);
                    reducer.reduce(&values[start..=end])
                })
                .collect();
            for (index, (&actual, &expected)) in actual.iter().zip(&expected).enumerate() {
                assert!(
                    (actual.is_nan() && expected.is_nan())
                        || (actual - expected).abs()
                            <= f64::EPSILON * 32.0 * actual.abs().max(expected.abs()).max(1.0),
                    "size {size}, reducer {reducer:?}, index {index}: {actual:?} != {expected:?}"
                );
            }
        }
    }
}

#[test]
fn an_overflowing_mean_does_not_poison_later_windows() {
    let means = Window::new(2).mean(&[f64::MAX, f64::MAX, -f64::MAX, 1.0]);
    assert_eq!(means[0], f64::MAX);
    assert_eq!(means[1], f64::MAX);
    assert_eq!(means[2], 0.0);
    assert_eq!(means[3], -f64::MAX / 2.0);
}

#[test]
#[should_panic(expected = "Reducer::Percentile requires a position in [0, 1]")]
fn invalid_percentiles_are_rejected_even_for_empty_input() {
    Window::new(3).reduce(&[], Reducer::Percentile(2.0));
}
