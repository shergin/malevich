//! Colormaps: continuous value-to-color scales for gridded marks.

use std::borrow::Cow;

use crate::render::Color;

/// A midpoint compared by bit pattern, so the spec types that hold a colormap
/// keep the `Eq` they promise. Construction rejects non-finite values; a
/// deserialized non-finite midpoint degrades to a linear map and is caught by
/// spec validation.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(transparent)
)]
struct Midpoint(f64);

impl PartialEq for Midpoint {
    fn eq(&self, other: &Midpoint) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for Midpoint {}

/// A continuous colormap: linear interpolation through RGB stops, optionally
/// centered on a data midpoint.
///
/// The named constants are a curated set that stays distinguishable down the
/// whole color ladder (truecolor → 256 → 16 → plain shade): sequential
/// [`VIRIDIS`](Colormap::VIRIDIS) (the default), [`MAGMA`](Colormap::MAGMA),
/// [`CIVIDIS`](Colormap::CIVIDIS), and [`GREYS`](Colormap::GREYS); diverging
/// [`RED_BLUE`](Colormap::RED_BLUE) and
/// [`PURPLE_ORANGE`](Colormap::PURPLE_ORANGE), whose ends are named in
/// low-to-high order. Any custom map is just a list of stops.
///
/// Diverging maps encode signed or centered data honestly only when anchored:
/// [`centered_at`](Colormap::centered_at) pins a data value (0 for
/// correlations, 1 for ratios) to the map's middle and spans the larger side
/// symmetrically, so equal magnitudes on either side get equal intensity.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Colormap {
    stops: Cow<'static, [(u8, u8, u8)]>,
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Option::is_none")
    )]
    midpoint: Option<Midpoint>,
}

impl Colormap {
    /// The default sequential map: [`VIRIDIS`](Colormap::VIRIDIS).
    pub const DEFAULT: Colormap = Colormap::VIRIDIS;

    /// A viridis approximation — perceptually ordered, colorblind-safe,
    /// readable on dark and light backgrounds.
    pub const VIRIDIS: Colormap = Colormap::new(&[
        (68, 1, 84),
        (59, 82, 139),
        (33, 145, 140),
        (94, 201, 98),
        (253, 231, 37),
    ]);

    /// A magma approximation — perceptually ordered, near-black to pale yellow.
    pub const MAGMA: Colormap = Colormap::new(&[
        (0, 0, 4),
        (81, 18, 124),
        (183, 55, 121),
        (252, 137, 97),
        (252, 253, 191),
    ]);

    /// A cividis approximation — perceptually ordered, optimized for
    /// red-green color vision deficiency.
    pub const CIVIDIS: Colormap = Colormap::new(&[
        (0, 32, 77),
        (65, 77, 107),
        (124, 123, 120),
        (188, 175, 111),
        (255, 233, 69),
    ]);

    /// A plain grey ramp, dim to bright.
    pub const GREYS: Colormap = Colormap::new(&[(64, 64, 64), (250, 250, 250)]);

    /// Diverging red → neutral → blue (ColorBrewer RdBu). Anchor it with
    /// [`centered_at`](Colormap::centered_at).
    pub const RED_BLUE: Colormap = Colormap::new(&[
        (202, 0, 32),
        (244, 165, 130),
        (247, 247, 247),
        (146, 197, 222),
        (5, 113, 176),
    ]);

    /// Diverging purple → neutral → orange (ColorBrewer PuOr, low end purple).
    /// Anchor it with [`centered_at`](Colormap::centered_at).
    pub const PURPLE_ORANGE: Colormap = Colormap::new(&[
        (94, 60, 153),
        (178, 171, 210),
        (247, 247, 247),
        (253, 184, 99),
        (230, 97, 1),
    ]);

    /// The canonical names [`named`](Colormap::named) resolves, for help text
    /// and option listings.
    pub const NAMES: [&'static str; 6] = [
        "viridis",
        "magma",
        "cividis",
        "greys",
        "red-blue",
        "purple-orange",
    ];

    /// Looks up a named built-in map (see [`NAMES`](Colormap::NAMES);
    /// `"grays"` is accepted for `"greys"`). Diverging maps come back
    /// unanchored — apply [`centered_at`](Colormap::centered_at) to center
    /// them on a data value.
    pub fn named(name: &str) -> Option<Colormap> {
        match name {
            "viridis" => Some(Colormap::VIRIDIS),
            "magma" => Some(Colormap::MAGMA),
            "cividis" => Some(Colormap::CIVIDIS),
            "greys" | "grays" => Some(Colormap::GREYS),
            "red-blue" => Some(Colormap::RED_BLUE),
            "purple-orange" => Some(Colormap::PURPLE_ORANGE),
            _ => None,
        }
    }

