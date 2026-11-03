//! Repository tasks.
//!
//! ```text
//! cargo xtask man            # write man/iced-libadwaita-<component>.3 for every component
//! cargo xtask man --check    # fail if the checked-in pages are out of date (for CI)
//! ```
//!
//! The man pages are generated from the source, as rustdoc or JSDoc generates reference pages:
//! the `///` doc comments and the public API of every component in `src/widget` and
//! `src/window/dialog.rs`. A component is a public struct that becomes an `Element`
//! (`impl From<X> for Element`) or is a `Widget`. Its builder methods (`pub fn f(mut self, ..) ->
//! Self`) are its properties; those taking an `Element` are slots, and `on_*` ones are events.
//! Write the doc comment and the page follows; there is nothing else to keep in step.

use quote::ToTokens;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use syn::{FnArg, ImplItem, Item, Type, Visibility};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("man") => {
            let check = args.iter().any(|a| a == "--check");
            if !man(check) {
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("usage: cargo xtask man [--check]");
            std::process::exit(2);
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Extraction
// ---------------------------------------------------------------------------------------------

/// One public function or method.
#[derive(Debug, Clone, Default)]
struct Func {
    name: String,
    /// `name(arg: Type, ..)`, without the receiver.
    signature: String,
    doc: String,
}

#[derive(Debug, Default)]
struct Component {
    name: String,
    /// The name it is exported under from `iced_libadwaita::widget`, in snake case.
    alias: String,
    file: String,
    doc: String,
    /// The `//!` of the file, when it holds this one component.
    module_doc: String,
    /// `Type::new(..)` and other associated functions without a receiver.
    constructors: Vec<Func>,
    /// Free functions that build it.
    functions: Vec<Func>,
    properties: Vec<Func>,
    slots: Vec<Func>,
    events: Vec<Func>,
    methods: Vec<Func>,
    /// A component that is a function (`button(..)`, `scale(..)`), with its return type.
    returns: Option<String>,
}

fn doc_of(attrs: &[syn::Attribute]) -> String {
    let mut lines = Vec::new();
    for a in attrs {
        if !a.path().is_ident("doc") {
            continue;
        }
        if let syn::Meta::NameValue(nv) = &a.meta
            && let syn::Expr::Lit(l) = &nv.value
            && let syn::Lit::Str(s) = &l.lit
        {
            let v = s.value();
            lines.push(v.strip_prefix(' ').unwrap_or(&v).to_string());
        }
    }
    lines.join("\n")
}

/// Tokens as source-like text: `impl Into < String >` becomes `impl Into<String>`.
fn tidy(t: impl ToTokens) -> String {
    let mut s = t.to_token_stream().to_string();
    for (from, to) in [
        (" :: ", "::"),
        (":: ", "::"),
        (" < ", "<"),
        (" <", "<"),
        ("< ", "<"),
        (" >", ">"),
        (" ,", ","),
        ("& ", "&"),
        ("Fn (", "Fn("),
        ("FnMut (", "FnMut("),
        ("FnOnce (", "FnOnce("),
        (" (", "("),
        ("( ", "("),
        (" )", ")"),
        (" [", "["),
        ("[ ", "["),
        (" ]", "]"),
        (" ;", ";"),
        (" . ", "."),
    ] {
        s = s.replace(from, to);
    }
    s.replace("' a", "'a")
        .replace("& 'a", "&'a")
        .replace("'a[", "'a [")
}

fn is_pub(v: &Visibility) -> bool {
    matches!(v, Visibility::Public(_))
}

fn last_ident(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(p) => p.path.segments.last().map(|s| s.ident.to_string()),
        Type::Reference(r) => last_ident(&r.elem),
        _ => None,
    }
}

fn returns_self(sig: &syn::Signature, type_name: &str) -> bool {
    match &sig.output {
        syn::ReturnType::Type(_, ty) => {
            matches!(last_ident(ty).as_deref(), Some("Self"))
                || last_ident(ty).as_deref() == Some(type_name)
        }
        _ => false,
    }
}

fn func(f: &syn::ImplItemFn) -> Func {
    let params: Vec<String> = f
        .sig
        .inputs
        .iter()
        .filter_map(|a| match a {
            FnArg::Typed(pt) => Some(format!("{}: {}", tidy(&pt.pat), tidy(&pt.ty))),
            FnArg::Receiver(_) => None,
        })
        .collect();
    Func {
        name: f.sig.ident.to_string(),
        signature: format!("{}({})", f.sig.ident, params.join(", ")),
        doc: doc_of(&f.attrs),
    }
}

fn free_func(f: &syn::ItemFn) -> (Func, Option<String>) {
    let params: Vec<String> = f
        .sig
        .inputs
        .iter()
        .filter_map(|a| match a {
            FnArg::Typed(pt) => Some(format!("{}: {}", tidy(&pt.pat), tidy(&pt.ty))),
            FnArg::Receiver(_) => None,
        })
        .collect();
    let ret = match &f.sig.output {
        syn::ReturnType::Type(_, ty) => last_ident(ty),
        _ => None,
    };
    (
        Func {
            name: f.sig.ident.to_string(),
            signature: format!("{}({})", f.sig.ident, params.join(", ")),
            doc: doc_of(&f.attrs),
        },
        ret,
    )
}

/// Whether a parameter type takes a child: an `Element` or something `Into<Element>`.
fn takes_element(f: &syn::ImplItemFn) -> bool {
    f.sig.inputs.iter().any(|a| match a {
        FnArg::Typed(pt) => tidy(&pt.ty).contains("Element"),
        _ => false,
    })
}

fn parse_file(
    path: &Path,
    root: &Path,
    exports: &BTreeMap<String, String>,
    known: &[String],
) -> Vec<Component> {
    let src = fs::read_to_string(path).unwrap_or_default();
    let Ok(file) = syn::parse_file(&src) else {
        eprintln!("skipping {}: does not parse", path.display());
        return Vec::new();
    };
    let rel = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let module_doc = doc_of(&file.attrs);

    let mut structs: BTreeMap<String, Component> = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut is_component: Vec<String> = Vec::new();
    let mut free: Vec<(Func, Option<String>)> = Vec::new();

    for item in &file.items {
        match item {
            Item::Struct(s) if is_pub(&s.vis) && exports.contains_key(&s.ident.to_string()) => {
                let name = s.ident.to_string();
                order.push(name.clone());
                structs.insert(
                    name.clone(),
                    Component {
                        alias: snake(&exports[&name]),
                        name,
                        file: rel.clone(),
                        doc: doc_of(&s.attrs),
                        ..Default::default()
                    },
                );
            }
            Item::Fn(f) if is_pub(&f.vis) && exports.contains_key(&f.sig.ident.to_string()) => {
                free.push(free_func(f))
            }
            _ => {}
        }
    }
    for item in &file.items {
        let Item::Impl(imp) = item else { continue };
        let Some(self_name) = last_ident(&imp.self_ty) else {
            continue;
        };
        if let Some((_, path, _)) = &imp.trait_ {
            let tr = path.segments.last();
            // `impl From<X> for Element` and `impl Widget for X` make X a component.
            if let Some(seg) = tr {
                if seg.ident == "From"
                    && self_name == "Element"
                    && let syn::PathArguments::AngleBracketed(a) = &seg.arguments
                    && let Some(syn::GenericArgument::Type(t)) = a.args.first()
                    && let Some(n) = last_ident(t)
                {
                    is_component.push(n);
                }
                if seg.ident == "Widget" {
                    is_component.push(self_name.clone());
                }
            }
            continue;
        }
        let Some(c) = structs.get_mut(&self_name) else {
            continue;
        };
        for it in &imp.items {
            let ImplItem::Fn(f) = it else { continue };
            if !is_pub(&f.vis) {
                continue;
            }
            let entry = func(f);
            match f.sig.receiver() {
                None => c.constructors.push(entry),
                Some(r) => {
                    let by_value = r.reference.is_none();
                    if by_value && returns_self(&f.sig, &self_name) {
                        if entry.name.starts_with("on_") {
                            c.events.push(entry);
                        } else if takes_element(f) {
                            c.slots.push(entry);
                        } else {
                            c.properties.push(entry);
                        }
                    } else {
                        c.methods.push(entry);
                    }
                }
            }
        }
    }
    let mut out = Vec::new();
    // Free functions that return an element and are not the constructor of a struct component
    // are components of their own.
    let attached: Vec<String> = known.to_vec();
    for (f, ret) in &free {
        if f.doc.trim().is_empty() || ret.is_none() {
            continue;
        }
        let r = ret.clone().unwrap();
        // A function is a component of its own unless it is the struct's own namesake, or it
        // returns something that is neither an element nor a component.
        if !(is_element_like(&r) || attached.contains(&r))
            || (attached.contains(&r) && snake(&r) == snake(&f.name))
        {
            continue;
        }
        let alias = snake(&exports[&f.name]);
        let shown = Func {
            signature: f.signature.replacen(&f.name, &alias, 1),
            name: alias.clone(),
            doc: String::new(),
        };
        out.push(Component {
            alias,
            name: f.name.clone(),
            file: rel.clone(),
            doc: f.doc.clone(),
            constructors: vec![shown],
            returns: Some(r),
            ..Default::default()
        });
    }
    for name in order {
        if !is_component.contains(&name) {
            continue;
        }
        let mut c = structs.remove(&name).unwrap();
        c.functions = free
            .iter()
            .filter(|(_, ret)| ret.as_deref() == Some(&name))
            .map(|(f, _)| Func {
                signature: f.signature.replacen(&f.name, &snake(&exports[&f.name]), 1),
                ..f.clone()
            })
            .collect();
        out.push(c);
    }
    // A file with one component describes it in its module docs.
    if out.len() == 1 {
        out[0].module_doc = module_doc;
    }
    out
}

/// Return types that make a free function a component: an element or an iced widget builder.
fn is_element_like(ret: &str) -> bool {
    matches!(
        ret,
        "Element"
            | "Button"
            | "Checkbox"
            | "Radio"
            | "Slider"
            | "ProgressBar"
            | "Scrollable"
            | "Container"
            | "Row"
            | "Column"
            | "Text"
            | "Svg"
            | "Rule"
            | "TextInput"
            | "Space"
    )
}

/// The names `src/widget/mod.rs` re-exports (`pub use`): original name to exported name
/// (`Align as PopoverAlign` exports `Align` as `PopoverAlign`). Only these are public API.
fn exported(mod_rs: &Path) -> BTreeMap<String, String> {
    fn walk(t: &syn::UseTree, out: &mut BTreeMap<String, String>) {
        match t {
            syn::UseTree::Path(p) => walk(&p.tree, out),
            syn::UseTree::Name(n) => {
                out.insert(n.ident.to_string(), n.ident.to_string());
            }
            syn::UseTree::Rename(r) => {
                out.insert(r.ident.to_string(), r.rename.to_string());
            }
            syn::UseTree::Group(g) => g.items.iter().for_each(|i| walk(i, out)),
            syn::UseTree::Glob(_) => {}
        }
    }
    let mut out = BTreeMap::new();
    let src = fs::read_to_string(mod_rs).unwrap_or_default();
    if let Ok(file) = syn::parse_file(&src) {
        for item in &file.items {
            if let Item::Use(u) = item
                && is_pub(&u.vis)
            {
                walk(&u.tree, &mut out);
            }
        }
    }
    out
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|e| e == "rs") {
            out.push(p);
        }
    }
}

