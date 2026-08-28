//! The line mark: a polyline through ordered points.

use std::ops::Range;
use std::sync::Arc;

use crate::data::{IntoSeries, Series};
use crate::mark::Categories;
use crate::render::Color;
use crate::scale::Colormap;

/// How a line renders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LineStyle {
    /// Through the charset's subpixels — braille dots, octant ink. The default.
    #[default]
    Pixels,
    /// Whole-cell box-drawing corners (`╭╮╰╯│─`) — the classic asciichart look:
    /// one cell per column, smooth elbows, instantly legible at low resolution.
    /// ASCII charsets draw `/`-free equivalents (`+`, `-`, `|`).
    Corners,
}

/// The stroke pattern along a path: solid ink, dashes, or dots. The
/// pattern's phase runs continuously along a polyline, so it flows through
/// joints instead of restarting at every segment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Dash {
    /// Unbroken ink. The default.
    #[default]
    Solid,
    /// Even dashes with gaps — reference lines, targets, baselines.
    Dashed,
    /// Point-length dashes; a round pen draws literal dots.
    Dotted,
}

impl Dash {
    /// The serde skip guard: solid is the default and stays off the wire.
    #[cfg(feature = "serde")]
    pub(crate) fn is_solid(&self) -> bool {
        *self == Dash::Solid
    }
}

/// A polyline through ordered points; gaps (`NaN`) break it visibly.
///
/// Data enters three ways: y values against their indices ([`Line::y`]), paired
/// series ([`Line::xy`]), or a function sampled at raster resolution
/// ([`Line::function`]).
#[derive(Clone)]
pub struct Line<'a> {
    pub(crate) source: Source<'a>,
    pub(crate) color: Option<Color>,
    pub(crate) label: Option<String>,
    pub(crate) style: LineStyle,
    pub(crate) color_by: Option<Categories>,
    pub(crate) glow: bool,
    pub(crate) dash: Dash,
    pub(crate) grade: Option<(Series<'a>, Colormap)>,
}

#[derive(Clone)]
pub(crate) enum Source<'a> {
    Points {
        x: Option<Series<'a>>,
        y: Series<'a>,
    },
    Function {
        domain: (f64, f64),
        function: Arc<dyn Fn(f64) -> f64 + Send + Sync>,
    },
}

impl<'a> Line<'a> {
    /// A line through `values` plotted against their indices `0, 1, 2, …`.
    pub fn y(values: impl IntoSeries<'a>) -> Line<'a> {
        Line {
            source: Source::Points {
                x: None,
                y: values.into_series(),
            },
            color: None,
            label: None,
            style: LineStyle::Pixels,
            color_by: None,
            glow: false,
            dash: Dash::Solid,
            grade: None,
        }
    }

    /// A line through the points `(x[i], y[i])`.
    ///
    /// # Panics
    ///
    /// Panics if the two series have different lengths.
    pub fn xy(x: impl IntoSeries<'a>, y: impl IntoSeries<'a>) -> Line<'a> {
        let x = x.into_series();
        let y = y.into_series();
        let line = Line {
            source: Source::Points { x: Some(x), y },
            color: None,
            label: None,
            style: LineStyle::Pixels,
            color_by: None,
            glow: false,
            dash: Dash::Solid,
            grade: None,
        };
        line.validate()
            .expect("Line::xy requires series of equal length");
        line
    }

