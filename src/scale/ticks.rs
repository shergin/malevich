//! Tick placement for linear axes.
//!
//! Implements the extended Wilkinson algorithm (Justin Talbot, Sharon Lin, Pat
//! Hanrahan, "An Extension of Wilkinson's Algorithm for Positioning Tick Labels on
//! Axes", IEEE InfoVis 2010): a branch-and-bound search over step mantissas, skip
//! amounts, label counts, and magnitudes, scored by a weighted sum of simplicity,
//! coverage, density, and legibility.
//!
//! One refinement over the paper: chosen tick values are carried as exact decimals
//! (integer mantissa times a power of ten), so labels are produced by integer math —
//! no binary-float artifacts, and a uniform number of decimals across the axis.

use super::format;

/// Step mantissas in preference order, as `(integer mantissa, value)` with
/// `value = mantissa / 10`, so that every tick value stays an exact decimal.
const STEPS: [(i128, f64); 6] = [
    (10, 1.0),
    (50, 5.0),
    (20, 2.0),
    (25, 2.5),
    (40, 4.0),
    (30, 3.0),
];

/// Score weights for simplicity, coverage, density, and legibility.
const WEIGHTS: [f64; 4] = [0.25, 0.2, 0.5, 0.05];

/// Powers of ten that are exactly representable in `f64`.
const POW10: [f64; 23] = [
    1e0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10, 1e11, 1e12, 1e13, 1e14, 1e15, 1e16,
    1e17, 1e18, 1e19, 1e20, 1e21, 1e22,
];

/// One axis tick: a position in data coordinates and its label.
#[derive(Debug, Clone, PartialEq)]
pub struct Tick {
    /// Position in data coordinates.
    pub value: f64,
    /// Label text: an exact decimal rendering of `value`, which parses back to it.
    /// Axes reaching ten thousand (or a ten-thousandth) carry one shared SI prefix
    /// (`20k`, `2.5M`, `100µ`); the numeric part times the prefix factor still
    /// equals `value` exactly. Zero is always plain `0`.
    pub label: String,
}

/// Ticks chosen for a linear axis: ascending, uniformly spaced, decimal-exact.
///
/// All labels of a set share the same number of fraction digits, so they align when
/// stacked on an axis.
#[derive(Debug, Clone, PartialEq)]
pub struct Ticks {
    ticks: Vec<Tick>,
    step: Option<f64>,
}

impl Ticks {
    /// Places approximately `target` ticks over `[min, max]` with the extended
    /// Wilkinson algorithm.
    ///
    /// The bounds may be given in either order. A `target` below 2 is treated as 2,
    /// and the returned count is close to, not exactly, `target`. Equal bounds yield
    /// a single tick at that value. The chosen ticks may extend beyond the data range
    /// (that is the algorithm's coverage trade-off), typically by less than one step
    /// on either side.
    ///
    /// # Panics
    ///
    /// Panics if `min` or `max` is not finite.
    pub fn linear(min: f64, max: f64, target: usize) -> Ticks {
        assert!(
            min.is_finite() && max.is_finite(),
            "Ticks::linear requires finite bounds, got {min} and {max}"
        );
        let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
        let target = target.max(2);
        if lo == hi {
            return Ticks {
                ticks: vec![Tick {
                    value: lo,
                    label: lo.to_string(),
                }],
                step: None,
            };
        }
        match search(lo, hi, target) {
            Some(best) => materialize(&best),
            None => Ticks {
                ticks: vec![
                    Tick {
                        value: lo,
                        label: lo.to_string(),
                    },
                    Tick {
                        value: hi,
                        label: hi.to_string(),
                    },
                ],
                step: Some(hi - lo),
            },
        }
    }

    /// One tick per band category at its index, labels fitted to `budget` cells
    /// with `ellipsis`. Band geometry decides where they land; this only carries
    /// the (position, label) pairs through the tick pipeline, so band axes reuse
    /// the same chrome as numeric ones.
    pub(crate) fn bands(categories: &[String], budget: usize, ellipsis: char) -> Ticks {
        Ticks {
            ticks: categories
                .iter()
                .enumerate()
                .map(|(index, category)| Tick {
                    value: index as f64,
                    label: crate::render::fit_width_with(category, budget, ellipsis),
                })
                .collect(),
            step: Some(1.0),
        }
    }

