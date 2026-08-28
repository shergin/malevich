//! Marks: the geometric primitives that draw data.
//!
//! A mark binds channels (data) to a drawing rule; it holds no scales, no layout, and
//! no terminal state. Marks are layered onto a [`crate::Plot`], which resolves shared
//! scales across all layers and rasterizes. [`Mark`] is the closed set of them —
//! chart types compose marks, they never extend the set.

mod area;
mod bars;
mod categories;
mod cells;
mod line;
mod points;
mod range;
mod rule;
mod text;

pub use area::Area;
pub use bars::Bars;
pub(crate) use bars::Placement;
pub(crate) use categories::Categories;
pub use cells::Cells;
pub(crate) use line::Source;
pub use line::{Dash, Line, LineStyle};
pub use points::{PointStyle, Points};
pub use range::Range;
pub(crate) use range::RangePlacement;
pub(crate) use rule::Orientation;
pub use rule::Rule;
pub use text::Text;

/// Any mark, ready to be layered onto a plot.
///
/// Constructed via `From` — `plot.layer(Line::y(&data))` works directly.
#[non_exhaustive]
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Mark<'a> {
    /// A polyline through ordered points.
    Line(Line<'a>),
    /// Unconnected point markers.
    Points(Points<'a>),
    /// Filled columns over a categorical axis.
    Bars(Bars<'a>),
    /// A filled region between two edges.
    Area(Area<'a>),
    /// A value grid drawn as shaded, colored cells.
    Cells(Cells<'a>),
    /// Vertical intervals: error bars, boxes, event ticks.
    Range(Range<'a>),
    /// A reference line across the plot.
    Rule(Rule),
    /// A text annotation at data coordinates.
    Text(Text),
}

impl<'a> Mark<'a> {
    /// Detaches from any borrowed storage, making the mark `'static`.
    pub fn into_owned(self) -> Mark<'static> {
        match self {
            Mark::Line(line) => Mark::Line(line.into_owned()),
            Mark::Points(points) => Mark::Points(points.into_owned()),
            Mark::Bars(bars) => Mark::Bars(bars.into_owned()),
            Mark::Area(area) => Mark::Area(area.into_owned()),
            Mark::Cells(cells) => Mark::Cells(cells.into_owned()),
            Mark::Range(range) => Mark::Range(range.into_owned()),
            Mark::Rule(rule) => Mark::Rule(rule),
            Mark::Text(text) => Mark::Text(text),
        }
    }

    /// Checks this mark's channel invariants, returning the first violation.
    ///
    /// The constructors enforce these already; this re-checks a mark that arrived
    /// another way (deserialization) so the fallible API can report bad specs.
    pub(crate) fn validate(&self) -> Result<(), crate::Error> {
        match self {
            Mark::Line(mark) => mark.validate(),
            Mark::Points(mark) => mark.validate(),
            Mark::Bars(mark) => mark.validate(),
            Mark::Area(mark) => mark.validate(),
            Mark::Cells(mark) => mark.validate(),
            Mark::Range(mark) => mark.validate(),
            Mark::Rule(mark) => mark.validate(),
            Mark::Text(mark) => mark.validate(),
        }
    }
}

/// Errors unless the two channel lengths match.
pub(crate) fn pair(mark: &'static str, a: usize, b: usize) -> Result<(), crate::Error> {
    if a == b {
        Ok(())
    } else {
        Err(crate::Error::UnequalChannels {
            mark,
            lengths: (a, b),
        })
    }
}

impl<'a> From<Line<'a>> for Mark<'a> {
    fn from(line: Line<'a>) -> Mark<'a> {
        Mark::Line(line)
    }
}

impl<'a> From<Points<'a>> for Mark<'a> {
    fn from(points: Points<'a>) -> Mark<'a> {
        Mark::Points(points)
    }
}

impl<'a> From<Bars<'a>> for Mark<'a> {
    fn from(bars: Bars<'a>) -> Mark<'a> {
        Mark::Bars(bars)
    }
}

impl<'a> From<Area<'a>> for Mark<'a> {
    fn from(area: Area<'a>) -> Mark<'a> {
        Mark::Area(area)
    }
}

impl<'a> From<Rule> for Mark<'a> {
    fn from(rule: Rule) -> Mark<'a> {
        Mark::Rule(rule)
    }
}

impl<'a> From<Text> for Mark<'a> {
    fn from(text: Text) -> Mark<'a> {
        Mark::Text(text)
    }
}

impl<'a> From<Cells<'a>> for Mark<'a> {
    fn from(cells: Cells<'a>) -> Mark<'a> {
        Mark::Cells(cells)
    }
}

impl<'a> From<Range<'a>> for Mark<'a> {
    fn from(range: Range<'a>) -> Mark<'a> {
        Mark::Range(range)
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn payload_validation_owns_scalar_constructor_invariants() {
        let mut bars = Bars::spans(0.0, 1.0, [1.0]);
        bars.placement = Placement::Spans {
            start: 0.0,
            width: 0.0,
        };
        assert!(matches!(
            bars.validate(),
            Err(crate::Error::InvalidParameter { .. })
        ));

        let rule = Rule {
            orientation: Orientation::Horizontal(f64::NAN),
            color: None,
            label: None,
            dash: Dash::Solid,
        };
        assert!(matches!(
            rule.validate(),
            Err(crate::Error::InvalidParameter { .. })
        ));

        let text = Text {
            x: f64::INFINITY,
            y: 0.0,
            text: String::new(),
            color: None,
        };
        assert!(matches!(
            text.validate(),
            Err(crate::Error::InvalidParameter { .. })
        ));

        let mut line = Line::function(0.0..1.0, f64::sin);
        let Source::Function { domain, .. } = &mut line.source else {
            unreachable!("the test constructed a function line")
        };
        *domain = (1.0, 1.0);
        assert!(matches!(
            line.validate(),
            Err(crate::Error::InvalidParameter { .. })
        ));
    }
}
