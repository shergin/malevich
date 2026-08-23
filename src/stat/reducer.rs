//! The shared reducer vocabulary: one set of names for every aggregating stat.

/// A named aggregation — how a set of values collapses to one number. The one
/// vocabulary shared by [`Agg`](super::Agg), [`Window`](super::Window), and
/// binned reduction ([`binned`](super::binned)), following the Observable Plot
/// convention.
///
/// Non-finite values are excluded before reducing (the gap convention). An
/// empty set reduces to `0` for [`Count`](Reducer::Count) and
/// [`Sum`](Reducer::Sum) — real answers — and to a gap (`NaN`) for everything
/// else.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum Reducer {
    /// The number of finite values.
    Count,
    /// Their sum.
    Sum,
    /// Their mean.
    Mean,
    /// Their median — [`Percentile`](Reducer::Percentile) at 0.5.
    Median,
    /// The smallest.
    Min,
    /// The largest.
    Max,
    /// The type-7 quantile at a position in `[0, 1]` — the same estimator the
    /// box plot's quartiles use (the R default).
    Percentile(f64),
}

impl Reducer {
    /// Reduces `values`, excluding non-finite members.
    ///
    /// # Panics
    ///
    /// Panics when a [`Percentile`](Reducer::Percentile) position is not in
    /// `[0, 1]`.
    pub fn reduce(&self, values: &[f64]) -> f64 {
        if let Reducer::Percentile(position) = self {
            assert!(
                (0.0..=1.0).contains(position),
                "Reducer::Percentile requires a position in [0, 1]"
            );
        }
        let mut finite: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
        match self {
            Reducer::Count => finite.len() as f64,
            Reducer::Sum => finite.iter().sum(),
            Reducer::Mean if finite.is_empty() => f64::NAN,
            Reducer::Mean => finite.iter().sum::<f64>() / finite.len() as f64,
            Reducer::Min => finite.iter().copied().fold(f64::NAN, f64::min),
            Reducer::Max => finite.iter().copied().fold(f64::NAN, f64::max),
            Reducer::Median | Reducer::Percentile(_) if finite.is_empty() => f64::NAN,
            Reducer::Median => {
                finite.sort_by(f64::total_cmp);
                quantile_sorted(&finite, 0.5)
            }
            Reducer::Percentile(position) => {
                finite.sort_by(f64::total_cmp);
                quantile_sorted(&finite, *position)
            }
        }
    }
}

/// The type-7 quantiles of one sample at several positions, sorting once —
/// the efficient shape for Q–Q plots and multi-quantile summaries. Non-finite
/// values are excluded; an empty sample yields gaps.
///
/// # Panics
///
/// Panics when a position is outside `[0, 1]`.
pub fn quantiles(values: &[f64], positions: &[f64]) -> Vec<f64> {
    assert!(
        positions.iter().all(|p| (0.0..=1.0).contains(p)),
        "quantiles requires positions in [0, 1]"
    );
    let mut finite: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if finite.is_empty() {
        return vec![f64::NAN; positions.len()];
    }
    finite.sort_by(f64::total_cmp);
    positions
        .iter()
        .map(|&position| quantile_sorted(&finite, position))
        .collect()
}

/// The type-7 quantile of an ascending-sorted, non-empty slice (the R
/// default: linear interpolation of the order statistics).
pub(crate) fn quantile_sorted(sorted: &[f64], p: f64) -> f64 {
    let position = (sorted.len() - 1) as f64 * p;
    let index = position.floor() as usize;
    let fraction = position - index as f64;
    if index + 1 < sorted.len() {
        crate::numeric::lerp(sorted[index], sorted[index + 1], fraction)
    } else {
        sorted[index]
    }
}

#[cfg(test)]
#[path = "tests/reducer_tests.rs"]
mod tests;