    /// Places decade ticks (`10²`-style labels) over the positive range
    /// `[min, max]`, striding decades when there are many more than `target`.
    ///
    /// Ranges narrower than one full decade fall back to [`Ticks::linear`] —
    /// value labels read better than fractional powers there.
    ///
    /// # Panics
    ///
    /// Panics if the bounds are not finite and positive.
    pub fn log10(min: f64, max: f64, target: usize) -> Ticks {
        assert!(
            min.is_finite() && max.is_finite() && min > 0.0 && max > 0.0,
            "Ticks::log10 requires finite positive bounds, got {min} and {max}"
        );
        let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
        let target = target.max(2);
        let first = lo.log10().ceil() as i32;
        let last = hi.log10().floor() as i32;
        if last - first < 1 {
            return Ticks::linear(lo, hi, target);
        }
        let decades = (last - first + 1) as usize;
        let stride = decades.div_ceil(target).max(1) as i32;
        let ticks: Vec<Tick> = (first..=last)
            .step_by(stride as usize)
            .map(|power| Tick {
                value: 10f64.powi(power),
                label: power_of_ten_label(power),
            })
            .collect();
        Ticks { ticks, step: None }
    }

    /// Builds a set from precomputed ticks (time axes build these); no uniform step.
    pub(crate) fn from_time(ticks: Vec<Tick>) -> Ticks {
        Ticks { ticks, step: None }
    }

    /// The ticks, ascending.
    pub fn as_slice(&self) -> &[Tick] {
        &self.ticks
    }

    /// Iterates over the ticks, ascending.
    pub fn iter(&self) -> std::slice::Iter<'_, Tick> {
        self.ticks.iter()
    }

    /// The number of ticks; at least 1.
    pub fn len(&self) -> usize {
        self.ticks.len()
    }

    /// Whether there are no ticks. Never true for values produced by this crate.
    pub fn is_empty(&self) -> bool {
        self.ticks.is_empty()
    }

    /// The spacing between adjacent ticks in data coordinates, or `None` when there
    /// is no single spacing: a lone tick, or the non-uniform ticks of a log or time
    /// axis. Returned as an option rather than a sentinel `0`, which a caller could
    /// mistake for a real spacing.
    pub fn step(&self) -> Option<f64> {
        self.step
    }
}

impl<'a> IntoIterator for &'a Ticks {
    type Item = &'a Tick;
    type IntoIter = std::slice::Iter<'a, Tick>;

    fn into_iter(self) -> Self::IntoIter {
        self.ticks.iter()
    }
}

/// A labeling candidate: `count` values at `(start + t * skip) * step_mantissa`,
/// scaled by `10^exp10`.
struct Candidate {
    start: i128,
    skip: i128,
    step_mantissa: i128,
    exp10: i32,
    count: usize,
}

/// Search caps. The score-based pruning below terminates the loops on its own for
/// sane inputs; the caps are defensive bounds so the library can never spin.
const MAX_SKIP: i128 = 20;
const MAX_MAGNITUDE_STEPS: i32 = 60;

/// Start indices beyond this magnitude would overflow the exact-decimal mantissa;
/// such ranges (huge value, tiny span) fall back to plain endpoint ticks.
const MAX_INDEX: f64 = 1e14;

