use crate::render::{Canvas, Color, PlotRect, PointShape};

use super::PixelCanvas;

const RED: Color = Color::Rgb(255, 0, 0);

fn rect() -> PlotRect {
    PlotRect {
        gutter: 0,
        top: 0,
        columns: 4,
        rows: 4,
    }
}

#[test]
fn a_new_canvas_is_fully_transparent() {
    let canvas = PixelCanvas::new(4, 4, (8, 16));
    assert_eq!(canvas.size(), (32, 64));
    for y in 0..64 {
        for x in 0..32 {
            assert_eq!(canvas.get(x, y), None);
        }
    }
}

#[test]
fn oversized_canvas_geometry_is_rejected_before_allocation() {
    let error = PixelCanvas::try_new(usize::MAX, 2, (2, 2), None)
        .err()
        .expect("oversized canvas must fail");
    assert!(matches!(error, crate::Error::DimensionTooLarge { .. }));
}

#[test]
fn dot_stamps_a_point_marker_and_clips_outside() {
    let mut canvas = PixelCanvas::new(2, 2, (8, 8));
    canvas.point(3.4, 5.6, PointShape::Dot, RED);
    // At the classic density the point pen is 2×2 — one step above the
    // 1-pixel stroke, so scatter dots read over lines.
    assert_eq!(canvas.get(3, 6), Some(RED));
    canvas.point(-2.0, 0.0, PointShape::Dot, RED);
    canvas.point(1000.0, 0.0, PointShape::Dot, RED);
    canvas.point(f64::NAN, 0.0, PointShape::Dot, RED);
    let drawn = (0..16)
        .flat_map(|y| (0..16).map(move |x| (x, y)))
        .filter(|&(x, y)| canvas.get(x, y).is_some())
        .count();
    assert_eq!(drawn, 4);
}

#[test]
fn plus_and_cross_points_keep_their_pixel_shapes() {
    let mut canvas = PixelCanvas::new(6, 3, (8, 8));
    canvas.point(10.0, 10.0, PointShape::Plus, RED);
    assert_eq!(canvas.get(12, 10), Some(RED));
    assert_eq!(canvas.get(10, 12), Some(RED));
    assert_eq!(canvas.get(12, 12), None);

    canvas.point(30.0, 10.0, PointShape::Cross, RED);
    assert_eq!(canvas.get(32, 12), Some(RED));
    assert_eq!(canvas.get(28, 8), Some(RED));
    assert_eq!(canvas.get(32, 10), None);
}

#[test]
fn strokes_scale_with_cell_density() {
    // A retina-dense cell (20×44 device pixels): stroke 3, not a hairline.
    let mut canvas = PixelCanvas::new(4, 2, (20, 44));
    canvas.line((10.0, 20.0), (70.0, 20.0), RED);
    for y in 19..=21 {
        assert_eq!(canvas.get(40, y), Some(RED), "row {y} should carry ink");
    }
    assert_eq!(canvas.get(40, 17), None);
    assert_eq!(canvas.get(40, 23), None);
    // The classic 8×16 cell keeps the exact 1-pixel stroke it always had.
    let mut classic = PixelCanvas::new(4, 2, (8, 16));
    classic.line((2.0, 8.0), (20.0, 8.0), RED);
    assert_eq!(classic.get(10, 8), Some(RED));
    assert_eq!(classic.get(10, 7), None);
    assert_eq!(classic.get(10, 9), None);
}

#[test]
fn points_read_above_lines_at_any_density() {
    let mut canvas = PixelCanvas::new(2, 1, (20, 44));
    canvas.point(20.0, 22.0, PointShape::Dot, RED);
    let drawn = (0..44)
        .flat_map(|y| (0..40).map(move |x| (x, y)))
        .filter(|&(x, y)| canvas.get(x, y).is_some())
        .count();
    // Stroke 3 at this density: the pen is an anti-aliased disc of
    // radius 2 — 9 solid pixels (strictly inside the radius), fringe at
    // and past the rim.
    assert_eq!(drawn, 9);
    assert!(canvas.rgba(22, 23)[3] > 0, "the fringe carries partial ink");
    assert!(canvas.rgba(22, 23)[3] < 128, "but never reads as solid");
}

