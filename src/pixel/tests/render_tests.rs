use crate::pixel::{Capabilities, Graphics, Protocol, Source};
use crate::plot::Frame;
use crate::{Line, Plot, Points};

fn graphics() -> Graphics {
    Graphics::new(Protocol::Sixel).cell_size(4, 8)
}

fn sample() -> Plot<'static> {
    let x: Vec<f64> = (0..32).map(f64::from).collect();
    let y: Vec<f64> = x.iter().map(|v| (v * 0.4).sin()).collect();
    Plot::new()
        .layer(Line::xy(x.clone(), y.clone()).label("wave"))
        .layer(Points::xy(x, y))
        .title("hybrid")
}

#[test]
fn hybrid_output_weaves_text_chrome_around_a_sixel_panel() {
    let out = sample().render_pixels(&Frame::plain(40, 12), &graphics());
    // The chrome is ordinary text…
    assert!(out.contains("hybrid"), "title missing");
    // …the panel is a sixel payload bracketed by cursor save/restore…
    assert!(out.contains("\x1b7"), "missing DECSC");
    assert!(out.contains("\x1bP0;1;0q"), "missing sixel introducer");
    assert!(out.ends_with("\x1b8"), "missing DECRC at the end");
    // …reached by relative movement, never absolute addressing.
    assert!(
        out.contains("[9A") || out.contains("[10A"),
        "missing cursor-up: {:?}",
        &out[out.len().min(200)..]
    );
    assert!(
        !out.contains("\x1b[H"),
        "absolute addressing is scroll-unsafe"
    );
}

#[test]
fn marks_ink_the_panel_image_not_the_text_grid() {
    let out = sample().render_pixels(&Frame::plain(40, 12), &graphics());
    let text = &out[..out.find('\x1b').expect("a sixel payload follows the text")];
    // Braille (or any subpixel ink) would mean marks leaked onto the cell grid.
    assert!(
        !text.chars().any(|c| ('\u{2800}'..='\u{28FF}').contains(&c)),
        "marks drew on the text grid"
    );
}

#[test]
fn pixel_rendering_is_deterministic() {
    let (a, b) = (
        sample().render_pixels(&Frame::plain(40, 12), &graphics()),
        sample().render_pixels(&Frame::plain(40, 12), &graphics()),
    );
    assert_eq!(a, b);
}

#[test]
fn an_explicit_capability_context_selects_pixels_or_cells() {
    let frame = Frame::plain(40, 12);
    let pixels = Capabilities {
        protocols: vec![Protocol::Sixel],
        cell_size: Some((4, 8)),
        source: Source::Sniffed,
    };
    assert_eq!(
        sample().render_with_capabilities(&frame, &pixels),
        sample().render_pixels(&frame, &graphics())
    );

    let cells = Capabilities {
        protocols: Vec::new(),
        cell_size: Some((4, 8)),
        source: Source::Sniffed,
    };
    assert_eq!(
        sample().render_with_capabilities(&frame, &cells),
        sample().render(&frame)
    );
}

#[test]
fn a_zero_cell_size_degrades_to_text_only() {
    let gfx = Graphics::new(Protocol::Sixel).cell_size(0, 8);
    let out = sample().render_pixels(&Frame::plain(40, 12), &gfx);
    assert!(out.contains("hybrid"));
    assert!(
        !out.contains("\x1bP"),
        "no image payload without a cell size"
    );
}

#[test]
fn an_empty_frame_renders_to_nothing_and_does_not_panic() {
    let out = sample().render_pixels(&Frame::plain(0, 0), &graphics());
    assert!(!out.contains("\x1bP"));
}

#[test]
fn fallible_pixel_render_rejects_extreme_rasters_and_anchors() {
    let frame = Frame::plain(u16::MAX as usize, 1);
    let dense = Graphics::new(Protocol::Kitty).cell_size(u16::MAX, u16::MAX);
    assert!(matches!(
        sample().try_render_pixels(&frame, &dense),
        Err(crate::Error::DimensionTooLarge { .. })
    ));

    assert!(matches!(
        sample().try_render_pixels_at(&Frame::plain(40, 12), &graphics(), usize::MAX),
        Err(crate::Error::DimensionTooLarge { .. })
    ));
}

#[test]
fn the_corners_style_falls_back_to_a_pixel_line() {
    let x: Vec<f64> = (0..16).map(f64::from).collect();
    let y: Vec<f64> = x.iter().map(|v| v * 0.5).collect();
    let plot = Plot::new().layer(Line::xy(x, y).style(crate::mark::LineStyle::Corners));
    let out = plot.render_pixels(&Frame::plain(40, 12), &graphics());
    let text = &out[..out.find('\x1b').expect("a sixel payload follows the text")];
    for corner in ['\u{256D}', '\u{256E}', '\u{256F}', '\u{2570}'] {
        assert!(
            !text.contains(corner),
            "corner glyph {corner} leaked into hybrid output"
        );
    }
    assert!(out.contains("\x1bP0;1;0q"));
}

