//! The rim: request decoding shared by the language bindings.
//!
//! A binding — the npm package's wasm module, the Python extension — owns its
//! builder and its environment detection, and hands this crate two JSON
//! strings and a column table: a versioned [`Document`] whose series may be
//! `{ "col": N }` references, and a [`Frame`]. This crate decodes them,
//! expands a crate preset by name over the columns, and packs a [`Raster`]
//! into planes a cell host paints without decoding ANSI. Nothing here reads
//! the environment, and nothing here draws: rendering stays the crate's.
//!
//! Column references exist only inside a request; a stored document never
//! carries them (see [`malevich::data::with_columns`]). The two bindings read
//! the same shared goldens, so a change here shows up in both suites.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::fmt;

use malevich::data::with_columns;
use malevich::pixel::Protocol;
use malevich::render::{Color, Raster};
use malevich::scale::Colormap;
use malevich::stat::{Bandwidth, KdeOptions, Normalization, StepDirection, Whiskers};
use malevich::{
    BoxOptions, ContourOptions, DensityOptions, DescribeOptions, Document, EcdfOptions, Frame,
    Grid, HeatmapOptions, Histogram2dOptions, HistogramOptions, Plot, StairsOptions, TableOptions,
    TrendOptions, ViolinOptions,
};

/// Why a request could not be served.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum RequestError {
    /// The document, frame, or options JSON did not decode, or the decoded
    /// spec failed validation.
    Format(String),
    /// No preset has this name.
    UnknownPreset(String),
    /// A preset needed a column the request did not carry.
    MissingColumn {
        /// The preset asking.
        preset: String,
        /// The zero-based column it asked for.
        index: usize,
    },
    /// The crate refused the input.
    Plot(malevich::Error),
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestError::Format(detail) => f.write_str(detail),
            RequestError::UnknownPreset(name) => write!(f, "unknown preset '{name}'"),
            RequestError::MissingColumn { preset, index } => {
                write!(f, "preset '{preset}' is missing column {index}")
            }
            RequestError::Plot(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for RequestError {}

impl From<malevich::Error> for RequestError {
    fn from(error: malevich::Error) -> RequestError {
        RequestError::Plot(error)
    }
}

impl From<serde_json::Error> for RequestError {
    fn from(error: serde_json::Error) -> RequestError {
        RequestError::Format(error.to_string())
    }
}

/// A request's outcome.
pub type Result<T> = std::result::Result<T, RequestError>;

/// Decodes a request: a versioned document whose series may be `{ "col": N }`
/// references into `columns`, and a frame. The document is validated.
pub fn decode_request(
    document: &str,
    frame: &str,
    columns: Vec<Vec<f64>>,
) -> Result<(Document, Frame)> {
    // The envelope is decoded here rather than as a `Document` so that a
    // validation failure keeps its typed `malevich::Error`; the crate's own
    // deserializer has to flatten it into a serde message.
    let envelope: Envelope = with_columns(columns, || serde_json::from_str(document))?;
    if envelope.version != Document::VERSION {
        return Err(RequestError::Format(format!(
            "unsupported malevich document version {}; this build supports version {}",
            envelope.version,
            Document::VERSION
        )));
    }
    let document = match envelope.content {
        Content::Plot(plot) => Document::plot(*plot)?,
        Content::Grid(grid) => Document::grid(grid)?,
    };
    let frame: Frame = serde_json::from_str(frame)?;
    Ok((document, frame))
}

#[derive(serde::Deserialize)]
struct Envelope {
    version: u32,
    #[serde(flatten)]
    content: Content,
}

#[derive(serde::Deserialize)]
#[serde(tag = "kind", content = "spec", rename_all = "snake_case")]
enum Content {
    Plot(Box<Plot<'static>>),
    Grid(Grid<'static>),
}

/// The plot a plot-only operation (raster, mapping, pixels, cards) needs, or
/// the error a grid document earns.
pub fn plot_of(document: &Document) -> Result<&Plot<'static>> {
    document.as_plot().ok_or_else(|| {
        RequestError::Format(
            "raster, mapping, pixels, and cards are plot-only; render a grid as a string".into(),
        )
    })
}

/// The protocol a request names: `sixel`, `kitty`, or `iterm2` (case and the
/// `iterm` spelling tolerated).
pub fn parse_protocol(name: &str) -> Option<Protocol> {
    Some(match name.trim().to_ascii_lowercase().as_str() {
        "sixel" => Protocol::Sixel,
        "kitty" => Protocol::Kitty,
        "iterm2" | "iterm" => Protocol::ITerm2,
        _ => return None,
    })
}

/// Serializes a document the way the bindings expect it back: compact JSON,
/// series inlined.
pub fn document_json(document: &Document) -> Result<String> {
    Ok(serde_json::to_string(document)?)
}

/// The presets [`expand_preset`] knows, in the spelling it accepts.
pub const PRESETS: &[&str] = &[
    "hist",
    "stairs",
    "ecdf",
    "density",
    "trend",
    "hist2d",
    "heatmap",
    "contour",
    "contourf",
    "box_plot",
    "violin",
    "describe",
    "table",
    "error_bars",
    "error_bars_asymmetric",
    "quiver",
    "sparkline",
];

/// Runs a crate preset over `columns` and returns the resulting document,
/// series inlined (stats presets emit small expansions). `options` is a JSON
/// object (or empty) whose fields are the preset's `_with` options in
/// snake_case; `camelCase` preset names are accepted. Furniture such as a title
/// is applied by the binding afterwards.
pub fn expand_preset(name: &str, options: &str, columns: Vec<Vec<f64>>) -> Result<Document> {
    let options = if options.trim().is_empty() {
        "{}"
    } else {
        options
    };
    let plot = match canonical(name).as_str() {
        "hist" => {
            let o: Hist = serde_json::from_str(options)?;
            let config = HistogramOptions::new(o.max_bins.unwrap_or(60))
                .normalization(o.normalization.unwrap_or_default())
                .cumulative(o.cumulative.unwrap_or(false));
            malevich::hist_with(column(&columns, 0, name)?, config)?
        }
        "stairs" => {
            let o: Stairs = serde_json::from_str(options)?;
            let direction = match o
                .direction
                .as_deref()
                .map(str::to_ascii_lowercase)
                .as_deref()
            {
                None | Some("post") => StepDirection::Post,
                Some("pre") => StepDirection::Pre,
                Some("mid") => StepDirection::Mid,
                Some(other) => {
                    return Err(RequestError::Format(format!(
                        "unknown step direction '{other}' (expected Post, Pre, or Mid)"
                    )));
                }
            };
            malevich::stairs_with(
                column(&columns, 0, name)?,
                StairsOptions::new().direction(direction),
            )
        }
        "ecdf" => {
            let o: Ecdf = serde_json::from_str(options)?;
            let mut config = EcdfOptions::new();
            if let Some(alpha) = o.band {
                config = config.band(alpha);
            }
            malevich::ecdf_with(column(&columns, 0, name)?, config)?
        }
        "density" => {
            let o: Kde = serde_json::from_str(options)?;
            let config = DensityOptions::new(o.samples.unwrap_or(256)).kde(o.kde());
            malevich::density_with(column(&columns, 0, name)?, config)?
        }
        "trend" => {
            let o: Trend = serde_json::from_str(options)?;
            let mut config = TrendOptions::new();
            if let Some(multiplier) = o.band {
                config = config.band(multiplier);
            }
            if let Some(samples) = o.band_samples {
                config.band_samples = samples;
            }
            malevich::trend_with(
                column(&columns, 0, name)?,
                column(&columns, 1, name)?,
                config,
            )?
        }
        "hist2d" => {
            let o: Hist2d = serde_json::from_str(options)?;
            let mut config = Histogram2dOptions::new(o.columns.unwrap_or(48), o.rows.unwrap_or(32));
            if let Some(colormap) = o.colormap {
                config = config.colormap(colormap);
            }
            if let Some(colorbar) = o.colorbar {
                config = config.colorbar(colorbar);
            }
            malevich::hist2d_with(
                column(&columns, 0, name)?,
                column(&columns, 1, name)?,
                config,
            )?
        }
        "heatmap" => {
            let o: GridOptions = serde_json::from_str(options)?;
            let mut config = HeatmapOptions::new();
            if let Some(colormap) = o.colormap {
                config = config.colormap(colormap);
            }
            if let Some(colorbar) = o.colorbar {
                config = config.colorbar(colorbar);
            }
            malevich::heatmap_with(o.columns, column(&columns, 0, name)?, config)?
        }
        "contour" | "contourf" => {
            let o: Contour = serde_json::from_str(options)?;
            let mut config = match o.levels {
                None => ContourOptions::default(),
                Some(Levels::Count(target)) => ContourOptions::automatic(target),
                Some(Levels::Explicit(levels)) => ContourOptions::explicit(levels),
            };
            if let Some(colormap) = o.colormap {
                config = config.colormap(colormap);
            }
            let values = column(&columns, 0, name)?;
            if canonical(name) == "contour" {
                malevich::contour_with(o.columns, values, config)?
            } else {
                malevich::contourf_with(o.columns, values, config)?
            }
        }
        "box_plot" => {
            let o: BoxPlot = serde_json::from_str(options)?;
            let groups = grouped(&o.categories, &columns, name)?;
            let mut config = BoxOptions::new();
            if let Some(whiskers) = o.whiskers {
                config = config.whiskers(whiskers.into_crate()?);
            }
            malevich::box_plot_with(o.categories, groups, config)?
        }
        "violin" => {
            let o: Violin = serde_json::from_str(options)?;
            let groups = grouped(&o.categories, &columns, name)?;
            let config = ViolinOptions::new(o.samples.unwrap_or(128)).kde(o.kde.kde());
            malevich::violin_with(o.categories, groups, config)?
        }
        "describe" => {
            let o: Describe = serde_json::from_str(options)?;
            let groups = grouped(&o.names, &columns, name)?;
            let mut config = DescribeOptions::new();
            if let Some(bins) = o.histogram {
                config = config.histogram(bins);
            }
            malevich::describe_with(o.names, groups, config)?
        }
        "table" => {
            let o: Table = serde_json::from_str(options)?;
            let mut config = TableOptions::new();
            if let Some(colormap) = o.colormap {
                config = config.colormap(colormap);
            }
            malevich::table_with(o.rows, o.columns, column(&columns, 0, name)?, config)?
        }
        "error_bars" => malevich::error_bars(
            column(&columns, 0, name)?,
            column(&columns, 1, name)?,
            same_length(&columns, 3, name)?,
        ),
        "error_bars_asymmetric" => {
            same_length(&columns, 4, name)?;
            malevich::error_bars_asymmetric(
                column(&columns, 0, name)?,
                column(&columns, 1, name)?,
                column(&columns, 2, name)?,
                column(&columns, 3, name)?,
            )
        }
        "quiver" => {
            same_length(&columns, 4, name)?;
            malevich::quiver(
                column(&columns, 0, name)?,
                column(&columns, 1, name)?,
                column(&columns, 2, name)?,
                column(&columns, 3, name)?,
            )
        }
        "sparkline" => malevich::sparkline(column(&columns, 0, name)?),
        _ => return Err(RequestError::UnknownPreset(name.to_owned())),
    };
    Ok(Document::plot(plot)?)
}

/// `camelCase` and `kebab-case` spellings of a preset name fold to the crate's.
fn canonical(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 2);
    for ch in name.trim().chars() {
        if ch.is_ascii_uppercase() {
            out.push('_');
            out.push(ch.to_ascii_lowercase());
        } else if ch == '-' {
            out.push('_');
        } else {
            out.push(ch);
        }
    }
    out
}

fn column<'a>(columns: &'a [Vec<f64>], index: usize, preset: &str) -> Result<&'a [f64]> {
    columns
        .get(index)
        .map(Vec::as_slice)
        .ok_or_else(|| RequestError::MissingColumn {
            preset: preset.to_owned(),
            index,
        })
}

