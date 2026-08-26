use super::Cells;

const fn assert_send_sync<T: Send + Sync>() {}
const _: () = assert_send_sync::<Cells<'static>>();

#[test]
#[should_panic(expected = "divide the value count")]
fn ragged_grids_panic() {
    Cells::matrix(3, &[1.0, 2.0][..]);
}

#[test]
fn checked_grid_shapes_return_typed_errors() {
    assert!(matches!(
        Cells::try_matrix(0, &[1.0, 2.0][..]),
        Err(crate::Error::EmptyDimension { .. })
    ));
    assert!(matches!(
        Cells::try_matrix(3, &[1.0, 2.0][..]),
        Err(crate::Error::NonRectangular { .. })
    ));
}

#[test]
fn debug_stays_curated() {
    let cells = Cells::matrix(2, &[1.0, 2.0, 3.0, 4.0][..]);
    let debug = format!("{cells:?}");
    assert!(debug.contains("columns: 2") && debug.contains("rows: 2"));
    assert!(!debug.contains("1.0"), "debug dumps data: {debug}");
}

#[test]
#[should_panic(expected = "non-empty bounds")]
fn degenerate_extents_panic() {
    let _ = Cells::matrix(2, &[1.0, 2.0][..]).extents((1.0, 1.0), (0.0, 1.0));
}

#[test]
fn checked_extents_return_typed_errors() {
    let cells = Cells::matrix(2, &[1.0, 2.0][..]);
    assert!(matches!(
        cells.try_extents((1.0, 1.0), (0.0, 1.0)),
        Err(crate::Error::InvalidParameter { .. })
    ));
}

#[test]
fn reversed_extents_remain_an_explicit_axis_flip() {
    let cells = Cells::matrix(2, &[1.0, 2.0][..]).extents((2.0, 0.0), (1.0, 0.0));
    assert_eq!(cells.extents, Some(((2.0, 0.0), (1.0, 0.0))));
}

#[test]
#[should_panic(expected = "divide the pixel count")]
fn ragged_rgb_grids_panic() {
    Cells::rgb(3, &[(0u8, 0, 0), (1, 1, 1)][..]);
}

#[test]
fn checked_rgb_shapes_return_typed_errors() {
    assert!(matches!(
        Cells::try_rgb(0, &[(0u8, 0, 0)][..]),
        Err(crate::Error::EmptyDimension { .. })
    ));
    assert!(matches!(
        Cells::try_rgb(2, &[(0u8, 0, 0)][..]),
        Err(crate::Error::NonRectangular { .. })
    ));
    let image = Cells::try_rgb(2, &[(0u8, 0, 0), (9, 9, 9)][..]).expect("one row");
    assert!(image.validate().is_ok());
    assert_eq!(image.rows(), 1);
}

#[test]
fn a_grid_with_both_channels_fails_validation() {
    // Unreachable through the constructors; only deserialization can build it.
    let mut cells = Cells::matrix(1, &[1.0][..]);
    cells.rgb = Some(vec![(0u8, 0, 0)].into());
    assert!(matches!(
        cells.validate(),
        Err(crate::Error::InvalidParameter { .. })
    ));
}

#[test]
fn rgb_grids_detach_into_owned_pixels() {
    let pixels = [(1u8, 2, 3), (4, 5, 6)];
    let owned = Cells::rgb(2, &pixels[..]).into_owned();
    assert_eq!(owned.rows(), 1);
    let debug = format!("{owned:?}");
    assert!(debug.contains("columns: 2"));
}

#[test]
#[should_panic(expected = "divide the label count")]
fn ragged_class_grids_panic() {
    Cells::classes(3, ["a", "b"]);
}

#[test]
fn checked_class_shapes_return_typed_errors() {
    assert!(matches!(
        Cells::try_classes(0, ["a"]),
        Err(crate::Error::EmptyDimension { .. })
    ));
    assert!(matches!(
        Cells::try_classes(2, ["a", "b", "c"]),
        Err(crate::Error::NonRectangular { .. })
    ));
    let regions = Cells::try_classes(2, ["a", "b", "b", "a"]).expect("rectangular");
    assert!(regions.validate().is_ok());
    assert_eq!(regions.rows(), 2);
    // Extents apply — the decision-boundary grid maps onto feature space.
    assert!(
        Cells::classes(2, ["a", "b", "b", "a"])
            .try_extents((-1.0, 1.0), (-1.0, 1.0))
            .is_ok()
    );
}

#[test]
fn a_grid_with_several_channels_fails_validation() {
    let mut cells = Cells::classes(1, ["a"]);
    cells.rgb = Some(vec![(0u8, 0, 0)].into());
    assert!(matches!(
        cells.validate(),
        Err(crate::Error::InvalidParameter { .. })
    ));
}
