use super::super::{Canvas, Charset, Color, ColorMode, PlotRect, PointShape};
use super::Surface;

const ONE_CELL: PlotRect = PlotRect {
    gutter: 0,
    top: 0,
    columns: 1,
    rows: 1,
};

fn patch(surface: &mut Surface, row: usize, sample: Option<(f64, Color)>) {
    Canvas::patch(surface, 0, row, ONE_CELL, sample);
}

const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<Surface>();
const _: () = assert_send_sync::<Color>();
const _: () = assert_send_sync::<Charset>();

#[test]
fn a_raster_matches_the_surface_string() {
    let mut surface = Surface::new(4, 2, Charset::Braille);
    surface.dot(0.0, 0.0, Color::Cyan);
    surface.dot(3.0, 3.0, Color::Yellow);
    surface.text(2, 0, "ab", Color::Green);
    let raster = surface.to_raster();
    assert_eq!(raster.width(), 4);
    assert_eq!(raster.height(), 2);
    assert_eq!(raster.encode(ColorMode::Plain), surface.to_plain());
    assert_eq!(
        raster.encode(ColorMode::TrueColor),
        surface.encode(ColorMode::TrueColor)
    );
}

#[test]
fn a_dot_lights_one_braille_subpixel() {
    let mut surface = Surface::new(2, 1, Charset::Braille);
    surface.dot(0.0, 0.0, Color::Default);
    assert_eq!(surface.to_plain(), "\u{2801}");
}

#[test]
fn point_shapes_stay_distinct_in_colorless_cell_output() {
    let mut surface = Surface::new(3, 1, Charset::Braille);
    Canvas::point(&mut surface, 0.0, 0.0, PointShape::Dot, Color::Default);
    Canvas::point(&mut surface, 2.0, 0.0, PointShape::Plus, Color::Default);
    Canvas::point(&mut surface, 4.0, 0.0, PointShape::Cross, Color::Default);
    assert_eq!(surface.to_plain(), "\u{2801}+x");
}

#[test]
fn a_bottom_row_line_renders_as_lower_dots() {
    let mut surface = Surface::new(2, 1, Charset::Braille);
    surface.line((0.0, 3.0), (3.0, 3.0), Color::Default);
    assert_eq!(surface.to_plain(), "\u{28C0}\u{28C0}");
}

#[test]
fn a_left_column_line_renders_as_a_wall() {
    let mut surface = Surface::new(1, 1, Charset::Braille);
    surface.line((0.0, 0.0), (0.0, 3.0), Color::Default);
    assert_eq!(surface.to_plain(), "\u{2847}");
}

#[test]
fn a_diagonal_descends_across_cells() {
    let mut surface = Surface::new(2, 1, Charset::Braille);
    surface.line((0.0, 3.0), (3.0, 0.0), Color::Default);
    assert_eq!(surface.to_plain(), "\u{2860}\u{280A}");
}

#[test]
fn segments_clip_to_the_surface() {
    let mut surface = Surface::new(2, 1, Charset::Braille);
    surface.line((-100.0, 2.0), (100.0, 2.0), Color::Default);
    assert_eq!(surface.to_plain(), "\u{2824}\u{2824}");
}

#[test]
fn fully_outside_segments_draw_nothing() {
    let mut surface = Surface::new(2, 1, Charset::Braille);
    surface.line((-5.0, -5.0), (-1.0, -1.0), Color::Default);
    surface.set(99, 0, Color::Default);
    surface.set(-1, 0, Color::Default);
    assert_eq!(surface.to_plain(), "");
}

#[test]
fn non_finite_coordinates_draw_nothing() {
    let mut surface = Surface::new(2, 1, Charset::Braille);
    surface.line((f64::NAN, 0.0), (3.0, 3.0), Color::Default);
    surface.dot(f64::INFINITY, 0.0, Color::Default);
    assert_eq!(surface.to_plain(), "");
}

