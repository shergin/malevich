//! The categorical color scale: distinct colors for distinct categories.

use std::borrow::Cow;

use crate::render::Color;

/// A categorical color scale: the colors a `color_by` channel cycles through,
/// in category order (first appearance first).
///
/// The default is [`OKABE_ITO`](Palette::OKABE_ITO) — the Okabe–Ito palette
/// (Wong 2011) without print-black: seven colors distinguishable under the
/// common color-vision deficiencies, on dark and light backgrounds. More
/// categories than colors wrap around; in plain output categories separate by
/// marker shape instead, so the wrap never hides a group.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Palette {
    colors: Cow<'static, [Color]>,
}

impl Palette {
    /// Okabe–Ito (Wong 2011), print-black omitted: orange, sky blue, bluish
    /// green, vermillion, reddish purple, blue, yellow. The default.
    pub const OKABE_ITO: Palette = Palette::new(&[
        Color::Rgb(230, 159, 0),
        Color::Rgb(86, 180, 233),
        Color::Rgb(0, 158, 115),
        Color::Rgb(213, 94, 0),
        Color::Rgb(204, 121, 167),
        Color::Rgb(0, 114, 178),
        Color::Rgb(240, 228, 66),
    ]);

    /// A custom palette over a static color list.
    ///
    /// # Panics
    ///
    /// Panics with an empty list.
    pub const fn new(colors: &'static [Color]) -> Palette {
        assert!(
            !colors.is_empty(),
            "Palette::new requires at least one color"
        );
        Palette {
            colors: Cow::Borrowed(colors),
        }
    }

    /// Builds a palette from runtime-owned colors, without copying.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyDimension`](crate::Error::EmptyDimension) for an
    /// empty list.
    pub fn try_from_colors(colors: Vec<Color>) -> crate::Result<Palette> {
        if colors.is_empty() {
            return Err(crate::Error::EmptyDimension {
                what: "Palette colors",
            });
        }
        Ok(Palette {
            colors: Cow::Owned(colors),
        })
    }

    /// The colors, in cycle order.
    pub fn colors(&self) -> &[Color] {
        &self.colors
    }

    /// Checks invariants after any construction path.
    pub(crate) fn validate(&self) -> crate::Result<()> {
        if self.colors.is_empty() {
            Err(crate::Error::EmptyDimension {
                what: "Palette colors",
            })
        } else {
            Ok(())
        }
    }

    /// The color for category `index`, wrapping past the end. An empty palette
    /// (possible only through deserialization) degrades to the default color.
    pub(crate) fn color(&self, index: usize) -> Color {
        match self.colors.len() {
            0 => Color::Default,
            len => self.colors[index % len],
        }
    }
}

impl Default for Palette {
    fn default() -> Palette {
        Palette::OKABE_ITO
    }
}

#[cfg(test)]
#[path = "tests/palette_tests.rs"]
mod tests;