fn search(dmin: f64, dmax: f64, target: usize) -> Option<Candidate> {
    let range = dmax - dmin;
    // A span so wide it overflows to infinity has no nice ticks; fall back to the
    // endpoints rather than driving the magnitude search into non-finite arithmetic.
    if !range.is_finite() {
        return None;
    }
    let target_f = target as f64;
    // Ten times the target covers the useful density range; cap it absolutely so an
    // extreme target cannot blow the search up (no axis wants hundreds of ticks).
    let count_cap = target.saturating_mul(10).saturating_add(10).min(200);
    let mut best: Option<Candidate> = None;
    let mut best_score = f64::NEG_INFINITY;

    'search: for skip in 1..=MAX_SKIP {
        let skip_f = skip as f64;
        for (rank, &(step_mantissa, step_value)) in STEPS.iter().enumerate() {
            let s_max = simplicity_max(rank, skip_f);
            if score(s_max, 1.0, 1.0) <= best_score {
                // Simplicity only decreases for later mantissas and larger skips.
                break 'search;
            }
            for count in 2..=count_cap {
                let d_max = density_max(count, target_f);
                if score(s_max, 1.0, d_max) <= best_score {
                    break;
                }
                let delta = range / (count as f64 + 1.0) / (skip_f * step_value);
                let z0 = delta.log10().ceil() as i32;
                for dz in 0..MAX_MAGNITUDE_STEPS {
                    let z = z0 + dz;
                    let step = skip_f * step_value * 10f64.powi(z);
                    let span = step * (count as f64 - 1.0);
                    let c_max = coverage_max(dmin, dmax, span);
                    if score(s_max, c_max, d_max) <= best_score {
                        break;
                    }
                    let min_start = (dmax / step).floor() * skip_f - (count as f64 - 1.0) * skip_f;
                    let max_start = (dmin / step).ceil() * skip_f;
                    if !(min_start.abs() <= MAX_INDEX && max_start.abs() <= MAX_INDEX) {
                        continue;
                    }
                    if min_start > max_start {
                        continue;
                    }
                    // The useful grid offsets span at most a few tick counts; a
                    // window wider than that means a step so fine the candidate is
                    // hopeless anyway. Skip it rather than walk 10^14 start indices.
                    if max_start - min_start > 4.0 * count as f64 {
                        continue;
                    }
                    let unit = step_value * 10f64.powi(z);
                    for start in (min_start as i128)..=(max_start as i128) {
                        let l_min = start as f64 * unit;
                        let l_max = l_min + span;
                        let s = simplicity(rank, skip_f, zero_included(start, skip, count));
                        let c = coverage(dmin, dmax, l_min, l_max);
                        let d = density(count, target_f, dmin, dmax, l_min, l_max);
                        let total = score_full(s, c, d, 1.0);
                        if total > best_score {
                            best_score = total;
                            best = Some(Candidate {
                                start,
                                skip,
                                step_mantissa,
                                exp10: z - 1,
                                count,
                            });
                        }
                    }
                }
            }
        }
    }
    best
}

fn materialize(candidate: &Candidate) -> Ticks {
    let mut mantissas: Vec<i128> = (0..candidate.count)
        .map(|t| (candidate.start + t as i128 * candidate.skip) * candidate.step_mantissa)
        .collect();
    let mut exp10 = candidate.exp10;
    while exp10 < 0 && mantissas.iter().all(|mantissa| mantissa % 10 == 0) {
        for mantissa in &mut mantissas {
            *mantissa /= 10;
        }
        exp10 += 1;
    }
    let prefix = si_prefix(&mantissas, exp10);
    let ticks: Vec<Tick> = mantissas
        .iter()
        .map(|&mantissa| Tick {
            value: value_of(mantissa, exp10),
            label: match prefix {
                Some((shift, suffix)) if mantissa != 0 => {
                    format!("{}{suffix}", format::decimal(mantissa, exp10 - shift))
                }
                // On a prefixed axis zero is deliberately bare.
                Some(_) => "0".to_string(),
                None => format::decimal(mantissa, exp10),
            },
        })
        .collect();
    // Computed from the integer mantissa difference, so the step itself is
    // decimal-exact (0.8, never 0.8000000000000003).
    let step = value_of(mantissas[1] - mantissas[0], exp10);
    Ticks {
        ticks,
        step: Some(step),
    }
}

/// Formats `10^power` with Unicode superscripts: `1`, `10`, `10²`, `10⁻³`.
fn power_of_ten_label(power: i32) -> String {
    const DIGITS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    match power {
        0 => "1".to_string(),
        1 => "10".to_string(),
        _ => {
            let mut label = String::from("10");
            if power < 0 {
                label.push('⁻');
            }
            let mut digits = Vec::new();
            let mut remaining = power.unsigned_abs();
            while remaining > 0 {
                digits.push(DIGITS[(remaining % 10) as usize]);
                remaining /= 10;
            }
            label.extend(digits.into_iter().rev());
            label
        }
    }
}

