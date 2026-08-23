//! The bars mark: values as filled columns over a categorical axis.

use crate::data::{IntoSeries, Series};
use crate::render::Color;

/// Filled vertical bars, one per category, rising (or falling) from a zero
/// baseline — the y domain always includes zero, because bar length *is* the
/// encoding.
///
/// Bars put a band scale on the x axis. Other layers in the same plot position
/// their x values against category indices: `0.0` is the center of the first band,
/// `1.0` the second, and so on.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bars<'a> {
    pub(crate) placement: Placement,
    pub(crate) values: Series<'a>,
    pub(crate) color: Option<Color>,
    pub(crate) label: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub(crate) color_by: Option<Vec<String>>,
}

/// Where bars sit on the x axis: named bands, or contiguous numeric spans.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) enum Placement {
    Bands(Vec<String>),
    Spans { start: f64, width: f64 },
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
            color: None,
            label: None,
            color_by: None,
        };
        bars.validate()
            .expect("Bars::spans requires a finite start and a positive width");
        bars
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
        let groups: Vec<String> = groups.into_iter().map(Into::into).collect();
        self.color_by = Some(groups);
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
        }
        if let Some(groups) = &self.color_by {
            super::pair("Bars: color_by and values", groups.len(), self.values.len())?;
        }
        Ok(())
    }

    /// Detaches from any borrowed storage, making the mark `'static`.
    pub fn into_owned(self) -> Bars<'static> {
        Bars {
            placement: self.placement,
            values: self.values.into_owned(),
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
            .field("color", &self.color)
            .finish()
    }
}

#[cfg(test)]
#[path = "tests/bars_tests.rs"]
mod tests;
