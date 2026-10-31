//! The versioned configuration registry: a read-only history of every shared-desktop-config
//! property, bundled with the toolkit.
//!
//! Applications link their toolkit statically, so applications built with different toolkit
//! versions read the same `desktop.conf` side by side. The registry is what keeps that safe:
//!
//! * each application reads **only** the properties its bundled toolkit version knows;
//! * unknown properties are **ignored, never deleted or rewritten** (a newer toolkit may have
//!   written them, and a settings tool may show them);
//! * a missing property uses its [`default`](Property::default);
//! * **deprecation never removes historical support**: an application built with an older
//!   toolkit keeps using a deprecated property. Only *new builds* are forbidden from
//!   referencing it: [`validate_build_properties`] fails for them;
//! * there is no automatic translation between a deprecated property and its
//!   [`replacement`](Property::replacement), which is informational.
//!
//! Releases append to [`PROPERTIES`] and set `deprecated`; they never erase an entry.

/// The type of a property's value, for validation and for a settings UI.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ValueType {
    /// `true`/`false` (also `yes`/`no`, `on`/`off`, `1`/`0`).
    Bool,
    /// A number in `min..=max`.
    Number { min: f64, max: f64 },
    /// One of a fixed set of words.
    Choice(&'static [&'static str]),
    /// Free text.
    Text,
}

impl ValueType {
    /// Whether `value` is valid for this type.
    pub fn accepts(&self, value: &str) -> bool {
        match self {
            Self::Bool => matches!(
                value.to_ascii_lowercase().as_str(),
                "true" | "yes" | "on" | "1" | "false" | "no" | "off" | "0"
            ),
            Self::Number { min, max } => value
                .parse::<f64>()
                .is_ok_and(|n| (*min..=*max).contains(&n)),
            Self::Choice(words) => words.contains(&value),
            Self::Text => true,
        }
    }
}

/// One property of the shared desktop configuration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Property {
    /// The `[section]` of the key file.
    pub section: &'static str,
    /// The property identifier.
    pub key: &'static str,
    pub ty: ValueType,
    /// The value used when the property is missing, as written in the file.
    pub default: &'static str,
    /// The first toolkit version that supports the property.
    pub since: &'static str,
    /// The first toolkit version in which **new builds** must not reference it. Older builds
    /// keep reading it.
    pub deprecated: Option<&'static str>,
    /// A newer property to use instead. Informational: never an automatic alias or migration.
    pub replacement: Option<&'static str>,
}

/// The toolkit version this registry is bundled with.
pub const TOOLKIT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// `major.minor.patch` as numbers (missing parts are 0); `None` if not numeric.
fn version(v: &str) -> Option<[u64; 3]> {
    let mut parts = [0u64; 3];
    for (i, p) in v.split('.').take(3).enumerate() {
        parts[i] = p.split(['-', '+']).next()?.parse().ok()?;
    }
    Some(parts)
}

/// Whether version `a` is at least version `b`.
fn at_least(a: &str, b: &str) -> bool {
    matches!((version(a), version(b)), (Some(a), Some(b)) if a >= b)
}

impl Property {
    /// Whether a toolkit of version `toolkit` reads this property.
    pub fn is_supported_by(&self, toolkit: &str) -> bool {
        at_least(toolkit, self.since)
    }

    /// Whether a **new build** against toolkit `toolkit` must not reference this property.
    pub fn is_deprecated_in(&self, toolkit: &str) -> bool {
        self.deprecated.is_some_and(|d| at_least(toolkit, d))
    }
}

const fn p(
    section: &'static str,
    key: &'static str,
    ty: ValueType,
    default: &'static str,
) -> Property {
    Property {
        section,
        key,
        ty,
        default,
        since: "0.1.0",
        deprecated: None,
        replacement: None,
    }
}

use ValueType::{Bool, Choice, Number, Text};

const ACCENTS: &[&str] = &[
    "system", "blue", "teal", "green", "yellow", "orange", "red", "pink", "purple", "slate",
];
const MOTION: &[&str] = &["default", "none"];
const POSITIONS: &[&str] = &[
    "top-left",
    "top-center",
    "top-right",
    "bottom-left",
    "bottom-center",
    "bottom-right",
];

