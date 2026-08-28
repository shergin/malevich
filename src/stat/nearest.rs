//! Nearest-value lookup: the crosshair-snapping primitive.

/// The index of the finite value nearest `target`, the first on ties, or
/// `None` when `values` is empty or holds no finite value (or `target` is not
/// finite).
///
/// This is what makes a cursor readout honest: instead of interpolating a
/// value that was never observed, snap to the datum that actually exists.
/// One linear scan — the values need not be sorted.
///
/// ```
/// let dates = [10.0, 20.0, f64::NAN, 40.0];
/// assert_eq!(malevich::stat::nearest(&dates, 24.0), Some(1));
/// assert_eq!(malevich::stat::nearest(&dates, 35.0), Some(3));
/// assert_eq!(malevich::stat::nearest(&[], 1.0), None);
/// ```
pub fn nearest(values: &[f64], target: f64) -> Option<usize> {
    if !target.is_finite() {
        return None;
    }
    let mut best: Option<(usize, f64)> = None;
    for (index, &value) in values.iter().enumerate() {
        if !value.is_finite() {
            continue;
        }
        let distance = (value - target).abs();
        if best.is_none_or(|(_, nearest)| distance < nearest) {
            best = Some((index, distance));
        }
    }
    best.map(|(index, _)| index)
}

#[cfg(test)]
#[path = "tests/nearest_tests.rs"]
mod tests;
