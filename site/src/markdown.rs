//! Markdown to HTML for the site: pulldown-cmark plus the site's own rules —
//! links into the repository become links into the site, `text` fences become
//! terminal plates, code fences are highlighted, headings get anchors, and a
//! `{{directive}}` on a line of its own becomes a figure the library renders.

use std::cell::Cell;
use std::fs;

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::Site;
use crate::figures::Registry;
use crate::highlight::{escape, highlight};
use crate::pages::{self, Page};

/// What a page body needs to know while it renders.
pub struct Context<'a> {
    pub site: &'a Site,
    pub figures: &'a Registry,
    pub page: &'a Page,
    /// The repository directory relative links resolve against.
    pub source_dir: &'a str,
}

impl Context<'_> {
    /// The relative prefix that reaches the site root from this page.
    pub fn root(&self) -> String {
        self.page.root()
    }

    /// A site-internal href for a site-absolute URL.
    pub fn href(&self, url: &str) -> String {
        format!("{}{}", self.root(), url.trim_start_matches('/'))
    }
}

/// A heading in a rendered body, for the table of contents.
pub struct Heading {
    pub level: u8,
    pub id: String,
    pub text: String,
}

/// A rendered body.
pub struct Body {
    pub html: String,
    pub headings: Vec<Heading>,
    /// Plain text, for the search index.
    pub summary: String,
    /// Page scripts the body asked for, as paths relative to the site root.
    pub scripts: Vec<String>,
}

/// Renders a markdown source into a body.
pub fn render(source: &str, context: &Context) -> Body {
    let plates = Cell::new(0);
    let mut scripts = Vec::new();
    let expanded = expand_directives(source, context, &plates, &mut scripts);
    let mut body = render_markdown(&expanded, context);
    body.scripts.append(&mut scripts);
    body
}

/// Renders a markdown fragment with no directives, as HTML.
pub fn fragment(source: &str, context: &Context) -> String {
    render_markdown(source, context).html
}

fn render_markdown(source: &str, context: &Context) -> Body {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let mut parser = Parser::new_ext(source, options);
    let mut events = Vec::new();
    let mut headings = Vec::new();
    let mut used = Vec::new();
    let mut text = String::new();
    let mut dropped_title = false;
    while let Some(event) = parser.next() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                let mut inner = Vec::new();
                let mut label = String::new();
                for event in parser.by_ref() {
                    match event {
                        Event::End(TagEnd::Heading(_)) => break,
                        Event::Text(ref t) | Event::Code(ref t) => {
                            label.push_str(t);
                            inner.push(event);
                        }
                        other => inner.push(other),
                    }
                }
                // The first h1 is the page title; the layout prints it.
                if level == HeadingLevel::H1 && !dropped_title {
                    dropped_title = true;
                    continue;
                }
                let id = slug(&label, &mut used);
                let n = level as u8;
                events.push(Event::Html(format!("<h{n} id=\"{id}\">").into()));
                events.extend(inner);
                events.push(Event::Html(
                    format!("<a class=\"anchor\" href=\"#{id}\" aria-label=\"Link to this section\">#</a></h{n}>\n")
                        .into(),
                ));
                headings.push(Heading {
                    level: n,
                    id,
                    text: label,
                });
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                let language = match kind {
                    CodeBlockKind::Fenced(language) => language.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                let mut code = String::new();
                for event in parser.by_ref() {
                    match event {
                        Event::End(TagEnd::CodeBlock) => break,
                        Event::Text(t) => code.push_str(&t),
                        _ => {}
                    }
                }
                events.push(Event::Html(code_block(&language, &code).into()));
            }
            Event::Start(Tag::Link {
                link_type,
                dest_url,
                title,
                id,
            }) => {
                let dest = rewrite_link(&dest_url, context);
                events.push(Event::Start(Tag::Link {
                    link_type,
                    dest_url: dest.into(),
                    title,
                    id,
                }));
            }
            Event::Start(Tag::Image {
                link_type,
                dest_url,
                title,
                id,
            }) => {
                let dest = rewrite_image(&dest_url, context);
                events.push(Event::Start(Tag::Image {
                    link_type,
                    dest_url: dest.into(),
                    title,
                    id,
                }));
            }
            Event::Html(ref html)
                if html.trim_start().starts_with("<!-- generated:")
                    || html.trim_start().starts_with("<!-- /generated") => {}
            Event::Text(ref t) => {
                text.push_str(t);
                text.push(' ');
                events.push(event);
            }
            other => events.push(other),
        }
    }
    let mut html = String::with_capacity(source.len() * 2);
    pulldown_cmark::html::push_html(&mut html, events.into_iter());
    let summary: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    Body {
        html,
        headings,
        summary,
        scripts: Vec::new(),
    }
}

