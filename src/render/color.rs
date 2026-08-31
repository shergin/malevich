//! Colors and color modes, with honest downhill quantization.
//!
//! A [`Color`] names intent (a palette entry, a 256-index, or exact RGB); a
//! [`ColorMode`] names what the output may carry. Encoding resolves every color to
//! the mode's tier: RGB quantizes to the 256-color cube, 256-indices quantize to the
//! nearest of the 16 ANSI colors, and in [`ColorMode::Plain`] color vanishes
//! entirely. The 16 named colors are never upconverted — they stay palette-relative,
//! so terminal themes keep deciding what they look like.

/// How much color the output may carry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ColorMode {
    /// No escape codes at all: safe for files, pipes, and logs.
    Plain,
    /// The 16-color ANSI palette.
    Ansi16,
    /// The xterm 256-color palette.
    Ansi256,
    /// 24-bit RGB.
    TrueColor,
}

/// A terminal color.
///
/// The named variants map to SGR codes 30–37 and 90–97; what they look like is the
/// terminal theme's decision — which is what makes them safe defaults.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[allow(missing_docs)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Color {
    #[default]
    Default,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    /// An index into the xterm 256-color palette.
    Ansi256(u8),
    /// An exact 24-bit color.
    Rgb(u8, u8, u8),
}

/// A color resolved against a mode: what will actually be emitted.
///
/// Equality on resolved colors drives run-length encoding, so two colors that
/// quantize identically share one escape sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Resolved {
    Default,
    /// A complete SGR code for one of the 16 palette colors (30–37, 90–97).
    Indexed16(u8),
    Indexed256(u8),
    Rgb(u8, u8, u8),
}

impl Color {
    /// Resolves this color to what `mode` can carry.
    pub(crate) fn resolve(self, mode: ColorMode) -> Resolved {
        if mode == ColorMode::Plain {
            return Resolved::Default;
        }
        match self {
            Color::Default => Resolved::Default,
            Color::Ansi256(index) => match mode {
                ColorMode::Ansi16 => {
                    let (r, g, b) = ansi256_to_rgb(index);
                    Resolved::Indexed16(rgb_to_16(r, g, b))
                }
                _ => Resolved::Indexed256(index),
            },
            Color::Rgb(r, g, b) => match mode {
                ColorMode::TrueColor => Resolved::Rgb(r, g, b),
                ColorMode::Ansi256 => Resolved::Indexed256(rgb_to_256(r, g, b)),
                _ => Resolved::Indexed16(rgb_to_16(r, g, b)),
            },
            named => Resolved::Indexed16(named.sgr()),
        }
    }

    /// The concrete RGB this color denotes in output that cannot stay
    /// palette-relative: named colors freeze to the xterm defaults the quantizer
    /// already assumes, `Default` to a mid-gray readable on dark and light
    /// backgrounds alike.
    pub(crate) fn to_rgb(self) -> (u8, u8, u8) {
        match self {
            Color::Default => (128, 128, 128),
            Color::Ansi256(index) => ansi256_to_rgb(index),
            Color::Rgb(r, g, b) => (r, g, b),
            named => {
                let sgr = named.sgr();
                let offset = if sgr >= 90 { sgr - 90 + 8 } else { sgr - 30 };
                PALETTE16[offset as usize]
            }
        }
    }

    /// The SGR foreground code for a named color.
    fn sgr(self) -> u8 {
        match self {
            Color::Black => 30,
            Color::Red => 31,
            Color::Green => 32,
            Color::Yellow => 33,
            Color::Blue => 34,
            Color::Magenta => 35,
            Color::Cyan => 36,
            Color::White => 37,
            Color::BrightBlack => 90,
            Color::BrightRed => 91,
            Color::BrightGreen => 92,
            Color::BrightYellow => 93,
            Color::BrightBlue => 94,
            Color::BrightMagenta => 95,
            Color::BrightCyan => 96,
            Color::BrightWhite => 97,
            _ => 39,
        }
    }
}

