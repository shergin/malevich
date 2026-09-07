//! Tiny wasm-bindgen surface over malevich.
//!
//! TypeScript owns the builder. This crate rasterizes and encodes: a versioned
//! `Document` plus a `Frame`, with optional out-of-band `Float64Array` columns
//! bound through `{ "col": N }` series fields.

use malevich::data::with_columns;
use malevich::render::{Color, Raster};
use malevich::{Document, Frame, Mapping, Viewport};
use wasm_bindgen::prelude::*;

fn js_error(error: impl std::fmt::Display) -> JsError {
    JsError::new(&error.to_string())
}

fn js_columns(value: &JsValue) -> Result<Vec<Vec<f64>>, JsError> {
    if value.is_null() || value.is_undefined() {
        return Ok(Vec::new());
    }
    let array = js_sys::Array::from(value);
    let mut columns = Vec::with_capacity(array.length() as usize);
    for index in 0..array.length() {
        let values = js_sys::Float64Array::new(&array.get(index));
        let mut column = vec![0.0; values.length() as usize];
        values.copy_to(&mut column);
        columns.push(column);
    }
    Ok(columns)
}

fn decode_document(document: &str, columns: Vec<Vec<f64>>) -> Result<Document, JsError> {
    with_columns(columns, || serde_json::from_str(document)).map_err(js_error)
}

fn decode_frame(frame: &str) -> Result<Frame, JsError> {
    serde_json::from_str(frame).map_err(js_error)
}

fn document_plot(document: &Document) -> Result<&malevich::Plot<'static>, JsError> {
    document
        .as_plot()
        .ok_or_else(|| JsError::new("raster and mapping are plot-only; render a grid as a string"))
}

/// Small plots and goldens. `document` and `frame` are JSON. Series are inline
/// arrays (gaps = `null`) or `{ "col": N }` into `columns`.
#[wasm_bindgen]
pub fn render_document(document: &str, frame: &str) -> Result<String, JsError> {
    render_columns(document, frame, &JsValue::UNDEFINED)
}

/// Large series. `columns` is a JS array of `Float64Array`. The document may
/// use `{ "col": N }` in place of an inline array on any series field.
#[wasm_bindgen]
pub fn render_columns(document: &str, frame: &str, columns: &JsValue) -> Result<String, JsError> {
    let document = decode_document(document, js_columns(columns)?)?;
    let frame = decode_frame(frame)?;
    document.try_render(&frame).map_err(js_error)
}

/// Hybrid pixel render. `protocol` is `sixel`, `kitty`, or `iterm2`.
/// Detection stays in JS; this path is pure over the named protocol.
#[wasm_bindgen]
pub fn render_pixels_columns(
    document: &str,
    frame: &str,
    columns: &JsValue,
    protocol: &str,
    cell_width: u16,
    cell_height: u16,
) -> Result<String, JsError> {
    let document = decode_document(document, js_columns(columns)?)?;
    let frame = decode_frame(frame)?;
    let plot = document_plot(&document)?;
    let protocol = match protocol {
        "sixel" | "Sixel" => malevich::pixel::Protocol::Sixel,
        "kitty" | "Kitty" => malevich::pixel::Protocol::Kitty,
        "iterm2" | "iTerm2" | "ITerm2" => malevich::pixel::Protocol::ITerm2,
        other => {
            return Err(JsError::new(&format!("unknown pixel protocol '{other}'")));
        }
    };
    let graphics = malevich::pixel::Graphics::new(protocol).cell_size(cell_width, cell_height);
    plot.try_render_pixels(&frame, &graphics).map_err(js_error)
}

/// Same inputs as [`render_columns`]; returns a packed raster:
/// `{ width, height, glyphs, fg, bg, columns }` where `fg`/`bg` are packed
/// colors (4 bytes/cell) and `columns` is one byte per cell.
#[wasm_bindgen]
pub fn raster_columns(document: &str, frame: &str, columns: &JsValue) -> Result<JsValue, JsError> {
    let document = decode_document(document, js_columns(columns)?)?;
    let frame = decode_frame(frame)?;
    let plot = document_plot(&document)?;
    raster_to_js(&plot.try_raster(&frame).map_err(js_error)?)
}

/// Same inputs as [`render_columns`]; returns a [`JsMapping`] wrapping the
/// resolved geometry of this render.
#[wasm_bindgen]
pub fn mapping_columns(
    document: &str,
    frame: &str,
    columns: &JsValue,
) -> Result<JsMapping, JsError> {
    let document = decode_document(document, js_columns(columns)?)?;
    let frame = decode_frame(frame)?;
    let plot = document_plot(&document)?;
    Ok(JsMapping {
        inner: plot.mapping(&frame),
    })
}

