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

/// Compresses `raw` into a zlib stream (header, one final fixed-Huffman
/// deflate block, adler-32 trailer).
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
    // Fixed-Huffman block, final.
    writer.put_bits(1, 1);
    writer.put_bits(1, 2);
    compress_block(raw, &mut writer);
    put_literal(&mut writer, 256); // end of block
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
const MIN_MATCH: usize = 4;
const MAX_MATCH: usize = 258;
/// Hash-chain candidates examined per position: the speed/ratio knob. Flat
/// rasters find their long match on the first try; this only bounds the
/// pathological middle ground.
const MAX_CHAIN: usize = 48;
const HASH_BITS: u32 = 15;

fn compress_block(raw: &[u8], writer: &mut BitWriter) {
    let mut head = vec![-1i64; 1 << HASH_BITS];
    // Chains live in a window-sized ring (zlib's layout): memory stays
    // constant however large the raster. An aliased slot can only hold an
    // older, smaller position, so the distance guard below stays sound and
    // `MAX_CHAIN` bounds any wasted walk.
    let mut prev = vec![-1i64; WINDOW];
    let mut position = 0usize;
    while position < raw.len() {
        let (mut best_len, mut best_dist) = (0usize, 0usize);
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
        if best_len >= MIN_MATCH {
            put_match(writer, best_len, best_dist);
            // Index every covered position so later matches can reach into
            // this run.
            let end = (position + best_len).min(raw.len().saturating_sub(MIN_MATCH - 1));
            for at in position..end {
                insert(raw, at, &mut head, &mut prev);
            }
            position += best_len;
        } else {
            put_literal(writer, u16::from(raw[position]));
            if position + MIN_MATCH <= raw.len() {
                insert(raw, position, &mut head, &mut prev);
            }
            position += 1;
        }
    }
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
    while len < limit && raw[start + len] == raw[position + len] {
        len += 1;
    }
    len
}

// RFC 1951 §3.2.5 — length codes 257..=285 and distance codes 0..=29.
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
        assert_eq!((u16::from(stream[0]) << 8 | u16::from(stream[1])) % 31, 0, "zlib FCHECK");
        let body = &stream[2..stream.len() - 4];
        let mut reader = Bits { data: body, position: 0 };
        let final_block = reader.take(1);
        assert_eq!(final_block, 1, "single final block expected");
        let kind = reader.take(2);
        assert_eq!(kind, 1, "fixed-Huffman block expected");
        let mut out = Vec::new();
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
        assert!((0b1_1001_0000..=0b1_1111_1111).contains(&code), "bad code {code:b}");
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