#[test]
fn text_wins_over_pixels_in_shared_cells() {
    let mut surface = Surface::new(3, 1, Charset::Braille);
    surface.line((0.0, 0.0), (5.0, 0.0), Color::Default);
    surface.text(0, 0, "ab", Color::Default);
    assert_eq!(surface.to_plain(), "ab\u{2809}");
}

#[test]
fn text_clips_at_the_edges() {
    let mut surface = Surface::new(3, 1, Charset::Ascii);
    surface.text(-1, 0, "abcde", Color::Default);
    assert_eq!(surface.to_plain(), "bcd");
    let mut surface = Surface::new(3, 1, Charset::Ascii);
    surface.text(0, -1, "abc", Color::Default);
    surface.text(0, 1, "abc", Color::Default);
    assert_eq!(surface.to_plain(), "");
}

#[test]
fn wide_glyphs_occupy_two_cells_and_keep_alignment() {
    let mut surface = Surface::new(5, 1, Charset::Ascii);
    surface.text(0, 0, "\u{65E5}\u{672C}", Color::Default);
    surface.text(4, 0, "x", Color::Default);
    assert_eq!(surface.to_plain(), "\u{65E5}\u{672C}x");
}

#[test]
fn a_wide_glyph_straddling_the_edge_is_dropped_whole() {
    let mut surface = Surface::new(3, 1, Charset::Ascii);
    surface.text(0, 0, "\u{65E5}\u{672C}", Color::Default);
    assert_eq!(surface.to_plain(), "\u{65E5}");
}

#[test]
fn overwriting_half_a_wide_glyph_blanks_the_other_half() {
    let mut surface = Surface::new(4, 1, Charset::Ascii);
    surface.text(0, 0, "\u{65E5}", Color::Default);
    surface.text(1, 0, "a", Color::Default);
    assert_eq!(surface.to_plain(), " a");

    let mut surface = Surface::new(4, 1, Charset::Ascii);
    surface.text(0, 0, "\u{65E5}", Color::Default);
    surface.text(0, 0, "b", Color::Default);
    assert_eq!(surface.to_plain(), "b");
}

#[test]
fn combining_marks_are_dropped_at_the_cell_grid() {
    let mut surface = Surface::new(4, 1, Charset::Ascii);
    surface.text(0, 0, "e\u{0301}x", Color::Default);
    assert_eq!(surface.to_plain(), "ex");
}

#[test]
fn plain_output_trims_trailing_spaces_and_keeps_leading_ones() {
    let mut surface = Surface::new(4, 2, Charset::Ascii);
    surface.set(0, 0, Color::Default);
    surface.set(1, 1, Color::Default);
    assert_eq!(surface.to_plain(), "*\n *");
}

#[test]
fn empty_surfaces_encode_to_nothing() {
    assert_eq!(Surface::new(0, 0, Charset::Braille).to_plain(), "");
    assert_eq!(
        Surface::new(0, 0, Charset::Braille).encode(ColorMode::Ansi16),
        ""
    );
    assert_eq!(
        Surface::new(3, 0, Charset::Ascii).encode(ColorMode::Ansi16),
        ""
    );
}

#[test]
fn oversized_surfaces_are_fallible_and_the_convenience_constructor_stays_safe() {
    let error =
        Surface::try_new(usize::MAX, 0, Charset::Braille).expect_err("oversized surface must fail");
    assert!(matches!(error, crate::Error::DimensionTooLarge { .. }));

    let surface = Surface::new(usize::MAX, 0, Charset::Braille);
    assert_eq!(surface.size(), (0, 0));
    assert_eq!(surface.to_plain(), "");
}

#[test]
fn uncolored_surfaces_encode_identically_in_both_encoders() {
    let mut surface = Surface::new(4, 2, Charset::Braille);
    surface.line((0.0, 0.0), (7.0, 7.0), Color::Default);
    assert_eq!(surface.encode(ColorMode::Ansi16), surface.to_plain());
}

