//! The subpixel surface: the grid marks draw on, and its string encoders.

use super::canvas::{Canvas, PlotRect, PointShape};
use super::charset::Charset;
use super::color::{Color, ColorMode, Resolved};

/// One character cell: a subpixel pattern, a text slot, independent foreground
/// and background colors, and an optional colorless fallback glyph.
///
/// Text wins over pixels when the cell prints — labels are never corrupted by marks
/// drawing underneath them.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Cell {
    bits: u8,
    text: Text,
    foreground: Color,
    background: Color,
    /// `0..=3` is a final plain shade, `0x80..=0x83` is a pending top sample,
    /// and `u8::MAX` means the styled glyph is also its plain fallback.
    plain_shade: u8,
}

/// The text slot of a cell. A wide glyph (CJK) occupies its own cell plus a
/// `Continuation` to its right; the pair is kept consistent on every overwrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Text {
    None,
    Glyph(char),
    Continuation,
}

const EMPTY: Cell = Cell {
    bits: 0,
    text: Text::None,
    foreground: Color::Default,
    background: Color::Default,
    plain_shade: u8::MAX,
};

const SHADE_RAMP: [char; 4] = ['\u{2591}', '\u{2592}', '\u{2593}', '\u{2588}'];
const PENDING_TOP: u8 = 0x80;

/// A grid of character cells addressed in subpixel coordinates.
///
/// The surface is pure raster state: origin at the top-left, y growing downward,
/// `width * height` cells at the charset's subpixel density. Drawing is infallible —
/// coordinates outside the surface clip away, non-finite coordinates draw nothing.
/// Ordinary subpixel drawing retains last-write foreground semantics. Cell patches
/// can additionally use the background channel to carry a second vertical sample.
#[derive(Clone, PartialEq)]
pub struct Surface {
    width: usize,
    height: usize,
    charset: Charset,
    columns: usize,
    rows: usize,
    cells: Vec<Cell>,
    /// An optional drawing clip in subpixel coordinates `(x0, y0, x1, y1)`, upper
    /// bounds exclusive. When set, [`Surface::set`] and [`Surface::text`] draw only
    /// inside it — used to confine marks to the plot rectangle so their ink never
    /// escapes into the axes or gutter. Chrome is drawn with no clip.
    clip: Option<(i64, i64, i64, i64)>,
}

impl Surface {
    /// Creates an empty surface of `width * height` cells encoded with `charset`.
    ///
    /// A request beyond the renderer's defensive geometry limit degrades to a 0×0
    /// surface. Use [`Surface::try_new`] when the caller needs the typed error.
    pub fn new(width: usize, height: usize, charset: Charset) -> Surface {
        Surface::try_new(width, height, charset).unwrap_or_else(|_| Surface::empty(charset))
    }

    /// Creates an empty surface, rejecting dimensions that overflow or exceed the
    /// renderer's defensive cell budget.
    pub fn try_new(width: usize, height: usize, charset: Charset) -> crate::Result<Surface> {
        let (columns, rows) = charset.pixels_per_cell();
        let count = super::frame_cells(width, height)?;
        let mut cells = Vec::new();
        super::reserve_vec(&mut cells, count, "surface cells")?;
        cells.resize(count, EMPTY);
        Ok(Surface {
            width,
            height,
            charset,
            columns,
            rows,
            cells,
            clip: None,
        })
    }

    fn empty(charset: Charset) -> Surface {
        let (columns, rows) = charset.pixels_per_cell();
        Surface {
            width: 0,
            height: 0,
            charset,
            columns,
            rows,
            cells: Vec::new(),
            clip: None,
        }
    }

    /// Confines subsequent drawing to the subpixel rectangle `[x0, x1) x [y0, y1)`.
    pub(crate) fn set_clip(&mut self, x0: i64, y0: i64, x1: i64, y1: i64) {
        self.clip = Some((x0, y0, x1, y1));
    }

    /// Removes any drawing clip.
    pub(crate) fn clear_clip(&mut self) {
        self.clip = None;
    }

