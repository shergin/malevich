# Vision

Malevich is terminal plotting for Rust: a small grammar of marks, honest axes,
millions of points. A plot is a plain value and rendering is a pure function of
that value and a frame. The first look at any data should happen in the
terminal — and be excellent — so that reaching for a browser or matplotlib
becomes a choice, not a necessity.

The goal is not the biggest chart catalog; it is the smallest vocabulary that
composes into one. Better a few strict rules than lots of features. One black
square on a plain ground beats a mural.

That means three commitments:

- **The whole basic chart catalog, from eight marks.** Marks × a statistics
  layer × shared scales compose into everything the catalog names. A chart
  type is a preset: a name for a grammar expansion, never a peer of the
  grammar.
- **Every claim provable.** The oracle is drawing every point; aggregation
  must match it pixel for pixel. A preset must match its expansion byte for
  byte. An advertised number has a bench behind it, and every chart in these
  docs is program output, verified in CI. A claim is one assert away from
  proof.
- **Every terminal answered.** Output degrades down a ladder — real pixels,
  octants, quadrants, ASCII; truecolor to plain — and never fails, never
  probes where escapes are unsafe, never owns the screen.

A chart library usually grows by accretion: one function per chart type, one
option per request. Malevich does not. The plot is written once, as data;
everything after it is a stage reading the same spec:

- Construction stacks layers on shared scales. The plot carries no terminal,
  no thread, no global — it is `Clone + Send + Sync` because there is nothing
  in it that could not be.
- Resolution unions the layers' domains, places ticks, and lays out furniture,
  per frame, at render time. Large layers reduce to the raster here,
  pixel-identically.
- Rasterization draws marks onto a subpixel surface — or onto real pixels
  where the terminal speaks them. The same mark code serves both fidelities.
- Encoding writes the surface as a `String`: glyphs and SGR for a terminal,
  spans for a notebook card, cells for a TUI buffer.

Adoption may follow; it is never chased. No chart-type races, no config
surface for its own sake, no dependency bazaar.

## The rules

Five rules, one axis each: what a chart means, what the core owns, what is
true, where it must hold, and how a claim earns its place.

1. **The plot is the spec.** A plot is layers, scales, and furniture — data,
   complete and serializable. Rendering is a pure function of plot and frame;
   the frame is run state, not plot state, so one spec renders concurrently at
   many sizes. Environment reading lives only in named conveniences, never
   inside render.
2. **The grammar is closed.** Eight marks, a statistics layer, four position
   scales. A feature must be a composition of existing concepts; a new concept
   must pay for itself across many features. Every preset is provably its own
   grammar expansion, so the front door never forks the grammar.
3. **The full draw is the truth.** The oracle is drawing every point.
   Aggregation reproduces its pixels exactly; extremes survive; `NaN` is a
   visible gap; out-of-range data clips rather than smears; quantization is
   disclosed. Nothing is sampled away silently.
4. **Every terminal is answered.** Rendering degrades down declared ladders
   and never fails: furniture sheds before data, ASCII always works, piped
   output is clean plain text, and no escape byte is written where it is not
   safe. The library never owns the terminal.
5. **Claims are measured; figures are output.** Every advertised number comes
   from the bench suite and is recorded with its machine and date. Every chart
   in the documentation is spliced program output — regenerated, diffed, and
   failed in CI when stale. No figure is typed by hand.

## Principles

Constraints the vision names without arguing. One file per principle; the
type names in each "Spelled today" section may rot, the rest must not.

- [Presets are packaging](principles/presets-are-packaging.md)
- [The frame is run state](principles/frame-is-run-state.md)
- [The full draw is the oracle](principles/full-draw-oracle.md)
- [What earns a concept](principles/what-earns-a-concept.md)
- [Degradation is the contract](principles/degradation-is-the-contract.md)
- [The axes are the product](principles/axes-are-the-product.md)
- [Conversion lives at the rim](principles/conversion-at-the-rim.md)

## The name

Kazimir Malevich painted a black square on a plain ground and meant it: a
small vocabulary of geometric forms, composed deliberately. That is the design
budget of this library.
