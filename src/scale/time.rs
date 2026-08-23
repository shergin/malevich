//! Time ticks: calendar-aware placement and multi-scale labels over unix seconds.
//!
//! Time on an axis is unix seconds (UTC) as `f64`. Ticks come from a fixed interval
//! ladder — 1/5/15/30 seconds and minutes, 1/3/6/12 hours, days, weeks, months,
//! years — aligned to calendar boundaries, with labels that show the largest unit
//! that matters: `14:05`, but `Aug 2` at midnight, and `2027` at January. Calendar
//! math is exact Gregorian arithmetic (Howard Hinnant's civil-date algorithms), no
//! dependencies, UTC only.

use super::ticks::{Tick, Ticks};

const MINUTE: i64 = 60;
const HOUR: i64 = 3_600;
const DAY: i64 = 86_400;
const MIN_CALENDAR_YEAR: i32 = -999_999;
const MAX_CALENDAR_YEAR: i32 = 999_999;
const MAX_TIME_TARGET: usize = 200;
const MAX_TIME_TICKS: usize = 512;
const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// One rung of the interval ladder.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Interval {
    /// A fixed number of seconds, aligned to multiples since the epoch.
    Seconds(i64),
    /// Whole weeks, aligned to Mondays.
    Weeks(i64),
    /// Whole months, aligned to month starts.
    Months(i64),
    /// Whole years, aligned to January firsts.
    Years(i64),
}

impl Interval {
    fn approximate_seconds(self) -> f64 {
        match self {
            Interval::Seconds(s) => s as f64,
            Interval::Weeks(w) => (w * 7 * DAY) as f64,
            Interval::Months(m) => m as f64 * 30.44 * DAY as f64,
            Interval::Years(y) => y as f64 * 365.25 * DAY as f64,
        }
    }
}

/// The ladder, densest first.
fn ladder() -> impl Iterator<Item = Interval> {
    let fixed = [
        1,
        5,
        15,
        30,
        MINUTE,
        5 * MINUTE,
        15 * MINUTE,
        30 * MINUTE,
        HOUR,
        3 * HOUR,
        6 * HOUR,
        12 * HOUR,
        DAY,
        2 * DAY,
    ]
    .into_iter()
    .map(Interval::Seconds);
    let calendar = [
        Interval::Weeks(1),
        Interval::Months(1),
        Interval::Months(3),
        Interval::Months(6),
        Interval::Years(1),
    ]
    .into_iter();
    // Beyond single years: 2, 5, 10, 20, 50, … years. The supported calendar
    // range needs no step beyond ten million years; keeping the ladder finite
    // also makes integer overflow impossible.
    let years = (0..=6).flat_map(|magnitude: u32| {
        let power = 10i64.pow(magnitude);
        [2i64, 5, 10]
            .into_iter()
            .filter_map(move |base| base.checked_mul(power).map(Interval::Years))
    });
    fixed.chain(calendar).chain(years)
}

impl Ticks {
    /// Places about `target` calendar-aligned ticks over `[min, max]` unix seconds
    /// (UTC), with multi-scale labels.
    ///
    /// Calendar labels support years -999999 through 999999. Finite timestamps
    /// outside that range fall back to numeric endpoint labels. Targets above 200
    /// are capped, and generation has an independent 512-tick safety bound.
    ///
    /// # Panics
    ///
    /// Panics if the bounds are not finite.
    pub fn time(min: f64, max: f64, target: usize) -> Ticks {
        assert!(
            min.is_finite() && max.is_finite(),
            "Ticks::time requires finite bounds, got {min} and {max}"
        );
        let (lo, hi) = if min <= max { (min, max) } else { (max, min) };
        let target = target.clamp(2, MAX_TIME_TARGET);
        let Some((lo_seconds, hi_seconds)) = calendar_seconds(lo, hi) else {
            return numeric_fallback(lo, hi);
        };
        if lo == hi {
            return calendar_point(lo, lo_seconds);
        }
        let span = (hi - lo).max(1.0);
        let interval = ladder()
            .find(|interval| span / interval.approximate_seconds() <= target as f64)
            .unwrap_or(Interval::Years(10_000_000));
        let stamps = generate(lo_seconds, hi_seconds, interval);
        if stamps.is_empty() {
            return calendar_point(lo, lo_seconds);
        }
        let ticks = label(&stamps, interval);
        Ticks::from_time(ticks)
    }
}

fn supported_seconds() -> (i64, i64) {
    let first = days_from_civil(MIN_CALENDAR_YEAR, 1, 1) * DAY;
    let last = days_from_civil(MAX_CALENDAR_YEAR, 12, 31) * DAY + DAY - 1;
    (first, last)
}

fn calendar_seconds(lo: f64, hi: f64) -> Option<(i64, i64)> {
    let (supported_lo, supported_hi) = supported_seconds();
    let lo = lo.floor();
    let hi = hi.ceil();
    if lo < supported_lo as f64 || hi > supported_hi as f64 {
        return None;
    }
    Some((lo as i64, hi as i64))
}

fn numeric_fallback(lo: f64, hi: f64) -> Ticks {
    let mut ticks = vec![Tick {
        value: lo,
        label: lo.to_string(),
    }];
    if hi != lo {
        ticks.push(Tick {
            value: hi,
            label: hi.to_string(),
        });
    }
    Ticks::from_time(ticks)
}

