import type { Color } from "../color.js";

/** Map a malevich color onto an Ink `Text` color prop. */
export function inkColor(color: Color): string | undefined {
  if (color === "Default") {
    return undefined;
  }
  if (typeof color === "string") {
    const bright = color.startsWith("Bright");
    const base = bright ? color.slice(6) : color;
    const name = base.toLowerCase();
    if (name === "black" && bright) {
      return "gray";
    }
    return name;
  }
  if ("rgb" in color) {
    const [r, g, b] = color.rgb;
    return `#${hex(r)}${hex(g)}${hex(b)}`;
  }
  return undefined;
}

function hex(value: number): string {
  return value.toString(16).padStart(2, "0");
}
