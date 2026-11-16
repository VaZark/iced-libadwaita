// DECISION (user, 2026-11-16): theme and desktop files are RON only. There is no INI syntax, no
// `.conf` fallback and no `(section, key) -> String` map: a file is read as RON values and the
// schema types them (`VariableType::from_ron`), so structured tokens (gradients, shadows) are
// native Rust-shaped data. `../weaver` and `../organiser-apps` are converted only on approval.
//! A theme or desktop file, read natively as RON: a map of sections to maps of keys, each key a
//! RON value that the schema turns into a typed [`Value`](super::Value) (see
//! [`VariableType::from_ron`](super::VariableType::from_ron)).
//!
//! ```ron
//! {
//!     // Comments are plain RON comments.
//!     "colors": { "accent": "#3584e4", "window-bg": "#fafafb" },
//!     "metrics": { "radius-window": 15 },
//!     "gradients": { "titlebar": (angle: 180, stops: [(0.0, "#eeeeee"), (1.0, "#c5c5c5")]) },
//!     "dark:colors": { "window-bg": "#242424" },
//! }
//! ```
//!
//! Section names are strings (a variation is `"dark:colors"`), so they may contain `-` and `:`.
use std::collections::BTreeMap;

/// The sections of a file and the keys in each. Empty text is an empty document.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Document {
    sections: BTreeMap<String, BTreeMap<String, ron::Value>>,
}

impl Document {
    /// Parses `text`. A syntax error says where (`line:column`) and what; a document that is
    /// not a map of sections to maps of keys says which section is wrong.
    pub fn parse(text: &str) -> Result<Self, String> {
        use ron::Value as R;
        // Nothing but whitespace and `//` comments (a fresh template) sets nothing.
        if text.lines().all(|l| {
            let l = l.trim();
            l.is_empty() || l.starts_with("//")
        }) {
            return Ok(Self::default());
        }
        let value: R = ron::from_str(text).map_err(|e| e.to_string())?;
        let R::Map(sections) = value else {
            return Err("a file is a map of sections: { \"colors\": { \"accent\": \"#3584e4\" } }".into());
        };
        let mut doc = Self::default();
        for (name, keys) in sections.iter() {
            let R::String(name) = name else {
                return Err(format!("a section name must be a string, not {name:?}"));
            };
            let R::Map(keys) = keys else {
                return Err(format!("section {name:?} must be a map of keys: {{ \"key\": value }}"));
            };
            let section = doc.sections.entry(name.clone()).or_default();
            for (key, value) in keys.iter() {
                let R::String(key) = key else {
                    return Err(format!("a key in {name:?} must be a string, not {key:?}"));
                };
                section.insert(key.clone(), value.clone());
            }
        }
        Ok(doc)
    }

    /// Lays `later` over this document: its keys replace the same keys here.
    pub fn merge(&mut self, later: Document) {
        for (section, keys) in later.sections {
            self.sections.entry(section).or_default().extend(keys);
        }
    }

    /// The value of `key` in `section`, if the file sets it.
    pub fn get(&self, section: &str, key: &str) -> Option<&ron::Value> {
        self.sections.get(section)?.get(key)
    }

    /// The string a key holds (`None` when unset or not a string).
    pub fn text(&self, section: &str, key: &str) -> Option<&str> {
        match self.get(section, key)? {
            ron::Value::String(s) => Some(s),
            _ => None,
        }
    }

    /// Sets `key` in `section`.
    pub fn set(&mut self, section: &str, key: &str, value: ron::Value) {
        self.sections
            .entry(section.to_string())
            .or_default()
            .insert(key.to_string(), value);
    }

