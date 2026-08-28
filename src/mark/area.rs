//! The area mark: a filled region between two edges.

use crate::data::{IntoSeries, Series};
use crate::render::Color;

/// A filled region: from the zero baseline up to a series, or between two series.
///
/// Fills are drawn as vertical subpixel runs, so they are solid in every charset
/// and their edges keep subpixel precision. Gaps (`NaN`) in any edge break the fill.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Area<'a> {
    pub(crate) x: Option<Series<'a>>,
    pub(crate) low: Option<Series<'a>>,
    pub(crate) high: Series<'a>,
    pub(crate) horizontal: bool,
    pub(crate) color: Option<Color>,
    pub(crate) label: Option<String>,
    /// Absent means opaque; wire documents omit it then.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Option::is_none", default)
    )]
    pub(crate) opacity: Option<f64>,
}

impl<'a> Area<'a> {
    /// A fill from zero up to `values`, against their indices.
    pub fn y(values: impl IntoSeries<'a>) -> Area<'a> {
        Area {
            x: None,
            low: None,
            high: values.into_series(),
            horizontal: false,
            color: None,
            label: None,
            opacity: None,
        }
    }

    /// A fill from zero up to `y` over `x`.
    ///
    /// # Panics
    ///
    /// Panics if the two series have different lengths.
    pub fn xy(x: impl IntoSeries<'a>, y: impl IntoSeries<'a>) -> Area<'a> {
        let x = x.into_series();
        let y = y.into_series();
        let area = Area {
            x: Some(x),
            low: None,
            high: y,
            horizontal: false,
            color: None,
            label: None,
            opacity: None,
        };
        area.validate()
            .expect("Area::xy requires series of equal length");
        area
    }

    /// A band between `low` and `high` over `x` — confidence intervals, stacked
    /// layers, min/max envelopes.
    ///
    /// # Panics
    ///
    /// Panics if the three series have different lengths.
    pub fn between(
        x: impl IntoSeries<'a>,
        low: impl IntoSeries<'a>,
        high: impl IntoSeries<'a>,
    ) -> Area<'a> {
        let x = x.into_series();
        let low = low.into_series();
        let high = high.into_series();
        let area = Area {
            x: Some(x),
            low: Some(low),
            high,
            horizontal: false,
            color: None,
            label: None,
            opacity: None,
        };
        area.validate()
            .expect("Area::between requires series of equal length");
        area
    }

    /// A horizontal band: for each `y`, a fill between `x_low` and `x_high` — the
    /// shape of violins and horizontal envelopes.
    ///
    /// # Panics
    ///
    /// Panics if the three series have different lengths.
    pub fn horizontal(
        y: impl IntoSeries<'a>,
        x_low: impl IntoSeries<'a>,
        x_high: impl IntoSeries<'a>,
    ) -> Area<'a> {
        let y = y.into_series();
        let x_low = x_low.into_series();
        let x_high = x_high.into_series();
        let area = Area {
            x: Some(y),
            low: Some(x_low),
            high: x_high,
            horizontal: true,
            color: None,
            label: None,
            opacity: None,
        };
        area.validate()
            .expect("Area::horizontal requires series of equal length");
        area
    }

    /// Sets an explicit color; without one, layers take colors from the palette.
    #[must_use]
    pub fn color(mut self, color: Color) -> Area<'a> {
        self.color = Some(color);
        self
    }

    /// A translucent wash on pixel targets: `opacity` in `(0, 1]` scales
    /// the fill's coverage, so the terminal background — and layers
    /// beneath — read through it. Cell targets stay solid.
    ///
    /// # Panics
    ///
    /// Panics if `opacity` is not finite or outside `(0, 1]`.
    #[must_use]
    pub fn opacity(mut self, opacity: f64) -> Area<'a> {
        self.opacity = Some(opacity);
        self.validate()
            .expect("Area::opacity requires a value in (0, 1]");
        self
    }

    /// Names this layer in the legend.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Area<'a> {
        self.label = Some(label.into());
        self
    }

    /// Checks the paired channels, including values decoded without a constructor.
    pub(crate) fn validate(&self) -> crate::Result<()> {
        if let Some(x) = &self.x {
            super::pair("Area: x and high", x.len(), self.high.len())?;
        }
        if let Some(low) = &self.low {
            super::pair("Area: low and high", low.len(), self.high.len())?;
        }
        if let Some(opacity) = self.opacity
            && !(opacity.is_finite() && 0.0 < opacity && opacity <= 1.0)
        {
            return Err(crate::Error::InvalidParameter {
                detail: "Area opacity must be finite and in (0, 1]",
            });
        }
        Ok(())
    }

    /// Detaches from any borrowed storage, making the mark `'static`.
    pub fn into_owned(self) -> Area<'static> {
        Area {
            x: self.x.map(Series::into_owned),
            low: self.low.map(Series::into_owned),
            high: self.high.into_owned(),
            horizontal: self.horizontal,
            color: self.color,
            label: self.label,
            opacity: self.opacity,
        }
    }
}

impl std::fmt::Debug for Area<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Area")
            .field("points", &self.high.len())
            .field("banded", &self.low.is_some())
            .field("color", &self.color)
            .finish()
    }
}

#[cfg(test)]
#[path = "tests/area_tests.rs"]
mod tests;
