import React from "react";
import { Box, Text } from "ink";
import type { Charset } from "../color.js";
import { Theme, type ThemeJSON } from "../color.js";
import { Frame } from "../frame.js";
import type { Plot } from "../plot.js";
import { inkColor } from "./color.js";
import { applyOverlays } from "./overlays.js";
import type { PlotState } from "./PlotState.js";

export type PlotWidgetProps = {
  plot: Plot;
  width?: number;
  height?: number;
  charset?: Charset;
  theme?: ThemeJSON;
  /** Terminal-cell origin of this widget. Default `{ column: 0, row: 0 }`. */
  origin?: { column: number; row: number };
  /** Interaction controller the host keeps between frames. */
  state?: PlotState;
  /** Tint the cursor's row and column (on by default, stateful only). */
  crosshair?: boolean;
  /** Write the cursor's data coordinates in the panel's top-right (on by default). */
  readout?: boolean;
  /** Snap the cursor to the nearest datum on Line/Points layers (on by default). */
  snap?: boolean;
};

/**
 * An Ink widget that paints a plot as a `Box` of `Text` rows.
 *
 * Size comes from the parent (or `width`/`height` props), not `Frame.detect`.
 * Pass `state` for a stateful render: the widget applies the viewport, caches
 * the mapping, and draws the interaction chrome. It never reads input — the
 * host feeds `PlotState.onMouse` (directly, or via `usePlotInteraction`).
 */
export function PlotWidget({
  plot,
  width = 80,
  height = 16,
  charset = "Quadrants",
  theme = Theme.DARK,
  origin = { column: 0, row: 0 },
  state,
  crosshair = true,
  readout = true,
  snap = true,
}: PlotWidgetProps): React.ReactElement {
  const frame = new Frame({
    width,
    height,
    charset,
    color: "TrueColor",
    theme,
  });
  const drawn = state ? plot.viewport(state.viewport()) : plot;
  let raster = drawn.raster(frame);
  if (state) {
    state.capture(drawn.mapping(frame), {
      column: origin.column,
      row: origin.row,
      width,
      height,
    });
    raster = applyOverlays(raster, state, plot, { crosshair, readout, snap }, isLight(theme));
  }
  const rows = raster.rows();
  return (
    <Box flexDirection="column">
      {rows.map((row, index) => (
        <Text key={index}>
          {row.map((cell, column) => (
            <Text
              key={column}
              color={inkColor(cell.foreground)}
              backgroundColor={inkColor(cell.background)}
            >
              {cell.glyph}
            </Text>
          ))}
        </Text>
      ))}
    </Box>
  );
}

function isLight(theme: ThemeJSON): boolean {
  return theme.palette[theme.palette.length - 1] === "Black";
}