fn snake(name: &str) -> String {
    let mut s = String::new();
    for (i, ch) in name.chars().enumerate() {
        if ch.is_uppercase() {
            if i > 0 {
                s.push('_');
            }
            s.extend(ch.to_lowercase());
        } else {
            s.push(ch);
        }
    }
    s
}

// ---------------------------------------------------------------------------------------------
// roff
// ---------------------------------------------------------------------------------------------

fn esc(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\e"),
            '-' => out.push_str("\\-"),
            '\u{2014}' => out.push_str("\\(em"),
            '\u{2013}' => out.push_str("\\(en"),
            '\u{2018}' | '\u{2019}' => out.push('\''),
            '\u{201c}' => out.push_str("\\(lq"),
            '\u{201d}' => out.push_str("\\(rq"),
            '\u{2026}' => out.push_str("..."),
            '\u{00d7}' => out.push_str("\\(mu"),
            '\u{2192}' => out.push_str("\\(->"),
            '\u{2022}' => out.push_str("\\(bu"),
            c if !c.is_ascii() => {
                let _ = write!(out, "\\[u{:04X}]", c as u32);
            }
            c => out.push(c),
        }
    }
    out
}

/// A line that may not start with a control character.
fn guard(line: &str) -> String {
    if line.starts_with('.') || line.starts_with('\'') {
        format!("\\&{line}")
    } else {
        line.to_string()
    }
}

