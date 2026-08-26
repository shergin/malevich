//! Exponentially weighted smoothing: the training-curve idiom.

/// The debiased exponentially weighted moving average of `values` at
/// smoothing factor `alpha` in `[0, 1)` — TensorBoard's scalar smoothing:
/// `state = alpha * state + (1 - alpha) * value`, divided by `1 - alpha^t` so
/// early outputs are unbiased instead of dragged toward zero. `alpha = 0` is
/// the identity; `0.97` is the familiar heavy smoothing.
///
/// A gap (`NaN`) stays a gap in the output and leaves the smoothing state
/// untouched, so the average resumes after it rather than absorbing it. A
/// scan over the ordered series — a batch transform, deliberately not a
/// mergeable accumulator (its value depends on every prior element in
/// order).
///
/// # Panics
///
/// Panics when `alpha` is not in `[0, 1)`.
pub fn ewma(values: &[f64], alpha: f64) -> Vec<f64> {
    assert!(
        (0.0..1.0).contains(&alpha),
        "ewma requires a smoothing factor in [0, 1)"
    );
    let mut state = 0.0f64;
    let mut weight = 1.0f64;
    values
        .iter()
        .map(|&value| {
            if !value.is_finite() {
                return f64::NAN;
            }
            state = alpha * state + (1.0 - alpha) * value;
            weight *= alpha;
            state / (1.0 - weight)
        })
        .collect()
}

#[cfg(test)]
#[path = "tests/ewma_tests.rs"]
mod tests;
