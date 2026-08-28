//! A minimal zlib/DEFLATE compressor: LZ77 over a 32 KiB window into
//! fixed-Huffman blocks (RFC 1950/1951).
//!
//! Plot rasters are flat — long runs of identical pixels — which LZ77
//! collapses into a few length/distance pairs; fixed Huffman then costs
//! nothing to describe. The point is transport weight: a Retina-sized kitty
//! panel is tens of megabytes raw, and the terminal's escape parser is the
//! slowest link in the whole pipeline. Dynamic Huffman would shave a few
//! percent more at real complexity; stored blocks (the previous approach)
//! shave nothing.

/// Compresses `raw` into a zlib stream (header, fixed-Huffman deflate
/// blocks over bounded spans, adler-32 trailer).
pub(crate) fn zlib_compress(raw: &[u8]) -> Vec<u8> {
    let mut writer = BitWriter {
        // Flat input compresses ~100×; start smaller and let growth handle
        // incompressible data.
        out: Vec::with_capacity(64 + raw.len() / 32),
        buffer: 0,
        bits: 0,
    };
    // zlib: CM=8 CINFO=7, check bits for FLEVEL=0.
    writer.out.extend_from_slice(&[0x78, 0x01]);
    let mut head = vec![-1i64; 1 << HASH_BITS];
    let mut prev = vec![-1i64; WINDOW];
    let mut position = 0usize;
    loop {
        let target = (position + BLOCK_SPAN).min(raw.len());
        let final_block = target == raw.len();
        writer.put_bits(u32::from(final_block), 1);
        writer.put_bits(1, 2); // fixed-Huffman
        position = compress_span(raw, position, target, &mut writer, &mut head, &mut prev);
        put_literal(&mut writer, 256); // end of block
        if final_block {
            break;
        }
    }
    writer.flush();
    let mut out = writer.out;
    out.extend_from_slice(&adler32(raw).to_be_bytes());
    out
}

/// The zlib adler-32 checksum (shared with the PNG encoder).
pub(crate) fn adler32(bytes: &[u8]) -> u32 {
    const MOD: u32 = 65521;
    let (mut a, mut b) = (1u32, 0u32);
    for chunk in bytes.chunks(5552) {
        // 5552 is the classic largest run before the sums can overflow u32.
        for &byte in chunk {
            a += u32::from(byte);
            b += a;
        }
        a %= MOD;
        b %= MOD;
    }
    (b << 16) | a
}

const WINDOW: usize = 32 * 1024;
/// Input bytes per deflate block, the conventional zlib flush cadence.
/// Bounded blocks alone do NOT protect a streaming inflater: block output
/// positions drift, so their spans still straddle window-aligned drain
/// boundaries — a real fred stream carried four hundred straddling matches
/// under the old assumption. The actual protection is at match emission
/// (`compress_span` truncates every match at 32 KiB output boundaries).
const BLOCK_SPAN: usize = 16 * 1024;
const MIN_MATCH: usize = 4;
/// Matches longer than this index only their fringes (see `compress_span`).
const MAX_INSERT: usize = 64;
const MAX_MATCH: usize = 258;
/// Hash-chain candidates examined per position: the speed/ratio knob. Flat
/// rasters find their long match on the first try; this only bounds the
/// pathological middle ground.
const MAX_CHAIN: usize = 48;
const HASH_BITS: u32 = 15;

