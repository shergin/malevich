//! The page table: every URL on the site, what it is built from, and how the
//! navigation groups it.

/// Where a page's body comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// A markdown file under `site/content/`.
    Content(&'static str),
    /// A markdown file in the repository, rendered with its links rewritten.
    Repo(&'static str),
    /// A repository markdown file with site material inserted after named
    /// headings: the repository stays the single source of the text, the
    /// site adds the plates.
    RepoWith(&'static str, &'static [(&'static str, &'static str)]),
    /// The front page.
    Home,
    /// The gallery, built from `EXAMPLES.md` and the example sources.
    Gallery,
    /// The in-browser plates, drawn by the wasm build.
    Live,
    /// The playground.
    Playground,
}

/// One page.
#[derive(Debug, Clone, Copy)]
pub struct Page {
    /// The site-absolute URL, always a directory (`/guide/marks/`).
    pub url: &'static str,
    /// The title, as the navigation and the browser tab show it.
    pub title: &'static str,
    /// One line for the section index and the search results.
    pub blurb: &'static str,
    /// Where the body comes from.
    pub source: Source,
}

impl Page {
    /// The output path relative to the output directory.
    pub fn path(&self) -> String {
        format!("{}index.html", self.url.trim_start_matches('/'))
    }

    /// The relative prefix that reaches the site root from this page.
    pub fn root(&self) -> String {
        let depth = self
            .url
            .trim_matches('/')
            .split('/')
            .filter(|part| !part.is_empty())
            .count();
        "../".repeat(depth)
    }

    /// The repository directory relative links in this page's source resolve
    /// against.
    pub fn source_dir(&self) -> &'static str {
        match self.source {
            Source::Repo(path) | Source::RepoWith(path, _) => {
                path.rsplit_once('/').map_or("", |(dir, _)| dir)
            }
            _ => "",
        }
    }

    /// Whether the search index lists the page.
    pub fn searchable(&self) -> bool {
        !matches!(self.source, Source::Home)
    }
}

/// A navigation section.
pub struct Section {
    pub title: &'static str,
    /// Built, searched, and paged. Left out of the sidebar when false.
    pub in_nav: bool,
    pub pages: &'static [Page],
}

const fn page(url: &'static str, title: &'static str, blurb: &'static str, source: Source) -> Page {
    Page {
        url,
        title,
        blurb,
        source,
    }
}

