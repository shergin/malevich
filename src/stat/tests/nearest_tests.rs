use super::nearest;

#[test]
fn finds_the_nearest_finite_value() {
    let values = [10.0, 20.0, 30.0, 40.0];
    assert_eq!(nearest(&values, 24.0), Some(1));
    assert_eq!(nearest(&values, 26.0), Some(2));
    assert_eq!(nearest(&values, -100.0), Some(0));
    assert_eq!(nearest(&values, 100.0), Some(3));
}

#[test]
fn gaps_do_not_participate() {
    let values = [10.0, f64::NAN, 30.0];
    assert_eq!(nearest(&values, 19.0), Some(0));
    assert_eq!(nearest(&values, 21.0), Some(2));
    assert_eq!(nearest(&[f64::NAN, f64::NAN], 1.0), None);
}

#[test]
fn ties_take_the_first_and_edge_cases_are_none() {
    assert_eq!(nearest(&[10.0, 30.0], 20.0), Some(0), "first on ties");
    assert_eq!(nearest(&[], 1.0), None);
    assert_eq!(nearest(&[1.0, 2.0], f64::NAN), None);
    assert_eq!(nearest(&[f64::INFINITY, 5.0], 100.0), Some(1));
}

#[test]
fn values_need_not_be_sorted() {
    let values = [40.0, 10.0, 30.0, 20.0];
    assert_eq!(nearest(&values, 21.0), Some(3));
    assert_eq!(nearest(&values, 34.0), Some(2));
}
