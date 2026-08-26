//! Receiver operating characteristic: the classifier's threshold sweep.

/// The ROC curve of binary classification scores: one `(false positive rate,
/// true positive rate)` point per distinct score, sweeping the threshold from
/// strictest down, prefixed with `(0, 0)` — the standard step construction,
/// with tied scores grouped into one step. Feeds a line; the area under it is
/// [`auc`].
///
/// Non-finite scores are excluded together with their labels (the gap
/// convention). When either class is absent the rates would divide by zero,
/// so the result is empty vectors, not fabricated points. A batch transform
/// in the [`ecdf`](super::ecdf) family: order statistics over the complete
/// sample, deliberately not a mergeable accumulator.
///
/// # Panics
///
/// Panics if the slices have different lengths.
pub fn roc(scores: &[f64], labels: &[bool]) -> (Vec<f64>, Vec<f64>) {
    assert_eq!(
        scores.len(),
        labels.len(),
        "roc requires slices of equal length"
    );
    let mut pairs: Vec<(f64, bool)> = scores
        .iter()
        .zip(labels)
        .filter(|(score, _)| score.is_finite())
        .map(|(&score, &label)| (score, label))
        .collect();
    let positives = pairs.iter().filter(|(_, label)| *label).count();
    let negatives = pairs.len() - positives;
    if positives == 0 || negatives == 0 {
        return (Vec::new(), Vec::new());
    }
    pairs.sort_by(|a, b| b.0.total_cmp(&a.0));

    let (mut fpr, mut tpr) = (vec![0.0], vec![0.0]);
    let (mut false_hits, mut true_hits) = (0usize, 0usize);
    let mut index = 0;
    while index < pairs.len() {
        let threshold = pairs[index].0;
        while index < pairs.len() && pairs[index].0 == threshold {
            if pairs[index].1 {
                true_hits += 1;
            } else {
                false_hits += 1;
            }
            index += 1;
        }
        fpr.push(false_hits as f64 / negatives as f64);
        tpr.push(true_hits as f64 / positives as f64);
    }
    (fpr, tpr)
}

/// The trapezoid integral of the polyline `y` over `x` — the area under an
/// ROC or precision–recall curve. A point with a non-finite member breaks the
/// polyline (the gap convention) and the gap contributes no area; with no
/// complete segment at all the area is a gap (`NaN`), never an invented zero.
///
/// # Panics
///
/// Panics if the slices have different lengths.
pub fn auc(x: &[f64], y: &[f64]) -> f64 {
    assert_eq!(x.len(), y.len(), "auc requires slices of equal length");
    let mut area = None;
    let mut previous: Option<(f64, f64)> = None;
    for (&px, &py) in x.iter().zip(y) {
        if !(px.is_finite() && py.is_finite()) {
            previous = None;
            continue;
        }
        if let Some((qx, qy)) = previous {
            *area.get_or_insert(0.0) += (px - qx) * (py + qy) / 2.0;
        }
        previous = Some((px, py));
    }
    area.unwrap_or(f64::NAN)
}

#[cfg(test)]
#[path = "tests/roc_tests.rs"]
mod tests;
