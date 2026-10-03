//! The figures: plots built in `site/figures/*.rs`, rendered by the library at
//! build time. A page asks for one with a `{{figure name}}` line; the registry
//! renders it as the SVG terminal card, as the ANSI a tty would receive, or as
//! plain text, and shows beside it the exact code that ran — the block is both
//! `include!`d and `include_str!`ed, so the two cannot drift.

use std::cell::Cell;

use malevich::pixel::{Graphics, Protocol};
use malevich::render::{Charset, ColorMode};
use malevich::{Document, Frame, Plot, Theme};

use crate::ansi;
use crate::highlight::{escape, highlight};
use crate::markdown::Context;

/// One figure: a plot, the frame it is meant for, and its source.
pub struct Figure {
    pub name: &'static str,
    pub caption: &'static str,
    pub frame: Frame,
    pub source: &'static str,
    pub plot: Plot<'static>,
}

/// Every figure, by name.
pub struct Registry {
    figures: Vec<Figure>,
    rendered: Cell<usize>,
}

/// A frame for the cards: quadrants, truecolor, the dark card.
pub fn card(width: usize, height: usize) -> Frame {
    Frame {
        width,
        height,
        charset: Charset::Quadrants,
        color: ColorMode::TrueColor,
        theme: Theme::DARK,
    }
}

/// A card with a chosen charset.
pub fn card_in(charset: Charset, width: usize, height: usize) -> Frame {
    Frame {
        charset,
        ..card(width, height)
    }
}

macro_rules! catalog {
    ($($name:literal => ($caption:literal, $frame:expr)),* $(,)?) => {
        vec![$(Figure {
            name: $name,
            caption: $caption,
            frame: $frame,
            source: include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/figures/", $name, ".rs")),
            plot: include!(concat!(env!("CARGO_MANIFEST_DIR"), "/figures/", $name, ".rs")),
        }),*]
    };
}

mod catalog;

impl Registry {
    pub fn new() -> Registry {
        Registry {
            figures: catalog::all(),
            rendered: Cell::new(0),
        }
    }

    /// How many figures were rendered so far.
    pub fn rendered(&self) -> usize {
        self.rendered.get()
    }

    /// Every figure's name, in catalog order.
    pub fn names(&self) -> impl Iterator<Item = &'static str> + '_ {
        self.figures.iter().map(|figure| figure.name)
    }

    pub fn get(&self, name: &str) -> Option<&Figure> {
        self.figures.iter().find(|figure| figure.name == name)
    }

    fn figure(&self, name: &str) -> Result<&Figure, String> {
        self.get(name)
            .ok_or_else(|| format!("no figure named {name}"))
    }

