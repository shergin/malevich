//! `Series`: one column of scalar data, and the `IntoSeries` ingestion trait.

use std::borrow::Cow;

/// One column of scalar data: contiguous `f64`, where `NaN` is a gap.
///
/// A series either borrows a caller's `&[f64]` (zero-copy) or owns its values (any
/// other input converts and copies exactly once at ingestion). Gaps (`NaN`) are
/// preserved — they render as visible breaks, never interpolated across.
///
/// ```
/// use malevich::data::{IntoSeries, Series};
///
/// let borrowed: Series = (&[1.0, 2.0, f64::NAN, 4.0][..]).into_series();
/// let counted: Series = (0..5).map(|i| i as f64).collect();
/// assert_eq!(borrowed.extent(), Some((1.0, 4.0)));
/// assert_eq!(counted.len(), 5);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Series<'a> {
    values: Cow<'a, [f64]>,
}

impl<'a> Series<'a> {
    /// The values, in order, gaps included.
    pub fn as_slice(&self) -> &[f64] {
        &self.values
    }

    /// Iterates over the values, gaps included.
    pub fn iter(&self) -> impl Iterator<Item = f64> + '_ {
        self.values.iter().copied()
    }

    /// The number of values, gaps included.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the series has no values.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// The `(min, max)` over finite values, or `None` if there are none.
    ///
    /// Gaps (`NaN`) and infinities do not participate: an axis domain must be finite.
    pub fn extent(&self) -> Option<(f64, f64)> {
        let mut extent: Option<(f64, f64)> = None;
        for value in self.iter().filter(|value| value.is_finite()) {
            extent = match extent {
                None => Some((value, value)),
                Some((min, max)) => Some((min.min(value), max.max(value))),
            };
        }
        extent
    }

    /// Detaches from any borrowed storage, making the series `'static`.
    pub fn into_owned(self) -> Series<'static> {
        Series {
            values: Cow::Owned(self.values.into_owned()),
        }
    }
}

impl FromIterator<f64> for Series<'static> {
    fn from_iter<I: IntoIterator<Item = f64>>(iter: I) -> Self {
        Series {
            values: Cow::Owned(iter.into_iter().collect()),
        }
    }
}

impl FromIterator<f32> for Series<'static> {
    fn from_iter<I: IntoIterator<Item = f32>>(iter: I) -> Self {
        Series {
            values: Cow::Owned(iter.into_iter().map(f64::from).collect()),
        }
    }
}

/// Conversion into a [`Series`]: the single ingestion boundary of the crate.
///
/// Borrowed `f64` slices are zero-copy; all other implementations convert and copy
/// once. Implemented for slices, arrays, and vectors of every primitive numeric type
/// (integers wider than 53 bits round to the nearest representable `f64`), and for
/// `Series` itself.
pub trait IntoSeries<'a> {
    /// Converts `self` into a series.
    fn into_series(self) -> Series<'a>;
}

impl<'a> IntoSeries<'a> for Series<'a> {
    fn into_series(self) -> Series<'a> {
        self
    }
}

impl<'a> IntoSeries<'a> for &'a [f64] {
    fn into_series(self) -> Series<'a> {
        Series {
            values: Cow::Borrowed(self),
        }
    }
}

impl<'a> IntoSeries<'a> for &'a Vec<f64> {
    fn into_series(self) -> Series<'a> {
        Series {
            values: Cow::Borrowed(self),
        }
    }
}

impl<'a, const N: usize> IntoSeries<'a> for &'a [f64; N] {
    fn into_series(self) -> Series<'a> {
        Series {
            values: Cow::Borrowed(self),
        }
    }
}

impl<'a> IntoSeries<'a> for Vec<f64> {
    fn into_series(self) -> Series<'a> {
        Series {
            values: Cow::Owned(self),
        }
    }
}

impl<'a, const N: usize> IntoSeries<'a> for [f64; N] {
    fn into_series(self) -> Series<'a> {
        Series {
            values: Cow::Owned(self.to_vec()),
        }
    }
}

