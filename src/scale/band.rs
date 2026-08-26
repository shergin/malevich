//! The band scale: positions for a run of categories across a raster range.

/// Evenly spaced bands for `count` categories across a subpixel range, with
/// proportional padding between and around them (the d3 band-scale model).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Band {
    count: usize,
    start: f64,
    step: f64,
    bandwidth: f64,
}

/// Fraction of one step left as padding between adjacent bands (and half of it
/// outside the outermost bands).
const PADDING: f64 = 0.25;

impl Band {
    /// Lays out `count` bands across `range` (ascending, inclusive).
    pub fn new(count: usize, range: (f64, f64)) -> Band {
        let span = (range.1 - range.0).max(0.0);
        let steps = count as f64 + PADDING;
        let step = if count == 0 { 0.0 } else { span / steps };
        Band {
            count,
            start: range.0 + step * PADDING,
            step,
            bandwidth: step * (1.0 - PADDING),
        }
    }

    /// The number of bands.
    pub fn count(&self) -> usize {
        self.count
    }

    /// The width of one band.
    pub fn bandwidth(&self) -> f64 {
        self.bandwidth
    }

    /// The distance between the starts of adjacent bands.
    pub fn step(&self) -> f64 {
        self.step
    }

    /// The left edge of band `index`.
    pub fn position(&self, index: usize) -> f64 {
        self.start + self.step * index as f64
    }

    /// The center of band `index`.
    pub fn center(&self, index: usize) -> f64 {
        self.position(index) + self.bandwidth / 2.0
    }

    /// The band whose span contains `position` — `None` in the padding between
    /// bands and outside the outermost ones.
    pub fn index_at(&self, position: f64) -> Option<usize> {
        if self.count == 0 || self.step <= 0.0 {
            return None;
        }
        let offset = position - self.start;
        if offset < 0.0 {
            return None;
        }
        let index = (offset / self.step).floor();
        let within = offset - index * self.step;
        let index = index as usize;
        (index < self.count && within < self.bandwidth).then_some(index)
    }
}

#[cfg(test)]
#[path = "tests/band_tests.rs"]
mod tests;
