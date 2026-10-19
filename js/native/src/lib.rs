//! Tiny wasm-bindgen surface over malevich.
//!
//! TypeScript owns the builder. This crate rasterizes and encodes: a versioned
//! `Document` plus a `Frame`, with optional out-of-band `Float64Array` columns
//! bound through `{ "col": N }` series fields. Request decoding, preset
//! expansion, and raster packing are `malevich-rim`, shared with the Python
//! binding; this file is the JS glue around it.

use malevich::{Mapping, Viewport};
use malevich_rim as rim;
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

fn decode(
    document: &str,
    frame: &str,
    columns: &JsValue,
) -> Result<(malevich::Document, malevich::Frame), JsError> {
    rim::decode_request(document, frame, js_columns(columns)?).map_err(js_error)
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
    let (document, frame) = decode(document, frame, columns)?;
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
    let (document, frame) = decode(document, frame, columns)?;
    let plot = rim::plot_of(&document).map_err(js_error)?;
    let protocol = rim::parse_protocol(protocol)
        .ok_or_else(|| JsError::new(&format!("unknown pixel protocol '{protocol}'")))?;
    let graphics = malevich::pixel::Graphics::new(protocol).cell_size(cell_width, cell_height);
    plot.try_render_pixels(&frame, &graphics).map_err(js_error)
}

/// Same inputs as [`render_columns`]; returns a packed raster:
/// `{ width, height, glyphs, fg, bg, columns }` where `fg`/`bg` are packed
/// colors (4 bytes/cell) and `columns` is one byte per cell.
#[wasm_bindgen]
pub fn raster_columns(document: &str, frame: &str, columns: &JsValue) -> Result<JsValue, JsError> {
    let (document, frame) = decode(document, frame, columns)?;
    let plot = rim::plot_of(&document).map_err(js_error)?;
    let packed = rim::pack_raster(&plot.try_raster(&frame).map_err(js_error)?);
    let object = js_sys::Object::new();
    set(&object, "width", &(packed.width as f64).into())?;
    set(&object, "height", &(packed.height as f64).into())?;
    set(&object, "glyphs", &packed.glyphs.into())?;
    set(
        &object,
        "fg",
        &js_sys::Uint8Array::from(packed.foreground.as_slice()).into(),
    )?;
    set(
        &object,
        "bg",
        &js_sys::Uint8Array::from(packed.background.as_slice()).into(),
    )?;
    set(
        &object,
        "columns",
        &js_sys::Uint8Array::from(packed.columns.as_slice()).into(),
    )?;
    Ok(object.into())
}

fn set(object: &js_sys::Object, key: &str, value: &JsValue) -> Result<(), JsError> {
    js_sys::Reflect::set(object, &key.into(), value)
        .map(|_| ())
        .map_err(|_| JsError::new(&format!("failed to set raster {key}")))
}

/// Same inputs as [`render_columns`]; returns a [`JsMapping`] wrapping the
/// resolved geometry of this render.
#[wasm_bindgen]
pub fn mapping_columns(
    document: &str,
    frame: &str,
    columns: &JsValue,
) -> Result<JsMapping, JsError> {
    let (document, frame) = decode(document, frame, columns)?;
    let plot = rim::plot_of(&document).map_err(js_error)?;
    Ok(JsMapping {
        inner: plot.mapping(&frame),
    })
}

/// Runs a crate preset against `columns` and returns a versioned Document JSON
/// (series inlined — stats presets emit small expansions). Furniture such as
/// a title is applied in TypeScript after this call.
#[wasm_bindgen]
pub fn expand_preset(name: &str, options: &str, columns: &JsValue) -> Result<String, JsError> {
    let document = rim::expand_preset(name, options, js_columns(columns)?).map_err(js_error)?;
    rim::document_json(&document).map_err(js_error)
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
