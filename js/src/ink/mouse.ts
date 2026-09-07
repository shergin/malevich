import type { Mouse, MouseButton } from "./PlotState.js";

const SGR = /^\u001b\[<(\d+);(\d+);(\d+)([Mm])$/;
const ENABLE = "\u001b[?1000h\u001b[?1002h\u001b[?1003h\u001b[?1006h";
const DISABLE = "\u001b[?1006l\u001b[?1003l\u001b[?1002l\u001b[?1000l";

/**
 * DECSET mouse tracking: any-event (1003) + SGR coordinates (1006).
 * Write to the same stream Ink renders on; disable on the way out.
 * The widget never does this — the host, or `usePlotInteraction`, does.
 */
export function enableMouse(stream: { write(chunk: string): unknown }): void {
  stream.write(ENABLE);
}

export function disableMouse(stream: { write(chunk: string): unknown }): void {
  stream.write(DISABLE);
}

/**
 * Parses one SGR mouse sequence (`CSI < btn ; col ; row M/m`) into the
 * backend-neutral `Mouse` vocabulary. Coordinates become 0-based.
 * Returns `undefined` for anything that is not a mouse event.
 */
export function parseMouse(sequence: string): Mouse | undefined {
  const match = SGR.exec(sequence);
  if (!match) {
    return undefined;
  }
  const button = Number(match[1]);
  const column = Number(match[2]) - 1;
  const row = Number(match[3]) - 1;
  const release = match[4] === "m";
  if (column < 0 || row < 0) {
    return undefined;
  }

  if (button >= 64 && button < 68) {
    const kind =
      button === 64
        ? "scrollUp"
        : button === 65
          ? "scrollDown"
          : button === 66
            ? "scrollLeft"
            : "scrollRight";
    return { kind, column, row };
  }

  const motion = (button & 32) !== 0;
  const which = mouseButton(button & 3);
  if (motion && (button & 3) === 3) {
    return { kind: "moved", column, row };
  }
  if (release) {
    return { kind: "up", button: which ?? "left", column, row };
  }
  if (motion && which) {
    return { kind: "drag", button: which, column, row };
  }
  if (which) {
    return { kind: "down", button: which, column, row };
  }
  return { kind: "moved", column, row };
}

function mouseButton(code: number): MouseButton | undefined {
  if (code === 0) {
    return "left";
  }
  if (code === 1) {
    return "middle";
  }
  if (code === 2) {
    return "right";
  }
  return undefined;
}
