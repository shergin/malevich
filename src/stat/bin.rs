//! Histogram binning: uniform bins with nice-number edges.

use crate::scale::Ticks;

/// A uniform histogram: `bins` counting buckets of `width`, starting at `start`.
///
/// A mergeable monoid: partial histograms over chunks combine with [`Bins::merge`].
/// Values outside the covered range are ignored (the [`Bins::auto`] constructor
/// sizes the range to the data, so nothing drops there); the right edge of the last
/// bin is inclusive, so the maximum lands inside.
#[derive(Debug, Clone, PartialEq)]
pub struct Bins {
    start: f64,
    width: f64,
    counts: Vec<u64>,
}

impl Bins {
    /// An empty histogram of `bins` buckets of `width`, starting at `start`.
    ///
    /// # Panics
    ///
    /// Panics if `width` is not finite and positive, `bins` is zero, or the
    /// requested allocation exceeds the defensive statistics budget. Use
    /// [`Bins::try_new`] for caller-controlled geometry.
    pub fn new(start: f64, width: f64, bins: usize) -> Bins {
        Bins::try_new(start, width, bins).expect(
            "Bins::new requires a finite start, positive width, and a bounded non-empty grid",
        )
    }

    /// Fallible counterpart to [`Bins::new`] for caller-controlled bin counts.
    pub fn try_new(start: f64, width: f64, bins: usize) -> crate::Result<Bins> {
        if !(start.is_finite() && width.is_finite() && width > 0.0) {
            return Err(crate::Error::InvalidParameter {
                detail: "Bins needs a finite start and a finite positive width",
            });
        }
        Self::validate_count(bins)?;
        let mut counts = Vec::new();
        counts
            .try_reserve_exact(bins)
            .map_err(|_| crate::Error::AllocationFailed {
                what: "Bins buckets",
            })?;
        counts.resize(bins, 0);
        Ok(Bins {
            start,
            width,
            counts,
        })
    }

    /// A histogram with exactly `count` equal-width bins over the finite values'
    /// extent. Returns `None` when there are no finite values. A constant sample
    /// receives a small finite extent around its value.
    ///
    /// Unlike manually subtracting the endpoints, this keeps opposite-sign finite
    /// extremes representable whenever the requested per-bin width is representable.
    ///
    /// ```
    /// use malevich::stat::Bins;
    ///
    /// let bins = Bins::try_uniform(&[0.0, 0.5, 1.0], 2).unwrap().unwrap();
    /// assert_eq!(bins.counts(), [1, 2]);
    /// ```
    pub fn try_uniform(values: &[f64], count: usize) -> crate::Result<Option<Bins>> {
        Self::validate_count(count)?;
        let extent = values
            .iter()
            .copied()
            .filter(|value| value.is_finite())
            .fold(None, |extent, value| {
                Some(extent.map_or((value, value), |(min, max): (f64, f64)| {
                    (min.min(value), max.max(value))
                }))
            });
        let Some((min, max)) = extent else {
            return Ok(None);
        };
        let (start, end) = if min == max {
            crate::numeric::extent_around(min)
        } else {
            (min, max)
        };
        let width = crate::numeric::covering_span_per(start, end, count).ok_or(
            crate::Error::InvalidParameter {
                detail: "histogram extent cannot be represented with the requested bin count",
            },
        )?;
        let mut histogram = Self::try_new(start, width, count)?;
        for &value in values {
            histogram.add(value);
        }
        Ok(Some(histogram))
    }

    fn validate_count(bins: usize) -> crate::Result<()> {
        if bins == 0 {
            return Err(crate::Error::EmptyDimension {
                what: "Bins buckets",
            });
        }
        if bins > super::MAX_STAT_ELEMENTS {
            return Err(crate::Error::DimensionTooLarge {
                what: "Bins bucket count",
                requested: bins,
                limit: super::MAX_STAT_ELEMENTS,
            });
        }
        Ok(())
    }

