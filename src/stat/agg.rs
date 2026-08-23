//! Group-by aggregation with the shared reducer vocabulary.

use std::collections::HashMap;
use std::collections::hash_map::Entry;

use crate::data::IntoSeries;

/// Values grouped by string keys, ready to reduce.
///
/// Groups keep first-appearance order; non-finite values are ignored. The named
/// methods are sugar over [`reduce`](Agg::reduce) with the crate's one
/// [`Reducer`](super::Reducer) vocabulary — the same names reduce windows and
/// bins, and `reduce(Reducer::Percentile(q))` opens the quantiles. Each returns
/// `(categories, values)`, which feeds [`crate::mark::Bars::new`] directly.
///
/// ```
/// use malevich::stat::Agg;
///
/// let (categories, means) = Agg::by(
///     ["a", "b", "a", "b"],
///     &[1.0, 10.0, 3.0, 30.0][..],
/// )
/// .mean();
/// assert_eq!(categories, ["a", "b"]);
/// assert_eq!(means, [2.0, 20.0]);
/// ```
#[derive(Debug, Clone)]
pub struct Agg {
    /// Each label owns its stable first-seen group index. Keeping labels only in
    /// the lookup avoids a second owned copy solely to preserve order.
    keys: HashMap<String, usize>,
    groups: Vec<Vec<f64>>,
}

impl Agg {
    /// Applies any named [`Reducer`](super::Reducer) per group — the percentile door:
    /// `agg.reduce(Reducer::Percentile(0.95))`.
    pub fn reduce(self, reducer: super::Reducer) -> (Vec<String>, Vec<f64>) {
        let mut ordered_keys = vec![None; self.groups.len()];
        for (key, index) in self.keys {
            ordered_keys[index] = Some(key);
        }
        let keys = ordered_keys
            .into_iter()
            .map(|key| key.expect("every group has one interned key"))
            .collect();
        let values = self
            .groups
            .iter()
            .map(|group| reducer.reduce(group))
            .collect();
        (keys, values)
    }

    /// Groups `values` by their paired `keys`.
    ///
    /// # Panics
    ///
    /// Panics if there are not exactly as many keys as values.
    pub fn by<'a>(
        keys: impl IntoIterator<Item = impl Into<String>>,
        values: impl IntoSeries<'a>,
    ) -> Agg {
        let values = values.into_series();
        let mut keys = keys.into_iter();
        let mut result = Agg {
            keys: HashMap::new(),
            groups: Vec::new(),
        };
        for value in values.iter() {
            let Some(key) = keys.next() else {
                panic!("Agg::by requires one key per value");
            };
            let index = match result.keys.entry(key.into()) {
                Entry::Occupied(entry) => *entry.get(),
                Entry::Vacant(entry) => {
                    let index = result.groups.len();
                    entry.insert(index);
                    result.groups.push(Vec::new());
                    index
                }
            };
            if value.is_finite() {
                result.groups[index].push(value);
            }
        }
        assert!(keys.next().is_none(), "Agg::by requires one key per value");
        result
    }

    /// The number of finite values per group.
    pub fn count(self) -> (Vec<String>, Vec<f64>) {
        self.reduce(super::Reducer::Count)
    }

    /// The sum per group (0 for empty groups).
    pub fn sum(self) -> (Vec<String>, Vec<f64>) {
        self.reduce(super::Reducer::Sum)
    }

    /// The mean per group (a gap for empty groups).
    pub fn mean(self) -> (Vec<String>, Vec<f64>) {
        self.reduce(super::Reducer::Mean)
    }

    /// The minimum per group (a gap for empty groups).
    pub fn min(self) -> (Vec<String>, Vec<f64>) {
        self.reduce(super::Reducer::Min)
    }

    /// The maximum per group (a gap for empty groups).
    pub fn max(self) -> (Vec<String>, Vec<f64>) {
        self.reduce(super::Reducer::Max)
    }

    /// The median per group (a gap for empty groups; the mean of the middle pair
    /// for even counts).
    pub fn median(self) -> (Vec<String>, Vec<f64>) {
        self.reduce(super::Reducer::Median)
    }
}

#[cfg(test)]
#[path = "tests/agg_tests.rs"]
mod tests;
