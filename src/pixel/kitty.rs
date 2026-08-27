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
    encode_rgba(image.width, image.height, &rgba)
}

pub(crate) fn encode_rgba(width: usize, height: usize, rgba: &[u8]) -> String {
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
            let _ = write!(out, "a=T,f=32,o=z,s={width},v={height},C=1,q=2,");
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