/// Inline Markdown to roff: links keep their text, `code` and **bold** become bold.
fn inline(text: &str) -> String {
    // Links: [text](url) and [`Type`] keep the text.
    let mut s = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '['
            && let Some(close) = chars[i..].iter().position(|&c| c == ']')
        {
            let inner: String = chars[i + 1..i + close].iter().collect();
            let after = i + close + 1;
            if after < chars.len() && chars[after] == '(' {
                if let Some(end) = chars[after..].iter().position(|&c| c == ')') {
                    s.push_str(&inner);
                    i = after + end + 1;
                    continue;
                }
            } else if inner.starts_with('`') {
                s.push_str(&inner);
                i = after;
                continue;
            }
        }
        s.push(chars[i]);
        i += 1;
    }
    let s = esc(&s);
    let mut out = String::new();
    let mut code = false;
    let mut bold = false;
    let b: Vec<char> = s.chars().collect();
    let mut k = 0;
    while k < b.len() {
        if b[k] == '`' {
            out.push_str(if code { "\\fR" } else { "\\fB" });
            code = !code;
        } else if !code && b[k] == '*' && k + 1 < b.len() && b[k + 1] == '*' {
            out.push_str(if bold { "\\fR" } else { "\\fB" });
            bold = !bold;
            k += 1;
        } else {
            out.push(b[k]);
        }
        k += 1;
    }
    if code || bold {
        out.push_str("\\fR");
    }
    out
}

