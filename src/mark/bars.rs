//! The bars mark: values as filled columns over a categorical axis.

use crate::data::{IntoSeries, Series};
use crate::mark::Categories;
use crate::render::Color;

/// Filled vertical bars rising (or falling) from a zero baseline — the y domain
/// always includes zero, because bar length *is* the encoding — or, with
/// [`Bars::base`], from a per-bar base: bar `i` spans
/// `base[i] .. base[i] + value[i]`, the shape of stacked bars and waterfalls,
/// where the value still encodes the segment's length.
///
/// Bars sit on the x axis three ways. [`Bars::new`] places one per named band
/// and puts a band scale on the x axis; other layers in the same plot then
/// position their x values against category indices: `0.0` is the center of the
/// first band, `1.0` the second, and so on. [`Bars::spans`] covers contiguous
/// numeric spans (the histogram shape). [`Bars::at`] centers each bar at a free
/// numeric position — side-by-side grouped bars within bands, bars over a time
/// axis.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bars<'a> {
    pub(crate) placement: Placement<'a>,
    pub(crate) values: Series<'a>,
    /// Absent means the zero baseline; wire documents omit it then.
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub(crate) base: Option<Series<'a>>,
    pub(crate) color: Option<Color>,
    pub(crate) label: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub(crate) color_by: Option<Categories>,
}

/// Where bars sit on the x axis: named bands, contiguous numeric spans, or
/// free numeric centers.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) enum Placement<'a> {
    Bands(Vec<String>),
    Spans { start: f64, width: f64 },
    At { x: Series<'a>, width: f64 },
}

impl<'a> Bars<'a> {
    /// Bars for `values`, one per category.
    ///
    /// # Panics
    ///
    /// Panics if the number of categories differs from the number of values.
    pub fn new(
        categories: impl IntoIterator<Item = impl Into<String>>,
        values: impl IntoSeries<'a>,
    ) -> Bars<'a> {
        let categories: Vec<String> = categories.into_iter().map(Into::into).collect();
        let values = values.into_series();
        let bars = Bars {
            placement: Placement::Bands(categories),
            values,
            base: None,
            color: None,
            label: None,
            color_by: None,
        };
        bars.validate()
            .expect("Bars::new requires one category per value");
        bars
    }

    /// Bars over contiguous numeric spans: bar `i` covers
    /// `[start + i * width, start + (i + 1) * width)` on a continuous x axis.
    /// This is the histogram shape — see [`crate::hist`].
    ///
    /// # Panics
    ///
    /// Panics if `width` is not finite and positive or `start` is not finite.
    pub fn spans(start: f64, width: f64, values: impl IntoSeries<'a>) -> Bars<'a> {
        let bars = Bars {
            placement: Placement::Spans { start, width },
            values: values.into_series(),
            base: None,
            color: None,
            label: None,
            color_by: None,
        };
        bars.validate()
            .expect("Bars::spans requires a finite start and a positive width");
        bars
    }

    /// Bars centered at numeric positions: bar `i` covers
    /// `[x[i] - width / 2, x[i] + width / 2]` on a continuous x axis — or, on a
    /// bands axis, positions in band-index space (`0.0` is the first band's
    /// center), which is how grouped bars sit side by side within their bands:
    /// one layer per series, offset around each band's index. A gap (`NaN`) in
    /// `x` skips that bar.
    ///
    /// # Panics
    ///
    /// Panics if the two series have different lengths, or `width` is not
    /// finite and positive.
    pub fn at(x: impl IntoSeries<'a>, width: f64, values: impl IntoSeries<'a>) -> Bars<'a> {
        let bars = Bars {
            placement: Placement::At {
                x: x.into_series(),
                width,
            },
            values: values.into_series(),
            base: None,
            color: None,
            label: None,
            color_by: None,
        };
        bars.validate()
            .expect("Bars::at requires one position per value and a finite positive width");
        bars
    }

    /// Starts each bar at a per-bar base instead of zero: bar `i` spans
    /// `base[i] .. base[i] + value[i]`, so the value keeps encoding the
    /// segment's length. Stack bars by giving each layer the running total of
    /// the layers below it — the `low` half of [`stack`](crate::stat::stack) is
    /// exactly that. A gap (`NaN`) in the base skips that bar. The y domain
    /// includes zero only while some bars layer still rises from the zero
    /// baseline.
    ///
    /// # Panics
    ///
    /// Panics if the base length differs from the number of values.
    #[must_use]
    pub fn base(mut self, base: impl IntoSeries<'a>) -> Bars<'a> {
        self.base = Some(base.into_series());
        self.validate()
            .expect("Bars::base requires one base per value");
        self
    }

    /// Sets an explicit color; without one, layers take colors from the palette.
    #[must_use]
    pub fn color(mut self, color: Color) -> Bars<'a> {
        self.color = Some(color);
        self
    }

    /// Names this layer in the legend. The legend appears once any layer is
    /// labeled (and the frame is tall enough for it).
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Bars<'a> {
        self.label = Some(label.into());
        self
    }

    /// Colors each bar by its category: distinct group names (in order of
    /// first appearance) take colors from the plot's categorical
    /// [`Palette`](crate::scale::Palette) and appear in the legend. The bands
    /// on the axis stay the bars' own categories; this channel groups them.
    /// Replaces the constant color and layer label.
    ///
    /// # Panics
    ///
    /// Panics if the number of group names differs from the number of bars.
    #[must_use]
    pub fn color_by(mut self, groups: impl IntoIterator<Item = impl Into<String>>) -> Bars<'a> {
        self.color_by = Some(Categories::new(groups));
        self.validate()
            .expect("Bars::color_by requires one group per bar");
        self
    }

    /// Checks placement and channel invariants, including deserialized values.
    pub(crate) fn validate(&self) -> crate::Result<()> {
        match &self.placement {
            Placement::Bands(categories) => {
                super::pair(
                    "Bars: categories and values",
                    categories.len(),
                    self.values.len(),
                )?;
            }
            Placement::Spans { start, width }
                if !(start.is_finite() && width.is_finite() && *width > 0.0) =>
            {
                return Err(crate::Error::InvalidParameter {
                    detail: "Bars spans need a finite start and finite positive width",
                });
            }
            Placement::Spans { .. } => {}
            Placement::At { x, width } => {
                super::pair("Bars: x and values", x.len(), self.values.len())?;
                if !(width.is_finite() && *width > 0.0) {
                    return Err(crate::Error::InvalidParameter {
                        detail: "Bars positions need a finite positive width",
                    });
                }
            }
        }
        if let Some(base) = &self.base {
            super::pair("Bars: base and values", base.len(), self.values.len())?;
        }
        if let Some(groups) = &self.color_by {
            super::pair("Bars: color_by and values", groups.len(), self.values.len())?;
        }
        Ok(())
    }

    /// Detaches from any borrowed storage, making the mark `'static`.
    pub fn into_owned(self) -> Bars<'static> {
        Bars {
            placement: match self.placement {
                Placement::Bands(categories) => Placement::Bands(categories),
                Placement::Spans { start, width } => Placement::Spans { start, width },
                Placement::At { x, width } => Placement::At {
                    x: x.into_owned(),
                    width,
                },
            },
            values: self.values.into_owned(),
            base: self.base.map(Series::into_owned),
            color: self.color,
            label: self.label,
            color_by: self.color_by,
        }
    }
}

impl std::fmt::Debug for Bars<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bars")
            .field("bars", &self.values.len())
            .field("based", &self.base.is_some())
            .field("color", &self.color)
            .finish()
    }
}

#[cfg(test)]
#[path = "tests/bars_tests.rs"]
mod tests;
