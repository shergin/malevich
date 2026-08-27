use super::encode;
use crate::pixel::deflate::inflate;
use crate::pixel::render::Image;

/// Test-side base64 decode (the crate only needs the encode direction).
fn base64_decode(text: &str) -> Vec<u8> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut bits = 0u32;
    let mut have = 0u32;
    let mut out = Vec::new();
    for byte in text.bytes().filter(|&b| b != b'=') {
        let value = ALPHABET.iter().position(|&a| a == byte).expect("base64") as u32;
        bits = bits << 6 | value;
        have += 6;
        if have >= 8 {
            have -= 8;
            out.push((bits >> have) as u8);
        }
    }
    out
}

/// The concatenated payload of every chunk, decoded and inflated.
fn pixels_of(out: &str) -> Vec<u8> {
    let payload: String = out
        .split("\x1b_G")
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| {
            let body = chunk.split_once(';').expect("options;payload").1;
            body.trim_end_matches("\x1b\\")
        })
        .collect();
    inflate(&base64_decode(&payload))
}

fn image(width: usize, height: usize, pixels: Vec<Option<(u8, u8, u8)>>) -> Image {
    Image {
        width,
        height,
        pixels,
    }
}

#[test]
fn a_single_pixel_encodes_to_one_complete_apc_escape() {
    let out = encode(&image(1, 1, vec![Some((255, 0, 0))]));
    assert!(
        out.starts_with("\x1b_Ga=T,f=32,o=z,s=1,v=1,C=1,q=2,m=0;"),
        "{out:?}"
    );
    assert!(out.ends_with("\x1b\\"));
    assert_eq!(pixels_of(&out), [255, 0, 0, 255]);
}

#[test]
fn transparent_pixels_carry_zero_alpha() {
    let out = encode(&image(1, 1, vec![None]));
    assert_eq!(pixels_of(&out), [0, 0, 0, 0]);
}

#[test]
fn large_images_chunk_at_4096_bytes_of_payload() {
    // Deterministic noise resists compression, forcing several chunks.
    let mut state = 1u64;
    let pixels: Vec<Option<(u8, u8, u8)>> = (0..8192)
        .map(|_| {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let bytes = state.to_le_bytes();
            Some((bytes[5], bytes[6], bytes[7]))
        })
        .collect();
    let out = encode(&image(128, 64, pixels.clone()));
    let escapes = out.matches("\x1b_G").count();
    assert!(escapes >= 3, "expected several chunks, got {escapes}");
    // Control keys only on the first chunk; continuation flags on all.
    assert_eq!(out.matches("a=T").count(), 1);
    assert_eq!(out.matches("m=1;").count(), escapes - 1);
    assert_eq!(out.matches("m=0;").count(), 1);
    assert_eq!(out.matches("\x1b\\").count(), escapes);
    // And the reassembled payload is the exact raster.
    let rgba = pixels_of(&out);
    assert_eq!(rgba.len(), 8192 * 4);
    let (r, g, b) = pixels[100].unwrap();
    assert_eq!(&rgba[400..404], &[r, g, b, 255]);
}

#[test]
fn encoding_is_deterministic() {
    let pixels = vec![Some((9, 8, 7)), None, Some((1, 2, 3)), None];
    assert_eq!(
        encode(&image(2, 2, pixels.clone())),
        encode(&image(2, 2, pixels))
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
            if width == 0 || height == 0 {
                assert!(first.is_empty());
            } else {
                assert!(first.starts_with("\x1b_G"));
                assert!(first.ends_with("\x1b\\"));
            }
        }
    }
}

#[test]
fn a_placement_rectangle_pins_the_image_to_its_cells() {
    let out = super::encode_rgba(2, 1, (5, 3), &[1, 2, 3, 255, 4, 5, 6, 255]);
    assert!(
        out.starts_with("\x1b_Ga=T,f=32,o=z,s=2,v=1,c=5,r=3,C=1,q=2,m=0;"),
        "{out:?}"
    );
    // Without a placement the keys stay absent (the test-wrapper path).
    let bare = encode(&image(2, 1, vec![Some((1, 2, 3)), Some((4, 5, 6))]));
    assert!(!bare.contains("c="), "{bare:?}");
}
