//! The gallery pages: `EXAMPLES.md` re-read as plates with their sources, and
//! the in-browser plates the wasm build draws.

use std::fs;

use crate::highlight::{escape, highlight};
use crate::markdown::{self, Body, Context};

/// One gallery entry, as `EXAMPLES.md` records it.
pub struct Entry {
    pub name: String,
    pub story: String,
    pub output: String,
}

/// One gallery section.
pub struct Section {
    pub title: String,
    pub intro: String,
    pub entries: Vec<Entry>,
}

/// Parses `EXAMPLES.md`, the generated gallery, into sections.
pub fn examples(context: &Context) -> Vec<Section> {
    let source = fs::read_to_string(context.site.root.join("EXAMPLES.md")).expect("EXAMPLES.md");
    let mut sections: Vec<Section> = Vec::new();
    let mut lines = source.lines().peekable();
    while let Some(line) = lines.next() {
        if let Some(title) = line.strip_prefix("## ") {
            let mut intro = String::new();
            while let Some(next) = lines.peek() {
                if next.starts_with("### ") || next.starts_with("## ") {
                    break;
                }
                let next = lines.next().expect("peeked");
                if !next.trim().is_empty() {
                    intro.push_str(next.trim());
                    intro.push(' ');
                }
            }
            sections.push(Section {
                title: title.to_string(),
                intro: intro.trim().to_string(),
                entries: Vec::new(),
            });
        } else if let Some(name) = line.strip_prefix("### ") {
            let mut story = String::new();
            while let Some(next) = lines.peek() {
                if next.starts_with("```") {
                    break;
                }
                let next = lines.next().expect("peeked");
                if next.starts_with("Source: ") || next.trim().is_empty() {
                    continue;
                }
                story.push_str(next.trim());
                story.push(' ');
            }
            lines.next(); // the opening fence
            let mut output = String::new();
            for next in lines.by_ref() {
                if next.starts_with("```") {
                    break;
                }
                output.push_str(next);
                output.push('\n');
            }
            let section = sections.last_mut().expect("an entry follows a section");
            section.entries.push(Entry {
                name: name.to_string(),
                story: story.trim().to_string(),
                output: output.trim_end_matches('\n').to_string(),
            });
        }
    }
    sections
}

/// The example's source, with its doc comment separated from the code.
pub fn source(context: &Context, name: &str) -> (String, String) {
    let path = context
        .site
        .root
        .join("examples")
        .join(format!("{name}.rs"));
    let text =
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    let mut doc = String::new();
    let mut code = String::new();
    let mut in_doc = true;
    for line in text.lines() {
        if in_doc {
            if let Some(rest) = line.strip_prefix("//!") {
                doc.push_str(rest.trim());
                doc.push('\n');
                continue;
            }
            in_doc = false;
            if line.trim().is_empty() {
                continue;
            }
        }
        code.push_str(line);
        code.push('\n');
    }
    (doc.trim().to_string(), code.trim_end().to_string())
}

/// One entry as HTML: the story, the card (and the pipe text beside it), and
/// the source folded away.
pub fn entry_html(context: &Context, entry: &Entry, number: usize) -> String {
    let (doc, code) = source(context, &entry.name);
    let story = markdown::fragment(&entry.story, context);
    let doc = markdown::fragment(&doc, context);
    let cards = load_cards(context, &entry.name);
    let mut card_html = String::from("<div class=\"view-card card-set\">\n");
    for (index, svg) in cards.iter().enumerate() {
        let label = if cards.len() == 1 {
            entry.name.clone()
        } else {
            format!("{}, pane {}", entry.name, index + 1)
        };
        card_html.push_str(&crate::figures::present_svg(svg, &label));
        card_html.push('\n');
    }
    card_html.push_str("</div>\n");
    format!(
        "<section class=\"example\" id=\"{name}\">\n<h3><a class=\"anchor-name\" href=\"#{name}\">{name}</a></h3>\n{story}\
         <figure class=\"plate\">\n{card_html}<pre class=\"term view-pipe\">{plate}</pre>\n<figcaption><span class=\"plate-number\">Plate {number}.</span> <code class=\"view-card\">cargo run --example {name} -- --svg</code><code class=\"view-pipe\">cargo run --example {name}</code></figcaption></figure>\n\
         <details class=\"source\"><summary>The source, <code>examples/{name}.rs</code></summary>\n<div class=\"source-doc\">{doc}</div>\n<div class=\"code\" data-lang=\"Rust\"><pre><code>{code}</code></pre></div>\n<p class=\"source-link\"><a href=\"https://github.com/shergin/malevich/blob/main/examples/{name}.rs\">On GitHub</a></p></details>\n</section>\n",
        name = escape(&entry.name),
        plate = escape(&entry.output),
        code = highlight("rust", &code),
    )
}