    /// Bins sized to the data: bin count by the larger of Sturges' rule and
    /// Freedman–Diaconis (the NumPy `auto` policy), capped at `limit`, with widths
    /// and edges snapped to the same nice decimals ticks use. `None` without finite
    /// values or when the requested cap cannot represent the complete span; use
    /// [`Bins::try_auto`] to distinguish those cases.
    pub fn auto(values: &[f64], limit: usize) -> Option<Bins> {
        Bins::try_auto(values, limit).ok().flatten()
    }

    /// Fallible automatic binning. Unlike [`Bins::auto`], this distinguishes an
    /// empty sample from finite data whose requested bin cap cannot represent the
    /// complete numeric span.
    pub fn try_auto(values: &[f64], limit: usize) -> crate::Result<Option<Bins>> {
        let mut finite: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
        if finite.is_empty() {
            return Ok(None);
        }
        let n = finite.len();
        let (min, max) = finite
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &v| {
                (lo.min(v), hi.max(v))
            });
        if min == max {
            return Self::try_uniform(&finite, 1);
        }

        let sturges = (n as f64).log2().ceil() as usize + 1;
        let quarter = n / 4;
        let (_, q1, _) = finite.select_nth_unstable_by(quarter, f64::total_cmp);
        let q1 = *q1;
        let upper = (3 * n) / 4;
        let (_, q3, _) = finite.select_nth_unstable_by(upper.min(n - 1), f64::total_cmp);
        let q3 = *q3;
        let iqr = crate::numeric::span_per(q1, q3, 1);
        let fd = if let Some(iqr) = iqr {
            let width = 2.0 * iqr / (n as f64).cbrt();
            crate::numeric::span_ratio(min, max, width)
                .map(|count| count.ceil() as usize)
                .unwrap_or(0)
        } else {
            0
        };
        let target = sturges.max(fd).clamp(1, limit.max(1));

