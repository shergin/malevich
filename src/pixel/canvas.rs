//! The pixel canvas: the device-pixel raster the plot panel draws on.
//!
//! Same raster convention as the cell surface — origin top-left, y downward,
//! drawing is infallible (outside clips away, non-finite draws nothing) — but a
//! subpixel here is one device pixel. A pixel holds `Option<Color>`: `None` is
//! transparency, so the terminal background shows through everywhere marks did
//! not draw, exactly as it does between glyphs in cell output.

use super::font;
use crate::render::{Canvas, Color, PlotRect, PointShape};

/// A device-pixel raster covering the whole frame; encoders crop the plot panel.
pub(crate) struct PixelCanvas {
    width: usize,
    height: usize,
    cell: (usize, usize),
    /// Line width in device pixels, derived from the cell density like the
    /// font scale: a stroke is proportional to the cell, not to the device
    /// pixel, so retina-dense cells do not thin the ink. 1 at the classic
    /// 8×16 cell; heavier as cells grow.
    stroke: i64,
    /// Point-marker side in device pixels: one step above the stroke, so
    /// scatter dots read over lines at every density.
    point: i64,
    pixels: Vec<Option<Color>>,
    /// An optional drawing clip in pixel coordinates `(x0, y0, x1, y1)`, upper
    /// bounds exclusive — the same contract as the cell surface's clip.
    clip: Option<(i64, i64, i64, i64)>,
}

impl PixelCanvas {
    /// An empty (fully transparent) canvas of `columns × rows` cells, each
    /// `cell.0 × cell.1` device pixels.
    #[cfg(test)]
    pub(crate) fn new(columns: usize, rows: usize, cell: (usize, usize)) -> PixelCanvas {
        PixelCanvas::try_new(columns, rows, cell, None).unwrap_or_else(|_| PixelCanvas::empty(cell))
    }

    /// Fallible construction for caller-controlled frame and cell geometry.
    pub(crate) fn try_new(
        columns: usize,
        rows: usize,
        cell: (usize, usize),
        stroke: Option<u8>,
    ) -> crate::Result<PixelCanvas> {
        let width = columns
            .checked_mul(cell.0)
            .ok_or(crate::Error::DimensionTooLarge {
                what: "device-pixel width",
                requested: usize::MAX,
                limit: crate::render::MAX_DEVICE_PIXELS,
            })?;
        let height = rows
            .checked_mul(cell.1)
            .ok_or(crate::Error::DimensionTooLarge {
                what: "device-pixel height",
                requested: usize::MAX,
                limit: crate::render::MAX_DEVICE_PIXELS,
            })?;
        let count = crate::render::checked_area(
            "device-pixel count",
            width,
            height,
            crate::render::MAX_DEVICE_PIXELS,
        )?;
        // The host's override wins (a reduced-density raster scaled up on
        // screen wants its native ink weight back); otherwise derive from
        // the cell, 1 at the classic 8x16.
        let stroke = match stroke {
            Some(width) if width > 0 => i64::from(width),
            _ => (cell.1.saturating_add(8) / 16).max(1) as i64,
        };
        let mut pixels = Vec::new();
        crate::render::reserve_vec(&mut pixels, count, "device-pixel canvas")?;
        pixels.resize(count, None);
        Ok(PixelCanvas {
            width,
            height,
            cell,
            stroke,
            point: stroke + 1,
            pixels,
            clip: None,
        })
    }

    #[cfg(test)]
    fn empty(cell: (usize, usize)) -> PixelCanvas {
        PixelCanvas {
            width: 0,
            height: 0,
            cell,
            stroke: 1,
            point: 2,
            pixels: Vec::new(),
            clip: None,
        }
    }

    /// Stamps a `side × side` square centered on `(x, y)` — the pen the stroke
    /// ops draw with; clipping applies per pixel.
    fn stamp(&mut self, x: i64, y: i64, side: i64, color: Color) {
        let (x0, y0) = (x - side / 2, y - side / 2);
        for dy in 0..side {
            for dx in 0..side {
                self.set(x0 + dx, y0 + dy, color);
            }
        }
    }

