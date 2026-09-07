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

/** Wire form of a categorical palette — `{ colors: [...] }` matching crate serde. */
export type PaletteJSON = { colors: unknown[] };

export const Palette = {
  OKABE_ITO: {
    colors: [
      { Rgb: [230, 159, 0] },
      { Rgb: [86, 180, 233] },
      { Rgb: [0, 158, 115] },
      { Rgb: [213, 94, 0] },
      { Rgb: [204, 121, 167] },
      { Rgb: [0, 114, 178] },
      { Rgb: [240, 228, 66] },
    ],
  } satisfies PaletteJSON,
  of(colors: Iterable<string | Color>): PaletteJSON {
    const list = [...colors].map((color) => colorToJSON(canonicalizeColor(color)));
    if (list.length === 0) {
      throw new TypeError("Palette.of requires at least one color");
    }
    return { colors: list };
  },
};

/** Wire form of a continuous colormap. */
export type ColormapJSON = {
  stops: [number, number, number][];
  midpoint?: number;
  log?: boolean;
};

export const Colormap = {
  VIRIDIS: {
    stops: [
      [68, 1, 84],
      [59, 82, 139],
      [33, 145, 140],
      [94, 201, 98],
      [253, 231, 37],
    ],
  } satisfies ColormapJSON,
  MAGMA: {
    stops: [
      [0, 0, 4],
      [81, 18, 124],
      [183, 55, 121],
      [252, 137, 97],
      [252, 253, 191],
    ],
  } satisfies ColormapJSON,
  CIVIDIS: {
    stops: [
      [0, 32, 77],
      [65, 77, 107],
      [124, 123, 120],
      [188, 175, 111],
      [255, 233, 69],
    ],
  } satisfies ColormapJSON,
  GREYS: { stops: [[64, 64, 64], [250, 250, 250]] } satisfies ColormapJSON,
  RED_BLUE: {
    stops: [
      [202, 0, 32],
      [244, 165, 130],
      [247, 247, 247],
      [146, 197, 222],
      [5, 113, 176],
    ],
  } satisfies ColormapJSON,
  PURPLE_ORANGE: {
    stops: [
      [94, 60, 153],
      [178, 171, 210],
      [247, 247, 247],
      [253, 184, 99],
      [230, 97, 1],
    ],
  } satisfies ColormapJSON,
  named(name: string): ColormapJSON {
    const key = name.trim().toLowerCase().replaceAll("_", "-");
    const maps: Record<string, ColormapJSON> = {
      viridis: Colormap.VIRIDIS,
      magma: Colormap.MAGMA,
      cividis: Colormap.CIVIDIS,
      greys: Colormap.GREYS,
      grays: Colormap.GREYS,
      "red-blue": Colormap.RED_BLUE,
      "purple-orange": Colormap.PURPLE_ORANGE,
    };
    const map = maps[key];
    if (!map) {
      throw new TypeError(`unknown colormap '${name}'`);
    }
    return { ...map, stops: map.stops.map((stop) => [...stop] as [number, number, number]) };
  },
  of(stops: Iterable<[number, number, number]>): ColormapJSON {
    const list = [...stops];
    if (list.length < 2) {
      throw new TypeError("Colormap.of requires at least two stops");
    }
    return { stops: list };
  },
  centeredAt(map: ColormapJSON, midpoint: number): ColormapJSON {
    if (!Number.isFinite(midpoint)) {
      throw new TypeError("Colormap.centeredAt requires a finite midpoint");
    }
    return { ...map, midpoint };
  },
  log(map: ColormapJSON): ColormapJSON {
    return { ...map, log: true };
  },
};

export type Reducer = "Count" | "Sum" | "Mean" | "Median" | "Min" | "Max" | { Percentile: number };