impl Resolved {
    /// Appends one SGR sequence for the foreground and/or background channels
    /// that changed. Keeping both parameters in one control sequence avoids
    /// doubling terminal transitions for two-color cells.
    pub(crate) fn write_transition(
        foreground: Option<Resolved>,
        background: Option<Resolved>,
        out: &mut String,
    ) {
        if foreground.is_none() && background.is_none() {
            return;
        }
        out.push_str("\x1b[");
        if let Some(color) = foreground {
            color.write_parameters(out, false);
            if background.is_some() {
                out.push(';');
            }
        }
        if let Some(color) = background {
            color.write_parameters(out, true);
        }
        out.push('m');
    }

    fn write_parameters(self, out: &mut String, background: bool) {
        use std::fmt::Write as _;
        let _ = match self {
            Resolved::Default => write!(out, "{}", if background { 49 } else { 39 }),
            Resolved::Indexed16(code) => {
                write!(out, "{}", if background { code + 10 } else { code })
            }
            Resolved::Indexed256(index) => {
                write!(out, "{};5;{index}", if background { 48 } else { 38 })
            }
            Resolved::Rgb(r, g, b) => {
                write!(out, "{};2;{r};{g};{b}", if background { 48 } else { 38 })
            }
        };
    }
}

/// The xterm default RGB values of the 16 palette colors, in SGR order
/// (30–37 then 90–97). Used only for quantization distance — the terminal's real
/// palette may differ, which is the point of named colors.
const PALETTE16: [(u8, u8, u8); 16] = [
    (0, 0, 0),
    (205, 0, 0),
    (0, 205, 0),
    (205, 205, 0),
    (0, 0, 238),
    (205, 0, 205),
    (0, 205, 205),
    (229, 229, 229),
    (127, 127, 127),
    (255, 0, 0),
    (0, 255, 0),
    (255, 255, 0),
    (92, 92, 255),
    (255, 0, 255),
    (0, 255, 255),
    (255, 255, 255),
];

/// The cube axis levels of the xterm 256-color palette.
const CUBE: [u8; 6] = [0, 95, 135, 175, 215, 255];

/// Quantizes RGB onto the xterm 256-color palette (color cube or gray ramp).
pub(crate) fn rgb_to_256(r: u8, g: u8, b: u8) -> u8 {
    if r == g && g == b {
        return match r {
            0..=3 => 16,
            248..=255 => 231,
            v => 232 + (v - 8) / 10,
        };
    }
    let axis = |c: u8| match c {
        0..=47 => 0u8,
        48..=114 => 1,
        c => (c - 35) / 40,
    };
    16 + 36 * axis(r) + 6 * axis(g) + axis(b)
}

/// The RGB value of an xterm 256-color palette index.
pub(crate) fn ansi256_to_rgb(index: u8) -> (u8, u8, u8) {
    match index {
        0..=15 => PALETTE16[index as usize],
        16..=231 => {
            let n = index - 16;
            (
                CUBE[(n / 36) as usize],
                CUBE[((n / 6) % 6) as usize],
                CUBE[(n % 6) as usize],
            )
        }
        gray => {
            let v = 8 + 10 * (gray - 232);
            (v, v, v)
        }
    }
}

/// Quantizes RGB to the nearest of the 16 palette colors, returning its SGR code.
pub(crate) fn rgb_to_16(r: u8, g: u8, b: u8) -> u8 {
    let mut best = 0usize;
    let mut best_distance = u32::MAX;
    for (index, &(pr, pg, pb)) in PALETTE16.iter().enumerate() {
        let distance = (i32::from(r) - i32::from(pr)).pow(2) as u32
            + (i32::from(g) - i32::from(pg)).pow(2) as u32
            + (i32::from(b) - i32::from(pb)).pow(2) as u32;
        if distance < best_distance {
            best_distance = distance;
            best = index;
        }
    }
    if best < 8 {
        30 + best as u8
    } else {
        90 + (best as u8 - 8)
    }
}

#[cfg(test)]
#[path = "tests/color_tests.rs"]
mod tests;