#[test]
fn a_horizontal_line_fills_every_pixel_between_its_endpoints() {
    let mut canvas = PixelCanvas::new(4, 1, (8, 8));
    canvas.line((2.0, 3.0), (20.0, 3.0), RED);
    for x in 2..=20 {
        assert_eq!(canvas.get(x, 3), Some(RED), "x={x}");
    }
    assert_eq!(canvas.get(1, 3), None);
    assert_eq!(canvas.get(21, 3), None);
}

#[test]
fn lines_respect_the_clip_rectangle() {
    let mut canvas = PixelCanvas::new(4, 1, (8, 8));
    canvas.set_clip(8, 0, 16, 8);
    canvas.line((0.0, 4.0), (31.0, 4.0), RED);
    for x in 0..32 {
        let expected = (8..16).contains(&x);
        assert_eq!(canvas.get(x as usize, 4).is_some(), expected, "x={x}");
    }
    canvas.clear_clip();
    canvas.line((0.0, 5.0), (31.0, 5.0), RED);
    assert_eq!(canvas.get(0, 5), Some(RED));
    assert_eq!(canvas.get(31, 5), Some(RED));
}

#[test]
fn a_bar_fills_its_exact_rectangle_from_the_baseline() {
    let mut canvas = PixelCanvas::new(4, 4, (8, 8));
    // Plot-local: a bar over x ∈ [4, 12), from the value end at y=8 down to the
    // baseline at y=24.
    canvas.bar((4.0, 12.0), 8.0, 24.0, true, rect(), RED);
    for y in 8..24 {
        for x in 4..12 {
            assert_eq!(canvas.get(x, y), Some(RED), "({x}, {y})");
        }
    }
    assert_eq!(canvas.get(3, 8), None);
    assert_eq!(canvas.get(12, 8), None);
    assert_eq!(canvas.get(4, 7), None);
    assert_eq!(canvas.get(4, 24), None);
}

#[test]
fn a_zero_width_bar_still_draws_one_pixel_column() {
    let mut canvas = PixelCanvas::new(4, 4, (8, 8));
    canvas.bar((6.2, 6.4), 0.0, 4.0, true, rect(), RED);
    assert_eq!(canvas.get(6, 2), Some(RED));
}

#[test]
fn the_gutter_offsets_bars_into_the_plot_rectangle() {
    let mut canvas = PixelCanvas::new(4, 4, (8, 8));
    let offset = PlotRect {
        gutter: 2,
        top: 1,
        columns: 2,
        rows: 3,
    };
    canvas.bar((0.0, 4.0), 0.0, 8.0, true, offset, RED);
    // gutter 2 cells × 8 px, top 1 cell × 8 px.
    assert_eq!(canvas.get(16, 8), Some(RED));
    assert_eq!(canvas.get(15, 8), None);
    assert_eq!(canvas.get(16, 7), None);
}

#[test]
fn the_marker_clears_a_band_through_a_fill() {
    let mut canvas = PixelCanvas::new(4, 4, (8, 8));
    canvas.bar((0.0, 32.0), 0.0, 32.0, true, rect(), RED);
    canvas.marker(16.0, 8.0, 16.0, RED);
    assert_eq!(canvas.get(16, 16), None);
    assert_eq!(canvas.get(8, 16), None);
    assert_eq!(canvas.get(24, 16), None);
    // Above and below the band the fill survives.
    assert_eq!(canvas.get(16, 12), Some(RED));
    assert_eq!(canvas.get(16, 20), Some(RED));
    // Outside the reach the fill survives.
    assert_eq!(canvas.get(4, 16), Some(RED));
}

