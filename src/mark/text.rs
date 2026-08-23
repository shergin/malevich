//! The text mark: an annotation anchored at data coordinates.

use crate::render::Color;

/// A text annotation at a data position.
///
/// The text starts at the cell containing the anchor point and extends right,
/// clipping at the plot edge. The anchor extends the axis domains, so an annotation
/// is never silently off-plot.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Text {
    pub(crate) x: f64,
    pub(crate) y: f64,
    pub(crate) text: String,
    pub(crate) color: Option<Color>,
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
