/**
 * Fire-and-forget Ink widget. For the interactive grammar — hover, zoom,
 * pan, rubber-band — see `ink-zoom.tsx` and `ink-linked.tsx`.
 *
 *   cd js && npm run build && npx tsx examples/ink-hello.tsx
 */

import React from "react";
import { render } from "ink";
import { line } from "../dist/index.js";
import { PlotWidget } from "../dist/ink/index.js";

const loss = Array.from({ length: 40 }, (_, i) => 4 * Math.exp(-0.08 * i) + 0.4);
const chart = line(loss).title("training");

render(<PlotWidget plot={chart} width={process.stdout.columns ?? 80} height={16} />);