#[test]
fn ansi_runs_emit_one_code_per_color_change() {
    let mut surface = Surface::new(3, 1, Charset::Ascii);
    surface.set(0, 0, Color::Red);
    surface.set(1, 0, Color::Red);
    surface.set(2, 0, Color::Blue);
    assert_eq!(
        surface.encode(ColorMode::Ansi16),
        "\x1b[31m**\x1b[34m*\x1b[0m"
    );
}

#[test]
fn the_last_write_owns_a_shared_cell_color() {
    let mut surface = Surface::new(2, 1, Charset::Ascii);
    surface.set(0, 0, Color::Red);
    surface.set(1, 0, Color::Red);
    surface.set(1, 0, Color::Blue);
    assert_eq!(
        surface.encode(ColorMode::Ansi16),
        "\x1b[31m*\x1b[34m*\x1b[0m"
    );
}

#[test]
fn colored_rows_end_with_a_reset_even_after_trimming() {
    let mut surface = Surface::new(4, 1, Charset::Ascii);
    surface.set(0, 0, Color::Green);
    let encoded = surface.encode(ColorMode::Ansi16);
    assert_eq!(encoded, "\x1b[32m*\x1b[0m");
}

#[test]
fn two_vertical_samples_snapshot_every_string_color_tier() {
    let mut surface = Surface::new(1, 1, Charset::Quadrants);
    patch(&mut surface, 0, Some((0.0, Color::Rgb(255, 0, 0))));
    patch(&mut surface, 1, Some((1.0, Color::Rgb(0, 0, 255))));

    let snapshots = [
        (ColorMode::Plain, "\u{2593}"),
        (ColorMode::Ansi16, "\x1b[91;44m\u{2580}\x1b[0m"),
        (ColorMode::Ansi256, "\x1b[38;5;196;48;5;21m\u{2580}\x1b[0m"),
        (
            ColorMode::TrueColor,
            "\x1b[38;2;255;0;0;48;2;0;0;255m\u{2580}\x1b[0m",
        ),
    ];
    for (mode, expected) in snapshots {
        assert_eq!(surface.encode(mode), expected, "{mode:?}");
    }
}

#[test]
fn same_color_and_missing_half_cell_patches_have_stable_fallbacks() {
    let mut solid = Surface::new(1, 1, Charset::Quadrants);
    patch(&mut solid, 0, Some((0.0, Color::Red)));
    patch(&mut solid, 1, Some((1.0, Color::Red)));
    assert_eq!(solid.to_plain(), "\u{2593}");
    assert_eq!(solid.encode(ColorMode::Ansi16), "\x1b[31m\u{2588}\x1b[0m");

    let mut top = Surface::new(1, 1, Charset::Quadrants);
    patch(&mut top, 0, Some((0.25, Color::Red)));
    patch(&mut top, 1, None);
    assert_eq!(top.to_plain(), "\u{2592}");
    assert_eq!(top.encode(ColorMode::Ansi16), "\x1b[31m\u{2580}\x1b[0m");

    let mut bottom = Surface::new(1, 1, Charset::Quadrants);
    patch(&mut bottom, 0, None);
    patch(&mut bottom, 1, Some((0.75, Color::Blue)));
    assert_eq!(bottom.to_plain(), "\u{2588}");
    assert_eq!(bottom.encode(ColorMode::Ansi16), "\x1b[34m\u{2584}\x1b[0m");
}

#[test]
fn foreground_and_background_reset_together_at_run_boundaries() {
    let rect = PlotRect {
        columns: 2,
        ..ONE_CELL
    };
    let mut surface = Surface::new(2, 1, Charset::Quadrants);
    Canvas::patch(&mut surface, 0, 0, rect, Some((0.0, Color::Red)));
    Canvas::patch(&mut surface, 0, 1, rect, Some((1.0, Color::Blue)));
    surface.text(1, 0, "x", Color::Green);
    assert_eq!(
        surface.encode(ColorMode::Ansi16),
        "\x1b[31;44m\u{2580}\x1b[32;49mx\x1b[0m"
    );

    surface.text(0, 0, "y", Color::Yellow);
    assert_eq!(
        surface.encode(ColorMode::Ansi16),
        "\x1b[33my\x1b[32mx\x1b[0m"
    );
}

