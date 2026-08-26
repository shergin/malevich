//! Statistical transforms: aggregation that runs before scales see the data.
//!
//! The execution model follows the operation: [`Moments`], [`Fit`], [`Bins`], and
//! [`M4`] are online accumulators with explicit merge contracts; [`Reducer`] maps a
//! collection to one value and buffers only for order statistics; [`Window`], KDE,
//! ECDF, LTTB, contours, and stacking are batch transforms. The plot pipeline
//! inserts [`m4`] automatically for large line layers; everything is also public
//! API for direct use.

mod agg;
mod bin;
mod box_stats;
mod contour;
mod ecdf;
mod ewma;
mod fit;
mod kde;
mod lttb;
mod m4;
mod moments;
mod reducer;
mod roc;
mod stack;
mod window;

/// Defensive ceiling for caller-selected statistical output geometry.
pub(crate) const MAX_STAT_ELEMENTS: usize = 1_000_000;

pub use agg::Agg;
pub use bin::{Bins, Histogram2d, binned, bins2, try_bins2};
pub use box_stats::BoxStats;
pub use contour::{Contour, contours};
pub use ecdf::ecdf;
pub use ewma::ewma;
pub use fit::Fit;
pub use kde::kde;
pub use lttb::lttb;
pub use m4::{M4, m4};
pub(crate) use m4::{m4_mapped, m4_mapped_categories};
pub use moments::Moments;
pub(crate) use reducer::ReducerState;
pub use reducer::{Reducer, quantiles};
pub use roc::{auc, roc};
pub use stack::stack;
pub use window::Window;