    /// Expands one `{{kind arguments | caption}}` directive.
    pub fn directive(
        &self,
        kind: &str,
        arguments: &[&str],
        caption: Option<&str>,
        context: &Context,
        plates: &Cell<usize>,
        scripts: &mut Vec<String>,
    ) -> Result<String, String> {
        let next = || {
            plates.set(plates.get() + 1);
            plates.get()
        };
        let first = || {
            arguments
                .first()
                .copied()
                .ok_or_else(|| "a figure name is required".to_string())
        };
        let caption_of = |figure: &Figure| caption.unwrap_or(figure.caption).to_string();
        match kind {
            // The SVG card with the code that built it.
            "figure" => {
                let figure = self.figure(first()?)?;
                let show_code = !arguments.contains(&"nocode");
                let svg = self.svg(figure, &figure.frame);
                Ok(plate(
                    next(),
                    &svg,
                    &caption_of(figure),
                    show_code.then_some(figure.source),
                    "card",
                ))
            }
            // The light card, for the theme story.
            "light" => {
                let figure = self.figure(first()?)?;
                let svg = self.svg(
                    figure,
                    &Frame {
                        theme: Theme::LIGHT,
                        ..figure.frame
                    },
                );
                Ok(plate(next(), &svg, &caption_of(figure), None, "card light"))
            }
            // Plain text: what a pipe, a log, or a language model sees.
            "plain" => {
                let figure = self.figure(first()?)?;
                let frame = Frame::plain(figure.frame.width, figure.frame.height);
                let text = figure.plot.render(&frame);
                let inner = format!("<pre class=\"term\">{}</pre>", escape(&text));
                Ok(plate(next(), &inner, &caption_of(figure), None, "text"))
            }
            // The portable text form: quadrants, no color.
            "portable" => {
                let figure = self.figure(first()?)?;
                let frame = Frame::portable(figure.frame.width, figure.frame.height);
                let text = figure.plot.render(&frame);
                let inner = format!("<pre class=\"term\">{}</pre>", escape(&text));
                Ok(plate(next(), &inner, &caption_of(figure), None, "text"))
            }
            // The ANSI bytes of one color mode, decoded like a terminal would.
            "ansi" => {
                let figure = self.figure(first()?)?;
                let mode = arguments.get(1).copied().unwrap_or("TrueColor");
                let frame = Frame {
                    color: color_mode(mode)?,
                    ..figure.frame
                };
                let inner = format!(
                    "<pre class=\"term\">{}</pre>",
                    ansi::to_html(&figure.plot.render(&frame))
                );
                Ok(plate(next(), &inner, &caption_of(figure), None, "text"))
            }
            // The charset ladder: one card per tier.
            "charsets" => {
                let figure = self.figure(first()?)?;
                let tiers = [
                    (Charset::Octants, "Octants — 2×4 blocks, Unicode 16"),
                    (Charset::Sextants, "Sextants — 2×3 blocks, Unicode 13"),
                    (Charset::Braille, "Braille — 2×4 dots"),
                    (
                        Charset::Quadrants,
                        "Quadrants — 2×2 blocks, the UTF-8 default",
                    ),
                    (Charset::HalfBlocks, "Half blocks — 1×2"),
                    (Charset::Ascii, "ASCII — 1×1, the guaranteed fallback"),
                ];
                let mut inner = String::from("<div class=\"ladder\">");
                for (charset, label) in tiers {
                    let svg = self.svg(
                        figure,
                        &Frame {
                            charset,
                            ..figure.frame
                        },
                    );
                    inner.push_str(&format!(
                        "<div class=\"rung\"><p class=\"rung-label\">{}</p>{svg}</div>",
                        escape(label)
                    ));
                }
                inner.push_str("</div>");
                Ok(plate(
                    next(),
                    &inner,
                    &caption_of(figure),
                    None,
                    "ladder-plate",
                ))
            }
            // The color ladder: the encoder's own SGR bytes at every mode.
            "colors" => {
                let figure = self.figure(first()?)?;
                let tiers = [
                    (ColorMode::TrueColor, "TrueColor — 24-bit RGB"),
                    (ColorMode::Ansi256, "Ansi256 — the xterm 256-color palette"),
                    (
                        ColorMode::Ansi16,
                        "Ansi16 — the sixteen named colors, picked in OKLab",
                    ),
                    (ColorMode::Plain, "Plain — no escape byte at all"),
                ];
                let mut inner = String::from("<div class=\"ladder\">");
                for (color, label) in tiers {
                    let frame = Frame {
                        color,
                        ..figure.frame
                    };
                    let html = ansi::to_html(&figure.plot.render(&frame));
                    inner.push_str(&format!("<div class=\"rung\"><p class=\"rung-label\">{}</p><pre class=\"term\">{html}</pre></div>", escape(label)));
                }
                inner.push_str("</div>");
                Ok(plate(
                    next(),
                    &inner,
                    &caption_of(figure),
                    None,
                    "ladder-plate",
                ))
            }
            // The same plot at several sizes: `{{sizes name 80x20 40x10}}`.
            "sizes" => {
                let figure = self.figure(first()?)?;
                let mut inner = String::from("<div class=\"sizes\">");
                for size in &arguments[1..] {
                    let (width, height) = size
                        .split_once('x')
                        .and_then(|(w, h)| Some((w.parse().ok()?, h.parse().ok()?)))
                        .ok_or_else(|| format!("bad size {size}"))?;
                    let svg = self.svg(
                        figure,
                        &Frame {
                            width,
                            height,
                            ..figure.frame
                        },
                    );
                    inner.push_str(&format!("<div class=\"size\"><p class=\"rung-label\">{width}×{height}</p>{svg}</div>"));
                }
                inner.push_str("</div>");
                Ok(plate(
                    next(),
                    &inner,
                    &caption_of(figure),
                    None,
                    "sizes-plate",
                ))
            }
            // Two or more cards side by side.
            "pair" => {
                let mut inner = String::from("<div class=\"pair\">");
                let mut captions = Vec::new();
                for name in arguments {
                    let figure = self.figure(name)?;
                    inner.push_str(&self.svg(figure, &figure.frame));
                    captions.push(figure.caption);
                }
                inner.push_str("</div>");
                let caption = caption
                    .map(str::to_string)
                    .unwrap_or_else(|| captions.join(" "));
                Ok(plate(next(), &inner, &caption, None, "pair-plate"))
            }
            // The HTML card itself, as a notebook would show it.
            "html" => {
                let figure = self.figure(first()?)?;
                let html = figure.plot.to_html(&figure.frame);
                Ok(plate(
                    next(),
                    &format!("<div class=\"html-card\">{html}</div>"),
                    &caption_of(figure),
                    None,
                    "card-html",
                ))
            }
            // The cell card above the pixel card: `to_svg` and `to_svg_pixels`.
            "pixelcard" => {
                let figure = self.figure(first()?)?;
                self.rendered.set(self.rendered.get() + 1);
                let graphics = Graphics::new(Protocol::Kitty);
                let cards = [
                    ("Plot::to_svg", figure.plot.to_svg(&figure.frame)),
                    (
                        "Plot::to_svg_pixels",
                        figure.plot.to_svg_pixels(&figure.frame, &graphics),
                    ),
                ];
                let mut inner = String::from("<div class=\"sizes\">");
                for (label, svg) in cards {
                    inner.push_str(&format!(
                        "<div class=\"size\"><p class=\"rung-label\"><code>{label}</code></p>{}</div>",
                        present_svg(&svg, &format!("{label}: {}", figure.caption)),
                    ));
                }
                inner.push_str("</div>");
                Ok(plate(
                    next(),
                    &inner,
                    &caption_of(figure),
                    None,
                    "sizes-plate",
                ))
            }
            // The head of the SVG source, to show what a card is made of.
            "svgsource" => {
                let figure = self.figure(first()?)?;
                let svg = figure.plot.to_svg(&figure.frame);
                let lines: Vec<&str> = svg.lines().take(12).collect();
                let shown = format!("{}\n…\n</svg>", lines.join("\n"));
                Ok(crate::markdown::code_block("xml", &shown))
            }
            // The serialized Document, pretty-printed.
            "json" => {
                let figure = self.figure(first()?)?;
                let document =
                    Document::plot(figure.plot.clone()).map_err(|error| error.to_string())?;
                let json =
                    serde_json::to_string_pretty(&document).map_err(|error| error.to_string())?;
                Ok(crate::markdown::code_block("json", &json))
            }
            // The code of a figure, without rendering it.
            "code" => {
                let figure = self.figure(first()?)?;
                Ok(crate::markdown::code_block("rust", &dedent(figure.source)))
            }
            // A gallery example: its plate and its source.
            "example" => {
                let name = first()?;
                let sections = crate::gallery::examples(context);
                let entry = sections
                    .iter()
                    .flat_map(|section| section.entries.iter())
                    .find(|entry| entry.name == name)
                    .ok_or_else(|| format!("no gallery example named {name}"))?;
                Ok(crate::gallery::entry_html(context, entry, next()))
            }
            // Interactive pieces the wasm build powers.
            "explorer" => {
                scripts.push("ladder.js".to_string());
                Ok(explorer(next()))
            }
            "resizer" => {
                scripts.push("ladder.js".to_string());
                Ok(resizer(next()))
            }
            other => Err(format!("unknown directive {other}")),
        }
    }