/// The immutable historical registry shipped with this toolkit version.
pub const PROPERTIES: &[Property] = &[
    p(
        "appearance",
        "color-scheme",
        Choice(&["system", "light", "dark"]),
        "system",
    ),
    p("appearance", "accent", Choice(ACCENTS), "system"),
    p("appearance", "flat-headerbars", Bool, "true"),
    p("appearance", "auto-contrast", Bool, "true"),
    p(
        "appearance",
        "density",
        Choice(&["default", "compact"]),
        "default",
    ),
    p(
        "appearance",
        "text-scale",
        Number { min: 0.5, max: 3.0 },
        "1",
    ),
    p("appearance", "shadows", Bool, "true"),
    p(
        "appearance",
        "shadow-scale",
        Number {
            min: 0.0,
            max: 10.0,
        },
        "1",
    ),
    p(
        "appearance",
        "shadow-intensity",
        Number {
            min: 0.0,
            max: 10.0,
        },
        "1",
    ),
    p(
        "motion",
        "animations",
        Choice(&["system", "on", "off", "true", "false"]),
        "system",
    ),
    p("motion", "reduced-motion", Bool, "false"),
    p("motion", "page-change", Choice(MOTION), "default"),
    p("motion", "page-push", Choice(MOTION), "default"),
    p("motion", "popover", Choice(MOTION), "default"),
    p("motion", "selection", Choice(MOTION), "default"),
    p("motion", "drawer", Choice(MOTION), "default"),
    p("motion", "dialog", Choice(MOTION), "default"),
    p("motion", "sheet", Choice(MOTION), "default"),
    p("motion", "toast", Choice(MOTION), "default"),
    p("motion", "reveal", Choice(MOTION), "default"),
    p("motion", "switch", Choice(MOTION), "default"),
    p("motion", "carousel", Choice(MOTION), "default"),
    p("motion", "layout", Choice(MOTION), "default"),
    p(
        "breakpoints",
        "sm",
        Number {
            min: 200.0,
            max: 10000.0,
        },
        "576",
    ),
    p(
        "breakpoints",
        "md",
        Number {
            min: 200.0,
            max: 10000.0,
        },
        "768",
    ),
    p(
        "breakpoints",
        "lg",
        Number {
            min: 200.0,
            max: 10000.0,
        },
        "992",
    ),
    p(
        "breakpoints",
        "xl",
        Number {
            min: 200.0,
            max: 10000.0,
        },
        "1200",
    ),
    p(
        "breakpoints",
        "xxl",
        Number {
            min: 200.0,
            max: 10000.0,
        },
        "1400",
    ),
    p(
        "layout",
        "page-max-width",
        Number {
            min: 300.0,
            max: 4000.0,
        },
        "600",
    ),
    p("windows", "decorations", Choice(&["csd", "ssd"]), "csd"),
    p("windows", "button-layout", Text, "system"),
    p(
        "windows",
        "mobile-max-width",
        Number {
            min: 200.0,
            max: 10000.0,
        },
        "575",
    ),
    p(
        "windows",
        "min-width",
        Number {
            min: 200.0,
            max: 2000.0,
        },
        "360",
    ),
    p(
        "windows",
        "min-height",
        Number {
            min: 100.0,
            max: 2000.0,
        },
        "200",
    ),
    p(
        "navigation",
        "collapse-width",
        Number {
            min: 200.0,
            max: 10000.0,
        },
        "767",
    ),
    p(
        "navigation",
        "min-content-width",
        Number {
            min: 200.0,
            max: 10000.0,
        },
        "520",
    ),
    p(
        "navigation",
        "sidebar-fraction",
        Number { min: 0.1, max: 0.5 },
        "0.25",
    ),
    p(
        "navigation",
        "sidebar-min-width",
        Number {
            min: 100.0,
            max: 600.0,
        },
        "180",
    ),
    p(
        "navigation",
        "sidebar-max-width",
        Number {
            min: 100.0,
            max: 800.0,
        },
        "280",
    ),
    p(
        "navigation",
        "drawer-max-fraction",
        Number {
            min: 0.3,
            max: 0.95,
        },
        "0.85",
    ),
    p(
        "navigation",
        "mobile-sidebar",
        Choice(&["drawer", "first-page"]),
        "drawer",
    ),
    p(
        "menus",
        "presentation",
        Choice(&["auto", "dropdown", "full-page"]),
        "auto",
    ),
    p("shortcuts", "enabled", Bool, "true"),
    p(
        "shortcuts",
        "hints",
        Choice(&["auto", "always", "never"]),
        "auto",
    ),
    p("toasts", "position", Choice(POSITIONS), "bottom-center"),
    p(
        "toasts",
        "target",
        Choice(&["window", "content"]),
        "content",
    ),
    p("rows", "collapse", Choice(&["stacked", "flex"]), "stacked"),
    p(
        "rows",
        "wrap-basis",
        Number {
            min: 0.0,
            max: 2000.0,
        },
        "128",
    ),
    p("icons", "theme", Text, "system"),
    p("policy", "allow-app-overrides", Bool, "true"),
];

