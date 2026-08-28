# Performance

Fast is a feature, and claims are measured: every advertised number has a
bench behind it, recorded with its machine and date. This file is the
public story; [BENCHMARKS.md](../BENCHMARKS.md) is the authoritative dated
record it summarizes.

The reason speed and honesty coexist is the oracle: the fast path is proven
byte-identical to drawing every point, so there is no fidelity knob being
traded away. See
[The full draw is the oracle](principles/full-draw-oracle.md).

## The mechanisms

- **Lines: M4 to the raster.** Large line layers reduce to
  min/max/first/last per raster column, bucketed by the column each point
  renders into — O(n) once, then O(width × height), and pixel-identical to
  the full draw by construction. The reduction is auto-inserted past four
  points per column.
- **Grids: bucket-exact reduction.** Cells grids denser than the raster
  reduce through the shared `Reducer` vocabulary — every screen bucket
  owns the cells whose centers fall inside it. Cost is linear in the grid,
  about 10 ns per cell, not in the raster.
- **No allocation per point.** Rendering retains labels and identities at
  construction and borrows them thereafter; CI enforces allocation
  ceilings (at most 275 allocations and 64 KiB for the 10k-point render)
  so a structural regression — an allocation per point, a new large
  intermediate — fails the build.

## Measured

A snapshot from one 2021 MacBook Pro (Apple M1 Pro), single-threaded,
end to end — construct, resolve, reduce, rasterize, encode. Order of
magnitude and which mechanism wins, not a promise for your machine.

| measurement | result |
|---|---:|
| line, 10,000 points, 80×20 | 69 µs |
| line, 10,000,000 points, 80×20 | 32 ms |
| cells, 2048×2048 grid onto 80×24 | 44 ms |
| streaming least squares, 1M pairs (`stat::Fit`) | 5.2 ms |
| 100,000 points, 5 categories via `color_by` | 2.1 ms |
| 100,000 points, 100,000 categories | 5.3 ms |

The categorical pair is a structural fence: runtime grows with input plus
legend size, not input × category count. The ten-million-point row is the
README's "tens of milliseconds" claim; the cells row is its matrix analog
(4.19 million cells onto ~4k screen buckets).

## Rerun on yours

```sh
cargo bench --bench render -- render/line_10m_80x20
cargo bench --bench render -- render/cells_2048x2048_80x24
cargo bench --bench alloc
```

The bench suite is the only source docs quote numbers from. Baselines,
machine details, confidence intervals, and the update protocol live in
[BENCHMARKS.md](../BENCHMARKS.md).
