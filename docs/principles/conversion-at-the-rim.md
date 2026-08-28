# Conversion lives at the rim

The core computes in `f64`, monomorphically. Every other numeric shape
converts exactly once, at ingestion, and never again.

## Why

The tempting design is a core generic over a `Float` trait: accept `f32`,
integers, decimals, anything, all the way down. The cost surfaces
everywhere the library actually earns its keep. Tick placement needs
literals (`10.0`, `0.5`, the SI thresholds); exact-decimal formatting needs
a concrete mantissa; the KDE needs constants; every generic bound infects
every signature; and the hot loops either monomorphize into code-size bloat
or dispatch per point. Topos — this library's sibling — pays a documented
"generic-literal gap" tax for payload genericity because one engine over
scalars and tensors is its literal thesis. Malevich has no such thesis: a
generic float core would buy nothing and cost literals, tick math, and
formatting everywhere.

The opposite temptation is accepting nothing but `&[f64]`, which taxes
every caller with a conversion loop and an allocation they must remember to
write — and half of them convert wrongly at the first `NaN`.

## The idea

Make the boundary a trait and the interior a type. Anything series-shaped —
slices, arrays, vectors of any primitive numeric type, iterators — converts
exactly once at the rim into contiguous `f64` where `NaN` is the gap.
Borrowed `f64` slices cross for free. Inside the rim there is one numeric
world: monomorphic `f64`, no bounds, no dispatch, literals and constants
used freely, hot loops the optimizer can see through.

The rim is also where the ecosystem plugs in without becoming a dependency.
ndarray's contiguous arrays borrow zero-copy behind a feature; polars needs
no feature at all, because a contiguous column is already a borrowed slice
and its null-yielding iterator maps straight onto the gap convention. The
convention is the interface: big data libraries integrate by meeting `f64`
plus `NaN`, not by being linked.

`f64` is the right single type because it holds every `f32`, every `u32`,
and every count exactly, and a chart raster cannot resolve the difference
that remains at `u64` extremes.

## Consequences

- One implementation of ticks, formatting, statistics, and rasterization —
  no generic variants to test in every width.
- Ingestion cost is explicit, bounded, and paid once; render loops never
  convert.
- The gap convention is universal because ingestion establishes it:
  everything after the rim may assume `NaN` means gap.
- New input shapes are `IntoSeries` implementations, not core changes.
- No `Float` bound ever appears in a public signature.

## Not this

- A core generic over a float trait, or per-point `Into<f64>` calls in a
  draw loop.
- A second numeric path for `f32` "for performance."
- Depending on a dataframe library to accept its columns.
- Converting lazily inside stats, so the same series pays per use.

See [What earns a concept](what-earns-a-concept.md) for the closed-core
posture generally, and [Vision](../vision.md) rule 2.

## Spelled today

`data::Series` is the contiguous `f64` column; `data::IntoSeries` is the
rim, with zero-copy borrowing for `f64` slices and `FromIterator` for
iterators. The `ndarray` feature adds zero-copy ingestion for contiguous
arrays; the polars recipe in the README is two lines against the public
rim. Function sampling arrives with the marks, not through the rim. This
section may rot; the rest must not.
