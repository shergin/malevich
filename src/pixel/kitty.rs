//! The kitty graphics encoder: zlib-deflated RGBA over APC escapes,
//! chunked base64.
//!
//! Direct transmission (`a=T`, `f=32`) with `o=z` compression — both in the
//! protocol's core, no files, no shared memory. Compression is what makes
//! big panels viable: a Retina-sized frame is tens of raw megabytes, and
//! the terminal's escape parser, not the encoder, pays for every byte.
//! Transparent pixels carry alpha 0, so the terminal background shows
//! through undrawn panel area.
//! `C=1` keeps the cursor where it is (the render path brackets the image with
//! DECSC/DECRC regardless), and `q=2` suppresses terminal responses, which a
//! one-way render string could never read.

use std::fmt::Write as _;

use super::base64;
use super::deflate;
#[cfg(test)]
use super::render::Image;

#[cfg(test)]
pub(crate) fn encode(image: &Image) -> String {
    let mut rgba = Vec::with_capacity(image.pixels.len() * 4);
    for pixel in &image.pixels {
        match pixel {
            Some((r, g, b)) => rgba.extend_from_slice(&[*r, *g, *b, 255]),
            None => rgba.extend_from_slice(&[0, 0, 0, 0]),
        }
    }
    encode_rgba(image.width, image.height, (0, 0), &rgba, None)
}

/// Encodes raw RGBA. A nonzero `placement` (columns, rows) pins the image
/// to that cell rectangle via `c=`/`r=`: the terminal scales as needed, so
/// a host may transmit fewer device pixels than the rectangle holds (a
/// speed knob — Retina-sized panels cost the terminal real decode and
/// upload time) and stays correct even when cell-size detection was off.
///
/// An `id` makes the transmission a replacement: the image data replaces
/// what the id held, and the fixed placement id (`p=1`) makes the placement
/// replace too — one placement per (image, placement) pair is the protocol's
/// rule, and terminals (Ghostty included) treat an *unspecified* placement id
/// as "add another placement", which would stack every repaint on screen.
/// With both keys the swap is atomic: no visible gap, no separate delete.
/// Interactive hosts assign one stable id per panel; scrollback strings stay
/// id-less, since each print is a new image.
pub(crate) fn encode_rgba(
    width: usize,
    height: usize,
    placement: (usize, usize),
    rgba: &[u8],
    id: Option<u32>,
) -> String {
    if width == 0 || height == 0 {
        return String::new();
    }
    let payload = base64::encode(&deflate::zlib_compress(rgba));
    let mut out = String::new();
    // The protocol caps escape payloads at 4096 bytes; the first chunk carries
    // the control keys, the rest only their continuation flag.
    let mut chunks = payload.as_bytes().chunks(4096).peekable();
    let mut first = true;
    while let Some(chunk) = chunks.next() {
        let more = u8::from(chunks.peek().is_some());
        out.push_str("\x1b_G");
        if first {
            let _ = write!(out, "a=T,f=32,o=z,s={width},v={height},");
            if let Some(id) = id {
                let _ = write!(out, "i={id},p=1,");
            }
            if placement.0 > 0 && placement.1 > 0 {
                let _ = write!(out, "c={},r={},", placement.0, placement.1);
            }
            out.push_str("C=1,q=2,");
            first = false;
        }
        let _ = write!(out, "m={more};");
        // Base64 is ASCII; the chunk boundary cannot split a character.
        out.push_str(std::str::from_utf8(chunk).expect("base64 is ASCII"));
        out.push_str("\x1b\\");
    }
    out
}

#[cfg(test)]
#[path = "tests/kitty_tests.rs"]
mod tests;