    /// The size in cells as `(width, height)`.
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// The size in subpixels as `(width, height)`.
    pub fn subpixel_size(&self) -> (usize, usize) {
        // Construction bounds each cell dimension to u16 and every charset density
        // to at most four, so these products cannot overflow. Keep saturating
        // arithmetic as a final defense if those internal constants ever change.
        (
            self.width.saturating_mul(self.columns),
            self.height.saturating_mul(self.rows),
        )
    }

    /// Sets the subpixel at `(x, y)`; outside the surface or the active clip this
    /// does nothing.
    pub fn set(&mut self, x: i64, y: i64, color: Color) {
        let (sw, sh) = self.subpixel_size();
        if x < 0 || y < 0 || x >= sw as i64 || y >= sh as i64 {
            return;
        }
        if let Some((x0, y0, x1, y1)) = self.clip
            && (x < x0 || y < y0 || x >= x1 || y >= y1)
        {
            return;
        }
        let (x, y) = (x as usize, y as usize);
        let index = (y / self.rows) * self.width + x / self.columns;
        let cell = &mut self.cells[index];
        cell.bits |= self.charset.bit(x % self.columns, y % self.rows);
        cell.foreground = color;
    }

    /// Sets the subpixel nearest to `(x, y)`; non-finite coordinates draw nothing.
    pub fn dot(&mut self, x: f64, y: f64, color: Color) {
        if x.is_finite() && y.is_finite() {
            self.set(x.round() as i64, y.round() as i64, color);
        }
    }

    /// Draws a line between two subpixel positions, clipped to the surface.
    ///
    /// Non-finite endpoints draw nothing (a gap breaks a polyline before it reaches
    /// this call, but the surface defends itself regardless).
    pub fn line(&mut self, from: (f64, f64), to: (f64, f64), color: Color) {
        if !(from.0.is_finite() && from.1.is_finite() && to.0.is_finite() && to.1.is_finite()) {
            return;
        }
        let (sw, sh) = self.subpixel_size();
        if sw == 0 || sh == 0 {
            return;
        }
        // Clip to the surface, tightened to the active clip rectangle: this bounds
        // the Bresenham walk to the drawable region, so even wildly out-of-range
        // finite endpoints cost only the pixels actually on screen.
        let (mut wx0, mut wy0, mut wx1, mut wy1) =
            (0.0f64, 0.0f64, (sw - 1) as f64, (sh - 1) as f64);
        if let Some((x0, y0, x1, y1)) = self.clip {
            wx0 = wx0.max(x0 as f64);
            wy0 = wy0.max(y0 as f64);
            wx1 = wx1.min((x1 - 1) as f64);
            wy1 = wy1.min((y1 - 1) as f64);
        }
        super::canvas::trace_line(from, to, (wx0, wy0, wx1, wy1), |x, y| {
            self.set(x, y, color);
        });
    }

    /// Writes text starting at the cell `(column, row)`; cells outside clip away.
    ///
    /// Text overrides any pixels in the same cells and is measured in display
    /// columns: a wide glyph (CJK) occupies two cells, and one that would straddle
    /// the surface edge is dropped whole. Zero-width characters (combining marks) do
    /// not survive the cell grid and are dropped. Overwriting half of a wide glyph
    /// blanks its other half — alignment is never corrupted.
    pub fn text(&mut self, column: i64, row: i64, text: &str, color: Color) {
        use unicode_width::UnicodeWidthChar;

        if row < 0 || row >= self.height as i64 {
            return;
        }
        let row = row as usize;
        let mut column = column;
        for glyph in text.chars() {
            let width = glyph.width().unwrap_or(0) as i64;
            if width == 0 {
                continue;
            }
            let fits = column >= 0 && column + width <= self.width as i64;
            if fits {
                self.place(row, column as usize, Text::Glyph(glyph), color);
                for offset in 1..width {
                    self.place(row, (column + offset) as usize, Text::Continuation, color);
                }
            }
            column += width;
        }
    }

    /// Puts one text slot into a cell, breaking any wide-glyph pair it overlaps.
    fn place(&mut self, row: usize, column: usize, text: Text, color: Color) {
        self.place_styled(row, column, text, color, Color::Default, u8::MAX);
    }

