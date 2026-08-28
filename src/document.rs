//! Versioned, validated persistence for serde-enabled plot and grid specs.

use crate::{Frame, Grid, Plot};

/// The payload carried by a [`Document`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DocumentKind {
    /// One retained plot.
    Plot,
    /// A row-major grid of plots.
    Grid,
}

/// A versioned persistent plot document.
///
/// Raw [`Plot`] and [`Grid`] values continue to implement serde for compatibility,
/// but they have no schema discriminator. New stored or transmitted specs should
/// use this envelope, whose JSON shape is `{ "version": 1, "kind": "plot" |
/// "grid", "spec": ... }`. Deserialization rejects unknown versions and validates
/// the decoded payload before returning it.
///
/// See the repository's `docs/serde.md` for the compatibility policy and fixtures.
#[derive(Debug, Clone)]
pub struct Document {
    content: Content,
}

#[derive(Debug, Clone)]
enum Content {
    Plot(Plot<'static>),
    Grid(Grid<'static>),
}

impl Document {
    /// The current persistent schema version.
    pub const VERSION: u32 = 1;

    /// Wraps a valid plot in a versioned document and owns all borrowed data.
    pub fn plot(plot: Plot<'_>) -> crate::Result<Document> {
        plot.validate()?;
        Ok(Document {
            content: Content::Plot(plot.into_owned()),
        })
    }

    /// Wraps a valid grid in a versioned document and owns all borrowed data.
    pub fn grid(grid: Grid<'_>) -> crate::Result<Document> {
        grid.validate()?;
        Ok(Document {
            content: Content::Grid(grid.into_owned()),
        })
    }

    /// The schema version this document serializes as.
    pub const fn version(&self) -> u32 {
        Self::VERSION
    }

    /// Whether the payload is a plot or a grid.
    pub const fn kind(&self) -> DocumentKind {
        match self.content {
            Content::Plot(_) => DocumentKind::Plot,
            Content::Grid(_) => DocumentKind::Grid,
        }
    }

    /// The plot payload, or `None` for a grid document.
    pub fn as_plot(&self) -> Option<&Plot<'static>> {
        match &self.content {
            Content::Plot(plot) => Some(plot),
            Content::Grid(_) => None,
        }
    }

    /// The grid payload, or `None` for a plot document.
    pub fn as_grid(&self) -> Option<&Grid<'static>> {
        match &self.content {
            Content::Grid(grid) => Some(grid),
            Content::Plot(_) => None,
        }
    }

    /// Re-checks the payload's semantic invariants.
    pub fn validate(&self) -> crate::Result<()> {
        match &self.content {
            Content::Plot(plot) => plot.validate(),
            Content::Grid(grid) => grid.validate(),
        }
    }

    /// Renders the payload, degrading to an empty string on a frame-limit error.
    pub fn render(&self, frame: &Frame) -> String {
        match &self.content {
            Content::Plot(plot) => plot.render(frame),
            Content::Grid(grid) => grid.render(frame),
        }
    }

    /// Validates and renders the payload through its fallible boundary.
    pub fn try_render(&self, frame: &Frame) -> crate::Result<String> {
        match &self.content {
            Content::Plot(plot) => plot.try_render(frame),
            Content::Grid(grid) => grid.try_render(frame),
        }
    }
}

impl<'a> TryFrom<Plot<'a>> for Document {
    type Error = crate::Error;

    fn try_from(plot: Plot<'a>) -> Result<Self, Self::Error> {
        Document::plot(plot)
    }
}

impl<'a> TryFrom<Grid<'a>> for Document {
    type Error = crate::Error;

    fn try_from(grid: Grid<'a>) -> Result<Self, Self::Error> {
        Document::grid(grid)
    }
}

#[derive(serde::Serialize)]
struct EnvelopeRef<'a> {
    version: u32,
    #[serde(flatten)]
    content: ContentRef<'a>,
}

#[derive(serde::Serialize)]
#[serde(tag = "kind", content = "spec", rename_all = "snake_case")]
enum ContentRef<'a> {
    Plot(&'a Plot<'static>),
    Grid(&'a Grid<'static>),
}

impl serde::Serialize for Document {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let content = match &self.content {
            Content::Plot(plot) => ContentRef::Plot(plot),
            Content::Grid(grid) => ContentRef::Grid(grid),
        };
        EnvelopeRef {
            version: Self::VERSION,
            content,
        }
        .serialize(serializer)
    }
}

#[derive(serde::Deserialize)]
struct OwnedEnvelope {
    version: u32,
    #[serde(flatten)]
    content: OwnedContent,
}

#[derive(serde::Deserialize)]
#[serde(tag = "kind", content = "spec", rename_all = "snake_case")]
enum OwnedContent {
    Plot(Plot<'static>),
    Grid(Grid<'static>),
}

impl<'de> serde::Deserialize<'de> for Document {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error as _;

        let envelope = <OwnedEnvelope as serde::Deserialize>::deserialize(deserializer)?;
        if envelope.version != Self::VERSION {
            return Err(D::Error::custom(format!(
                "unsupported malevich document version {}; this build supports version {}",
                envelope.version,
                Self::VERSION
            )));
        }
        let document = Document {
            content: match envelope.content {
                OwnedContent::Plot(plot) => Content::Plot(plot),
                OwnedContent::Grid(grid) => Content::Grid(grid),
            },
        };
        document.validate().map_err(D::Error::custom)?;
        Ok(document)
    }
}