/// Runs a crate preset against `columns` and returns a versioned Document JSON
/// (series inlined — stats presets emit small expansions). Furniture such as
/// a title is applied in TypeScript after this call.
#[wasm_bindgen]
pub fn expand_preset(name: &str, options: &str, columns: &JsValue) -> Result<String, JsError> {
    let columns = js_columns(columns)?;
    let plot = match name {
        "hist" => {
            let options: HistOptions =
                serde_json::from_str(empty_as_object(options)).map_err(js_error)?;
            let values = column(&columns, 0)?;
            malevich::hist_with(
                values,
                malevich::HistogramOptions::new(options.max_bins.unwrap_or(60)),
            )
            .map_err(js_error)?
        }
        "stairs" => malevich::stairs(column(&columns, 0)?),
        "ecdf" => malevich::ecdf(column(&columns, 0)?),
        "density" => malevich::density(column(&columns, 0)?),
        "trend" => malevich::trend(column(&columns, 0)?, column(&columns, 1)?),
        "hist2d" => malevich::hist2d(column(&columns, 0)?, column(&columns, 1)?),
        "heatmap" => {
            let options: HeatmapOptions =
                serde_json::from_str(empty_as_object(options)).map_err(js_error)?;
            malevich::heatmap(options.columns, column(&columns, 0)?)
        }
        "box_plot" | "boxPlot" => {
            let options: NamedGroups =
                serde_json::from_str(empty_as_object(options)).map_err(js_error)?;
            malevich::box_plot(options.categories, columns)
        }
        "violin" => {
            let options: NamedGroups =
                serde_json::from_str(empty_as_object(options)).map_err(js_error)?;
            malevich::violin(options.categories, columns)
        }
        "describe" => {
            let options: DescribeOptions =
                serde_json::from_str(empty_as_object(options)).map_err(js_error)?;
            malevich::describe(options.names, columns)
        }
        "table" => {
            let options: TableOptions =
                serde_json::from_str(empty_as_object(options)).map_err(js_error)?;
            let mut table = malevich::TableOptions::new();
            if let Some(colormap) = options.colormap {
                table = table.colormap(colormap);
            }
            malevich::table_with(options.rows, options.columns, column(&columns, 0)?, table)
                .map_err(js_error)?
        }
        "error_bars" | "errorBars" => malevich::error_bars(
            column(&columns, 0)?,
            column(&columns, 1)?,
            column(&columns, 2)?,
        ),
        other => {
            return Err(JsError::new(&format!("unknown preset '{other}'")));
        }
    };
    let document = Document::plot(plot).map_err(js_error)?;
    serde_json::to_string(&document).map_err(js_error)
}

fn empty_as_object(options: &str) -> &str {
    if options.trim().is_empty() {
        "{}"
    } else {
        options
    }
}

fn column(columns: &[Vec<f64>], index: usize) -> Result<&[f64], JsError> {
    columns
        .get(index)
        .map(Vec::as_slice)
        .ok_or_else(|| JsError::new(&format!("preset is missing column {index}")))
}

#[derive(serde::Deserialize)]
struct HistOptions {
    max_bins: Option<usize>,
}

#[derive(serde::Deserialize)]
struct HeatmapOptions {
    columns: usize,
}

#[derive(serde::Deserialize)]
struct NamedGroups {
    categories: Vec<String>,
}

#[derive(serde::Deserialize)]
struct DescribeOptions {
    names: Vec<String>,
}

#[derive(serde::Deserialize)]
struct TableOptions {
    rows: Vec<String>,
    columns: Vec<String>,
    #[serde(default)]
    colormap: Option<malevich::scale::Colormap>,
}

/// Resolved geometry of one render, queryable from JavaScript.
#[wasm_bindgen]
pub struct JsMapping {
    inner: Mapping,
}

#[wasm_bindgen]
impl JsMapping {
    /// Data coordinates at the center of the frame cell `(column, row)`.
    #[wasm_bindgen(js_name = dataAt)]
    pub fn data_at(&self, column: u32, row: u32) -> JsValue {
        match self.inner.data_at(column as usize, row as usize) {
            Some((x, y)) => pair(x, y),
            None => JsValue::UNDEFINED,
        }
    }

    /// Frame cell where the data point `(x, y)` draws.
    #[wasm_bindgen(js_name = cellAt)]
    pub fn cell_at(&self, x: f64, y: f64) -> JsValue {
        match self.inner.cell_at(x, y) {
            Some((column, row)) => pair(column as f64, row as f64),
            None => JsValue::UNDEFINED,
        }
    }

