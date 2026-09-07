/**
 * Interactive Ink analog of `cargo run --release --example zoom --features ratatui`.
 *
 * Wheel zoom · left-drag pan · right-drag rubber-band · +/− zoom · arrows pan
 * · r reset · q quit.
 *
 *   cd js && npm run build && npm run example:zoom
 */

import React, { useMemo, useRef, useState } from "react";
import { Box, Text, render, useApp, useInput } from "ink";
import { Color, Line, Plot } from "../dist/index.js";
import { PlotState, PlotWidget, usePlotInteraction } from "../dist/ink/index.js";

function signal(n: number): { x: Float64Array; y: Float64Array } {
  const x = new Float64Array(n);
  const y = new Float64Array(n);
  let lcg = 0x2545f4914f6cdd1dn;
  for (let i = 0; i < n; i++) {
    const t = i / n;
    lcg = BigInt.asUintN(64, lcg * 6364136223846793005n + 1442695040888963407n);
    const noise = Number(lcg >> 33n) / 0x80000000 - 1;
    const spike = lcg % 1_000_003n < 2n ? 6 : 0;
    x[i] = i;
    y[i] =
      Math.sin(t * 12 * Math.PI * 2) * 2 +
      Math.sin(t * 397 * Math.PI * 2) * 0.6 +
      noise * 0.25 +
      spike;
  }
  return { x, y };
}

function App({ n }: { n: number }): React.ReactElement {
  const { exit } = useApp();
  const state = useRef(new PlotState()).current;
  const [, bump] = useState(0);
  const { x, y } = useMemo(() => signal(n), [n]);
  const cols = process.stdout.columns ?? 80;
  const rows = process.stdout.rows ?? 24;

  usePlotInteraction(state, { onChange: () => bump((v) => v + 1) });
  useInput((input, key) => {
    if (input === "q" || key.escape) {
      exit();
    }
  });

  const window = state.viewport().x;
  const showing = window ? `${window[0].toFixed(0)}..${window[1].toFixed(0)}` : "all";
  const plot = new Plot()
    .layer(Line.xy(x, y).color(Color.Cyan))
    .title(
      `${n} points, showing ${showing} — wheel zoom · drag pan · right-drag zoom · r reset · q quit`,
    );

  return (
    <Box flexDirection="column">
      <PlotWidget
        plot={plot}
        state={state}
        width={cols}
        height={Math.max(8, rows - 1)}
        charset="Braille"
      />
      <Text dimColor>
        +/− zoom   arrows pan   r reset   q quit
      </Text>
    </Box>
  );
}

const n = Number(process.argv[2] ?? 2_000_000) || 2_000_000;
render(<App n={n} />);
