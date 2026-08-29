use super::{Graphics, Protocol};

#[test]
fn economical_halves_retina_density_and_keeps_ink_weight() {
    let retina = Graphics::new(Protocol::Kitty)
        .cell_size(18, 38)
        .economical();
    assert_eq!(retina.cell_size, (9, 19));
    assert_eq!(retina.stroke, Some(2), "one step above the halved default");

    // Ordinary density has nothing to give back.
    let ordinary = Graphics::new(Protocol::Kitty).economical();
    assert_eq!(ordinary.cell_size, (8, 16));
    assert_eq!(ordinary.stroke, None);

    // Sixel has no placement scaling and stays native.
    let sixel = Graphics::new(Protocol::Sixel)
        .cell_size(18, 38)
        .economical();
    assert_eq!(sixel.cell_size, (18, 38));
    assert_eq!(sixel.stroke, None);
}
