#[cfg(feature = "pixel")]
use super::write_pixel_panel;
use super::{Ink, ink, num};

#[test]
fn block_elements_map_to_their_defined_geometry() {
    // Full block: one sub-cell, set.
    assert_eq!(
        ink('\u{2588}'),
        Some(Ink {
            columns: 2,
            rows: 2,
            bits: 0b1111
        })
    );
    // Lower half: the bottom row of the quadrant grid.
    assert_eq!(
        ink('\u{2584}'),
        Some(Ink {
            columns: 2,
            rows: 2,
            bits: 0b1100
        })
    );
    // Lower one eighth: the last of eight rows.
    assert_eq!(
        ink('\u{2581}'),
        Some(Ink {
            columns: 1,
            rows: 8,
            bits: 0b1000_0000
        })
    );
    // Lower seven eighths: everything but the first row.
    assert_eq!(
        ink('\u{2587}'),
        Some(Ink {
            columns: 1,
            rows: 8,
            bits: 0b1111_1110
        })
    );
    // Left one eighth: the first of eight columns; left seven eighths: all but the last.
    assert_eq!(
        ink('\u{258F}'),
        Some(Ink {
            columns: 8,
            rows: 1,
            bits: 0b0000_0001
        })
    );
    assert_eq!(
        ink('\u{2589}'),
        Some(Ink {
            columns: 8,
            rows: 1,
            bits: 0b0111_1111
        })
    );
    // Upper eighth and right eighth.
    assert_eq!(
        ink('\u{2594}'),
        Some(Ink {
            columns: 1,
            rows: 8,
            bits: 1
        })
    );
    assert_eq!(
        ink('\u{2595}'),
        Some(Ink {
            columns: 8,
            rows: 1,
            bits: 0b1000_0000
        })
    );
}

/// The ink on a fine common grid (8 columns × 24 rows), so geometries on
/// different sub-cell grids compare as pictures.
fn coverage(ink: Ink) -> Vec<bool> {
    let mut mask = vec![false; 8 * 24];
    for row in 0..24u8 {
        for column in 0..8u8 {
            let grid_column = column * ink.columns / 8;
            let grid_row = row * ink.rows / 24;
            mask[usize::from(row) * 8 + usize::from(column)] = ink.bit(grid_column, grid_row);
        }
    }
    mask
}

#[test]
fn sextants_and_octants_invert_their_codecs() {
    use crate::render::Charset;
    // Legacy glyphs (▌, ▐, █, the quadrants) come back on the quadrant grid;
    // the picture must still be the codec's pattern.
    for (charset, rows, count) in [
        (Charset::Sextants, 3u8, 64u16),
        (Charset::Octants, 4u8, 256u16),
    ] {
        for bits in 1..count {
            let bits = bits as u8;
            let glyph = charset.glyph(bits);
            let found = ink(glyph).unwrap_or_else(|| panic!("{glyph:?} ({bits:#b}) must be ink"));
            let expected = Ink {
                columns: 2,
                rows,
                bits,
            };
            assert_eq!(
                coverage(found),
                coverage(expected),
                "{charset:?} pattern {bits:#b} ({glyph})"
            );
        }
    }
}

#[test]
fn text_glyphs_are_not_ink() {
    for glyph in [
        ' ', 'a', '\u{2502}', '\u{2524}', '\u{2800}', '\u{28FF}', '\u{2026}', '*', '#',
    ] {
        assert_eq!(ink(glyph), None, "{glyph:?} must stay text");
    }
}

#[test]
fn numbers_print_without_trailing_zeros() {
    assert_eq!(num(16.0), "16");
    assert_eq!(num(7.8), "7.8");
    assert_eq!(num(23.4), "23.4");
    assert_eq!(num(0.975), "0.975");
    assert_eq!(num(5.0 / 3.0), "1.667");
}

#[cfg(feature = "pixel")]
#[test]
fn a_pixel_panel_merges_runs_down_rows_and_keeps_faint_coverage() {
    // Alpha 0 is skipped. A run repeated in the next row grows one rectangle
    // down; a run that changes closes. Alpha 10 still composites onto the
    // card background instead of disappearing.
    const RED: [u8; 4] = [255, 0, 0, 255];
    const BLUE: [u8; 4] = [0, 0, 255, 255];
    const NONE: [u8; 4] = [255, 0, 0, 0];
    const FAINT: [u8; 4] = [255, 0, 0, 10];
    let rgba = [RED, RED, NONE, FAINT, RED, RED, BLUE, FAINT].concat();
    let mut out = String::new();
    write_pixel_panel(
        &mut out,
        (10.0, 20.0, 40.0, 20.0),
        4,
        2,
        &rgba,
        (13, 17, 23),
    );
    assert_eq!(
        out,
        "<path fill=\"#ff0000\" d=\"M10 20H30V40H10Z\"/>\n\
         <path fill=\"#0000ff\" d=\"M30 30H40V40H30Z\"/>\n\
         <path fill=\"#161016\" d=\"M40 20H50V40H40Z\"/>\n"
    );
}
