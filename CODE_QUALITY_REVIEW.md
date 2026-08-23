# Malevich code-quality and simplification review

Audit date: 2026-08-23

Repository revision: `ed3d43d6cab1`

Scope: the core library, `kaz` CLI, demos, examples, benchmarks, fuzz target, documentation, and CI

## Executive summary

Malevich is already a notably disciplined Rust project. It has a coherent retained-mode plotting model, a genuinely useful generic canvas abstraction, restrained dependencies, explicit serialization and terminology documents, extensive deterministic tests, allocation budgets, MSRV and semver checks, generated-document verification, and careful terminal-output hardening. The core forbids unsafe code. The full workspace test, lint, documentation, dependency-policy, and generated-doc checks all pass.

The most important weaknesses are not ordinary style problems. They are places where strong public concepts are broader than the implementations that enforce them:

1. Rendering is documented as total for malformed retained values, but a deserialized ragged large line and extreme time domains can panic.
2. M4 claims gap preservation and mergeability, but its bucket representation cannot represent multiple gaps and its merge loses a boundary gap.
3. Many APIs validate that inputs are finite but then perform arithmetic that is not closed over finite `f64` values. This causes NaN, infinity, wrong bins, or panics from valid finite inputs.
4. Constructor invariants and post-deserialization validation are maintained separately and have drifted.
5. Categorical color is implemented by expanding one logical channel into `k` full-length synthetic layers, creating an avoidable `O(n * k)` time-and-memory path.

These are connected. The code needs a smaller set of explicit boundary concepts:

- one local owner for every type invariant;
- shared, tested numeric-domain operations rather than scattered `is_finite` checks;
- a first-class category channel rather than NaN-masked layer expansion;
- explicit distinction between streaming accumulators and batch transforms;
- one prepared render pipeline shared by cell and pixel targets;
- one CLI recipe shared by rendering and code emission.

The recommended order is to lock in regression tests for the reproduced failures, restore totality and gap correctness, centralize invariants and numeric policy, then simplify categories, statistics execution, rendering orchestration, and CLI construction. The changes should preserve the compact M4 probe design that the project has already benchmarked; introducing a second parallel metadata model would repeat an experiment the project correctly rejected.

## Priority map

| ID | Priority | Area | Finding | Main consequence |
|---|---:|---|---|---|
| F1 | P0 | Rendering | A deserialized ragged large line panics inside mapped M4 | Violates the public no-panic rendering contract |
| F2 | P0 | M4 | Multiple gaps and partition-boundary gaps are not preserved | Visually reconnects data that must remain disconnected |
| F3 | P0 | Numeric policy | Finite endpoints can overflow intermediate arithmetic | NaN mappings, incorrect bins, time panics, CLI panics |
| F4 | P1 | Invariants | Constructor checks and `Mark::validate` have drifted | Invalid documents are accepted; downstream code receives impossible states |
| F5 | P1 | Categories | `color_by` expands into `O(n * k)` masked arrays/layers | Large memory use, repeated work, and amplification of gap bugs |
| F6 | P1 | CLI boundaries | Civil-time parsing and numeric selectors accept malformed input | Wrong timestamps, panic on extreme years, silent empty plots |
| F7 | P2 | Statistics | One reducer interface hides streaming and batch execution needs | Repeated allocation, `O(n * w)` windows, duplicated numerical logic |
| F8 | P2 | Rendering structure | Cell and hybrid raster paths duplicate orchestration | Behavioral drift risk and unnecessarily difficult changes |
| F9 | P2 | CLI structure | Rendering and code emission build the same chart twice in different forms | Duplication, extra parsing/work, weak semantic equivalence guarantees |
| F10 | P2 | Public API | `_with`, panicking, fallible, and degrading APIs do not form a clear system | Callers cannot infer safety from naming |
| F11 | P2 | Concepts/docs | “Every statistic is a mergeable monoid” overstates the implementation | Blurs ordered summaries, accumulators, and batch transforms |
| F12 | P3 | CI/dependencies | Demo tests are not exercised in CI; a few dependency duplicates remain | Small coverage and maintenance costs |

Priority definitions:

- **P0:** correctness or totality issue that should be fixed before relying on the affected public contract.
- **P1:** serious boundary or resource issue; address in the next focused quality cycle.
- **P2:** structural simplification or performance work after correctness is pinned down.
- **P3:** maintenance improvement with low immediate risk.

## Scope and method

