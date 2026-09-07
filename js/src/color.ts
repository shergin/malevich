export type NamedColor =
  | "Default"
  | "Black"
  | "Red"
  | "Green"
  | "Yellow"
  | "Blue"
  | "Magenta"
  | "Cyan"
  | "White"
  | "BrightBlack"
  | "BrightRed"
  | "BrightGreen"
  | "BrightYellow"
  | "BrightBlue"
  | "BrightMagenta"
  | "BrightCyan"
  | "BrightWhite";

export type Color = NamedColor | { ansi256: number } | { rgb: [number, number, number] };

const NAMED: Record<string, NamedColor> = {
  default: "Default",
  black: "Black",
  red: "Red",
  green: "Green",
  yellow: "Yellow",
  blue: "Blue",
  magenta: "Magenta",
  cyan: "Cyan",
  white: "White",
  brightblack: "BrightBlack",
  brightred: "BrightRed",
  brightgreen: "BrightGreen",
  brightyellow: "BrightYellow",
  brightblue: "BrightBlue",
  brightmagenta: "BrightMagenta",
  brightcyan: "BrightCyan",
  brightwhite: "BrightWhite",
};

export const Color = {
  Default: "Default" as const,
  Black: "Black" as const,
  Red: "Red" as const,
  Green: "Green" as const,
  Yellow: "Yellow" as const,
  Blue: "Blue" as const,
  Magenta: "Magenta" as const,
  Cyan: "Cyan" as const,
  White: "White" as const,
  BrightBlack: "BrightBlack" as const,
  BrightRed: "BrightRed" as const,
  BrightGreen: "BrightGreen" as const,
  BrightYellow: "BrightYellow" as const,
  BrightBlue: "BrightBlue" as const,
  BrightMagenta: "BrightMagenta" as const,
  BrightCyan: "BrightCyan" as const,
  BrightWhite: "BrightWhite" as const,
  ansi256: (index: number): Color => ({ ansi256: index }),
  rgb: (r: number, g: number, b: number): Color => ({ rgb: [r, g, b] }),
};

export function canonicalizeColor(color: string | Color): Color {
  if (typeof color !== "string") {
    return color;
  }
  const named = NAMED[color.replaceAll("_", "").toLowerCase()];
  if (!named) {
    throw new TypeError(`unknown color '${color}'`);
  }
  return named;
}

/** Wire form matching the crate's serde: unit strings, `{ "Ansi256": n }`, `{ "Rgb": [r,g,b] }`. */
export function colorToJSON(color: Color | null): unknown {
  if (color === null) {
    return null;
  }
  if (typeof color === "string") {
    return color;
  }
  if ("ansi256" in color) {
    return { Ansi256: color.ansi256 };
  }
  return { Rgb: color.rgb };
}

export type ColorMode = "Plain" | "Ansi16" | "Ansi256" | "TrueColor";

export type Charset =
  | "Ascii"
  | "HalfBlocks"
  | "Quadrants"
  | "Sextants"
  | "Octants"
  | "Braille";

export type ThemeJSON = { palette: NamedColor[] };

export const Theme = {
  DARK: {
    palette: ["Cyan", "Yellow", "Green", "Magenta", "Blue", "Red"],
  } satisfies ThemeJSON,
  LIGHT: {
    palette: ["Blue", "Red", "Green", "Magenta", "Cyan", "Black"],
  } satisfies ThemeJSON,
  detect(): ThemeJSON {
    const env = typeof process !== "undefined" ? process.env : {};
    const background = env.COLORFGBG?.split(";").at(-1);
    return background === "7" || background === "15" ? Theme.LIGHT : Theme.DARK;
  },
};

export type ScaleJSON = "Auto" | "Linear" | "Log" | "Time" | { Bands: string[] };

export function scaleToJSON(scale: "Auto" | "Linear" | "Log" | "Time" | { bands: string[] } | ScaleJSON): ScaleJSON {
  if (typeof scale === "string") {
    return scale;
  }
  if ("bands" in scale) {
    return { Bands: scale.bands };
  }
  return scale;
}
