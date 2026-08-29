//! `Viewport`: a domain window as a value, with pure zoom and pan arithmetic.

/// An axis window pair for interactive viewing: zoom and pan as plain domain
/// arithmetic over the [`Plot::x_domain`](crate::Plot::x_domain) /
/// [`Plot::y_domain`](crate::Plot::y_domain) scale options — never a render
/// mode.
///
/// `None` on an axis means automatic: the plot fits that axis to its data (or
/// to its own fixed domain) as if no viewport existed. A window becomes fixed
/// by seeding from a rendered plot's [`Mapping::viewport`](crate::plot::Mapping::viewport)
/// — "the view I am looking at" — and then transforming it: every method is a
/// pure function returning a new value, so a host stores one `Viewport`, feeds
/// gestures through it, and applies it with [`Plot::viewport`](crate::Plot::viewport)
/// on the next frame. Transforms on an unfixed axis are no-ops: there is no
/// window to move.
///
/// Log axes zoom and pan in decade space (so equal gestures cover equal
/// factors), time axes in seconds, linear axes in value space. All windows are
/// finite by construction.
///
/// The wire form carries only the windows: which space an axis transforms in
/// (decade or value) is derived from the plot's scale at seeding time, not
/// spec — persisting it would let a stored flag disagree with a plot whose
/// scale has since changed. A restored viewport is therefore complete for
/// [`Plot::viewport`](crate::Plot::viewport) (which reads only the windows);
/// to transform it, render once and re-seed from the mapping first, which is
/// the gesture lifecycle anyway.
///
/// ```
/// use malevich::Viewport;
///
/// let view = Viewport::auto();          // both axes automatic
/// assert!(view.is_auto());
/// // After a render: mapping.viewport() fixes the current domains, then
/// // view.zoom_x(0.5, cursor_x) halves the x window around the cursor.
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Viewport {
    #[cfg_attr(feature = "serde", serde(default))]
    x: Option<(f64, f64)>,
    #[cfg_attr(feature = "serde", serde(default))]
    y: Option<(f64, f64)>,
    /// Derived from the plot's scale when seeded; never on the wire.
    #[cfg_attr(feature = "serde", serde(skip))]
    log_x: bool,
    /// Derived from the plot's scale when seeded; never on the wire.
    #[cfg_attr(feature = "serde", serde(skip))]
    log_y: bool,
}

impl Viewport {
    /// The automatic viewport: both axes fit their data.
    pub fn auto() -> Viewport {
        Viewport::default()
    }

    /// A viewport with the given fixed windows; built by
    /// [`Mapping::viewport`](crate::plot::Mapping::viewport).
    pub(crate) fn seeded(
        x: Option<(f64, f64)>,
        y: Option<(f64, f64)>,
        log_x: bool,
        log_y: bool,
    ) -> Viewport {
        let sanitize = |window: Option<(f64, f64)>, log: bool| {
            window.filter(|&(lo, hi)| {
                lo.is_finite() && hi.is_finite() && lo < hi && (!log || lo > 0.0)
            })
        };
        Viewport {
            x: sanitize(x, log_x),
            y: sanitize(y, log_y),
            log_x,
            log_y,
        }
    }

    /// The x window, when fixed.
    pub fn x(&self) -> Option<(f64, f64)> {
        self.x
    }

    /// The y window, when fixed.
    pub fn y(&self) -> Option<(f64, f64)> {
        self.y
    }

    /// Whether both axes are automatic.
    pub fn is_auto(&self) -> bool {
        self.x.is_none() && self.y.is_none()
    }

    /// Scales the x window around `anchor` by `factor` — below 1 zooms in,
    /// above 1 zooms out; the anchor keeps its position, which is what puts
    /// "zoom at the cursor" one call away. No-op on an unfixed axis or a
    /// non-positive/non-finite factor.
    #[must_use]
    pub fn zoom_x(mut self, factor: f64, anchor: f64) -> Viewport {
        self.x = zoom(self.x, self.log_x, factor, anchor);
        self
    }

    /// Scales the y window around `anchor` by `factor`; see [`Viewport::zoom_x`].
    #[must_use]
    pub fn zoom_y(mut self, factor: f64, anchor: f64) -> Viewport {
        self.y = zoom(self.y, self.log_y, factor, anchor);
        self
    }

    /// Zooms both axes around a data-point anchor; see [`Viewport::zoom_x`].
    #[must_use]
    pub fn zoom(self, factor: f64, anchor: (f64, f64)) -> Viewport {
        self.zoom_x(factor, anchor.0).zoom_y(factor, anchor.1)
    }

    /// Shifts the x window by `fraction` of its own span — positive toward
    /// larger x. No-op on an unfixed axis.
    #[must_use]
    pub fn pan_x(mut self, fraction: f64) -> Viewport {
        self.x = pan(self.x, self.log_x, fraction);
        self
    }

