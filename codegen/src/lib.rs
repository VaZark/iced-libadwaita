//! Build-time compiler for the component-catalog XML vocabulary.
use std::{fs, path::Path};

#[path = "../../src/catalog.rs"]
mod catalog;

#[derive(Debug)]
pub enum Error {
    Xml(roxmltree::Error),
    Io(std::io::Error),
    Invalid(String),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Xml(e) => write!(f, "{e}"),
            Self::Io(e) => write!(f, "{e}"),
            Self::Invalid(e) => write!(f, "{e}"),
        }
    }
}
impl std::error::Error for Error {}
impl From<roxmltree::Error> for Error {
    fn from(e: roxmltree::Error) -> Self {
        Self::Xml(e)
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

pub fn compile_file(input: impl AsRef<Path>, output: impl AsRef<Path>) -> Result<(), Error> {
    let input = input.as_ref();
    let generated = compile(&fs::read_to_string(input)?)?;
    if let Some(parent) = output.as_ref().parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(output, generated)?;
    println!("cargo:rerun-if-changed={}", input.display());
    println!(
        "cargo:rerun-if-changed={}",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../src/catalog.rs")
    );
    Ok(())
}

pub fn compile(xml: &str) -> Result<String, Error> {
    let doc = roxmltree::Document::parse(xml)?;
    let root = doc.root_element();
    let node = if root.tag_name().name() == "interface" {
        let elements: Vec<_> = root.children().filter(|n| n.is_element()).collect();
        if elements.len() != 1 {
            return Err(invalid("interface must contain exactly one component"));
        }
        elements[0]
    } else {
        root
    };
    let body = element(node)?;
    Ok(format!("// Generated; do not edit.\npub fn view<'a>() -> iced_libadwaita::widget::Element<'a, crate::Message> {{\n    {body}\n}}\n"))
}

fn invalid(message: impl Into<String>) -> Error {
    Error::Invalid(message.into())
}
fn quote(s: &str) -> String {
    format!("{s:?}")
}

fn element(node: roxmltree::Node<'_, '_>) -> Result<String, Error> {
    let xml_tag = node.tag_name().name();
    let tag = xml_name(xml_tag)?;
    let schema =
        catalog::find(&tag).ok_or_else(|| invalid(format!("unknown component <{tag}>")))?;
    // The catalog is authoritative for names. The backend separately lists supported
    // constructor signatures until all catalog entries have typed generation adapters.
    if ![
        "preferences_page",
        "preferences_group",
        "action_row",
        "header_bar",
        "toolbar_view",
        "page",
    ]
    .contains(&tag)
    {
        return Err(invalid(format!(
            "<{tag}> exists in the catalog but has no XML codegen adapter yet"
        )));
    }
    let mut attrs = std::collections::BTreeMap::new();

    for attr in node.attributes() {
        if attr.name() == "slot" {
            continue;
        }
        let name = xml_name(attr.name())?;
        let allowed = schema.props.contains(&name.as_str())
            || schema.events.contains(&name.as_str())
            || constructor_arguments(&tag).contains(&name.as_str());
        if !allowed {
            return Err(invalid(format!(
                "<{tag}>: unknown property '{}'",
                attr.name()
            )));
        }
        attrs.insert(name, attr.value().to_owned());
    }
    let mut children = Vec::new();
    for child in node.children().filter(|n| n.is_element()) {
        let child_slot = xml_name(
            child
                .attribute("slot")
                .unwrap_or(default_slot(&tag).replace("_", "-").as_str()),
        )?;
        if !schema.slots.contains(&child_slot.as_str()) {
            return Err(invalid(format!(
                "<{tag}>: invalid child slot '{child_slot}'"
            )));
        }
        children.push((child_slot, element(child)?));
    }
    if node
        .children()
        .any(|n| n.is_text() && !n.text().unwrap_or("").trim().is_empty())
    {
        return Err(invalid(format!("<{tag}>: text children are not supported")));
    }
    let required = |key: &str| {
        attrs
            .get(key)
            .map(String::as_str)
            .ok_or_else(|| invalid(format!("<{tag}>: missing required attribute '{key}'")))
    };
    let mut expression = match tag {
        "preferences_page" => "iced_libadwaita::widget::preferences_page()".to_owned(),
        "preferences_group" => "iced_libadwaita::widget::preferences_group()".to_owned(),
        "action_row" => format!(
            "iced_libadwaita::widget::ActionRow::new({})",
            quote(required("title")?)
        ),
        "header_bar" => "iced_libadwaita::widget::header_bar()".to_owned(),
        "toolbar_view" => {
            let (i, _) = children
                .iter()
                .enumerate()
                .find(|(_, (slot, _))| slot == "content")
                .ok_or_else(|| invalid("<toolbar_view>: missing content child"))?;
            let content = &children[i].1;
            format!("iced_libadwaita::widget::toolbar_view({content})")
        }
        "page" => {
            let (i, _) = children
                .iter()
                .enumerate()
                .find(|(_, (slot, _))| slot == "content")
                .ok_or_else(|| invalid("<page>: missing content child"))?;
            format!("iced_libadwaita::widget::page({})", children[i].1)
        }
        _ => unreachable!(),
    };
    for (key, value) in &attrs {
        if constructor_arguments(&tag).contains(&key.as_str()) {
            continue;
        }
        // Only methods with known, compatible string/bool signatures are emitted.
        let arg = match (tag, key.as_str()) {
            ("preferences_group", "title" | "description")
            | ("action_row", "subtitle")
            | ("header_bar", "title" | "subtitle") => quote(value),
            ("header_bar", "flat") | ("action_row", "chevron") => {
                if value != "true" && value != "false" {
                    return Err(invalid(format!("<{tag}>: {key} must be true or false")));
                }
                value.to_owned()
            }
            _ => return Err(invalid(format!(
                "<{tag}>: '{key}' is catalogued but its typed codegen mapping is not implemented"
            ))),
        };
        expression.push_str(&format!(".{key}({arg})"));
    }
    for (child_slot, child) in &children {
        if (tag == "toolbar_view" || tag == "page") && child_slot == "content" {
            continue;
        }
        let method = match (tag, child_slot.as_str()) {
            ("preferences_page", "push") | ("preferences_group", "push") => "push",
            ("header_bar", "pack_start" | "pack_end" | "title_widget") => child_slot,
            ("toolbar_view", "top_bar" | "bottom_bar") => child_slot,
            _ => {
                return Err(invalid(format!(
                    "<{tag}>: slot '{child_slot}' has no codegen mapping"
                )))
            }
        };
        expression.push_str(&format!(".{method}({child})"));
    }
    if tag == "toolbar_view" && children.iter().filter(|(s, _)| s == "content").count() != 1 {
        return Err(invalid("<toolbar_view> requires exactly one content child"));
    }
    if tag == "page" && children.iter().filter(|(s, _)| s == "content").count() != 1 {
        return Err(invalid("<page> requires exactly one content child"));
    }
    Ok(format!("({expression}).into()"))
}

fn xml_name(name: &str) -> Result<String, Error> {
    if name.is_empty()
        || name.starts_with('-')
        || name.ends_with('-')
        || !name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        || name.contains("--")
    {
        return Err(invalid(format!(
            "invalid XML identifier '{name}'; use kebab-case"
        )));
    }
    Ok(name.replace('-', "_"))
}

fn constructor_arguments(tag: &str) -> &'static [&'static str] {
    match tag {
        "action_row" => &["title"],
        _ => &[],
    }
}
fn default_slot(tag: &str) -> &'static str {
    match tag {
        "preferences_page" | "preferences_group" => "push",
        _ => "content",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generates_catalog_tags() {
        let xml = r#"<toolbar-view><header-bar slot="top-bar" title="Settings"/><page><preferences-group title="Appearance"><action-row title="Theme" subtitle="System"/></preferences-group></page></toolbar-view>"#;
        let rust = compile(xml).unwrap();
        assert!(rust.contains("toolbar_view("));
        assert!(rust.contains(".top_bar("));
        assert!(rust.contains("ActionRow::new"));
    }
    #[test]
    fn rejects_unknown_attribute() {
        assert!(compile(r#"<action-row title="Hi" typo="yes"/>"#)
            .unwrap_err()
            .to_string()
            .contains("unknown property"));
    }
    #[test]
    fn rejects_invalid_slot() {
        assert!(compile(
            r#"<preferences-page><action-row slot="top-bar" title="Hi"/></preferences-page>"#
        )
        .is_err());
    }
    #[test]
    fn rejects_underscored_xml_names() {
        assert!(compile(r#"<action_row title="Hi"/>"#).is_err());
        assert!(compile(r#"<action-row title="Hi" on_press="Message::Click"/>"#).is_err());
    }
    #[test]
    fn rejects_unmapped_catalog_component() {
        assert!(compile("<switch-row/>")
            .unwrap_err()
            .to_string()
            .contains("no XML codegen adapter"));
    }
}