/// The columns a preset reads side by side must agree in length; the crate's
/// plain presets assert this, and a binding would rather have an error.
fn same_length<'a>(columns: &'a [Vec<f64>], count: usize, preset: &str) -> Result<&'a [f64]> {
    let last = column(columns, count - 1, preset)?;
    for (index, candidate) in columns.iter().take(count).enumerate() {
        if candidate.len() != last.len() {
            return Err(RequestError::Format(format!(
                "preset '{preset}': column {index} has {} values but column {} has {}",
                candidate.len(),
                count - 1,
                last.len()
            )));
        }
    }
    Ok(last)
}

/// One group per category, in order; a mismatch is an error, not a panic.
fn grouped<'a>(names: &[String], columns: &'a [Vec<f64>], preset: &str) -> Result<Vec<&'a [f64]>> {
    if names.len() != columns.len() {
        return Err(RequestError::Format(format!(
            "preset '{preset}': {} names but {} columns",
            names.len(),
            columns.len()
        )));
    }
    Ok(columns.iter().map(Vec::as_slice).collect())
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct Hist {
    max_bins: Option<usize>,
    normalization: Option<Normalization>,
    cumulative: Option<bool>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct Stairs {
    direction: Option<String>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct Ecdf {
    band: Option<f64>,
}

/// KDE options shared by `density` and `violin`: a fixed `bandwidth`, or a
/// `bandwidth_scale` multiplying Silverman's rule; neither means Silverman.
#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct Kde {
    samples: Option<usize>,
    bandwidth: Option<f64>,
    bandwidth_scale: Option<f64>,
    cumulative: Option<bool>,
}

impl Kde {
    fn kde(&self) -> KdeOptions {
        let mut options = KdeOptions::new();
        if let Some(fixed) = self.bandwidth {
            options = options.bandwidth(Bandwidth::Fixed(fixed));
        } else if let Some(scale) = self.bandwidth_scale {
            options = options.bandwidth(Bandwidth::Scale(scale));
        }
        if self.cumulative == Some(true) {
            options = options.cumulative();
        }
        options
    }
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct Trend {
    band: Option<f64>,
    band_samples: Option<usize>,
}

#[derive(serde::Deserialize, Default)]
#[serde(default)]
struct Hist2d {
    columns: Option<usize>,
    rows: Option<usize>,
    colormap: Option<Colormap>,
    colorbar: Option<bool>,
}

#[derive(serde::Deserialize)]
struct GridOptions {
    columns: usize,
    #[serde(default)]
    colormap: Option<Colormap>,
    #[serde(default)]
    colorbar: Option<bool>,
}

#[derive(serde::Deserialize)]
struct Contour {
    columns: usize,
    #[serde(default)]
    levels: Option<Levels>,
    #[serde(default)]
    colormap: Option<Colormap>,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum Levels {
    Count(usize),
    Explicit(Vec<f64>),
}

#[derive(serde::Deserialize)]
struct BoxPlot {
    categories: Vec<String>,
    #[serde(default)]
    whiskers: Option<WhiskersWire>,
}

/// `"MinMax"`, `{ "Tukey": 1.5 }`, or `{ "Percentiles": [5, 95] }`.
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum WhiskersWire {
    Named(String),
    Tukey {
        #[serde(rename = "Tukey")]
        k: f64,
    },
    Percentiles {
        #[serde(rename = "Percentiles")]
        range: (f64, f64),
    },
}

impl WhiskersWire {
    fn into_crate(self) -> Result<Whiskers> {
        Ok(match self {
            WhiskersWire::Named(name) if name.eq_ignore_ascii_case("minmax") => Whiskers::MinMax,
            WhiskersWire::Named(name) => {
                return Err(RequestError::Format(format!(
                    "unknown whiskers '{name}' (expected MinMax, {{\"Tukey\": k}}, or {{\"Percentiles\": [lo, hi]}})"
                )));
            }
            WhiskersWire::Tukey { k } => Whiskers::Tukey(k),
            WhiskersWire::Percentiles { range } => Whiskers::Percentiles(range.0, range.1),
        })
    }
}

#[derive(serde::Deserialize)]
struct Violin {
    categories: Vec<String>,
    #[serde(default)]
    samples: Option<usize>,
    #[serde(flatten)]
    kde: Kde,
}

#[derive(serde::Deserialize)]
struct Describe {
    names: Vec<String>,
    #[serde(default)]
    histogram: Option<usize>,
}

#[derive(serde::Deserialize)]
struct Table {
    rows: Vec<String>,
    columns: Vec<String>,
    #[serde(default)]
    colormap: Option<Colormap>,
}

/// A [`Raster`] as parallel planes: one glyph per cell as a string of
/// `width * height` code points, and `foreground`/`background` as four bytes
/// per cell (see [`pack_color`]), plus one `columns` byte per cell (`0` for
/// the continuation of a wide glyph).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackedRaster {
    /// Width in cells, chrome included.
    pub width: usize,
    /// Height in cells, chrome included.
    pub height: usize,
    /// One code point per cell, row-major.
    pub glyphs: String,
    /// Four bytes per cell.
    pub foreground: Vec<u8>,
    /// Four bytes per cell.
    pub background: Vec<u8>,
    /// Display columns per cell: `2`, `1`, or `0` for a continuation.
    pub columns: Vec<u8>,
}

/// Packs a [`Color`] into four bytes: tag `0` is the terminal default, `1`
/// a named color (`index` 0–15 in the crate's `Black..BrightWhite` order),
/// `2` an `Ansi256` index, `3` an `Rgb` triple.
pub fn pack_color(color: Color) -> [u8; 4] {
    match color {
        Color::Default => [0, 0, 0, 0],
        Color::Black => [1, 0, 0, 0],
        Color::Red => [1, 1, 0, 0],
        Color::Green => [1, 2, 0, 0],
        Color::Yellow => [1, 3, 0, 0],
        Color::Blue => [1, 4, 0, 0],
        Color::Magenta => [1, 5, 0, 0],
        Color::Cyan => [1, 6, 0, 0],
        Color::White => [1, 7, 0, 0],
        Color::BrightBlack => [1, 8, 0, 0],
        Color::BrightRed => [1, 9, 0, 0],
        Color::BrightGreen => [1, 10, 0, 0],
        Color::BrightYellow => [1, 11, 0, 0],
        Color::BrightBlue => [1, 12, 0, 0],
        Color::BrightMagenta => [1, 13, 0, 0],
        Color::BrightCyan => [1, 14, 0, 0],
        Color::BrightWhite => [1, 15, 0, 0],
        Color::Ansi256(index) => [2, index, 0, 0],
        Color::Rgb(r, g, b) => [3, r, g, b],
    }
}

/// Packs a raster into planes (see [`PackedRaster`]).
pub fn pack_raster(raster: &Raster) -> PackedRaster {
    let count = raster.cells().len();
    let mut glyphs = String::with_capacity(count);
    let mut foreground = Vec::with_capacity(count * 4);
    let mut background = Vec::with_capacity(count * 4);
    let mut columns = Vec::with_capacity(count);
    for cell in raster.cells() {
        glyphs.push(cell.glyph);
        foreground.extend_from_slice(&pack_color(cell.foreground));
        background.extend_from_slice(&pack_color(cell.background));
        columns.push(cell.columns);
    }
    PackedRaster {
        width: raster.width(),
        height: raster.height(),
        glyphs,
        foreground,
        background,
        columns,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINE: &str = r#"{"version":1,"kind":"plot","spec":{"layers":[{"Line":{"x":null,"y":{"col":0},"color":null,"label":null,"style":"Pixels"}}]}}"#;
    const PLAIN: &str = r#"{"width":40,"height":10,"charset":"Braille","color":"Plain","theme":{"palette":["Cyan","Yellow","Green","Magenta","Blue","Red"]}}"#;

    #[test]
    fn a_column_reference_binds_to_the_request_table() {
        let (document, frame) =
            decode_request(LINE, PLAIN, vec![vec![1.0, 5.0, 2.0, 8.0]]).expect("decodes");
        let inline = Document::plot(malevich::line(&[1.0, 5.0, 2.0, 8.0][..])).expect("valid");
        assert_eq!(document.try_render(&frame).unwrap(), inline.render(&frame));
    }

    #[test]
    fn a_validation_failure_keeps_its_typed_error() {
        let cells = r#"{"version":1,"kind":"plot","spec":{"layers":[{"Cells":{"columns":2,"values":[1,2,3],"extents":null,"colormap":{"stops":[[0,0,0],[1,1,1]]}}}]}}"#;
        let error = decode_request(cells, PLAIN, Vec::new()).unwrap_err();
        assert!(
            matches!(
                error,
                RequestError::Plot(malevich::Error::NonRectangular { .. })
            ),
            "{error}"
        );
        let future = r#"{"version":2,"kind":"plot","spec":{"layers":[]}}"#;
        let error = decode_request(future, PLAIN, Vec::new()).unwrap_err();
        assert!(matches!(error, RequestError::Format(_)), "{error}");
    }

    #[test]
    fn an_unbound_column_is_a_format_error() {
        let error = decode_request(LINE, PLAIN, Vec::new()).unwrap_err();
        assert!(matches!(error, RequestError::Format(_)), "{error}");
    }

    #[test]
    fn every_listed_preset_expands_or_names_its_missing_column() {
        for name in PRESETS {
            let result = expand_preset(name, "", Vec::new());
            assert!(
                !matches!(result, Err(RequestError::UnknownPreset(_))),
                "{name} is listed but unknown"
            );
        }
        assert!(matches!(
            expand_preset("pie", "", Vec::new()),
            Err(RequestError::UnknownPreset(_))
        ));
    }

    #[test]
    fn preset_expansion_matches_the_crate_preset() {
        let values = vec![1.0, 2.0, 2.5, 2.7, 3.0, 3.1, 3.2, 4.0, 5.5];
        let frame = Frame::plain(40, 10);
        let expanded = expand_preset("hist", "{}", vec![values.clone()]).expect("hist");
        assert_eq!(
            expanded.render(&frame),
            malevich::hist(&values[..]).render(&frame)
        );
        let json = document_json(&expanded).expect("serializes");
        let (again, _) = decode_request(&json, PLAIN, Vec::new()).expect("round-trips inline");
        assert_eq!(again.render(&frame), expanded.render(&frame));
    }

    #[test]
    fn camel_case_and_options_are_accepted() {
        let groups = vec![vec![1.0, 2.0, 3.0, 4.0], vec![2.0, 3.0, 4.0, 9.0]];
        let camel = expand_preset(
            "boxPlot",
            r#"{"categories":["a","b"],"whiskers":"MinMax"}"#,
            groups.clone(),
        )
        .expect("camelCase");
        let snake = expand_preset(
            "box_plot",
            r#"{"categories":["a","b"],"whiskers":"MinMax"}"#,
            groups.clone(),
        )
        .expect("snake_case");
        let frame = Frame::plain(40, 10);
        assert_eq!(camel.render(&frame), snake.render(&frame));
        let expected = malevich::box_plot_with(
            ["a", "b"],
            groups.iter().map(Vec::as_slice),
            BoxOptions::new().whiskers(Whiskers::MinMax),
        )
        .expect("crate preset");
        assert_eq!(snake.render(&frame), expected.render(&frame));
    }

    #[test]
    fn mismatched_groups_are_errors_not_panics() {
        let error = expand_preset("violin", r#"{"categories":["a"]}"#, Vec::new()).unwrap_err();
        assert!(matches!(error, RequestError::Format(_)), "{error}");
        let error =
            expand_preset("quiver", "", vec![vec![1.0], vec![1.0], vec![1.0], vec![]]).unwrap_err();
        assert!(matches!(error, RequestError::Format(_)), "{error}");
    }

    #[test]
    fn packed_raster_has_one_entry_per_cell() {
        let raster = malevich::line(&[1.0, 5.0, 2.0, 8.0][..]).raster(&Frame::plain(20, 6));
        let packed = pack_raster(&raster);
        assert_eq!(packed.glyphs.chars().count(), 20 * 6);
        assert_eq!(packed.foreground.len(), 20 * 6 * 4);
        assert_eq!(packed.background.len(), 20 * 6 * 4);
        assert_eq!(packed.columns.len(), 20 * 6);
        assert_eq!(pack_color(Color::Rgb(1, 2, 3)), [3, 1, 2, 3]);
        assert_eq!(pack_color(Color::BrightWhite), [1, 15, 0, 0]);
    }

    #[test]
    fn protocols_parse_by_name() {
        assert_eq!(parse_protocol("Kitty"), Some(Protocol::Kitty));
        assert_eq!(parse_protocol("iterm"), Some(Protocol::ITerm2));
        assert_eq!(parse_protocol("sixel"), Some(Protocol::Sixel));
        assert_eq!(parse_protocol("png"), None);
    }
}