/// Chooses one SI prefix for a whole axis, from the magnitude of its largest tick:
/// engaged at ten thousand and up (`k`, `M`, `G`, `T`) or below a thousandth
/// (`µ`, `n`, `p`). Zero keeps its bare label. The numeric part of a prefixed label
/// times the prefix factor equals the tick value exactly.
fn si_prefix(mantissas: &[i128], exp10: i32) -> Option<(i32, char)> {
    let max = mantissas.iter().map(|m| m.unsigned_abs()).max()?;
    if max == 0 {
        return None;
    }
    let digits = max.to_string().len() as i32;
    let magnitude = digits - 1 + exp10;
    if magnitude < 4 && magnitude > -4 {
        return None;
    }
    let shift = (3 * magnitude.div_euclid(3)).clamp(-12, 12);
    let suffix = match shift {
        3 => 'k',
        6 => 'M',
        9 => 'G',
        12 => 'T',
        -6 => '\u{00B5}',
        -9 => 'n',
        -12 => 'p',
        _ => return None,
    };
    Some((shift, suffix))
}

/// Converts `mantissa * 10^exp10` to the nearest `f64`.
///
/// For `|exp10| <= 22` the power of ten is exact and the single multiplication or
/// division rounds correctly, so the result equals what parsing the decimal label
/// produces.
fn value_of(mantissa: i128, exp10: i32) -> f64 {
    let m = mantissa as f64;
    let e = exp10.unsigned_abs() as usize;
    match POW10.get(e) {
        Some(&p) if exp10 >= 0 => m * p,
        Some(&p) => m / p,
        None => m * 10f64.powi(exp10),
    }
}

fn zero_included(start: i128, skip: i128, count: usize) -> bool {
    start <= 0 && start + (count as i128 - 1) * skip >= 0 && start.rem_euclid(skip) == 0
}

fn simplicity(rank: usize, skip: f64, zero: bool) -> f64 {
    let n = (STEPS.len() - 1) as f64;
    1.0 - rank as f64 / n - skip + if zero { 1.0 } else { 0.0 }
}

fn simplicity_max(rank: usize, skip: f64) -> f64 {
    let n = (STEPS.len() - 1) as f64;
    1.0 - rank as f64 / n - skip + 1.0
}

fn coverage(dmin: f64, dmax: f64, l_min: f64, l_max: f64) -> f64 {
    let range = dmax - dmin;
    let over = (dmax - l_max).powi(2) + (dmin - l_min).powi(2);
    1.0 - 0.5 * over / (0.1 * range).powi(2)
}

fn coverage_max(dmin: f64, dmax: f64, span: f64) -> f64 {
    let range = dmax - dmin;
    if span > range {
        let half = (span - range) / 2.0;
        1.0 - half.powi(2) / (0.1 * range).powi(2)
    } else {
        1.0
    }
}

fn density(count: usize, target: f64, dmin: f64, dmax: f64, l_min: f64, l_max: f64) -> f64 {
    let r = (count as f64 - 1.0) / (l_max - l_min);
    let rt = (target - 1.0) / (dmax.max(l_max) - dmin.min(l_min));
    2.0 - (r / rt).max(rt / r)
}

fn density_max(count: usize, target: f64) -> f64 {
    if count as f64 >= target {
        2.0 - (count as f64 - 1.0) / (target - 1.0)
    } else {
        1.0
    }
}

/// Upper-bound score used for pruning; legibility is at its maximum of 1.
fn score(simplicity: f64, coverage: f64, density: f64) -> f64 {
    score_full(simplicity, coverage, density, 1.0)
}

fn score_full(simplicity: f64, coverage: f64, density: f64, legibility: f64) -> f64 {
    WEIGHTS[0] * simplicity + WEIGHTS[1] * coverage + WEIGHTS[2] * density + WEIGHTS[3] * legibility
}

#[cfg(test)]
#[path = "tests/ticks_tests.rs"]
mod tests;
