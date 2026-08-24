//! Small, shared floating-point operations whose intermediate values must stay
//! representable when their finite endpoints are representable.

/// The normalized position of `value` between two finite endpoints.
///
/// Scaling first avoids overflowing `end - start` for opposite-sign extremes.
/// Values outside the endpoints may still produce an infinite position when the
/// mathematical result itself is not representable.
#[inline]
pub(crate) fn inverse_lerp(start: f64, end: f64, value: f64) -> f64 {
    if start == end {
        return if value.is_nan() { value } else { 0.5 };
    }
    let numerator = value - start;
    let denominator = end - start;
    if numerator.is_finite() && denominator.is_finite() {
        return numerator / denominator;
    }
    if value == start {
        return 0.0;
    }
    if value == end {
        return 1.0;
    }
    if !(start.is_finite() && end.is_finite()) {
        return f64::NAN;
    }
    let scale = start.abs().max(end.abs());
    let scaled_start = start / scale;
    let scaled_end = end / scale;
    (value / scale - scaled_start) / (scaled_end - scaled_start)
}

/// Linear interpolation that does not form an overflowing endpoint difference.
#[inline]
pub(crate) fn lerp(start: f64, end: f64, position: f64) -> f64 {
    if position == 0.0 {
        return start;
    }
    if position == 1.0 {
        return end;
    }
    if start.is_sign_negative() == end.is_sign_negative() {
        start + (end - start) * position
    } else {
        start * (1.0 - position) + end * position
    }
}

/// The midpoint of two finite values, including opposite-sign extremes.
#[inline]
pub(crate) fn midpoint(start: f64, end: f64) -> f64 {
    lerp(start, end, 0.5)
}

/// A finite span divided into `parts` equal pieces.
pub(crate) fn span_per(start: f64, end: f64, parts: usize) -> Option<f64> {
    if parts == 0 || !(start.is_finite() && end.is_finite() && start < end) {
        return None;
    }
    let divisor = parts as f64;
    let direct = (end - start) / divisor;
    if direct.is_finite() && direct > 0.0 {
        return Some(direct);
    }
    let scaled = end / divisor - start / divisor;
    (scaled.is_finite() && scaled > 0.0).then_some(scaled)
}

/// A per-part span rounded upward just enough for `parts` fused steps from
/// `start` to cover `end`.
///
/// Division can round a mathematically exact last edge just below `end`, most
/// visibly when a tiny endpoint is added to a much larger same-sign span. The
/// initial quotient is within a few ulps; the bound keeps this adjustment total
/// if that assumption is ever invalidated by a platform implementation.
pub(crate) fn covering_span_per(start: f64, end: f64, parts: usize) -> Option<f64> {
    let mut width = span_per(start, end, parts)?;
    for _ in 0..8 {
        let covered = width.mul_add(parts as f64, start);
        if covered >= end {
            return Some(width);
        }
        if covered.is_nan() {
            return None;
        }
        width = width.next_up();
        if !width.is_finite() {
            return None;
        }
    }
    None
}

/// How many `unit`-sized steps fit between finite endpoints.
pub(crate) fn span_ratio(start: f64, end: f64, unit: f64) -> Option<f64> {
    if !(start.is_finite() && end.is_finite() && unit.is_finite() && unit > 0.0) {
        return None;
    }
    let direct = (end - start) / unit;
    if direct.is_finite() {
        return Some(direct);
    }
    let scaled = end / unit - start / unit;
    scaled.is_finite().then_some(scaled)
}

/// A finite, non-empty extent around one finite value.
///
/// Prefer the familiar half-unit padding while it remains representable. At the
/// ends of the floating-point range, fall back to adjacent finite values.
pub(crate) fn extent_around(value: f64) -> (f64, f64) {
    let conventional = (value - 0.5, value + 0.5);
    if conventional.0.is_finite() && conventional.1.is_finite() && conventional.0 < conventional.1 {
        return conventional;
    }

    let lower = value.next_down();
    let upper = value.next_up();
    if lower.is_finite() && upper.is_finite() {
        (lower, upper)
    } else if lower.is_finite() {
        (lower, value)
    } else {
        (value, upper)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        covering_span_per, extent_around, inverse_lerp, lerp, midpoint, span_per, span_ratio,
    };

    #[test]
    fn opposite_sign_extremes_keep_their_endpoints_and_midpoint() {
        let low = -f64::MAX;
        let high = f64::MAX;
        assert_eq!(inverse_lerp(low, high, low), 0.0);
        assert_eq!(inverse_lerp(low, high, 0.0), 0.5);
        assert_eq!(inverse_lerp(low, high, high), 1.0);
        assert_eq!(lerp(low, high, 0.0), low);
        assert_eq!(lerp(low, high, 0.5), 0.0);
        assert_eq!(lerp(low, high, 1.0), high);
        assert_eq!(midpoint(low, high), 0.0);
    }

    #[test]
    fn equal_subnormals_do_not_round_their_midpoint_to_zero() {
        let value = f64::from_bits(1);
        assert_eq!(midpoint(value, value), value);
    }

    #[test]
    fn constant_extreme_extents_stay_finite_and_non_empty() {
        for value in [-f64::MAX, f64::MAX] {
            let (low, high) = extent_around(value);
            assert!(low.is_finite() && high.is_finite() && low < high);
            assert!(low <= value && value <= high);
        }
    }

    #[test]
    fn extreme_spans_can_be_divided_without_forming_the_whole_span() {
        assert_eq!(span_per(-f64::MAX, f64::MAX, 2), Some(f64::MAX));
        assert_eq!(span_ratio(-f64::MAX, f64::MAX, f64::MAX), Some(2.0));
        assert_eq!(span_per(-f64::MAX, f64::MAX, 1), None);
    }

    #[test]
    fn covering_width_compensates_for_a_rounded_last_edge() {
        let start = -2.380_536_100_667_193_7e146;
        let end = -1.080_372_508_640_452_4e-296;
        let plain = span_per(start, end, 3).unwrap();
        assert!(plain.mul_add(3.0, start) < end);

        let covering = covering_span_per(start, end, 3).unwrap();
        assert!(covering.mul_add(3.0, start) >= end);
    }
}
