//! The text mark: an annotation anchored at data coordinates.

use crate::render::Color;

/// How a [`Text`] annotation sits relative to its anchor.
///
/// [`Left`](Align::Left) is the default: the text starts at the cell containing
/// the anchor and extends right, clipping at the plot edge. [`Center`](Align::Center)
/// straddles the anchor and [`Right`](Align::Right) ends at it — except on a
/// [`Bands`](crate::Scale::Bands) x axis, where the band nearest the anchor is the
/// box, with exactly the geometry the band's own label uses (its rounded center,
/// its step-wide budget), so aligned text and band labels land in lockstep. Text
/// wider than the box clips to it, ending with a truncation `.` — digits from a
/// neighboring column are never mixed into a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Align {
    /// Start at the anchor and extend right — the classic annotation.
    #[default]
    Left,
    /// Straddle the anchor; within its band on a bands axis.
    Center,
    /// End at the anchor; against its band's right edge on a bands axis.
    Right,
}

/// A text annotation at a data position.
///
/// The text starts at the cell containing the anchor point and extends right,
/// clipping at the plot edge. The anchor extends the axis domains, so an annotation
/// is never silently off-plot. [`Text::align`] repositions the text relative to the
/// anchor — the channel table cells and annotated heatmaps are built from.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Text {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) text: String,
    pub(crate) color: Option<Color>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "align_is_default")
    )]
    pub(crate) align: Align,
}

#[cfg(feature = "serde")]
#[allow(clippy::trivially_copy_pass_by_ref)]
fn align_is_default(align: &Align) -> bool {
    *align == Align::Left
}

impl Text {
    /// An annotation anchored at `(x, y)`.
    ///
    /// # Panics
    ///
    /// Panics if the anchor is not finite.
    pub fn at(x: f64, y: f64, text: impl Into<String>) -> Text {
        let text = Text {
            x,
            y,
            text: text.into(),
            color: None,
            align: Align::Left,
        };
        text.validate().expect("Text::at requires a finite anchor");
        text
    }

    /// Sets an explicit color; without one, annotations draw in the default
    /// foreground.
    #[must_use]
    pub fn color(mut self, color: Color) -> Text {
        self.color = Some(color);
        self
    }

    /// Sets the alignment; the default is [`Align::Left`], today's
    /// start-at-the-anchor behavior.
    #[must_use]
    pub fn align(mut self, align: Align) -> Text {
        self.align = align;
        self
    }

    /// Checks the annotation anchor after any construction path.
    pub(crate) fn validate(&self) -> crate::Result<()> {
        if self.x.is_finite() && self.y.is_finite() {
            Ok(())
        } else {
            Err(crate::Error::InvalidParameter {
                detail: "a Text anchor must be finite",
            })
        }
    }
}