/// The `--svg` output of a gallery example: one card, or several split by a marker.
fn load_cards(context: &Context, name: &str) -> Vec<String> {
    let path = context
        .site
        .root
        .join("examples/cards")
        .join(format!("{name}.svg"));
    let text = fs::read_to_string(&path).unwrap_or_else(|_| {
        panic!(
            "missing {} — run cargo run --example regen_docs",
            path.display()
        )
    });
    text.split("\n<!-- card -->\n")
        .map(str::trim)
        .filter(|card| !card.is_empty())
        .map(str::to_string)
        .collect()
}

/// The gallery page.
pub fn render(context: &Context) -> Body {
    let sections = examples(context);
    let mut html = String::new();
    let mut headings = Vec::new();
    let mut text = String::new();
    html.push_str("<p>The showcase and the system test, one artifact, read as a ladder from the first plot to composition and style. Each plate is the plot value drawn as the quadrant card. The pipe text — braille or quadrants, no color, the bytes <code>cargo run --example NAME</code> prints — is the other switch. The doc generator writes both, and CI fails when either is stale.</p>\n");
    html.push_str("<p>Colored, sized to your terminal, and with real pixels where the terminal speaks them: <code>cargo run --example showcase --features pixel</code>. The wasm build draws the same figures, cells beside pixels, <a href=\"live/\">in the browser</a>.</p>\n");
    html.push_str("<div class=\"gallery-bar\">\n<div class=\"gallery-switch\" role=\"group\" aria-label=\"Plate rendering\"><button type=\"button\" data-view=\"card\" aria-pressed=\"true\">Cards</button><button type=\"button\" data-view=\"pipe\" aria-pressed=\"false\">What a pipe sees</button></div>\n");
    html.push_str("<nav class=\"gallery-index\" aria-label=\"Sections\"><ol>\n");
    for section in &sections {
        let id = section_id(&section.title);
        html.push_str(&format!(
            "<li><a href=\"#{id}\">{}</a> <span>{}</span></li>\n",
            escape(&section.title),
            section.entries.len()
        ));
    }
    html.push_str("</ol></nav>\n</div>\n");
    let mut number = 0;
    for section in &sections {
        let id = section_id(&section.title);
        html.push_str(&format!("<h2 id=\"{id}\">{}<a class=\"anchor\" href=\"#{id}\" aria-label=\"Link to this section\">#</a></h2>\n", escape(&section.title)));
        html.push_str(&format!(
            "<p class=\"section-intro\">{}</p>\n",
            markdown_inline(&section.intro, context)
        ));
        headings.push(markdown::Heading {
            level: 2,
            id: id.clone(),
            text: section.title.clone(),
        });
        for entry in &section.entries {
            number += 1;
            html.push_str(&entry_html(context, entry, number));
            text.push_str(&entry.name);
            text.push(' ');
            text.push_str(&entry.story);
            text.push(' ');
            headings.push(markdown::Heading {
                level: 3,
                id: entry.name.clone(),
                text: entry.name.clone(),
            });
        }
    }
    Body {
        html,
        headings,
        summary: text,
        scripts: Vec::new(),
    }
}

fn markdown_inline(source: &str, context: &Context) -> String {
    let html = markdown::fragment(source, context);
    html.trim()
        .trim_start_matches("<p>")
        .trim_end_matches("</p>")
        .to_string()
}

fn section_id(title: &str) -> String {
    title
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

/// The in-browser plates: the wasm build draws them after the page loads.
pub fn live(context: &Context) -> Body {
    let html = format!(
        "<p>The same plots, drawn twice by the engine compiled to WebAssembly. On the left, a cell grid in the ASCII charset, the bottom rung of the ladder, at truecolor. On the right, the device-pixel panel a capable terminal would place, decoded from the iTerm2 protocol bytes the pixel path emits. Below each plate, the Rust and the TypeScript that build it.</p>\n\
         <p>The last plate is live: a long line through M4. Wheel to zoom, drag to pan, and watch the clock. A zoom is a domain window, so the reduction re-aggregates to the visible columns on every frame.</p>\n\
         <div id=\"figures\" class=\"figures\"><p class=\"status\" id=\"status\">Composing figures…</p></div>\n\
         <p class=\"engine-note\">Drawn in the browser by engine <span id=\"engine\">{version}</span>. The pixel plate is a PNG from the iTerm2 path. The cell plate is the <code>Ascii</code> charset at truecolor.</p>\n",
        version = escape(&context.site.version)
    );
    Body { html, headings: Vec::new(), summary: "Gallery figures drawn in the browser by the wasm build: cells beside pixels, and a live ten-million-point line through M4.".to_string(), scripts: vec!["plates.js".to_string()] }
}
