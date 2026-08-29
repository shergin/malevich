//! The scale specification: what an axis means, as a value.

/// The scale of one axis.
///
/// Set with [`crate::Plot::x_scale`] / [`crate::Plot::y_scale`]; the sugar methods
/// (`log_y()`, `time_x()`) are shorthands for the common cases. [`Auto`](Scale::Auto)
/// is the default and adapts to the layers; an explicit scale is always honored.
///
/// # Mark compatibility
///
/// `Auto` resolves the x axis to bands for [`Bars::new`](crate::Bars::new) and
/// [`Range::over`](crate::Range::over), and to linear otherwise. The explicit x-axis
/// contract is:
///
/// | Mark | Linear / Time | Log | Bands |
/// | --- | --- | --- | --- |
/// | Line, Points, Area, numeric Range, Rule, Text | yes | yes; non-positive values are gaps | yes; positions are band indices |
/// | `Bars::new`, `Range::over` | no | no | yes |
/// | `Bars::spans` | yes | yes | no |
/// | `Bars::at` | yes | yes | yes; positions are band indices (grouped bars) |
/// | Cells | yes | yes, with positive extents | yes; grid indices map to bands, no extents |
///
/// On y, Bands positions continuous marks against band indices exactly like x,
/// and maps Cells rows onto the bands top-down — row 0 is the top band, so a
/// labeled matrix reads in matrix order. Bars are rejected on a Bands y axis
/// (their length is numeric). Linear and Time accept every mark. Log accepts Line,
/// Points, Range, Rule, Text, banded Area, and Cells with positive extents; it rejects
/// Bars and zero-baseline Area because zero has no logarithmic position.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[non_exhaustive]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Scale {
    /// Chosen from the layers: categorical when a bars or band-range layer is present,
    /// linear otherwise. The default — and the only scale that infers, so setting any
    /// other value is respected even when a categorical layer is also present.
    #[default]
    Auto,
    /// A continuous linear axis.
    Linear,
    /// Base-10 logarithmic: decade ticks, and values at or below zero become gaps.
    Log,
    /// Unix seconds (UTC): calendar-aligned ticks with multi-scale labels.
    Time,
    /// Named bands — the categorical axis of bar charts, box plots, and violins,
    /// and, on either axis, the labeled rows and columns of a Cells matrix
    /// (confusion matrices, attention maps). Continuous layers position against
    /// band indices (0 is the first band's center; on y, the top band).
    Bands(Vec<String>),
}

impl Scale {
    /// Bands from anything yielding names — sugar for [`Scale::Bands`].
    pub fn bands(categories: impl IntoIterator<Item = impl Into<String>>) -> Scale {
        Scale::Bands(categories.into_iter().map(Into::into).collect())
    }
}
