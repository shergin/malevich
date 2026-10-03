//! The Keeling curve: monthly mean CO₂ at Mauna Loa since 1958 (NOAA GML, public
//! domain — see examples/data/README.md), on a calendar axis.

use malevich::{Frame, Line, Plot};
include!("support/svg_card.rs");

fn main() {
    let (stamps, ppm): (Vec<f64>, Vec<f64>) = include_str!("data/co2_monthly.csv")
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut parts = line.split(',');
            let year: i64 = parts.next()?.parse().ok()?;
            let month: u64 = parts.next()?.parse().ok()?;
            let ppm: f64 = parts.next()?.parse().ok()?;
            Some((month_stamp(year, month), ppm))
        })
        .unzip();
    let chart = Plot::new()
        .layer(Line::xy(&stamps[..], &ppm[..]))
        .title("atmospheric CO2 at Mauna Loa (NOAA)")
        .y_label("ppm")
        .time_x();
    let frame = Frame::plain(76, 18);
    if svg_card(&chart, &frame) {
        return;
    }
    println!("{}", chart.render_best(&frame));
}

/// The first of the month as unix seconds (Hinnant's civil-date arithmetic).
fn month_stamp(year: i64, month: u64) -> f64 {
    let y = year - i64::from(month <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    ((era * 146_097 + doe as i64 - 719_468) * 86_400) as f64
}