#[test]
fn patches_are_single_pixels_inside_the_plot_rectangle() {
    let mut canvas = PixelCanvas::new(4, 4, (8, 8));
    assert_eq!(canvas.patch_density(), (8, 8));
    let offset = PlotRect {
        gutter: 1,
        top: 1,
        columns: 3,
        rows: 3,
    };
    canvas.patch(3, 5, offset, Some((0.5, RED)));
    assert_eq!(canvas.get(11, 13), Some(RED));
    assert_eq!(canvas.get(10, 13), None);
}

#[test]
fn text_blits_the_baked_font_and_skips_what_it_lacks() {
    let mut canvas = PixelCanvas::new(4, 1, (8, 8));
    canvas.text(0, 0, "A", RED);
    let ink = |canvas: &PixelCanvas, x0: usize| {
        (0..8)
            .flat_map(|y| (0..8).map(move |x| (x, y)))
            .filter(|&(x, y)| canvas.get(x0 + x, y).is_some())
            .count()
    };
    let drawn = ink(&canvas, 0);
    assert!(drawn > 10, "letter A should ink a good part of its cell");
    // The glyph never leaks outside its cell at scale 1.
    assert_eq!(ink(&canvas, 8), 0);
    // Unsupported glyphs advance without ink: the é draws nothing, the B that
    // follows it lands one cell further right.
    canvas.text(1, 0, "\u{e9}B", RED);
    assert_eq!(ink(&canvas, 8), 0);
    assert!(ink(&canvas, 16) > 10);
}

#[test]
fn text_scales_up_in_large_cells() {
    let mut canvas = PixelCanvas::new(2, 1, (16, 32));
    canvas.text(0, 0, "#", RED);
    let drawn = (0..32)
        .flat_map(|y| (0..16).map(move |x| (x, y)))
        .filter(|&(x, y)| canvas.get(x, y).is_some())
        .count();
    // At scale 2 every font pixel covers four device pixels.
    let mut reference = PixelCanvas::new(2, 1, (8, 8));
    reference.text(0, 0, "#", RED);
    let base = (0..8)
        .flat_map(|y| (0..8).map(move |x| (x, y)))
        .filter(|&(x, y)| reference.get(x, y).is_some())
        .count();
    assert_eq!(drawn, base * 4);
}

#[test]
fn a_stroke_override_outweighs_the_cell_default() {
    // The classic cell derives a hairline; the host asks for 3.
    let mut canvas = PixelCanvas::try_new(4, 2, (8, 16), Some(3)).unwrap();
    canvas.line((2.0, 8.0), (20.0, 8.0), RED);
    for y in 7..=9 {
        assert_eq!(canvas.get(10, y), Some(RED), "row {y} should carry ink");
    }
    assert_eq!(canvas.get(10, 5), None);
    assert_eq!(canvas.get(10, 11), None);
    // Zero behaves like unset: back to the derived hairline.
    let mut zero = PixelCanvas::try_new(4, 2, (8, 16), Some(0)).unwrap();
    zero.line((2.0, 8.0), (20.0, 8.0), RED);
    assert_eq!(zero.get(10, 8), Some(RED));
    assert_eq!(zero.get(10, 7), None);
}

#[test]
fn coverage_semantics_solid_ink_and_raw_rgba() {
    let mut canvas = PixelCanvas::new(2, 1, (8, 8));
    canvas.line((2.0, 4.0), (10.0, 4.0), RED);
    // Solid ink reads through get(); the raw pixel carries full alpha.
    assert_eq!(canvas.get(5, 4), Some(RED));
    assert_eq!(canvas.rgba(5, 4), [255, 0, 0, 255]);
    // Bare canvas is transparent in both views.
    assert_eq!(canvas.get(5, 1), None);
    assert_eq!(canvas.rgba(5, 1), [0, 0, 0, 0]);
    // Out of bounds is transparent, never a panic.
    assert_eq!(canvas.rgba(999, 999), [0, 0, 0, 0]);
}

