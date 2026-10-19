//! The native module of the `supremus` Python package.
//!
//! Python owns the builder, the environment detection, and every host
//! protocol. This module rasterizes and encodes: a versioned `Document` plus a
//! `Frame`, with series bound out of band as columns — the same request the
//! npm package's wasm module takes, decoded by the shared `malevich-rim`.
//!
//! A column is a Python `list`/`tuple` of numbers (ints, floats, bools, `None`
//! for a gap), extracted here in one pass, or any object exposing a contiguous
//! `float64` buffer (`array('d')`, numpy, whatever `to_numpy()` returns),
//! copied once. Rendering runs with the GIL released. Nothing here reads the
//! environment.

use malevich::pixel::Graphics;
use malevich::{Document, Frame};
use malevich_rim as rim;
use pyo3::buffer::PyBuffer;
use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyList, PyTuple};

pyo3::create_exception!(
    _native,
    EngineError,
    PyValueError,
    "The engine refused a request. `args` is `(code, message)`."
);

/// The crate's error variant as a stable code the facade exposes.
fn code(error: &rim::RequestError) -> &'static str {
    match error {
        rim::RequestError::Format(_) => "Format",
        rim::RequestError::UnknownPreset(_) => "UnknownPreset",
        rim::RequestError::MissingColumn { .. } => "MissingColumn",
        rim::RequestError::Plot(error) => match error {
            malevich::Error::UnequalChannels { .. } => "UnequalChannels",
            malevich::Error::NonRectangular { .. } => "NonRectangular",
            malevich::Error::EmptyDimension { .. } => "EmptyDimension",
            malevich::Error::NonFiniteDomain { .. } => "NonFiniteDomain",
            malevich::Error::IncompatibleScale { .. } => "IncompatibleScale",
            malevich::Error::InvalidParameter { .. } => "InvalidParameter",
            malevich::Error::DimensionTooLarge { .. } => "DimensionTooLarge",
            malevich::Error::AllocationFailed { .. } => "AllocationFailed",
            _ => "Engine",
        },
        _ => "Engine",
    }
}

fn engine_error(error: rim::RequestError) -> PyErr {
    EngineError::new_err((code(&error), error.to_string()))
}

fn plot_error(error: malevich::Error) -> PyErr {
    engine_error(rim::RequestError::Plot(error))
}

/// One series as the engine wants it: contiguous `f64`, `NaN` for a gap.
fn column(value: &Bound<'_, PyAny>) -> PyResult<Vec<f64>> {
    if value.is_instance_of::<PyList>() || value.is_instance_of::<PyTuple>() {
        let values: Vec<Option<f64>> = value.extract().map_err(|error| {
            PyTypeError::new_err(format!("a series is numbers and None gaps; {error}"))
        })?;
        return Ok(values
            .into_iter()
            .map(|value| value.unwrap_or(f64::NAN))
            .collect());
    }
    match PyBuffer::<f64>::get(value) {
        Ok(buffer) => buffer.to_vec(value.py()),
        Err(error) => Err(PyTypeError::new_err(format!(
            "a series is a list, a tuple, or a float64 buffer (array('d'), numpy, to_numpy()); \
             got {}: {error}",
            value.get_type().name()?
        ))),
    }
}

fn to_columns(values: Vec<Bound<'_, PyAny>>) -> PyResult<Vec<Vec<f64>>> {
    values.iter().map(column).collect()
}

/// Decodes with the GIL released; the request strings and columns are owned.
fn decode(
    py: Python<'_>,
    document: &str,
    frame: &str,
    columns: Vec<Vec<f64>>,
) -> PyResult<(Document, Frame)> {
    let (document, frame) = (document.to_owned(), frame.to_owned());
    py.detach(move || rim::decode_request(&document, &frame, columns).map_err(engine_error))
}

/// The request as one owned bundle, so a render can run detached.
fn request(
    py: Python<'_>,
    document: &str,
    frame: &str,
    values: Vec<Bound<'_, PyAny>>,
) -> PyResult<(Document, Frame)> {
    let columns = to_columns(values)?;
    decode(py, document, frame, columns)
}