The review followed data from public construction and deserialization through mark validation, resolution, layout, reduction, drawing, encoding, CLI parsing, and generated Rust emission. It also inspected tests, benchmarks, fuzzing, CI workflows, serialization fixtures, terminology, and performance notes.

The following checks passed at the audited revision:

- `cargo test --workspace --all-features --all-targets`
  - 385 core library tests
  - 74 CLI unit tests
  - 26 CLI integration tests
  - 3 packaging tests
  - 13 demo tests
  - examples and benchmark binaries
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --doc`: 26 doctests
- `RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features`
- `cargo run --example regen_docs -- --check`
- `cargo deny check`

One-off adversarial regression tests were used to verify boundary behavior and were removed after the audit. All persistent work from this review is this report.

## Current architecture

The main library pipeline is conceptually sound:

```text
Plot + retained Marks
        |
        v
resolve/probe ----------> Layout
        |                   |
        +------ target dimensions
        |
        v
ResolvedLayer values
        |
        v
generic Canvas
   /             \
Surface       PixelCanvas
   \             /
     encoders/output
```

Particularly good decisions include:

- `Plot` and marks are retained values, which makes validation, serialization, testing, and deterministic rerendering possible.
- `Canvas` carries the common drawing vocabulary across terminal cells and pixels instead of maintaining two unrelated renderers.
- Layout and geometry resource limits are explicit rather than accidental.
- M4 uses the actual raster width during final resolution. The two-pass probe/final design is performance-motivated and documented.
- The mark grammar is closed and expressed with enums. Exhaustive matches are often clearer here than a trait object hierarchy.
- Serialization is versioned and backed by golden fixtures.
- Terminal encoding treats control bytes as a security boundary.

The desired simplification should preserve these strengths. The goal is not to replace the architecture, but to make its boundaries truthful and its shared concepts explicit.

## Detailed findings

### F1 — Rendering is not total for all deserializable plots

**Evidence**

`Plot::render` in `src/plot/plot.rs:212-215` deliberately enters an infallible path. The module and crate documentation in `src/plot/plot.rs:9-14`, `src/lib.rs:23-27`, and `src/error.rs:5-13` describe malformed retained content as something rendering should shed or degrade rather than panic on.

A raw deserialized line with one x value and 1,000 y values renders safely while it stays below reduction thresholds, because ordinary line drawing effectively zips channels. Once the same retained value is large enough to use mapped M4, `src/stat/m4.rs:270-288` indexes x by every y index. The reproduced result was an index-out-of-bounds panic at `src/stat/m4.rs:272`.

Extreme but finite time domains expose a second totality failure. `Ticks::time(f64::MAX, f64::MAX, 5)` panics in debug mode during unchecked integer time stepping in `src/scale/time.rs:105-160`. In a wrapping build, the same increment can wrap from `i64::MAX` to `i64::MIN` and make the termination distance effectively enormous.

**Root cause**

The infallible renderer is tolerant at its outer boundary, but inner algorithms still assume constructor-established invariants. Deserialization creates a second construction route, and some size-dependent algorithms make stricter assumptions than their small-data equivalents.

**Recommendation**

Make totality a property of every private operation used by the infallible renderer:

1. Normalize paired channels once, before choosing a small or reduced algorithm. At minimum, mapped reduction must iterate paired x/y values rather than indexing one channel by the other.
2. Keep strict validation available to callers, but do not depend on it for memory safety or panic freedom.
3. Give time tick generation a supported civil-time range, checked increments, and a hard output bound.
4. Add a corpus test that deserializes malformed shapes for every mark and renders each on both sides of every algorithm threshold.
5. Add `catch_unwind` only to the test oracle. Do not catch panics inside production rendering; make the algorithms total instead.

**Acceptance criteria**

- Every successfully deserialized `Plot` can be passed to every infallible render method without a panic.
- Small and large versions of the same malformed channel shape degrade consistently.
- Tick generation always terminates and emits no more than its documented bound.

### F2 — M4's gap representation cannot meet its stated contract

**Evidence: multiple gaps in one bucket**

`Bucket` stores one `gap: Option<f64>` in `src/stat/m4.rs:12-21`. When a bucket sees another gap, the assignment around `src/stat/m4.rs:94` replaces the previous one.

For a single bucket containing y values:

```text
0, NaN, 10, NaN, 0
```

the reduced output was:

```text
0, 10, NaN, 0
```

The first separation disappeared, so the output reconnects 0 to 10. That contradicts the stated rule that a gap never reconnects values it separated. This is not merely a different sample choice; it changes the topology of the drawn path.

**Evidence: ordered merge**

Two ordered partial summaries were compared:

```text
left:  (0.1, 1)
right: gap, (0.3, 2)
```

Sequential summarization preserved:

```text
(0.1, 1), gap, (0.3, 2)
```

Merging the partial summaries produced the gap before the left point. The merge in `src/stat/m4.rs:134-152` combines the two bucket gap fields but does not translate the right summary's leading-gap state into the boundary after the left summary's last sample. Existing merge coverage in `src/stat/tests/m4_tests.rs:32-49` exercises gap-free data; gap tests cover sequential reduction rather than arbitrary ordered partitions.

**Evidence: invalid x values**

Raw line drawing in `src/plot/draw.rs:235-252` resets continuity when a point is invalid. Mapped M4 in `src/stat/m4.rs:270-288` ignores non-finite x values and non-finite mapped x positions without recording a break. A large line can therefore bridge over a NaN x or a value invalid under a logarithmic transform even when the small-line path does not.

Categorical line resolution makes this more important: points outside a category are represented as NaN masks before reduction. Alternating categories can create several artificial gaps in one raster bucket.

**Conceptual issue**

Exact preservation of an arbitrary number of gaps is incompatible with a representation that has exactly one gap slot and a strict four-emitted-items-per-bucket rule. The implementation needs an explicit tradeoff rather than a stronger claim than its state can encode.

**Recommendation**

Model path segmentation directly:

- Partition a line into finite runs before reduction, including invalid x, invalid mapped x, invalid y, and category transitions.
- Reduce samples within a run; do not encode path topology as a special sample inside an ordinary extrema bucket.
- Keep ordered leading/trailing connectivity state in mergeable summaries.
- If an absolute output cap can force gap loss, define and test that degradation explicitly. Prefer preserving disconnection over extrema when both cannot fit: inventing a connection is more visually misleading than omitting an extremum.
- Require ordered partition composition; do not describe M4 as an unordered monoid.

A compact `ReducedPath` made of segments is a simpler model than progressively adding more optional gap fields to `Bucket`. If materializing all segments is too costly, the same model can be streamed to the drawing stage.

**Tests to add**

- Sequential reduction equals reduction merged across every ordered partition point.
- Associativity across three ordered partitions, including leading, trailing, and multiple internal gaps.
- Small/raw and large/reduced paths have identical raster connectivity for NaN x, NaN y, log-invalid x, and category changes.
- Randomized tests compare connectivity and extrema envelopes, not only emitted samples.

### F3 — “Finite input” is not a sufficient numeric-domain contract

**Reproduced cases**

1. `Linear::new((-f64::MAX, f64::MAX), (0.0, 1.0)).map(f64::MAX)` returns NaN instead of 1.0. The subtraction in `src/scale/linear.rs:22-32` overflows to infinity, followed by an infinity/infinity operation.
2. `try_bins2` with x values `[-f64::MAX, f64::MAX]` and two x bins placed both samples in the first bin, producing `[2, 0]` rather than `[1, 1]`. The span in `src/stat/bin.rs:254-292` becomes infinity and the normalized coordinate collapses.
3. `kaz hist` and `kaz hist --bins 2` panic for input containing `-1e308` and `1e308`. Automatic bin selection permits the finite range, derives a non-representable span/width, then reaches a panicking constructor.
4. Extreme time domains panic during integer conversion or stepping in `src/scale/time.rs`.

Related code uses the same unsafe arithmetic shape:

- type-7 quantile interpolation as `a + p * (b - a)`;
- naive summation in reducers and KDE moments;
- midpoint calculations as `(a + b) / 2`;
- error-bar and bar-span endpoint arithmetic;
- fit and colormap normalization.

Some of these will degrade harmlessly in a specific caller, but they do not share a deliberate policy.

**Root cause**

The project often validates operands, then treats all derived arithmetic as valid. IEEE-754 finite numbers are not closed under addition, subtraction, multiplication, or conversion to integer time. Scattered post-operation `is_finite` checks will remain incomplete because each algorithm chooses its own fallback.

**Recommendation: introduce a numeric boundary layer**

Keep it small and concrete rather than creating a general numeric framework:

- `finite_extent(a, b)`: order endpoints and state whether the span is representable;
- `finite_midpoint(a, b)`: overflow-safe midpoint;
- `inverse_lerp(a, b, x)` and `lerp(a, b, t)`: scaled implementations that handle opposite-sign extremes;
- checked range expansion and endpoint addition;
- checked float-to-time and civil-time conversion;
- stable summation/moments where aggregation accuracy matters.

Decide the contract for an unrepresentable span:

- A fallible statistics or constructor API should return a typed error such as `NumericError::UnrepresentableSpan`.
- An infallible renderer should choose a deterministic degradation: clamp, skip, or use a neutral coordinate.
- A CLI must report the typed error, never expose a constructor panic.

Do not silently narrow all APIs to “reasonable” numbers unless that supported range is explicit in their documentation. Existing APIs generally say finite, so either support all finite operands or make the narrower domain a checked error.

**Test matrix**

Use deterministic bit-pattern cases around:

- `±f64::MAX` and opposite-sign extremes;
- smallest normal and subnormal values;
- adjacent representable endpoints;
- signed zero;
- degenerate domains;
- values immediately inside and outside supported calendar bounds.

For each primitive, assert one of two outcomes: a finite deterministic result or an intentional typed error. NaN, infinity, panic, and unbounded iteration are never implicit outcomes.

### F4 — Invariant ownership is duplicated and has drifted

**Evidence**

Public constructors enforce invariants locally, while `Mark::validate` re-encodes a second list in `src/mark/mod.rs:70-170`. The second list is incomplete:

- `Bars::spans` checks finite starts and positive finite widths in `src/mark/bars.rs:61-79`; `Mark::validate` does not.
- `Rule` constructors require finite orientation values in `src/mark/rule.rs:25-51`; validation does not.
- `Text` constructors require finite coordinates in `src/mark/text.rs:19-35`; validation does not.

A raw serialized `Bars` mark with span width 0 successfully deserialized and `plot.validate()` returned `Ok`. Consequently `Document` validation in `src/document.rs:167-186` accepts a value that public construction rejects, despite `SERDE.md` describing complete-payload validation.

There are similar policy gaps around domain ordering and an empty deserialized custom palette. Some paths intentionally degrade an empty palette; the question is not whether that fallback is safe, but whether strict document validation should call the payload valid.

**Recommendation**

Give every retained type one invariant owner:

1. Add a local checked constructor or `validate(&self)` to each mark payload.
2. Have public panicking convenience constructors call the checked form and document the panic.
3. Have `Mark::validate` dispatch to the payload method; it should not restate payload fields.
4. Have `Plot::validate` own only cross-object invariants such as channel compatibility and scale/mark relationships.
5. Have `Document` call the same validation graph after decoding.

Separate three concepts:

- **Syntactically decodable:** bytes can form the versioned retained representation.
- **Strictly valid:** all public construction invariants and cross-object invariants hold.
- **Render-tolerable:** even a merely decodable value can be rendered without panic, with malformed pieces shed.

That separation makes strict import and robust rendering compatible instead of asking one validator to perform both jobs.

**Tests to add**

Generate one malformed serialized case for every constructor condition and assert:

- direct checked construction rejects it;
- strict document validation rejects it with the same error class;
- infallible rendering does not panic.

A constructor/validator parity table can be maintained as a test macro so new fields cannot quietly create a third policy.

### F5 — Categorical color should be a channel, not synthetic layers

**Evidence**

`categorize` in `src/plot/resolve.rs:517-541` finds each label by a linear scan of the distinct-label list. This is `O(n * k)` for `n` rows and `k` categories. The implementation relies on a comment that category counts are “legend-sized,” but no API or CLI boundary enforces that assumption.

`expand_color_by` in `src/plot/resolve.rs:560-676` then creates a full-length, NaN-masked copy for every category:

- points and bars retain roughly `k` arrays of length `n`;
- range-like marks can retain several arrays per category;
- lines rebuild a mask and invoke M4 for each category.

Unique IDs supplied through CLI `--by` are enough to make this path quadratic in work and potentially quadratic in retained values. Existing geometry/frame resource limits do not bound category cardinality or this pre-draw expansion.

The masking scheme also turns category membership into fake numeric gaps, coupling category resolution to M4's weakest behavior.

**Recommendation**

Introduce one internal category representation:

```text
Categories {
    labels: Vec<String>,       // stable first-seen order
    ids: Vec<CategoryId>,      // one compact ID per datum
}
```

Build it once with stable hash interning. Store each distinct label once and reuse the IDs across marks, aggregations, legend construction, and CLI grouping.

Then:

- draw points/bars/ranges by looking up the palette entry per datum;
- define line category transitions explicitly as path boundaries or colored segments;
- generate legend entries from the label table;
- keep resolved geometry independent of legend expansion;
- preserve the current version-1 document wire form with a serialization adapter if compatibility requires it.

This changes complexity toward `O(n + k)` and gives category transitions a direct semantic meaning. As an interim defense, reject or cap unreasonable category cardinality at untrusted CLI/document boundaries, but do not treat a cap as the final architecture.

**Measure before and after**

Add benchmarks for 100k rows with 5, 100, and 100k distinct categories. Track peak allocations, not just runtime. The unique-category case should grow linearly and terminate with a deliberate policy.

### F6 — CLI input boundaries accept invalid values

#### Civil time

The parser in `cli/src/time.rs:38-54` checks a day range of 1–31 but does not validate the day against the month or leap year. A reproduced input of `2024-02-31` was normalized to a March timestamp instead of becoming a gap/error.

Timezone splitting in `cli/src/time.rs:59-76` does not bound timezone hours and minutes. `+99:99` was accepted and converted.

An extreme year such as `9223372036854775807-01-01` panics in the Gregorian conversion near `cli/src/time.rs:118` because intermediate era arithmetic is unchecked.

The core scale and CLI contain related civil-time calculations, increasing the chance of policy drift.

#### Column selectors

`cli/src/input.rs:49-55` accepts any numeric selector before comparing it with table width. An out-of-range numeric `--cols 999` exits successfully, produces an empty series, and reports values as unparseable. A missing named column correctly errors, so selector forms have inconsistent semantics.

**Recommendation**

- Create one checked `CivilDateTime -> UnixSeconds` implementation with leap-year/month-length checks, timezone bounds, checked year arithmetic, and a documented supported range.
- Use it from both core time ticks and CLI parsing where practical. If feature boundaries prevent sharing code, share exhaustive conformance vectors.
- Treat a malformed time value according to the existing row policy—gap or diagnostic—but never normalize impossible dates or panic.
- Resolve numeric and named selectors through one schema-aware path. Numeric selectors must be inside the widest/header-defined table width; ragged individual rows may still produce missing cells.
- Fuzz civil time and selector parsing because both accept untrusted stdin and have compact grammars.

### F7 — Reducer execution conflates streaming summaries with batch statistics

**Evidence**

`Reducer::reduce` in `src/stat/reducer.rs:39-63` first copies all finite values even for count, sum, mean, minimum, and maximum. Those reducers need constant state.

`Window` in `src/stat/window.rs:29-64` repeatedly reduces a trailing slice, producing `O(n * w)` work and repeated allocation for a window of width `w`.

`binned` in `src/stat/bin.rs:320-331` builds `Vec<Vec<_>>` buckets, after which the reducer copies each bucket again. `Agg` in `src/stat/agg.rs:47-75` uses linear key lookup, stores all observations, and then repeats reducer work.

KDE in `src/stat/kde.rs:15-36` duplicates quantile/moment logic rather than reusing the project's quantile and online-moment concepts.

**Conceptual issue**

The public `Reducer` enum is a useful user choice, but it is not one execution model:

- count/sum/mean/min/max are streaming accumulators;
- median/percentile are order statistics that generally retain samples;
- windowed execution has incremental algorithms distinct from one-shot reduction;
- KDE, ECDF, and LTTB are batch transforms, not reducers or mergeable accumulators.

**Recommendation**

Compile `Reducer` into a private execution strategy:

- streaming accumulator for count/sum/mean/min/max;
- sample-buffer strategy for median/percentile;
- prefix count/sum for rolling count/sum/mean;
- monotonic deques for rolling min/max;
- explicit buffered fallback for rolling order statistics.

Bins should own one accumulator per bin when the reducer streams, not one vector per bin. Aggregation should use stable key interning and the same accumulator strategy. Reuse a numerically stable moment/quantile implementation in KDE.

Avoid exposing a large trait framework unless another implementation actually needs it. A private enum with specialized state is likely simpler and easier to optimize.

### F8 — Rendering orchestration is duplicated across target types

**Evidence**

`Plot::try_rasterize_hybrid` in `src/plot/plot.rs:543-647` and the cell-oriented `try_rasterize_with` in `src/plot/plot.rs:672-765` repeat:

- target/canvas setup;
- configuration extraction;
- extent probing;
- layout construction;
- target-width-aware resolution;
- chrome and layer drawing.

The individual mark matches in resolution and drawing are not inherently a problem: they make the closed mark grammar visible. The higher-level pipeline duplication is where policy can drift.

**Recommendation**

Extract a small prepared-render pipeline:

```text
validated/tolerable Plot
        |
        v
