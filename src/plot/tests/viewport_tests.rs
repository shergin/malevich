use crate::plot::Frame;
use crate::{Line, Plot, Viewport};

#[test]
fn the_automatic_viewport_transforms_nothing() {
    let auto = Viewport::auto();
    assert!(auto.is_auto());
    assert_eq!(auto.zoom_x(0.5, 5.0).x(), None, "no window to zoom");
    assert_eq!(auto.pan_x(0.5).x(), None, "no window to pan");
    assert_eq!(auto.clamp_x(0.0, 1.0).x(), None);
}

#[test]
fn zoom_scales_around_the_anchor() {
    let view = Viewport::auto().with_x(0.0, 10.0);
    assert_eq!(view.zoom_x(0.5, 5.0).x(), Some((2.5, 7.5)));
    assert_eq!(view.zoom_x(0.5, 0.0).x(), Some((0.0, 5.0)), "anchored edge");
    assert_eq!(view.zoom_x(2.0, 5.0).x(), Some((-5.0, 15.0)), "zooming out");
    assert_eq!(
        view.zoom_x(0.0, 5.0).x(),
        Some((0.0, 10.0)),
        "degenerate factor ignored"
    );
    assert_eq!(view.zoom_x(f64::NAN, 5.0).x(), Some((0.0, 10.0)));
}

#[test]
fn pan_shifts_by_a_fraction_of_the_span() {
    let view = Viewport::auto().with_x(0.0, 10.0).with_y(100.0, 200.0);
    assert_eq!(view.pan_x(0.1).x(), Some((1.0, 11.0)));
    assert_eq!(view.pan_x(-0.5).x(), Some((-5.0, 5.0)));
    assert_eq!(view.pan_y(0.25).y(), Some((125.0, 225.0)));
    assert_eq!(
        view.pan_x(f64::INFINITY).x(),
        Some((0.0, 10.0)),
        "non-finite ignored"
    );
}

#[test]
fn clamp_slides_the_window_inside_the_extent() {
    let view = Viewport::auto().with_x(8.0, 12.0);
    assert_eq!(view.clamp_x(0.0, 10.0).x(), Some((6.0, 10.0)));
    assert_eq!(view.clamp_x(9.0, 20.0).x(), Some((9.0, 13.0)));
    assert_eq!(
        view.clamp_x(0.0, 20.0).x(),
        Some((8.0, 12.0)),
        "already inside"
    );
    assert_eq!(
        view.clamp_x(9.0, 10.0).x(),
        Some((9.0, 10.0)),
        "wider than the extent"
    );
}

#[test]
fn tail_follows_the_stream_and_reset_returns_to_auto() {
    let view = Viewport::auto().tail(100.0, 30.0);
    assert_eq!(view.x(), Some((70.0, 100.0)));
    assert!(view.reset().is_auto());
    let both = Viewport::auto().with_x(0.0, 1.0).with_y(0.0, 1.0);
    assert_eq!(both.reset_x().x(), None);
    assert_eq!(both.reset_x().y(), Some((0.0, 1.0)));
}

#[test]
fn invalid_windows_are_ignored() {
    let view = Viewport::auto().with_x(f64::NAN, 1.0);
    assert_eq!(view.x(), None);
    assert_eq!(Viewport::auto().with_x(3.0, 3.0).x(), None, "empty window");
    assert_eq!(Viewport::auto().tail(f64::NAN, 5.0).x(), None);
    assert_eq!(Viewport::auto().tail(10.0, 0.0).x(), None);
}

#[test]
fn a_mapping_seeds_the_viewport_with_the_resolved_domains() {
    let plot = Plot::new()
        .layer(Line::xy(&[0.0, 10.0][..], &[0.0, 10.0][..]))
        .x_domain(0.0, 10.0)
        .y_domain(0.0, 10.0);
    let view = plot.mapping(&Frame::plain(60, 20)).viewport();
    assert_eq!(view.x(), Some((0.0, 10.0)));
    assert_eq!(view.y(), Some((0.0, 10.0)));
}

#[test]
fn a_log_mapping_zooms_in_decade_space() {
    let plot = Plot::new()
        .layer(Line::xy(&[1.0, 1000.0][..], &[1.0, 1000.0][..]))
        .log_x()
        .x_domain(1.0, 1000.0);
    let view = plot.mapping(&Frame::plain(60, 20)).viewport();
    assert_eq!(view.x(), Some((1.0, 1000.0)));
    let zoomed = view.zoom_x(0.5, 10.0).x().expect("still fixed");
    // Decades (0, 3) halved around 1: (0.5, 2).
    assert!((zoomed.0.log10() - 0.5).abs() < 1e-9, "lo: {}", zoomed.0);
    assert!((zoomed.1.log10() - 2.0).abs() < 1e-9, "hi: {}", zoomed.1);
    let panned = view.pan_x(1.0 / 3.0).x().expect("still fixed");
    assert!((panned.0.log10() - 1.0).abs() < 1e-9, "log pan in decades");
}

#[test]
fn a_bands_mapping_leaves_the_axis_unfixed() {
    let plot = crate::bar(["a", "b"], &[1.0, 2.0][..]);
    let view = plot.mapping(&Frame::plain(40, 12)).viewport();
    assert_eq!(view.x(), None, "categorical axes have no window");
    assert!(view.y().is_some(), "the numeric axis is fixed");
}

#[test]
fn applying_a_viewport_equals_setting_the_domains() {
    let data: Vec<f64> = (0..100).map(|i| (f64::from(i) * 0.3).sin()).collect();
    let plot = Plot::new().layer(Line::y(&data[..]));
    let frame = Frame::plain(60, 18);
    let view = Viewport::auto().with_x(20.0, 60.0).with_y(-0.5, 0.5);
    let via_viewport = plot.clone().viewport(view).render(&frame);
    let via_domains = plot
        .clone()
        .x_domain(20.0, 60.0)
        .y_domain(-0.5, 0.5)
        .render(&frame);
    assert_eq!(via_viewport, via_domains, "viewport is domain sugar");
    let untouched = plot.clone().viewport(Viewport::auto()).render(&frame);
    assert_eq!(untouched, plot.render(&frame), "auto leaves the plot alone");
}