    /// A line through `function`, sampled once per subpixel column over `domain`.
    ///
    /// Sampling at raster resolution means the drawn curve is as smooth as the
    /// surface can express, regardless of the domain's size.
    ///
    /// # Panics
    ///
    /// Panics if the domain is not finite or is empty.
    pub fn function(
        domain: Range<f64>,
        function: impl Fn(f64) -> f64 + Send + Sync + 'static,
    ) -> Line<'a> {
        let line = Line {
            source: Source::Function {
                domain: (domain.start, domain.end),
                function: Arc::new(function),
            },
            color: None,
            label: None,
            style: LineStyle::Pixels,
            color_by: None,
            glow: false,
            dash: Dash::Solid,
            grade: None,
        };
        line.validate()
            .expect("Line::function requires a finite, non-empty domain");
        line
    }

    /// The retained point channels — `(x, y)`, with `x` `None` for
    /// index-positioned values — or `None` for a function-backed line.
    /// In-crate presentation (the widget's snap readout) reads data here.
    #[cfg(feature = "ratatui")]
    pub(crate) fn channels(&self) -> Option<(Option<&Series<'a>>, &Series<'a>)> {
        match &self.source {
            Source::Points { x, y } => Some((x.as_ref(), y)),
            Source::Function { .. } => None,
        }
    }

    /// Sets the rendering style; [`LineStyle::Pixels`] by default.
    #[must_use]
    pub fn style(mut self, style: LineStyle) -> Line<'a> {
        self.style = style;
        self
    }

    /// Sets an explicit color; without one, layers take colors from the palette.
    #[must_use]
    pub fn color(mut self, color: Color) -> Line<'a> {
        self.color = Some(color);
        self
    }

    /// Draws a soft halo around the stroke on pixel targets — a wide
    /// under-stroke in the line's own hue at low intensity. Cell targets
    /// ignore it.
    #[must_use]
    pub fn glow(mut self) -> Line<'a> {
        self.glow = true;
        self
    }

    /// Grades the stroke through `colormap` by per-point `values`: each
    /// point takes the color of its value within the values' finite range —
    /// a trajectory that shows progression along a third variable. Grading
    /// draws every point (column reduction would drop the values), so it
    /// suits trajectories rather than million-point series. Replaces the
    /// constant color; the legend swatch wears the final point's color.
    ///
    /// # Panics
    ///
    /// Panics if the number of values differs from the number of points,
    /// if the line is function-backed, or combined with
    /// [`color_by`](Line::color_by).
    #[must_use]
    pub fn grade(mut self, values: impl IntoSeries<'a>, colormap: Colormap) -> Line<'a> {
        self.grade = Some((values.into_series(), colormap));
        self.validate()
            .expect("Line::grade requires one value per point on a point-backed line");
        self
    }

    /// Sets the stroke pattern; [`Dash::Solid`] by default.
    #[must_use]
    pub fn dash(mut self, dash: Dash) -> Line<'a> {
        self.dash = dash;
        self
    }

    /// Names this layer in the legend. The legend appears once any layer is
    /// labeled (and the frame is tall enough for it).
    #[must_use]
    pub fn label(mut self, label: impl Into<String>) -> Line<'a> {
        self.label = Some(label.into());
        self
    }

    /// Colors the line by per-point categories: each category's run draws in
    /// its palette color as a separate segment (breaks between categories are
    /// honest gaps), the legend names the categories, and colorless output
    /// keeps them apart by segmentation. Replaces the constant color and
    /// layer label.
    ///
    /// # Panics
    ///
    /// Panics if the number of categories differs from the number of points,
    /// or if the line is function-backed (a sampled function has no
    /// per-point categories).
    #[must_use]
    pub fn color_by(mut self, categories: impl IntoIterator<Item = impl Into<String>>) -> Line<'a> {
        self.color_by = Some(Categories::new(categories));
        self.validate()
            .expect("Line::color_by requires one category per point and point data");
        self
    }

    /// Checks source and channel invariants, including deserialized values.
    pub(crate) fn validate(&self) -> crate::Result<()> {
        match &self.source {
            Source::Points { x, y } => {
                if let Some(x) = x {
                    super::pair("Line: x and y", x.len(), y.len())?;
                }
                if let Some(categories) = &self.color_by {
                    super::pair("Line: color_by and y", categories.len(), y.len())?;
                }
                if let Some((values, colormap)) = &self.grade {
                    super::pair("Line: grade and y", values.len(), y.len())?;
                    colormap.validate()?;
                    if self.color_by.is_some() {
                        return Err(crate::Error::InvalidParameter {
                            detail: "a Line cannot be both graded and colored by category",
                        });
                    }
                }
            }
            Source::Function { domain, .. } => {
                if !(domain.0.is_finite() && domain.1.is_finite() && domain.0 < domain.1) {
                    return Err(crate::Error::InvalidParameter {
                        detail: "a function Line needs a finite non-empty domain",
                    });
                }
                if self.color_by.is_some() {
                    return Err(crate::Error::InvalidParameter {
                        detail: "a function Line cannot have a color_by channel",
                    });
                }
                if self.grade.is_some() {
                    return Err(crate::Error::InvalidParameter {
                        detail: "a function Line cannot be graded (it has no per-point values)",
                    });
                }
            }
        }
        Ok(())
    }

    /// Detaches from any borrowed storage, making the mark `'static`.
    pub fn into_owned(self) -> Line<'static> {
        Line {
            source: match self.source {
                Source::Points { x, y } => Source::Points {
                    x: x.map(Series::into_owned),
                    y: y.into_owned(),
                },
                Source::Function { domain, function } => Source::Function { domain, function },
            },
            color: self.color,
            label: self.label,
            style: self.style,
            color_by: self.color_by,
            glow: self.glow,
            dash: self.dash,
            grade: self
                .grade
                .map(|(values, colormap)| (values.into_owned(), colormap)),
        }
    }
}

