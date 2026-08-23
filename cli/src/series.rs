//! Column semantics: rows of fields into plottable series (D-C6).
//!
//! A field that will not parse as a number becomes `NaN` — the honest gap the
//! library already draws — and is counted so the caller can print the one-line
//! tally. A field that is simply absent (a short row) is a structural gap, not a
//! parse failure, so it becomes `NaN` without adding to the tally. With `--time-x`,
//! the x column is parsed as a timestamp instead (see [`crate::time`]).

use std::collections::HashMap;

use crate::args::Fmt;
use crate::input::Table;

/// An index into a [`Dataset`]'s unique parsed channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Channel(usize);

impl Channel {
    /// Position in the dataset's unique channel store.
    pub fn index(self) -> usize {
        self.0
    }
}

/// One plottable series: channel references plus an optional label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Series {
    pub x: Option<Channel>,
    pub y: Channel,
    pub label: Option<String>,
}

/// Unique parsed columns, the series that reference them, and the parse tally.
/// Shared x channels (`xyy`) occupy one buffer regardless of series count.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Dataset {
    channels: Vec<Vec<f64>>,
    pub series: Vec<Series>,
    pub unparsed: usize,
}

impl Dataset {
    /// Every unique channel, in first input-column order.
    pub fn channels(&self) -> &[Vec<f64>] {
        &self.channels
    }

    /// Resolves a lightweight channel reference to its parsed values.
    pub fn channel(&self, channel: Channel) -> &[f64] {
        &self.channels[channel.index()]
    }

    /// The explicit x values for `series`, or `None` for row indices.
    pub fn x(&self, series: &Series) -> Option<&[f64]> {
        series.x.map(|channel| self.channel(channel))
    }

    /// The y values for `series`.
    pub fn y(&self, series: &Series) -> &[f64] {
        self.channel(series.y)
    }
}

/// The default column mapping when `--fmt` is unset: a lone column is a y-series
/// over its index; two or more columns are x plus y-series sharing it.
pub fn default_fmt(columns: usize) -> Fmt {
    if columns <= 1 { Fmt::Y } else { Fmt::Xyy }
}

/// Parses one field as a number, tolerating surrounding whitespace. Non-finite
/// spellings (`inf`, `nan`) are treated as gaps, not values — they carry no
/// position, and silently plotting them would lie.
fn parse_number(field: &str) -> Option<f64> {
    let trimmed = field.trim();
    if trimmed.is_empty() {
        return None;
    }
    trimmed
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite())
}

/// Builds column `index` from the raw rows, parsing each present field — as a
/// timestamp when `time`, otherwise a number. Returns the channel and the count of
/// present-but-unparseable fields (absent fields are structural gaps, uncounted).
fn build_column(table: &Table, index: usize, time: bool) -> (Vec<f64>, usize) {
    let mut out = Vec::with_capacity(table.rows.len());
    let mut unparsed = 0;
    for row in &table.rows {
        match row.get(index) {
            Some(field) => {
                let parsed = if time {
                    crate::time::parse(field)
                } else {
                    parse_number(field)
                };
                match parsed {
                    Some(value) => out.push(value),
                    None => {
                        unparsed += 1;
                        out.push(f64::NAN);
                    }
                }
            }
            None => out.push(f64::NAN),
        }
    }
    (out, unparsed)
}

/// All columns, column-major, squaring up ragged rows with `NaN`.
fn numeric_columns(table: &Table) -> (Vec<Vec<f64>>, usize) {
    let mut columns = Vec::with_capacity(table.width());
    let mut unparsed = 0;
    for index in 0..table.width() {
        let (column, count) = build_column(table, index, false);
        unparsed += count;
        columns.push(column);
    }
    (columns, unparsed)
}

/// The header name for column `index`, if the table carried one.
fn label(table: &Table, index: usize) -> Option<String> {
    table
        .header
        .as_ref()
        .and_then(|names| names.get(index))
        .cloned()
}