/// Emits LZ77 symbols for `raw[start..target]`; a match at the boundary
/// may overrun `target` by up to `MAX_MATCH`, so the caller gets the
/// position actually reached. `head`/`prev` persist across spans: block
/// boundaries reset only the entropy coder, never the 32 KiB dictionary,
/// so matches keep reaching into earlier blocks.
///
/// Chains live in a window-sized ring (zlib's layout): memory stays
/// constant however large the raster. An aliased slot can only hold an
/// older, smaller position, so the distance guard below stays sound and
/// `MAX_CHAIN` bounds any wasted walk.
fn compress_span(
    raw: &[u8],
    start: usize,
    target: usize,
    writer: &mut BitWriter,
    head: &mut [i64],
    prev: &mut [i64],
) -> usize {
    let mut position = start;
    while position < target {
        let (mut best_len, mut best_dist) = (0usize, 0usize);
        // A streaming inflater drains its window-sized output buffer at
        // 32 KiB boundaries, and Zig ≤0.15's flate (Ghostty ≤1.3.1) cannot
        // suspend a fixed-Huffman match across that drain — the whole
        // transmission aborts. So no match may cross a window-aligned output
        // position: truncate at the boundary (one shortened match per
        // 32 KiB), and a remainder too short to stand as a match falls
        // through to literals, which suspend fine.
        let room = WINDOW - (position & (WINDOW - 1));
        if position + MIN_MATCH <= raw.len() {
            let mut candidate = head[hash(raw, position)];
            let mut tries = MAX_CHAIN;
            while candidate >= 0 && tries > 0 {
                let start = candidate as usize;
                if start >= position || position - start > WINDOW {
                    break;
                }
                let len = match_length(raw, start, position);
                if len > best_len {
                    best_len = len;
                    best_dist = position - start;
                    if len >= MAX_MATCH {
                        break;
                    }
                }
                candidate = prev[start & (WINDOW - 1)];
                tries -= 1;
            }
        }
        let best_len = best_len.min(room);
        if best_len >= MIN_MATCH {
            debug_assert!(
                (position & (WINDOW - 1)) + best_len <= WINDOW,
                "a match may never cross a window-aligned output boundary"
            );
            put_match(writer, best_len, best_dist);
            // Index covered positions so later matches can reach into this
            // run — but only the fringes of a long match (zlib's
            // `max_insert_length` trick): hashing every byte of a flat run
            // dominates the whole compressor, and interior anchors add
            // nothing that the head and tail fringes don't. A run that
            // outlives the window self-heals: the chain walk breaks, one
            // literal re-anchors it.
            let end = (position + best_len).min(raw.len().saturating_sub(MIN_MATCH - 1));
            if best_len <= MAX_INSERT {
                for at in position..end {
                    insert(raw, at, head, prev);
                }
            } else {
                let fringe = 8;
                for at in position..(position + fringe).min(end) {
                    insert(raw, at, head, prev);
                }
                for at in end.saturating_sub(fringe).max(position + fringe)..end {
                    insert(raw, at, head, prev);
                }
            }
            position += best_len;
        } else {
            put_literal(writer, u16::from(raw[position]));
            if position + MIN_MATCH <= raw.len() {
                insert(raw, position, head, prev);
            }
            position += 1;
        }
    }
    position
}

#[inline]
fn hash(raw: &[u8], position: usize) -> usize {
    let word = u32::from(raw[position])
        | u32::from(raw[position + 1]) << 8
        | u32::from(raw[position + 2]) << 16
        | u32::from(raw[position + 3]) << 24;
    (word.wrapping_mul(0x9E37_79B1) >> (32 - HASH_BITS)) as usize
}

#[inline]
fn insert(raw: &[u8], position: usize, head: &mut [i64], prev: &mut [i64]) {
    let slot = hash(raw, position);
    prev[position & (WINDOW - 1)] = head[slot];
    head[slot] = position as i64;
}

#[inline]
fn match_length(raw: &[u8], start: usize, position: usize) -> usize {
    let limit = (raw.len() - position).min(MAX_MATCH);
    let mut len = 0;
    // Compare a word at a time; the xor's first set byte is the mismatch.
    // (`from_le_bytes` puts byte 0 in the low bits on every platform, so
    // `trailing_zeros` counts matching leading bytes.)
    while len + 8 <= limit {
        let a = u64::from_le_bytes(raw[start + len..start + len + 8].try_into().unwrap());
        let b = u64::from_le_bytes(raw[position + len..position + len + 8].try_into().unwrap());
        let diff = a ^ b;
        if diff != 0 {
            return len + (diff.trailing_zeros() / 8) as usize;
        }
        len += 8;
    }
    while len < limit && raw[start + len] == raw[position + len] {
        len += 1;
    }
    len
}

// RFC 1951 §3.2.5 — length codes 257..=285 and distance codes 0..=29.
const LENGTH_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

fn put_match(writer: &mut BitWriter, len: usize, dist: usize) {
    let code = LENGTH_BASE.partition_point(|&base| usize::from(base) <= len) - 1;
    put_literal(writer, 257 + code as u16);
    writer.put_bits(
        (len - usize::from(LENGTH_BASE[code])) as u32,
        u32::from(LENGTH_EXTRA[code]),
    );
    let dcode = DIST_BASE.partition_point(|&base| usize::from(base) <= dist) - 1;
    writer.put_huffman(dcode as u32, 5);
    writer.put_bits(
        (dist - usize::from(DIST_BASE[dcode])) as u32,
        u32::from(DIST_EXTRA[dcode]),
    );
}

/// Writes one literal/length symbol in the fixed Huffman code
/// (RFC 1951 §3.2.6).
fn put_literal(writer: &mut BitWriter, symbol: u16) {
    let (code, bits) = match symbol {
        0..=143 => (0b0011_0000 + u32::from(symbol), 8),
        144..=255 => (0b1_1001_0000 + u32::from(symbol) - 144, 9),
        256..=279 => (u32::from(symbol) - 256, 7),
        _ => (0b1100_0000 + u32::from(symbol) - 280, 8),
    };
    writer.put_huffman(code, bits);
}

/// LSB-first bit packing; Huffman codes go most-significant-bit first, as
/// the format demands.
struct BitWriter {
    out: Vec<u8>,
    buffer: u64,
    bits: u32,
}

impl BitWriter {
    #[inline]
    fn put_bits(&mut self, value: u32, count: u32) {
        self.buffer |= u64::from(value) << self.bits;
        self.bits += count;
        while self.bits >= 8 {
            self.out.push((self.buffer & 0xFF) as u8);
            self.buffer >>= 8;
            self.bits -= 8;
        }
    }

