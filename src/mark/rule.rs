//! The rule mark: a reference line across the whole plot.

use super::MarkKind;
use super::line::Dash;
use crate::render::Color;

/// A reference line spanning the plot: horizontal at a y value, or vertical at an
/// x value — or a span, the band between two values on one axis. The zero line, a
/// target, a threshold, a highlighted period — annotations, not data.
///
/// A rule extends the axis domain to include its position, so it is always visible.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) enum Orientation {
    Horizontal(f64),
    Vertical(f64),
    HorizontalSpan(f64, f64),
    VerticalSpan(f64, f64),
}

/// A reference line across the plot area, or a span across it.
///
/// A span ([`Rule::h_span`], [`Rule::v_span`]) washes the band between two
/// values across the whole plot in the rule's color: a recession, a warm-up
/// phase, a tolerance window. On cell targets the wash is a light subpixel
/// texture that marks drawn after it still show through; on pixel targets it
/// is a translucent fill. Layers draw in order, so a span layered first sits
/// behind the data.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Rule {
    pub(crate) orientation: Orientation,
    pub(crate) color: Option<Color>,
    pub(crate) label: Option<String>,
    /// Solid by default; wire documents omit it then.
    #[cfg_attr(
        feature = "serde",
        serde(skip_serializing_if = "Dash::is_solid", default)
    )]
    pub(crate) dash: Dash,
}

impl Rule {
    /// A horizontal rule at `y`, spanning the plot's width.
    ///
    /// # Panics
    ///
    /// Panics if `y` is not finite.
    pub fn h(y: f64) -> Rule {
        let rule = Rule {
            orientation: Orientation::Horizontal(y),
            color: None,
            label: None,
            dash: Dash::Solid,
        };
        rule.validate().expect("Rule::h requires a finite position");
        rule
    }

    /// A vertical rule at `x`, spanning the plot's height.
    ///
    /// # Panics
    ///
    /// Panics if `x` is not finite.
    pub fn v(x: f64) -> Rule {
        let rule = Rule {
            orientation: Orientation::Vertical(x),
            color: None,
            label: None,
            dash: Dash::Solid,
        };
        rule.validate().expect("Rule::v requires a finite position");
        rule
    }

    /// A horizontal span between `y0` and `y1`, across the plot's width — a
    /// tolerance band, a target range.
    ///
    /// # Panics
    ///
    /// Panics if either bound is not finite.
    pub fn h_span(y0: f64, y1: f64) -> Rule {
        let rule = Rule {
            orientation: Orientation::HorizontalSpan(y0, y1),
            color: None,
            label: None,
            dash: Dash::Solid,
        };
        rule.validate()
            .expect("Rule::h_span requires finite bounds");
        rule
    }

    /// A vertical span between `x0` and `x1`, across the plot's height — a
    /// recession, a warm-up phase, an event's duration.
    ///
    /// # Panics
    ///
    /// Panics if either bound is not finite.
    pub fn v_span(x0: f64, x1: f64) -> Rule {
        let rule = Rule {
            orientation: Orientation::VerticalSpan(x0, x1),
            color: None,
            label: None,
            dash: Dash::Solid,
        };
        rule.validate()
            .expect("Rule::v_span requires finite bounds");
        rule
    }

    /// Sets the stroke pattern; [`Dash::Solid`] by default. A dashed or
    /// dotted rule reads as annotation at a glance — a target, not data.
    /// A span has no stroke and ignores it.
    #[must_use]
    pub fn dash(mut self, dash: Dash) -> Rule {
        self.dash = dash;
        self
    }

    /// Sets an explicit color; without one, rules draw in the default foreground —
    /// annotations should recede, not compete.
    #[must_use]
    pub fn color(mut self, color: Color) -> Rule {
        self.color = Some(color);
        self
    }

    /// Names this rule in the legend.
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Rule {
        self.label = Some(label.into());
        self
    }
}

impl MarkKind for Rule {
    type Owned = Rule;

    /// Checks the rule position after any construction path.
    fn validate(&self) -> crate::Result<()> {
        let finite = match self.orientation {
            Orientation::Horizontal(value) | Orientation::Vertical(value) => value.is_finite(),
            Orientation::HorizontalSpan(a, b) | Orientation::VerticalSpan(a, b) => {
                a.is_finite() && b.is_finite()
            }
        };
        if finite {
            Ok(())
        } else {
            Err(crate::Error::InvalidParameter {
                detail: "a Rule position must be finite",
            })
        }
    }

    fn into_owned(self) -> Rule {
        self
    }
}

#[cfg(test)]
#[path = "tests/rule_tests.rs"]
mod tests;
