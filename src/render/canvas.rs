//! `Canvas`: the drawing-target contract shared by cell and pixel rasters.
//!
//! Mark drawing is generic over this trait and monomorphizes per target: the same
//! code draws glyph cells ([`super::Surface`]) and device pixels (the `pixel`
//! feature). The low-level ops take subpixel coordinates; the mid-level ops
//! (`bar`, `marker`, `patch`) exist because targets fill at different precision —
//! eighth-block ramps and chrome glyphs on cells, exact rectangles on pixels — and
//! the choice belongs to the target, not to the mark.

use super::color::Color;

/// A target-independent point marker selected by the public mark style.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PointShape {
    Dot,
    Plus,
    Cross,
    Asterisk,
    Circle,
}

/// The plot rectangle in cell coordinates: where marks may draw, chrome excluded.
#[derive(Debug, Clone, Copy)]
pub(crate) struct PlotRect {
    /// Cell columns left of the plot area (y labels and axis).
    pub gutter: usize,
    /// Cell rows above the plot area (title and legend).
    pub top: usize,
    /// Plot width in cells.
    pub columns: usize,
    /// Plot height in cells.
    pub rows: usize,
}

/// A raster marks draw on. Subpixel coordinates: origin top-left, y downward.
pub(crate) trait Canvas {
    /// Confines subsequent drawing to the subpixel rectangle `[x0, x1) × [y0, y1)`.
    fn set_clip(&mut self, x0: i64, y0: i64, x1: i64, y1: i64);

    /// Removes any drawing clip.
    fn clear_clip(&mut self);

    /// Scales the coverage of subsequent draws (a translucent wash for
    /// fills and bands), `0.0..=1.0`. Blending targets honor it; glyph
    /// targets have no notion of partial ink and ignore it.
    fn set_opacity(&mut self, _opacity: f64) {}

    /// Accumulating ink: overlapping draws add coverage instead of
    /// compositing over — overplotting reads as brightness (density
    /// scatters). Glyph targets ignore it.
    fn set_accumulate(&mut self, _on: bool) {}

    /// Draws a point marker centered nearest `(x, y)` at target-native fidelity.
    fn point(&mut self, x: f64, y: f64, shape: PointShape, color: Color);

    /// Draws a line between two subpixel positions, clipped to the target.
    fn line(&mut self, from: (f64, f64), to: (f64, f64), color: Color);

    /// A soft, wide under-stroke beneath a line segment — the glow pass a
    /// blending target draws before the stroke itself. Glyph targets have
    /// no notion of a fringe and ignore it.
    fn glow(&mut self, _from: (f64, f64), _to: (f64, f64), _color: Color) {}

    /// Writes text starting at the cell `(column, row)`; cells outside clip away.
    fn text(&mut self, column: i64, row: i64, text: &str, color: Color);

    /// An annotation anchored at a subpixel position: `cell` is the
    /// target's subpixels per cell. Glyph targets snap to the containing
    /// cell; pixel targets place the ink exactly, vertically centered on
    /// the anchor.
    fn note(&mut self, x: f64, y: f64, cell: (f64, f64), text: &str, color: Color) {
        if cell.0 > 0.0 && cell.1 > 0.0 {
            self.text(
                (x / cell.0).round() as i64,
                (y / cell.1).round() as i64,
                text,
                color,
            );
        }
    }

    /// Fills one bar covering `span` in plot-local subpixel columns, from the
    /// baseline to the value end (both plot-local subpixel rows), at the target's
    /// precision. `positive` anchors the partial fill: bottom-up above the
    /// baseline, top-down below it.
    fn bar(
        &mut self,
        span: (f64, f64),
        end: f64,
        baseline: f64,
        positive: bool,
        rect: PlotRect,
        color: Color,
    );

    /// Draws the range marker crossbar centered on `sx` with `half_width` reach at
    /// subpixel row `sy` (frame-absolute), such that it reads over a fill of the
    /// same color.
    fn marker(&mut self, sx: f64, half_width: f64, sy: f64, color: Color);

    /// The cell-patch sampling density per terminal cell. Glyph targets encode two
    /// vertical samples with foreground/background half blocks; pixel targets use
    /// their full device-pixel cell size.
    fn patch_density(&self) -> (usize, usize);

    /// Fills the cell patch at patch-grid `(column, row)` inside `rect`.
    /// `sample` carries normalized intensity and color; `None` records a gap.
    fn patch(&mut self, column: usize, row: usize, rect: PlotRect, sample: Option<(f64, Color)>);
}

/// Walks the segment `from → to` as subpixels: Liang–Barsky clip to `window`
/// `(x0, y0, x1, y1)` (inclusive bounds), then a Bresenham walk calling `plot`
/// per subpixel. The clip bounds the walk, so even wildly out-of-range finite
/// endpoints cost only the subpixels actually inside the window.
pub(crate) fn trace_line(
    from: (f64, f64),
    to: (f64, f64),
    window: (f64, f64, f64, f64),
    mut plot: impl FnMut(i64, i64),
) {
    let (wx0, wy0, wx1, wy1) = window;
    if wx0 > wx1 || wy0 > wy1 {
        return;
    }
    let Some((from, to)) = clip_segment(from, to, (wx0, wy0), (wx1, wy1)) else {
        return;
    };
    let (x1, y1) = (to.0.round() as i64, to.1.round() as i64);
    let (mut x, mut y) = (from.0.round() as i64, from.1.round() as i64);
    let dx = (x1 - x).abs();
    let dy = -(y1 - y).abs();
    let sx = if x < x1 { 1 } else { -1 };
    let sy = if y < y1 { 1 } else { -1 };
    let mut error = dx + dy;
    loop {
        plot(x, y);
        if x == x1 && y == y1 {
            return;
        }
        let doubled = 2 * error;
        if doubled >= dy {
            error += dy;
            x += sx;
        }
        if doubled <= dx {
            error += dx;
            y += sy;
        }
    }
}

/// Liang–Barsky clipping of the segment `from → to` against the rectangle
/// `[min.0, max.0] x [min.1, max.1]`; `None` when the segment lies entirely outside.
fn clip_segment(
    from: (f64, f64),
    to: (f64, f64),
    min: (f64, f64),
    max: (f64, f64),
) -> Option<((f64, f64), (f64, f64))> {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let mut enter = 0.0f64;
    let mut exit = 1.0f64;
    let tests = [
        (-dx, from.0 - min.0),
        (dx, max.0 - from.0),
        (-dy, from.1 - min.1),
        (dy, max.1 - from.1),
    ];
    for (p, q) in tests {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                if r > exit {
                    return None;
                }
                enter = enter.max(r);
            } else {
                if r < enter {
                    return None;
                }
                exit = exit.min(r);
            }
        }
    }
    if enter > exit {
        return None;
    }
    Some((
        (from.0 + enter * dx, from.1 + enter * dy),
        (from.0 + exit * dx, from.1 + exit * dy),
    ))
}