    fn place_styled(
        &mut self,
        row: usize,
        column: usize,
        text: Text,
        foreground: Color,
        background: Color,
        plain_shade: u8,
    ) {
        let Some(index) = self.cell_index(row, column) else {
            return;
        };
        let base = row * self.width;
        // Overwriting a continuation orphans the wide glyph to its left.
        if self.cells[index].text == Text::Continuation && column > 0 {
            self.blank(base + column - 1);
        }
        // Overwriting a wide glyph orphans its continuation to the right.
        if column + 1 < self.width && self.cells[base + column + 1].text == Text::Continuation {
            self.blank(base + column + 1);
        }
        let cell = &mut self.cells[index];
        cell.text = text;
        cell.foreground = foreground;
        cell.background = background;
        cell.plain_shade = plain_shade;
    }

    fn cell_index(&self, row: usize, column: usize) -> Option<usize> {
        if row >= self.height || column >= self.width {
            return None;
        }
        if let Some((x0, y0, x1, y1)) = self.clip {
            let (col, row) = (column as i64, row as i64);
            let (px, py) = (self.columns as i64, self.rows as i64);
            if col < x0 / px || col >= x1 / px || row < y0 / py || row >= y1 / py {
                return None;
            }
        }
        Some(row * self.width + column)
    }

    fn blank(&mut self, index: usize) {
        let bits = self.cells[index].bits;
        self.cells[index] = Cell {
            bits,
            text: Text::Glyph(' '),
            ..EMPTY
        };
    }

    /// Encodes as plain text — no escape codes ever. Sugar for
    /// [`Surface::encode`] with [`ColorMode::Plain`].
    pub fn to_plain(&self) -> String {
        self.encode(ColorMode::Plain)
    }

    /// Encodes the surface at the color tier of `mode`.
    ///
    /// Colors resolve to what the mode can carry (RGB quantizes downhill; see
    /// [`Color`]); an SGR sequence is emitted only when a resolved foreground or
    /// background changes along a row, so colors that quantize identically share
    /// one sequence, and any colored row ends with a reset. Rows are joined by
    /// newlines with trailing spaces trimmed. In [`ColorMode::Plain`] the output
    /// carries no escapes at all.
    pub fn encode(&self, mode: ColorMode) -> String {
        self.try_encode(mode).unwrap_or_default()
    }

    /// Encodes the surface, rejecting a worst-case string beyond the defensive
    /// output budget or a failed reservation.
    pub fn try_encode(&self, mode: ColorMode) -> crate::Result<String> {
        self.encode_rows(mode, true)
    }

    /// The full-width sibling of [`Surface::try_encode`]: rows keep their
    /// trailing default-background spaces, so the encoded block covers every
    /// cell of the grid. The hybrid pixel path uses it so a block reprinted
    /// in place fully replaces its predecessor — a shorter title must erase
    /// the longer one under it. Ordinary renders keep trimming: scrollback
    /// and piped files want no trailing-space freight.
    pub(crate) fn try_encode_full_width(&self, mode: ColorMode) -> crate::Result<String> {
        self.encode_rows(mode, false)
    }