    /// A custom colormap over evenly spaced RGB stops.
    ///
    /// # Panics
    ///
    /// Panics with fewer than two stops.
    pub const fn new(stops: &'static [(u8, u8, u8)]) -> Colormap {
        assert!(
            stops.len() >= 2,
            "Colormap::new requires at least two stops"
        );
        Colormap {
            stops: Cow::Borrowed(stops),
            midpoint: None,
        }
    }

    /// Builds a colormap from runtime-owned, evenly spaced RGB stops.
    ///
    /// The vector is retained without copying, so generated palettes and palettes
    /// loaded from configuration do not need to be leaked into `'static` storage.
    ///
    /// # Errors
    ///
    /// Returns [`Error::EmptyDimension`](crate::Error::EmptyDimension) when fewer
    /// than two stops are supplied.
    pub fn try_from_stops(stops: Vec<(u8, u8, u8)>) -> crate::Result<Colormap> {
        if stops.len() < 2 {
            return Err(crate::Error::EmptyDimension {
                what: "Colormap stops",
            });
        }
        Ok(Colormap {
            stops: Cow::Owned(stops),
            midpoint: None,
        })
    }

    /// Centers the map on a data value: `midpoint` maps to ramp position 0.5,
    /// and the value range spans the larger side symmetrically, so equal
    /// magnitudes on either side of the midpoint get equal intensity. The
    /// colorbar shows the symmetric range.
    ///
    /// # Panics
    ///
    /// Panics when `midpoint` is not finite.
    #[must_use]
    pub fn centered_at(mut self, midpoint: f64) -> Colormap {
        assert!(
            midpoint.is_finite(),
            "Colormap::centered_at requires a finite midpoint"
        );
        self.midpoint = Some(Midpoint(midpoint));
        self
    }

    /// The centered data value, when this map has one.
    pub fn midpoint(&self) -> Option<f64> {
        self.midpoint.map(|midpoint| midpoint.0)
    }

    /// The evenly spaced RGB stops, from the low end to the high end.
    pub fn stops(&self) -> &[(u8, u8, u8)] {
        &self.stops
    }

    /// Checks invariants after any construction path.
    pub(crate) fn validate(&self) -> crate::Result<()> {
        if self.stops.len() < 2 {
            return Err(crate::Error::EmptyDimension {
                what: "Colormap stops",
            });
        }
        if self
            .midpoint
            .is_some_and(|midpoint| !midpoint.0.is_finite())
        {
            return Err(crate::Error::InvalidParameter {
                detail: "a colormap midpoint must be finite",
            });
        }
        Ok(())
    }

    /// The centered midpoint when it is usable; a non-finite one (possible only
    /// through deserialization) degrades to a linear map.
    fn active_midpoint(&self) -> Option<f64> {
        self.midpoint().filter(|midpoint| midpoint.is_finite())
    }

    /// The value range the ramp displays for data observed in `[low, high]`:
    /// the range itself for a linear map, the symmetric span around the
    /// midpoint for a centered one.
    pub(crate) fn display_domain(&self, low: f64, high: f64) -> (f64, f64) {
        match self.active_midpoint() {
            Some(midpoint) => {
                let half = (high - midpoint).max(midpoint - low);
                let half = if half > 0.0 { half } else { 1.0 };
                (midpoint - half, midpoint + half)
            }
            None => (low, high),
        }
    }

    /// The ramp position in `[0, 1]` for `value` among data observed in
    /// `[low, high]` — linear across the range, or centered per
    /// [`centered_at`](Colormap::centered_at). `NaN` maps to the low end,
    /// matching [`color`](Colormap::color).
    pub fn position_in(&self, value: f64, low: f64, high: f64) -> f64 {
        let (start, end) = self.display_domain(low, high);
        let position = if end > start {
            crate::numeric::inverse_lerp(start, end, value)
        } else {
            0.0
        };
        if position.is_finite() {
            position.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// The color at `position` in `[0, 1]` (clamped; `NaN` maps to the low end).
    ///
    /// A colormap built through [`Colormap::new`] always has at least two stops;
    /// one deserialized with too few degrades gracefully rather than panicking.
    pub fn color(&self, position: f64) -> Color {
        match self.stops.len() {
            0 => return Color::Default,
            1 => {
                let (r, g, b) = self.stops[0];
                return Color::Rgb(r, g, b);
            }
            _ => {}
        }
        let position = if position.is_finite() {
            position.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let scaled = position * (self.stops.len() - 1) as f64;
        let index = (scaled as usize).min(self.stops.len() - 2);
        let t = scaled - index as f64;
        let (r0, g0, b0) = self.stops[index];
        let (r1, g1, b1) = self.stops[index + 1];
        let lerp = |a: u8, b: u8| (f64::from(a) + (f64::from(b) - f64::from(a)) * t) as u8;
        Color::Rgb(lerp(r0, r1), lerp(g0, g1), lerp(b0, b1))
    }
}

impl Default for Colormap {
    fn default() -> Colormap {
        Colormap::DEFAULT
    }
}

#[cfg(test)]
#[path = "tests/colormap_tests.rs"]
mod tests;