    #[inline]
    fn put_huffman(&mut self, code: u32, count: u32) {
        let mut reversed = 0u32;
        for bit in 0..count {
            reversed |= ((code >> bit) & 1) << (count - 1 - bit);
        }
        self.put_bits(reversed, count);
    }

    fn flush(&mut self) {
        if self.bits > 0 {
            self.out.push((self.buffer & 0xFF) as u8);
            self.buffer = 0;
            self.bits = 0;
        }
    }
}

#[cfg(test)]
pub(crate) use oracle::inflate;

/// Test-only reference inflater for the exact subset this compressor
/// emits — shared by this module's tests and the PNG tests.
#[cfg(test)]
mod oracle {
    use super::adler32;

    /// Decodes the zlib stream `zlib_compress` produces. Panics on anything
    /// malformed — this is a test oracle, not a library.
    pub(crate) fn inflate(stream: &[u8]) -> Vec<u8> {
        assert_eq!(stream[0], 0x78, "zlib CMF");
        assert_eq!(
            (u16::from(stream[0]) << 8 | u16::from(stream[1])) % 31,
            0,
            "zlib FCHECK"
        );
        let body = &stream[2..stream.len() - 4];
        let mut reader = Bits {
            data: body,
            position: 0,
        };
        let mut out = Vec::new();
        loop {
            let final_block = reader.take(1);
            let kind = reader.take(2);
            assert_eq!(kind, 1, "fixed-Huffman block expected");
            let block_start = out.len();
            loop {
                let symbol = read_fixed_literal(&mut reader);
                match symbol {
                    0..=255 => out.push(symbol as u8),
                    256 => break,
                    257..=285 => {
                        let index = (symbol - 257) as usize;
                        let length = usize::from(LENGTH_BASE[index])
                            + reader.take(u32::from(LENGTH_EXTRA[index])) as usize;
                        let dcode = reader.take_huffman(5) as usize;
                        let distance = usize::from(DIST_BASE[dcode])
                            + reader.take(u32::from(DIST_EXTRA[dcode])) as usize;
                        assert!(distance <= out.len(), "distance into the void");
                        for _ in 0..length {
                            out.push(out[out.len() - distance]);
                        }
                    }
                    _ => panic!("impossible symbol {symbol}"),
                }
            }
            // The compatibility contract behind `BLOCK_SPAN`: streaming
            // decoders are only guaranteed 32 KiB of output room per block
            // (Zig ≤0.15 aborts beyond it), so no block may decode past it.
            assert!(
                out.len() - block_start <= 32 * 1024,
                "block decoded {} bytes, past the 32 KiB drain guarantee",
                out.len() - block_start
            );
            if final_block == 1 {
                break;
            }
        }
        let trailer = &stream[stream.len() - 4..];
        assert_eq!(trailer, adler32(&out).to_be_bytes(), "adler-32 mismatch");
        out
    }

    fn read_fixed_literal(reader: &mut Bits) -> u16 {
        // RFC 1951 §3.2.6, decoded by code-length ranges.
        let mut code = reader.take_huffman(7);
        if code <= 0b001_0111 {
            return (256 + code) as u16;
        }
        code = code << 1 | reader.take(1);
        if (0b0011_0000..=0b1011_1111).contains(&code) {
            return (code - 0b0011_0000) as u16;
        }
        if (0b1100_0000..=0b1100_0111).contains(&code) {
            return (280 + code - 0b1100_0000) as u16;
        }
        code = code << 1 | reader.take(1);
        assert!(
            (0b1_1001_0000..=0b1_1111_1111).contains(&code),
            "bad code {code:b}"
        );
        (144 + code - 0b1_1001_0000) as u16
    }

    const LENGTH_BASE: [u16; 29] = [
        3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115,
        131, 163, 195, 227, 258,
    ];
    const LENGTH_EXTRA: [u8; 29] = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
    ];
    const DIST_BASE: [u16; 30] = [
        1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
        2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
    ];
    const DIST_EXTRA: [u8; 30] = [
        0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12,
        13, 13,
    ];

    /// LSB-first bit reader; Huffman codes are read MSB-first.
    struct Bits<'a> {
        data: &'a [u8],
        position: usize,
    }

    impl Bits<'_> {
        fn take(&mut self, count: u32) -> u32 {
            let mut value = 0u32;
            for bit in 0..count {
                let byte = self.data[self.position / 8];
                value |= u32::from(byte >> (self.position % 8) & 1) << bit;
                self.position += 1;
            }
            value
        }

        fn take_huffman(&mut self, count: u32) -> u32 {
            let mut value = 0u32;
            for _ in 0..count {
                value = value << 1 | self.take(1);
            }
            value
        }
    }
}

#[cfg(test)]
#[path = "tests/deflate_tests.rs"]
mod tests;