        // Snap the bin width and edges to the nice decimals the tick engine picks,
        // so bin boundaries land on readable numbers.
        let cap = limit.clamp(1, super::MAX_STAT_ELEMENTS);
        let ticks = Ticks::linear(min, max, target.min(50));
        let fallback_width = crate::numeric::covering_span_per(min, max, target).ok_or(
            crate::Error::InvalidParameter {
                detail: "histogram span cannot be represented at the requested bin cap",
            },
        )?;
        let mut width = ticks
            .step()
            .filter(|step| step.is_finite() && *step > 0.0)
            .unwrap_or(fallback_width);
        let snapped_start = (min / width).floor() * width;
        let mut start = if snapped_start.is_finite() && snapped_start <= min {
            snapped_start
        } else {
            min
        };
        let mut bins = crate::numeric::span_ratio(start, max, width)
            .map(|count| count.ceil() as usize)
            .unwrap_or(usize::MAX)
            .max(1);
        // Never drop data to honor the cap: if the nice width needs more bins than
        // allowed, widen it so the same span fits in `cap` bins. Coverage is the
        // contract; readable edges are the preference that yields first. Falling
        // back to an exact `cap`-way split from `min` covers both endpoints without
        // requiring the complete span to be representable first.
        if bins > cap {
            start = min;
            width = crate::numeric::covering_span_per(min, max, cap).ok_or(
                crate::Error::InvalidParameter {
                    detail: "histogram span cannot be represented at the requested bin cap",
                },
            )?;
            bins = cap;
        }
        let mut result = Bins::try_new(start, width, bins)?;
        for &value in &finite {
            result.add(value);
        }
        Ok(Some(result))
    }

    /// Counts one value; non-finite and out-of-range values are ignored.
    pub fn add(&mut self, value: f64) {
        if let Some(index) = self.bucket(value) {
            self.counts[index] += 1;
        }
    }

    /// The bucket a value counts into — the one bucketing rule [`Bins::add`]
    /// and [`binned`] share: `None` for non-finite or out-of-range values,
    /// last-edge inclusive.
    fn bucket(&self, value: f64) -> Option<usize> {
        if !value.is_finite() {
            return None;
        }
        if value < self.start {
            return None;
        }
        let end = self.end();
        if end.is_finite() && end > self.start {
            if value > end {
                return None;
            }
            let position = crate::numeric::inverse_lerp(self.start, end, value);
            let index = (position * self.counts.len() as f64) as usize;
            return if index < self.counts.len() {
                Some(index)
            } else {
                Some(self.counts.len() - 1)
            };
        }

        // A caller can request geometry whose mathematical end exceeds MAX. Find
        // the first upper edge above the value without materializing that span.
        let mut low = 0usize;
        let mut high = self.counts.len();
        while low < high {
            let middle = low + (high - low) / 2;
            let upper = self.width.mul_add((middle + 1) as f64, self.start);
            if value < upper {
                high = middle;
            } else {
                low = middle + 1;
            }
        }
        Some(low.min(self.counts.len() - 1))
    }

    /// Merges another histogram with the same geometry into this one.
    ///
    /// # Panics
    ///
    /// Panics if the two histograms have different starts, widths, or bin counts.
    pub fn merge(&mut self, other: &Bins) {
        assert!(
            self.start == other.start
                && self.width == other.width
                && self.counts.len() == other.counts.len(),
            "Bins::merge requires identical geometry"
        );
        for (mine, theirs) in self.counts.iter_mut().zip(other.counts.iter()) {
            *mine += theirs;
        }
    }

    /// The left edge of the first bin.
    pub fn start(&self) -> f64 {
        self.start
    }

    /// The width of every bin.
    pub fn width(&self) -> f64 {
        self.width
    }

    /// The right edge of the last bin.
    pub fn end(&self) -> f64 {
        self.width.mul_add(self.counts.len() as f64, self.start)
    }

    /// The per-bin counts, in order.
    pub fn counts(&self) -> &[u64] {
        &self.counts
    }
}

/// The result of [`bins2`]: a 2D histogram — a density grid plus the data extents
/// it covers. Named to distinguish it from [`crate::Grid`], which is small multiples.
#[derive(Debug, Clone, PartialEq)]
pub struct Histogram2d {
    /// Row-major counts, row 0 at the bottom.
    pub counts: Vec<f64>,
    /// Columns per row.
    pub columns: usize,
    /// The x extent the grid covers.
    pub x: (f64, f64),
    /// The y extent the grid covers.
    pub y: (f64, f64),
}

/// 2D histogram counts on a uniform `columns` × `rows` grid over the data's finite
/// extent, or `None` without finite pairs.
///
/// # Panics
///
/// Panics if the series lengths differ, the grid is empty, or its area exceeds
/// the defensive statistics budget. Use [`try_bins2`] for caller-controlled grids.
pub fn bins2(x: &[f64], y: &[f64], columns: usize, rows: usize) -> Option<Histogram2d> {
    try_bins2(x, y, columns, rows)
        .expect("bins2 requires equal channels and a bounded non-empty grid")
}

