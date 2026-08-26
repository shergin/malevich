//! Regenerates every chart embedded in the docs, or verifies they are current.
//!
//! Two mechanisms, one honesty rule — no chart in any markdown file is typed by a
//! human; every one is real program output:
//!
//! - `EXAMPLES.md` is built whole from the gallery examples.
//! - Any markdown file may embed `<!-- generated:NAME -->` … `<!-- /generated -->`;
//!   the block between the markers is replaced with the stdout of
//!   `cargo run --example NAME` in a `text` fence.
//!
//! CI runs this with `--check` and fails on any stale file. Examples used here must
//! render fixed `Frame::plain` or `Frame::portable` frames so their output is
//! deterministic.

use std::process::Command;

/// The gallery: example name and the one-line story it tells.
const GALLERY: &[(&str, &str)] = &[
    (
        "sine",
        "Function sampling: curves drawn from `f(x)`, one sample per subpixel column.",
    ),
    (
        "loss",
        "A real training log: poorgrad's bigram model on 32k names — per-step loss, \
         rolling mean, and the known bigram limit as a rule.",
    ),
    (
        "languages",
        "Categorical bars from a zero baseline, with eighth-block precision at the top.",
    ),
    (
        "clusters",
        "Palmer penguins through one color_by channel: categories take palette \
         colors, name themselves in the legend, and cycle marker shapes in \
         colorless output.",
    ),
    (
        "volcano",
        "A volcano plot from the grammar, no preset: significance classes via \
         color_by, thresholds as Rules, grey pinned to the insignificant mass.",
    ),
    (
        "manhattan",
        "A Manhattan plot from the grammar, no preset: chromosomes alternate two \
         shades as unlabeled layers, the genome-wide threshold is a labeled Rule.",
    ),
    (
        "candles",
        "Candlesticks from the grammar, no preset: Range whiskers and bodies with \
         up/down days split by color_by.",
    ),
    (
        "fit",
        "Least squares as a stat: scatter, trend line, and a 95% confidence band \
         from one mergeable Fit accumulator — slope, intercept, and R² included.",
    ),
    (
        "qq",
        "A Q\u{2013}Q plot from the grammar, no preset: matched type-7 quantiles of two \
         samples against the identity line — the heavy tail peels off it.",
    ),
    (
        "waveform",
        "Ten million points through the auto-inserted M4 aggregation — pixel-identical \
         to drawing every point, in tens of milliseconds.",
    ),
    (
        "distribution",
        "Penguin body mass through automatic binning: a real, lumpy distribution.",
    ),
    (
        "powerlaw",
        "Log-log axes: power laws render straight, with decade ticks on both axes.",
    ),
    (
        "energy",
        "Stacked areas via the Stack stat: each layer sits on the sum of the ones below.",
    ),
    (
        "annotated",
        "Annotations: a Rule for the target line, a Text note at data coordinates.",
    ),
    (
        "correlation",
        "Signed data on a diverging colormap centered at zero: correlation and anti-correlation read as opposite colors, and the colorbar spans symmetrically.",
    ),
    (
        "confusion",
        "A confusion matrix from the grammar, no preset: a Cells matrix on Bands \
         axes — class names label rows and columns, counts sit on the cells as \
         Text, and row 0 is the top band so the chart reads in matrix order.",
    ),
    (
        "attention",
        "An attention map: token labels on both axes, a logarithmic colormap so \
         weights spanning decades stay distinguishable, and the causal mask's \
         zeros rendered as honest gaps — with decade ticks on the colorbar.",
    ),
    (
        "filters",
        "Convolution filters as images: a Gabor bank with color opponency \
         through Cells::rgb — direct colors, no colormap, and a luma shade \
         ramp when the output is a plain pipe.",
    ),
    (
        "boundary",
        "A decision boundary from the grammar, no preset: Cells::classes colors \
         the feature plane by predicted class through the categorical palette, \
         each region keeps a stable shade with matching legend swatches, and the \
         training points sit on top.",
    ),
    (
        "density2d",
        "A 2D histogram: point density on a grid, empty bins honestly blank.",
    ),
    (
        "contour",
        "The MATLAB peaks function as iso-lines: marching squares, tick-chosen levels, a labeled legend.",
    ),
    (
        "quiver",
        "A vector field: spiral flow into a sink, one arrow per grid point, drawn in data coordinates.",
    ),
    (
        "boxes",
        "Box plots: type-7 quartiles, Tukey whiskers, outliers — one Range mark with          body and marker channels per category.",
    ),
    (
        "violins",
        "The same flippers as mirrored kernel densities — separation as a shape, \
         not a summary.",
    ),
    (
        "measurements",
        "Error bars: a Range interval around each measured point.",
    ),
    (
        "timeseries",
        "The Keeling curve: monthly CO2 at Mauna Loa since 1958 (NOAA), on a calendar axis.",
    ),
    (
        "multiples",
        "Small multiples: a Grid of independent plots, axes shared by fixing          domains explicitly.",
    ),
    (
        "corners",
        "The asciichart homage: box-drawing corners, one glyph per column — with real axes underneath.",
    ),
    (
        "steps",
        "Step charts: stairs hold values flat between indices; an ECDF climbs a distribution from zero to one.",
    ),
    (
        "charsets",
        "The charset ladder: one curve at every subpixel density — solid blocks (octants, sextants, quadrants, half blocks), braille dots, and plain ASCII.",
    ),
];