/// A doc comment as roff paragraphs, lists and code blocks.
fn doc_roff(doc: &str) -> String {
    let mut out = Vec::new();
    let mut para: Vec<String> = Vec::new();
    let mut in_code = false;
    let flush = |para: &mut Vec<String>, out: &mut Vec<String>| {
        if !para.is_empty() {
            out.push(".PP".to_string());
            out.push(guard(&para.join(" ")));
            para.clear();
        }
    };
    for raw in doc.lines() {
        let line = raw.trim_end();
        if line.trim_start().starts_with("```") {
            flush(&mut para, &mut out);
            if in_code {
                out.push(".RE\n.fi".to_string());
            } else {
                out.push(".PP\n.nf\n.RS".to_string());
            }
            in_code = !in_code;
            continue;
        }
        if in_code {
            out.push(guard(&esc(line)));
            continue;
        }
        let t = line.trim_start();
        if t.is_empty() {
            flush(&mut para, &mut out);
        } else if let Some(item) = t.strip_prefix("* ").or_else(|| t.strip_prefix("- ")) {
            flush(&mut para, &mut out);
            out.push(format!(".IP \\(bu 2\n{}", guard(&inline(item))));
        } else if raw.starts_with("  ") && out.last().is_some_and(|l| l.starts_with(".IP")) {
            // A wrapped list item.
            let last = out.pop().unwrap();
            out.push(format!("{last} {}", inline(t)));
        } else if t.starts_with('#') {
            flush(&mut para, &mut out);
            out.push(format!(
                ".PP\n\\fB{}\\fR",
                inline(t.trim_start_matches('#').trim())
            ));
        } else {
            para.push(inline(t));
        }
    }
    flush(&mut para, &mut out);
    if in_code {
        out.push(".RE\n.fi".to_string());
    }
    out.join("\n")
}

/// The first sentence of a doc comment, flattened: the one line description.
fn summary(doc: &str) -> String {
    // "`Dock`: a surface..." and "Dock: a surface..." are described by what follows the colon.
    let doc = match doc.split_once(": ") {
        Some((head, rest))
            if head
                .trim_matches('`')
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_') =>
        {
            rest
        }
        _ => doc,
    };
    let first: Vec<&str> = doc.lines().take_while(|l| !l.trim().is_empty()).collect();
    let text = first.join(" ");
    let text = match text.find(". ") {
        Some(i) => text[..=i].to_string(),
        None => text,
    };
    inline(text.trim().trim_end_matches('.'))
        .replace("\\fB", "")
        .replace("\\fR", "")
}