    fn encode_rows(&self, mode: ColorMode, trim: bool) -> crate::Result<String> {
        let cells = super::frame_cells(self.width, self.height)?;
        // One UTF-8 scalar plus the longest possible color transition per cell.
        // The extra row bytes cover newlines and SGR resets.
        let bytes_per_cell = match mode {
            ColorMode::Plain => 4,
            ColorMode::Ansi16 => 16,
            ColorMode::Ansi256 => 32,
            ColorMode::TrueColor => 48,
        };
        let capacity = cells
            .checked_mul(bytes_per_cell)
            .and_then(|bytes| {
                self.height
                    .checked_mul(5)
                    .and_then(|rows| bytes.checked_add(rows))
            })
            .ok_or(crate::Error::DimensionTooLarge {
                what: "encoded output bytes",
                requested: usize::MAX,
                limit: super::MAX_OUTPUT_BYTES,
            })?;
        let mut out = String::new();
        super::reserve_string(&mut out, capacity, "encoded output bytes")?;
        if mode == ColorMode::Plain {
            for row in 0..self.height {
                if row > 0 {
                    out.push('\n');
                }
                let mut kept = out.len();
                for (_, glyph, _, _) in self.row(row) {
                    out.push(glyph);
                    if glyph != ' ' {
                        kept = out.len();
                    }
                }
                if trim {
                    out.truncate(kept);
                }
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
            for (styled, _, foreground, background) in self.row(row) {
                let glyph = styled;
                let next_foreground = foreground.resolve(mode);
                let next_background = background.resolve(mode);
                let foreground_change = (glyph != ' ' && next_foreground != current_foreground)
                    .then_some(next_foreground);
                let background_change =
                    (next_background != current_background).then_some(next_background);
                Resolved::write_transition(foreground_change, background_change, &mut out);
                if foreground_change.is_some() {
                    current_foreground = next_foreground;
                }
                current_background = next_background;
                out.push(glyph);
                // A background-colored space is visible; ordinary default spaces
                // remain trimmable and may inherit the current foreground.
                if glyph != ' ' || current_background != Resolved::Default {
                    kept = out.len();
                    kept_foreground = current_foreground;
                    kept_background = current_background;
                }
            }
            if trim {
                out.truncate(kept);
                if kept_foreground != Resolved::Default || kept_background != Resolved::Default {
                    out.push_str("\x1b[0m");
                }
            } else if current_foreground != Resolved::Default
                || current_background != Resolved::Default
            {
                out.push_str("\x1b[0m");
            }
        }
        Ok(out)
    }

    /// Encodes the cell grid as HTML element content with concrete-RGB span runs.
    ///
    /// Default-colored glyphs inherit from their enclosing element. Rows are
    /// newline-joined with trailing spaces trimmed, just like [`Surface::encode`].
    #[cfg(feature = "evcxr")]
    pub(crate) fn encode_html(&self) -> String {
        use std::fmt::Write as _;

        let mut out = String::with_capacity((self.width + 32) * self.height);
        for row in 0..self.height {
            if row > 0 {
                out.push('\n');
            }
            let mut current = (None, None);
            let mut kept = out.len();
            let mut kept_style = (None, None);
            for (glyph, _, foreground, background) in self.row(row) {
                let foreground = match foreground {
                    Color::Default => None,
                    color => Some(color.to_rgb()),
                };
                let background = match background {
                    Color::Default => None,
                    color => Some(color.to_rgb()),
                };
                // Foreground is immaterial on a space, but its background is not.
                let next = (
                    if glyph == ' ' { current.0 } else { foreground },
                    background,
                );
                if next != current {
                    if current != (None, None) {
                        out.push_str("</span>");
                    }
                    if next != (None, None) {
                        out.push_str("<span style=\"");
                        if let Some((r, g, b)) = next.0 {
                            let _ = write!(out, "color:#{r:02x}{g:02x}{b:02x}");
                            if next.1.is_some() {
                                out.push(';');
                            }
                        }
                        if let Some((r, g, b)) = next.1 {
                            let _ = write!(out, "background-color:#{r:02x}{g:02x}{b:02x}");
                        }
                        out.push_str("\">");
                    }
                    current = next;
                }
                super::html::escape(glyph, &mut out);
                if glyph != ' ' || background.is_some() {
                    kept = out.len();
                    kept_style = current;
                }
            }
            out.truncate(kept);
            if kept_style != (None, None) {
                out.push_str("</span>");
            }
        }
        out
    }

    /// Every printable cell as `(column, row, glyph, foreground, background)`,
    /// skipping wide-glyph continuations (the glyph to their left covers them).
    /// For adapters that write into cell buffers instead of strings.
    #[cfg_attr(not(feature = "ratatui"), allow(dead_code))]
    pub(crate) fn cells(&self) -> impl Iterator<Item = (usize, usize, char, Color, Color)> + '_ {
        self.cells.iter().enumerate().filter_map(|(index, cell)| {
            let (row, column) = (index / self.width.max(1), index % self.width.max(1));
            match cell.text {
                Text::Continuation => None,
                Text::Glyph(glyph) => Some((column, row, glyph, cell.foreground, cell.background)),
                Text::None => Some((
                    column,
                    row,
                    self.charset.glyph(cell.bits),
                    cell.foreground,
                    cell.background,
                )),
            }
        })
    }