/// Which input columns feed one series: `x` (row index when `None`), `y`, and the
/// column whose header names the series.
struct Spec {
    x: Option<usize>,
    y: usize,
    label: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParseAs {
    Number,
    Time,
}

/// The per-series column roles for a `--fmt`.
fn specs(fmt: Fmt, width: usize) -> Vec<Spec> {
    match fmt {
        Fmt::Y => (0..width)
            .map(|i| Spec {
                x: None,
                y: i,
                label: i,
            })
            .collect(),
        Fmt::Xy if width >= 2 => vec![Spec {
            x: Some(0),
            y: 1,
            label: 1,
        }],
        Fmt::Xyy if width >= 2 => (1..width)
            .map(|i| Spec {
                x: Some(0),
                y: i,
                label: i,
            })
            .collect(),
        Fmt::Xyxy => (0..width / 2)
            .map(|pair| Spec {
                x: Some(pair * 2),
                y: pair * 2 + 1,
                label: pair * 2 + 1,
            })
            .collect(),
        Fmt::Yx if width >= 2 => vec![Spec {
            x: Some(1),
            y: 0,
            label: 0,
        }],
        _ => Vec::new(),
    }
}

/// Maps columns onto series per `fmt`, parsing x columns as time when `time_x`.
pub fn dataset(table: &Table, fmt: Fmt, time_x: bool) -> Dataset {
    let specs = specs(fmt, table.width());
    // Record each referenced column's one parsing role. No current format uses
    // one column as both time and numeric data.
    let mut roles = vec![None; table.width()];
    for spec in &specs {
        insert_role(&mut roles, spec.y, ParseAs::Number);
        if let Some(x) = spec.x {
            let role = if time_x {
                ParseAs::Time
            } else {
                ParseAs::Number
            };
            insert_role(&mut roles, x, role);
        }
    }

    let mut channels = Vec::with_capacity(roles.iter().flatten().count());
    let mut channel_for = vec![None; table.width()];
    let mut unparsed = 0;
    for (index, role) in roles.into_iter().enumerate() {
        if let Some(role) = role {
            let (column, count) = build_column(table, index, role == ParseAs::Time);
            channel_for[index] = Some(Channel(channels.len()));
            channels.push(column);
            unparsed += count;
        }
    }
    let series = specs
        .iter()
        .map(|spec| Series {
            x: spec.x.map(|index| {
                channel_for[index].expect("every x role was assigned a parsed channel")
            }),
            y: channel_for[spec.y].expect("every y role was assigned a parsed channel"),
            label: label(table, spec.label),
        })
        .collect();
    Dataset {
        channels,
        series,
        unparsed,
    }
}

fn insert_role(roles: &mut [Option<ParseAs>], index: usize, role: ParseAs) {
    match roles[index] {
        Some(existing) => debug_assert_eq!(existing, role),
        None => roles[index] = Some(role),
    }
}

/// Resolves the effective `--fmt` for a value-shaped chart (line, scatter),
/// applying the per-column-count default when none was given.
pub fn resolve_fmt(table: &Table, requested: Option<Fmt>) -> Fmt {
    requested.unwrap_or_else(|| default_fmt(table.width()))
}

/// Every numeric field, flattened into one sample set — the input shape for the
/// distribution charts (hist, density, ecdf), which pool all values.
pub fn flatten(table: &Table) -> (Vec<f64>, usize) {
    let mut values = Vec::new();
    let mut unparsed = 0;
    for row in &table.rows {
        for field in row {
            match parse_number(field) {
                Some(value) => values.push(value),
                None => unparsed += 1,
            }
        }
    }
    (values, unparsed)
}

/// Each column as a group of finite samples, named by its header (else its 1-based
/// position) — the input shape for box and violin (columns are groups).
pub fn groups(table: &Table) -> (Vec<String>, Vec<Vec<f64>>, usize) {
    let (columns, unparsed) = numeric_columns(table);
    let categories = (0..columns.len())
        .map(|index| label(table, index).unwrap_or_else(|| (index + 1).to_string()))
        .collect();
    // Box and violin summarize finite samples; NaN gaps carry nothing here.
    let groups = columns
        .into_iter()
        .map(|column| {
            column
                .into_iter()
                .filter(|value| value.is_finite())
                .collect()
        })
        .collect();
    (categories, groups, unparsed)
}

/// The first two columns as x and y — the input shape for a 2D histogram. The x
/// column parses as time under `time_x`.
pub fn xy(table: &Table, time_x: bool) -> (Vec<f64>, Vec<f64>, usize) {
    let (x, ux) = build_column(table, 0, time_x);
    let (y, uy) = build_column(table, 1, false);
    (x, y, ux + uy)
}

/// The rows as a row-major grid for a heatmap: the column count, the values with
/// row 0 at the bottom, and the unparsed tally. Input rows are flipped so a matrix
/// typed top-to-bottom appears the same way.
pub fn matrix(table: &Table) -> (usize, Vec<f64>, usize) {
    let (columns, unparsed) = numeric_columns(table);
    let cols = columns.len();
    let rows = table.rows.len();
    let mut values = Vec::with_capacity(cols * rows);
    for heat_row in 0..rows {
        // Heatmap row 0 is the bottom; the input's last line lands there.
        let source = rows - 1 - heat_row;
        for column in &columns {
            values.push(column[source]);
        }
    }
    (cols, values, unparsed)
}

/// `label value` rows for a bar chart: the first field labels the bar, the second
/// is its height. A missing height is a gap; an unparseable one is counted.
pub fn labeled_values(table: &Table) -> (Vec<String>, Vec<f64>, usize) {
    let mut labels = Vec::with_capacity(table.rows.len());
    let mut values = Vec::with_capacity(table.rows.len());
    let mut unparsed = 0;
    for row in &table.rows {
        let Some(name) = row.first() else { continue };
        labels.push(name.clone());
        match row.get(1) {
            Some(field) => match parse_number(field) {
                Some(value) => values.push(value),
                None => {
                    unparsed += 1;
                    values.push(f64::NAN);
                }
            },
            None => values.push(f64::NAN),
        }
    }
    (labels, values, unparsed)
}

/// Frequency counts of the first field of each row, most frequent first, ties
/// broken by label — the `count` chart's one bit of CLI-side shaping (D-C3).
pub fn counts(table: &Table) -> Vec<(String, f64)> {
    // Insertion-ordered accumulation keeps the first-seen order available as the
    // final tie-breaker, so equal counts stay deterministic.
    let mut order: Vec<String> = Vec::new();
    let mut tally: HashMap<&str, u64> = HashMap::new();
    for row in &table.rows {
        let Some(field) = row.first() else { continue };
        let entry = tally.entry(field.as_str()).or_insert(0);
        if *entry == 0 {
            order.push(field.clone());
        }
        *entry += 1;
    }
    let mut counts: Vec<(String, f64)> = order
        .into_iter()
        .map(|name| {
            let count = tally[name.as_str()];
            (name, count as f64)
        })
        .collect();
    counts.sort_by(|a, b| {
        (b.1)
            .partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });
    counts
}

#[cfg(test)]
#[path = "tests/series_tests.rs"]
mod tests;
