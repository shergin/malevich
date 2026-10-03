//! The generator behind <https://shergin.github.io/malevich/>.
//!
//! One rule from the repository applies here too: no chart on the site is
//! drawn by hand. Every figure is a plot built in `site/figures/*.rs`,
//! rendered by the library at build time — as the SVG terminal card, as the
//! ANSI a tty would receive, or as the plain text a pipe would see — and the
//! code shown beside it is the code that ran, included verbatim.
//!
//! ```sh
//! cargo run -p malevich-site            # writes site/dist
//! ./site/build.sh                       # the same, plus the wasm the live pages use
//! ```

mod ansi;
mod figures;
mod gallery;
mod highlight;
mod home;
mod layout;
mod markdown;
mod pages;
mod playground;

use std::fs;
use std::path::{Path, PathBuf};

use pages::{Page, Source};

/// Everything a page needs to know about where it lives.
pub struct Site {
    /// The repository root.
    pub root: PathBuf,
    /// The output directory.
    pub out: PathBuf,
    /// The crate version, from the root manifest.
    pub version: String,
    /// The minimum Rust, from the root manifest.
    pub rust_version: String,
    /// The crate's required dependencies, from the root manifest, in order.
    pub dependencies: Vec<String>,
    /// The CLI version, from its manifest.
    pub cli_version: String,
    /// The npm version, from `js/package.json`.
    pub js_version: String,
}

/// One entry in the client-side search index.
#[derive(serde::Serialize)]
struct SearchEntry {
    url: String,
    title: String,
    section: String,
    headings: Vec<HeadingHit>,
    text: String,
}

/// A heading the search box can jump to. The id is the page's own anchor.
#[derive(serde::Serialize)]
struct HeadingHit {
    text: String,
    id: String,
}

fn main() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = manifest
        .parent()
        .expect("site sits inside the repo")
        .to_path_buf();
    let mut out = manifest.join("dist");
    let mut api_docs = None;
    let mut arguments = std::env::args().skip(1);
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--out" => out = PathBuf::from(arguments.next().expect("--out takes a directory")),
            "--api-docs" => {
                api_docs = Some(PathBuf::from(
                    arguments
                        .next()
                        .expect("--api-docs takes a rustdoc directory"),
                ));
            }
            other => panic!("unknown argument {other}"),
        }
    }

    let site = Site {
        version: manifest_version(&root.join("Cargo.toml")),
        rust_version: manifest_field(&root.join("Cargo.toml"), "rust-version"),
        dependencies: required_dependencies(&root.join("Cargo.toml")),
        cli_version: manifest_version(&root.join("cli/Cargo.toml")),
        js_version: package_version(&root.join("js/package.json")),
        root,
        out,
    };

    prepare(&site.out);
    copy_dir(&site.root.join("site/static"), &site.out);
    let figures = figures::Registry::new();
    let mut index = Vec::new();
    let mut written = 0;
    let mut broken_api_links = Vec::new();
    for section in pages::SECTIONS {
        for page in section.pages {
            let rendered = render_page(&site, &figures, section.title, page);
            if let Some(docs) = &api_docs {
                broken_api_links.extend(
                    broken_api_links_in(docs, &rendered.html)
                        .into_iter()
                        .map(|link| format!("{} → {link}", page.url)),
                );
            }
            write(&site.out.join(page.path()), &rendered.html);
            written += 1;
            if page.searchable() {
                index.push(SearchEntry {
                    url: page.url.to_string(),
                    title: page.title.to_string(),
                    section: section.title.to_string(),
                    headings: rendered.headings,
                    text: rendered.summary,
                });
            }
        }
    }
    // A contact sheet of every figure, for proofing: SITE_CONTACT_SHEET=1.
    if std::env::var_os("SITE_CONTACT_SHEET").is_some() {
        let page = Page {
            url: "/contact/",
            title: "Every figure",
            blurb: "The contact sheet.",
            source: Source::Content("contact.md"),
        };
        let mut source = String::new();
        for name in figures.names() {
            source.push_str(&format!("## {name}\n\n{{{{figure {name}}}}}\n\n"));
        }
        let context = markdown::Context {
            site: &site,
            figures: &figures,
            page: &page,
            source_dir: "",
        };
        let body = markdown::render(&source, &context);
        write(
            &site.out.join("contact/index.html"),
            &layout::page(&site, "Proofing", &page, &body),
        );
    }
    assert!(
        broken_api_links.is_empty(),
        "docs.rs links with no matching item in the local rustdoc:\n{}",
        broken_api_links.join("\n")
    );
    let index = serde_json::to_string(&index).expect("search index serializes");
    write(&site.out.join("search.json"), &index);
    write(&site.out.join(".nojekyll"), "");
    let assets = site.out.join("assets/examples");
    for name in [
        "showcase-lines.png",
        "showcase-2d.png",
        "suprematist-composition.png",
    ] {
        fs::create_dir_all(&assets).expect("assets directory");
        fs::copy(site.root.join("examples").join(name), assets.join(name))
            .unwrap_or_else(|error| panic!("copying {name}: {error}"));
    }
    println!(
        "wrote {written} pages and {} figures to {}",
        figures.rendered(),
        site.out.display()
    );
}

/// A rendered page plus what the search index keeps of it.
struct RenderedPage {
    html: String,
    headings: Vec<HeadingHit>,
    summary: String,
}

