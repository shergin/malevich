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
