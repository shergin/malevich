//! Rolling windows: trailing reduces with the shared reducer vocabulary.

use std::collections::VecDeque;

/// A trailing window of `size` values, reduced at every position.
///
/// The first positions reduce partial windows (no warm-up gap in the chart), and
/// gaps (`NaN`) are excluded from each window's reduction. Empty finite windows
/// follow the reducer's policy: zero for count/sum and a gap otherwise. The named
/// methods are sugar over [`reduce`](Window::reduce) with the crate's one
/// [`Reducer`](super::Reducer) vocabulary, shared with [`super::Agg`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Window {
    size: usize,
}

impl Window {
    /// A window of `size` trailing values.
    ///
    /// # Panics
    ///
    /// Panics if `size` is zero.
    pub fn new(size: usize) -> Window {
        assert!(size > 0, "Window::new requires a non-zero size");
        Window { size }
    }

    /// Applies any named [`Reducer`](super::Reducer) over each trailing
    /// window — rolling medians and percentiles included:
    /// `window.reduce(&latencies, Reducer::Percentile(0.95))`.
    pub fn reduce(&self, values: &[f64], reducer: super::Reducer) -> Vec<f64> {
        match reducer {
            super::Reducer::Count => self.rolling_count(values),
            super::Reducer::Sum => self.rolling_additive(values, false),
            super::Reducer::Mean => self.rolling_additive(values, true),
            super::Reducer::Min => self.rolling_extreme(values, true),
            super::Reducer::Max => self.rolling_extreme(values, false),
            super::Reducer::Median => self.rolling_quantile(values, 0.5),
            super::Reducer::Percentile(position) => {
                assert!(
                    (0.0..=1.0).contains(&position),
                    "Reducer::Percentile requires a position in [0, 1]"
                );
                self.rolling_quantile(values, position)
            }
        }
    }

    /// The rolling mean.
    pub fn mean(&self, values: &[f64]) -> Vec<f64> {
        self.reduce(values, super::Reducer::Mean)
    }

    /// The rolling sum (0 when nothing is finite).
    pub fn sum(&self, values: &[f64]) -> Vec<f64> {
        self.reduce(values, super::Reducer::Sum)
    }

    /// The rolling median.
    pub fn median(&self, values: &[f64]) -> Vec<f64> {
        self.reduce(values, super::Reducer::Median)
    }

    /// The rolling minimum.
    pub fn min(&self, values: &[f64]) -> Vec<f64> {
        self.reduce(values, super::Reducer::Min)
    }

    /// The rolling maximum.
    pub fn max(&self, values: &[f64]) -> Vec<f64> {
        self.reduce(values, super::Reducer::Max)
    }

    fn rolling_count(&self, values: &[f64]) -> Vec<f64> {
        let mut count = 0usize;
        let mut reduced = Vec::with_capacity(values.len());
        for (index, &value) in values.iter().enumerate() {
            if index >= self.size && values[index - self.size].is_finite() {
                count -= 1;
            }
            if value.is_finite() {
                count += 1;
            }
            reduced.push(count as f64);
        }
        reduced
    }

    /// Rolling sum/mean share the same add/remove state. If a finite window's sum
    /// overflows, recompute that exceptional window so an infinity does not poison
    /// every later result; means use their robust one-shot reducer in that case.
    fn rolling_additive(&self, values: &[f64], mean: bool) -> Vec<f64> {
        let mut count = 0usize;
        let mut sum = 0.0;
        let mut reduced = Vec::with_capacity(values.len());
        for (end, &value) in values.iter().enumerate() {
            if end >= self.size {
                let outgoing = values[end - self.size];
                if outgoing.is_finite() {
                    count -= 1;
                    sum -= outgoing;
                }
            }
            if value.is_finite() {
                count += 1;
                sum += value;
            }

            let start = (end + 1).saturating_sub(self.size);
            if !sum.is_finite() {
                sum = values[start..=end]
                    .iter()
                    .copied()
                    .filter(|value| value.is_finite())
                    .sum();
            }
            reduced.push(if !mean {
                sum
            } else if count == 0 {
                f64::NAN
            } else if sum.is_finite() {
                sum / count as f64
            } else {
                super::Reducer::Mean.reduce(&values[start..=end])
            });
        }
        reduced
    }

    /// A monotonic deque keeps only candidates that can become the window's
    /// extremum. Every finite value enters and leaves at most once.
    fn rolling_extreme(&self, values: &[f64], minimum: bool) -> Vec<f64> {
        let mut candidates = VecDeque::<usize>::with_capacity(self.size.min(values.len()));
        let mut reduced = Vec::with_capacity(values.len());
        for (index, &value) in values.iter().enumerate() {
            let start = (index + 1).saturating_sub(self.size);
            while candidates
                .front()
                .is_some_and(|&candidate| candidate < start)
            {
                candidates.pop_front();
            }
            if value.is_finite() {
                while candidates.back().is_some_and(|&candidate| {
                    if minimum {
                        values[candidate] >= value
                    } else {
                        values[candidate] <= value
                    }
                }) {
                    candidates.pop_back();
                }
                candidates.push_back(index);
            }
            reduced.push(
                candidates
                    .front()
                    .map_or(f64::NAN, |&candidate| values[candidate]),
            );
        }
        reduced
    }

    /// Order statistics are inherently buffered here. Reusing one sample buffer
    /// makes that cost explicit and avoids one allocation per output position.
    fn rolling_quantile(&self, values: &[f64], position: f64) -> Vec<f64> {
        let mut sample = Vec::with_capacity(self.size.min(values.len()));
        let mut reduced = Vec::with_capacity(values.len());
        for end in 0..values.len() {
            let start = (end + 1).saturating_sub(self.size);
            sample.clear();
            sample.extend(
                values[start..=end]
                    .iter()
                    .copied()
                    .filter(|value| value.is_finite()),
            );
            if sample.is_empty() {
                reduced.push(f64::NAN);
            } else {
                sample.sort_by(f64::total_cmp);
                reduced.push(super::reducer::quantile_sorted(&sample, position));
            }
        }
        reduced
    }
}

#[cfg(test)]
#[path = "tests/window_tests.rs"]
mod tests;
