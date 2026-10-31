//! Read-only property history for shared desktop configuration.
//!
//! Each statically linked toolkit release reads only the keys it knows.
//! Deprecation forbids *new build artifacts* from referencing a key, but does
//! not delete the key from older binaries or rewrite user configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Property {
    pub section: &'static str,
    pub key: &'static str,
    pub since: &'static str,
    /// First toolkit version in which new builds must not reference this key.
    pub deprecated: Option<&'static str>,
    /// Informational only: never automatically maps old settings to new ones.
    pub replacement: Option<&'static str>,
}

macro_rules! property {
    ($section:literal, $key:literal) => {
        Property { section: $section, key: $key, since: "0.1.0", deprecated: None, replacement: None }
    };
}

/// The immutable historical registry shipped with this toolkit version.
/// Future releases append entries and mark deprecated keys; never erase history.
pub const PROPERTIES: &[Property] = &[
    property!("appearance", "color-scheme"),
    property!("appearance", "accent"),
    property!("appearance", "flat-headerbars"),
    property!("appearance", "density"),
    property!("appearance", "text-scale"),
    property!("appearance", "shadows"),
    property!("appearance", "shadow-scale"),
    property!("appearance", "shadow-intensity"),
    property!("motion", "animations"),
    property!("motion", "reduced-motion"),
    property!("motion", "page-change"),
    property!("motion", "page-push"),
    property!("motion", "popover"),
    property!("motion", "selection"),
    property!("motion", "drawer"),
    property!("motion", "dialog"),
    property!("motion", "sheet"),
    property!("motion", "toast"),
    property!("motion", "reveal"),
    property!("motion", "switch"),
    property!("motion", "carousel"),
    property!("windows", "decorations"),
    property!("windows", "button-layout"),
    property!("windows", "mobile-max-width"),
    property!("navigation", "collapse-width"),
    property!("navigation", "mobile-sidebar"),
    property!("menus", "presentation"),
    property!("rows", "collapse"),
    property!("rows", "wrap-basis"),
    property!("icons", "theme"),
    property!("policy", "allow-app-overrides"),
];

pub fn find(section: &str, key: &str) -> Option<&'static Property> {
    PROPERTIES.iter().find(|p| p.section == section && p.key == key)
}

/// Validates properties referenced by new application build artifacts.
/// Unknown and deprecated properties are errors, not runtime migrations.
pub fn validate_build_properties<'a>(
    properties: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for (section, key) in properties {
        match find(section, key) {
            None => errors.push(format!("[{section}] {key}: unknown property")),
            Some(p) if p.deprecated.is_some() => errors.push(format!(
                "[{section}] {key}: deprecated since {}; replacement: {}",
                p.deprecated.unwrap_or_default(),
                p.replacement.unwrap_or("none")
            )),
            _ => {}
        }
    }
    if errors.is_empty() { Ok(()) } else { Err(errors) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unique_registry() {
        let mut keys = std::collections::HashSet::new();
        for p in PROPERTIES { assert!(keys.insert((p.section, p.key))); }
    }
    #[test]
    fn known_properties_validate() {
        assert!(validate_build_properties([("appearance", "density")]).is_ok());
        assert!(validate_build_properties([("appearance", "not-a-key")]).is_err());
    }
}
