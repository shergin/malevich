use super::{M4, m4, m4_mapped, m4_mapped_categories};

fn normalized(series: (Vec<f64>, Vec<f64>)) -> Vec<Option<(u64, u64)>> {
    series
        .0
        .into_iter()
        .zip(series.1)
        .map(|(x, y)| {
            if x.is_nan() || y.is_nan() {
                None
            } else {
                Some((x.to_bits(), y.to_bits()))
            }
        })
        .collect()
}

fn runs(values: &[f64]) -> Vec<Vec<f64>> {
    let mut runs = Vec::new();
    let mut starts_run = true;
    for &value in values {
        if value.is_nan() {
            starts_run = true;
            continue;
        }
        if starts_run {
            runs.push(Vec::new());
            starts_run = false;
        }
        runs.last_mut()
            .expect("a finite value starts a run")
            .push(value);
    }
    runs
}

fn wave(n: usize) -> (Vec<f64>, Vec<f64>) {
    let x: Vec<f64> = (0..n).map(|i| i as f64).collect();
    let y: Vec<f64> = (0..n)
        .map(|i| (i as f64 * 0.37).sin() * (1.0 + i as f64 * 0.001))
        .collect();
    (x, y)
}

#[test]
fn each_column_keeps_its_extremes_and_endpoints() {
    let mut aggregate = M4::new((0.0, 10.0), 1);
    for (x, y) in [(0.0, 5.0), (2.0, -3.0), (5.0, 9.0), (10.0, 1.0)] {
        aggregate.add(x, y);
    }
    let (x, y) = aggregate.emit();
    // first (0,5), min (2,-3), max (5,9), last (10,1) — in x order.
    assert_eq!(x, [0.0, 2.0, 5.0, 10.0]);
    assert_eq!(y, [5.0, -3.0, 9.0, 1.0]);
}

#[test]
fn extreme_domains_normalize_without_overflow() {
    let mut aggregate = M4::new((-f64::MAX, f64::MAX), 2);
    for (x, y) in [(-f64::MAX, -1.0), (0.0, 0.0), (f64::MAX, 1.0)] {
        aggregate.add(x, y);
    }
    let (x, y) = aggregate.emit();
    assert_eq!(x, [-f64::MAX, 0.0, f64::MAX]);
    assert_eq!(y, [-1.0, 0.0, 1.0]);

    let lo = f64::from_bits(1);
    let hi = f64::from_bits(3);
    let mut tiny = M4::new((lo, hi), 2);
    tiny.add(lo, 1.0);
    tiny.add(hi, 3.0);
    assert_eq!(tiny.emit(), (vec![lo, hi], vec![1.0, 3.0]));
}

#[test]
fn caller_selected_column_count_is_bounded() {
    assert!(matches!(
        M4::try_new((0.0, 1.0), usize::MAX),
        Err(crate::Error::DimensionTooLarge { .. })
    ));
}

#[test]
fn merged_chunks_equal_one_sequential_pass() {
    let (x, y) = wave(10_000);
    let mut sequential = M4::new((0.0, 9_999.0), 160);
    for (&xv, &yv) in x.iter().zip(y.iter()) {
        sequential.add(xv, yv);
    }
    let mut merged = M4::new((0.0, 9_999.0), 160);
    for chunk in x.chunks(997).zip(y.chunks(997)).map(|(cx, cy)| {
        let mut partial = M4::new((0.0, 9_999.0), 160);
        for (&xv, &yv) in cx.iter().zip(cy.iter()) {
            partial.add(xv, yv);
        }
        partial
    }) {
        merged.merge(&chunk);
    }
    assert_eq!(sequential.emit(), merged.emit());
}

#[test]
fn gaps_survive_downsampling() {
    let x: Vec<f64> = (0..1000).map(|i| i as f64).collect();
    let mut y: Vec<f64> = (0..1000).map(|i| (i as f64 * 0.1).sin()).collect();
    y[500] = f64::NAN;
    let (_, emitted_y) = m4(&x, &y, 10).unwrap();
    assert!(
        emitted_y.iter().any(|value| value.is_nan()),
        "the gap vanished"
    );
}