/// The navigation, in order. Principles stay published at their URLs and out of
/// the sidebar: Vision links them.
pub const SECTIONS: &[Section] = &[
    Section {
        title: "Start here",
        in_nav: true,
        pages: &[
            page(
                "/",
                "malevich",
                "Terminal plotting for Rust. A small grammar of marks, honest axes, millions of points.",
                Source::Home,
            ),
            page(
                "/guide/start/",
                "Getting started",
                "Install, the first plot, the presets, and what a pipe does to it.",
                Source::Content("guide/start.md"),
            ),
            page(
                "/guide/choose/",
                "Which chart",
                "You have a shape of data. Here is the preset, the plate, and the page that explains it.",
                Source::Content("guide/choose.md"),
            ),
            page(
                "/playground/",
                "Playground",
                "Paste numbers, pick a chart, resize the terminal, and read the Rust that would draw it.",
                Source::Playground,
            ),
        ],
    },
    Section {
        title: "Learn",
        in_nav: true,
        pages: &[
            page(
                "/guide/grammar/",
                "The grammar",
                "One chart, built up mark by mark, on scales the layers share.",
                Source::Content("guide/grammar.md"),
            ),
            page(
                "/guide/composition/",
                "Composition",
                "Small multiples, tables beside charts, shared windows, and the honest forms of a pie.",
                Source::Content("guide/composition.md"),
            ),
            page(
                "/guide/streaming/",
                "Live charts",
                "A sliding window, a repaint in place, and a CLI that plots forever.",
                Source::Content("guide/streaming.md"),
            ),
            page(
                "/guide/recipes/",
                "Recipes",
                "Benchmarks piped through jq, the pie, tornado bars, two scales, and out-of-range rules.",
                Source::RepoWith(
                    "docs/recipes.md",
                    &[
                        (
                            "## Tornado and breakdown bars",
                            "{{figure comp_tornado nocode}}",
                        ),
                        (
                            "## Two series, two scales: a `Grid` and a shared window",
                            "{{pair comp_shared_a comp_shared_b}}",
                        ),
                        (
                            "## Positions clip, colors squish",
                            "{{pair scale_domain_fixed scale_colormap_domain | Positions clip on the left: the curve leaves the fixed window and is drawn nowhere. Colors squish on the right: values past the fixed color domain take the `under` and `over` colors, and the colorbar shows the range.}}",
                        ),
                        ("## Plain text is agent-legible", "{{plain hero}}"),
                    ],
                ),
            ),
        ],
    },
    Section {
        title: "Look up",
        in_nav: true,
        pages: &[
            page(
                "/guide/marks/",
                "The eight marks",
                "Line, Points, Bars, Area, Cells, Range, Rule, Text. Every channel, and a plate for each.",
                Source::Content("guide/marks.md"),
            ),
            page(
                "/guide/stats/",
                "The statistics layer",
                "Bins, densities, quartiles, fits, windows, stacks, M4. What runs before the scales see the data.",
                Source::Content("guide/stats.md"),
            ),
            page(
                "/guide/scales/",
                "Scales and axes",
                "Linear, integer, log, time, and band axes. Ticks that are exact. Colormaps and palettes.",
                Source::Content("guide/scales.md"),
            ),
            page(
                "/guide/furniture/",
                "Furniture",
                "Titles, labels, legends, colorbars, and what sheds first when the frame shrinks.",
                Source::Content("guide/furniture.md"),
            ),
            page(
                "/guide/terminals/",
                "Frames and terminals",
                "The charset and color ladders, what detection reads, the overrides, and a live explorer.",
                Source::RepoWith(
                    "docs/terminal.md",
                    &[
                        (
                            "## The charset ladder",
                            "{{charsets start_layers | One plot value, at every rung. Block tiers become crisp rectangles on this page, because the SVG card draws them as the rectangles they denote. Braille and box drawing stay text, and the font draws them.}}",
                        ),
                        (
                            "## The color ladder",
                            "{{colors ladder_heat | The encoder's own SGR bytes, at every color mode, decoded the way a terminal would draw them. Truecolor, the 256-color cube, the sixteen named colors picked in OKLab, and the plain shade ramp with marker cycling.}}\n\n{{explorer}}",
                        ),
                        ("## Small frames", "{{resizer}}"),
                    ],
                ),
            ),
            page(
                "/guide/interaction/",
                "Interaction",
                "Mapping, Viewport, and the widget. Zoom, pan, and crosshairs, without owning the terminal.",
                Source::RepoWith(
                    "docs/interaction.md",
                    &[(
                        "## The physics",
                        "{{pair inter_full inter_zoomed | A zoom is a domain window. The whole series is on the left. On the right, a `Viewport` over three thousand of its hundred thousand points, where M4 re-aggregates to the new columns, and the ripple the wide view could only hint at is drawn in full.}}",
                    )],
                ),
            ),
            page(
                "/guide/pixels/",
                "Real pixels",
                "Sixel, kitty, and iTerm2 panels, with the text chrome left around them.",
                Source::RepoWith(
                    "docs/pixels.md",
                    &[
                        (
                            "## Turn it on",
                            "![Loss curves, a calendar time axis, and smoothing: cell rendering beside pixel rendering](../examples/showcase-lines.png)\n\n![A 2D density, contour lines, and a vector field: cell rendering beside pixel rendering](../examples/showcase-2d.png)\n\n*The showcase in a kitty terminal. Every chart twice: cells on the left, the same plot value as a real image on the right. Title, axes, and legend stay text. Only the plot rectangle becomes pixels.*",
                        ),
                        (
                            "## On a page",
                            "{{pixelcard hero | One plot value, two SVG cards. The chrome is the same cell card in both. The panel is quadrant cells in the first, and in the second the device-pixel raster a graphics terminal shows, the note at data coordinates in the pixel font.}}",
                        ),
                    ],
                ),
            ),
            page(
                "/guide/notebooks/",
                "Notebooks and cards",
                "Evcxr, the HTML card, the SVG card, and the contract a terminal card keeps.",
                Source::RepoWith(
                    "docs/notebooks.md",
                    &[
                        (
                            "## What a cell shows",
                            "{{html start_layers | The HTML card itself, embedded in this page exactly as `Plot::to_html` emitted it. One `<pre>` of colored spans, and nothing external.}}",
                        ),
                        (
                            "## Custom frames",
                            "{{svgsource start_line}}\n\n{{light start_layers | The SVG card on the light theme. Every figure on this site is this card, inlined into the page.}}",
                        ),
                        (
                            "## The terminal-card contract",
                            "{{plain start_layers | The same grid, with no color: what a stripped card, a log, or a language model reads.}}",
                        ),
                    ],
                ),
            ),
            page(
                "/guide/serde/",
                "Specs as data",
                "A document: the versioned envelope a plot travels in.",
                Source::RepoWith(
                    "docs/serde.md",
                    &[(
                        "## Version 1",
                        "{{figure start_layers nocode}}\n\nThe plot above, as the document `Document::plot` produces it. Every layer, every scale, every piece of furniture, the series inline, and gaps as `null`:\n\n{{json start_layers}}",
                    )],
                ),
            ),
            page(
                "/guide/performance/",
                "Performance",
                "M4 to the raster, bucket-exact grids, and a bench behind the numbers.",
                Source::RepoWith(
                    "docs/performance.md",
                    &[
                        (
                            "## The mechanisms",
                            "{{pair stat_m4 stat_stride | Speed and honesty are the same claim. On the left, M4 keeps the first, last, minimum, and maximum per rendered column, so the three one-sample spikes survive by construction. On the right, a stride sampler is just as fast and silently lost all three.}}\n\n{{figure mark_cells_extents nocode}}",
                        ),
                        (
                            "## Measured",
                            "{{figure perf_bench | The benchmark table, drawn by the library it measures. Horizontal bars on a log axis, with an SI unit, from the 2026-09-24 baseline.}}",
                        ),
                    ],
                ),
            ),
            page(
                "/guide/refusals/",
                "What it will not be",
                "The requests it declines, each with the reason, and the answer that already exists.",
                Source::Content("guide/refusals.md"),
            ),
        ],
    },
    Section {
        title: "Gallery",
        in_nav: true,
        pages: &[
            page(
                "/gallery/",
                "The gallery",
                "Fifty-odd charts, each drawn as a card, with the pipe text one switch away and the source folded under it.",
                Source::Gallery,
            ),
            page(
                "/gallery/live/",
                "In the browser",
                "The same engine, as wasm: cells beside pixels, and ten million points you can zoom.",
                Source::Live,
            ),
        ],
    },
    Section {
        title: "Why it is shaped this way",
        in_nav: true,
        pages: &[page(
            "/principles/",
            "Vision",
            "The argument, and the five rules.",
            Source::Repo("docs/vision.md"),
        )],
    },
    Section {
        title: "Principles",
        in_nav: false,
        pages: &[
            page(
                "/principles/presets-are-packaging/",
                "Presets are packaging",
                "A preset is a name for a grammar expansion, proven byte-identical to that expansion.",
                Source::Repo("docs/principles/presets-are-packaging.md"),
            ),
            page(
                "/principles/frame-is-run-state/",
                "The frame is run state",
                "A plot describes a chart. A frame describes one drawing of it.",
                Source::Repo("docs/principles/frame-is-run-state.md"),
            ),
            page(
                "/principles/full-draw-oracle/",
                "The full draw is the oracle",
                "Anything faster than drawing every point has to land on the same pixels.",
                Source::Repo("docs/principles/full-draw-oracle.md"),
            ),
            page(
                "/principles/what-earns-a-concept/",
                "What earns a concept",
                "Real charts have to demand it, and no composition of the rest can draw it.",
                Source::Repo("docs/principles/what-earns-a-concept.md"),
            ),
            page(
                "/principles/degradation-is-the-contract/",
                "Degradation is the contract",
                "Every terminal gets the best chart it can carry. No terminal gets a failure.",
                Source::Repo("docs/principles/degradation-is-the-contract.md"),
            ),
            page(
                "/principles/axes-are-the-product/",
                "The axes are the product",
                "What sets it apart lives exactly where everyone else got bored.",
                Source::Repo("docs/principles/axes-are-the-product.md"),
            ),
            page(
                "/principles/conversion-at-the-rim/",
                "Conversion lives at the rim",
                "The core computes in f64. Every other numeric shape converts once, on the way in.",
                Source::Repo("docs/principles/conversion-at-the-rim.md"),
            ),
        ],
    },
    Section {
        title: "Reference",
        in_nav: true,
        pages: &[
            page(
                "/concepts/",
                "Terminology",
                "The vocabulary contract. Every public concept, what it means, what it maps to, illustrated.",
                Source::RepoWith(
                    "docs/terminology.md",
                    &[
                        (
                            "## Plot",
                            "[`Plot` on docs.rs](https://docs.rs/malevich/latest/malevich/plot/struct.Plot.html).\n\n{{figure hero nocode}}",
                        ),
                        (
                            "## Layer",
                            "[`Plot::layer` on docs.rs](https://docs.rs/malevich/latest/malevich/plot/struct.Plot.html#method.layer).\n\n{{figure grammar_3 nocode}}",
                        ),
                        (
                            "## Mark",
                            "[`Mark` on docs.rs](https://docs.rs/malevich/latest/malevich/mark/enum.Mark.html).\n\n{{figure mark_rule nocode}}",
                        ),
                        ("## Channel", "{{figure mark_points_color_by nocode}}"),
                        (
                            "## Series",
                            "[`Series` on docs.rs](https://docs.rs/malevich/latest/malevich/data/struct.Series.html).\n\n{{figure start_gap nocode}}",
                        ),
                        (
                            "## Stat",
                            "[`stat` on docs.rs](https://docs.rs/malevich/latest/malevich/stat/index.html).\n\n{{figure stat_window nocode}}",
                        ),
                        (
                            "## Reducer",
                            "[`Reducer` on docs.rs](https://docs.rs/malevich/latest/malevich/stat/enum.Reducer.html).\n\n{{figure stat_binned nocode}}",
                        ),
                        (
                            "## Scale",
                            "[`Scale` on docs.rs](https://docs.rs/malevich/latest/malevich/scale/enum.Scale.html).\n\n{{figure scale_log nocode}}",
                        ),
                        (
                            "## Ticks",
                            "[`Ticks` on docs.rs](https://docs.rs/malevich/latest/malevich/scale/struct.Ticks.html).\n\n{{figure scale_context nocode}}",
                        ),
                        (
                            "## Frame",
                            "[`Frame` on docs.rs](https://docs.rs/malevich/latest/malevich/plot/struct.Frame.html).\n\n{{sizes start_layers 60x12 36x8 | One plot, two frames.}}",
                        ),
                        (
                            "## Viewport",
                            "[`Viewport` on docs.rs](https://docs.rs/malevich/latest/malevich/plot/struct.Viewport.html).\n\n{{pair inter_full inter_zoomed}}",
                        ),
                        (
                            "## Card",
                            "[`Plot::to_svg` on docs.rs](https://docs.rs/malevich/latest/malevich/plot/struct.Plot.html#method.to_svg).\n\n{{light hero}}",
                        ),
                        (
                            "## Charset",
                            "[`Charset` on docs.rs](https://docs.rs/malevich/latest/malevich/render/enum.Charset.html).\n\n{{charsets start_line}}",
                        ),
                        (
                            "## Theme",
                            "[`Theme` on docs.rs](https://docs.rs/malevich/latest/malevich/struct.Theme.html).\n\n{{pair palette_okabe_ito palette_muted | The categorical `Palette` lives in the spec. The `Theme` belongs to the frame.}}",
                        ),
                        (
                            "## Preset",
                            "[`line` on docs.rs](https://docs.rs/malevich/latest/malevich/fn.line.html).\n\n{{figure grammar_preset nocode}}",
                        ),
                        (
                            "## Stream",
                            "[`stream` on docs.rs](https://docs.rs/malevich/latest/malevich/stream/index.html).\n\n{{figure stream_tail nocode}}",
                        ),
                        (
                            "## Mapping",
                            "[`Mapping` on docs.rs](https://docs.rs/malevich/latest/malevich/plot/struct.Mapping.html).",
                        ),
                        (
                            "## Widget",
                            "[`PlotWidget` on docs.rs](https://docs.rs/malevich/latest/malevich/struct.PlotWidget.html).",
                        ),
                        (
                            "## Grid",
                            "[`Grid` on docs.rs](https://docs.rs/malevich/latest/malevich/plot/struct.Grid.html).",
                        ),
                    ],
                ),
            ),
            page(
                "/cli/",
                "kaz, the command line",
                "Pipe data to an honest plot. One subcommand per chart, the plot on stderr, the data still flowing.",
                Source::Repo("cli/README.md"),
            ),
            page(
                "/js/",
                "JavaScript",
                "The same engine as wasm on npm, and an Ink widget.",
                Source::Repo("js/README.md"),
            ),
            page(
                "/benchmarks/",
                "Benchmarks",
                "The dated record behind every number these docs quote.",
                Source::Repo("BENCHMARKS.md"),
            ),
            page(
                "/changelog/",
                "Changelog",
                "Every release, written for a person.",
                Source::Repo("CHANGELOG.md"),
            ),
            page(
                "/acknowledgements/",
                "Acknowledgements",
                "The algorithms, libraries, and grammars this project learned from, one by one.",
                Source::Repo("ACKNOWLEDGEMENTS.md"),
            ),
        ],
    },
];