#[test]
fn diagonal_strokes_carry_an_anti_aliased_fringe() {
    let mut canvas = PixelCanvas::new(4, 2, (8, 8));
    canvas.line((4.0, 4.0), (24.0, 11.0), RED);
    // The spine is solid; off-spine neighbors carry partial coverage —
    // some ink, less than solid — instead of the old hard staircase.
    let mut solid = 0;
    let mut fringe = 0;
    for y in 0..16 {
        for x in 0..32 {
            match canvas.rgba(x, y)[3] {
                0 => {}
                a if a >= 128 => solid += 1,
                _ => fringe += 1,
            }
        }
    }
    assert!(solid >= 20, "the stroke itself is solid: {solid}");
    assert!(fringe >= 8, "a soft edge exists: {fringe}");
}

#[test]
fn round_caps_extend_past_the_endpoints() {
    let mut canvas = PixelCanvas::try_new(4, 2, (8, 8), Some(4)).unwrap();
    canvas.line((10.0, 8.0), (20.0, 8.0), RED);
    // Ink extends past the endpoints into the cap; the cap's rim itself
    // is a soft fringe.
    assert!(canvas.get(9, 8).is_some(), "left cap");
    assert!(canvas.get(21, 8).is_some(), "right cap");
    assert!(canvas.rgba(8, 8)[3] > 0, "the rim carries fringe ink");
    // The cap is round: the corner past the endpoint at full radius in
    // both axes is outside.
    assert_eq!(canvas.get(8, 10), None);
}

#[test]
fn subpixel_endpoints_split_coverage_between_rows() {
    let mut canvas = PixelCanvas::new(4, 1, (8, 8));
    // A hairline exactly between two pixel rows: neither row is solid,
    // both carry the same partial ink.
    canvas.line((4.0, 3.5), (20.0, 3.5), RED);
    let above = canvas.rgba(10, 3)[3];
    let below = canvas.rgba(10, 4)[3];
    assert_eq!(above, below);
    assert!(above > 0 && above <= 128, "split coverage: {above}");
    // Neither row reads as solid ink.
    assert_eq!(canvas.get(10, 3), None);
    assert_eq!(canvas.get(10, 4), None);
}

#[test]
fn crossing_strokes_blend_where_they_overlap() {
    let mut canvas = PixelCanvas::new(4, 1, (8, 8));
    canvas.line((4.0, 3.5), (20.0, 3.5), Color::Rgb(200, 0, 0));
    canvas.line((4.0, 3.5), (20.0, 3.5), Color::Rgb(0, 0, 200));
    // Equal partial coverages: source-over leaves a mix, not a replace.
    let [r, _, b, a] = canvas.rgba(10, 3);
    assert!(r > 0 && b > 0, "both inks present: r={r} b={b}");
    assert!(a > 0);
}

#[test]
fn glow_lays_a_soft_halo_the_stroke_rides_over() {
    let mut canvas = PixelCanvas::new(4, 2, (8, 8));
    canvas.glow((4.0, 8.0), (24.0, 8.0), RED);
    canvas.line((4.0, 8.0), (24.0, 8.0), RED);
    // The stroke itself is full strength.
    assert_eq!(canvas.rgba(14, 8), [255, 0, 0, 255]);
    // Beyond the stroke, the halo carries faint ink that fades outward.
    let near = canvas.rgba(14, 9)[3];
    let far = canvas.rgba(14, 10)[3];
    assert!(near > 0 && near < 128, "near halo is faint: {near}");
    assert!(far < near, "the halo fades with distance: {far} < {near}");
    // Well outside the reach: nothing.
    assert_eq!(canvas.rgba(14, 13), [0; 4]);
}

#[test]
fn opacity_washes_fills_and_resets() {
    let mut canvas = PixelCanvas::new(4, 1, (8, 8));
    canvas.set_opacity(0.2);
    canvas.line((2.0, 3.0), (20.0, 3.0), RED);
    let washed = canvas.rgba(10, 3)[3];
    assert!((45..=58).contains(&washed), "a fifth of full ink: {washed}");
    // Solid ops route through the wash too.
    canvas.bar((2.0, 6.0), 5.0, 6.0, true, rect(), RED);
    assert!(canvas.rgba(3, 5)[3] < 128, "washed bar");
    canvas.set_opacity(1.0);
    canvas.line((2.0, 1.0), (20.0, 1.0), RED);
    assert_eq!(canvas.rgba(10, 1), [255, 0, 0, 255]);
}