fn funcs_roff(out: &mut String, title: &str, intro: &str, items: &[Func], prefix: &str) {
    if items.is_empty() {
        return;
    }
    let _ = writeln!(out, ".SH {title}");
    if !intro.is_empty() {
        let _ = writeln!(out, "{intro}");
    }
    for f in items {
        let _ = writeln!(
            out,
            ".TP\n.B {}",
            guard(&esc(&format!("{prefix}{}", f.signature)))
        );
        if f.doc.trim().is_empty() {
            continue;
        }
        let body = doc_roff(&f.doc);
        // A tagged paragraph takes plain text; its further paragraphs keep their indent.
        let _ = writeln!(out, "{}", body.replacen(".PP\n", "", 1));
    }
}

fn page(c: &Component, version: &str) -> String {
    let alias = c.alias.clone();
    let title = format!(
        "ICED\\-LIBADWAITA\\-{}",
        esc(&alias).to_uppercase().replace('_', "\\-")
    );
    let mut o = String::new();
    let _ = writeln!(
        o,
        ".\\\" Generated by `cargo xtask man` from {}; do not edit.",
        c.file
    );
    let _ = writeln!(
        o,
        ".TH {title} 3 \"\" \"iced-libadwaita {version}\" \"iced-libadwaita Manual\""
    );
    let one = if summary(&c.doc).is_empty() {
        summary(&c.module_doc)
    } else {
        summary(&c.doc)
    };
    let _ = writeln!(
        o,
        ".SH NAME\n{} \\- {}",
        esc(&alias),
        if one.is_empty() {
            format!("the {} component", c.name)
        } else {
            one
        }
    );
    let _ = writeln!(o, ".SH SYNOPSIS\n.nf");
    let first_ctor = c
        .constructors
        .iter()
        .find(|f| f.name == "new")
        .or(c.constructors.first());
    if let Some(f) = c.functions.first() {
        let _ = writeln!(o, "{}", guard(&esc(&f.signature)));
    }
    if let Some(f) = first_ctor {
        let text = if c.returns.is_some() {
            f.signature.clone()
        } else {
            format!("{}::{}", c.name, f.signature)
        };
        let _ = writeln!(o, "{}", guard(&esc(&text)));
    }
    let _ = writeln!(o, ".fi");
    let docs = [c.doc.as_str(), c.module_doc.as_str()]
        .into_iter()
        .filter(|d| !d.trim().is_empty())
        .collect::<Vec<_>>();
    if !docs.is_empty() {
        let _ = writeln!(o, ".SH DESCRIPTION");
        for d in docs {
            let _ = writeln!(o, "{}", doc_roff(d));
        }
    }
    funcs_roff(&mut o, "FUNCTIONS", "", &c.functions, "");
    if c.returns.is_none() {
        funcs_roff(
            &mut o,
            "CONSTRUCTORS",
            "",
            &c.constructors,
            &format!("{}::", c.name),
        );
    }
    funcs_roff(
        &mut o,
        "PROPERTIES",
        "Builder methods, chained after the constructor.",
        &c.properties,
        ".",
    );
    funcs_roff(
        &mut o,
        "SLOTS",
        "Builder methods that take children.",
        &c.slots,
        ".",
    );
    funcs_roff(
        &mut o,
        "EVENTS",
        "Callbacks that produce the application's message.",
        &c.events,
        ".",
    );
    funcs_roff(&mut o, "METHODS", "", &c.methods, ".");
    let _ = writeln!(o, ".SH SOURCE\n{}", esc(&c.file));
    let _ = writeln!(
        o,
        ".SH SEE ALSO\n\\fBiced\\-libadwaita\\fR(7), \\fBiced\\-libadwaita\\-components\\fR(3)"
    );
    o
}