/// Markdown files scanned for `<!-- generated:NAME -->` blocks.
const SPLICED: &[&str] = &["README.md"];

/// Examples that are deliberately not in the gallery: infrastructure, the colored
/// tour (environment-dependent), interactive demos, README splice sources, and the
/// pixel/HTML demos (image escapes and HTML have no place in a markdown gallery).
const EXEMPT: &[&str] = &[
    "regen_docs",
    "showcase",
    "live",
    "tui",
    "readme_sample",
    "readme_bars",
    "pixels",
    "evcxr",
    "suprematist",
    "promo",
];

fn main() {
    let check = std::env::args().any(|argument| argument == "--check");
    let mut stale = Vec::new();

    // No example may silently drop out of the docs: everything in examples/ must
    // be in the gallery, spliced into a doc, or explicitly exempt above.
    let mut missing = Vec::new();
    for entry in std::fs::read_dir("examples").expect("examples directory") {
        let name = entry.expect("directory entry").file_name();
        let Some(name) = name.to_str().and_then(|n| n.strip_suffix(".rs")) else {
            continue;
        };
        if EXEMPT.contains(&name) || GALLERY.iter().any(|(listed, _)| *listed == name) {
            continue;
        }
        missing.push(name.to_string());
    }
    assert!(
        missing.is_empty(),
        "examples missing from the gallery (list them or exempt them): {missing:?}"
    );

    let gallery = gallery_content();
    apply("EXAMPLES.md", gallery, check, &mut stale);

    for path in SPLICED {
        let content = std::fs::read_to_string(path)
            .unwrap_or_else(|error| panic!("failed to read {path}: {error}"));
        apply(path, splice(&content), check, &mut stale);
    }

    if check {
        if stale.is_empty() {
            println!("All generated docs are current.");
        } else {
            eprintln!(
                "Stale generated docs: {}. Run: cargo run --example regen_docs",
                stale.join(", ")
            );
            std::process::exit(1);
        }
    }
}

/// Runs one example and returns its stdout with the trailing newline trimmed.
fn output_of(name: &str) -> String {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let output = Command::new(cargo)
        .args(["run", "--quiet", "--example", name])
        .output()
        .expect("failed to run cargo");
    assert!(
        output.status.success(),
        "example {name} failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("example output is not UTF-8")
        .trim_end_matches('\n')
        .to_string()
}

fn gallery_content() -> String {
    let mut content = String::from(
        "<!-- GENERATED FILE — do not edit. Every byte of this file is produced by\n\
         examples/regen_docs.rs from the gallery examples; edit those instead and\n\
         run `cargo run --example regen_docs`. -->\n\n\
         # Gallery\n\n\
         The showcase and the system test in one artifact. This whole file is\n\
         generated from the examples (unlike README.md, which splices marked\n\
         blocks); regenerate with `cargo run --example regen_docs` — CI fails when\n\
         it is stale. Every example renders a fixed deterministic frame, so output\n\
         is deterministic.\n",
    );
    for (name, story) in GALLERY {
        content.push_str(&format!(
            "\n## {name}\n\n{story}\nSource: [examples/{name}.rs](examples/{name}.rs)\n\n\
             ```text\n{}\n```\n",
            output_of(name)
        ));
    }
    content
}

/// Replaces every `<!-- generated:NAME -->` block with the named example's output.
fn splice(content: &str) -> String {
    const OPEN: &str = "<!-- generated:";
    const OPEN_END: &str = " -->";
    const CLOSE: &str = "<!-- /generated -->";

    let mut result = String::with_capacity(content.len());
    let mut rest = content;
    while let Some(start) = rest.find(OPEN) {
        let (head, tail) = rest.split_at(start);
        result.push_str(head);
        let name_end = tail[OPEN.len()..]
            .find(OPEN_END)
            .expect("unterminated generated marker");
        let name = &tail[OPEN.len()..OPEN.len() + name_end];
        let marker_end = OPEN.len() + name_end + OPEN_END.len();
        result.push_str(&tail[..marker_end]);
        let close = tail.find(CLOSE).expect("missing closing generated marker");
        result.push_str(&format!("\n```text\n{}\n```\n", output_of(name)));
        rest = &tail[close..];
    }
    result.push_str(rest);
    result
}

/// Writes `content` to `path`, or in check mode records staleness.
fn apply(path: &str, content: String, check: bool, stale: &mut Vec<String>) {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    if existing == content {
        return;
    }
    if check {
        stale.push(path.to_string());
    } else {
        std::fs::write(path, content)
            .unwrap_or_else(|error| panic!("failed to write {path}: {error}"));
        println!("{path} regenerated.");
    }
}
