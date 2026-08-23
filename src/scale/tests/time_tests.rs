use super::super::Ticks;
use super::{MAX_TIME_TICKS, civil_from_days, days_from_civil, supported_seconds};

const DAY: i64 = 86_400;

fn labels(ticks: &Ticks) -> Vec<&str> {
    ticks.iter().map(|tick| tick.label.as_str()).collect()
}

#[test]
fn civil_dates_roundtrip_across_eras_and_leap_years() {
    for &(year, month, day) in &[
        (1970, 1, 1),
        (2000, 2, 29),
        (2024, 2, 29),
        (2026, 8, 2),
        (1969, 12, 31),
        (1900, 3, 1),
        (2100, 1, 1),
    ] {
        let days = days_from_civil(year, month, day);
        assert_eq!(civil_from_days(days), (year, month, day));
    }
    assert_eq!(days_from_civil(1970, 1, 1), 0);
}

#[test]
fn hour_ticks_show_the_date_at_midnight() {
    let start = days_from_civil(2026, 8, 1) * DAY + 12 * 3_600;
    let end = days_from_civil(2026, 8, 2) * DAY + 4 * 3_600;
    let ticks = Ticks::time(start as f64, end as f64, 6);
    assert_eq!(
        labels(&ticks),
        ["12:00", "15:00", "18:00", "21:00", "Aug 2", "03:00"]
    );
}

#[test]
fn day_ticks_show_the_year_at_january_first() {
    let start = days_from_civil(2026, 12, 29) * DAY;
    let end = days_from_civil(2027, 1, 3) * DAY;
    let ticks = Ticks::time(start as f64, end as f64, 7);
    assert_eq!(
        labels(&ticks),
        ["Dec 29", "Dec 30", "Dec 31", "2027", "Jan 2", "Jan 3"]
    );
}

#[test]
fn month_ticks_quarter_and_mark_the_new_year() {
    let start = days_from_civil(2026, 1, 1) * DAY;
    let end = days_from_civil(2027, 6, 1) * DAY;
    let ticks = Ticks::time(start as f64, end as f64, 8);
    assert_eq!(labels(&ticks), ["2026", "Apr", "Jul", "Oct", "2027", "Apr"]);
}

#[test]
fn decade_spans_use_round_years() {
    let start = days_from_civil(1988, 6, 1) * DAY;
    let end = days_from_civil(2026, 1, 1) * DAY;
    let ticks = Ticks::time(start as f64, end as f64, 5);
    assert_eq!(labels(&ticks), ["1990", "2000", "2010", "2020"]);
}

#[test]
fn week_ticks_land_on_mondays() {
    let start = days_from_civil(2026, 7, 1) * DAY;
    let end = days_from_civil(2026, 8, 20) * DAY;
    let ticks = Ticks::time(start as f64, end as f64, 8);
    for tick in &ticks {
        let days = (tick.value as i64).div_euclid(DAY);
        assert_eq!((days - 4).rem_euclid(7), 0, "not a Monday: {}", tick.label);
    }
    assert!(ticks.len() >= 5);
}

#[test]
fn second_ticks_show_full_clock_time() {
    let start = days_from_civil(2026, 8, 2) * DAY + 3_600;
    let ticks = Ticks::time(start as f64, start as f64 + 90.0, 6);
    assert_eq!(ticks.as_slice()[0].label, "01:00:00");
}

#[test]
fn finite_values_outside_the_calendar_range_fall_back_without_panicking() {
    let one = Ticks::time(f64::MAX, f64::MAX, 5);
    assert_eq!(one.len(), 1);
    assert_eq!(one.as_slice()[0].value, f64::MAX);

    let both = Ticks::time(-f64::MAX, f64::MAX, usize::MAX);
    assert_eq!(both.len(), 2);
    assert_eq!(both.as_slice()[0].value, -f64::MAX);
    assert_eq!(both.as_slice()[1].value, f64::MAX);
}

#[test]
fn supported_calendar_generation_is_bounded_for_hostile_targets() {
    let (low, high) = supported_seconds();
    let ticks = Ticks::time(low as f64, high as f64, usize::MAX);
    assert!(!ticks.is_empty());
    assert!(ticks.len() <= MAX_TIME_TICKS);
    assert!(ticks.iter().all(|tick| tick.value.is_finite()));
}

#[test]
fn subsecond_domains_still_produce_ticks() {
    let ticks = Ticks::time(0.1, 0.2, 5);
    assert!(!ticks.is_empty());
    assert!(ticks.len() <= MAX_TIME_TICKS);
}
