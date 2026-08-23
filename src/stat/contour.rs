//! Contour lines: marching squares over a uniform grid.

/// The iso-lines of one level: segment endpoints flattened into `x`/`y` with a
/// NaN joint after each segment, ready for [`crate::Line::xy`] as drawn.
///
/// Coordinates are in grid units — the value at row `r`, column `c` sits at
/// `(c, r)`, matching [`crate::Cells::matrix`] with row 0 at the bottom.
#[derive(Debug, Clone, PartialEq)]
pub struct Contour {
    /// The level this line traces.
    pub level: f64,
    /// Segment x coordinates, NaN after each segment.
    pub x: Vec<f64>,
    /// Segment y coordinates, NaN after each segment.
    pub y: Vec<f64>,
}

/// Traces iso-lines of a row-major grid (row 0 at the bottom) at each level.
///
/// Classic marching squares: every 2×2 block of grid values contributes the
/// segments of the crossing contour, with endpoints linearly interpolated onto the
/// block's edges. Saddle blocks are disambiguated by the block's center average.
/// Blocks touching a non-finite value produce nothing — gaps in the data are gaps
/// in the contour. Adjacent blocks interpolate shared edges identically, so joined
/// segments meet exactly.
///
/// # Panics
///
/// Panics if `columns` is zero or does not divide `values.len()`.
pub fn contours(values: &[f64], columns: usize, levels: &[f64]) -> Vec<Contour> {
    assert!(
        columns > 0 && values.len().is_multiple_of(columns),
        "contours requires a rectangular grid"
    );
    let rows = values.len() / columns;
    levels
        .iter()
        .map(|&level| {
            let mut line = Contour {
                level,
                x: Vec::new(),
                y: Vec::new(),
            };
            for r in 0..rows.saturating_sub(1) {
                for c in 0..columns - 1 {
                    let corners = [
                        values[r * columns + c],           // bottom-left
                        values[r * columns + c + 1],       // bottom-right
                        values[(r + 1) * columns + c + 1], // top-right
                        values[(r + 1) * columns + c],     // top-left
                    ];
                    if corners.iter().any(|v| !v.is_finite()) {
                        continue;
                    }
                    march(&mut line, (c as f64, r as f64), corners, level);
                }
            }
            line
        })
        .collect()
}

/// One block edge, by index: bottom, right, top, left. Grid-unit endpoints plus
/// the corner indices holding the values at them, oriented left-to-right /
/// bottom-to-top so that neighboring blocks interpolate a shared edge from the
/// same values in the same order — joined segments meet exactly, not within an
/// ulp.
const EDGES: [Edge; 4] = [
    Edge {
        from: (0.0, 0.0),
        to: (1.0, 0.0),
        a: 0,
        b: 1,
    },
    Edge {
        from: (1.0, 0.0),
        to: (1.0, 1.0),
        a: 1,
        b: 2,
    },
    Edge {
        from: (0.0, 1.0),
        to: (1.0, 1.0),
        a: 3,
        b: 2,
    },
    Edge {
        from: (0.0, 0.0),
        to: (0.0, 1.0),
        a: 0,
        b: 3,
    },
];

/// Grid-unit endpoints of an edge and the corner indices whose values sit at them.
struct Edge {
    from: (f64, f64),
    to: (f64, f64),
    a: usize,
    b: usize,
}

/// Emits the segments crossing one 2×2 block with corners `[bl, br, tr, tl]`.
fn march(line: &mut Contour, origin: (f64, f64), corners: [f64; 4], level: f64) {
    let case = corners
        .iter()
        .enumerate()
        .filter(|&(_, &v)| v >= level)
        .fold(0usize, |bits, (index, _)| bits | 1 << index);
    // Edge pairs each segment connects, per case (bits: 1 bl, 2 br, 4 tr, 8 tl).
    // The two saddle cases connect around whichever diagonal the center average
    // puts inside the level.
    let center_inside = corners.iter().sum::<f64>() / 4.0 >= level;
    let segments: &[(usize, usize)] = match case {
        0 | 15 => &[],
        1 | 14 => &[(3, 0)],
        2 | 13 => &[(0, 1)],
        3 | 12 => &[(3, 1)],
        4 | 11 => &[(1, 2)],
        6 | 9 => &[(0, 2)],
        7 | 8 => &[(2, 3)],
        5 => {
            if center_inside {
                &[(0, 1), (2, 3)]
            } else {
                &[(3, 0), (1, 2)]
            }
        }
        _ => {
            if center_inside {
                &[(3, 0), (1, 2)]
            } else {
                &[(0, 1), (2, 3)]
            }
        }
    };
    for &(from, to) in segments {
        for edge in [from, to] {
            let (x, y) = crossing(edge, corners, level);
            line.x.push(origin.0 + x);
            line.y.push(origin.1 + y);
        }
        line.x.push(f64::NAN);
        line.y.push(f64::NAN);
    }
}

/// Where the level crosses `edge`, interpolated between its corner values.
fn crossing(edge: usize, corners: [f64; 4], level: f64) -> (f64, f64) {
    let Edge { from, to, a, b } = EDGES[edge];
    let (a, b) = (corners[a], corners[b]);
    let t = crate::numeric::inverse_lerp(a, b, level);
    (
        crate::numeric::lerp(from.0, to.0, t),
        crate::numeric::lerp(from.1, to.1, t),
    )
}

#[cfg(test)]
#[path = "tests/contour_tests.rs"]
mod tests;
