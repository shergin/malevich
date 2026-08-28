//! Hybrid pixel rendering: text chrome woven around an image panel.
//!
//! The output is one string in three movements: the full text grid (chrome as
//! always, panel cells blank), a cursor walk to the panel origin, the image, and
//! a walk back. Cursor movement is bracketed by DECSC/DECRC — where a terminal
//! leaves the cursor after an image varies by protocol and emulator, and
//! restoring the saved position sidesteps the whole question. Columns are
//! addressed absolutely (CHA), rows only ever relatively — so the block prints
//! correctly at any *row*, and `column` anchors it horizontally: every text row
//! and the cursor walk jump there first, which is what lets a host paste a
//! pixel plot beside other content (cells left, pixels right).

use std::fmt::Write as _;

use super::{Graphics, PixelCanvas, Protocol, iterm, kitty, sixel};
use crate::plot::{Frame, Plot};
use crate::render::PlotRect;

pub(crate) fn render(plot: &Plot<'_>, frame: &Frame, graphics: &Graphics, column: usize) -> String {
    try_render(plot, frame, graphics, column)
        .unwrap_or_else(|_| at_column(&plot.render(frame), column).unwrap_or_default())
}

/// The bounded hybrid render path used by the public fallible API.
pub(crate) fn try_render(
    plot: &Plot<'_>,
    frame: &Frame,
    graphics: &Graphics,
    column: usize,
) -> crate::Result<String> {
    try_render_mapped(plot, frame, graphics, column).map(|(block, _)| block)
}

/// [`try_render`] plus the render's resolved [`Mapping`] — the one-pass seam
/// the ratatui widget's pixel path caches its hit-testing geometry from.
pub(crate) fn try_render_mapped(
    plot: &Plot<'_>,
    frame: &Frame,
    graphics: &Graphics,
    column: usize,
) -> crate::Result<(String, crate::plot::Mapping)> {
    let cell = (graphics.cell_size.0 as usize, graphics.cell_size.1 as usize);
    validate_geometry(frame, graphics.protocol, cell)?;
    if cell.0 == 0 || cell.1 == 0 {
        // No pixel geometry to draw into: degrade to ordinary cell output.
        let block = at_column(&plot.try_render_unvalidated(frame)?, column)?;
        return Ok((block, plot.mapping(frame)));
    }
    let (surface, canvas, rect, mapping) =
        plot.try_rasterize_hybrid(frame, cell, graphics.stroke)?;
    // Full-width rows: the block owns its whole rectangle, so reprinting
    // it in place replaces the previous block entirely (a shorter title
    // erases the longer one it lands on).
    let mut out = at_column(&surface.try_encode_full_width(frame.color)?, column)?;
    if rect.columns == 0 || rect.rows == 0 {
        return Ok((out, mapping));
    }
    let payload = if graphics.protocol == Protocol::Sixel {
        // Sixel has no alpha channel: it consumes the thresholded image,
        // where only solid ink survives.
        let image = crop(&canvas, rect)?;
        if image.width == 0 || image.height == 0 {
            return Ok((out, mapping));
        }
        sixel::encode(&image)
    } else {
        // The alpha-capable protocols take the raster verbatim — one pass,
        // no intermediate buffer, anti-aliased coverage intact.
        let (width, height, rgba) = crop_rgba(&canvas, rect)?;
        if width == 0 || height == 0 {
            return Ok((out, mapping));
        }
        match graphics.protocol {
            Protocol::Kitty => kitty::encode_rgba(width, height, (rect.columns, rect.rows), &rgba),
            Protocol::ITerm2 => iterm::encode_rgba(width, height, rect.columns, rect.rows, &rgba),
            Protocol::Sixel => unreachable!("sixel took the image path above"),
        }
    };
    let extra = payload
        .len()
        .checked_add(96)
        .ok_or(crate::Error::DimensionTooLarge {
            what: "pixel output bytes",
            requested: usize::MAX,
            limit: crate::render::MAX_OUTPUT_BYTES,
        })?;
    let total = out
        .len()
        .checked_add(extra)
        .ok_or(crate::Error::DimensionTooLarge {
            what: "pixel output bytes",
            requested: usize::MAX,
            limit: crate::render::MAX_OUTPUT_BYTES,
        })?;
    crate::render::checked_dimension("pixel output bytes", total, crate::render::MAX_OUTPUT_BYTES)?;
    crate::render::reserve_string(&mut out, extra, "pixel output")?;
    out.push_str("\x1b7");
    // CHA is 1-based; land on the block's column, then walk to the panel.
    let anchor = column
        .checked_add(1)
        .ok_or(crate::Error::DimensionTooLarge {
            what: "pixel column anchor",
            requested: usize::MAX,
            limit: usize::MAX - 1,
        })?;
    let _ = write!(out, "\x1b[{anchor}G");
    let up = frame.height - 1 - rect.top;
    if up > 0 {
        let _ = write!(out, "\x1b[{up}A");
    }
    if rect.gutter > 0 {
        let _ = write!(out, "\x1b[{}C", rect.gutter);
    }
    out.push_str(&payload);
    out.push_str("\x1b8");
    Ok((out, mapping))
}

