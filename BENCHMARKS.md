# Benchmark baselines

Malevich treats performance as a measured engineering constraint, not a portable
speed promise. Wall-clock results vary with hardware, compiler, power state, and
background load. This file is the authoritative dated record behind the README's
“tens of milliseconds” claim.

## 2026-08-25 addition (released in 1.18.0)

- Revision: `b0887bc` (the commit introducing the measured reduction)
- Machine, OS, compiler, profile: as in the 2026-08-07 baseline below

| Measurement | Estimate | 95% interval |
| --- | ---: | ---: |
| `render/cells_2048x2048_80x24` | 43.720 ms | 43.535–43.944 ms |

```sh
cargo bench --bench render -- render/cells_2048x2048_80x24
```

The matrix analog of the ten-million-point line: 4.19 million cells max-reduce
onto ~4k screen buckets in tens of milliseconds, about 10 ns per cell. The
bucket-exact reduction walks every covered cell once, so the cost is linear in
the grid, not in the raster.

## 2026-08-24 baseline (1.17.0)

- Revision: `7bbc202`
- Machine, OS, compiler, profile: as in the 2026-08-07 baseline below

| Measurement | Estimate | 95% interval |
| --- | ---: | ---: |
| `render/line_10k_80x20` | 69.182 µs | 68.837–69.594 µs |
| `render/line_10m_80x20` | 31.878 ms | 31.752–32.034 ms |
| `stat/fit_1m` | 5.2392 ms | 5.2163–5.2711 ms |
| `render/color_by_100k/5_categories` | 2.0587 ms | 2.0502–2.0685 ms |
| `render/color_by_100k/100000_categories` | 5.3453 ms | 5.3271–5.3665 ms |

```sh
cargo bench --bench render -- render/line_10k_80x20
cargo bench --bench render -- render/line_10m_80x20
cargo bench --bench render -- stat/fit_1m
cargo bench --bench render -- render/color_by_100k/5_categories
cargo bench --bench render -- render/color_by_100k/100000_categories
```

Against 1.16.0 on the same machine, the 10k render is 4.1% higher after making
gaps explicit path topology; the ten-million-point render is 5.9% lower after
selecting the ordinary affine map once and keeping each M4 bucket's current run
directly addressable. The fit result is within 0.6% of its prior estimate.

The categorical rows are the new structural fence. Both render the same 100,000
points; the second also carries 100,000 distinct labels and identities. Runtime
therefore grows with input plus legend size, not with input × category count as the
old masked-layer expansion did.

## 2026-08-15 baseline (1.16.0)

- Revision: `3b17b0d`
- Machine, OS, compiler, profile: as in the 2026-08-07 baseline below

| Measurement | Estimate | 95% interval |
| --- | ---: | ---: |
| `render/line_10k_80x20` | 66.466 µs | 66.322–66.615 µs |
| `render/line_10m_80x20` | 33.878 ms | 33.659–34.194 ms |
| `stat/fit_1m` | 5.2083 ms | 5.2019–5.2149 ms |

```sh
cargo bench --bench render -- render/line_10k_80x20
cargo bench --bench render -- render/line_10m_80x20
cargo bench --bench render -- stat/fit_1m
```

Rerun for 1.16.0 because resolution changed (the `color_by` layer expansion
and the shared line-reduction helper): the render rows came out 2.2% and 6.5%
lower than the 2026-08-07 baseline on the same machine — the categorical
channel costs the headline path nothing measurable.

`stat/fit_1m` is one million `(x, y)` pairs through the streaming
least-squares accumulator (`stat::Fit`): bivariate Welford, single-threaded,
no allocation in the loop. The accumulator merges associatively, so hosts can
split this scan across chunks and combine.

## 2026-08-07 baseline

- Revision: `7ff2bc0`
- Machine: 2021 MacBook Pro, Apple M1 Pro (10 cores), 32 GB RAM
- OS: macOS 26.5.2 (Darwin 25.5.0), arm64
- Compiler: `rustc 1.97.1 (8bab26f4f 2026-07-14)`, LLVM 22.1.8
- Profile: Cargo `bench` / optimized, Criterion 0.5 default 100-sample run

| Measurement | Estimate | 95% interval |
| --- | ---: | ---: |
| `render/line_10k_80x20` | 67.928 µs | 67.887–67.973 µs |
| `render/line_10m_80x20` | 36.226 ms | 36.202–36.251 ms |

Commands:

```sh
cargo bench --bench render -- render/line_10k_80x20
cargo bench --bench render -- render/line_10m_80x20
```

The benchmark is end to end: construct the preset, resolve domains and layout,
perform M4 reduction, rasterize an 80×20 braille frame, and encode the final string.
It is single-threaded. The ten-million-point input vectors are prepared outside the
timed iteration.

The earlier `0f3ad5a` record on this machine was 81.818 µs and 42.260 ms,
respectively. The current measurements are 17.0% and 14.3% lower. These are
same-machine historical comparisons, not portable performance promises.

### Profiling decision

A five-second Instruments Time Profiler capture of the 10k case attributed 2,670
of 5,108 leaf samples (about 52%) to resolution. A measured A/B implementation
kept the compact resolved-layer probe rather than duplicating every mark's domain
rules in a parallel metadata type: the smaller design was faster and retains one
source of truth. The accepted change instead:

- keeps implicit coordinates symbolic;
- summarizes a line into only the two endpoints needed by its linear or log axis;
- retains the probed layout for drawing, avoiding a second round of tick formatting,
  gutter measurement, and colorbar work.

Cell and hybrid device-pixel rasterizers both use that same prepared-render phase.
Their target policy contains only sampling density, marker cycling, downsampling,
and the pixel fallback for cell-only corner glyphs; no parallel mark metadata exists.

The pixel-exact raw-versus-M4 oracle and all rendering snapshots remained identical.

## Allocation contract

At revision `7bbc202`, optimized on the machine above, the 10k render measured **183
allocations and 58,388 allocated bytes**, producing 2,966 output bytes. A 100k-point
plot with one unique category per point measured **67 allocations and 34,141 bytes**,
producing 1,791 output bytes. Rust 1.88 is the CI authority for the ceilings:

```sh
cargo bench --bench alloc
```

CI runs that harness on Ubuntu 24.04 with Rust 1.88 and `--check`. It permits at most
275 allocations and 64 KiB of heap traffic. Those ceilings intentionally leave
headroom for compiler and allocator details while catching structural regressions
such as an allocation per input point or a new large intermediate buffer. CI does not
gate wall-clock time on shared runners.

The line measurement includes gap-aware M4 state and the two-color cell surface. The
categorical measurement proves that rendering does not allocate per point or per
category: labels and identities are retained when the mark is built, while render
preparation borrows them. The unchanged 64 KiB ceiling still catches a larger
per-cell representation or a manufactured intermediate.

To update this record, benchmark an otherwise idle machine, record the revision,
hardware, OS, compiler, commands, point estimates, and confidence intervals, then
change the allocation ceilings only when a reviewed design change explains the new
traffic.