    /// The printable glyphs of one row, in order. Continuation cells emit nothing:
    /// the wide glyph to their left covers their column.
    fn row(&self, row: usize) -> impl Iterator<Item = (char, char, Color, Color)> + '_ {
        self.cells[row * self.width..(row + 1) * self.width]
            .iter()
            .filter_map(|cell| match cell.text {
                Text::Continuation => None,
                Text::Glyph(glyph) => Some((
                    glyph,
                    SHADE_RAMP
                        .get(cell.plain_shade as usize)
                        .copied()
                        .unwrap_or(glyph),
                    cell.foreground,
                    cell.background,
                )),
                Text::None => {
                    let glyph = self.charset.glyph(cell.bits);
                    Some((glyph, glyph, cell.foreground, cell.background))
                }
            })
    }
}

impl Canvas for Surface {
    fn set_clip(&mut self, x0: i64, y0: i64, x1: i64, y1: i64) {
        Surface::set_clip(self, x0, y0, x1, y1);
    }

    fn clear_clip(&mut self) {
        Surface::clear_clip(self);
    }

    fn dot(&mut self, x: f64, y: f64, color: Color) {
        Surface::dot(self, x, y, color);
    }

    fn point(&mut self, x: f64, y: f64, shape: PointShape, color: Color) {
        if shape == PointShape::Dot {
            Surface::dot(self, x, y, color);
            return;
        }
        if !x.is_finite() || !y.is_finite() {
            return;
        }
        let column = (x.round() as i64).div_euclid(self.columns as i64);
        let row = (y.round() as i64).div_euclid(self.rows as i64);
        let glyph = match shape {
            PointShape::Dot => unreachable!(),
            PointShape::Plus => "+",
            PointShape::Cross => "x",
            PointShape::Asterisk => "*",
            PointShape::Circle => "o",
        };
        Surface::text(self, column, row, glyph, color);
    }

    fn line(&mut self, from: (f64, f64), to: (f64, f64), color: Color) {
        Surface::line(self, from, to, color);
    }

    fn text(&mut self, column: i64, row: i64, text: &str, color: Color) {
        Surface::text(self, column, row, text, color);
    }

    /// One bar as cell-aligned columns from the zero baseline, with eighth-block
    /// partial fills at the value end (upward bars) or coarse upper-block fills
    /// (downward bars — Unicode has no lower-anchored upper ramp).
    fn bar(
        &mut self,
        span: (f64, f64),
        end: f64,
        baseline: f64,
        positive: bool,
        rect: PlotRect,
        color: Color,
    ) {
        let (px, py) = (self.columns, self.rows);
        let ramp = self.charset.fill_ramp();
        let eighths = ramp.len() == 8;
        let mut buffer = [0u8; 4];
        let baseline = baseline / py as f64;
        let end = end / py as f64;
        let left = (span.0 / px as f64).round() as i64;
        let right = ((span.1 / px as f64).round() as i64).max(left + 1);
        // Clamp to the plot columns before iterating: a bar whose span maps far
        // off-screen (distant data under a narrow domain) must not spin a giant
        // loop just to have every cell clipped away.
        let left = left.clamp(0, rect.columns as i64);
        let right = right.clamp(0, rect.columns as i64);

        for column in left..right {
            let cell_column = rect.gutter as i64 + column;
            if positive {
                // Upward: full cells from the (snapped-down) baseline, a
                // bottom-anchored partial at the top.
                let bottom = baseline.ceil().min(rect.rows as f64);
                let top = end.max(0.0);
                let mut row = top.floor();
                while row < bottom {
                    let coverage = ((row + 1.0 - top).min(1.0) * 8.0).round() as usize;
                    let glyph: Option<char> = if eighths {
                        (coverage >= 1).then(|| ramp[coverage.min(8) - 1])
                    } else {
                        (coverage >= 4).then(|| ramp[0])
                    };
                    if let Some(glyph) = glyph {
                        Surface::text(
                            self,
                            cell_column,
                            rect.top as i64 + row as i64,
                            glyph.encode_utf8(&mut buffer),
                            color,
                        );
                    }
                    row += 1.0;
                }
            } else {
                // Downward: full cells from the (snapped-up) baseline, a coarse
                // top-anchored partial at the bottom.
                let top = baseline.floor().max(0.0);
                let bottom = end.min(rect.rows as f64);
                let mut row = top;
                while row < bottom.ceil() {
                    let coverage = (bottom - row).min(1.0);
                    let glyph: Option<char> = if !eighths {
                        (coverage >= 0.5).then(|| ramp[0])
                    } else if coverage >= 7.0 / 8.0 {
                        Some('\u{2588}')
                    } else if coverage >= 0.5 {
                        Some('\u{2580}')
                    } else if coverage >= 1.0 / 8.0 {
                        Some('\u{2594}')
                    } else {
                        None
                    };
                    if let Some(glyph) = glyph {
                        Surface::text(
                            self,
                            cell_column,
                            rect.top as i64 + row as i64,
                            glyph.encode_utf8(&mut buffer),
                            color,
                        );
                    }
                    row += 1.0;
                }
            }
        }
    }