/// Fallible counterpart to [`bins2`] for caller-controlled grid geometry.
pub fn try_bins2(
    x: &[f64],
    y: &[f64],
    columns: usize,
    rows: usize,
) -> crate::Result<Option<Histogram2d>> {
    if x.len() != y.len() {
        return Err(crate::Error::UnequalChannels {
            mark: "bins2: x and y",
            lengths: (x.len(), y.len()),
        });
    }
    if columns == 0 || rows == 0 {
        return Err(crate::Error::EmptyDimension { what: "bins2 grid" });
    }
    let cells = columns
        .checked_mul(rows)
        .ok_or(crate::Error::DimensionTooLarge {
            what: "bins2 cell count",
            requested: usize::MAX,
            limit: super::MAX_STAT_ELEMENTS,
        })?;
    if cells > super::MAX_STAT_ELEMENTS {
        return Err(crate::Error::DimensionTooLarge {
            what: "bins2 cell count",
            requested: cells,
            limit: super::MAX_STAT_ELEMENTS,
        });
    }
    let mut x_extent: Option<(f64, f64)> = None;
    let mut y_extent: Option<(f64, f64)> = None;
    for (&xv, &yv) in x.iter().zip(y.iter()) {
        if !xv.is_finite() || !yv.is_finite() {
            continue;
        }
        x_extent = Some(x_extent.map_or((xv, xv), |(lo, hi)| (lo.min(xv), hi.max(xv))));
        y_extent = Some(y_extent.map_or((yv, yv), |(lo, hi)| (lo.min(yv), hi.max(yv))));
    }
    let (Some(x_extent), Some(y_extent)) = (x_extent, y_extent) else {
        return Ok(None);
    };
    // A constant coordinate leaves a zero-width extent that later renders blank
    // (the inverse mapping rejects equal endpoints). Give it a scale-aware span,
    // always numerically distinct even at large magnitudes, so the cells show.
    let widen = |(lo, hi): (f64, f64)| -> (f64, f64) {
        if lo < hi {
            (lo, hi)
        } else {
            crate::numeric::extent_around(lo)
        }
    };
    let mut counts = Vec::new();
    counts
        .try_reserve_exact(cells)
        .map_err(|_| crate::Error::AllocationFailed {
            what: "bins2 cells",
        })?;
    counts.resize(cells, 0.0f64);
    for (&xv, &yv) in x.iter().zip(y.iter()) {
        if !xv.is_finite() || !yv.is_finite() {
            continue;
        }
        let column = if x_extent.0 == x_extent.1 {
            0
        } else {
            (crate::numeric::inverse_lerp(x_extent.0, x_extent.1, xv) * columns as f64) as usize
        };
        let row = if y_extent.0 == y_extent.1 {
            0
        } else {
            (crate::numeric::inverse_lerp(y_extent.0, y_extent.1, yv) * rows as f64) as usize
        };
        counts[row.min(rows - 1) * columns + column.min(columns - 1)] += 1.0;
    }
    Ok(Some(Histogram2d {
        counts,
        columns,
        x: widen(x_extent),
        y: widen(y_extent),
    }))
}

/// Reduces `y` per bin of its paired `x`, over the bins' geometry — binned
/// means, medians, percentiles, any [`Reducer`](super::Reducer). One value per
/// bin, bucketed by the exact rule [`Bins::add`] counts with; a bin that
/// catches nothing reduces like an empty set (a gap, except `Count` and
/// `Sum`).
///
/// ```
/// use malevich::stat::{Bins, Reducer, binned};
///
/// let x = [0.5, 1.5, 1.6, 2.5];
/// let y = [10.0, 20.0, 30.0, 40.0];
/// let bins = Bins::new(0.0, 1.0, 3);
/// assert_eq!(binned(&x, &y, &bins, Reducer::Mean), [10.0, 25.0, 40.0]);
/// ```
///
/// # Panics
///
/// Panics if the two slices have different lengths.
pub fn binned(x: &[f64], y: &[f64], bins: &Bins, reducer: super::Reducer) -> Vec<f64> {
    assert_eq!(x.len(), y.len(), "binned requires slices of equal length");
    let mut buckets: Vec<Vec<f64>> = vec![Vec::new(); bins.counts().len()];
    for (&position, &value) in x.iter().zip(y) {
        if let Some(index) = bins.bucket(position) {
            buckets[index].push(value);
        }
    }
    buckets
        .iter()
        .map(|bucket| reducer.reduce(bucket))
        .collect()
}

#[cfg(test)]
#[path = "tests/bin_tests.rs"]
mod tests;
