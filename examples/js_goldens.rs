//! Shared JS/Rust goldens: a list of (Document, Frame, expected string).
//!
//!   cargo run --example js_goldens --features serde           # write
//!   cargo run --example js_goldens --features serde -- --check
//!
//! The JS suite renders the same documents through wasm and diffs the strings.
//! One oracle — the crate — and one fixture both sides read.

use malevich::scale::{Colormap, Palette};
use malevich::{
    Area, Bars, Cells, Document, Frame, Line, LineStyle, Plot, Range, Rule, Text, bar, heatmap,
    hist, line,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize)]
struct Golden {
    name: String,
    document: serde_json::Value,
    frame: serde_json::Value,
    expected: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/js/goldens.json");
    let goldens = goldens()?;
    let encoded = serde_json::to_string_pretty(&goldens)?;
    if std::env::args().any(|arg| arg == "--check") {
        let on_disk = std::fs::read_to_string(&path)?;
        if on_disk.trim() != encoded.trim() {
            eprintln!("js goldens stale; run: cargo run --example js_goldens --features serde");
            std::process::exit(1);
        }
        eprintln!("{} goldens current", goldens.len());
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, encoded)?;
    eprintln!("wrote {} goldens to {}", goldens.len(), path.display());
    Ok(())
}

fn goldens() -> Result<Vec<Golden>, Box<dyn std::error::Error>> {
    let plain = Frame::plain(40, 10);
    let portable = Frame::portable(40, 10);
    let mut out = Vec::new();
    push(&mut out, "line", line(&[1.0, 5.0, 2.0, 8.0][..]), &plain)?;
    push(
        &mut out,
        "bar",
        bar(["mon", "tue", "wed"], &[3.0, 7.0, 4.5][..]),
        &plain,
    )?;
    push(
        &mut out,
        "hist",
        hist(&[1.0, 2.0, 2.5, 2.7, 3.0, 3.1, 3.2, 4.0, 5.5][..]),
        &plain,
    )?;
    push(
        &mut out,
        "annotated",
        Plot::new()
            .layer(Line::y(&[1.0, 5.0, 2.0, 8.0][..]))
            .layer(Rule::h(4.0).label("mid"))
            .layer(Text::at(2.0, 6.0, "hi"))
            .title("annotated"),
        &plain,
    )?;
    push(
        &mut out,
        "area",
        Plot::new().layer(Area::y(&[1.0, 3.0, 2.0, 4.0][..]).label("fill")),
        &plain,
    )?;
    push(
        &mut out,
        "stacked-bars",
        Plot::new()
            .layer(Bars::new(["a", "b", "c"], &[1.0, 2.0, 1.5][..]).label("low"))
            .layer(
                Bars::new(["a", "b", "c"], &[0.5, 1.0, 0.5][..])
                    .base(&[1.0, 2.0, 1.5][..])
                    .label("high"),
            )
            .title("stack"),
        &plain,
    )?;
    push(
        &mut out,
        "cells",
        Plot::new()
            .layer(
                Cells::matrix(4, &(0..16).map(f64::from).collect::<Vec<_>>()[..])
                    .colormap(Colormap::VIRIDIS),
            )
            .title("grid"),
        &portable,
    )?;
    push(
        &mut out,
        "range",
        Plot::new()
            .layer(
                Range::over(["a", "b"], &[1.0, 2.0][..], &[4.0, 5.0][..])
                    .body(&[2.0, 3.0][..], &[3.0, 4.0][..])
                    .marker(&[2.5, 3.5][..])
                    .label("box"),
            )
            .title("intervals"),
        &plain,
    )?;
    push(
        &mut out,
        "heatmap",
        heatmap(4, &(0..16).map(f64::from).collect::<Vec<_>>()[..]),
        &portable,
    )?;
    push(
        &mut out,
        "color-by",
        Plot::new()
            .layer(
                Line::y(&[1.0, 2.0, 3.0, 4.0][..])
                    .color_by(["a", "a", "b", "b"])
                    .style(LineStyle::Corners),
            )
            .palette(Palette::OKABE_ITO)
            .title("groups"),
        &plain,
    )?;
    Ok(out)
}

fn push(
    out: &mut Vec<Golden>,
    name: &str,
    plot: Plot<'_>,
    frame: &Frame,
) -> Result<(), Box<dyn std::error::Error>> {
    let expected = plot.render(frame);
    let document = Document::plot(plot)?;
    out.push(Golden {
        name: name.to_string(),
        document: serde_json::to_value(&document)?,
        frame: serde_json::to_value(frame)?,
        expected,
    });
    Ok(())
}