    /// The marker crossbar as text: a run of the chrome marker glyph, which reads
    /// over a same-color fill because the glyph differs from the fill texture.
    fn marker(&mut self, sx: f64, half_width: f64, sy: f64, color: Color) {
        let (px, py) = (self.columns as f64, self.rows as f64);
        let row = (sy / py).round() as i64;
        let from_cell = ((sx - half_width) / px).round() as i64;
        let to_cell = ((sx + half_width) / px).round() as i64;
        let glyph = self.charset.chrome().marker;
        for cell in from_cell..=to_cell {
            Surface::text(self, cell, row, glyph, color);
        }
    }

    fn patch_density(&self) -> (usize, usize) {
        if self.charset == Charset::Ascii {
            (1, 1)
        } else {
            (1, 2)
        }
    }

    /// Two vertically adjacent cell samples share a terminal cell: the upper
    /// half-block foreground carries the top color and its background carries the
    /// bottom color. Plain output substitutes an averaged shade-ramp glyph.
    fn patch(&mut self, column: usize, row: usize, rect: PlotRect, sample: Option<(f64, Color)>) {
        let sample = sample.map(|(intensity, color)| (((intensity * 4.0) as u8).min(3), color));
        let cell_column = rect.gutter + column;

        if self.charset == Charset::Ascii {
            if let Some((shade, color)) = sample {
                let glyph = SHADE_RAMP[shade as usize];
                self.place(rect.top + row, cell_column, Text::Glyph(glyph), color);
            }
            return;
        }

        let cell_row = rect.top + row / 2;
        if row.is_multiple_of(2) {
            if let Some((shade, color)) = sample {
                self.place_styled(
                    cell_row,
                    cell_column,
                    Text::Glyph('\u{2580}'),
                    color,
                    Color::Default,
                    PENDING_TOP + shade,
                );
            }
            return;
        }

        let Some(index) = self.cell_index(cell_row, cell_column) else {
            return;
        };
        let pending = self.cells[index]
            .plain_shade
            .checked_sub(PENDING_TOP)
            .filter(|shade| *shade < SHADE_RAMP.len() as u8)
            .map(|shade| (shade, self.cells[index].foreground));
        match (pending, sample) {
            (Some((top_shade, top_color)), Some((bottom_shade, bottom_color))) => {
                let shade = (u16::from(top_shade) + u16::from(bottom_shade)).div_ceil(2) as u8;
                let (glyph, foreground, background) = if top_color == bottom_color {
                    ('\u{2588}', top_color, Color::Default)
                } else {
                    ('\u{2580}', top_color, bottom_color)
                };
                self.place_styled(
                    cell_row,
                    cell_column,
                    Text::Glyph(glyph),
                    foreground,
                    background,
                    shade,
                );
            }
            (Some((top_shade, _)), None) => self.cells[index].plain_shade = top_shade,
            (None, Some((bottom_shade, bottom_color))) => self.place_styled(
                cell_row,
                cell_column,
                Text::Glyph('\u{2584}'),
                bottom_color,
                Color::Default,
                bottom_shade,
            ),
            (None, None) => {}
        }
    }
}

impl std::fmt::Debug for Surface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Surface")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("charset", &self.charset)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
#[path = "tests/surface_tests.rs"]
mod tests;
