//! M4 downsampling: min/max/first/last per raster column.
//!
//! Jugel, Fischer, Mahlmann, Markl, "M4: A Visualization-Oriented Time Series Data
//! Aggregation" (PVLDB 2014): keeping the first, last, minimum, and maximum point of
//! every raster column reproduces that column's pixels exactly. The plot pipeline
//! buckets by the rendered column ([`m4_mapped`]), so the auto-inserted reduction is
//! pixel-identical to drawing every point. Finite runs are summarized independently:
//! gaps retain path topology, using O(width + gaps) memory when a column contains
//! several disconnected runs.

/// One uninterrupted run's aggregate: the four points that matter, in `(x, y)`
/// pairs.
#[derive(Debug, Clone, Copy)]
struct Run {
    first: (f64, f64),
    last: (f64, f64),
    min: (f64, f64),
    max: (f64, f64),
    break_before: bool,
    category: usize,
}

impl Run {
    fn in_category(point: (f64, f64), break_before: bool, category: usize) -> Run {
        Run {
            first: point,
            last: point,
            min: point,
            max: point,
            break_before,
            category,
        }
    }

    fn add(&mut self, point: (f64, f64)) {
        self.last = point;
        if point.1 < self.min.1 {
            self.min = point;
        }
        if point.1 > self.max.1 {
            self.max = point;
        }
    }

    fn merge(&mut self, later: Run) {
        debug_assert_eq!(self.category, later.category);
        self.last = later.last;
        if later.min.1 < self.min.1 {
            self.min = later.min;
        }
        if later.max.1 > self.max.1 {
            self.max = later.max;
        }
    }
}

/// A raster column normally has one run. Additional storage is paid only when
/// gaps divide that column into several runs.
#[derive(Debug, Clone)]
struct Bucket {
    first: Run,
    additional: Vec<Run>,
}

impl Bucket {
    fn new(run: Run) -> Bucket {
        Bucket {
            first: run,
            additional: Vec::new(),
        }
    }

    fn last_mut(&mut self) -> &mut Run {
        match self.additional.last_mut() {
            Some(run) => run,
            None => &mut self.first,
        }
    }

    fn push(&mut self, run: Run) {
        self.additional.push(run);
    }

    fn append(&mut self, later: Bucket) {
        self.additional.push(later.first);
        self.additional.extend(later.additional);
    }

    fn into_runs(self) -> impl Iterator<Item = Run> {
        std::iter::once(self.first).chain(self.additional)
    }
}

/// An M4 aggregator over a fixed x-domain divided into equal columns.
///
/// A mergeable monoid: aggregates built over chunks of a series combine with
/// [`M4::merge`] into exactly the state a single pass would have produced, provided
/// chunks are merged in series order (first/last are scan-order concepts).
#[derive(Debug, Clone)]
pub struct M4 {
    domain: (f64, f64),
    buckets: Vec<Option<Bucket>>,
    /// A gap after the last finite point makes the next run disconnected.
    pending_gap: bool,
}

impl M4 {
    /// An empty aggregator over `domain`, one bucket per raster `column`.
    ///
    /// # Panics
    ///
    /// Panics if the domain is not finite, `columns` is zero, or the requested
    /// allocation exceeds the defensive statistics budget. Use [`M4::try_new`]
    /// for caller-controlled geometry.
    pub fn new(domain: (f64, f64), columns: usize) -> M4 {
        M4::try_new(domain, columns)
            .expect("M4::new requires a finite domain and a bounded non-empty grid")
    }

    /// Fallible counterpart to [`M4::new`] for caller-controlled column counts.
    pub fn try_new(domain: (f64, f64), columns: usize) -> crate::Result<M4> {
        if !(domain.0.is_finite() && domain.1.is_finite()) {
            return Err(crate::Error::InvalidParameter {
                detail: "M4 needs a finite domain",
            });
        }
        if columns == 0 {
            return Err(crate::Error::EmptyDimension { what: "M4 columns" });
        }
        if columns > super::MAX_STAT_ELEMENTS {
            return Err(crate::Error::DimensionTooLarge {
                what: "M4 column count",
                requested: columns,
                limit: super::MAX_STAT_ELEMENTS,
            });
        }
        let mut buckets = Vec::new();
        buckets
            .try_reserve_exact(columns)
            .map_err(|_| crate::Error::AllocationFailed { what: "M4 buckets" })?;
        buckets.resize(columns, None);
        Ok(M4 {
            domain,
            buckets,
            pending_gap: false,
        })
    }

    /// Accumulates one point. A non-finite `y` records a gap; points with a
    /// non-finite `x` also record a gap, while finite out-of-domain x values are
    /// ignored.
    pub fn add(&mut self, x: f64, y: f64) {
        if !x.is_finite() {
            self.gap();
            return;
        }
        if let Some(index) = self.bucket_index(x) {
            self.record(index, x, y);
        }
    }

    /// Records `(x, y)` into bucket `index`. A non-finite `y` marks a gap there.
    fn record(&mut self, index: usize, x: f64, y: f64) {
        self.record_category(index, x, y, 0);
    }