/// `document` and `frame` are JSON; series are inline arrays (gaps `null`) or
/// `{ "col": N }` into `columns`. Returns the ANSI or plain string.
#[pyfunction]
fn render_columns(
    py: Python<'_>,
    document: &str,
    frame: &str,
    columns: Vec<Bound<'_, PyAny>>,
) -> PyResult<String> {
    let (document, frame) = request(py, document, frame, columns)?;
    py.detach(move || document.try_render(&frame).map_err(plot_error))
}

/// The HTML terminal card (`Plot::to_html`); plot documents only.
#[pyfunction]
fn html_columns(
    py: Python<'_>,
    document: &str,
    frame: &str,
    columns: Vec<Bound<'_, PyAny>>,
) -> PyResult<String> {
    let (document, frame) = request(py, document, frame, columns)?;
    py.detach(move || {
        let plot = rim::plot_of(&document).map_err(engine_error)?;
        Ok(plot.to_html(&frame))
    })
}

/// The SVG terminal card (`Plot::to_svg`); plot documents only.
#[pyfunction]
fn svg_columns(
    py: Python<'_>,
    document: &str,
    frame: &str,
    columns: Vec<Bound<'_, PyAny>>,
) -> PyResult<String> {
    let (document, frame) = request(py, document, frame, columns)?;
    py.detach(move || {
        let plot = rim::plot_of(&document).map_err(engine_error)?;
        Ok(plot.to_svg(&frame))
    })
}

/// The SVG card with the panel as device pixels (`Plot::to_svg_pixels`).
#[pyfunction]
fn svg_pixels_columns(
    py: Python<'_>,
    document: &str,
    frame: &str,
    columns: Vec<Bound<'_, PyAny>>,
    cell_width: u16,
    cell_height: u16,
) -> PyResult<String> {
    let (document, frame) = request(py, document, frame, columns)?;
    py.detach(move || {
        let plot = rim::plot_of(&document).map_err(engine_error)?;
        let graphics =
            Graphics::new(malevich::pixel::Protocol::Kitty).cell_size(cell_width, cell_height);
        plot.try_to_svg_pixels(&frame, &graphics)
            .map_err(plot_error)
    })
}

/// Hybrid pixel render. `protocol` is `sixel`, `kitty`, or `iterm2`; detection
/// stays in Python, this path is pure over the named protocol.
#[pyfunction]
fn render_pixels_columns(
    py: Python<'_>,
    document: &str,
    frame: &str,
    columns: Vec<Bound<'_, PyAny>>,
    protocol: &str,
    cell_width: u16,
    cell_height: u16,
) -> PyResult<String> {
    let protocol = rim::parse_protocol(protocol)
        .ok_or_else(|| PyValueError::new_err(format!("unknown pixel protocol '{protocol}'")))?;
    let (document, frame) = request(py, document, frame, columns)?;
    py.detach(move || {
        let plot = rim::plot_of(&document).map_err(engine_error)?;
        let graphics = Graphics::new(protocol).cell_size(cell_width, cell_height);
        plot.try_render_pixels(&frame, &graphics)
            .map_err(plot_error)
    })
}

/// `(width, height, glyphs, foreground, background, columns)` of a packed
/// raster.
type Planes<'py> = (
    usize,
    usize,
    String,
    Bound<'py, PyBytes>,
    Bound<'py, PyBytes>,
    Bound<'py, PyBytes>,
);

/// The packed raster: `(width, height, glyphs, foreground, background,
/// columns)`. `glyphs` is one code point per cell; `foreground` and
/// `background` are four bytes per cell (tag, then index or r, g, b);
/// `columns` is one byte per cell, `0` for a wide glyph's continuation.
#[pyfunction]
fn raster_columns<'py>(
    py: Python<'py>,
    document: &str,
    frame: &str,
    columns: Vec<Bound<'py, PyAny>>,
) -> PyResult<Planes<'py>> {
    let (document, frame) = request(py, document, frame, columns)?;
    let packed = py.detach(move || {
        let plot = rim::plot_of(&document).map_err(engine_error)?;
        let raster = plot.try_raster(&frame).map_err(plot_error)?;
        Ok::<_, PyErr>(rim::pack_raster(&raster))
    })?;
    Ok((
        packed.width,
        packed.height,
        packed.glyphs,
        PyBytes::new(py, &packed.foreground),
        PyBytes::new(py, &packed.background),
        PyBytes::new(py, &packed.columns),
    ))
}