#[test]
fn unsorted_x_refuses_to_downsample() {
    let x = [0.0, 5.0, 3.0, 8.0];
    let y = [1.0, 2.0, 3.0, 4.0];
    assert!(m4(&x, &y, 4).is_none());
}

#[test]
fn emitted_points_never_exceed_four_per_column() {
    let (x, y) = wave(50_000);
    let (ex, _) = m4(&x, &y, 100).unwrap();
    assert!(ex.len() <= 400, "emitted {} points", ex.len());
    assert!(ex.windows(2).all(|pair| pair[0] <= pair[1]), "not sorted");
}

#[test]
fn all_gap_series_emit_nothing() {
    assert!(m4(&[f64::NAN, f64::NAN], &[1.0, 2.0], 4).is_none());
}

#[test]
fn a_gap_between_finite_values_in_one_column_does_not_reconnect_them() {
    // Everything lands in a single column; the NaN separates the -1s from the +1s.
    let x = [0.0, 1.0, 2.0, 3.0, 4.0];
    let y = [-1.0, -1.0, f64::NAN, 1.0, 1.0];
    let (_, oy) = m4(&x, &y, 1).unwrap();
    let gap = oy.iter().position(|v| v.is_nan()).expect("gap preserved");
    assert!(
        oy[..gap].iter().all(|&v| v < 0.0),
        "only the low values precede the break"
    );
    assert!(
        oy[gap + 1..].iter().all(|&v| v > 0.0),
        "only the high values follow the break"
    );
}

#[test]
fn several_gaps_in_one_column_keep_every_run_disconnected() {
    let x = [0.0, 1.0, 2.0, 3.0, 4.0];
    let y = [0.0, f64::NAN, 10.0, f64::NAN, 0.0];
    let (_, emitted) = m4(&x, &y, 1).unwrap();
    assert_eq!(runs(&emitted), [vec![0.0], vec![10.0], vec![0.0]]);
}

#[test]
fn merges_preserve_gaps_at_and_inside_partition_boundaries() {
    let x = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
    let y = [1.0, f64::NAN, 2.0, 4.0, f64::NAN, 3.0, f64::NAN, 5.0];
    let mut sequential = M4::new((0.0, 7.0), 2);
    for (&xv, &yv) in x.iter().zip(&y) {
        sequential.add(xv, yv);
    }
    let expected = normalized(sequential.emit());

    for first_split in 0..=x.len() {
        for second_split in first_split..=x.len() {
            let mut first = M4::new((0.0, 7.0), 2);
            let mut second = M4::new((0.0, 7.0), 2);
            let mut third = M4::new((0.0, 7.0), 2);
            for (&xv, &yv) in x[..first_split].iter().zip(&y[..first_split]) {
                first.add(xv, yv);
            }
            for (&xv, &yv) in x[first_split..second_split]
                .iter()
                .zip(&y[first_split..second_split])
            {
                second.add(xv, yv);
            }
            for (&xv, &yv) in x[second_split..].iter().zip(&y[second_split..]) {
                third.add(xv, yv);
            }

            let mut left_associative = first.clone();
            left_associative.merge(&second);
            left_associative.merge(&third);
            assert_eq!(
                normalized(left_associative.emit()),
                expected,
                "partition {first_split}, {second_split}"
            );

            let mut right = second;
            right.merge(&third);
            let mut right_associative = first;
            right_associative.merge(&right);
            assert_eq!(
                normalized(right_associative.emit()),
                expected,
                "right-associated partition {first_split}, {second_split}"
            );
        }
    }
}

#[test]
fn a_leading_gap_in_a_column_breaks_before_its_points() {
    let x = [0.0, 1.0, 2.0];
    let y = [f64::NAN, 3.0, 5.0];
    let (_, oy) = m4(&x, &y, 1).unwrap();
    assert!(oy[0].is_nan(), "the break comes first");
    assert!(oy[1..].iter().all(|v| v.is_finite()));
}