RenderRequest + TargetPolicy
        |
        v
probe -> Layout -> resolved scene
        |
        v
Canvas drawing
```

`TargetPolicy` should contain only the real differences: cell/pixel density, coordinate conversion, marker cycling, and target construction. A `DrawContext` can group layout, scales, palette, and clipping state if it reduces repeated long parameter lists.

Do **not** introduce a parallel “mark metadata” tree. `BENCHMARKS.md` records that such a design was measured and rejected; the compact `ResolvedLayer` probe was faster and kept one source of truth. Extract the orchestration around the current probe rather than undoing that result.

Keep allocation and raster snapshot benchmarks around this refactor. Structural deduplication is valuable only if it preserves the deliberately optimized resolution path.

### F9 — The CLI needs one chart recipe

**Evidence**

`cli/src/main.rs:69-91` builds a runtime chart before checking whether the caller requested emitted Rust source. `emit::program` then independently reparses/transforms the same table.

`cli/src/chart.rs` and `cli/src/emit.rs` duplicate command dispatch, histogram construction, and plot furniture. Emission tests establish that generated programs compile, but compilation does not prove that runtime rendering and emitted source represent the same chart semantics.

The input representation also multiplies ownership: raw input, owned table fields, parsed numeric columns, and cloned series can coexist. `series::dataset` in `cli/src/series.rs:150-176` clones shared x data for multi-y charts.

**Recommendation**

Parse arguments and data once into a typed `Recipe`:

```text
Args + Table
     |
     v
