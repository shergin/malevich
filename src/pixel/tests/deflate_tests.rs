//! Roundtrip tests: a minimal test-only inflater for the exact subset the
//! compressor emits (bounded fixed-Huffman blocks in a zlib wrapper),
//! plus checksum and ratio checks. If the compressor drifts outside that
//! subset, these tests fail loudly rather than silently trusting it.

use super::{adler32, inflate, zlib_compress};

fn roundtrip(raw: &[u8]) -> usize {
    let compressed = zlib_compress(raw);
    assert_eq!(inflate(&compressed), raw, "roundtrip mismatch");
    compressed.len()
}

#[test]
fn empty_and_tiny_inputs() {
    roundtrip(b"");
    roundtrip(b"a");
    roundtrip(b"abc");
    roundtrip(b"abcd");
    roundtrip(&[0, 0, 0, 0]);
    roundtrip(&[144, 200, 255, 128]); // the 9-bit literal range
}

#[test]
fn text_and_repetition() {
    roundtrip(b"the quick brown fox jumps over the lazy dog");
    roundtrip(b"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa");
    roundtrip("mixed ασκII ▲▼ and unicode ␀".as_bytes());
}

#[test]
fn a_flat_raster_collapses() {
    // A megapixel of transparent RGBA — the shape of an empty plot panel.
    let raw = vec![0u8; 4 * 1024 * 1024];
    let size = roundtrip(&raw);
    assert!(size < raw.len() / 100, "flat data must crush: {size} bytes");
}

#[test]
fn structured_raster_roundtrips() {
    // Horizontal color runs with an occasional edge, like a chart.
    let mut raw = Vec::with_capacity(400_000);
    for row in 0..250u32 {
        for column in 0..400u32 {
            let on = (column / 7 + row / 11) % 5 == 0;
            let pixel = if on { [220, 66, 52, 255] } else { [0, 0, 0, 0] };
            raw.extend_from_slice(&pixel);
        }
    }
    let size = roundtrip(&raw);
    assert!(size < raw.len() / 20, "structured data compresses: {size}");
}

#[test]
fn incompressible_noise_survives() {
    // Deterministic pseudo-noise (no Math.random in tests): xorshift.
    let mut state = 0x9E37_79B9_7F4A_7C15u64;
    let raw: Vec<u8> = (0..100_000)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 33) as u8
        })
        .collect();
    let size = roundtrip(&raw);
    // Fixed Huffman costs a hair over 8 bits per literal; noise may grow.
    assert!(size < raw.len() + raw.len() / 6 + 64);
}

#[test]
fn all_match_lengths_and_far_distances() {
    // Exercise every length code: runs of 3..=300 against a 1-distance.
    for length in [
        3usize, 4, 10, 11, 18, 19, 34, 66, 130, 131, 257, 258, 259, 300,
    ] {
        let mut raw = vec![7u8];
        raw.extend(std::iter::repeat_n(7u8, length));
        raw.push(9);
        roundtrip(&raw);
    }
    // A far match: two copies of a block separated by ~30 KiB of noise.
    let mut state = 1u64;
    let mut raw: Vec<u8> = (0..30_000)
        .map(|_| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 33) as u8
        })
        .collect();
    let block: Vec<u8> = (0..200).map(|i| (i * 7 % 251) as u8).collect();
    raw.splice(0..0, block.iter().copied());
    raw.extend_from_slice(&block);
    roundtrip(&raw);
}

#[test]
fn block_boundary_edges() {
    // One exact span, a span plus one byte, and a flat run whose boundary
    // match overshoots into the end of input (forcing an empty final
    // block). The oracle asserts every block stays under the 32 KiB
    // drain guarantee.
    roundtrip(&vec![7u8; 16 * 1024]);
    roundtrip(&vec![7u8; 16 * 1024 + 1]);
    roundtrip(&vec![7u8; 16 * 1024 + 100]);
    // Incompressible spans keep literals bounded per block too.
    let mut state = 3u64;
    let raw: Vec<u8> = (0..40_000)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 33) as u8
        })
        .collect();
    roundtrip(&raw);
}

#[test]
fn adler_reference_values() {
    assert_eq!(adler32(b""), 1);
    assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
}