    /// Records a point whose category is part of its path identity. A category
    /// transition starts a new run even when both adjacent values are finite.
    fn record_category(&mut self, index: usize, x: f64, y: f64, category: usize) {
        if !y.is_finite() {
            self.gap();
            return;
        }
        let point = (x, y);
        let break_before = std::mem::take(&mut self.pending_gap);
        match &mut self.buckets[index] {
            Some(bucket) => {
                if break_before || bucket.last_mut().category != category {
                    bucket.push(Run::in_category(point, true, category));
                } else {
                    bucket.last_mut().add(point);
                }
            }
            None => {
                self.buckets[index] =
                    Some(Bucket::new(Run::in_category(point, break_before, category)));
            }
        }
    }

    fn gap(&mut self) {
        self.pending_gap = true;
    }

    /// Merges `later` into `self`, as if `later`'s points had been added after
    /// `self`'s. Both sides must share the domain and column count.
    ///
    /// # Panics
    ///
    /// Panics if the two aggregators have different domains or column counts.
    pub fn merge(&mut self, later: &M4) {
        assert!(
            self.domain == later.domain && self.buckets.len() == later.buckets.len(),
            "M4::merge requires identical domains and column counts"
        );
        let self_last = self.buckets.iter().rposition(Option::is_some);
        let later_first = later.buckets.iter().position(Option::is_some);
        let boundary_gap = self.pending_gap;

        for (index, (mine, theirs)) in self
            .buckets
            .iter_mut()
            .zip(later.buckets.iter())
            .enumerate()
        {
            let Some(theirs) = theirs else { continue };
            let mut theirs = theirs.clone();
            if Some(index) == later_first && boundary_gap {
                theirs.first.break_before = true;
            }
            match mine {
                Some(bucket) => {
                    let same_boundary_bucket =
                        Some(index) == self_last && Some(index) == later_first;
                    if same_boundary_bucket && !theirs.first.break_before {
                        bucket.last_mut().merge(theirs.first);
                        bucket.additional.extend(theirs.additional);
                    } else {
                        bucket.append(theirs);
                    }
                }
                None => *mine = Some(theirs),
            }
        }
        self.pending_gap = if later_first.is_some() {
            later.pending_gap
        } else {
            self.pending_gap || later.pending_gap
        };
    }

    /// Emits the aggregated series: up to four finite points per uninterrupted run
    /// in each column, with a gap marker (`NaN`) before every disconnected run.
    pub fn emit(self) -> (Vec<f64>, Vec<f64>) {
        let (x, y, _) = self.emit_inner(false);
        (x, y)
    }

    /// Emits category identity beside each point. Gap entries carry the following
    /// run's category, though renderers deliberately ignore identity at a gap.
    fn emit_categories(self) -> (Vec<f64>, Vec<f64>, Vec<usize>) {
        self.emit_inner(true)
    }

    fn emit_inner(self, include_categories: bool) -> (Vec<f64>, Vec<f64>, Vec<usize>) {
        // Append a point unless it duplicates the last one written (collapses the
        // repeated first/min/max/last of a flat column into one).
        fn push(
            x: &mut Vec<f64>,
            y: &mut Vec<f64>,
            categories: &mut Vec<usize>,
            include_categories: bool,
            point: (f64, f64),
            category: usize,
        ) {
            if x.last() != Some(&point.0) || y.last() != Some(&point.1) {
                x.push(point.0);
                y.push(point.1);
                if include_categories {
                    categories.push(category);
                }
            }
        }
        let capacity = self.buckets.len() * 4;
        let mut x: Vec<f64> = Vec::with_capacity(capacity);
        let mut y: Vec<f64> = Vec::with_capacity(capacity);
        let mut categories = Vec::with_capacity(if include_categories { capacity } else { 0 });
        for bucket in self.buckets.into_iter().flatten() {
            for run in bucket.into_runs() {
                if run.break_before && y.last().is_none_or(|value| !value.is_nan()) {
                    x.push(f64::NAN);
                    y.push(f64::NAN);
                    if include_categories {
                        categories.push(run.category);
                    }
                }
                let mut points = [run.first, run.min, run.max, run.last];
                points.sort_by(|a, b| a.0.total_cmp(&b.0));
                for point in points {
                    push(
                        &mut x,
                        &mut y,
                        &mut categories,
                        include_categories,
                        point,
                        run.category,
                    );
                }
            }
        }
        (x, y, categories)
    }

    fn bucket_index(&self, x: f64) -> Option<usize> {
        let (lo, hi) = self.domain;
        if x < lo || x > hi {
            return None;
        }
        if hi == lo {
            return Some(0);
        }
        let position = crate::numeric::inverse_lerp(lo, hi, x) * self.buckets.len() as f64;
        Some((position as usize).min(self.buckets.len() - 1))
    }
}