Recipe / PreparedChart
   /                 \
into_plot()       to_rust_source()
```

The recipe should carry semantic operations—selected channels, transforms, grouping, labels, scales—not generated Rust tokens or fully cloned marks. Both backends consume it. Plot furniture should be applied once as recipe data.

As a minimal first step, branch to code emission before building a runtime `Plot` and extract shared histogram/selection logic. Later, make the table columnar or borrowing where it materially reduces peak memory, and share repeated x channels until final mark construction.

Add fixture tests that compare a normalized semantic description from both backends. Pixel equality is unnecessarily brittle; equivalent marks, transforms, scales, and furniture are the right oracle.

### F10 — Fallibility does not have a predictable public vocabulary

**Evidence**

Several `_with` functions return `Result` but remain partially panicking:

- `heatmap_with` checks colormap configuration, then calls the panicking `Cells::matrix` in `src/presets.rs:503-515`.
- `trend_with` returns `Result` but still documents a panic for unequal channels.
- `hist2d` delegates to a checked function and uses `expect` for some invalid states.
- `hist` can panic for extreme finite input without a corresponding documented numeric-domain restriction.

The suffix `_with` usually communicates configurability, not whether all data errors are returned. Callers therefore cannot derive the safety contract from the name or return type.

**Recommendation**

Adopt one layering without breaking existing 1.x callers:

- `try_*` and `try_*_with` are fully fallible for data and configuration errors;
- convenience `*` and `*_with` wrappers call the checked core and explicitly document any panic;
- infallible rendering remains a separate robustness boundary and degrades malformed retained values.

Add checked APIs first, deprecate ambiguous behavior only on the project's normal compatibility schedule, and use one error taxonomy for shape, numeric-domain, allocation, and configuration failures.

### F11 — The statistical vocabulary should describe actual algebra

`src/stat/mod.rs:1-7`, `TERMINOLOGY.md:58-72`, and related overview text describe every aggregator/statistic as a mergeable monoid. That is a useful design ambition for `Moments`, `Fit`, bins, and M4, but it does not accurately describe every public item in `stat`:

- `Window`, KDE, ECDF, and LTTB are batch transforms;
- median/percentile reduction normally retains a sample multiset;
- `Agg` is orchestration over keys rather than itself one scalar summary;
- M4 composition is ordered and currently fails its gap law.

**Recommendation**

Use three terms:

- **online accumulator:** has update, ordered/unordered merge as explicitly stated, and finish;
- **reducer:** maps a finite collection to one result and may buffer;
- **batch transform:** maps a series/table to another series/table.

State identity, associativity, commutativity, ordering requirements, and approximation separately. This makes test laws obvious and prevents an attractive abstraction from hiding materially different algorithms.

### F12 — CI and dependency maintenance have small gaps

The CI surface is stronger than typical: format, clippy, default/all-feature tests, Windows, documentation warnings, generated docs, MSRV, semver, dependency policy, allocation limits, packaging, and scheduled terminal-reply fuzzing are all represented.

Two improvements are worthwhile:

1. Demos are workspace members and contain 13 tests, but the current split CI commands rely on default members/core and separate CLI execution. Add an explicit demo/workspace test job so those tests cannot regress unnoticed.
2. `cargo deny check` passes but reports stale/unencountered license allowances and dependency duplication. The direct dev dependency on `crossterm 0.28` differs from `ratatui`'s `crossterm 0.29`, contributing to duplicate terminal/system dependency versions. Aligning them is likely low-risk after testing. Other duplicates such as proc-macro crates may be legitimate transitive/dev splits.

Fuzzing currently targets the terminal reply parser. The highest-value additions are:

- raw document/plot decode followed by infallible render;
- CLI civil-time parsing;
- delimiter/header/selector parsing;
- M4 ordered partition and gap sequences.

## A simpler target model

The project does not need a rewrite. Four small shared concepts would remove most of the accidental complexity.

### 1. Validated local values

Each mark, scale, palette, and document type owns its invariants. Construction, deserialization validation, and CLI conversion all call the same checked functions.

```text
external bytes ----> decodable value ----> strict validation
                              |                    |
                              |                    v
                              +------------> valid Plot
                              |
                              +------------> tolerant render