    /// Shifts the y window by `fraction` of its own span — positive toward
    /// larger y. No-op on an unfixed axis.
    #[must_use]
    pub fn pan_y(mut self, fraction: f64) -> Viewport {
        self.y = pan(self.y, self.log_y, fraction);
        self
    }

    /// Fixes the x window directly (ascending, finite bounds) — the
    /// rubber-band zoom primitive. Non-finite or empty windows are ignored.
    #[must_use]
    pub fn with_x(mut self, low: f64, high: f64) -> Viewport {
        let (low, high) = (low.min(high), low.max(high));
        if low.is_finite() && high.is_finite() && low < high && (!self.log_x || low > 0.0) {
            self.x = Some((low, high));
        }
        self
    }

    /// Fixes the y window directly; see [`Viewport::with_x`].
    #[must_use]
    pub fn with_y(mut self, low: f64, high: f64) -> Viewport {
        let (low, high) = (low.min(high), low.max(high));
        if low.is_finite() && high.is_finite() && low < high && (!self.log_y || low > 0.0) {
            self.y = Some((low, high));
        }
        self
    }

    /// Slides the x window inside `[low, high]` without changing its span; a
    /// window wider than the extent becomes the extent. The "never scroll past
    /// the data" clamp. No-op on an unfixed axis or an invalid extent.
    #[must_use]
    pub fn clamp_x(mut self, low: f64, high: f64) -> Viewport {
        self.x = clamp(self.x, low, high);
        self
    }

    /// Slides the y window inside `[low, high]`; see [`Viewport::clamp_x`].
    #[must_use]
    pub fn clamp_y(mut self, low: f64, high: f64) -> Viewport {
        self.y = clamp(self.y, low, high);
        self
    }

    /// Fixes the x window to the trailing `width` ending at `latest` — the
    /// follow-the-stream view. Ignored unless both are finite and `width` is
    /// positive.
    #[must_use]
    pub fn tail(mut self, latest: f64, width: f64) -> Viewport {
        if latest.is_finite() && width.is_finite() && width > 0.0 {
            self.x = Some((latest - width, latest));
        }
        self
    }

    /// Back to automatic on both axes.
    #[must_use]
    pub fn reset(mut self) -> Viewport {
        self.x = None;
        self.y = None;
        self
    }

    /// Back to automatic on x only.
    #[must_use]
    pub fn reset_x(mut self) -> Viewport {
        self.x = None;
        self
    }

    /// Back to automatic on y only.
    #[must_use]
    pub fn reset_y(mut self) -> Viewport {
        self.y = None;
        self
    }
}

/// Window scaling around an anchor, in the axis's own space (decades on log).
fn zoom(window: Option<(f64, f64)>, log: bool, factor: f64, anchor: f64) -> Option<(f64, f64)> {
    let (lo, hi) = window?;
    if !(factor.is_finite() && factor > 0.0 && anchor.is_finite()) {
        return window;
    }
    let (lo, hi, anchor) = if log {
        if anchor <= 0.0 {
            return window;
        }
        (lo.log10(), hi.log10(), anchor.log10())
    } else {
        (lo, hi, anchor)
    };
    let new_lo = anchor - (anchor - lo) * factor;
    let new_hi = anchor + (hi - anchor) * factor;
    if !(new_lo.is_finite() && new_hi.is_finite()) || new_lo >= new_hi {
        return window;
    }
    Some(if log {
        (10f64.powf(new_lo), 10f64.powf(new_hi))
    } else {
        (new_lo, new_hi)
    })
}

/// Window shifting by a fraction of the span, in the axis's own space.
fn pan(window: Option<(f64, f64)>, log: bool, fraction: f64) -> Option<(f64, f64)> {
    let (lo, hi) = window?;
    if !fraction.is_finite() {
        return window;
    }
    let (lo, hi) = if log {
        (lo.log10(), hi.log10())
    } else {
        (lo, hi)
    };
    let shift = (hi - lo) * fraction;
    let (new_lo, new_hi) = (lo + shift, hi + shift);
    if !(new_lo.is_finite() && new_hi.is_finite()) || new_lo >= new_hi {
        return window;
    }
    Some(if log {
        (10f64.powf(new_lo), 10f64.powf(new_hi))
    } else {
        (new_lo, new_hi)
    })
}

/// Slides a window inside an extent, preserving its span where possible.
fn clamp(window: Option<(f64, f64)>, low: f64, high: f64) -> Option<(f64, f64)> {
    let (lo, hi) = window?;
    if !(low.is_finite() && high.is_finite()) || low >= high {
        return window;
    }
    let span = hi - lo;
    if span >= high - low {
        return Some((low, high));
    }
    if lo < low {
        Some((low, low + span))
    } else if hi > high {
        Some((high - span, high))
    } else {
        Some((lo, hi))
    }
}

#[cfg(test)]
#[path = "tests/viewport_tests.rs"]
mod tests;