    /// Frame column where the data value `x` draws.
    #[wasm_bindgen(js_name = columnAt)]
    pub fn column_at(&self, x: f64) -> JsValue {
        match self.inner.column_at(x) {
            Some(column) => JsValue::from(column as f64),
            None => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(js_name = formatX)]
    pub fn format_x(&self, value: f64) -> String {
        self.inner.format_x(value)
    }

    #[wasm_bindgen(js_name = formatY)]
    pub fn format_y(&self, value: f64) -> String {
        self.inner.format_y(value)
    }

    #[wasm_bindgen(getter, js_name = xDomain)]
    pub fn x_domain(&self) -> JsValue {
        let (lo, hi) = self.inner.x_domain();
        pair(lo, hi)
    }

    #[wasm_bindgen(getter, js_name = yDomain)]
    pub fn y_domain(&self) -> JsValue {
        let (lo, hi) = self.inner.y_domain();
        pair(lo, hi)
    }

    #[wasm_bindgen(getter, js_name = plotArea)]
    pub fn plot_area(&self) -> JsValue {
        let Some(panel) = self.inner.plot_area() else {
            return JsValue::UNDEFINED;
        };
        let object = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&object, &"column".into(), &(panel.column as f64).into());
        let _ = js_sys::Reflect::set(&object, &"row".into(), &(panel.row as f64).into());
        let _ = js_sys::Reflect::set(&object, &"width".into(), &(panel.width as f64).into());
        let _ = js_sys::Reflect::set(&object, &"height".into(), &(panel.height as f64).into());
        object.into()
    }

    /// Category labels on a bands x axis, in band order. `undefined` on a
    /// continuous axis — the signal that x gestures (zoom, pan, rubber-band)
    /// should leave the window untouched.
    #[wasm_bindgen(getter, js_name = xCategories)]
    pub fn x_categories(&self) -> JsValue {
        strings(self.inner.x_categories())
    }

    /// Category labels on a bands y axis, in band order (band 0 is the top
    /// row). `undefined` on a continuous axis.
    #[wasm_bindgen(getter, js_name = yCategories)]
    pub fn y_categories(&self) -> JsValue {
        strings(self.inner.y_categories())
    }

    /// Seeds a [`JsViewport`] from this mapping's resolved domains.
    pub fn viewport(&self) -> JsViewport {
        JsViewport {
            inner: self.inner.viewport(),
        }
    }
}

/// Axis window pair for zoom and pan. Transforms are pure; apply with
/// `Plot.viewport` in TypeScript (as `xDomain` / `yDomain` on the spec).
#[wasm_bindgen]
pub struct JsViewport {
    inner: Viewport,
}

#[wasm_bindgen]
impl JsViewport {
    pub fn auto() -> JsViewport {
        JsViewport {
            inner: Viewport::auto(),
        }
    }

    #[wasm_bindgen(js_name = zoomX)]
    pub fn zoom_x(&self, factor: f64, anchor: f64) -> JsViewport {
        JsViewport {
            inner: self.inner.zoom_x(factor, anchor),
        }
    }

    #[wasm_bindgen(js_name = zoomY)]
    pub fn zoom_y(&self, factor: f64, anchor: f64) -> JsViewport {
        JsViewport {
            inner: self.inner.zoom_y(factor, anchor),
        }
    }

    #[wasm_bindgen(js_name = panX)]
    pub fn pan_x(&self, fraction: f64) -> JsViewport {
        JsViewport {
            inner: self.inner.pan_x(fraction),
        }
    }

    #[wasm_bindgen(js_name = panY)]
    pub fn pan_y(&self, fraction: f64) -> JsViewport {
        JsViewport {
            inner: self.inner.pan_y(fraction),
        }
    }

    pub fn reset(&self) -> JsViewport {
        JsViewport {
            inner: self.inner.reset(),
        }
    }

    #[wasm_bindgen(js_name = resetX)]
    pub fn reset_x(&self) -> JsViewport {
        JsViewport {
            inner: self.inner.reset_x(),
        }
    }

    #[wasm_bindgen(js_name = resetY)]
    pub fn reset_y(&self) -> JsViewport {
        JsViewport {
            inner: self.inner.reset_y(),
        }
    }

    #[wasm_bindgen(js_name = withX)]
    pub fn with_x(&self, low: f64, high: f64) -> JsViewport {
        JsViewport {
            inner: self.inner.with_x(low, high),
        }
    }

    #[wasm_bindgen(js_name = withY)]
    pub fn with_y(&self, low: f64, high: f64) -> JsViewport {
        JsViewport {
            inner: self.inner.with_y(low, high),
        }
    }

    #[wasm_bindgen(js_name = clampX)]
    pub fn clamp_x(&self, low: f64, high: f64) -> JsViewport {
        JsViewport {
            inner: self.inner.clamp_x(low, high),
        }
    }

    #[wasm_bindgen(js_name = clampY)]
    pub fn clamp_y(&self, low: f64, high: f64) -> JsViewport {
        JsViewport {
            inner: self.inner.clamp_y(low, high),
        }
    }