/// The resolved geometry of one render.
#[pyfunction]
fn mapping_columns(
    py: Python<'_>,
    document: &str,
    frame: &str,
    columns: Vec<Bound<'_, PyAny>>,
) -> PyResult<Mapping> {
    let (document, frame) = request(py, document, frame, columns)?;
    py.detach(move || {
        let plot = rim::plot_of(&document).map_err(engine_error)?;
        Ok(Mapping {
            inner: plot.mapping(&frame),
        })
    })
}

/// Runs a crate preset over `columns` and returns a document JSON with the
/// expansion inlined. `options` is a JSON object (or empty).
#[pyfunction]
fn expand_preset(
    py: Python<'_>,
    name: &str,
    options: &str,
    columns: Vec<Bound<'_, PyAny>>,
) -> PyResult<String> {
    let columns = to_columns(columns)?;
    let (name, options) = (name.to_owned(), options.to_owned());
    py.detach(move || {
        let document = rim::expand_preset(&name, &options, columns).map_err(engine_error)?;
        rim::document_json(&document).map_err(engine_error)
    })
}

/// Decodes and validates a document without rendering it; the frame may be
/// any valid frame. Returns `"plot"` or `"grid"`.
#[pyfunction]
fn validate_columns(
    py: Python<'_>,
    document: &str,
    columns: Vec<Bound<'_, PyAny>>,
) -> PyResult<&'static str> {
    const FRAME: &str = r#"{"width":1,"height":1,"charset":"Ascii","color":"Plain","theme":{"palette":["Cyan","Yellow","Green","Magenta","Blue","Red"]}}"#;
    let (document, _) = request(py, document, FRAME, columns)?;
    Ok(match document.kind() {
        malevich::DocumentKind::Plot => "plot",
        malevich::DocumentKind::Grid => "grid",
        _ => "document",
    })
}

/// The preset names `expand_preset` accepts.
#[pyfunction]
fn preset_names() -> Vec<&'static str> {
    rim::PRESETS.to_vec()
}

/// Resolved geometry of one render: hit-testing, axis-formatted labels, the
/// plot rectangle, and the seed of a viewport.
#[pyclass(frozen, name = "Mapping", module = "supremus._native")]
pub struct Mapping {
    inner: malevich::Mapping,
}

#[pymethods]
impl Mapping {
    /// Data coordinates at the center of the frame cell `(column, row)`, or
    /// `None` outside the plot rectangle.
    fn data_at(&self, column: usize, row: usize) -> Option<(f64, f64)> {
        self.inner.data_at(column, row)
    }

    /// The frame cell where the data point `(x, y)` draws.
    fn cell_at(&self, x: f64, y: f64) -> Option<(usize, usize)> {
        self.inner.cell_at(x, y)
    }

    /// The frame column where the data value `x` draws.
    fn column_at(&self, x: f64) -> Option<usize> {
        self.inner.column_at(x)
    }

    /// `x` as the x axis labels it.
    fn format_x(&self, value: f64) -> String {
        self.inner.format_x(value)
    }

    /// `y` as the y axis labels it.
    fn format_y(&self, value: f64) -> String {
        self.inner.format_y(value)
    }

    #[getter]
    fn x_domain(&self) -> (f64, f64) {
        self.inner.x_domain()
    }

    #[getter]
    fn y_domain(&self) -> (f64, f64) {
        self.inner.y_domain()
    }

    /// The plot rectangle as `(column, row, width, height)`, or `None` when
    /// the frame left no room for a panel.
    #[getter]
    fn plot_area(&self) -> Option<(usize, usize, usize, usize)> {
        self.inner
            .plot_area()
            .map(|panel| (panel.column, panel.row, panel.width, panel.height))
    }

