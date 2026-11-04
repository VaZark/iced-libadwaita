//! The layer rule, enforced: each layer of `src/` imports only the layers it may (see DESIGN.md,
//! "Layers"). Comments and `#[cfg(test)]` code are ignored.

use std::fs;
use std::path::{Path, PathBuf};

/// (layer, the layers it may import besides itself)
const ALLOWED: &[(&str, &[&str])] = &[
    ("primitives", &[]),
    ("layouts", &["binds", "primitives", "styles"]),
    ("binds", &["primitives", "styles"]),
    ("components", &["layouts", "binds", "primitives", "styles"]),
    (
        "metas",
        &["components", "layouts", "binds", "primitives", "styles"],
    ),
    (
        "widgets",
        &[
            "metas",
            "components",
            "layouts",
            "binds",
            "primitives",
            "styles",
            "windows",
        ],
    ),
    ("styles", &[]),
    ("windows", &["layouts", "styles"]),
];

/// The names of `crate::<name>` that are layers.
const LAYERS: &[&str] = &[
    "primitives",
    "layouts",
    "binds",
    "components",
    "metas",
    "widgets",
    "styles",
    "windows",
];

/// What the headless layer may not take from iced: widgets and events. Geometry value types
/// (`Point`, `Size`, `Rectangle`, `Vector`, `Padding`) are fine.
const NOT_IN_PRIMITIVES: &[&str] = &[
    "iced::widget",
    "iced::Element",
    "iced::Event",
    "iced::advanced",
    "iced::mouse",
    "iced::keyboard",
    "iced::touch",
    "iced::window",
];

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap().flatten() {
        let p = entry.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

/// The code of a file: no comment lines, nothing from the first `#[cfg(test)]` on.
fn code(src: &str) -> String {
    let mut out = String::new();
    for line in src.lines() {
        let t = line.trim_start();
        if t.starts_with("#[cfg(test)]") {
            break;
        }
        if t.starts_with("//") {
            continue;
        }
        out.push_str(line.split("//").next().unwrap_or(""));
        out.push('\n');
    }
    out
}

fn ident(s: &str) -> &str {
    let end = s
        .find(|c: char| !(c.is_alphanumeric() || c == '_'))
        .unwrap_or(s.len());
    &s[..end]
}

/// Every `crate::<ident>` target in `code`, including the items of `crate::{a, b::c}`.
fn crate_targets(code: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = code;
    while let Some(i) = rest.find("crate::") {
        rest = &rest[i + "crate::".len()..];
        if let Some(group) = rest.strip_prefix('{') {
            let mut depth = 1;
            let mut item = String::new();
            let mut items = Vec::new();
            for c in group.chars() {
                match c {
                    '{' => depth += 1,
                    '}' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    ',' if depth == 1 => {
                        items.push(std::mem::take(&mut item));
                        continue;
                    }
                    _ => {}
                }
                item.push(c);
            }
            items.push(item);
            out.extend(items.iter().map(|i| ident(i.trim()).to_string()));
        } else {
            out.push(ident(rest).to_string());
        }
    }
    out
}

#[test]
fn layers_import_only_what_they_may() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut broken = Vec::new();
    for (layer, may) in ALLOWED {
        let mut files = Vec::new();
        rust_files(&src.join(layer), &mut files);
        assert!(!files.is_empty(), "src/{layer} has no files");
        for f in files {
            let code = code(&fs::read_to_string(&f).unwrap());
            for target in crate_targets(&code) {
                if LAYERS.contains(&target.as_str())
                    && target != *layer
                    && !may.contains(&target.as_str())
                {
                    broken.push(format!("{}: {layer} may not import {target}", f.display()));
                }
            }
            if *layer == "primitives" {
                for banned in NOT_IN_PRIMITIVES {
                    if code.contains(banned) {
                        broken.push(format!("{}: primitives may not use {banned}", f.display()));
                    }
                }
            }
        }
    }
    assert!(
        broken.is_empty(),
        "layer violations:\n{}",
        broken.join("\n")
    );
}