    /// The RON text of this document: sections and keys in the order `order` ranks them (a
    /// section's base first, then its variations), `header` as `//` comments on top. A value the
    /// schema understands is written the way a theme file is written by hand (a gradient as
    /// `(angle: .., stops: [..])`); anything else is kept as it was read.
    pub fn to_text(&self, header: &str, schema: &super::Schema) -> String {
        let mut out = String::new();
        for line in header.lines() {
            out.push_str(&format!("// {line}\n"));
        }
        out.push_str("{\n");
        let ron_text = |section: &str, key: &str, v: &ron::Value| -> String {
            let base = section.split_once(':').map_or(section, |(_, b)| b);
            if let Some(spec) = schema.get(base, key)
                && let Ok(typed) = spec.ty.from_ron(v)
            {
                return typed.to_ron_text();
            }
            ron::ser::to_string(v).unwrap_or_else(|_| "()".into())
        };
        let rank = |section: &str| {
            let base = section.split_once(':').map_or(section, |(_, b)| b);
            schema.specs().iter().position(|s| s.section == base).unwrap_or(usize::MAX)
        };
        let mut sections: Vec<&String> = self.sections.keys().collect();
        sections.sort_by_key(|s| {
            let variation = s.split_once(':').map_or(0, |(v, _)| 1 + usize::from(v != "light"));
            (rank(s), s.split_once(':').map_or(s.as_str(), |(_, b)| b).to_string(), variation)
        });
        for section in sections {
            let keys = &self.sections[section];
            if keys.is_empty() {
                continue;
            }
            out.push_str(&format!("\n    {section:?}: {{\n"));
            let base = section.split_once(':').map_or(section.as_str(), |(_, b)| b);
            let mut keys: Vec<(&String, &ron::Value)> = keys.iter().collect();
            keys.sort_by_key(|(k, _)| {
                (
                    schema.specs().iter().position(|s| s.section == base && s.key == **k).unwrap_or(usize::MAX),
                    (*k).clone(),
                )
            });
            for (key, v) in keys {
                out.push_str(&format!("        {key:?}: {},\n", ron_text(section, key, v)));
            }
            out.push_str("    },\n");
        }
        out.push_str("}\n");
        out
    }

    /// Every `(section, key, value)`, in section then key order.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &str, &ron::Value)> {
        self.sections.iter().flat_map(|(section, keys)| {
            keys.iter()
                .map(move |(key, value)| (section.as_str(), key.as_str(), value))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_sections_keys_comments_and_variations() {
        let doc = Document::parse(
            r##"
            // a comment
            {
                "colors": { "accent": "#3584e4" },
                "dark:colors": { "accent": "#78aeed" }, /* trailing comma and block comment */
                "metrics": { "radius-window": 15, "scale": 1.5, "flat": true },
            }
            "##,
        )
        .unwrap();
        assert_eq!(doc.text("colors", "accent"), Some("#3584e4"));
        assert_eq!(doc.text("dark:colors", "accent"), Some("#78aeed"));
        assert!(matches!(doc.get("metrics", "radius-window"), Some(ron::Value::Number(_))));
        assert_eq!(doc.get("metrics", "missing"), None);
        assert_eq!(doc.entries().count(), 5);
    }

    #[test]
    fn empty_text_is_an_empty_document() {
        assert_eq!(Document::parse("").unwrap(), Document::default());
        assert_eq!(Document::parse("  \n// only a comment\n").unwrap(), Document::default());
    }

    #[test]
    fn a_syntax_error_says_where() {
        let e = Document::parse("{\n  \"colors\": { \"accent\": #3584e4 },\n}").unwrap_err();
        assert!(e.contains("2:"), "{e}");
    }

    #[test]
    fn the_shape_is_checked() {
        assert!(Document::parse("[1, 2]").unwrap_err().contains("map of sections"));
        assert!(Document::parse("{ \"colors\": 3 }").unwrap_err().contains("colors"));
        assert!(Document::parse("{ 1: {} }").unwrap_err().contains("string"));
        assert!(Document::parse("{ \"colors\": { 1: 2 } }").unwrap_err().contains("string"));
    }
}