fn calendar_point(value: f64, stamp: i64) -> Ticks {
    let mut tick = label(&[stamp], Interval::Seconds(1))
        .into_iter()
        .next()
        .unwrap_or(Tick {
            value,
            label: value.to_string(),
        });
    tick.value = value;
    Ticks::from_time(vec![tick])
}

/// Tick timestamps for `interval` covering `[lo, hi]`, calendar-aligned.
fn generate(lo: i64, hi: i64, interval: Interval) -> Vec<i64> {
    let mut stamps = Vec::new();
    match interval {
        Interval::Seconds(step) => {
            if let Some(t) = aligned_at_or_after(lo, step, 0) {
                push_fixed(&mut stamps, t, hi, step);
            }
        }
        Interval::Weeks(weeks) => {
            let Some(step) = weeks
                .checked_mul(7)
                .and_then(|value| value.checked_mul(DAY))
            else {
                return stamps;
            };
            // The epoch was a Thursday; day 4 after it was the first Monday.
            let monday_offset = 4 * DAY;
            if let Some(t) = aligned_at_or_after(lo, step, monday_offset) {
                push_fixed(&mut stamps, t, hi, step);
            }
        }
        Interval::Months(step) => {
            let (mut year, mut month, _) = civil_from_days(lo.div_euclid(DAY));
            // Round the month down to a multiple of the step within the year.
            month = ((month - 1) / step as u32 * step as u32) + 1;
            while stamps.len() < MAX_TIME_TICKS {
                let Some(t) = days_from_civil(year, month, 1).checked_mul(DAY) else {
                    break;
                };
                if t > hi {
                    break;
                }
                if t >= lo {
                    stamps.push(t);
                }
                let advanced = (month as i64 - 1) + step;
                let Ok(year_delta) = i32::try_from(advanced / 12) else {
                    break;
                };
                let Some(next_year) = year.checked_add(year_delta) else {
                    break;
                };
                year = next_year;
                month = (advanced % 12) as u32 + 1;
            }
        }
        Interval::Years(step) => {
            let (year, ..) = civil_from_days(lo.div_euclid(DAY));
            let Some(mut year) = i64::from(year).div_euclid(step).checked_mul(step) else {
                return stamps;
            };
            while stamps.len() < MAX_TIME_TICKS {
                let Ok(civil_year) = i32::try_from(year) else {
                    break;
                };
                let Some(t) = days_from_civil(civil_year, 1, 1).checked_mul(DAY) else {
                    break;
                };
                if t > hi {
                    break;
                }
                if t >= lo {
                    stamps.push(t);
                }
                let Some(next_year) = year.checked_add(step) else {
                    break;
                };
                year = next_year;
            }
        }
    }
    stamps
}

fn aligned_at_or_after(lo: i64, step: i64, offset: i64) -> Option<i64> {
    let shifted = lo.checked_sub(offset)?;
    let aligned = shifted
        .div_euclid(step)
        .checked_mul(step)?
        .checked_add(offset)?;
    if aligned < lo {
        aligned.checked_add(step)
    } else {
        Some(aligned)
    }
}

fn push_fixed(stamps: &mut Vec<i64>, mut value: i64, hi: i64, step: i64) {
    while value <= hi && stamps.len() < MAX_TIME_TICKS {
        stamps.push(value);
        let Some(next) = value.checked_add(step) else {
            break;
        };
        value = next;
    }
}

/// Multi-scale labels: each tick shows its interval's unit, except where a larger
/// unit rolls over — midnight shows the date, January shows the year.
fn label(stamps: &[i64], interval: Interval) -> Vec<Tick> {
    stamps
        .iter()
        .map(|&t| {
            let days = t.div_euclid(DAY);
            let second_of_day = t.rem_euclid(DAY);
            let (year, month, day) = civil_from_days(days);
            let (hour, minute, second) = (
                second_of_day / HOUR,
                (second_of_day % HOUR) / MINUTE,
                second_of_day % MINUTE,
            );
            let month_name = MONTHS[(month - 1) as usize];
            let label = match interval {
                Interval::Seconds(step) if step < MINUTE => {
                    format!("{hour:02}:{minute:02}:{second:02}")
                }
                Interval::Seconds(step) if step < DAY => {
                    if second_of_day == 0 {
                        format!("{month_name} {day}")
                    } else {
                        format!("{hour:02}:{minute:02}")
                    }
                }
                Interval::Seconds(_) | Interval::Weeks(_) => {
                    if month == 1 && day == 1 {
                        format!("{year}")
                    } else {
                        format!("{month_name} {day}")
                    }
                }
                Interval::Months(_) => {
                    if month == 1 {
                        format!("{year}")
                    } else {
                        month_name.to_string()
                    }
                }
                Interval::Years(_) => format!("{year}"),
            };
            Tick {
                value: t as f64,
                label,
            }
        })
        .collect()
}

/// Hinnant's `civil_from_days`: days since 1970-01-01 to `(year, month, day)`.
fn civil_from_days(days: i64) -> (i32, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = (z - era * 146_097) as u64;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era as i64 + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let mp = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * mp + 2) / 5 + 1) as u32;
    let month = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    ((year + i64::from(month <= 2)) as i32, month, day)
}

/// Hinnant's `days_from_civil`: `(year, month, day)` to days since 1970-01-01.
fn days_from_civil(year: i32, month: u32, day: u32) -> i64 {
    let year = i64::from(year) - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = (year - era * 400) as u64;
    let month = u64::from(month);
    let day_of_year =
        (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + u64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era as i64 - 719_468
}

#[cfg(test)]
#[path = "tests/time_tests.rs"]
mod tests;
