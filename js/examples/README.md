# examples

TypeScript tours of the JS rim. They live **in this repo**, next to the
package, the same way `examples/` sits next to the Rust crate. One engine,
one golden suite — not a second website, not a separate package.

Pixel side-by-side (cells vs sixel/kitty) is Phase 4 of [`notes/js.md`](../../notes/js.md).
Until then these render cells, sized to your terminal via `Frame.detect()`.

```sh
npx malevich              # published package: a tour
printf '1 5 2 8' | npx malevich line

cd js
npm install
npm run build
npm run showcase          # the analog of `cargo run --example showcase`
npx tsx examples/loss.ts
npx tsx examples/languages.ts
npx tsx examples/distribution.ts
node examples/hello.mjs   # the one-liner
npm run example:ink       # fire-and-forget Ink widget
npm run example:zoom      # interactive: wheel / drag / rubber-band
npm run example:linked    # two panes, shared x, mirrored crosshair
```

| file | what |
|---|---|
| `showcase.ts` | the full tour: loss, describe, glow, 15k density, calendar axis, 10M points, bars, hist, boxes, violins, KDE, ECDF, trend, heatmap, hist2d, color-by, grid |
| `loss.ts` | two series, a target rule, an annotation |
| `languages.ts` | categorical bars |
| `distribution.ts` | hist / density / ecdf / box / violin |
| `hello.mjs` | the README one-liner |
| `ink-hello.tsx` | the same plot as a fire-and-forget Ink widget |
| `ink-zoom.tsx` | analog of `cargo run --example zoom --features ratatui`: 2M points, the full gesture grammar |
| `ink-linked.tsx` | two stacked panes sharing an x window and a mirrored crosshair |

The Ink examples never let the widget own the terminal. `ink-zoom.tsx` uses
`usePlotInteraction` (the proven composition). `ink-linked.tsx` parses SGR
itself and calls `linkX` — the pattern, not a feature.
