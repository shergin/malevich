//! A decade of Mauna Loa CO₂ (NOAA), months down, years across — ninety-six
//! numbers in one stat table. Each year-column is formatted and colored on its
//! own scale: `table_with`'s colormap positions every value within its
//! column's extent, so the seasonal swing — the May crest, the September
//! trough — repeats down each column in color where the terminal has any,
//! while the year-over-year rise reads across every row in the digits
//! themselves. The numbers survive any pipe; the color is a second reading,
//! not the only one.

use malevich::Frame;
use malevich::scale::Colormap;

fn main() {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    const YEARS: std::ops::Range<i32> = 2018..2026;

    // Row-major month × year, gaps where a month is missing.
    let years: Vec<String> = YEARS.map(|year| year.to_string()).collect();
    let mut ppm = vec![f64::NAN; 12 * years.len()];
    for line in include_str!("data/co2_monthly.csv").lines().skip(1) {
        let mut parts = line.split(',');
        let year: Option<i32> = parts.next().and_then(|v| v.parse().ok());
        let month: Option<usize> = parts.next().and_then(|v| v.parse().ok());
        let value: Option<f64> = parts.next().and_then(|v| v.parse().ok());
        if let (Some(year), Some(month @ 1..=12), Some(value)) = (year, month, value)
            && YEARS.contains(&year)
        {
            ppm[(month - 1) * years.len() + (year - YEARS.start) as usize] = value;
        }
    }

    let chart = malevich::table_with(
        MONTHS,
        &years,
        &ppm[..],
        malevich::TableOptions::new().colormap(Colormap::VIRIDIS),
    )
    .expect("twelve months of complete years")
    .title("Mauna Loa CO\u{2082}, monthly mean ppm");
    println!("{}", chart.render(&Frame::plain(63, 15)));
}
