//! Terminal plotting: a small grammar of marks, honest axes, millions of points.
//!
//! Eight marks ([`Line`], [`Points`], [`Bars`], [`Area`], [`Cells`], [`Range`],
//! [`Rule`], [`Text`]) compose over shared scales into the basic chart catalog;
//! presets like [`line()`], [`hist`], [`box_plot`], and [`violin`] are one-line fronts
//! over that grammar. Large series aggregate to the raster before drawing (M4 —
//! pixel-exact for lines), axes use extended-Wilkinson tick placement with
//! exact-decimal labels, and everything renders to a plain `String` — colored for
//! your terminal via [`Frame::detect`], deterministic via [`Frame::plain`].
//!
//! ```
//! use malevich::{Frame, Line, Plot, Rule};
//!
//! let steps: Vec<f64> = (0..100).map(f64::from).collect();
//! let loss: Vec<f64> = steps.iter().map(|s| 4.0 * (-0.05 * s).exp() + 0.4).collect();
//! let chart = Plot::new()
//!     .layer(Line::xy(&steps[..], &loss[..]).label("loss"))
//!     .layer(Rule::h(0.5).label("target"))
//!     .title("training");
//! println!("{}", chart.render(&Frame::plain(60, 14)));
//! ```
//!
//! A plot is a plain value: `Clone + Send + Sync`, no global state, rendering is a
//! pure function of plot and frame. [`Plot::render`] never fails — it sheds what it
//! cannot draw — so building a plot inline needs no error handling. For a spec that
//! arrives from deserialization or configuration, [`Plot::validate`] and
//! [`Plot::try_render`] report the first problem as a typed [`Error`] instead.
//!
//! # Failure model
//!
//! Functions that return [`Result`] are the strict boundary: invalid data shapes,
//! configuration, numeric domains, and bounded resource requests return a typed
//! [`Error`] rather than asserting. A `try_` name distinguishes that checked twin
//! when the same operation also has a convenience form, such as
//! [`Cells::matrix`] / [`Cells::try_matrix`] and [`Plot::render`] /
//! [`Plot::try_render`]. The `_with` suffix means “configured with an options
//! value”; its return type, not the suffix, states whether the call is fallible.
//! Plain mark constructors and one-call presets may panic on their documented
//! programmer invariants, such as unequal paired channels. Infallible rendering is
//! intentionally different: it sheds malformed retained content and excessive
//! output instead of panicking.
//!
//! The modules follow the concepts (each defined in
//! the repository's `docs/terminology.md`): [`mark`] for the primitives, [`stat`] for
//! online accumulators, reducers, and batch transforms,
//! [`scale`] for ticks and colormaps, [`render`] for the subpixel surface and
//! charsets, [`stream`] for live charts, [`data`] for the ingestion rim.
//!
//! The gallery in `EXAMPLES.md` shows every chart type with its source, and
//! `cargo run --example showcase` renders a colored tour in your terminal.
//!
//! # Features
//!
//! - `evcxr` — rich HTML display for Evcxr Jupyter notebooks through
//!   [`Plot::evcxr_display`], plus deterministic [`Plot::to_html`] rendering for
//!   custom notebook frames and the [`evcxr`] module, whose stdout protocol and
//!   card colors let a crate draw its own types on the same background.
//! - `ndarray` — one-dimensional arrays and views plot directly; contiguous
//!   storage is zero-copy.
//! - `pixel` — the plot panel as a real image (sixel, kitty graphics, or iTerm2
//!   inline PNG) with text chrome around it: [`Plot::render_pixels`], the
//!   [`pixel::Capabilities`] query API, and [`Plot::render_best`] picking the
//!   best tier the terminal offers.
//! - `ratatui` — [`PlotWidget`], a `ratatui` widget rendering any plot into a
//!   `Buffer`; rendered stateful with a [`PlotState`], it becomes interactive:
//!   hit-testing through the cached [`Mapping`], zoom and pan through a
//!   [`Viewport`], and default mouse gestures fed via [`PlotState::on_mouse`].
//!   Combined with `pixel`, the widget draws its panel as a real image
//!   ([`PlotWidget::graphics`], emitted by
//!   [`Graphics::present`](pixel::Graphics::present)) with the interaction
//!   chrome rendered into the image itself.
//! - `serde` — every spec type (plots, marks, scales, themes, frames)
//!   round-trips through serde; `Document` is the versioned persistent envelope,
//!   gaps survive JSON as `null`, and function-backed lines refuse to serialize
//!   rather than lie.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "ratatui")]
mod adapter;
pub mod data;
#[cfg(feature = "serde")]
mod document;
mod error;
#[cfg(feature = "evcxr")]
pub mod evcxr;
pub mod mark;
mod numeric;
#[cfg(feature = "pixel")]
pub mod pixel;
pub mod plot;
mod presets;
pub mod render;
pub mod scale;
#[cfg(all(test, feature = "serde"))]
mod serde_tests;
pub mod stat;
pub mod stream;
mod theme;

#[cfg(feature = "ratatui")]
pub use adapter::{Mouse, MouseButton, PlotState, PlotWidget};
#[cfg(feature = "serde")]
pub use document::{Document, DocumentKind};
pub use error::{Error, Result};
pub use mark::{
    Area, Bars, Cells, Dash, Line, LineStyle, Mark, PointStyle, Points, Range, Rule, Text,
};
pub use plot::{Frame, Grid, Mapping, Panel, Plot, Viewport};
pub use presets::{
    ContourLevels, ContourOptions, DensityOptions, EcdfOptions, HeatmapOptions, Histogram2dOptions,
    HistogramOptions, TrendOptions, ViolinOptions, bar, box_plot, contour, contour_with, density,
    density_with, ecdf, ecdf_with, error_bars, error_bars_asymmetric, heatmap, heatmap_with, hist,
    hist_with, hist2d, hist2d_with, line, quiver, scatter, stairs, trend, trend_with, violin,
    violin_with,
};
pub use render::{Charset, Color, ColorMode};
pub use scale::Scale;
pub use theme::Theme;