    /// The SVG card of a figure in a frame, sized by CSS rather than by its
    /// own width and height, with an accessible name.
    pub fn svg(&self, figure: &Figure, frame: &Frame) -> String {
        self.rendered.set(self.rendered.get() + 1);
        present_svg(&figure.plot.to_svg(frame), figure.caption)
    }
}

/// An SVG card sized by CSS: the root keeps its `viewBox`, gains an accessible
/// name, and grows no wider than the width the encoder wrote.
pub fn present_svg(svg: &str, label: &str) -> String {
    let head_end = svg.find('>').expect("svg root element");
    let (head, rest) = svg.split_at(head_end);
    let mut width = None;
    let mut attributes = String::new();
    for attribute in head.split_whitespace().skip(1) {
        if let Some(value) = attribute.strip_prefix("width=\"") {
            width = value.strip_suffix('"');
            continue;
        }
        if attribute.starts_with("height=") {
            continue;
        }
        attributes.push(' ');
        attributes.push_str(attribute);
    }
    let width = width.expect("an SVG card states its width");
    format!(
        "<svg class=\"card\" role=\"img\" aria-label=\"{}\" style=\"max-width:{width}px\"{attributes}{rest}",
        escape(label),
    )
}

fn color_mode(name: &str) -> Result<ColorMode, String> {
    Ok(match name {
        "TrueColor" | "truecolor" => ColorMode::TrueColor,
        "Ansi256" | "256" => ColorMode::Ansi256,
        "Ansi16" | "16" => ColorMode::Ansi16,
        "Plain" | "plain" => ColorMode::Plain,
        other => return Err(format!("unknown color mode {other}")),
    })
}