fn render_page(
    site: &Site,
    figures: &figures::Registry,
    section: &str,
    page: &Page,
) -> RenderedPage {
    let context = markdown::Context {
        site,
        figures,
        page,
        source_dir: page.source_dir(),
    };
    let body = match page.source {
        Source::Content(path) => {
            markdown::render(&read(&site.root.join("site/content").join(path)), &context)
        }
        Source::Repo(path) => markdown::render(&read(&site.root.join(path)), &context),
        Source::RepoWith(path, inserts) => {
            markdown::render(&illustrate(&read(&site.root.join(path)), inserts), &context)
        }
        Source::Home => home::render(&context),
        Source::Gallery => gallery::render(&context),
        Source::Live => gallery::live(&context),
        Source::Playground => playground::render(&context),
    };
    let html = layout::page(site, section, page, &body);
    RenderedPage {
        html,
        headings: body
            .headings
            .iter()
            .map(|heading| HeadingHit {
                text: heading.text.clone(),
                id: heading.id.clone(),
            })
            .collect(),
        summary: body.summary,
    }
}

/// Inserts site material after the named headings of a repository document.
/// A heading that does not exist fails the build: the insert would vanish silently.
fn illustrate(source: &str, inserts: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(source.len() + 1024);
    let mut pending: Vec<&(&str, &str)> = inserts.iter().collect();
    for line in source.lines() {
        out.push_str(line);
        out.push('\n');
        if let Some(index) = pending
            .iter()
            .position(|(heading, _)| *heading == line.trim_end())
        {
            let (_, material) = pending.remove(index);
            out.push('\n');
            out.push_str(material);
            out.push('\n');
        }
    }
    assert!(
        pending.is_empty(),
        "headings not found for inserts: {:?}",
        pending.iter().map(|(h, _)| h).collect::<Vec<_>>()
    );
    out
}

fn prepare(out: &Path) {
    // Keep `wasm/` — build.sh fills it after this generator runs, and a local
    // iteration should not have to rebuild it — and replace everything else.
    if out.exists() {
        for entry in fs::read_dir(out).expect("output directory") {
            let entry = entry.expect("directory entry");
            if entry.file_name() == "wasm" {
                continue;
            }
            let path = entry.path();
            if path.is_dir() {
                fs::remove_dir_all(&path).expect("remove stale directory");
            } else {
                fs::remove_file(&path).expect("remove stale file");
            }
        }
    }
    fs::create_dir_all(out).expect("create output directory");
}

fn copy_dir(from: &Path, to: &Path) {
    for entry in fs::read_dir(from).unwrap_or_else(|error| panic!("{}: {error}", from.display())) {
        let entry = entry.expect("directory entry");
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            fs::create_dir_all(&target).expect("create directory");
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).expect("copy static file");
        }
    }
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("reading {}: {error}", path.display()))
}

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create page directory");
    }
    fs::write(path, content).unwrap_or_else(|error| panic!("writing {}: {error}", path.display()));
}

/// The links in `html` into this crate's API on docs.rs that a local
/// `cargo doc --no-deps --all-features` build does not have: no such page, or
/// no element with the link's anchor on it.
fn broken_api_links_in(docs: &Path, html: &str) -> Vec<String> {
    const PREFIX: &str = "https://docs.rs/malevich/latest/malevich/";
    assert!(
        docs.join("malevich/index.html").is_file(),
        "no rustdoc in {}: run cargo doc --no-deps --all-features",
        docs.display()
    );
    let mut broken = Vec::new();
    for rest in html.split("href=\"").skip(1) {
        let Some(link) = rest
            .split('"')
            .next()
            .and_then(|href| href.strip_prefix(PREFIX))
        else {
            continue;
        };
        let (path, anchor) = match link.split_once('#') {
            Some((path, anchor)) => (path, Some(anchor)),
            None => (link, None),
        };
        let found = fs::read_to_string(docs.join("malevich").join(path)).is_ok_and(|page| {
            anchor.is_none_or(|anchor| page.contains(&format!("id=\"{anchor}\"")))
        });
        if !found && !broken.iter().any(|seen| seen == link) {
            broken.push(link.to_string());
        }
    }
    broken
}

fn manifest_version(path: &Path) -> String {
    manifest_field(path, "version")
}

/// The first `key = "value"` line of a manifest.
fn manifest_field(path: &Path, key: &str) -> String {
    let prefix = format!("{key} = \"");
    read(path)
        .lines()
        .find_map(|line| line.strip_prefix(&prefix)?.strip_suffix('"'))
        .unwrap_or_else(|| panic!("{} has no {key}", path.display()))
        .to_string()
}

/// The names under `[dependencies]` that are not `optional`.
fn required_dependencies(path: &Path) -> Vec<String> {
    read(path)
        .lines()
        .skip_while(|line| line.trim() != "[dependencies]")
        .skip(1)
        .take_while(|line| !line.starts_with('['))
        .filter(|line| !line.trim_start().starts_with('#') && !line.contains("optional = true"))
        .filter_map(|line| Some(line.split_once('=')?.0.trim().to_string()))
        .filter(|name| !name.is_empty())
        .collect()
}

fn package_version(path: &Path) -> String {
    let package: serde_json::Value =
        serde_json::from_str(&read(path)).expect("package.json parses");
    package["version"]
        .as_str()
        .expect("package.json has a version")
        .to_string()
}
