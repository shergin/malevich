//! The plot pipeline: retained descriptions, frames, and rendering.

mod chrome;
mod draw;
pub(crate) mod frame;
mod grid;
mod layout;
mod mapping;
#[allow(clippy::module_inception)]
mod plot;
mod resolve;
mod viewport;

pub use frame::Frame;
pub use grid::Grid;
pub use mapping::{Mapping, Panel};
pub use plot::Plot;
pub use viewport::Viewport;

#[cfg(all(test, feature = "pixel"))]
pub(crate) use draw::dash_segment as test_dash_segment;