#[test]
fn mapped_reduction_keeps_at_most_four_points_per_target_column() {
    // Map 10_000 indices onto 20 columns; every column keeps <= 4 points, output
    // stays sorted by the mapped position.
    let y: Vec<f64> = (0..10_000).map(|i| (i as f64 * 0.05).sin()).collect();
    let columns = 20;
    let map = |x: f64| x / 10_000.0 * (columns as f64 - 1.0);
    let (rx, _) = m4_mapped(None, &y, columns, map).unwrap();
    assert!(rx.len() <= columns * 4, "kept {} points", rx.len());
    assert!(rx.windows(2).all(|pair| pair[0] <= pair[1]), "not sorted");
}

#[test]
fn mapped_reduction_refuses_non_ascending_x() {
    let x = [0.0, 3.0, 1.0, 9.0];
    let y = [1.0, 2.0, 3.0, 4.0];
    assert!(m4_mapped(Some(&x), &y, 4, |v| v).is_none());
}

#[test]
fn mapped_reduction_preserves_a_gap() {
    // A NaN y between two runs, all inside one column: the break survives.
    let y = [1.0, 1.0, f64::NAN, 5.0, 5.0];
    let (_, ry) = m4_mapped(None, &y, 1, |_| 0.0).unwrap();
    let gap = ry.iter().position(|v| v.is_nan()).expect("gap kept");
    assert!(ry[..gap].iter().all(|&v| v < 3.0));
    assert!(ry[gap + 1..].iter().all(|&v| v > 3.0));
}

#[test]
fn mapped_reduction_tolerates_ragged_explicit_channels() {
    let x = [0.0];
    let y = vec![1.0; 1_000];
    let (rx, ry) = m4_mapped(Some(&x), &y, 4, |_| 0.0).unwrap();
    assert_eq!(rx, [0.0]);
    assert_eq!(ry, [1.0]);
}

#[test]
fn invalid_x_and_mapping_values_break_the_path() {
    let x = [0.0, 1.0, f64::NAN, 3.0, 4.0];
    let y = [-1.0, -1.0, 0.0, 1.0, 1.0];
    let (_, invalid_x) = m4_mapped(Some(&x), &y, 1, |_| 0.0).unwrap();
    let invalid_x_runs = runs(&invalid_x);
    assert_eq!(invalid_x_runs.len(), 2);
    assert!(invalid_x_runs[0].iter().all(|value| *value < 0.0));
    assert!(invalid_x_runs[1].iter().all(|value| *value > 0.0));

    let x = [0.0, 1.0, 2.0, 3.0, 4.0];
    let (_, invalid_map) = m4_mapped(
        Some(&x),
        &y,
        1,
        |value| {
            if value == 2.0 { f64::NAN } else { 0.0 }
        },
    )
    .unwrap();
    let invalid_map_runs = runs(&invalid_map);
    assert_eq!(invalid_map_runs.len(), 2);
    assert!(invalid_map_runs[0].iter().all(|value| *value < 0.0));
    assert!(invalid_map_runs[1].iter().all(|value| *value > 0.0));
}

#[test]
fn category_transitions_are_emitted_as_aligned_path_breaks() {
    let y = [1.0, 2.0, 8.0, 9.0, 3.0, 4.0];
    let categories = [0, 0, 1, 1, 0, 0];
    let (x, reduced, reduced_categories) =
        m4_mapped_categories(None, &y, &categories, 1, |_| 0.0).unwrap();

    assert_eq!(x.len(), reduced.len());
    assert_eq!(reduced.len(), reduced_categories.len());
    assert_eq!(
        runs(&reduced),
        [vec![1.0, 2.0], vec![8.0, 9.0], vec![3.0, 4.0]]
    );

    let identities: Vec<Vec<usize>> =
        reduced
            .iter()
            .zip(reduced_categories)
            .fold(Vec::new(), |mut groups, (value, category)| {
                if value.is_nan() {
                    groups.push(Vec::new());
                } else {
                    if groups.is_empty() {
                        groups.push(Vec::new());
                    }
                    groups.last_mut().unwrap().push(category);
                }
                groups
            });
    assert_eq!(identities, [vec![0, 0], vec![1, 1], vec![0, 0]]);
}