    /// The size in device pixels as `(width, height)`.
    #[cfg(test)]
    pub(crate) fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// The cell size in device pixels as `(width, height)`.
    pub(crate) fn cell(&self) -> (usize, usize) {
        self.cell
    }

    /// The pixel at `(x, y)`: `None` outside the canvas or where nothing drew.
    pub(crate) fn get(&self, x: usize, y: usize) -> Option<Color> {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x]
        } else {
            None
        }
    }

    fn inside(&self, x: i64, y: i64) -> bool {
        if x < 0 || y < 0 || x >= self.width as i64 || y >= self.height as i64 {
            return false;
        }
        match self.clip {
            Some((x0, y0, x1, y1)) => x >= x0 && y >= y0 && x < x1 && y < y1,
            None => true,
        }
    }

    fn set(&mut self, x: i64, y: i64, color: Color) {
        if self.inside(x, y) {
            self.pixels[y as usize * self.width + x as usize] = Some(color);
        }
    }

    fn clear(&mut self, x: i64, y: i64) {
        if self.inside(x, y) {
            self.pixels[y as usize * self.width + x as usize] = None;
        }
    }

    /// The line-clip window: canvas bounds tightened to the active clip.
    fn window(&self) -> (f64, f64, f64, f64) {
        let (mut wx0, mut wy0) = (0.0f64, 0.0f64);
        let mut wx1 = self.width.saturating_sub(1) as f64;
        let mut wy1 = self.height.saturating_sub(1) as f64;
        if let Some((x0, y0, x1, y1)) = self.clip {
            wx0 = wx0.max(x0 as f64);
            wy0 = wy0.max(y0 as f64);
            wx1 = wx1.min((x1 - 1) as f64);
            wy1 = wy1.min((y1 - 1) as f64);
        }
        (wx0, wy0, wx1, wy1)
    }
}

impl Canvas for PixelCanvas {
    fn set_clip(&mut self, x0: i64, y0: i64, x1: i64, y1: i64) {
        self.clip = Some((x0, y0, x1, y1));
    }

    fn clear_clip(&mut self) {
        self.clip = None;
    }

    fn dot(&mut self, x: f64, y: f64, color: Color) {
        if x.is_finite() && y.is_finite() {
            let point = self.point;
            self.stamp(x.round() as i64, y.round() as i64, point, color);
        }
    }

    fn point(&mut self, x: f64, y: f64, shape: PointShape, color: Color) {
        if !x.is_finite() || !y.is_finite() {
            return;
        }
        let (x, y) = (x.round() as i64, y.round() as i64);
        match shape {
            PointShape::Dot => self.stamp(x, y, self.point, color),
            PointShape::Plus => {
                for offset in -self.point..=self.point {
                    self.set(x + offset, y, color);
                    self.set(x, y + offset, color);
                }
            }
            PointShape::Cross => {
                for offset in -self.point..=self.point {
                    self.set(x + offset, y + offset, color);
                    self.set(x + offset, y - offset, color);
                }
            }
            PointShape::Asterisk => {
                for offset in -self.point..=self.point {
                    self.set(x + offset, y, color);
                    self.set(x, y + offset, color);
                    self.set(x + offset, y + offset, color);
                    self.set(x + offset, y - offset, color);
                }
            }
            PointShape::Circle => {
                // A one-pixel ring at the marker radius: pixels whose squared
                // distance lands within half a pixel of the radius.
                let radius = self.point.max(1);
                let target = radius * radius;
                for dy in -radius..=radius {
                    for dx in -radius..=radius {
                        let distance = dx * dx + dy * dy;
                        if distance >= target - radius && distance <= target + radius {
                            self.set(x + dx, y + dy, color);
                        }
                    }
                }
            }
        }
    }