#[test]
fn dashes_gap_and_flow_through_joints() {
    use crate::mark::Dash;
    use crate::plot::test_dash_segment;
    let mut canvas = PixelCanvas::new(8, 1, (8, 8));
    // Two joined segments; the pattern's phase carries across the joint.
    let mut phase = 0.0;
    test_dash_segment(
        &mut canvas,
        (0.0, 3.0),
        (24.0, 3.0),
        RED,
        Dash::Dashed,
        &mut phase,
    );
    test_dash_segment(
        &mut canvas,
        (24.0, 3.0),
        (60.0, 3.0),
        RED,
        Dash::Dashed,
        &mut phase,
    );
    let on: Vec<bool> = (0..61).map(|x| canvas.get(x, 3).is_some()).collect();
    let runs = on.windows(2).filter(|w| w[0] != w[1]).count();
    assert!(runs >= 8, "several dashes and gaps: {runs} transitions");
    assert!(on.iter().any(|&i| i) && on.iter().any(|&i| !i));
    // Dotted: isolated round dots.
    let mut dotted = PixelCanvas::new(8, 1, (8, 8));
    test_dash_segment(
        &mut dotted,
        (0.0, 3.0),
        (60.0, 3.0),
        RED,
        Dash::Dotted,
        &mut 0.0,
    );
    let lit = (0..61).filter(|&x| dotted.get(x, 3).is_some()).count();
    assert!((5..=25).contains(&lit), "sparse dots: {lit} lit");
}

#[test]
fn accumulated_ink_brightens_where_markers_pile_up() {
    let mut canvas = PixelCanvas::new(4, 2, (8, 8));
    canvas.set_opacity(0.3);
    canvas.set_accumulate(true);
    for _ in 0..3 {
        canvas.point(10.0, 8.0, PointShape::Dot, RED);
    }
    canvas.point(24.0, 8.0, PointShape::Dot, RED);
    canvas.set_accumulate(false);
    canvas.set_opacity(1.0);
    let piled = canvas.rgba(10, 8)[3];
    let single = canvas.rgba(24, 8)[3];
    assert!(piled > single, "overplotting adds up: {piled} > {single}");
    assert!(piled >= 3 * single - 6, "roughly linear accumulation");
    // Ordinary blending would have capped a same-color pile at one coat.
    let mut plain = PixelCanvas::new(4, 2, (8, 8));
    plain.set_opacity(0.3);
    for _ in 0..3 {
        plain.point(10.0, 8.0, PointShape::Dot, RED);
    }
    assert_eq!(plain.rgba(10, 8)[3], single);
}

#[test]
fn notes_ride_their_anchor_not_the_cell_grid() {
    let mut canvas = PixelCanvas::new(6, 2, (8, 8));
    // An anchor mid-cell: the ink starts at the anchor's pixel column,
    // vertically centered on it — where cell-snapped text could not sit.
    canvas.note(11.0, 8.0, (8.0, 8.0), "A", RED);
    let ink: Vec<(usize, usize)> = (0..16)
        .flat_map(|y| (0..48).map(move |x| (x, y)))
        .filter(|&(x, y)| canvas.get(x, y).is_some())
        .collect();
    assert!(!ink.is_empty());
    let min_x = ink.iter().map(|&(x, _)| x).min().unwrap();
    let (min_y, max_y) = (
        ink.iter().map(|&(_, y)| y).min().unwrap(),
        ink.iter().map(|&(_, y)| y).max().unwrap(),
    );
    assert!((11..=13).contains(&min_x), "starts at the anchor: {min_x}");
    assert!(
        min_y >= 4 && max_y <= 12,
        "centered on y=8: {min_y}..{max_y}"
    );
}
