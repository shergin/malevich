use crate::pixel::{Graphics, Protocol};
use crate::plot::Frame;
use crate::scale::Scale;
use crate::{Color, Line, Plot, Theme};

fn wave() -> Plot<'static> {
    Plot::new()
        .layer(Line::y(&[0.0, 2.0, 1.0, 3.0, 0.5][..]))
        .title("panel")
}

fn graphics() -> Graphics {
    Graphics::new(Protocol::Kitty)
}

fn no_markup_surprises(svg: &str) {
    assert!(
        svg.chars()
            .all(|glyph| glyph == '\n' || !glyph.is_control()),
        "control byte in {svg}"
    );
    assert!(
        !svg.contains("<image"),
        "an embedded image is stripped by sanitizers"
    );
    assert!(!svg.contains("<script"));
    assert!(svg.contains("shape-rendering=\"crispEdges\""));
}

#[test]
fn a_pixel_card_draws_the_panel_as_rectangles_and_leaves_the_staircase_out() {
    let frame = Frame::portable(48, 14);
    let plot = wave();
    let cell = plot.to_svg(&frame);
    let pixels = plot.to_svg_pixels(&frame, &graphics());
    assert_ne!(cell, pixels);
    assert!(
        cell.contains("height=\"8\""),
        "the cell card keeps quadrant steps"
    );
    assert!(
        pixels.contains("<path fill="),
        "the panel is paths of device-pixel rectangles"
    );
    assert!(
        !pixels.contains("height=\"8\""),
        "quadrant steps do not survive under the panel"
    );
    assert!(pixels.contains(">panel</text>"));
    assert!(cell.contains(">panel</text>"));
    no_markup_surprises(&pixels);
    assert_eq!(pixels, plot.try_to_svg_pixels(&frame, &graphics()).unwrap());
}

#[test]
fn an_unlabeled_line_takes_the_card_foreground() {
    let frame = Frame::portable(40, 12);
    let plot = Plot::new().layer(Line::y(&[0.0, 1.0, 0.5, 2.0][..]));
    let dark = plot.to_svg_pixels(&frame, &graphics());
    assert!(dark.contains("<path fill=\"#e6edf3\""));
    assert!(!dark.contains("#808080"));
    let light_frame = Frame {
        theme: Theme::LIGHT,
        ..frame
    };
    let light = plot.to_svg_pixels(&light_frame, &graphics());
    assert_ne!(dark, light);
    assert!(light.contains("<path fill=\"#1f2328\""));
    assert!(!light.contains("#e6edf3"));
    assert!(!light.contains("#808080"));
}

#[test]
fn a_named_line_keeps_the_palette_rgb_the_terminal_canvas_uses() {
    let frame = Frame::portable(36, 12);
    let named = Plot::new().layer(Line::y(&[0.0, 1.0, 2.0][..]).color(Color::Cyan));
    let plain = Plot::new().layer(Line::y(&[0.0, 1.0, 2.0][..]));
    let (_, foreground) = frame.theme.card_rgb();
    assert_eq!(
        crop(&named, &frame, None),
        crop(&named, &frame, Some(foreground))
    );
    assert_ne!(
        crop(&plain, &frame, None),
        crop(&plain, &frame, Some(foreground))
    );
    let svg = named.to_svg_pixels(&frame, &graphics());
    assert!(svg.contains("#00cdcd"));
    let card = crop(&plain, &frame, Some(foreground));
    assert!(opaque(&card, foreground));
    assert!(!opaque(&card, (128, 128, 128)));
}

#[test]
fn a_zero_cell_size_degrades_to_the_cell_card() {
    let frame = Frame::portable(40, 12);
    let plot = wave();
    let graphics = Graphics::new(Protocol::Sixel).cell_size(0, 16);
    assert_eq!(plot.to_svg_pixels(&frame, &graphics), plot.to_svg(&frame));
    assert_eq!(
        plot.try_to_svg_pixels(&frame, &graphics).unwrap(),
        plot.to_svg(&frame)
    );
}

#[test]
fn an_unvalidated_spec_still_draws_pixels_like_render_pixels() {
    // Empty explicit bands fail validation, yet the infallible renderers draw
    // what they can. The pixel card follows render_pixels, not the checked twin.
    let frame = Frame::portable(40, 12);
    let plot = wave().x_scale(Scale::Bands(Vec::new()));
    assert!(plot.validate().is_err());
    assert!(plot.try_to_svg_pixels(&frame, &graphics()).is_err());
    let pixels = plot.to_svg_pixels(&frame, &graphics());
    assert_ne!(pixels, plot.to_svg(&frame));
    assert!(pixels.contains("<path fill="));
}

#[test]
fn a_text_table_stays_the_cell_card() {
    let frame = Frame::portable(24, 8);
    let plot = crate::table(["a"], ["v"], &[1.0][..]);
    assert_eq!(plot.to_svg_pixels(&frame, &graphics()), plot.to_svg(&frame));
}

#[test]
fn the_pixel_card_is_deterministic_and_ignores_the_protocol() {
    let frame = Frame::portable(40, 12);
    let plot = wave();
    let kitty = plot.to_svg_pixels(&frame, &Graphics::new(Protocol::Kitty));
    assert_eq!(
        kitty,
        plot.to_svg_pixels(&frame, &Graphics::new(Protocol::Kitty))
    );
    assert_eq!(
        kitty,
        plot.to_svg_pixels(&frame, &Graphics::new(Protocol::Sixel))
    );
    assert_eq!(
        kitty,
        plot.to_svg_pixels(&frame, &Graphics::new(Protocol::ITerm2))
    );
    assert_ne!(
        kitty,
        plot.to_svg_pixels(&frame, &Graphics::new(Protocol::Kitty).cell_size(4, 8))
    );
    assert_ne!(
        kitty,
        plot.to_svg_pixels(&frame, &Graphics::new(Protocol::Kitty).stroke(4))
    );
    no_markup_surprises(&kitty);
}

fn opaque(rgba: &[u8], color: (u8, u8, u8)) -> bool {
    rgba.as_chunks::<4>()
        .0
        .contains(&[color.0, color.1, color.2, 255])
}

fn crop(plot: &Plot<'_>, frame: &Frame, ink: Option<(u8, u8, u8)>) -> Vec<u8> {
    let (_, canvas, rect, _) = plot
        .try_rasterize_hybrid(frame, (8, 16), None, ink)
        .expect("raster");
    let (_, _, rgba) = crate::pixel::crop_rgba(&canvas, rect).expect("crop");
    rgba
}
