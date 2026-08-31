//! Exact decimal formatting for tick labels, and the shared set formatter.
//!
//! Tick values are represented as `mantissa * 10^exp10` with an integer mantissa, so
//! labels are produced by integer math alone: no binary-float artifacts (`0.30000...4`),
//! no `-0`, and a uniform number of decimals across an axis. [`NumberFormat`] makes
//! the same decisions once for an arbitrary set of values — a table column, a legend
//! readout — instead of for an axis's ticks.

/// Formats `mantissa * 10^exp10` as a plain decimal string.
///
/// For `exp10 >= 0` the result is an integer (zero is `"0"`, never `"0000"`). For
/// `exp10 < 0` the result carries exactly `-exp10` fraction digits, including for zero
/// (`"0.00"`), so that labels sharing an exponent align.
pub(crate) fn decimal(mantissa: i128, exp10: i32) -> String {
    if exp10 >= 0 {
        if mantissa == 0 {
            return "0".to_string();
        }
        let mut s = mantissa.to_string();
        s.extend(std::iter::repeat_n('0', exp10 as usize));
        return s;
    }

    let fraction_digits = exp10.unsigned_abs() as usize;
    let sign = if mantissa < 0 { "-" } else { "" };
    let digits = mantissa.unsigned_abs().to_string();
    let (integer, fraction) = if digits.len() > fraction_digits {
        let split = digits.len() - fraction_digits;
        (digits[..split].to_string(), digits[split..].to_string())
    } else {
        let padding = "0".repeat(fraction_digits - digits.len());
        ("0".to_string(), format!("{padding}{digits}"))
    };
    format!("{sign}{integer}.{fraction}")
}

/// The fixed significant-digit budget a value set is formatted at.
const SIGNIFICANT_DIGITS: i32 = 4;

/// Powers of ten that are exactly representable in `f64`.
const POW10: [f64; 23] = [
    1e0, 1e1, 1e2, 1e3, 1e4, 1e5, 1e6, 1e7, 1e8, 1e9, 1e10, 1e11, 1e12, 1e13, 1e14, 1e15, 1e16,
    1e17, 1e18, 1e19, 1e20, 1e21, 1e22,
];

/// `value * 10^exp10`, multiplying or dividing by an exact power of ten where one
/// exists, so a single correctly-rounded operation carries the scaling.
fn scale_by_pow10(value: f64, exp10: i32) -> f64 {
    let e = exp10.unsigned_abs() as usize;
    match POW10.get(e) {
        Some(&p) if exp10 >= 0 => value * p,
        Some(&p) => value / p,
        None => value * 10f64.powi(exp10),
    }
}

/// Uniform formatting for a set of related values — a table column, a readout
/// row: the decisions an axis makes once for its ticks, made once for the set.
///
/// [`NumberFormat::for_values`] derives a shared resolution from the set's
/// largest magnitude — a fixed significant-digit budget — and one SI prefix for
/// the whole set (`k`, `M`, `µ`, …), engaged at ten thousand and up or below a
/// thousandth, exactly like an axis. [`NumberFormat::format`] then renders any
/// value at that resolution as an exact decimal: every value carries the same
/// number of fraction digits, so a right-aligned column aligns at the decimal
/// point. Non-finite values format as `—`, the gap convention; zero on a
/// prefixed set is bare `0`, like a tick; an unprefixed set of whole numbers
/// keeps whole labels (a count column never reads `7.000`).
///
/// ```
/// use malevich::scale::NumberFormat;
///
/// let losses = NumberFormat::for_values(&[0.4821, 0.517, 1.104]);
/// assert_eq!(losses.format(0.4821), "0.482");
/// assert_eq!(losses.format(1.104), "1.104");
/// assert_eq!(losses.format(f64::NAN), "—");
///
/// let counts = NumberFormat::for_values(&[125_000.0, 98_500.0]);
/// assert_eq!(counts.format(125_000.0), "125.0k");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberFormat {
    /// The SI shift applied before rendering, with its suffix; `None` is unshifted.
    prefix: Option<(i32, char)>,
    /// Fraction digits every rendered value carries.
    fraction: i32,
}

impl NumberFormat {
    /// Derives the shared resolution and prefix from the finite values of the set.
    ///
    /// An empty or all-gap set formats whole numbers with no prefix.
    pub fn for_values(values: &[f64]) -> NumberFormat {
        let max_abs = values
            .iter()
            .copied()
            .filter(|value| value.is_finite())
            .fold(0.0f64, |max, value| max.max(value.abs()));
        if max_abs == 0.0 {
            return NumberFormat {
                prefix: None,
                fraction: 0,
            };
        }
        let magnitude = max_abs.log10().floor() as i32;
        let prefix = if magnitude >= 4 || magnitude <= -4 {
            let shift = (3 * magnitude.div_euclid(3)).clamp(-12, 12);
            let suffix = match shift {
                3 => Some('k'),
                6 => Some('M'),
                9 => Some('G'),
                12 => Some('T'),
                -6 => Some('\u{00B5}'),
                -9 => Some('n'),
                -12 => Some('p'),
                _ => None,
            };
            suffix.map(|suffix| (shift, suffix))
        } else {
            None
        };
        let shift = prefix.map_or(0, |(shift, _)| shift);
        // A set of whole numbers keeps whole labels — a count column never
        // reads `7.000`. Under a prefix the budget stays: `98.5k` needs it.
        let whole = prefix.is_none()
            && values
                .iter()
                .filter(|value| value.is_finite())
                .all(|value| value.fract() == 0.0);
        let fraction = if whole {
            0
        } else {
            (SIGNIFICANT_DIGITS - 1 - (magnitude - shift)).max(0)
        };
        NumberFormat { prefix, fraction }
    }

    /// Renders `value` at the set's resolution.
    ///
    /// Non-finite values are `—`. A value far outside the derived resolution's
    /// integer range falls back to Rust's plain `Display`.
    pub fn format(&self, value: f64) -> String {
        if !value.is_finite() {
            return "\u{2014}".to_string();
        }
        let shift = self.prefix.map_or(0, |(shift, _)| shift);
        let mantissa = scale_by_pow10(value, self.fraction - shift).round();
        if mantissa.abs() >= 1e30 {
            return format!("{value}");
        }
        let mantissa = mantissa as i128;
        if mantissa == 0 {
            // A prefixed zero is deliberately bare, like a tick's.
            return if self.prefix.is_some() {
                "0".to_string()
            } else {
                decimal(0, -self.fraction)
            };
        }
        let mut label = decimal(mantissa, -self.fraction);
        if let Some((_, suffix)) = self.prefix {
            label.push(suffix);
        }
        label
    }
}

#[cfg(test)]
#[path = "tests/format_tests.rs"]
mod tests;