impl std::fmt::Debug for Line<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Line");
        match &self.source {
            Source::Points { x, y } => {
                debug.field("points", &y.len());
                debug.field("indexed", &x.is_none());
            }
            Source::Function { domain, .. } => {
                debug.field("function_over", domain);
            }
        }
        debug.field("color", &self.color).finish()
    }
}

/// With the `serde` feature, point-backed lines round-trip; a function-backed
/// line refuses to serialize (a closure has no data representation) — sample it
/// into points first.
#[cfg(feature = "serde")]
mod serde_impls {
    use serde::ser::Error as _;

    use super::*;

    #[derive(serde::Serialize)]
    struct Repr<'s> {
        x: Option<&'s Series<'s>>,
        y: &'s Series<'s>,
        color: &'s Option<Color>,
        label: &'s Option<String>,
        style: LineStyle,
        #[serde(skip_serializing_if = "Option::is_none")]
        color_by: &'s Option<Categories>,
        /// Absent when off, keeping v1 wire documents byte-stable.
        #[serde(skip_serializing_if = "is_false")]
        glow: bool,
        #[serde(skip_serializing_if = "Dash::is_solid")]
        dash: Dash,
        #[serde(skip_serializing_if = "Option::is_none")]
        grade: &'s Option<(Series<'s>, Colormap)>,
    }

    fn is_false(value: &bool) -> bool {
        !*value
    }

    #[derive(serde::Deserialize)]
    struct OwnedRepr {
        x: Option<Series<'static>>,
        y: Series<'static>,
        color: Option<Color>,
        label: Option<String>,
        style: LineStyle,
        #[serde(default)]
        color_by: Option<Categories>,
        #[serde(default)]
        glow: bool,
        #[serde(default)]
        dash: Dash,
        #[serde(default)]
        grade: Option<(Series<'static>, Colormap)>,
    }

    impl serde::Serialize for Line<'_> {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            match &self.source {
                Source::Points { x, y } => Repr {
                    x: x.as_ref(),
                    y,
                    color: &self.color,
                    label: &self.label,
                    style: self.style,
                    color_by: &self.color_by,
                    glow: self.glow,
                    dash: self.dash,
                    grade: &self.grade,
                }
                .serialize(serializer),
                Source::Function { .. } => Err(S::Error::custom(
                    "a function-backed Line cannot be serialized; sample it into points first",
                )),
            }
        }
    }

    impl<'de, 'a> serde::Deserialize<'de> for Line<'a> {
        fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            let repr = OwnedRepr::deserialize(deserializer)?;
            Ok(Line {
                source: Source::Points {
                    x: repr.x,
                    y: repr.y,
                },
                color: repr.color,
                label: repr.label,
                style: repr.style,
                color_by: repr.color_by,
                glow: repr.glow,
                dash: repr.dash,
                grade: repr.grade,
            })
        }
    }
}

#[cfg(test)]
#[path = "tests/line_tests.rs"]
mod tests;
