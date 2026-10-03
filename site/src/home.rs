//! The front page.

use crate::markdown::{Body, Context};

pub fn render(context: &Context) -> Body {
    let source =
        std::fs::read_to_string(context.site.root.join("site/content/home.md")).expect("home.md");
    let site = context.site;
    let dependencies: Vec<String> = site
        .dependencies
        .iter()
        .map(|name| format!("<code>{name}</code>"))
        .collect();
    let dependencies = match dependencies.split_last() {
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
        None => "no dependencies".to_string(),
    };
    let source = source
        .replace("CRATE_VERSION", &site.version)
        .replace("RUST_VERSION", &site.rust_version)
        .replace("CRATE_DEPENDENCIES", &dependencies);
    crate::markdown::render(&source, context)
}
