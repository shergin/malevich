//! The cells mark: a value grid drawn as shaded, colored cells.

use crate::data::{IntoSeries, Series};
use crate::scale::Colormap;

/// A grid of values — a heatmap, a matrix, a 2D histogram.
///
/// Values normalize to the grid's own finite extent. Colored cell output packs two
/// vertical samples into an upper half block's foreground and background; plain
/// output substitutes an averaged shade-ramp glyph (`░▒▓█`). The value is therefore
/// readable with or without color. Gaps (`NaN`) render as blanks. Row 0 is the
/// bottom row — matrix y grows upward like any other y axis.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cells<'a> {
    pub(crate) columns: usize,
    pub(crate) values: Series<'a>,
    pub(crate) extents: Option<((f64, f64), (f64, f64))>,
    pub(crate) colormap: Colormap,
}

impl<'a> Cells<'a> {
    /// A grid from row-major `values`, `columns` wide; the row count is
    /// `values.len() / columns`. Axes show cell indices unless
    /// [`Cells::extents`] maps them to data coordinates.
    ///
    /// # Panics
    ///
    /// Panics if `columns` is zero or does not divide the value count evenly.
    pub fn matrix(columns: usize, values: impl IntoSeries<'a>) -> Cells<'a> {
        Cells::try_matrix(columns, values)
            .expect("Cells::matrix requires columns to divide the value count evenly")
    }

    /// Fallible counterpart to [`Cells::matrix`] for data-driven grid shapes.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyDimension`](crate::Error::EmptyDimension) when
    /// `columns` is zero, or [`Error::NonRectangular`](crate::Error::NonRectangular)
    /// when the values do not fill complete rows.
    pub fn try_matrix(columns: usize, values: impl IntoSeries<'a>) -> crate::Result<Cells<'a>> {
        let values = values.into_series();
        let cells = Cells {
            columns,
            values,
            extents: None,
            colormap: Colormap::DEFAULT,
        };
        cells.validate()?;
        Ok(cells)
    }

    /// Maps the grid onto data coordinates: the x axis spans `x`, the y axis `y`.
    ///
    /// # Panics
    ///
    /// Panics if the extents are not finite or either span is empty. Reversed
    /// endpoints are accepted and flip that grid axis.
    #[must_use]
    pub fn extents(self, x: (f64, f64), y: (f64, f64)) -> Cells<'a> {
        self.try_extents(x, y)
            .expect("Cells::extents requires finite, non-empty bounds")
    }

    /// Fallible counterpart to [`Cells::extents`] for computed domains.
    /// Reversed finite endpoints remain an explicit axis flip.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidParameter`](crate::Error::InvalidParameter) for
    /// non-finite or equal endpoints.
    pub fn try_extents(mut self, x: (f64, f64), y: (f64, f64)) -> crate::Result<Cells<'a>> {
        self.extents = Some((x, y));
        self.validate()?;
        Ok(self)
    }

    /// Sets the colormap; the default approximates viridis.
    #[must_use]
    pub fn colormap(mut self, colormap: Colormap) -> Cells<'a> {
        self.colormap = colormap;
        self
    }

    /// Checks grid, extent, and colormap invariants after any construction path.
    pub(crate) fn validate(&self) -> crate::Result<()> {
        if self.columns == 0 {
            return Err(crate::Error::EmptyDimension {
                what: "Cells columns",
            });
        }
        if !self.values.len().is_multiple_of(self.columns) {
            return Err(crate::Error::NonRectangular {
                mark: "Cells",
                shape: (self.values.len(), self.columns),
            });
        }
        self.colormap.validate()?;
        if let Some((x, y)) = self.extents {
            if !(x.0.is_finite() && x.1.is_finite() && y.0.is_finite() && y.1.is_finite()) {
                return Err(crate::Error::InvalidParameter {
                    detail: "Cells extents must be finite",
                });
            }
            if x.0 == x.1 || y.0 == y.1 {
                return Err(crate::Error::InvalidParameter {
                    detail: "Cells extents must be non-empty",
                });
            }
        }
        Ok(())
    }

    /// Detaches from any borrowed storage, making the mark `'static`.
    pub fn into_owned(self) -> Cells<'static> {
        Cells {
            columns: self.columns,
            values: self.values.into_owned(),
            extents: self.extents,
            colormap: self.colormap,
        }
    }
}

impl std::fmt::Debug for Cells<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Cells")
            .field("columns", &self.columns)
            .field("rows", &(self.values.len() / self.columns.max(1)))
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
#[path = "tests/cells_tests.rs"]
mod tests;