```

Strict validity and rendering totality remain deliberately separate.

### 2. Numeric-domain primitives

A small module owns robust spans, interpolation, midpoints, endpoint expansion, and civil-time conversion. Algorithms stop inventing local answers to overflow. Fallible APIs return one typed domain error; rendering has explicit deterministic fallbacks.

### 3. First-class channels

Numeric channels remain shared slices/owned vectors as appropriate. Categories become stable compact IDs plus one label dictionary. Marks do not become `k` synthetic layers merely to carry color, and gaps remain path topology rather than NaN-based control flow.

### 4. Execution plans at real boundaries

- A private reducer strategy separates streaming from buffered statistics.
- A prepared render plan shares target-independent orchestration but retains the measured compact probe.
- A CLI recipe shares semantic chart construction between runtime and source backends.

These are useful plans because each has two or more real consumers. Avoid adding generic traits where there is only one implementation.

## Recommended implementation sequence

### Phase 0 — Pin the contracts

Before structural work, add failing regression/law tests for:

1. malformed ragged marks above and below reduction thresholds;
2. extreme finite linear/bin/time domains;
3. M4 multiple gaps, invalid x, and arbitrary ordered partitions;
4. constructor versus document-validation parity;
5. invalid civil dates/timezones/extreme years;
6. out-of-range numeric column selectors;
7. unique-category resource growth.

This phase should change no output except where the current output violates a stated contract.

### Phase 1 — Restore totality and correctness

- Make mapped M4 pair-safe and path-segment aware.
- Add bounded checked time iteration.
- Introduce the first robust numeric helpers and migrate Linear, Bins/Bins2, quantile, and histogram construction.
- Move mark invariants to their payload types and make `Document` strict.
- Ensure CLI errors are returned rather than reaching `expect`/panicking constructors.

This phase is the release-critical work.

### Phase 2 — Remove expansion and repeated allocation

- Add `Categories` interning and stop expanding categorical marks.
- Compile reducers into streaming or buffered strategies.
- Give bins and aggregation per-group accumulator state.
- Add incremental fast paths for common rolling reducers.
- Reuse moment and quantile logic in KDE.

Measure peak allocation and output equivalence after each step.

### Phase 3 — Unify orchestration

- Extract the shared prepared-render pipeline around the existing compact probe.
- Introduce the CLI `Recipe` and make runtime/source output consume it.
- Add fully fallible `try_*` preset/mark construction and route convenience APIs through it.

Do not combine this phase with correctness changes; keeping output-stable refactors separate makes review much easier.

### Phase 4 — Tighten claims and automation

- Update terminology to distinguish accumulators, reducers, and batch transforms.
- Turn core promises into property/law suites.
- Add demo CI and the focused fuzz targets.
- Align the direct `crossterm` dev dependency and prune stale `cargo-deny` allowances.

## Suggested test laws

The project already has many example-based tests. The next improvement is to encode architectural promises as reusable laws.

| Promise | Test law |
|---|---|
| Infallible rendering | Every decodable retained value renders at representative frame sizes without panic or unbounded work |
| Constructor invariants | Checked constructor acceptance equals strict deserialized validation |
| Mergeable accumulator | Sequential summary equals every valid ordered partition; identity and associativity hold |
| M4 path fidelity | Reduction never introduces connectivity absent from the input |
| Target equivalence | Small/raw and large/reduced paths agree on connectivity and visible envelope |
| Numeric totality | Supported finite operands yield finite results; unsupported domains yield typed errors |
| Category scalability | Runtime and allocations are linear in `n + k` within a documented tolerance |
| CLI backend equivalence | Runtime Plot and emitted source normalize to the same recipe semantics |
| Resource bounds | User-controlled dimensions, categories, ticks, and encoded output terminate under explicit caps |

For floating-point property tests, generate raw bit patterns and classify expected domains rather than using only comfortable decimal ranges. For merges, generate arbitrary ordered partition trees rather than only one split.

## What not to change

Several tempting “cleanups” would make this code worse:

- Do not split large files solely to lower line counts. `src/presets.rs` and the main plot modules should be divided only where a stable concept gains an owner.
- Do not replace exhaustive mark matches with a deep trait-object hierarchy. The closed grammar and compiler-checked matches are an advantage.
- Do not reintroduce a parallel mark metadata model; the repository documents the benchmark evidence against it.
- Do not hide panics with broad `catch_unwind` or silently clamp every error. Boundary behavior should be explicit and locally tested.
- Do not add a large date/time, dataframe, or statistics dependency without comparing binary size, MSRV, compile time, and the narrow functionality required.
- Do not optimize M4 by weakening gap semantics without documenting the tradeoff. A false connection is a correctness error, not merely a sampling approximation.
- Do not make all rendering fallible just because malformed values exist. The split between strict validation and tolerant rendering is useful when it is consistently implemented.

## Positive assessment

The audit found no general code-health crisis. There are no production `TODO`/`FIXME` placeholders or unfinished stubs driving the recommendations. Most modules use clear domain names, tests sit near their behavior, documentation explains non-obvious design choices, and the project treats compatibility and resource usage seriously.

The strongest qualities to preserve are:

- a small, legible retained plotting vocabulary;
- generic drawing without unsafe code;
- deterministic output and extensive snapshots/frame sweeps;
- explicit allocation and geometry limits;
- versioned documents with golden compatibility fixtures;
- hardened terminal control handling;
- restrained features/dependencies and an enforced MSRV;
- benchmark notes that record rejected designs, not just successful ones.

The project is therefore in a good position for a quality-focused release. The work is less about adding abstractions than making five existing concepts exact: validity, totality, numeric domain, path continuity, and categorical identity.

## Reproduction notes

The following results were observed directly during this audit:

| Probe | Observed result |
|---|---|
| M4 one bucket over `[0, NaN, 10, NaN, 0]` | `[0, 10, NaN, 0]`; first disconnection lost |
| M4 merge where right partition begins with a gap | Gap moved before the left partition's final sample |
| Deserialized line with x length 1 and y length 1,000 | Panic at `src/stat/m4.rs:272` during render |
| Linear domain `[-MAX, MAX]` mapped at `MAX` | NaN |
| Two-bin x domain with values `[-MAX, MAX]` | Counts `[2, 0]` instead of `[1, 1]` |
| `kaz hist` over `-1e308` and `1e308` | Process exits 101 after a constructor panic |
| Time input `2024-02-31` | Accepted and normalized into March |
| Timezone `+99:99` | Accepted |
| Extreme signed year | Arithmetic panic in CLI civil-date conversion |
| Numeric selector `--cols 999` | Successful empty plot plus parse warnings, not a selector error |

These probes intentionally target contracts and boundary transitions, not representative everyday data. That is precisely why they are useful: ordinary test fixtures already pass, while the failures identify where the implementation model changes underneath a public promise.

## Final recommendation

Treat F1–F4 as one correctness program, not four isolated bug fixes. A total renderer cannot be achieved while validation drifts, M4 invents connections, and finite-domain arithmetic can manufacture non-finite intermediates. Establish shared invariant and numeric boundaries first, then repair M4 against law tests.

Next, make categories a real channel and reducers real execution strategies. Those changes remove the largest avoidable complexity and resource multipliers. Only after those semantics are stable should the render and CLI pipelines be deduplicated.

That sequence produces cleaner code because it reduces the number of concepts the implementation must simulate:

- one invariant path instead of constructor plus validator copies;
- one category vector instead of one masked layer per label;
- one reducer choice compiled to the right state instead of always collecting;
- one render orchestration with two target policies;
- one chart recipe with two output backends.

The result should be both simpler and more truthful without sacrificing the project's existing performance discipline.
