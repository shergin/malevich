//! The pixel canvas: the device-pixel raster the plot panel draws on.
//!
//! Same raster convention as the cell surface — origin top-left, y downward,
//! drawing is infallible (outside clips away, non-finite draws nothing) — but a
//! subpixel here is one device pixel. A pixel is straight (non-premultiplied)
//! RGBA with alpha as coverage: alpha 0 is transparency, so the terminal
//! background shows through everywhere marks did not draw, exactly as it does
//! between glyphs in cell output — and fractional coverage composites over
//! any background the terminal has.

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
    /// Straight RGBA per pixel; alpha 0 is bare canvas.
    pixels: Vec<[u8; 4]>,
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
        pixels.resize(count, [0; 4]);
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

    /// An anti-aliased stroked segment with round caps: coverage falls off
    /// over one pixel at the stroke's edge, and endpoints are sub-pixel —
    /// no rounding before rasterization. Long segments render in bounded
    /// chunks so a diagonal never scans its full bounding square; interior
    /// chunk caps land inside the stroke, where same-color blending
    /// absorbs them exactly.
    fn aa_segment(&mut self, from: (f64, f64), to: (f64, f64), half_width: f64, rgb: (u8, u8, u8)) {
        let (dx, dy) = (to.0 - from.0, to.1 - from.1);
        let length = (dx * dx + dy * dy).sqrt();
        if !length.is_finite() {
            return;
        }
        let chunks = (length / 32.0).ceil().max(1.0) as usize;
        for chunk in 0..chunks {
            let t0 = chunk as f64 / chunks as f64;
            let t1 = (chunk + 1) as f64 / chunks as f64;
            self.aa_span(
                (from.0 + dx * t0, from.1 + dy * t0),
                (from.0 + dx * t1, from.1 + dy * t1),
                half_width,
                rgb,
            );
        }
    }

    /// One bounded span of [`aa_segment`](Self::aa_segment): every pixel
    /// within reach blends by its distance to the segment. Pixel centers
    /// sit at integer coordinates, matching where cell-snapped drawing
    /// always put its ink.
    fn aa_span(&mut self, a: (f64, f64), b: (f64, f64), half_width: f64, rgb: (u8, u8, u8)) {
        let (wx0, wy0, wx1, wy1) = self.window();
        let pad = half_width + 1.0;
        let x0 = (a.0.min(b.0) - pad).floor().max(wx0) as i64;
        let x1 = (a.0.max(b.0) + pad).ceil().min(wx1) as i64;
        let y0 = (a.1.min(b.1) - pad).floor().max(wy0) as i64;
        let y1 = (a.1.max(b.1) + pad).ceil().min(wy1) as i64;
        let (ex, ey) = (b.0 - a.0, b.1 - a.1);
        let len2 = ex * ex + ey * ey;
        for py in y0..=y1 {
            for px in x0..=x1 {
                let (vx, vy) = (px as f64 - a.0, py as f64 - a.1);
                let t = if len2 > 0.0 {
                    ((vx * ex + vy * ey) / len2).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                let (cx, cy) = (vx - t * ex, vy - t * ey);
                let distance = (cx * cx + cy * cy).sqrt();
                let coverage = (half_width + 0.5 - distance).clamp(0.0, 1.0);
                if coverage > 0.0 {
                    self.blend(px, py, rgb, coverage);
                }
            }
        }
    }

    /// An anti-aliased filled disc at a sub-pixel center.
    fn aa_disc(&mut self, x: f64, y: f64, radius: f64, rgb: (u8, u8, u8)) {
        self.aa_span((x, y), (x, y), radius, rgb);
    }

    /// An anti-aliased one-pixel ring at a sub-pixel center.
    fn aa_ring(&mut self, x: f64, y: f64, radius: f64, rgb: (u8, u8, u8)) {
        let (wx0, wy0, wx1, wy1) = self.window();
        let pad = radius + 1.0;
        let x0 = (x - pad).floor().max(wx0) as i64;
        let x1 = (x + pad).ceil().min(wx1) as i64;
        let y0 = (y - pad).floor().max(wy0) as i64;
        let y1 = (y + pad).ceil().min(wy1) as i64;
        for py in y0..=y1 {
            for px in x0..=x1 {
                let (vx, vy) = (px as f64 - x, py as f64 - y);
                let distance = (vx * vx + vy * vy).sqrt();
                let coverage = (1.0 - (distance - radius).abs()).clamp(0.0, 1.0);
                if coverage > 0.0 {
                    self.blend(px, py, rgb, coverage);
                }
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

    /// Solid ink at `(x, y)`: the color where coverage exceeds half,
    /// `None` outside the canvas, where nothing drew, or where only an
    /// anti-aliased fringe landed (an exactly half-covered pixel is a
    /// fringe, not ink). Encoders needing exact coverage read
    /// [`PixelCanvas::rgba`].
    pub(crate) fn get(&self, x: usize, y: usize) -> Option<Color> {
        let [r, g, b, a] = self.rgba(x, y);
        (a > 128).then_some(Color::Rgb(r, g, b))
    }

    /// The raw straight-RGBA pixel; `[0; 4]` outside the canvas.
    pub(crate) fn rgba(&self, x: usize, y: usize) -> [u8; 4] {
        if x < self.width && y < self.height {
            self.pixels[y * self.width + x]
        } else {
            [0; 4]
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
            let (r, g, b) = color.to_rgb();
            self.pixels[y as usize * self.width + x as usize] = [r, g, b, 255];
        }
    }

    /// Composites `coverage` of `rgb` over the pixel: full coverage
    /// replaces, same-color partials keep the strongest coverage (so a
    /// polyline's overlapping joints never darken), different colors
    /// blend source-over in straight alpha.
    fn blend(&mut self, x: i64, y: i64, rgb: (u8, u8, u8), coverage: f64) {
        if !self.inside(x, y) {
            return;
        }
        let a_new = (coverage.clamp(0.0, 1.0) * 255.0).round() as u8;
        if a_new == 0 {
            return;
        }
        let index = y as usize * self.width + x as usize;
        let [r, g, b, a_old] = self.pixels[index];
        let new = [rgb.0, rgb.1, rgb.2, a_new];
        self.pixels[index] = if a_old == 0 || a_new == 255 {
            new
        } else if (r, g, b) == rgb {
            [r, g, b, a_old.max(a_new)]
        } else {
            let (sa, da) = (f64::from(a_new) / 255.0, f64::from(a_old) / 255.0);
            let out_a = sa + da * (1.0 - sa);
            let channel = |source: u8, dest: u8| {
                let blended = (f64::from(source) * sa + f64::from(dest) * da * (1.0 - sa)) / out_a;
                blended.round() as u8
            };
            [
                channel(rgb.0, r),
                channel(rgb.1, g),
                channel(rgb.2, b),
                (out_a * 255.0).round() as u8,
            ]
        };
    }

    fn clear(&mut self, x: i64, y: i64) {
        if self.inside(x, y) {
            self.pixels[y as usize * self.width + x as usize] = [0; 4];
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
            self.aa_disc(x, y, self.point as f64 / 2.0, color.to_rgb());
        }
    }

    fn point(&mut self, x: f64, y: f64, shape: PointShape, color: Color) {
        if !x.is_finite() || !y.is_finite() {
            return;
        }
        let (sx, sy) = (x, y);
        let (x, y) = (x.round() as i64, y.round() as i64);
        match shape {
            PointShape::Dot => self.aa_disc(sx, sy, self.point as f64 / 2.0, color.to_rgb()),
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
                self.aa_ring(sx, sy, self.point.max(1) as f64, color.to_rgb());
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
        self.aa_segment(from, to, self.stroke as f64 / 2.0, color.to_rgb());
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