#[test]
fn a_zero_column_anchor_is_exactly_render_pixels() {
    let frame = Frame::plain(40, 12);
    assert_eq!(
        sample().render_pixels_at(&frame, &graphics(), 0),
        sample().render_pixels(&frame, &graphics())
    );
}

#[test]
fn an_anchored_block_jumps_every_row_and_the_walk_to_its_column() {
    let out = sample().render_pixels_at(&Frame::plain(40, 12), &graphics(), 42);
    let text_end = out.find("\x1b7").expect("the weave follows the text");
    for row in out[..text_end].split('\n') {
        assert!(row.starts_with("\x1b[43G"), "unanchored row: {row:?}");
    }
    // The cursor walk lands on the block's column before walking the gutter.
    assert!(out.contains("\x1b7\x1b[43G"), "walk not anchored");
    // Rows stay relative: never a full cursor-position (CUP) escape.
    assert!(!out.contains("\x1b[H") && !out.contains(";1H"));
}

#[test]
fn text_rows_cover_the_full_frame_width() {
    // The block owns its rectangle: every text row spans the frame's width,
    // so reprinting a block in place fully replaces the previous one — a
    // shorter title erases the longer title it lands on.
    let frame = Frame::plain(40, 12);
    let out = sample().render_pixels(&frame, &graphics());
    let grid = out.split('\x1b').next().unwrap_or_default();
    let mut rows = 0;
    for (index, row) in out
        .split("\x1b7")
        .next()
        .unwrap_or_default()
        .split('\n')
        .enumerate()
    {
        // Each row leads with its column anchor; past that, plain mode
        // carries no escapes except the panel walk after DECSC, which the
        // split above already removed from the last row.
        let row = row.strip_prefix("\x1b[1G").unwrap_or(row);
        let row = row.split('\x1b').next().unwrap_or(row);
        assert_eq!(
            row.chars().count(),
            40,
            "row {index} is not full width: {row:?}"
        );
        rows += 1;
    }
    assert_eq!(rows, 12, "every frame row present");
    let _ = grid;
}

#[test]
fn full_width_rows_reset_trailing_color_state() {
    // In a colored mode the padded rows still end reset: trailing spaces
    // must not leak a foreground or background into whatever follows.
    let mut frame = Frame::plain(40, 12);
    frame.color = crate::render::ColorMode::TrueColor;
    let out = sample().render_pixels(&frame, &graphics());
    let text_grid = out.split("\x1b7").next().unwrap_or_default();
    for row in text_grid.split('\n').filter(|row| row.contains("\x1b[")) {
        assert!(
            row.ends_with("\x1b[0m") || !row.contains("38;"),
            "colored row does not end reset: {row:?}"
        );
    }
}

#[test]
fn a_flush_left_block_still_anchors_every_row() {
    // Raw-mode hosts: LF alone does not return the carriage, so even a
    // column-0 block must re-anchor each row or its chrome staircases.
    let out = sample().render_pixels(
        &Frame::plain(24, 8),
        &Graphics::new(Protocol::Kitty).cell_size(4, 8),
    );
    for row in out.split('\n').take_while(|row| !row.contains("\x1b_G")) {
        assert!(row.starts_with("\x1b[1G"), "unanchored row: {row:?}");
    }
}

#[test]
fn smooth_cells_read_as_a_continuous_field() {
    use crate::Cells;
    let values: &[f64] = &[0.0, 1.0, 1.0, 0.0];
    let distinct = |smooth: bool| {
        let cells = Cells::matrix(2, values);
        let cells = if smooth { cells.smooth() } else { cells };
        let plot = Plot::new().layer(cells);
        let (_, canvas, rect, _) = plot
            .try_rasterize_hybrid(&Frame::plain(24, 12), (8, 16), None)
            .unwrap();
        let (cw, ch) = canvas.cell();
        let y = (rect.top + rect.rows / 2) * ch;
        let mut colors = std::collections::BTreeSet::new();
        for x in rect.gutter * cw..(rect.gutter + rect.columns) * cw {
            if let Some(crate::Color::Rgb(r, g, b)) = canvas.get(x, y) {
                colors.insert((r, g, b));
            }
        }
        colors.len()
    };
    let blocky = distinct(false);
    let smooth = distinct(true);
    assert!(blocky <= 4, "nearest sampling is blocky: {blocky}");
    assert!(smooth > blocky * 3, "interpolation grades: {smooth}");
}
