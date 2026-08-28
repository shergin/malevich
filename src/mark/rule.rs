//! The rule mark: a reference line across the whole plot.

use super::line::Dash;
use crate::render::Color;

/// A reference line spanning the plot: horizontal at a y value, or vertical at an
/// x value. The zero line, a target, a threshold — annotations, not data.
///
/// A rule extends the axis domain to include its position, so it is always visible.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) enum Orientation {
    Horizontal(f64),
    Vertical(f64),
}

/// A reference line across the plot area.
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

    /// Sets the stroke pattern; [`Dash::Solid`] by default. A dashed or
    /// dotted rule reads as annotation at a glance — a target, not data.
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

    /// Checks the rule position after any construction path.
    pub(crate) fn validate(&self) -> crate::Result<()> {
        let position = match self.orientation {
            Orientation::Horizontal(value) | Orientation::Vertical(value) => value,
        };
        if position.is_finite() {
            Ok(())
        } else {
            Err(crate::Error::InvalidParameter {
                detail: "a Rule position must be finite",
            })
        }
    }
}

#[cfg(test)]
#[path = "tests/rule_tests.rs"]
mod tests;
