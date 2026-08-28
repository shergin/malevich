//! Rendering: the subpixel surface, charset codecs, and string encoders.
//!
//! Marks draw on a [`Surface`] in subpixel coordinates (raster convention: origin
//! top-left, y grows downward); a [`Charset`] codec maps each cell's subpixel pattern
//! to one glyph; encoders turn the cell grid into a plain or ANSI string. Nothing in
//! this module touches a terminal, and nothing in it panics: drawing outside the
//! surface clips, non-finite coordinates draw nothing.

mod canvas;
mod charset;
// Visible to the crate so tests elsewhere can drive the quantizers directly.
pub(crate) mod color;
#[cfg(feature = "evcxr")]
mod html;
mod limits;
mod surface;
mod width;

pub(crate) use canvas::{Canvas, PlotRect, PointShape};
pub use charset::Charset;
pub use color::{Color, ColorMode};
#[cfg(feature = "pixel")]
pub(crate) use color::{ansi256_to_rgb, rgb_to_256};
pub(crate) use limits::{
    MAX_DEVICE_PIXELS, MAX_OUTPUT_BYTES, frame_cells, reserve as reserve_vec, reserve_string,
};
#[cfg(feature = "pixel")]
pub(crate) use limits::{area as checked_area, dimension as checked_dimension};
pub use surface::Surface;
pub(crate) use width::{display_width, display_width_ansi, fit_width_with};