fn index(all: &[Component], version: &str) -> String {
    let mut o = String::new();
    let _ = writeln!(o, ".\\\" Generated by `cargo xtask man`; do not edit.");
    let _ = writeln!(
        o,
        ".TH ICED\\-LIBADWAITA\\-COMPONENTS 3 \"\" \"iced-libadwaita {version}\" \"iced-libadwaita Manual\""
    );
    let _ = writeln!(
        o,
        ".SH NAME\niced\\-libadwaita\\-components \\- the iced\\-libadwaita components"
    );
    let _ = writeln!(
        o,
        ".SH DESCRIPTION\nEach component has a page of its own, \\fBiced\\-libadwaita\\-\\fIalias\\fR(3), with its constructors, properties, slots and events. The pages are generated from the source's doc comments by \\fBcargo xtask man\\fR. See \\fBiced\\-libadwaita\\fR(7) for the conventions every component follows."
    );
    let _ = writeln!(o, ".SH COMPONENTS");
    let mut sorted: Vec<&Component> = all.iter().collect();
    sorted.sort_by_key(|c| c.alias.clone());
    for c in sorted {
        let alias = c.alias.clone();
        let s = if summary(&c.doc).is_empty() {
            summary(&c.module_doc)
        } else {
            summary(&c.doc)
        };
        let _ = writeln!(o, ".TP\n.B {}", esc(&alias));
        let _ = writeln!(o, "{}", guard(&if s.is_empty() { esc(&c.name) } else { s }));
    }
    let _ = writeln!(
        o,
        ".SH SEE ALSO\n\\fBiced\\-libadwaita\\fR(7), \\fBiced\\-libadwaita\\-desktop.conf\\fR(5)"
    );
    o
}

// ---------------------------------------------------------------------------------------------

fn man(check: bool) -> bool {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .canonicalize()
        .unwrap();
    let version = fs::read_to_string(root.join("Cargo.toml"))
        .ok()
        .and_then(|t| {
            t.lines().find_map(|l| {
                l.strip_prefix("version = ")
                    .map(|v| v.trim_matches('"').to_string())
            })
        })
        .unwrap_or_default();
    let mut files = Vec::new();
    rust_files(&root.join("src/widget"), &mut files);
    files.push(root.join("src/window/dialog.rs"));
    let exports = exported(&root.join("src/widget/mod.rs"));
    // Every public struct of the widget files: a function returning one is a component too.
    let known: Vec<String> = files
        .iter()
        .filter_map(|f| fs::read_to_string(f).ok())
        .filter_map(|t| syn::parse_file(&t).ok())
        .flat_map(|f| f.items)
        .filter_map(|i| match i {
            Item::Struct(s) if is_pub(&s.vis) => Some(s.ident.to_string()),
            _ => None,
        })
        .collect();
    let mut all: Vec<Component> = Vec::new();
    for f in &files {
        all.extend(parse_file(f, &root, &exports, &known));
    }
    all.sort_by_key(|c| c.alias.clone());
    all.dedup_by_key(|c| c.alias.clone());

    let dir = root.join("man");
    let mut pages: Vec<(PathBuf, String)> = all
        .iter()
        .map(|c| {
            (
                dir.join(format!("iced-libadwaita-{}.3", c.alias.replace('_', "-"))),
                page(c, &version),
            )
        })
        .collect();
    pages.push((
        dir.join("iced-libadwaita-components.3"),
        index(&all, &version),
    ));

    if check {
        let mut stale = Vec::new();
        for (p, text) in &pages {
            if fs::read_to_string(p).ok().as_deref() != Some(text) {
                stale.push(p.clone());
            }
        }
        if stale.is_empty() {
            println!("man pages are up to date ({} pages)", pages.len());
            return true;
        }
        for p in &stale {
            eprintln!("out of date: {}", p.display());
        }
        eprintln!("run `cargo xtask man`");
        return false;
    }
    fs::create_dir_all(&dir).unwrap();
    // Remove pages of components that no longer exist.
    let keep: Vec<&PathBuf> = pages.iter().map(|(p, _)| p).collect();
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.flatten() {
            let p = e.path();
            let n = p.file_name().unwrap().to_string_lossy().to_string();
            if n.starts_with("iced-libadwaita-") && n.ends_with(".3") && !keep.contains(&&p) {
                let _ = fs::remove_file(&p);
            }
        }
    }
    for (p, text) in &pages {
        fs::write(p, text).unwrap();
    }
    println!("wrote {} man pages to {}", pages.len(), dir.display());
    true
}