/// Implements the converting (copy-once) ingestion for a non-`f64` scalar type.
macro_rules! converting_into_series {
    ($($scalar:ty),*) => {$(
        impl<'a, 'b> IntoSeries<'a> for &'b [$scalar] {
            fn into_series(self) -> Series<'a> {
                Series {
                    values: Cow::Owned(self.iter().map(|&value| value as f64).collect()),
                }
            }
        }

        impl<'a, 'b> IntoSeries<'a> for &'b Vec<$scalar> {
            fn into_series(self) -> Series<'a> {
                self.as_slice().into_series()
            }
        }

        impl<'a> IntoSeries<'a> for Vec<$scalar> {
            fn into_series(self) -> Series<'a> {
                self.as_slice().into_series()
            }
        }

        impl<'a, 'b, const N: usize> IntoSeries<'a> for &'b [$scalar; N] {
            fn into_series(self) -> Series<'a> {
                self.as_slice().into_series()
            }
        }

        impl<'a, const N: usize> IntoSeries<'a> for [$scalar; N] {
            fn into_series(self) -> Series<'a> {
                self.as_slice().into_series()
            }
        }
    )*};
}

converting_into_series!(f32, i8, i16, i32, i64, isize, u8, u16, u32, u64, usize);

/// With the `serde` feature, a series encodes as a sequence of optional numbers:
/// gaps (`NaN`) become `None`/`null`, so they survive formats like JSON that
/// cannot carry `NaN`, and decode back to gaps exactly.
///
/// Deserialization also accepts `{ "col": N }` when [`with_columns`] is in
/// progress: the Nth bound buffer becomes this series. That is the WASM
/// columns lane — large data stays out of the JSON document. A column
/// reference without a bind, or an index past the bind, is an error. Stored
/// documents never use column references; they are a render-request spelling.
#[cfg(feature = "serde")]
impl serde::Serialize for Series<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(
            self.iter()
                .map(|value| if value.is_nan() { None } else { Some(value) }),
        )
    }
}

#[cfg(feature = "serde")]
thread_local! {
    static COLUMNS: std::cell::RefCell<Vec<Vec<f64>>> = const { std::cell::RefCell::new(Vec::new()) };
}

/// Binds `columns` for the duration of `f` so a series may deserialize from
/// `{ "col": N }` as the Nth buffer (cloned into the series). Nested binds
/// replace the table; the previous table is restored when `f` returns.
///
/// The JSON document stays the persistence format; column references exist
/// only inside this bind — a stored document that cannot be decoded without
/// a side buffer is a lie.
#[cfg(feature = "serde")]
pub fn with_columns<T>(columns: Vec<Vec<f64>>, f: impl FnOnce() -> T) -> T {
    struct Guard(Vec<Vec<f64>>);
    impl Drop for Guard {
        fn drop(&mut self) {
            COLUMNS.with(|slot| {
                *slot.borrow_mut() = std::mem::take(&mut self.0);
            });
        }
    }
    let previous = COLUMNS.with(|slot| std::mem::replace(&mut *slot.borrow_mut(), columns));
    let _guard = Guard(previous);
    f()
}

#[cfg(feature = "serde")]
impl<'de, 'a> serde::Deserialize<'de> for Series<'a> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        #[serde(untagged)]
        enum Wire {
            Values(Vec<Option<f64>>),
            Column { col: usize },
        }

        match Wire::deserialize(deserializer)? {
            Wire::Values(values) => Ok(values
                .into_iter()
                .map(|value| value.unwrap_or(f64::NAN))
                .collect()),
            Wire::Column { col } => {
                let values = COLUMNS.with(|slot| slot.borrow().get(col).cloned());
                match values {
                    Some(values) => Ok(values.into_iter().collect()),
                    None => Err(serde::de::Error::custom(format!(
                        "series column {col} is not bound; deserialize inside data::with_columns"
                    ))),
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "tests/series_tests.rs"]
mod tests;