/// Strips the outer braces and one indent level from a figure's block.
pub fn dedent(source: &str) -> String {
    let inner: Vec<&str> = source
        .lines()
        .skip_while(|line| line.trim() != "{")
        .skip(1)
        .collect();
    let end = inner
        .iter()
        .rposition(|line| line.trim() == "}")
        .unwrap_or(inner.len());
    inner[..end]
        .iter()
        .map(|line| line.strip_prefix("    ").unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}

/// A numbered plate: the rendered thing, its caption, and optionally the code.
fn plate(number: usize, inner: &str, caption: &str, code: Option<&str>, class: &str) -> String {
    let mut out = format!(
        "<figure class=\"plate {class}\">\n{inner}\n<figcaption><span class=\"plate-number\">Plate {number}.</span> {}</figcaption>\n",
        inline_markdown(caption)
    );
    if let Some(code) = code {
        out.push_str(&format!(
            "<details class=\"source\"><summary>The code that drew it</summary><div class=\"code\" data-lang=\"Rust\"><pre><code>{}</code></pre></div></details>\n",
            highlight("rust", &dedent(code))
        ));
    }
    out.push_str("</figure>\n");
    out
}

/// Backticks to `<code>` and nothing else — captions stay one sentence.
fn inline_markdown(text: &str) -> String {
    let mut out = String::new();
    let mut code = false;
    for part in text.split('`') {
        if code {
            out.push_str("<code>");
            out.push_str(&escape(part));
            out.push_str("</code>");
        } else {
            out.push_str(&escape(part));
        }
        code = !code;
    }
    out
}

fn explorer(number: usize) -> String {
    format!(
        "<figure class=\"plate explorer\" id=\"explorer\">\n\
         <div class=\"explorer-controls\">\n\
         <fieldset><legend>Charset</legend>\
         <label><input type=\"radio\" name=\"charset\" value=\"Octants\">Octants</label>\
         <label><input type=\"radio\" name=\"charset\" value=\"Sextants\">Sextants</label>\
         <label><input type=\"radio\" name=\"charset\" value=\"Braille\">Braille</label>\
         <label><input type=\"radio\" name=\"charset\" value=\"Quadrants\" checked>Quadrants</label>\
         <label><input type=\"radio\" name=\"charset\" value=\"HalfBlocks\">Half blocks</label>\
         <label><input type=\"radio\" name=\"charset\" value=\"Ascii\">ASCII</label></fieldset>\n\
         <fieldset><legend>Color</legend>\
         <label><input type=\"radio\" name=\"color\" value=\"TrueColor\" checked>TrueColor</label>\
         <label><input type=\"radio\" name=\"color\" value=\"Ansi256\">256</label>\
         <label><input type=\"radio\" name=\"color\" value=\"Ansi16\">16</label>\
         <label><input type=\"radio\" name=\"color\" value=\"Plain\">Plain</label></fieldset>\n\
         <fieldset><legend>Chart</legend>\
         <label><input type=\"radio\" name=\"chart\" value=\"lines\" checked>Lines</label>\
         <label><input type=\"radio\" name=\"chart\" value=\"heatmap\">Heatmap</label>\
         <label><input type=\"radio\" name=\"chart\" value=\"scatter\">Grouped scatter</label></fieldset>\n\
         </div>\n\
         <pre class=\"term\" id=\"explorer-plate\">Loading the engine…</pre>\n\
         <p class=\"readout\" id=\"explorer-readout\"></p>\n\
         <figcaption><span class=\"plate-number\">Plate {number}.</span> The same plot value, re-encoded live by the wasm build for the charset and color mode you choose: nothing is redrawn, only the frame changes.</figcaption>\n\
         </figure>\n"
    )
}

fn resizer(number: usize) -> String {
    format!(
        "<figure class=\"plate resizer\" id=\"resizer\">\n\
         <div class=\"explorer-controls\">\
         <label class=\"slider\">Width <input type=\"range\" id=\"resizer-width\" min=\"12\" max=\"120\" value=\"80\"> <output id=\"resizer-width-out\">80</output></label>\
         <label class=\"slider\">Height <input type=\"range\" id=\"resizer-height\" min=\"3\" max=\"30\" value=\"18\"> <output id=\"resizer-height-out\">18</output></label>\
         </div>\n\
         <pre class=\"term\" id=\"resizer-plate\">Loading the engine…</pre>\n\
         <figcaption><span class=\"plate-number\">Plate {number}.</span> Shrink the frame and watch the furniture shed — legend, then titles, then tick density — while the data region is the last thing standing. Grow it and the ticks re-search for a denser labeling.</figcaption>\n\
         </figure>\n"
    )
}

// Data the figures share: a deterministic noise source and the repository's
// vendored datasets, so a figure's block stays short enough to read.

/// A tiny deterministic xorshift, uniform in `[0, 1)`.
pub fn noise(seed: u64) -> impl FnMut() -> f64 {
    let mut state = seed.max(1) ^ 0x9e37_79b9_7f4a_7c15;
    move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// A standard normal draw from a uniform source (Box–Muller).
pub fn gaussian(unit: &mut impl FnMut() -> f64) -> f64 {
    let u = unit().max(1e-12);
    let v = unit();
    (-2.0 * u.ln()).sqrt() * (std::f64::consts::TAU * v).cos()
}

/// The Palmer penguins (CC0), as columns.
pub struct Penguins {
    pub species: Vec<String>,
    pub bill_length: Vec<f64>,
    pub bill_depth: Vec<f64>,
    pub flipper: Vec<f64>,
    pub mass: Vec<f64>,
}

impl Penguins {
    /// One species' values of a column.
    pub fn of<'a>(&'a self, species: &str, column: &'a [f64]) -> Vec<f64> {
        self.species
            .iter()
            .zip(column)
            .filter(|(name, _)| name.as_str() == species)
            .map(|(_, value)| *value)
            .collect()
    }
}

pub fn penguins() -> Penguins {
    let text = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../examples/data/penguins.csv"
    ));
    let mut data = Penguins {
        species: Vec::new(),
        bill_length: Vec::new(),
        bill_depth: Vec::new(),
        flipper: Vec::new(),
        mass: Vec::new(),
    };
    for line in text.lines().skip(1) {
        let mut parts = line.split(',');
        let species = parts.next().unwrap_or_default();
        let mut number = || {
            parts
                .next()
                .and_then(|v| v.parse().ok())
                .unwrap_or(f64::NAN)
        };
        let (length, depth, flipper, mass) = (number(), number(), number(), number());
        data.species.push(species.to_string());
        data.bill_length.push(length);
        data.bill_depth.push(depth);
        data.flipper.push(flipper);
        data.mass.push(mass);
    }
    data
}

