//! The error type for the fallible API.

use std::fmt;

/// Why a plot/grid spec or render request is invalid.
///
/// Returned by every strict construction, configuration, validation, and rendering
/// boundary. Invalid caller-controlled input to a `Result`-returning function is an
/// `Error`, never a constructor assertion. Plain constructors remain concise for
/// inline specs and may panic on documented programmer invariants; infallible render
/// methods instead shed malformed retained content. `validate` and `try_render` are
/// their strict counterparts.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// Two channels of one mark that must pair up have different lengths.
    UnequalChannels {
        /// The mark and the channels involved, e.g. `"Line: x and y"`.
        mark: &'static str,
        /// The two lengths, in the order named.
        lengths: (usize, usize),
    },
    /// A gridded mark's value count is not a whole number of rows.
    NonRectangular {
        /// The mark, e.g. `"Cells"`.
        mark: &'static str,
        /// The value count and the column count that does not divide it.
        shape: (usize, usize),
    },
    /// A required dimension is empty — zero columns, or a colormap with too few stops.
    EmptyDimension {
        /// What was empty, e.g. `"Cells columns"` or `"Colormap stops"`.
        what: &'static str,
    },
    /// A manual axis domain bound is not finite.
    NonFiniteDomain {
        /// The axis, `"x"` or `"y"`.
        axis: &'static str,
    },
    /// A scale cannot describe the data on its axis.
    IncompatibleScale {
        /// What conflicts, e.g. `"a log y axis needs a positive domain"`.
        detail: &'static str,
    },
    /// A constructor argument is outside the operation's mathematical domain.
    InvalidParameter {
        /// What the caller must change.
        detail: &'static str,
    },
    /// A caller-controlled dimension or derived area exceeds a defensive limit.
    DimensionTooLarge {
        /// The dimension being checked, e.g. `"frame cell count"`.
        what: &'static str,
        /// The requested value, or [`usize::MAX`] when its calculation overflowed.
        requested: usize,
        /// The largest accepted value.
        limit: usize,
    },
    /// Memory for a bounded operation could not be reserved.
    AllocationFailed {
        /// The allocation being attempted.
        what: &'static str,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::UnequalChannels { mark, lengths } => {
                write!(
                    f,
                    "{mark}: channels differ in length ({} and {})",
                    lengths.0, lengths.1
                )
            }
            Error::NonRectangular { mark, shape } => write!(
                f,
                "{mark}: {} values do not fill rows of {} columns",
                shape.0, shape.1
            ),
            Error::EmptyDimension { what } => write!(f, "{what} is empty"),
            Error::NonFiniteDomain { axis } => write!(f, "the {axis} domain is not finite"),
            Error::IncompatibleScale { detail } => write!(f, "incompatible scale: {detail}"),
            Error::InvalidParameter { detail } => write!(f, "invalid parameter: {detail}"),
            Error::DimensionTooLarge {
                what,
                requested,
                limit,
            } => write!(
                f,
                "{what} is too large (requested {requested}, limit {limit})"
            ),
            Error::AllocationFailed { what } => {
                write!(f, "could not reserve memory for {what}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// A [`Result`](std::result::Result) whose error is malevich's [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
