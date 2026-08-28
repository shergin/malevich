# The frame is run state

A plot describes a chart; a frame describes one rendering of it. The spec
never learns where it will be drawn.

## Why

When the terminal lives inside the chart object, everything downstream pays.
The plot cannot be built on one thread and rendered on another; it cannot be
rendered twice at two sizes; it cannot be serialized without lying about a
file descriptor; tests need a fake terminal instead of a string comparison.
Global registries and ambient configuration follow, because once one hidden
input is allowed, each new feature wants its own.

The subtler failure is a render that reads the environment. A function that
consults `$TERM` on every call is not reproducible, and a snapshot test
against it pins the CI machine, not the library.

## The idea

Split description from execution. The plot is layers, scales, and furniture —
data, complete and serializable. The frame is where and how to render: width
and height in cells, charset, color mode, theme. Rendering is a pure function
of the two values; call it with a different frame and the same spec resolves
its domains, places its ticks, and lays out its furniture again, for that
frame.

Because nothing in the plot references a terminal, a thread, or a global,
`Send + Sync` falls out of the design rather than being bolted on. Build in a
worker, render in the UI; snapshot-test the string; ship the spec over a
socket and render it on the other side.

Environment reading is not banned — it is named. Detection lives in explicit
conveniences that construct values: a detected frame, a detected graphics
choice. Those values then drive pure calls. The documented boundary is the
constructor, never the render.

Errors follow the same split. Construction panics on documented programmer
invariants, at the caller's line. Rendering never fails: it sheds what it
cannot draw, because a dashboard must survive a small terminal. A spec that
arrives from data — deserialization, configuration — gets the checked twins,
which report the first problem as a typed error instead.

## Consequences

- One spec renders concurrently at many sizes without locks; live and TUI
  code snapshots the value and renders on its own schedule.
- Every render path is snapshot-testable with a fixed frame; determinism is
  the default, not a test mode.
- Serialization serializes everything there is; no field is "except this one,
  which is a handle."
- Detection results are inspectable values a caller can log, cache, or
  override — not side effects inside a render.
- The library never owns the terminal: no raw mode, no event loop, no
  cleanup obligations on panic. In-place repaint is one buffered write.

## Not this

- A `Chart::show()` that grabs stdout, or a plot holding a writer.
- Rendering that consults environment variables, locale, or terminal size
  directly.
- A global theme, palette registry, or default-size setting.
- Panicking at render time because the terminal is small. Shedding is the
  contract; panics belong to construction.

See [Degradation is the contract](degradation-is-the-contract.md) for what
shedding means, and [Vision](../vision.md) rule 1.

## Spelled today

`Plot` is the spec (`Clone + Send + Sync`); `Frame` is the run state, with
`Frame::detect` the environment-reading constructor and `Frame::plain` /
`Frame::portable` the deterministic forms. `Plot::render` is the pure
function; `Plot::validate` and `Plot::try_render` are the checked twins for
foreign specs. In the `pixel` feature, `Capabilities::detect_for` constructs
the detection value and `render_with_capabilities` consumes it purely;
`render_best` is the documented stdout convenience. This section may rot; the
rest must not.