/// The first of the month as unix seconds (Hinnant's civil-date arithmetic).
pub fn month_stamp(year: i64, month: u64) -> f64 {
    let y = year - i64::from(month <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u64;
    let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    ((era * 146_097 + doe as i64 - 719_468) * 86_400) as f64
}

/// A calendar day as unix seconds.
pub fn day_stamp(year: i64, month: u64, day: u64) -> f64 {
    month_stamp(year, month) + (day - 1) as f64 * 86_400.0
}

/// Monthly CO₂ at Mauna Loa (NOAA): unix seconds and ppm.
pub fn co2() -> (Vec<f64>, Vec<f64>) {
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../examples/data/co2_monthly.csv"
    ))
    .lines()
    .skip(1)
    .filter_map(|line| {
        let mut parts = line.split(',');
        let year: i64 = parts.next()?.parse().ok()?;
        let month: u64 = parts.next()?.parse().ok()?;
        let ppm: f64 = parts.next()?.parse().ok()?;
        Some((month_stamp(year, month), ppm))
    })
    .unzip()
}

/// A real training log: per-step loss of topos's bigram model.
pub fn loss_log() -> Vec<f64> {
    include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../examples/data/topos_loss.csv"
    ))
    .lines()
    .filter_map(|line| line.split(',').nth(1)?.parse().ok())
    .collect()
}

/// Every release date in the changelog, as unix seconds.
pub fn release_dates() -> Vec<f64> {
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../CHANGELOG.md"))
        .lines()
        .filter(|line| line.starts_with("## ") && line.contains(" — "))
        .filter_map(|line| {
            let date = line.rsplit(" — ").next()?.trim();
            let mut parts = date.split('-');
            let year: i64 = parts.next()?.parse().ok()?;
            let month: u64 = parts.next()?.parse().ok()?;
            let day: u64 = parts.next()?.parse().ok()?;
            Some(day_stamp(year, month, day))
        })
        .collect()
}
