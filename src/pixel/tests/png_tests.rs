use super::{crc32, encode};
use crate::pixel::deflate::inflate;
use crate::pixel::render::Image;

fn image(width: usize, height: usize, pixels: Vec<Option<(u8, u8, u8)>>) -> Image {
    Image {
        width,
        height,
        pixels,
    }
}

#[test]
fn crc32_matches_the_classic_check_value() {
    assert_eq!(crc32(b"123456789".iter().copied()), 0xCBF4_3926);
    assert_eq!(crc32(b"".iter().copied()), 0);
}

#[test]
fn the_png_container_is_structurally_sound() {
    let png = encode(&image(
        2,
        2,
        vec![Some((255, 0, 0)), None, None, Some((0, 0, 255))],
    ));
    // Signature.
    assert_eq!(&png[..8], &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]);
    // IHDR: length 13, width 2, height 2, depth 8, color type 6 (RGBA).
    assert_eq!(&png[8..12], &13u32.to_be_bytes());
    assert_eq!(&png[12..16], b"IHDR");
    assert_eq!(&png[16..20], &2u32.to_be_bytes());
    assert_eq!(&png[20..24], &2u32.to_be_bytes());
    assert_eq!(&png[24..26], &[8, 6]);
    // The file ends with IEND and its fixed CRC.
    assert_eq!(
        &png[png.len() - 12..],
        &[0, 0, 0, 0, b'I', b'E', b'N', b'D', 0xAE, 0x42, 0x60, 0x82]
    );
}

#[test]
fn the_idat_stream_inflates_to_the_scanlines() {
    let png = encode(&image(2, 1, vec![Some((10, 20, 30)), None]));
    // Find IDAT and inflate it with the deflate module's test oracle
    // (which also verifies the adler-32 trailer).
    let idat = png
        .windows(4)
        .position(|w| w == b"IDAT")
        .expect("an IDAT chunk exists");
    let length = u32::from_be_bytes(png[idat - 4..idat].try_into().unwrap()) as usize;
    let data = &png[idat + 4..idat + 4 + length];
    // One scanline: filter 0, RGBA(10, 20, 30, 255), transparent RGBA(0,0,0,0).
    assert_eq!(inflate(data), &[0, 10, 20, 30, 255, 0, 0, 0, 0]);
}

#[test]
fn large_flat_images_compress_and_roundtrip() {
    // 200×90 solid RGBA = 72,090 scanline bytes; a flat panel must both
    // survive the roundtrip and actually shrink in transport.
    let png = encode(&image(200, 90, vec![Some((1, 2, 3)); 18000]));
    let idat = png
        .windows(4)
        .position(|w| w == b"IDAT")
        .expect("an IDAT chunk exists");
    let length = u32::from_be_bytes(png[idat - 4..idat].try_into().unwrap()) as usize;
    let data = &png[idat + 4..idat + 4 + length];
    let raw = inflate(data);
    assert_eq!(raw.len(), 90 * (1 + 200 * 4));
    assert!(
        length < raw.len() / 50,
        "flat raster must compress: {length} bytes for {} raw",
        raw.len()
    );
}

#[test]
fn every_tiny_raster_encodes_deterministically() {
    for width in 0..=4 {
        for height in 0..=4 {
            let pixels = (0..width * height)
                .map(|index| match index % 3 {
                    0 => Some((index as u8, 20, 30)),
                    1 => None,
                    _ => Some((40, index as u8, 60)),
                })
                .collect();
            let image = image(width, height, pixels);
            let first = encode(&image);
            assert_eq!(first, encode(&image));
            assert_eq!(
                &first[..8],
                &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]
            );
        }
    }
}