/// Every page, flattened.
pub fn all() -> impl Iterator<Item = &'static Page> {
    SECTIONS.iter().flat_map(|section| section.pages.iter())
}

/// The site URL a repository path is published at, if any.
pub fn url_for(repo_path: &str) -> Option<&'static str> {
    let path = repo_path.trim_start_matches("./");
    match path {
        "README.md" => return Some("/"),
        "EXAMPLES.md" => return Some("/gallery/"),
        "docs/README.md" => return Some("/guide/start/"),
        "docs/terminology.md" | "TERMINOLOGY.md" => return Some("/concepts/"),
        "docs/terminal.md" => return Some("/guide/terminals/"),
        "docs/notebooks.md" => return Some("/guide/notebooks/"),
        "docs/serde.md" | "SERDE.md" => return Some("/guide/serde/"),
        "docs/performance.md" => return Some("/guide/performance/"),
        "docs/principles" | "docs/principles/" => return Some("/principles/"),
        "gallery/README.md" | "site/README.md" => return Some("/gallery/live/"),
        "demos/README.md" | "demos" | "demos/" => return Some("/guide/interaction/"),
        "js/examples" | "js/examples/" => return Some("/js/"),
        "cli" | "cli/" => return Some("/cli/"),
        _ => {}
    }
    all()
        .find(|page| matches!(page.source, Source::Repo(source) | Source::RepoWith(source, _) if source == path))
        .map(|page| page.url)
}
