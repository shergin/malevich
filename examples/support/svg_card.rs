// Included by gallery examples for the site's gallery cards. Without `--svg`
// nothing changes, so the pipe snapshot stays byte-identical. With it, the
// example prints the same plot values in the same frames as the dark quadrant
// SVG card, one `<svg>` per plot, split by `<!-- card -->`.

/// One plot, in the frame its pipe text uses.
#[allow(dead_code)]
fn svg_card(plot: &malevich::Plot<'_>, frame: &malevich::Frame) -> bool {
    svg_cards(&[(plot, *frame)])
}

/// Several plots, each in the frame its pipe text uses.
#[allow(dead_code)]
fn svg_cards(cards: &[(&malevich::Plot<'_>, malevich::Frame)]) -> bool {
    print_cards(cards, malevich::Charset::Quadrants)
}

/// The panes of a `Grid` of `columns` columns, each in the frame the grid
/// gives it: the frame split evenly, one blank cell between neighbors.
#[allow(dead_code)]
fn svg_grid(plots: &[&malevich::Plot<'_>], columns: usize, frame: &malevich::Frame) -> bool {
    let columns = columns.min(plots.len());
    let rows = plots.len().div_ceil(columns);
    let pane = malevich::Frame {
        width: (frame.width - (columns - 1)) / columns,
        height: (frame.height - (rows - 1)) / rows,
        ..*frame
    };
    let cards: Vec<_> = plots.iter().map(|plot| (*plot, pane)).collect();
    svg_cards(&cards)
}

/// The charset ladder: each frame keeps its charset, because that is the chart.
#[allow(dead_code)]
fn svg_card_frames(cards: &[(&malevich::Plot<'_>, malevich::Frame)]) -> bool {
    if !std::env::args().any(|argument| argument == "--svg") {
        return false;
    }
    for (index, (plot, frame)) in cards.iter().enumerate() {
        if index > 0 {
            println!("<!-- card -->");
        }
        let frame = malevich::Frame {
            color: malevich::ColorMode::TrueColor,
            theme: malevich::Theme::DARK,
            ..*frame
        };
        print!("{}", plot.to_svg(&frame));
    }
    true
}

fn print_cards(cards: &[(&malevich::Plot<'_>, malevich::Frame)], charset: malevich::Charset) -> bool {
    let cards: Vec<_> = cards
        .iter()
        .map(|(plot, frame)| (*plot, malevich::Frame { charset, ..*frame }))
        .collect();
    svg_card_frames(&cards)
}