fn validate_geometry(frame: &Frame, protocol: Protocol, cell: (usize, usize)) -> crate::Result<()> {
    crate::render::frame_cells(frame.width, frame.height)?;
    if cell.0 == 0 || cell.1 == 0 {
        return Ok(());
    }
    let width = frame
        .width
        .checked_mul(cell.0)
        .ok_or(crate::Error::DimensionTooLarge {
            what: "device-pixel width",
            requested: usize::MAX,
            limit: crate::render::MAX_DEVICE_PIXELS,
        })?;
    let height = frame
        .height
        .checked_mul(cell.1)
        .ok_or(crate::Error::DimensionTooLarge {
            what: "device-pixel height",
            requested: usize::MAX,
            limit: crate::render::MAX_DEVICE_PIXELS,
        })?;
    let pixels = crate::render::checked_area(
        "device-pixel count",
        width,
        height,
        crate::render::MAX_DEVICE_PIXELS,
    )?;
    // Conservative protocol-specific upper bounds. Sixel may revisit a row for
    // several color planes; raw-RGBA protocols base64-expand four bytes per pixel.
    let factor = match protocol {
        Protocol::Sixel => 64,
        Protocol::Kitty | Protocol::ITerm2 => 6,
    };
    let estimated = pixels
        .checked_mul(factor)
        .and_then(|bytes| {
            frame
                .height
                .checked_mul(32)
                .and_then(|rows| bytes.checked_add(rows))
        })
        .and_then(|bytes| bytes.checked_add(65_536))
        .ok_or(crate::Error::DimensionTooLarge {
            what: "estimated pixel output bytes",
            requested: usize::MAX,
            limit: crate::render::MAX_OUTPUT_BYTES,
        })?;
    crate::render::checked_dimension(
        "estimated pixel output bytes",
        estimated,
        crate::render::MAX_OUTPUT_BYTES,
    )
}

/// Anchors every row of a rendered block at `column`: each row starts with an
/// absolute-column jump (CHA), so printing the block leaves anything to its
/// left untouched. Column 0 anchors too — pixel blocks live in interactive
/// hosts, and raw-mode LF does not return the carriage, so an unanchored
/// flush-left block would staircase across the screen.
fn at_column(text: &str, column: usize) -> crate::Result<String> {
    let anchor = column
        .checked_add(1)
        .ok_or(crate::Error::DimensionTooLarge {
            what: "pixel column anchor",
            requested: usize::MAX,
            limit: usize::MAX - 1,
        })?;
    let jump = format!("\x1b[{anchor}G");
    let rows = text.split('\n').count();
    let capacity = jump
        .len()
        .checked_mul(rows)
        .and_then(|jumps| text.len().checked_add(jumps))
        .ok_or(crate::Error::DimensionTooLarge {
            what: "anchored pixel text bytes",
            requested: usize::MAX,
            limit: crate::render::MAX_OUTPUT_BYTES,
        })?;
    let mut out = String::new();
    crate::render::reserve_string(&mut out, capacity, "anchored pixel text")?;
    for (index, row) in text.split('\n').enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&jump);
        out.push_str(row);
    }
    Ok(out)
}