#[test]
fn patch_pair_overwrites_are_atomic_and_identical_styles_share_one_sgr_run() {
    let rect = PlotRect {
        columns: 2,
        ..ONE_CELL
    };
    let mut surface = Surface::new(2, 1, Charset::Quadrants);
    for column in 0..2 {
        Canvas::patch(&mut surface, column, 0, rect, Some((0.0, Color::Red)));
        Canvas::patch(&mut surface, column, 1, rect, Some((1.0, Color::Blue)));
    }
    assert_eq!(
        surface.encode(ColorMode::Ansi16),
        "\x1b[31;44m\u{2580}\u{2580}\x1b[0m"
    );

    Canvas::patch(&mut surface, 0, 0, rect, None);
    Canvas::patch(&mut surface, 0, 1, rect, None);
    assert_eq!(
        surface.encode(ColorMode::Ansi16),
        "\x1b[31;44m\u{2580}\u{2580}\x1b[0m"
    );

    Canvas::patch(&mut surface, 0, 0, rect, Some((0.25, Color::Green)));
    Canvas::patch(&mut surface, 0, 1, rect, Some((0.75, Color::Yellow)));
    assert_eq!(
        surface.encode(ColorMode::Ansi16),
        "\x1b[32;43m\u{2580}\x1b[31;44m\u{2580}\x1b[0m"
    );
}

#[cfg(feature = "evcxr")]
#[test]
fn html_escapes_every_glyph_that_can_open_markup() {
    let mut surface = Surface::new(3, 1, Charset::Ascii);
    surface.text(0, 0, "<&>", Color::Default);
    assert_eq!(surface.encode_html(), "&lt;&amp;&gt;");
}

#[cfg(feature = "evcxr")]
#[test]
fn html_collapses_concrete_rgb_runs() {
    let mut surface = Surface::new(4, 1, Charset::Ascii);
    surface.set(0, 0, Color::Red);
    surface.set(1, 0, Color::Rgb(205, 0, 0));
    surface.set(2, 0, Color::Blue);
    assert_eq!(
        surface.encode_html(),
        "<span style=\"color:#cd0000\">**</span><span style=\"color:#0000ee\">*</span>"
    );
}

#[cfg(feature = "evcxr")]
#[test]
fn html_preserves_both_colors_of_a_half_block() {
    let mut surface = Surface::new(1, 1, Charset::Quadrants);
    patch(&mut surface, 0, Some((0.0, Color::Red)));
    patch(&mut surface, 1, Some((1.0, Color::Blue)));
    assert_eq!(
        surface.encode_html(),
        "<span style=\"color:#cd0000;background-color:#0000ee\">\u{2580}</span>"
    );
}

#[cfg(feature = "evcxr")]
#[test]
fn default_html_color_inherits_without_a_span() {
    let mut surface = Surface::new(3, 1, Charset::Ascii);
    surface.set(0, 0, Color::Red);
    surface.set(1, 0, Color::Default);
    surface.set(2, 0, Color::Red);
    let html = surface.encode_html();
    assert_eq!(
        html,
        "<span style=\"color:#cd0000\">*</span>*<span style=\"color:#cd0000\">*</span>"
    );
    assert!(!html.contains("#808080"));
}

#[cfg(feature = "evcxr")]
#[test]
fn html_spaces_extend_runs_and_trailing_spaces_are_trimmed() {
    let mut surface = Surface::new(6, 2, Charset::Ascii);
    surface.text(0, 0, "x", Color::Green);
    surface.text(2, 0, "x", Color::Green);
    surface.text(1, 1, "y", Color::Blue);
    assert_eq!(
        surface.encode_html(),
        "<span style=\"color:#00cd00\">x x</span>\n <span style=\"color:#0000ee\">y</span>"
    );
}
