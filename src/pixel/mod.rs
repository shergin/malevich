//! Pixel graphics: the plot panel as a real image over sixel, kitty, or iTerm2.
//!
//! The top rung of the resolution ladder (feature `pixel`). Output stays hybrid:
//! title, axes, tick labels, and legend render as text cells exactly as always —
//! crisp, selectable, colored by the terminal theme — and only the plot rectangle
//! becomes an image, drawn at device-pixel resolution. Marks draw through the same
//! generic code as cell output; the scales simply map into a denser raster, so M4
//! aggregation, clipping, and gap semantics all carry over unchanged.
//!
//! Nothing here touches a terminal at render time: rendering is a pure function
//! of the plot, the frame, and the graphics configuration, deterministic and
//! snapshot-testable like every other output path.

mod base64;
mod canvas;
mod capabilities;
mod deflate;
mod detect;
mod font;
mod iterm;
mod kitty;
mod png;
mod probe;
mod query;
mod render;
mod sixel;

pub(crate) use canvas::PixelCanvas;
pub use capabilities::{Capabilities, Source};
pub(crate) use render::{render, try_render};

/// The image protocol to emit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Protocol {
    /// DEC sixel (1987): palette-indexed bands of six vertical pixels. The most
    /// widely spoken pixel protocol — xterm, iTerm2, WezTerm, foot, Konsole,
    /// Windows Terminal, VS Code.
    Sixel,
    /// The kitty graphics protocol: raw RGBA with real alpha, the most capable
    /// of the three. kitty, Ghostty, WezTerm, Konsole.
    Kitty,
    /// iTerm2 inline images (OSC 1337): a PNG pinned to the panel's cell box.
    /// iTerm2, WezTerm, Konsole, VS Code, mintty.
    ITerm2,
}

/// How to draw the plot panel as pixels: which protocol, at what cell size.
///
/// A plain value, like [`Frame`](crate::Frame): rendering with the same plot,
/// frame, and graphics is deterministic, and nothing here touches a terminal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub struct Graphics {
    /// The image protocol to emit.
    pub protocol: Protocol,
    /// The terminal's cell size in device pixels `(width, height)` — the scale
    /// between the cell grid the chrome lives on and the panel raster.
    pub cell_size: (u16, u16),
}

impl Graphics {
    /// `protocol` at the common default cell size, 8×16 device pixels. Override
    /// with [`Graphics::cell_size`] when the terminal's real geometry is known —
    /// a mismatch only scales the image, it never misplaces the chrome.
    pub fn new(protocol: Protocol) -> Graphics {
        Graphics {
            protocol,
            cell_size: (8, 16),
        }
    }

    /// Sets the protocol, keeping everything else.
    #[must_use]
    pub fn protocol(mut self, protocol: Protocol) -> Graphics {
        self.protocol = protocol;
        self
    }

    /// Sets the cell size in device pixels. A zero dimension falls back to
    /// ordinary cell rendering (rendering never fails).
    #[must_use]
    pub fn cell_size(mut self, width: u16, height: u16) -> Graphics {
        self.cell_size = (width, height);
        self
    }
}
