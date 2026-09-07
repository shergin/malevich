import { Charset, ColorMode, Theme, ThemeJSON } from "./color.js";

export type FrameJSON = {
  width: number;
  height: number;
  charset: Charset;
  color: ColorMode;
  theme: ThemeJSON;
};

const CHARSETS: Record<string, Charset> = {
  ascii: "Ascii",
  half: "HalfBlocks",
  halfblock: "HalfBlocks",
  halfblocks: "HalfBlocks",
  quad: "Quadrants",
  quadrant: "Quadrants",
  quadrants: "Quadrants",
  sextant: "Sextants",
  sextants: "Sextants",
  octant: "Octants",
  octants: "Octants",
  braille: "Braille",
};

function env(name: string): string | undefined {
  if (typeof process === "undefined") {
    return undefined;
  }
  const value = process.env[name];
  return value === undefined || value === "" ? undefined : value;
}

export class Frame {
  readonly width: number;
  readonly height: number;
  readonly charset: Charset;
  readonly color: ColorMode;
  readonly theme: ThemeJSON;

  constructor(init: FrameJSON) {
    this.width = init.width;
    this.height = init.height;
    this.charset = init.charset;
    this.color = init.color;
    this.theme = init.theme;
  }

  /** Deterministic braille, no color — the 1.x snapshot frame. */
  static plain(width: number, height: number): Frame {
    return new Frame({
      width,
      height,
      charset: "Braille",
      color: "Plain",
      theme: Theme.DARK,
    });
  }

  /** Conservative Unicode: quadrants, no color. */
  static portable(width: number, height: number): Frame {
    return new Frame({
      width,
      height,
      charset: "Quadrants",
      color: "Plain",
      theme: Theme.DARK,
    });
  }

  /**
   * Reads the environment — analog of `Frame::detect_for`.
   * Pass stderr when the plot writes there; stdout is the default.
   */
  static detect(stream?: NodeJS.WriteStream): Frame {
    const target = stream ?? (typeof process !== "undefined" ? process.stdout : undefined);
    const width = target?.columns && target.columns > 0 ? target.columns : 80;
    const rows = target?.rows && target.rows > 0 ? target.rows : undefined;
    const height = rows === undefined ? 16 : Math.min(24, Math.max(8, Math.floor(rows / 3)));
    return new Frame({
      width,
      height,
      charset: detectCharset(),
      color: detectColor(Boolean(target?.isTTY)),
      theme: Theme.detect(),
    });
  }

  /** A copy with some fields replaced — `frame.with({ height: 8 })`. */
  with(patch: Partial<FrameJSON>): Frame {
    return new Frame({ ...this.toJSON(), ...patch });
  }

  toJSON(): FrameJSON {
    return {
      width: this.width,
      height: this.height,
      charset: this.charset,
      color: this.color,
      theme: this.theme,
    };
  }
}

function detectCharset(): Charset {
  const forced = env("MALEVICH_CHARSET");
  if (forced) {
    const named = CHARSETS[forced.trim().toLowerCase()];
    if (named) {
      return named;
    }
  }
  if (env("TERM") === "dumb") {
    return "Ascii";
  }
  for (const name of ["LC_ALL", "LC_CTYPE", "LANG"]) {
    const locale = env(name);
    if (locale) {
      return locale.toLowerCase().includes("utf") ? "Quadrants" : "Ascii";
    }
  }
  return "Quadrants";
}

function detectColor(isTerminal: boolean): ColorMode {
  if (env("NO_COLOR") !== undefined) {
    return "Plain";
  }
  const forced = env("CLICOLOR_FORCE");
  const forceOn = forced !== undefined && forced !== "0";
  if (!forceOn && !isTerminal) {
    return "Plain";
  }
  const term = env("TERM") ?? "";
  if (term === "dumb") {
    return "Plain";
  }
  const colorterm = env("COLORTERM") ?? "";
  if (colorterm === "truecolor" || colorterm === "24bit") {
    return "TrueColor";
  }
  if (term.includes("256color")) {
    return "Ansi256";
  }
  return "Ansi16";
}