/// A concrete color, resolved for pixel output.
pub(crate) type Rgb = (u8, u8, u8);

/// A row-major RGB image with transparency: `None` pixels stay undrawn, so the
/// terminal background shows through.
pub(crate) struct Image {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Option<Rgb>>,
}

/// The panel rectangle of the canvas as raw RGBA bytes, alpha 0 where
/// nothing drew — the kitty fast path (one pass, no [`Image`] detour).
fn crop_rgba(canvas: &PixelCanvas, rect: PlotRect) -> crate::Result<(usize, usize, Vec<u8>)> {
    let (x0, y0, width, height, count) = crop_geometry(canvas, rect)?;
    let bytes = count
        .checked_mul(4)
        .ok_or(crate::Error::DimensionTooLarge {
            what: "pixel crop bytes",
            requested: usize::MAX,
            limit: crate::render::MAX_DEVICE_PIXELS,
        })?;
    let mut rgba = Vec::new();
    crate::render::reserve_vec(&mut rgba, bytes, "pixel crop")?;
    for y in 0..height {
        for x in 0..width {
            rgba.extend_from_slice(&canvas.rgba(x0 + x, y0 + y));
        }
    }
    Ok((width, height, rgba))
}

/// The panel rectangle of the canvas as an [`Image`] of solid-ink pixels
/// (coverage at least half) — the shape sixel wants, having no alpha.
fn crop(canvas: &PixelCanvas, rect: PlotRect) -> crate::Result<Image> {
    let (x0, y0, width, height, count) = crop_geometry(canvas, rect)?;
    let mut pixels = Vec::new();
    crate::render::reserve_vec(&mut pixels, count, "pixel crop")?;
    for y in 0..height {
        for x in 0..width {
            pixels.push(canvas.get(x0 + x, y0 + y).map(|color| color.to_rgb()));
        }
    }
    Ok(Image {
        width,
        height,
        pixels,
    })
}

/// The checked pixel geometry of a crop: origin, size, and pixel count.
fn crop_geometry(
    canvas: &PixelCanvas,
    rect: PlotRect,
) -> crate::Result<(usize, usize, usize, usize, usize)> {
    let (cw, ch) = canvas.cell();
    let x0 = rect
        .gutter
        .checked_mul(cw)
        .ok_or(crate::Error::DimensionTooLarge {
            what: "pixel crop x offset",
            requested: usize::MAX,
            limit: crate::render::MAX_DEVICE_PIXELS,
        })?;
    let y0 = rect
        .top
        .checked_mul(ch)
        .ok_or(crate::Error::DimensionTooLarge {
            what: "pixel crop y offset",
            requested: usize::MAX,
            limit: crate::render::MAX_DEVICE_PIXELS,
        })?;
    let width = rect
        .columns
        .checked_mul(cw)
        .ok_or(crate::Error::DimensionTooLarge {
            what: "pixel crop width",
            requested: usize::MAX,
            limit: crate::render::MAX_DEVICE_PIXELS,
        })?;
    let height = rect
        .rows
        .checked_mul(ch)
        .ok_or(crate::Error::DimensionTooLarge {
            what: "pixel crop height",
            requested: usize::MAX,
            limit: crate::render::MAX_DEVICE_PIXELS,
        })?;
    let count = crate::render::checked_area(
        "pixel crop count",
        width,
        height,
        crate::render::MAX_DEVICE_PIXELS,
    )?;
    Ok((x0, y0, width, height, count))
}

#[cfg(test)]
#[path = "tests/render_tests.rs"]
mod tests;
