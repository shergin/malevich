import { useEffect } from "react";
import { useStdin, useStdout } from "ink";
import { disableMouse, enableMouse, parseMouse } from "./mouse.js";
import { PlotState } from "./PlotState.js";

export type PlotInteractionOptions = {
  /** Called when a gesture or key binding changed state — bump a render. */
  onChange?: () => void;
  /** Enable DECSET mouse tracking. Default true. */
  mouse?: boolean;
  /**
   * Bind `+`/`=` zoom in, `-` zoom out, arrows pan, `r` reset.
   * Default true. `q`/`escape` are left to the host.
   */
  keys?: boolean;
};

/**
 * Proven composition: enable mouse tracking on Ink's stdout, parse SGR
 * mouse (and the default key bindings) from Ink's stdin, and feed them to
 * a `PlotState`. The widget still never reads the terminal — this hook is
 * host code you can skip and replace.
 *
 * Keep `state` in a `useRef` (it is mutable, like ratatui's `PlotState`)
 * and re-render from `onChange`.
 */
export function usePlotInteraction(
  state: PlotState,
  options: PlotInteractionOptions = {},
): void {
  const { onChange, mouse = true, keys = true } = options;
  const { setRawMode, isRawModeSupported, internal_eventEmitter } = useStdin();
  const { stdout } = useStdout();

  useEffect(() => {
    if (!isRawModeSupported) {
      return;
    }
    setRawMode(true);
    if (mouse) {
      enableMouse(stdout);
    }
    return () => {
      if (mouse) {
        disableMouse(stdout);
      }
      setRawMode(false);
    };
  }, [isRawModeSupported, mouse, setRawMode, stdout]);

  useEffect(() => {
    const handle = (data: string) => {
      const event = parseMouse(data);
      if (event) {
        if (state.onMouse(event)) {
          onChange?.();
        }
        return;
      }
      if (!keys) {
        return;
      }
      if (applyKey(state, data)) {
        onChange?.();
      }
    };
    internal_eventEmitter.on("input", handle);
    return () => {
      internal_eventEmitter.removeListener("input", handle);
    };
  }, [internal_eventEmitter, keys, onChange, state]);
}

function applyKey(state: PlotState, data: string): boolean {
  switch (data) {
    case "+":
    case "=":
      return state.zoomIn();
    case "-":
      return state.zoomOut();
    case "\u001b[D":
      return state.panLeft();
    case "\u001b[C":
      return state.panRight();
    case "r":
    case "R":
      state.resetView();
      return true;
    default:
      return false;
  }
}