/// The registry entry for a property, whatever its lifecycle state.
pub fn find(section: &str, key: &str) -> Option<&'static Property> {
    PROPERTIES
        .iter()
        .find(|p| p.section == section && p.key == key)
}

/// The properties this toolkit reads: those introduced at or before [`TOOLKIT_VERSION`].
pub fn supported() -> impl Iterator<Item = &'static Property> {
    PROPERTIES
        .iter()
        .filter(|p| p.is_supported_by(TOOLKIT_VERSION))
}

/// Validates properties that a **new build** references (an app's bundled defaults, a build-time
/// UI definition…) against this toolkit version. Unknown and deprecated properties are errors;
/// nothing is migrated. A user's own `desktop.conf` is runtime input, not a build artifact, and
/// is never validated here.
pub fn validate_build_properties<'a>(
    properties: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<(), Vec<String>> {
    validate_in(PROPERTIES, TOOLKIT_VERSION, properties)
}

/// [`validate_build_properties`] against an explicit registry and toolkit version.
pub fn validate_in<'a>(
    registry: &[Property],
    toolkit: &str,
    properties: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for (section, key) in properties {
        match registry
            .iter()
            .find(|p| p.section == section && p.key == key)
        {
            Some(p) if p.is_supported_by(toolkit) => {
                if let Some(since) = p.deprecated.filter(|_| p.is_deprecated_in(toolkit)) {
                    errors.push(format!(
                        "[{section}] {key}: deprecated since {since}; replacement: {}",
                        p.replacement.unwrap_or("none")
                    ));
                }
            }
            _ => errors.push(format!("[{section}] {key}: unknown property")),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_registry_has_unique_entries_and_valid_defaults() {
        let mut seen = std::collections::HashSet::new();
        for p in PROPERTIES {
            assert!(
                seen.insert((p.section, p.key)),
                "duplicate [{}] {}",
                p.section,
                p.key
            );
            assert!(
                p.ty.accepts(p.default),
                "[{}] {}: default {:?} is not a valid value",
                p.section,
                p.key,
                p.default
            );
            assert!(
                version(p.since).is_some(),
                "[{}] {}: bad since {}",
                p.section,
                p.key,
                p.since
            );
            if let Some(d) = p.deprecated {
                assert!(
                    at_least(d, p.since),
                    "[{}] {}: deprecated before it was introduced",
                    p.section,
                    p.key
                );
            }
        }
    }

    #[test]
    fn every_trigger_is_a_motion_property() {
        for t in crate::animation::Trigger::ALL {
            assert!(
                find("motion", t.key()).is_some(),
                "motion trigger {} has no registry entry",
                t.key()
            );
        }
    }

    #[test]
    fn versions_compare_numerically() {
        assert!(at_least("0.10.0", "0.9.0"));
        assert!(at_least("1.0.0", "1.0.0"));
        assert!(!at_least("0.1.0", "0.2.0"));
        assert!(
            at_least("0.2.0-rc1", "0.2.0"),
            "a pre-release tag is ignored"
        );
    }

    /// Deprecation forbids *new builds*; it never removes support, and never rewrites anything.
    #[test]
    fn deprecation_blocks_new_builds_but_not_old_ones() {
        let registry = [
            Property {
                since: "0.1.0",
                deprecated: Some("0.3.0"),
                replacement: Some("[appearance] density"),
                ..p("appearance", "compact", Bool, "false")
            },
            Property {
                since: "0.2.0",
                ..p("appearance", "density", Bool, "false")
            },
        ];
        let props = [("appearance", "compact")];
        // A build against 0.2.0 may still use it (and so may anything built earlier).
        assert!(validate_in(&registry, "0.2.0", props).is_ok());
        assert!(validate_in(&registry, "0.1.0", props).is_ok());
        // From 0.3.0, a new build is rejected, with the (informational) replacement.
        let errors = validate_in(&registry, "0.3.0", props).unwrap_err();
        assert!(
            errors[0].contains("deprecated since 0.3.0")
                && errors[0].contains("[appearance] density"),
            "{errors:?}"
        );
        // A property newer than the toolkit does not exist for it.
        assert!(
            validate_in(&registry, "0.1.0", [("appearance", "density")]).unwrap_err()[0]
                .contains("unknown property")
        );
        // The entry stays in the history.
        assert!(registry.iter().any(|p| p.key == "compact"));
    }

    #[test]
    fn current_properties_validate_and_unknown_ones_do_not() {
        assert!(
            validate_build_properties([("appearance", "density"), ("toasts", "position")]).is_ok()
        );
        assert!(validate_build_properties([("appearance", "not-a-key")]).is_err());
        assert_eq!(
            supported().count(),
            PROPERTIES.len(),
            "nothing in the registry is newer than this toolkit"
        );
    }
}
