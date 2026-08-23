//! Kernel density estimation: a smooth distribution from a sample.

/// The Gaussian KDE of `values`, evaluated at `points` positions across the data
/// extent (padded by three bandwidths). Returns `(positions, densities)`, or `None`
/// without finite values.
///
/// Bandwidth follows Silverman's rule of thumb —
/// `0.9 * min(σ, IQR / 1.34) * n^(-1/5)` — and evaluation runs on linearly binned
/// counts with a truncated Gaussian kernel: O(n + points × kernel), no FFT.
/// Returns `None` when `points` exceeds the defensive statistics budget.
pub fn kde(values: &[f64], points: usize) -> Option<(Vec<f64>, Vec<f64>)> {
    if points > super::MAX_STAT_ELEMENTS {
        return None;
    }
    let finite: Vec<f64> = values.iter().copied().filter(|v| v.is_finite()).collect();
    if finite.is_empty() || points < 2 {
        return None;
    }
    let n = finite.len() as f64;
    let mean = finite.iter().sum::<f64>() / n;
    let variance = finite.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let sigma = variance.sqrt();

    let mut sorted = finite.clone();
    sorted.sort_by(f64::total_cmp);
    let quantile = |p: f64| super::reducer::quantile_sorted(&sorted, p);
    let iqr = quantile(0.75) - quantile(0.25);
    let spread = if iqr > 0.0 {
        sigma.min(iqr / 1.34)
    } else {
        sigma
    };
    let bandwidth = if spread > 0.0 {
        0.9 * spread * n.powf(-0.2)
    } else {
        // A degenerate sample still deserves a bump rather than a spike.
        1.0
    };

    let (low, high) = (sorted[0], sorted[sorted.len() - 1]);
    let start = low - 3.0 * bandwidth;
    let end = high + 3.0 * bandwidth;
    let step = crate::numeric::span_per(start, end, points - 1)?;
    // At extreme magnitudes the ±3σ padding can fall below the value's ULP, so the
    // grid collapses (step 0 or non-finite). A single point has no density curve;
    // refuse rather than binning through a zero step into a giant allocation.
    if !(step.is_finite() && step > 0.0) {
        return None;
    }

    // Linear binning onto the evaluation grid.
    let mut binned = vec![0.0f64; points];
    for &value in &finite {
        let position = crate::numeric::inverse_lerp(start, end, value) * (points - 1) as f64;
        let index = position.floor() as usize;
        let fraction = position - position.floor();
        if index + 1 < points {
            binned[index] += 1.0 - fraction;
            binned[index + 1] += fraction;
        } else {
            binned[points - 1] += 1.0;
        }
    }

    // Truncated Gaussian kernel over the binned counts, never wider than the grid.
    let radius = ((3.0 * bandwidth / step).ceil() as usize).clamp(1, points);
    let kernel: Vec<f64> = (0..=radius)
        .map(|k| {
            let distance = k as f64 * step / bandwidth;
            (-0.5 * distance * distance).exp()
        })
        .collect();
    let normalization = 1.0 / (n * bandwidth * (2.0 * std::f64::consts::PI).sqrt());

    let densities: Vec<f64> = (0..points)
        .map(|i| {
            let mut sum = binned[i] * kernel[0];
            for k in 1..=radius {
                if i >= k {
                    sum += binned[i - k] * kernel[k];
                }
                if i + k < points {
                    sum += binned[i + k] * kernel[k];
                }
            }
            sum * normalization
        })
        .collect();
    let positions = (0..points)
        .map(|index| crate::numeric::lerp(start, end, index as f64 / (points - 1) as f64))
        .collect();
    Some((positions, densities))
}

#[cfg(test)]
#[path = "tests/kde_tests.rs"]
mod tests;