    /// Band labels on a categorical x axis, in band order; `None` on a
    /// continuous axis.
    #[getter]
    fn x_categories(&self) -> Option<Vec<String>> {
        self.inner.x_categories().map(<[String]>::to_vec)
    }

    /// Band labels on a categorical y axis (band 0 is the top row); `None`
    /// on a continuous axis.
    #[getter]
    fn y_categories(&self) -> Option<Vec<String>> {
        self.inner.y_categories().map(<[String]>::to_vec)
    }

    /// A viewport seeded from this mapping's resolved domains.
    fn viewport(&self) -> Viewport {
        Viewport {
            inner: self.inner.viewport(),
        }
    }
}

/// An axis window pair with zoom and pan arithmetic. Every transform returns
/// a new viewport; log axes stay in decade space.
#[pyclass(frozen, name = "Viewport", module = "supremus._native")]
pub struct Viewport {
    inner: malevich::Viewport,
}

#[pymethods]
impl Viewport {
    /// Both axes automatic.
    #[staticmethod]
    fn auto() -> Viewport {
        Viewport {
            inner: malevich::Viewport::auto(),
        }
    }

    fn zoom_x(&self, factor: f64, anchor: f64) -> Viewport {
        Viewport {
            inner: self.inner.zoom_x(factor, anchor),
        }
    }

    fn zoom_y(&self, factor: f64, anchor: f64) -> Viewport {
        Viewport {
            inner: self.inner.zoom_y(factor, anchor),
        }
    }

    fn pan_x(&self, fraction: f64) -> Viewport {
        Viewport {
            inner: self.inner.pan_x(fraction),
        }
    }

    fn pan_y(&self, fraction: f64) -> Viewport {
        Viewport {
            inner: self.inner.pan_y(fraction),
        }
    }

    fn reset(&self) -> Viewport {
        Viewport {
            inner: self.inner.reset(),
        }
    }

    fn reset_x(&self) -> Viewport {
        Viewport {
            inner: self.inner.reset_x(),
        }
    }

    fn reset_y(&self) -> Viewport {
        Viewport {
            inner: self.inner.reset_y(),
        }
    }

    fn with_x(&self, low: f64, high: f64) -> Viewport {
        Viewport {
            inner: self.inner.with_x(low, high),
        }
    }

    fn with_y(&self, low: f64, high: f64) -> Viewport {
        Viewport {
            inner: self.inner.with_y(low, high),
        }
    }

    fn clamp_x(&self, low: f64, high: f64) -> Viewport {
        Viewport {
            inner: self.inner.clamp_x(low, high),
        }
    }

    fn clamp_y(&self, low: f64, high: f64) -> Viewport {
        Viewport {
            inner: self.inner.clamp_y(low, high),
        }
    }

    fn tail(&self, latest: f64, width: f64) -> Viewport {
        Viewport {
            inner: self.inner.tail(latest, width),
        }
    }

    #[getter]
    fn is_auto(&self) -> bool {
        self.inner.is_auto()
    }

    #[getter]
    fn x(&self) -> Option<(f64, f64)> {
        self.inner.x()
    }

    #[getter]
    fn y(&self) -> Option<(f64, f64)> {
        self.inner.y()
    }
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("EngineError", m.py().get_type::<EngineError>())?;
    m.add_function(wrap_pyfunction!(render_columns, m)?)?;
    m.add_function(wrap_pyfunction!(html_columns, m)?)?;
    m.add_function(wrap_pyfunction!(svg_columns, m)?)?;
    m.add_function(wrap_pyfunction!(svg_pixels_columns, m)?)?;
    m.add_function(wrap_pyfunction!(render_pixels_columns, m)?)?;
    m.add_function(wrap_pyfunction!(raster_columns, m)?)?;
    m.add_function(wrap_pyfunction!(mapping_columns, m)?)?;
    m.add_function(wrap_pyfunction!(expand_preset, m)?)?;
    m.add_function(wrap_pyfunction!(preset_names, m)?)?;
    m.add_function(wrap_pyfunction!(validate_columns, m)?)?;
    m.add_class::<Mapping>()?;
    m.add_class::<Viewport>()?;
    Ok(())
}
