//! The range mark: vertical intervals, with optional body and marker channels.

use crate::data::{IntoSeries, Series};
use crate::mark::Categories;
use crate::render::Color;

/// Vertical intervals at positions: error bars, box plots, candles, event ticks —
/// one mark, optional channels.
///
/// The base channels draw a thin whisker from `low` to `high` with end caps. An
/// optional [`Range::body`] fills a thick sub-interval (the box of a box plot, the
/// body of a candle), and an optional [`Range::marker`] draws a highlighted
/// crossbar (the median). Gaps (`NaN`) in any channel skip that interval.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Range<'a> {
    pub(crate) placement: RangePlacement<'a>,
    pub(crate) low: Series<'a>,
    pub(crate) high: Series<'a>,
    pub(crate) body: Option<(Series<'a>, Series<'a>)>,
    pub(crate) marker: Option<Series<'a>>,
    pub(crate) color: Option<Color>,
    pub(crate) label: Option<String>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    pub(crate) color_by: Option<Categories>,
}

/// Where ranges sit on the x axis.
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) enum RangePlacement<'a> {
    /// At numeric x positions (`None` means indices `0, 1, 2, …`).
    Numeric(Option<Series<'a>>),
    /// One range per named band, like bars.
    Bands(Vec<String>),
}

impl<'a> Range<'a> {
    /// Intervals from `low` to `high` at positions `x`.
    ///
    /// # Panics
    ///
    /// Panics if the series have different lengths.
    pub fn xy(
        x: impl IntoSeries<'a>,
        low: impl IntoSeries<'a>,
        high: impl IntoSeries<'a>,
    ) -> Range<'a> {
        let x = x.into_series();
        let low = low.into_series();
        let high = high.into_series();
        let range = Range {
            placement: RangePlacement::Numeric(Some(x)),
            low,
            high,
            body: None,
            marker: None,
            color: None,
            label: None,
            color_by: None,
        };
        range
            .validate()
            .expect("Range::xy requires series of equal length");
        range
    }

    /// Intervals from `low` to `high` against their indices.
    ///
    /// # Panics
    ///
    /// Panics if the series have different lengths.
    pub fn y(low: impl IntoSeries<'a>, high: impl IntoSeries<'a>) -> Range<'a> {
        let low = low.into_series();
        let high = high.into_series();
        let range = Range {
            placement: RangePlacement::Numeric(None),
            low,
            high,
            body: None,
            marker: None,
            color: None,
            label: None,
            color_by: None,
        };
        range
            .validate()
            .expect("Range::y requires series of equal length");
        range
    }

    /// One interval per named band — the box-plot arrangement. Bands share the
    /// categorical x axis with any bars in the plot.
    ///
    /// # Panics
    ///
    /// Panics if the series lengths differ from the category count.
    pub fn over(
        categories: impl IntoIterator<Item = impl Into<String>>,
        low: impl IntoSeries<'a>,
        high: impl IntoSeries<'a>,
    ) -> Range<'a> {
        let categories: Vec<String> = categories.into_iter().map(Into::into).collect();
        let low = low.into_series();
        let high = high.into_series();
        let range = Range {
            placement: RangePlacement::Bands(categories),
            low,
            high,
            body: None,
            marker: None,
            color: None,
            label: None,
            color_by: None,
        };
        range
            .validate()
            .expect("Range::over requires one category per interval");
        range
    }

    /// Adds a thick filled sub-interval — the box of a box plot.
    ///
    /// # Panics
    ///
    /// Panics if the body series lengths differ from the range's.
    #[must_use]
    pub fn body(mut self, low: impl IntoSeries<'a>, high: impl IntoSeries<'a>) -> Range<'a> {
        let low = low.into_series();
        let high = high.into_series();
        self.body = Some((low, high));
        self.validate()
            .expect("Range::body requires series matching the range length");
        self
    }

    /// Adds a highlighted crossbar at each value — the median of a box plot.
    ///
    /// # Panics
    ///
    /// Panics if the marker series length differs from the range's.
    #[must_use]
    pub fn marker(mut self, values: impl IntoSeries<'a>) -> Range<'a> {
        let values = values.into_series();
        self.marker = Some(values);
        self.validate()
            .expect("Range::marker requires a series matching the range length");
        self
    }

    /// Sets an explicit color; without one, layers take colors from the palette.
    #[must_use]
    pub fn color(mut self, color: Color) -> Range<'a> {
        self.color = Some(color);
        self
    }

    /// Names this layer in the legend.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Range<'a> {
        self.label = Some(label.into());
        self
    }

    /// Colors each interval by its category (up/down candles, condition
    /// groups): distinct categories in order of first appearance take colors
    /// from the plot's categorical [`Palette`](crate::scale::Palette) and
    /// appear in the legend. Replaces the constant color and layer label.
    ///
    /// # Panics
    ///
    /// Panics if the number of categories differs from the number of
    /// intervals.
    #[must_use]
    pub fn color_by(
        mut self,
        categories: impl IntoIterator<Item = impl Into<String>>,
    ) -> Range<'a> {
        self.color_by = Some(Categories::new(categories));
        self.validate()
            .expect("Range::color_by requires one category per interval");
        self
    }

    /// Checks placement and optional-channel invariants after any construction path.
    pub(crate) fn validate(&self) -> crate::Result<()> {
        let count = self.low.len();
        super::pair("Range: low and high", count, self.high.len())?;
        match &self.placement {
            RangePlacement::Numeric(Some(x)) => {
                super::pair("Range: x and low", x.len(), count)?;
            }
            RangePlacement::Bands(categories) => {
                super::pair("Range: categories and low", categories.len(), count)?;
            }
            RangePlacement::Numeric(None) => {}
        }
        if let Some((low, high)) = &self.body {
            super::pair("Range: body low and high", low.len(), high.len())?;
            super::pair("Range: body and low", low.len(), count)?;
        }
        if let Some(marker) = &self.marker {
            super::pair("Range: marker and low", marker.len(), count)?;
        }
        if let Some(categories) = &self.color_by {
            super::pair("Range: color_by and low", categories.len(), count)?;
        }
        Ok(())
    }

    /// Detaches from any borrowed storage, making the mark `'static`.
    pub fn into_owned(self) -> Range<'static> {
        Range {
            placement: match self.placement {
                RangePlacement::Numeric(x) => RangePlacement::Numeric(x.map(Series::into_owned)),
                RangePlacement::Bands(categories) => RangePlacement::Bands(categories),
            },
            low: self.low.into_owned(),
            high: self.high.into_owned(),
            body: self
                .body
                .map(|(low, high)| (low.into_owned(), high.into_owned())),
            marker: self.marker.map(Series::into_owned),
            color_by: self.color_by,
            color: self.color,
            label: self.label,
        }
    }
}

impl std::fmt::Debug for Range<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Range")
            .field("intervals", &self.low.len())
            .field("boxed", &self.body.is_some())
            .field("color", &self.color)
            .finish()
    }
}

#[cfg(test)]
#[path = "tests/range_tests.rs"]
mod tests;
