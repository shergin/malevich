import React, { Children, cloneElement, isValidElement } from "react";
import { Box, Text } from "ink";
import { PlotWidget, type PlotWidgetProps } from "./PlotWidget.js";

export type PlotColumnProps = {
  children: React.ReactNode;
};

/**
 * Stacks children in a column and injects `origin` into each `PlotWidget`
 * from the heights above it. A header `Text` counts as one row; a widget
 * uses its `height` prop (default 16). Pass an explicit `origin` to skip.
 */
export function PlotColumn({ children }: PlotColumnProps): React.ReactElement {
  let row = 0;
  const stacked = Children.map(children, (child) => {
    if (!isValidElement(child)) {
      return child;
    }
    if (child.type === PlotWidget) {
      const widget = child as React.ReactElement<PlotWidgetProps>;
      const height = widget.props.height ?? 16;
      const origin = widget.props.origin ?? { column: 0, row };
      row += height;
      return cloneElement(widget, { origin });
    }
    row += guessHeight(child);
    return child;
  });
  return <Box flexDirection="column">{stacked}</Box>;
}

function guessHeight(child: React.ReactElement): number {
  const props = child.props as { height?: number; children?: React.ReactNode };
  if (typeof props.height === "number") {
    return props.height;
  }
  if (child.type === Text) {
    const text = typeof props.children === "string" ? props.children : "";
    return Math.max(1, text.split("\n").length);
  }
  return 1;
}