    fn line(&mut self, from: (f64, f64), to: (f64, f64), color: Color) {
        if !(from.0.is_finite() && from.1.is_finite() && to.0.is_finite() && to.1.is_finite()) {
            return;
        }
        if self.width == 0 || self.height == 0 {
            return;
        }
        let stroke = self.stroke;
        if stroke == 1 {
            crate::render::trace_line(from, to, self.window(), |x, y| self.set(x, y, color));
        } else {
            let window = self.window();
            crate::render::trace_line(from, to, window, |x, y| self.stamp(x, y, stroke, color));
        }
    }

    /// Text through the baked font: glyphs advance by whole cells — labels land
    /// where cell rendering would put them — drawn at an integer scale that fits
    /// the cell box, vertically and horizontally centered within each cell.
    fn text(&mut self, column: i64, row: i64, text: &str, color: Color) {
        use unicode_width::UnicodeWidthChar;

        let (cw, ch) = (self.cell.0 as i64, self.cell.1 as i64);
        if cw == 0 || ch == 0 {
            return;
        }
        let scale = ((cw / 8).min(ch / 8)).max(1);
        let mut column = column;
        for glyph in text.chars() {
            let width = glyph.width().unwrap_or(0) as i64;
            if width == 0 {
                continue;
            }
            if let Some(bitmap) = font::glyph(glyph) {
                let x0 = column * cw + (cw * width - 8 * scale) / 2;
                let y0 = row * ch + (ch - 8 * scale) / 2;
                for (glyph_row, bits) in bitmap.iter().enumerate() {
                    for glyph_col in 0..8i64 {
                        if bits >> glyph_col & 1 == 1 {
                            for dy in 0..scale {
                                for dx in 0..scale {
                                    self.set(
                                        x0 + glyph_col * scale + dx,
                                        y0 + glyph_row as i64 * scale + dy,
                                        color,
                                    );
                                }
                            }
                        }
                    }
                }
            }
            column += width;
        }
    }

    /// One bar as an exact device-pixel rectangle — no cell snapping, at least
    /// one pixel wide and tall so a nonzero value never vanishes.
    fn bar(
        &mut self,
        span: (f64, f64),
        end: f64,
        baseline: f64,
        positive: bool,
        rect: PlotRect,
        color: Color,
    ) {
        if !(span.0.is_finite() && span.1.is_finite() && end.is_finite() && baseline.is_finite()) {
            return;
        }
        let x0 = (rect.gutter * self.cell.0) as i64;
        let y0 = (rect.top * self.cell.1) as i64;
        let left = x0 + span.0.round() as i64;
        let right = (x0 + span.1.round() as i64).max(left + 1);
        let (top, bottom) = if positive {
            (end, baseline)
        } else {
            (baseline, end)
        };
        let top = y0 + top.round() as i64;
        let bottom = (y0 + bottom.round() as i64).max(top + 1);
        for y in top..bottom {
            for x in left..right {
                self.set(x, y, color);
            }
        }
    }

    /// The marker crossbar reads by *clearing*: a thin band of terminal
    /// background across the fill — a gap, so contrast is guaranteed against any
    /// fill color, which a same-color stroke could not be.
    fn marker(&mut self, sx: f64, half_width: f64, sy: f64, _color: Color) {
        if !(sx.is_finite() && half_width.is_finite() && sy.is_finite()) {
            return;
        }
        let thickness = (self.cell.1 as i64 / 8).max(1);
        let from = (sx - half_width).round() as i64;
        let to = (sx + half_width).round() as i64;
        let top = sy.round() as i64 - thickness / 2;
        for y in top..top + thickness {
            for x in from..=to {
                self.clear(x, y);
            }
        }
    }

    /// Cells layers sample per device pixel: real-resolution heatmaps.
    fn patch_density(&self) -> (usize, usize) {
        self.cell
    }

    fn patch(&mut self, column: usize, row: usize, rect: PlotRect, sample: Option<(f64, Color)>) {
        if let Some((_, color)) = sample {
            self.set(
                (rect.gutter * self.cell.0 + column) as i64,
                (rect.top * self.cell.1 + row) as i64,
                color,
            );
        }
    }
}

#[cfg(test)]
#[path = "tests/canvas_tests.rs"]
mod tests;