    pub fn tail(&self, latest: f64, width: f64) -> JsViewport {
        JsViewport {
            inner: self.inner.tail(latest, width),
        }
    }

    #[wasm_bindgen(getter, js_name = isAuto)]
    pub fn is_auto(&self) -> bool {
        self.inner.is_auto()
    }

    #[wasm_bindgen(getter)]
    pub fn x(&self) -> JsValue {
        match self.inner.x() {
            Some((lo, hi)) => pair(lo, hi),
            None => JsValue::UNDEFINED,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn y(&self) -> JsValue {
        match self.inner.y() {
            Some((lo, hi)) => pair(lo, hi),
            None => JsValue::UNDEFINED,
        }
    }
}

fn strings(values: Option<&[String]>) -> JsValue {
    let Some(values) = values else {
        return JsValue::UNDEFINED;
    };
    let array = js_sys::Array::new();
    for value in values {
        array.push(&JsValue::from_str(value));
    }
    array.into()
}

fn pair(a: f64, b: f64) -> JsValue {
    let array = js_sys::Array::new();
    array.push(&JsValue::from(a));
    array.push(&JsValue::from(b));
    array.into()
}

fn pack_color(color: Color, out: &mut Vec<u8>) {
    match color {
        Color::Default => out.extend_from_slice(&[0, 0, 0, 0]),
        Color::Black => out.extend_from_slice(&[1, 0, 0, 0]),
        Color::Red => out.extend_from_slice(&[1, 1, 0, 0]),
        Color::Green => out.extend_from_slice(&[1, 2, 0, 0]),
        Color::Yellow => out.extend_from_slice(&[1, 3, 0, 0]),
        Color::Blue => out.extend_from_slice(&[1, 4, 0, 0]),
        Color::Magenta => out.extend_from_slice(&[1, 5, 0, 0]),
        Color::Cyan => out.extend_from_slice(&[1, 6, 0, 0]),
        Color::White => out.extend_from_slice(&[1, 7, 0, 0]),
        Color::BrightBlack => out.extend_from_slice(&[1, 8, 0, 0]),
        Color::BrightRed => out.extend_from_slice(&[1, 9, 0, 0]),
        Color::BrightGreen => out.extend_from_slice(&[1, 10, 0, 0]),
        Color::BrightYellow => out.extend_from_slice(&[1, 11, 0, 0]),
        Color::BrightBlue => out.extend_from_slice(&[1, 12, 0, 0]),
        Color::BrightMagenta => out.extend_from_slice(&[1, 13, 0, 0]),
        Color::BrightCyan => out.extend_from_slice(&[1, 14, 0, 0]),
        Color::BrightWhite => out.extend_from_slice(&[1, 15, 0, 0]),
        Color::Ansi256(index) => out.extend_from_slice(&[2, index, 0, 0]),
        Color::Rgb(r, g, b) => out.extend_from_slice(&[3, r, g, b]),
    }
}

fn raster_to_js(raster: &Raster) -> Result<JsValue, JsError> {
    let object = js_sys::Object::new();
    js_sys::Reflect::set(&object, &"width".into(), &(raster.width() as f64).into())
        .map_err(|_| JsError::new("failed to set raster width"))?;
    js_sys::Reflect::set(&object, &"height".into(), &(raster.height() as f64).into())
        .map_err(|_| JsError::new("failed to set raster height"))?;
    let mut glyphs = String::with_capacity(raster.cells().len());
    let mut foreground = Vec::with_capacity(raster.cells().len() * 4);
    let mut background = Vec::with_capacity(raster.cells().len() * 4);
    let mut columns = Vec::with_capacity(raster.cells().len());
    for cell in raster.cells() {
        glyphs.push(cell.glyph);
        pack_color(cell.foreground, &mut foreground);
        pack_color(cell.background, &mut background);
        columns.push(cell.columns);
    }
    js_sys::Reflect::set(&object, &"glyphs".into(), &glyphs.into())
        .map_err(|_| JsError::new("failed to set raster glyphs"))?;
    js_sys::Reflect::set(
        &object,
        &"fg".into(),
        &js_sys::Uint8Array::from(foreground.as_slice()).into(),
    )
    .map_err(|_| JsError::new("failed to set raster fg"))?;
    js_sys::Reflect::set(
        &object,
        &"bg".into(),
        &js_sys::Uint8Array::from(background.as_slice()).into(),
    )
    .map_err(|_| JsError::new("failed to set raster bg"))?;
    js_sys::Reflect::set(
        &object,
        &"columns".into(),
        &js_sys::Uint8Array::from(columns.as_slice()).into(),
    )
    .map_err(|_| JsError::new("failed to set raster columns"))?;
    Ok(object.into())
}