/// Downsamples an x-sorted series to at most four finite points per uninterrupted
/// run in each raster column, preserving each run's silhouette and every gap.
/// Rendered over the same domain into a raster `columns` wide, the reduction is
/// pixel-exact. Convenience over [`M4`].
///
/// Returns `None` when `x` is not sorted ascending (M4 reorders points within
/// columns, which only preserves the drawn line for monotonic x), when the series
/// has no finite x extent, or when `columns` exceeds the defensive statistics
/// budget.
///
/// # Panics
///
/// Panics if `x` and `y` have different lengths, as the mark constructors do.
pub fn m4(x: &[f64], y: &[f64], columns: usize) -> Option<(Vec<f64>, Vec<f64>)> {
    assert_eq!(x.len(), y.len(), "m4 requires series of equal length");
    let columns = columns.max(1);
    if columns > super::MAX_STAT_ELEMENTS {
        return None;
    }
    let mut lo = f64::INFINITY;
    let mut hi = f64::NEG_INFINITY;
    let mut previous = f64::NEG_INFINITY;
    for &value in x {
        if !value.is_finite() {
            continue;
        }
        if value < previous {
            return None;
        }
        previous = value;
        lo = lo.min(value);
        hi = hi.max(value);
    }
    if !lo.is_finite() {
        return None;
    }
    let mut aggregate = M4::try_new((lo, hi), columns).ok()?;
    for (&xv, &yv) in x.iter().zip(y.iter()) {
        aggregate.add(xv, yv);
    }
    Some(aggregate.emit())
}

/// Reduces a line to at most four points per raster column, bucketing by the column
/// each point actually *renders* into (`map(x)` rounded to a subpixel column) rather
/// than by the raw x-domain. Because the buckets are the drawn pixel columns, the
/// reduction is pixel-exact for that raster — and it follows a non-linear axis (log)
/// for free, since `map` is the axis's own forward transform.
///
/// `x = None` means the implicit indices `0, 1, 2, …`, materialized on the fly.
/// Returns `None` when x is not ascending (M4 reorders within a column, exact only
/// for monotonic x). Non-finite x and non-finite mapped positions (a non-positive
/// value on a log axis) break the path; positions outside `[0, columns)` are
/// skipped.
pub(crate) fn m4_mapped(
    x: Option<&[f64]>,
    y: &[f64],
    columns: usize,
    map: impl Fn(f64) -> f64,
) -> Option<(Vec<f64>, Vec<f64>)> {
    if columns == 0 || columns > super::MAX_STAT_ELEMENTS {
        return None;
    }
    let mut aggregate = M4::try_new((0.0, 1.0), columns).ok()?;
    let mut previous = f64::NEG_INFINITY;
    let length = x.map_or(y.len(), |values| values.len().min(y.len()));
    for (index, &yv) in y.iter().take(length).enumerate() {
        let xv = match x {
            Some(values) => values[index],
            None => index as f64,
        };
        if !xv.is_finite() {
            aggregate.gap();
            continue;
        }
        if xv < previous {
            return None;
        }
        previous = xv;
        if !yv.is_finite() {
            aggregate.gap();
            continue;
        }
        let position = map(xv);
        if !position.is_finite() {
            aggregate.gap();
            continue;
        }
        let column = position.round();
        if (0.0..columns as f64).contains(&column) {
            aggregate.record(column as usize, xv, yv);
        }
    }
    Some(aggregate.emit())
}

/// Category-aware counterpart to [`m4_mapped`]. Category transitions are path
/// boundaries, and the returned identities stay aligned with the reduced points.
pub(crate) fn m4_mapped_categories(
    x: Option<&[f64]>,
    y: &[f64],
    categories: &[usize],
    columns: usize,
    map: impl Fn(f64) -> f64,
) -> Option<(Vec<f64>, Vec<f64>, Vec<usize>)> {
    if columns == 0 || columns > super::MAX_STAT_ELEMENTS {
        return None;
    }
    let mut aggregate = M4::try_new((0.0, 1.0), columns).ok()?;
    let mut previous_x = f64::NEG_INFINITY;
    let mut previous_category = None;
    let length = x
        .map_or(y.len(), |values| values.len().min(y.len()))
        .min(categories.len());

    for (index, &yv) in y.iter().take(length).enumerate() {
        let category = categories[index];
        if previous_category.is_some_and(|previous| previous != category) {
            aggregate.gap();
        }
        previous_category = Some(category);

        let xv = match x {
            Some(values) => values[index],
            None => index as f64,
        };
        if !xv.is_finite() {
            aggregate.gap();
            continue;
        }
        if xv < previous_x {
            return None;
        }
        previous_x = xv;
        if !yv.is_finite() {
            aggregate.gap();
            continue;
        }
        let position = map(xv);
        if !position.is_finite() {
            aggregate.gap();
            continue;
        }
        let column = position.round();
        if (0.0..columns as f64).contains(&column) {
            aggregate.record_category(column as usize, xv, yv, category);
        }
    }
    Some(aggregate.emit_categories())
}

#[cfg(test)]
#[path = "tests/m4_tests.rs"]
mod tests;
