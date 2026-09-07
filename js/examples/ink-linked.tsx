/**
 * Two stacked panes sharing an x window and a mirrored crosshair — the
 * linked-panes pattern from docs/interaction.md, as an Ink app.
 *
 * Route the mouse to the pane it landed on, then `linkX(active, passive)`.
 * There is no linking feature: a view is a value, a hover is a data x.
 *
 *   cd js && npm run build && npm run example:linked
 */

import React, { useEffect, useMemo, useRef, useState } from "react";
import { Box, Text, render, useApp, useInput, useStdin, useStdout } from "ink";
import { Color, Line, Plot } from "../dist/index.js";
import {
  PlotState,
  PlotWidget,
  disableMouse,
  enableMouse,
  linkX,
  parseMouse,
} from "../dist/ink/index.js";

function series(n: number, phase: number): { x: Float64Array; y: Float64Array } {
  const x = new Float64Array(n);
  const y = new Float64Array(n);
  for (let i = 0; i < n; i++) {
    x[i] = i;
    y[i] = Math.sin(i / 40 + phase) * 2 + Math.cos(i / 9) * 0.4;
  }
  return { x, y };
}

function App(): React.ReactElement {
  const { exit } = useApp();
  const main = useRef(new PlotState()).current;
  const context = useRef(new PlotState()).current;
  const [, bump] = useState(0);
  const redraw = () => bump((v) => v + 1);
  const cols = process.stdout.columns ?? 80;
  const rows = process.stdout.rows ?? 24;
  const header = 1;
  const mainHeight = Math.max(8, Math.floor((rows - header - 1) * 0.65));
  const ctxHeight = Math.max(6, rows - header - mainHeight - 1);
  const { x, y: y1 } = useMemo(() => series(400, 0), []);
  const { y: y2 } = useMemo(() => series(400, 1.2), []);

  const { setRawMode, isRawModeSupported, internal_eventEmitter } = useStdin();
  const { stdout } = useStdout();

  useEffect(() => {
    if (!isRawModeSupported) {
      return;
    }
    setRawMode(true);
    enableMouse(stdout);
    return () => {
      disableMouse(stdout);
      setRawMode(false);
    };
  }, [isRawModeSupported, setRawMode, stdout]);

  useEffect(() => {
    const handle = (data: string) => {
      const event = parseMouse(data);
      if (!event) {
        return;
      }
      const inMain = inside(main, event.column, event.row);
      const inCtx = inside(context, event.column, event.row);
      const active = inMain ? main : inCtx ? context : undefined;
      const passive = active === main ? context : active === context ? main : undefined;
      if (!active || !passive) {
        return;
      }
      if (active.onMouse(event)) {
        linkX(active, passive);
        redraw();
      }
    };
    internal_eventEmitter.on("input", handle);
    return () => {
      internal_eventEmitter.removeListener("input", handle);
    };
  }, [context, internal_eventEmitter, main]);

  useInput((input, key) => {
    if (input === "q" || key.escape) {
      exit();
      return;
    }
    if (input === "r") {
      main.resetView();
      context.resetView();
      redraw();
    }
  });

  const mainPlot = new Plot()
    .layer(Line.xy(x, y1).color(Color.Cyan).label("train"))
    .title("train — hover either pane; the other mirrors x");
  const ctxPlot = new Plot()
    .layer(Line.xy(x, y2).color(Color.Yellow).label("val"))
    .title("val");

  return (
    <Box flexDirection="column">
      <Text dimColor>linked x · wheel zoom · drag pan · r reset · q quit</Text>
      <PlotWidget
        plot={mainPlot}
        state={main}
        origin={{ column: 0, row: header }}
        width={cols}
        height={mainHeight}
        charset="Braille"
      />
      <PlotWidget
        plot={ctxPlot}
        state={context}
        origin={{ column: 0, row: header + mainHeight }}
        width={cols}
        height={ctxHeight}
        charset="Braille"
      />
    </Box>
  );
}

function inside(state: PlotState, column: number, row: number): boolean {
  const rect = state.plotArea();
  if (!rect) {
    return false;
  }
  return (
    column >= rect.column &&
    column < rect.column + rect.width &&
    row >= rect.row &&
    row < rect.row + rect.height
  );
}

render(<App />);