/// GitHub's heading slug: lowercase, spaces to hyphens, punctuation dropped.
fn slug(text: &str, used: &mut Vec<String>) -> String {
    let mut base = String::new();
    for c in text.chars() {
        if c.is_alphanumeric() {
            base.extend(c.to_lowercase());
        } else if c == ' ' || c == '-' {
            base.push('-');
        }
    }
    let base = base.trim_matches('-').to_string();
    let base = if base.is_empty() {
        "section".to_string()
    } else {
        base
    };
    let mut candidate = base.clone();
    let mut n = 1;
    while used.contains(&candidate) {
        n += 1;
        candidate = format!("{base}-{n}");
    }
    used.push(candidate.clone());
    candidate
}

/// A fenced block: a terminal plate for `text`, highlighted code otherwise.
pub fn code_block(language: &str, code: &str) -> String {
    let code = code.trim_end_matches('\n');
    let language = language.split_whitespace().next().unwrap_or("");
    if language == "text" || language == "plate" {
        return format!(
            "<figure class=\"plate\"><pre class=\"term\">{}</pre></figure>\n",
            escape(code)
        );
    }
    let label = match language {
        "" => String::new(),
        "rust" | "rs" => "Rust".to_string(),
        "js" | "javascript" => "JavaScript".to_string(),
        "ts" | "typescript" | "tsx" => "TypeScript".to_string(),
        "sh" | "bash" | "shell" | "zsh" | "console" => "shell".to_string(),
        other => other.to_string(),
    };
    format!(
        "<div class=\"code\" data-lang=\"{label}\"><pre><code>{}</code></pre></div>\n",
        highlight(language, code)
    )
}

/// Joins a relative path onto a directory and normalizes `.` and `..`.
fn resolve(dir: &str, relative: &str) -> String {
    let mut parts: Vec<&str> = dir.split('/').filter(|part| !part.is_empty()).collect();
    for part in relative.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            part => parts.push(part),
        }
    }
    parts.join("/")
}

/// Rewrites a link destination from a markdown source into a site or GitHub URL.
pub fn rewrite_link(dest: &str, context: &Context) -> String {
    if dest.starts_with("http://")
        || dest.starts_with("https://")
        || dest.starts_with("mailto:")
        || dest.starts_with('#')
    {
        return dest.to_string();
    }
    if let Some(site_path) = dest.strip_prefix('/') {
        return format!("{}{site_path}", context.root());
    }
    let (path, fragment) = match dest.split_once('#') {
        Some((path, fragment)) => (path, Some(fragment)),
        None => (dest, None),
    };
    let repo_path = resolve(context.source_dir, path);
    let fragment = fragment.map(|f| format!("#{f}")).unwrap_or_default();
    // A few anchors moved with their content.
    if repo_path == "README.md" && fragment == "#what-it-will-not-be" {
        return context.href("/guide/refusals/");
    }
    if let Some(url) = pages::url_for(&repo_path) {
        return format!("{}{fragment}", context.href(url));
    }
    let on_disk = context.site.root.join(&repo_path);
    if on_disk.is_dir() {
        return format!("https://github.com/shergin/malevich/tree/main/{repo_path}");
    }
    if on_disk.exists() {
        return format!("https://github.com/shergin/malevich/blob/main/{repo_path}{fragment}");
    }
    dest.to_string()
}

/// Copies a repository image into the site and returns its URL.
fn rewrite_image(dest: &str, context: &Context) -> String {
    if dest.starts_with("http://") || dest.starts_with("https://") {
        return dest.to_string();
    }
    let repo_path = resolve(context.source_dir, dest);
    let from = context.site.root.join(&repo_path);
    if !from.exists() {
        return dest.to_string();
    }
    let to = context.site.out.join("assets").join(&repo_path);
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).expect("asset directory");
    }
    fs::copy(&from, &to).unwrap_or_else(|error| panic!("copying {}: {error}", from.display()));
    format!("{}assets/{repo_path}", context.root())
}

/// Replaces every `{{kind arguments}}` line with the HTML the figure registry
/// produces for it. Unknown directives fail the build: a page that names a
/// figure that does not exist is a broken page.
fn expand_directives(
    source: &str,
    context: &Context,
    plates: &Cell<usize>,
    scripts: &mut Vec<String>,
) -> String {
    let mut out = String::with_capacity(source.len());
    let mut in_fence = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
        }
        if !in_fence && trimmed.starts_with("{{") && trimmed.ends_with("}}") {
            let inner = trimmed[2..trimmed.len() - 2].trim();
            let (directive, caption) = match inner.split_once('|') {
                Some((directive, caption)) => (directive.trim(), Some(caption.trim())),
                None => (inner, None),
            };
            let mut words = directive.split_whitespace();
            let kind = words.next().unwrap_or("");
            let arguments: Vec<&str> = words.collect();
            let html = context
                .figures
                .directive(kind, &arguments, caption, context, plates, scripts)
                .unwrap_or_else(|error| panic!("{}: {{{{{inner}}}}}: {error}", context.page.url));
            out.push('\n');
            out.push_str(&html);
            out.push('\n');
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}
