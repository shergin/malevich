//! Encoded cell grid of one render: glyphs and colors, chrome included.
//!
//! Marks draw on a [`super::Surface`] in subpixels; a charset codec then maps
//! each cell to a glyph. A [`Raster`] is that encoded snapshot — what a TUI
//! host paints into its own buffer, without decoding an ANSI string. Rendering
//! a plot to a string is rasterize-then-encode; [`Raster::encode`] is the
//! second half, so a host that wants cells and a host that wants a `String`
//! share one grid.

use super::color::{Color, ColorMode, Resolved};
use super::limits;

/// One encoded cell of a [`Raster`].
///
/// A wide glyph (CJK) occupies its own cell plus a continuation to its right
/// (`columns == 0`). Hosts that write into a cell buffer skip continuations
/// the way the string encoder does: the glyph to the left covers them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RasterCell {
    /// The printable glyph, or a space on a continuation cell.
    pub glyph: char,
    /// Independent of [`Self::background`].
    pub foreground: Color,
    /// Independent of [`Self::foreground`].
    pub background: Color,
    /// Display columns this cell occupies: `2` for a wide glyph, `1` for an
    /// ordinary cell, `0` for the continuation of a wide glyph to its left.
    pub columns: u8,
}

/// Encoded cell grid of one render: glyphs and colors, chrome included.
///
/// Obtained from [`crate::Plot::raster`]. A plain value (`Clone + Send + Sync`)
/// describing one `(plot, frame)` pair. Row-major, `width * height` cells,
/// origin top-left. Continuation cells are present so a host can address every
/// column; string encoding skips them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Raster {
    width: usize,
    height: usize,
    cells: Vec<RasterCell>,
}

impl Raster {
    /// An empty raster: no cells, nothing to paint.
    pub fn empty() -> Raster {
        Raster {
            width: 0,
            height: 0,
            cells: Vec::new(),
        }
    }

    pub(crate) fn from_cells(width: usize, height: usize, cells: Vec<RasterCell>) -> Raster {
        debug_assert_eq!(cells.len(), width.saturating_mul(height));
        Raster {
            width,
            height,
            cells,
        }
    }

    /// Width in cells, chrome included.
    pub fn width(&self) -> usize {
        self.width
    }

    /// Height in cells, chrome included.
    pub fn height(&self) -> usize {
        self.height
    }

    /// Whether the raster has no cells.
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// The cell at `(column, row)`, or `None` outside the grid.
    pub fn cell(&self, column: usize, row: usize) -> Option<RasterCell> {
        if column >= self.width || row >= self.height {
            return None;
        }
        self.cells.get(row * self.width + column).copied()
    }

    /// Every cell, row-major, including wide-glyph continuations.
    pub fn cells(&self) -> &[RasterCell] {
        &self.cells
    }

    /// Encodes as plain text — no escape codes. Sugar for
    /// [`Raster::encode`] with [`ColorMode::Plain`].
    pub fn to_plain(&self) -> String {
        self.encode(ColorMode::Plain)
    }

    /// Encodes the raster at the color tier of `mode`.
    ///
    /// Matches [`super::Surface::encode`]: trailing default-background spaces
    /// are trimmed, colored rows reset, continuations are skipped. A plot's
    /// [`crate::Plot::render`] string is this encoding of its raster.
    pub fn encode(&self, mode: ColorMode) -> String {
        self.try_encode(mode).unwrap_or_default()
    }

    /// Encodes the raster, rejecting a payload beyond the defensive output budget.
    pub fn try_encode(&self, mode: ColorMode) -> crate::Result<String> {
        let count = limits::frame_cells(self.width, self.height)?;
        let bytes_per_cell = match mode {
            ColorMode::Plain => 4,
            ColorMode::Ansi16 => 16,
            ColorMode::Ansi256 => 32,
            ColorMode::TrueColor => 48,
        };
        let capacity = count
            .checked_mul(bytes_per_cell)
            .and_then(|bytes| {
                self.height
                    .checked_mul(5)
                    .and_then(|rows| bytes.checked_add(rows))
            })
            .ok_or(crate::Error::DimensionTooLarge {
                what: "encoded output bytes",
                requested: usize::MAX,
                limit: limits::MAX_OUTPUT_BYTES,
            })?;
        let mut out = String::new();
        limits::reserve_string(&mut out, capacity, "encoded output bytes")?;
        if mode == ColorMode::Plain {
            for row in 0..self.height {
                if row > 0 {
                    out.push('\n');
                }
                let mut kept = out.len();
                for cell in self.row(row) {
                    out.push(cell.glyph);
                    if cell.glyph != ' ' {
                        kept = out.len();
                    }
                }
                out.truncate(kept);
            }
            return Ok(out);
        }
        for row in 0..self.height {
            if row > 0 {
                out.push('\n');
            }
            let mut current_foreground = Resolved::Default;
            let mut current_background = Resolved::Default;
            let mut kept = out.len();
            let mut kept_foreground = Resolved::Default;
            let mut kept_background = Resolved::Default;
            for cell in self.row(row) {
                let next_foreground = cell.foreground.resolve(mode);
                let next_background = cell.background.resolve(mode);
                let foreground_change = (cell.glyph != ' '
                    && next_foreground != current_foreground)
                    .then_some(next_foreground);
                let background_change =
                    (next_background != current_background).then_some(next_background);
                Resolved::write_transition(foreground_change, background_change, &mut out);
                if foreground_change.is_some() {
                    current_foreground = next_foreground;
                }
                current_background = next_background;
                out.push(cell.glyph);
                if cell.glyph != ' ' || current_background != Resolved::Default {
                    kept = out.len();
                    kept_foreground = current_foreground;
                    kept_background = current_background;
                }
            }
            out.truncate(kept);
            if kept_foreground != Resolved::Default || kept_background != Resolved::Default {
                out.push_str("\x1b[0m");
            }
        }
        Ok(out)
    }

    fn row(&self, row: usize) -> impl Iterator<Item = RasterCell> + '_ {
        let start = row * self.width;
        self.cells[start..start + self.width]
            .iter()
            .copied()
            .filter(|cell| cell.columns != 0)
    }
}
