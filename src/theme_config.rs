//! Per-app theme configuration: how an app styles itself differently from the desktop, in
//! layers that can each be deleted and regenerated.
//!
//! The [desktop config](crate::config) is shared by every app. Sometimes one app needs its own
//! look: custom colors, its own paddings and radii, tokens for objects only it has, or just a
//! different vibe than something utilitarian. That is this module. Priority, **highest first**:
//!
//! 1. **Hardcoded** ([`Builder::hardcode`]): set in code. *Extremely discouraged.* Prefer a
//!    token in the app theme: a hardcoded value cannot be changed by the user or the
//!    packager, and every one is reported at startup ([`ThemeConfig::hardcoded`]).
//! 2. **The app theme file**, `$XDG_CONFIG_HOME/<app-id>/theme.conf`. The app declares its
//!    tokens and their defaults in a [`Schema`], so the file can be deleted: the app writes a
//!    fresh one (every token commented out with its default and a description).
//! 3. **The toolkit theme file**, `$XDG_CONFIG_HOME/iced-adaptive-design/theme.conf`. The toolkit
//!    declares the tokens it understands ([`Schema::toolkit`]) with their defaults; it too is
//!    regenerated when missing.
//!
//! Below these, an unset token is the **declared default**, and the toolkit's own look is
//! whatever the [desktop config](crate::config) makes of it. A token only changes anything
//! when someone sets it: the generated files are comments, so they document and never pin.
//!
//! # Tokens
//!
//! A token is a typed value ([`Value`]: a color, a number, a bool or text) under a section and a
//! key, in the same key-file format as the desktop config.
//!
//! * **Toolkit tokens** (see [`Schema::toolkit`]): `[colors]` and `[colors-dark]` (the palette:
//!   `accent`, `window-bg`, `card-bg`, …; `colors-dark` wins on a dark scheme), `[metrics]`
//!   (radii, paddings, heights, spacing) and `[typography]` (`text-scale`, `base-size`).
//!   Radii are **not** clamped to the HIG maxima here: a different vibe may be rounder or squarer.
//! * **App tokens**: any other section and key the app declares, for its own objects: a cover's
//!   size, a brand color, a chip's padding. Read them with
//!   [`Theme::token_color`](crate::Theme::token_color) and friends, or from the
//!   [`ThemeConfig`].
//!
//! ```
//! use iced_adaptive_design::theme_config::{Schema, ThemeConfig, Source};
//!
//! let schema = Schema::new()
//!     .color("cover", "background", "#9c4ad1", "The placeholder behind a missing cover")
//!     .number("cover", "size", 88.0, "Cover edge in px");
//!
//! let cfg = ThemeConfig::builder("org.example.Player")
//!     .schema(schema)
//!     .app_text("[cover]\nsize=120\n[metrics]\nradius-l=20\n")   // the app theme file's text
//!     .build();
//!
//! assert_eq!(cfg.number("cover", "size"), Some(120.0));            // the app file
//! assert_eq!(cfg.source("cover", "size"), Source::App);
//! assert_eq!(cfg.number("metrics", "radius-l"), Some(20.0));       // a toolkit token, set by the app
//! assert_eq!(cfg.source("cover", "background"), Source::Default);  // never set: the declared default
//! ```
//!
//! # Using it
//!
//! ```ignore
//! let cfg = ThemeConfig::builder("org.example.Player").schema(my_schema()).build();
//! theme_config::install(cfg.clone());                       // reports hardcoded tokens
//! config::install(Config::load().with_theme_config(&cfg));  // metric tokens reach every widget
//! let theme = Theme::from_config(&config::current()).with_theme_config(&cfg);
//! ```

mod variable;
pub use variable::{Rounding, VariableType};

use crate::metrics::{Metrics, Typography};
use crate::palette::{AccentColor, Palette, Scheme};
use iced::Color;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

// ----------------------------------------------------------------- values

/// A token's value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Decimal(f64),
    Length(f64),
    Percentage(f64),
    Color(Color),
    Number(f32),
    Bool(bool),
    Text(String),
}

/// What type a token holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Integer,
    Decimal,
    Length,
    Percentage,
    Color,
    Number,
    Bool,
    Text,
}

impl Value {
    pub fn kind(&self) -> Kind {
        match self {
            Self::Integer(_) => Kind::Integer,
            Self::Decimal(_) => Kind::Decimal,
            Self::Length(_) => Kind::Length,
            Self::Percentage(_) => Kind::Percentage,
            Self::Color(_) => Kind::Color,
            Self::Number(_) => Kind::Number,
            Self::Bool(_) => Kind::Bool,
            Self::Text(_) => Kind::Text,
        }
    }
    pub fn as_number(&self) -> Option<f64> {
        match self {
            Self::Integer(n) => Some(*n as f64),
            Self::Decimal(n) | Self::Length(n) | Self::Percentage(n) => Some(*n),
            Self::Number(n) => Some(*n as f64),
            _ => None,
        }
    }
    /// The value as it is written in a file.
    pub fn to_text(&self) -> String {
        match self {
            Self::Integer(n) => n.to_string(),
            Self::Decimal(n) => n.to_string(),
            Self::Length(n) => format!("{n}px"),
            Self::Percentage(n) => format!("{n}%"),
            Self::Color(c) => crate::color::to_hex(*c),
            Self::Number(n) => n.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::Text(t) => t.clone(),
        }
    }
}

// ----------------------------------------------------------------- schema

/// One declared token: where it lives, what it defaults to, what it is for.
#[derive(Debug, Clone, PartialEq)]
pub struct Spec {
    pub section: String,
    pub key: String,
    /// Used only when no valid value survives the cascade. None means intentionally unset.
    pub default: Option<Value>,
    pub ty: VariableType,
    pub doc: String,
}

/// The tokens a layer declares, with their defaults. The toolkit declares its own
/// ([`Schema::toolkit`]); an app declares the tokens of its own objects.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Schema {
    specs: Vec<Spec>,
    variations: Vec<String>,
}

impl Schema {
    pub fn new() -> Self {
        Self::default()
    }
    /// Restrict variation keys to this string enumeration. Light and dark are built in.
    pub fn variations(mut self, keys: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for key in keys {
            let key = key.into();
            assert!(
                !key.is_empty()
                    && key
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
                "invalid variation key"
            );
            if !["light", "dark"].contains(&key.as_str()) && !self.variations.contains(&key) {
                self.variations.push(key);
            }
        }
        self
    }
    pub fn variation_keys(&self) -> impl Iterator<Item = &str> {
        ["light", "dark"]
            .into_iter()
            .chain(self.variations.iter().map(String::as_str))
    }
    /// Validate a .conf against exactly this schema, without reading or writing files.
    pub fn validate(&self, text: &str) -> ValidationReport {
        let mut report = ValidationReport {
            errors: syntax_errors(text),
            unknown: Vec::new(),
        };
        for ((section, key), raw) in crate::config::parse_keyfile(text) {
            let base = if let Some((variation, base)) = section.split_once(':') {
                if !self.variation_keys().any(|k| k == variation) {
                    report.unknown.push((section, key));
                    continue;
                }
                base
            } else {
                &section
            };
            match self.get(base, &key) {
                Some(spec) if spec.ty.parse(&raw).is_none() => report
                    .errors
                    .push(format!("[{section}] {key}: invalid {:?}", spec.ty)),
                None => report.unknown.push((section, key)),
                _ => {}
            }
        }
        report
    }
    /// Declare a typed variable and its optional fallback. Invalid schema declarations panic.
    pub fn variable(
        mut self,
        section: &str,
        key: &str,
        ty: VariableType,
        fallback: Option<Value>,
        doc: &str,
    ) -> Self {
        assert!(
            ty.well_formed(),
            "invalid type constraints for [{section}] {key}"
        );
        assert!(
            !section.is_empty()
                && !section.contains([':', '[', ']', '\n', '\r'])
                && !key.is_empty()
                && !key.contains(['=', '[', ']', '\n', '\r']),
            "invalid variable identifier"
        );
        let fallback = fallback.map(|value| {
            assert_eq!(ty.kind(), value.kind(), "fallback type mismatch");
            if let Value::Color(color) = &value {
                assert!(
                    [color.r, color.g, color.b, color.a]
                        .into_iter()
                        .all(|n| n.is_finite() && (0.0..=1.0).contains(&n)),
                    "fallback color has invalid channels"
                );
            }

            ty.parse(&value.to_text())
                .expect("fallback violates its variable constraints")
        });
        self.specs
            .retain(|s| !(s.section == section && s.key == key));
        self.specs.push(Spec {
            section: section.into(),
            key: key.into(),
            default: fallback,
            ty,
            doc: doc.into(),
        });
        self
    }
    fn push(self, section: &str, key: &str, default: Value, doc: &str) -> Self {
        let ty = VariableType::untyped(default.kind())
            .expect("typed variables use Schema::variable");
        self.variable(section, key, ty, Some(default), doc)
    }
    /// A color token; the fallback accepts any supported color format.
    ///
    /// # Panics
    /// If `default` is not a color: a schema is written by the app's author, so a typo is a bug.
    pub fn color(self, section: &str, key: &str, default: &str, doc: &str) -> Self {
        let value = VariableType::Color
            .parse(default)
            .unwrap_or_else(|| panic!("theme token [{section}] {key}: {default:?} is not a color"));
        self.push(section, key, value, doc)
    }

    pub fn number(self, section: &str, key: &str, default: f32, doc: &str) -> Self {
        self.push(section, key, Value::Number(default), doc)
    }
    pub fn boolean(self, section: &str, key: &str, default: bool, doc: &str) -> Self {
        self.push(section, key, Value::Bool(default), doc)
    }
    pub fn text(self, section: &str, key: &str, default: &str, doc: &str) -> Self {
        self.push(section, key, Value::Text(default.into()), doc)
    }
    /// This schema with `other`'s tokens added (`other` wins a clash).
    pub fn with(mut self, other: &Schema) -> Self {
        self = self.variations(other.variations.clone());
        for s in &other.specs {
            self = self.variable(&s.section, &s.key, s.ty.clone(), s.default.clone(), &s.doc);
        }
        self
    }
    pub fn get(&self, section: &str, key: &str) -> Option<&Spec> {
        self.specs
            .iter()
            .find(|s| s.section == section && s.key == key)
    }
    pub fn specs(&self) -> &[Spec] {
        &self.specs
    }

    /// The tokens the toolkit understands, with the toolkit's own values as defaults.
    pub fn toolkit() -> Self {
        let mut s = Self::new();
        let base = Palette::new(Scheme::Light);
        let dark = Palette::new(Scheme::Dark);
        // The accent drives a family of colors, so it is not a plain palette field.
        s = s.push(
            "colors",
            "accent",
            Value::Color(base.accent_bg_color),
            "The accent color (derives its text and standalone variants)",
        );
        s = s.push(
            "colors-dark",
            "accent",
            Value::Color(dark.accent_bg_color),
            "The accent color on a dark scheme",
        );
        for (key, doc, get, _) in COLORS {
            s = s.push("colors", key, Value::Color(get(&base)), doc);
            s = s.push("colors-dark", key, Value::Color(get(&dark)), doc);
        }
        let m = Metrics::default();
        for (key, doc, get, _) in METRICS {
            s = s.push("metrics", key, Value::Number(get(&m)), doc);
        }
        let t = Typography::default();
        s = s.push(
            "typography",
            "text-scale",
            Value::Number(1.0),
            "Multiplies every text size",
        );
        s.push(
            "typography",
            "base-size",
            Value::Number(t.base),
            "The body text size in px, before the scale",
        )
    }

    /// The text of a file for this schema: every token commented out with its default and a
    /// description, so the file documents without pinning anything. Deleting a layer's file and
    /// asking for it again gives this.
    pub fn template(&self, title: &str) -> String {
        let mut out = format!(
            "# {title}\n# Generated from the declared defaults. Delete this file to get a fresh one.\n# Uncomment a line to change it; commented lines are the defaults and change nothing.\n"
        );
        let mut sections: Vec<&str> = Vec::new();
        for s in &self.specs {
            if !sections.contains(&s.section.as_str()) {
                sections.push(&s.section);
            }
        }
        for section in sections {
            out.push_str(&format!("\n[{section}]\n"));
            for s in self.specs.iter().filter(|s| s.section == section) {
                out.push_str(&format!(
                    "# {}\n# Type: {:?}\n#{}={}\n",
                    s.doc.replace('\n', "\n# "),
                    s.ty,
                    s.key,
                    s.default
                        .as_ref()
                        .map(Value::to_text)
                        .unwrap_or_else(|| "# unset (no fallback)".into())
                ));
            }
        }
        for variation in self.variation_keys() {
            for section in &sections_for_schema(self) {
                out.push_str(&format!("\n[{variation}:{section}]\n# Overrides [{section}] while {variation} is active.\n"));
                for spec in self.specs.iter().filter(|s| &s.section == section) {
                    out.push_str(&format!(
                        "#{}={}\n",
                        spec.key,
                        spec.default
                            .as_ref()
                            .map(Value::to_text)
                            .unwrap_or_else(|| "# unset (no fallback)".into())
                    ));
                }
            }
        }
        out
    }
}
/// Validation errors and undeclared keys, including undeclared variation keys.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidationReport {
    pub errors: Vec<String>,
    pub unknown: Vec<(String, String)>,
}
impl ValidationReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty() && self.unknown.is_empty()
    }
}
fn syntax_errors(text: &str) -> Vec<String> {
    let mut section = false;
    text.lines()
        .enumerate()
        .filter_map(|(i, line)| {
            let line = line.trim();
            if line.is_empty() || line.starts_with(['#', ';']) {
                return None;
            }
            if line.starts_with('[') {
                if line.ends_with(']')
                    && line.len() > 2
                    && !line[1..line.len() - 1].contains(['[', ']'])
                {
                    section = true;
                    return None;
                }
            } else if section
                && line
                    .split_once('=')
                    .is_some_and(|(key, _)| !key.trim().is_empty())
            {
                return None;
            }
            Some(format!(
                "line {}: expected [section] or key=value inside a section",
                i + 1
            ))
        })
        .collect()
}
fn sections_for_schema(schema: &Schema) -> Vec<String> {
    schema
        .specs
        .iter()
        .map(|s| s.section.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

// ----------------------------------------------------------- the tables

type ColorGet = fn(&Palette) -> Color;
type ColorSet = fn(&mut Palette, Color);
type MetricGet = fn(&Metrics) -> f32;
type MetricSet = fn(&mut Metrics, f32);

macro_rules! colors {
    ($($key:literal, $doc:literal, $field:ident;)*) => {
        /// The palette tokens: key, description, read, write.
        const COLORS: &[(&str, &str, ColorGet, ColorSet)] = &[
            $(($key, $doc, |p| p.$field, |p, c| p.$field = c),)*
        ];
    };
}

colors! {
    "window-bg", "The window background", window_bg_color;
    "window-fg", "Text on the window", window_fg_color;
    "view-bg", "Content views, entries", view_bg_color;
    "view-fg", "Text on views", view_fg_color;
    "card-bg", "Cards and boxed lists", card_bg_color;
    "card-fg", "Text on cards", card_fg_color;
    "headerbar-bg", "Header bars", headerbar_bg_color;
    "headerbar-fg", "Text on header bars", headerbar_fg_color;
    "sidebar-bg", "The sidebar", sidebar_bg_color;
    "sidebar-fg", "Text on the sidebar", sidebar_fg_color;
    "popover-bg", "Popovers and menus", popover_bg_color;
    "popover-fg", "Text on popovers", popover_fg_color;
    "dialog-bg", "Dialogs", dialog_bg_color;
    "dialog-fg", "Text on dialogs", dialog_fg_color;
    "success", "Success color", success_bg_color;
    "warning", "Warning color", warning_bg_color;
    "destructive", "Destructive action color", destructive_bg_color;
    "error", "Error color", error_bg_color;
}

macro_rules! metrics {
    ($($key:literal, $doc:literal, $get:expr, $set:expr;)*) => {
        /// The metric tokens: key, description, read, write.
        const METRICS: &[(&str, &str, MetricGet, MetricSet)] = &[
            $(($key, $doc, $get, $set),)*
        ];
    };
}

metrics! {
    "space-xs", "Smallest gap", |m| m.space_xs, |m, v| m.space_xs = v;
    "space-s", "Small gap", |m| m.space_s, |m, v| m.space_s = v;
    "space-m", "Medium gap", |m| m.space_m, |m, v| m.space_m = v;
    "space-l", "Large gap", |m| m.space_l, |m, v| m.space_l = v;
    "space-xl", "Largest gap", |m| m.space_xl, |m, v| m.space_xl = v;
    "radius-s", "Small corner radius", |m| m.radius_s, |m, v| m.radius_s = v;
    "radius-m", "Buttons and entries", |m| m.radius_m, |m, v| m.radius_m = v;
    "radius-l", "Cards and boxed lists", |m| m.radius_l, |m, v| m.radius_l = v;
    "radius-menu", "Menu items", |m| m.radius_menu, |m, v| m.radius_menu = v;
    "radius-popover", "Popovers", |m| m.radius_popover, |m, v| m.radius_popover = v;
    "radius-toast", "Toasts", |m| m.radius_toast, |m, v| m.radius_toast = v;
    "radius-window", "Windows and dialogs", |m| m.radius_window, |m, v| m.radius_window = v;
    "button-padding-y", "Button vertical padding", |m| m.button_padding.top, |m, v| { m.button_padding.top = v; m.button_padding.bottom = v };
    "button-padding-x", "Button horizontal padding", |m| m.button_padding.left, |m, v| { m.button_padding.left = v; m.button_padding.right = v };
    "button-min-height", "The least height of a button", |m| m.button_min_height, |m, v| m.button_min_height = v;
    "entry-padding-y", "Entry vertical padding", |m| m.entry_padding.top, |m, v| { m.entry_padding.top = v; m.entry_padding.bottom = v };
    "entry-padding-x", "Entry horizontal padding", |m| m.entry_padding.left, |m, v| { m.entry_padding.left = v; m.entry_padding.right = v };
    "entry-min-height", "The least height of an entry", |m| m.entry_min_height, |m, v| m.entry_min_height = v;
    "border-width", "Border width", |m| m.border_width, |m, v| m.border_width = v;
    "icon-size", "Icon size", |m| m.icon_size, |m, v| m.icon_size = v;
    "row-padding-y", "Row vertical padding", |m| m.row_padding.top, |m, v| { m.row_padding.top = v; m.row_padding.bottom = v };
    "row-padding-x", "Row horizontal padding", |m| m.row_padding.left, |m, v| { m.row_padding.left = v; m.row_padding.right = v };
    "row-min-height", "The least height of a row", |m| m.row_min_height, |m, v| m.row_min_height = v;
    "row-spacing", "Space between a row's parts", |m| m.row_spacing, |m, v| m.row_spacing = v;
    "card-padding", "Padding inside cards", |m| m.card_padding.top, |m, v| m.card_padding = iced::Padding::from(v);
    "dialog-padding", "Padding inside dialogs", |m| m.dialog_padding.top, |m, v| m.dialog_padding = iced::Padding::from(v);
    "popover-padding", "Padding inside popovers", |m| m.popover_padding, |m, v| m.popover_padding = v;
    "page-padding-y", "Page vertical padding", |m| m.page_padding.top, |m, v| { m.page_padding.top = v; m.page_padding.bottom = v };
    "page-padding-x", "Page horizontal padding", |m| m.page_padding.left, |m, v| { m.page_padding.left = v; m.page_padding.right = v };
    "group-spacing", "Space between groups", |m| m.group_spacing, |m, v| m.group_spacing = v;
    "clamp-maximum", "The readable width of a page", |m| m.clamp_maximum, |m, v| m.clamp_maximum = v;
    "headerbar-height", "Header bar height", |m| m.headerbar_height, |m, v| m.headerbar_height = v;
    "gutter", "The distance page content keeps from the window edge", |m| m.gutter, |m, v| m.gutter = v;
    "band-air", "Air above and below a toolbar band", |m| m.band_air, |m, v| m.band_air = v;
    "sidebar-width", "Sidebar width", |m| m.sidebar_width, |m, v| m.sidebar_width = v;
    "sidebar-row-height", "Sidebar row height", |m| m.sidebar_row_height, |m, v| m.sidebar_row_height = v;
    "scrollbar-width", "Scrollbar width", |m| m.scrollbar_width, |m, v| m.scrollbar_width = v;
}

// ------------------------------------------------------------ the layers

/// Where a token's value came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    /// Never set: the declared default (and, for a toolkit token, whatever the desktop config
    /// makes of it).
    Default,
    /// The toolkit theme file.
    Toolkit,
    /// The app theme file.
    App,
    /// Set in code. Discouraged.
    Hardcoded,
}

impl Source {
    pub fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Toolkit => "toolkit theme",
            Self::App => "app theme",
            Self::Hardcoded => "hardcoded",
        }
    }
}

/// The directory the user's config lives in.
fn config_home() -> Option<PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
}

/// Where the toolkit theme file is.
pub fn toolkit_path() -> Option<PathBuf> {
    config_home().map(|h| h.join("iced-adaptive-design/theme.conf"))
}

/// Where the theme file of the app `app_id` is.
pub fn app_path(app_id: &str) -> Option<PathBuf> {
    config_home().map(|h| h.join(app_id).join("theme.conf"))
}

/// Builds a [`ThemeConfig`]. See the [module](self).
pub struct Builder {
    app_id: String,
    schema: Schema,
    toolkit_text: Option<String>,
    app_text: Option<String>,
    hardcoded: Vec<(String, String, Value)>,
    files: bool,
    regenerate: bool,
    variation: Option<String>,
}

impl Builder {
    fn new(app_id: &str) -> Self {
        Self {
            app_id: app_id.to_string(),
            schema: Schema::new(),
            toolkit_text: None,
            app_text: None,
            hardcoded: Vec::new(),
            files: true,
            regenerate: true,
            variation: None,
        }
    }
    pub fn variation(mut self, key: impl Into<String>) -> Self {
        self.variation = Some(key.into());
        self
    }
    /// The tokens of the app's own objects, with their defaults.
    pub fn schema(mut self, schema: Schema) -> Self {
        self.schema = schema;
        self
    }
    /// Uses `text` as the app theme file instead of reading it (tests, a settings preview).
    /// Nothing is read or written for this layer.
    pub fn app_text(mut self, text: impl Into<String>) -> Self {
        self.app_text = Some(text.into());
        self
    }
    /// Uses `text` as the toolkit theme file instead of reading it.
    pub fn toolkit_text(mut self, text: impl Into<String>) -> Self {
        self.toolkit_text = Some(text.into());
        self
    }
    /// Reads no files and writes none: only the given texts, the hardcoded tokens and the
    /// defaults count.
    pub fn files(mut self, files: bool) -> Self {
        self.files = files;
        self
    }
    /// Whether a missing theme file is written from its declared defaults (default yes). That is
    /// what makes a file safe to delete.
    pub fn regenerate(mut self, regenerate: bool) -> Self {
        self.regenerate = regenerate;
        self
    }
    /// Sets a token **in code**, over every file. *Extremely discouraged*: use a token in the
    /// app theme, so the value can be changed without a rebuild and is documented in the
    /// generated file. Every hardcoded token is reported by [`install`] and listed in
    /// [`ThemeConfig::hardcoded`].
    #[must_use]
    pub fn hardcode(mut self, section: &str, key: &str, value: Value) -> Self {
        self.hardcoded.push((section.into(), key.into(), value));
        self
    }

    fn read_layer(
        &self,
        text: &Option<String>,
        path: Option<PathBuf>,
        template: &Schema,
        title: &str,
    ) -> String {
        if let Some(t) = text {
            return t.clone();
        }
        if !self.files {
            return String::new();
        }
        let Some(path) = path else {
            return String::new();
        };
        match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(_) => {
                if self.regenerate {
                    if let Some(dir) = path.parent() {
                        let _ = std::fs::create_dir_all(dir);
                    }
                    let _ = std::fs::write(&path, template.template(title));
                }
                String::new()
            }
        }
    }

    pub fn build(self) -> ThemeConfig {
        let toolkit = Schema::toolkit();
        let schema = toolkit.clone().with(&self.schema);
        let toolkit_text = self.read_layer(
            &self.toolkit_text,
            toolkit_path(),
            &toolkit,
            "iced-adaptive-design theme: the toolkit's default look",
        );
        let app_text = self.read_layer(
            &self.app_text,
            app_path(&self.app_id),
            &self.schema,
            &format!("{} theme: this app's look", self.app_id),
        );

        let mut cfg = ThemeConfig {
            values: BTreeMap::new(),
            schema,
            warnings: Vec::new(),
            unknown: Vec::new(),
            hardcoded: Vec::new(),
            variation: None,
        };
        for s in cfg.schema.specs().to_vec() {
            cfg.values
                .insert((s.section, s.key), (s.default, Source::Default));
        }
        // Lowest priority first, so a later layer overwrites an earlier one.
        for (text, source) in [(&toolkit_text, Source::Toolkit), (&app_text, Source::App)] {
            cfg.warnings.extend(
                syntax_errors(text)
                    .into_iter()
                    .map(|error| format!("{}: {error}", source.label())),
            );
            for ((section, key), raw) in crate::config::parse_keyfile(text) {
                cfg.set_from_text(&section, &key, &raw, source);
            }
        }
        for (section, key, value) in self.hardcoded {
            let valid = cfg.schema.get(&section, &key).is_some_and(|spec| {
                spec.ty.kind() == value.kind() && spec.ty.parse(&value.to_text()).is_some()
            });
            if valid {
                cfg.set_from_text(&section, &key, &value.to_text(), Source::Hardcoded);
                cfg.hardcoded.push((section, key));
            } else {
                cfg.warnings.push(format!(
                    "[{section}] {key}: invalid or undeclared hardcoded value; ignored"
                ));
            }
        }
        if let Some(variation) = self.variation {
            cfg = cfg.with_variation(&variation);
        }

        cfg
    }
}

/// The resolved tokens: hardcoded, over the app theme, over the toolkit theme, over the
/// declared defaults. See the [module](self).
#[derive(Debug, Clone, PartialEq)]
pub struct ThemeConfig {
    values: BTreeMap<(String, String), (Option<Value>, Source)>,
    schema: Schema,
    warnings: Vec<String>,
    unknown: Vec<(String, String)>,
    hardcoded: Vec<(String, String)>,
    variation: Option<String>,
}

impl ThemeConfig {
    /// Starts a config for the app `app_id` (the directory of its theme file).
    pub fn builder(app_id: &str) -> Builder {
        Builder::new(app_id)
    }

    fn set_from_text(&mut self, section: &str, key: &str, raw: &str, source: Source) {
        let base = if let Some((variation, base)) = section.split_once(':') {
            if !self.schema.variation_keys().any(|key| key == variation) {
                self.unknown.push((section.into(), key.into()));
                return;
            }
            base
        } else {
            section
        };
        let Some(spec) = self.schema.get(base, key) else {
            self.unknown.push((section.into(), key.into()));
            return;
        };
        match spec.ty.parse(raw) {
            Some(value) => { self.values.insert((section.into(), key.into()), (Some(value), source)); }
            None => self.warnings.push(format!("[{section}] {key}: {raw:?} is invalid for {:?}; next valid lower layer or declared fallback used", spec.ty)),
        }
    }
    /// Return a config with this declared variation cascaded over every base property.
    pub fn with_variation(mut self, variation: &str) -> Self {
        if self.schema.variation_keys().any(|key| key == variation) {
            self.variation = Some(variation.into());
        } else {
            self.warnings
                .push(format!("unknown variation {variation:?}; base values used"));
            self.variation = None;
        }
        self
    }
    pub fn active_variation(&self) -> Option<&str> {
        self.variation.as_deref()
    }
    fn resolved(&self, section: &str, key: &str) -> Option<&(Option<Value>, Source)> {
        if let Some(variation) = &self.variation {
            if let Some(value) = self
                .values
                .get(&(format!("{variation}:{section}"), key.into()))
            {
                return Some(value);
            }
        }
        self.values.get(&(section.into(), key.into()))
    }
    pub fn get(&self, section: &str, key: &str) -> Option<&Value> {
        self.resolved(section, key)
            .and_then(|(value, _)| value.as_ref())
    }

    pub fn color(&self, section: &str, key: &str) -> Option<Color> {
        match self.get(section, key)? {
            Value::Color(c) => Some(*c),
            _ => None,
        }
    }
    pub fn number(&self, section: &str, key: &str) -> Option<f32> {
        self.get(section, key)?
            .as_number()
            .map(|n| n as f32)
            .filter(|n| n.is_finite())
    }
    pub fn bool(&self, section: &str, key: &str) -> Option<bool> {
        match self.get(section, key)? {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub fn text(&self, section: &str, key: &str) -> Option<&str> {
        match self.get(section, key)? {
            Value::Text(t) => Some(t),
            _ => None,
        }
    }
    /// Where the token's value came from ([`Source::Default`] when it is not declared).
    pub fn source(&self, section: &str, key: &str) -> Source {
        self.resolved(section, key)
            .map_or(Source::Default, |(_, source)| *source)
    }
    /// Whether anyone set the token: it differs from "unset", so it should take effect.
    pub fn is_set(&self, section: &str, key: &str) -> bool {
        self.source(section, key) != Source::Default
    }
    /// Every resolved non-None token, its value and where it came from.
    pub fn entries(&self) -> impl Iterator<Item = (&Spec, &Value, Source)> {
        self.schema.specs().iter().filter_map(|s| {
            self.get(&s.section, &s.key)
                .map(|v| (s, v, self.source(&s.section, &s.key)))
        })
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }
    /// Invalid values and syntax found in the files. Invalid properties fall through to the next valid layer, then the declared fallback.
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }
    /// `(section, key)` found in the files but not declared by the toolkit or the app. Ignored
    /// and never deleted.
    pub fn unknown(&self) -> &[(String, String)] {
        &self.unknown
    }
    /// The tokens set in code. Each should be a token in the app theme instead.
    pub fn hardcoded(&self) -> &[(String, String)] {
        &self.hardcoded
    }

    /// The toolkit metric tokens that were set, applied to `metrics`. Radii are not clamped.
    pub fn apply_to_metrics(&self, metrics: &mut Metrics) {
        for (key, _, _, set) in METRICS {
            if self.is_set("metrics", key)
                && let Some(v) = self.number("metrics", key)
            {
                set(metrics, v);
            }
        }
    }

    /// The names (`metrics` keys) of the metric tokens that were set, for reports.
    pub fn metric_overrides(&self) -> Vec<(&'static str, f32)> {
        METRICS
            .iter()
            .filter(|(k, ..)| self.is_set("metrics", k))
            .filter_map(|(k, ..)| self.number("metrics", k).map(|v| (*k, v)))
            .collect()
    }

    /// The toolkit color tokens that were set for `scheme`, applied to `palette`:
    /// `[colors]`, then `[colors-dark]` over it on a dark scheme.
    pub fn apply_to_palette(&self, palette: &mut Palette, scheme: Scheme) {
        if self.variation.is_none() {
            return self
                .clone()
                .with_variation(if scheme.is_dark() { "dark" } else { "light" })
                .apply_to_palette(palette, scheme);
        }
        let variation_section = format!("{}:colors", self.variation.as_deref().unwrap());
        let mut sections = vec!["colors"];
        if scheme.is_dark() {
            sections.push("colors-dark");
        }
        // Explicit variation sections also win over the legacy dark-color section.
        sections.push(&variation_section);
        for section in sections {
            if self.is_set(section, "accent")
                && let Some(c) = self.color(section, "accent")
            {
                let rgb = ((c.r * 255.0).round() as u32) << 16
                    | ((c.g * 255.0).round() as u32) << 8
                    | (c.b * 255.0).round() as u32;
                *palette = palette.with_accent(AccentColor::Custom(rgb), scheme);
            }
            for (key, _, _, set) in COLORS {
                if self.is_set(section, key)
                    && let Some(c) = self.color(section, key)
                {
                    set(palette, c);
                }
            }
        }
    }

    /// The text sizes the `[typography]` tokens imply, or `None` when neither was set.
    pub fn typography(&self) -> Option<Typography> {
        if !self.is_set("typography", "text-scale") && !self.is_set("typography", "base-size") {
            return None;
        }
        let base = self.number("typography", "base-size")?;
        let scale = self.number("typography", "text-scale")?;
        Some(Typography::with_base(base * scale))
    }

    /// Every resolved token, for a [`Theme`](crate::Theme) to carry (see
    /// [`Theme::token_color`](crate::Theme::token_color)).
    pub fn tokens(&self) -> Tokens {
        Tokens(
            self.entries()
                .map(|(spec, value, _)| ((spec.section.clone(), spec.key.clone()), value.clone()))
                .collect(),
        )
    }
}

/// The resolved tokens a [`Theme`](crate::Theme) carries, so a style function can read an app's
/// own colors and sizes from the theme it is given.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Tokens(BTreeMap<(String, String), Value>);

impl Tokens {
    pub fn get(&self, section: &str, key: &str) -> Option<&Value> {
        self.0.get(&(section.to_string(), key.to_string()))
    }
    pub fn color(&self, section: &str, key: &str) -> Option<Color> {
        match self.get(section, key)? {
            Value::Color(c) => Some(*c),
            _ => None,
        }
    }
    pub fn number(&self, section: &str, key: &str) -> Option<f32> {
        self.get(section, key)?
            .as_number()
            .map(|n| n as f32)
            .filter(|n| n.is_finite())
    }
    pub fn bool(&self, section: &str, key: &str) -> Option<bool> {
        match self.get(section, key)? {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }
    pub fn text(&self, section: &str, key: &str) -> Option<&str> {
        match self.get(section, key)? {
            Value::Text(t) => Some(t),
            _ => None,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// ----------------------------------------------------------------- global

static CURRENT: RwLock<Option<Arc<ThemeConfig>>> = RwLock::new(None);

/// Installs `config` as the process-wide theme config and reports, on stderr, any token hardcoded
/// in code, any warning and any unknown token, so none is silent.
pub fn install(config: ThemeConfig) {
    if !config.hardcoded.is_empty() {
        eprintln!(
            "iced-adaptive-design: this app hardcodes theme tokens {:?} (discouraged: put them in the app theme)",
            config.hardcoded
        );
    }
    for w in &config.warnings {
        eprintln!("iced-adaptive-design: theme: {w}");
    }
    *CURRENT.write().unwrap() = Some(Arc::new(config));
}

/// The installed theme config, or one with only the toolkit's declared defaults (nothing set).
pub fn current() -> Arc<ThemeConfig> {
    if let Some(c) = CURRENT.read().unwrap().as_ref() {
        return c.clone();
    }
    let c = Arc::new(ThemeConfig::builder("").files(false).build());
    *CURRENT.write().unwrap() = Some(c.clone());
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema() -> Schema {
        Schema::new()
            .color("cover", "background", "#9c4ad1", "Behind a missing cover")
            .number("cover", "size", 88.0, "Cover edge")
            .boolean("cover", "rounded", true, "Round the corners")
            .text("brand", "name", "Weave", "Shown in the header")
    }

    fn build(toolkit: &str, app: &str) -> ThemeConfig {
        ThemeConfig::builder("test")
            .schema(schema())
            .toolkit_text(toolkit)
            .app_text(app)
            .build()
    }

    #[test]
    fn an_unset_token_is_its_declared_default() {
        let c = build("", "");
        assert_eq!(c.number("cover", "size"), Some(88.0));
        assert_eq!(c.source("cover", "size"), Source::Default);
        assert!(!c.is_set("cover", "size"));
        assert_eq!(
            c.number("metrics", "radius-l"),
            Some(Metrics::default().radius_l)
        );
        assert_eq!(c.text("brand", "name"), Some("Weave"));
    }

    #[test]
    fn the_app_theme_beats_the_toolkit_theme() {
        let c = build(
            "[cover]\nsize=100\n[metrics]\nradius-l=2\n",
            "[cover]\nsize=120\n",
        );
        assert_eq!(c.number("cover", "size"), Some(120.0));
        assert_eq!(c.source("cover", "size"), Source::App);
        // Only the toolkit file set this one.
        assert_eq!(c.number("metrics", "radius-l"), Some(2.0));
        assert_eq!(c.source("metrics", "radius-l"), Source::Toolkit);
    }

    #[test]
    fn hardcoded_beats_everything_and_is_reported() {
        let c = ThemeConfig::builder("test")
            .schema(schema())
            .toolkit_text("[cover]\nsize=100\n")
            .app_text("[cover]\nsize=120\n")
            .hardcode("cover", "size", Value::Number(64.0))
            .build();
        assert_eq!(c.number("cover", "size"), Some(64.0));
        assert_eq!(c.source("cover", "size"), Source::Hardcoded);
        assert_eq!(c.hardcoded(), &[("cover".to_string(), "size".to_string())]);
    }

    #[test]
    fn hardcoding_something_undeclared_or_of_the_wrong_type_is_ignored_with_a_warning() {
        let c = ThemeConfig::builder("test")
            .schema(schema())
            .files(false)
            .hardcode("cover", "size", Value::Text("big".into()))
            .hardcode("cover", "nope", Value::Number(1.0))
            .build();
        assert_eq!(c.number("cover", "size"), Some(88.0));
        assert_eq!(c.warnings().len(), 2);
        assert!(c.hardcoded().is_empty());
    }

    #[test]
    fn invalid_values_fall_back_to_the_lower_layer() {
        let c = build(
            "[cover]\nsize=100\n",
            "[cover]\nsize=wide\nbackground=nope\n",
        );
        assert_eq!(c.number("cover", "size"), Some(100.0));
        assert_eq!(c.source("cover", "size"), Source::Toolkit);
        assert_eq!(c.source("cover", "background"), Source::Default);
        assert_eq!(c.warnings().len(), 2);
    }

    #[test]
    fn unknown_tokens_are_listed_not_used() {
        let c = build("", "[cover]\ncolour=#fff\n[elsewhere]\nx=1\n");
        assert_eq!(c.unknown().len(), 2);
        assert!(c.warnings().is_empty());
    }

    #[test]
    fn a_template_documents_every_token_and_pins_nothing() {
        let t = schema().template("Test theme");
        assert!(t.contains("[cover]") && t.contains("#size=88") && t.contains("# Cover edge"));
        assert!(t.contains("#background=#9c4ad1") && t.contains("#rounded=true"));
        // Read back, it sets nothing: every line is a comment.
        let c = build("", &t);
        assert!(c.entries().all(|(_, _, s)| s == Source::Default));
        assert!(c.warnings().is_empty() && c.unknown().is_empty());
    }

    #[test]
    fn the_toolkit_template_reads_back_as_the_defaults() {
        let t = Schema::toolkit().template("Toolkit");
        let c = ThemeConfig::builder("x")
            .files(false)
            .toolkit_text(t)
            .build();
        assert!(c.entries().all(|(_, _, s)| s == Source::Default));
        assert!(c.warnings().is_empty());
    }

    #[test]
    fn a_missing_file_is_regenerated_and_a_deleted_one_comes_back() {
        let dir = std::env::temp_dir().join(format!("iad-theme-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        // SAFETY: the test sets the variable before any thread reads it and restores nothing
        // else; the name is unique to this test.
        unsafe { std::env::set_var("XDG_CONFIG_HOME", &dir) };
        let build = || {
            ThemeConfig::builder("org.example.App")
                .schema(schema())
                .build()
        };
        let _ = build();
        let app = app_path("org.example.App").unwrap();
        let toolkit = toolkit_path().unwrap();
        assert!(app.exists() && toolkit.exists());
        assert!(std::fs::read_to_string(&app).unwrap().contains("#size=88"));
        // The user edits it, then deletes it: the app writes a fresh one.
        std::fs::write(&app, "[cover]\nsize=200\n").unwrap();
        assert_eq!(build().number("cover", "size"), Some(200.0));
        std::fs::remove_file(&app).unwrap();
        assert_eq!(build().number("cover", "size"), Some(88.0));
        assert!(app.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn metric_tokens_apply_unclamped_and_only_when_set() {
        let c = build("", "[metrics]\nradius-l=30\ncard-padding=20\n");
        let mut m = Metrics::default();
        c.apply_to_metrics(&mut m);
        assert_eq!(m.radius_l, 30.0);
        assert_eq!(m.card_padding, iced::Padding::from(20.0));
        // Untouched.
        assert_eq!(m.radius_s, Metrics::default().radius_s);
        assert_eq!(c.metric_overrides().len(), 2);
    }

    #[test]
    fn color_tokens_apply_per_scheme() {
        let c = build(
            "",
            "[colors]\ncard-bg=#ff0000\n[colors-dark]\ncard-bg=#00ff00\n",
        );
        let mut light = Palette::new(Scheme::Light);
        c.apply_to_palette(&mut light, Scheme::Light);
        assert_eq!(crate::color::to_hex(light.card_bg_color), "#ff0000");
        let mut dark = Palette::new(Scheme::Dark);
        c.apply_to_palette(&mut dark, Scheme::Dark);
        assert_eq!(crate::color::to_hex(dark.card_bg_color), "#00ff00");
    }

    #[test]
    fn typography_follows_the_scale_and_base_only_when_set() {
        assert!(build("", "").typography().is_none());
        let t = build("", "[typography]\ntext-scale=1.25\n")
            .typography()
            .unwrap();
        assert!((t.base - Typography::default().base * 1.25).abs() < 1e-4);
    }

    #[test]
    fn the_accent_token_sets_a_custom_accent() {
        let c = build("", "[colors]\naccent=#e01b24\n");
        let mut p = Palette::new(Scheme::Light);
        c.apply_to_palette(&mut p, Scheme::Light);
        assert_eq!(crate::color::to_hex(p.accent_bg_color), "#e01b24");
    }

    #[test]
    fn a_schema_declared_twice_keeps_the_latest() {
        let s = Schema::new()
            .number("a", "x", 1.0, "first")
            .number("a", "x", 2.0, "second");
        assert_eq!(s.specs().len(), 1);
        assert_eq!(s.get("a", "x").unwrap().default, Some(Value::Number(2.0)));
    }

    #[test]
    fn a_theme_carries_the_tokens_and_does_not_clamp_radii() {
        let c = build(
            "",
            "[metrics]\nradius-l=40\n[cover]\nbackground=#102030\nsize=96\n",
        );
        let theme = crate::Theme::light().with_theme_config(&c);
        // Rounder than the HIG allows: a different vibe.
        assert_eq!(theme.metrics.radius_l, 40.0);
        assert!(theme.metrics.radius_l > Metrics::MAX_RADIUS_L);
        assert_eq!(
            crate::color::to_hex(theme.token_color("cover", "background").unwrap()),
            "#102030"
        );
        assert_eq!(theme.token_number("cover", "size"), Some(96.0));
        assert_eq!(theme.token_bool("cover", "rounded"), Some(true));
        assert_eq!(theme.token_text("brand", "name"), Some("Weave"));
        // A theme without a config has no tokens and its own look.
        let plain = crate::Theme::light();
        assert!(plain.tokens.is_empty());
        assert_eq!(
            plain.metrics.radius_l,
            Metrics::default().clamped().radius_l
        );
    }

    #[test]
    fn metric_tokens_reach_the_config_the_widgets_read() {
        let c = build("", "[metrics]\ncard-padding=30\nrow-min-height=70\n");
        let cfg = crate::config::Config::default().with_theme_config(&c);
        assert_eq!(cfg.metrics().card_padding, iced::Padding::from(30.0));
        assert_eq!(cfg.metrics().row_min_height, 70.0);
        // Over the density, not instead of it: compact keeps its other values.
        let compact = crate::config::Config::default()
            .opt_out(|o| {
                o.density(crate::config::Density::Compact);
            })
            .with_theme_config(&c);
        assert_eq!(compact.metrics().row_min_height, 70.0);
        assert_eq!(compact.metrics().space_m, Metrics::compact().space_m);
    }

    #[test]
    fn the_palette_token_changes_the_semantic_layer() {
        let c = build("", "[colors]\ncard-bg=#123456\n");
        let theme = crate::Theme::light().with_theme_config(&c);
        assert_eq!(crate::color::to_hex(theme.semantic.card.bg), "#123456");
    }
}

#[cfg(test)]
mod typed_variable_tests {
    use super::*;
    fn schema() -> Schema {
        Schema::new()
            .variations(["high-contrast", "dark"])
            .variable(
                "panel",
                "gap",
                VariableType::Length {
                    min: Some(0.0),
                    max: Some(100.0),
                },
                Some(Value::Length(8.0)),
                "Gap",
            )
            .variable(
                "panel",
                "optional",
                VariableType::String {
                    values: vec!["yes".into()],
                    accepts_random: false,
                },
                None,
                "Optional value",
            )
    }
    fn build(toolkit: &str, app: &str) -> ThemeConfig {
        ThemeConfig::builder("test")
            .files(false)
            .schema(schema())
            .toolkit_text(toolkit)
            .app_text(app)
            .build()
    }
    #[test]
    fn invalid_properties_use_lower_layers_then_fallback_including_none() {
        let cfg = build(
            "[panel]\ngap=12px\noptional=yes",
            "[panel]\ngap=500px\noptional=no",
        );
        assert_eq!(cfg.get("panel", "gap"), Some(&Value::Length(12.0)));
        assert_eq!(cfg.text("panel", "optional"), Some("yes"));
        assert_eq!(cfg.warnings().len(), 2);
        let cfg = build(
            "[panel]\ngap=no\noptional=no",
            "[panel]\ngap=500px\noptional=no",
        );
        assert_eq!(cfg.get("panel", "gap"), Some(&Value::Length(8.0)));
        assert_eq!(cfg.get("panel", "optional"), None);
    }
    #[test]
    fn variations_always_override_base_and_can_be_switched_without_losing_values() {
        let cfg = build(
            "[dark:panel]\ngap=20px",
            "[panel]\ngap=10px\n[high-contrast:panel]\ngap=30px",
        );
        assert_eq!(cfg.number("panel", "gap"), Some(10.0));
        assert_eq!(
            cfg.clone().with_variation("dark").number("panel", "gap"),
            Some(20.0)
        );
        assert_eq!(
            cfg.clone()
                .with_variation("high-contrast")
                .number("panel", "gap"),
            Some(30.0)
        );
        assert_eq!(
            cfg.with_variation("light").number("panel", "gap"),
            Some(10.0)
        );
    }
    #[test]
    fn invalid_variations_fall_through_to_base_values() {
        let cfg = build(
            "",
            "[panel]\ngap=12px\noptional=yes\n[dark:panel]\ngap=no\noptional=no",
        )
        .with_variation("dark");
        assert_eq!(cfg.get("panel", "gap"), Some(&Value::Length(12.0)));
        assert_eq!(cfg.text("panel", "optional"), Some("yes"));
        assert_eq!(cfg.tokens().text("panel", "optional"), Some("yes"));
        let cfg = build("[dark:panel]\ngap=24px", "[dark:panel]\ngap=no").with_variation("dark");
        assert_eq!(cfg.get("panel", "gap"), Some(&Value::Length(24.0)));
    }
    #[test]
    fn templates_validate_and_include_variations_and_optional_fallbacks() {
        let schema = schema();
        let template = schema.template("Example");
        assert!(template.contains("[dark:panel]") && template.contains("[high-contrast:panel]"));
        assert!(template.contains("#gap=8px") && template.contains("no fallback"));
        assert!(schema.validate(&template).is_valid());
        let report =
            schema.validate("[panel]\ngap=1em\nunknown=yes\n[typo:panel]\ngap=20px\nmalformed");
        assert_eq!(report.errors.len(), 2);
        assert_eq!(report.unknown.len(), 2);
    }
    #[test]
    fn themes_automatically_cascade_light_dark_and_respect_custom_variations() {
        let cfg = build(
            "",
            "[dark:metrics]\nrow-min-height=80\n[dark:colors]\ncard-bg=#123456\n[high-contrast:metrics]\nrow-min-height=90",
        );
        let light = crate::Theme::light().with_theme_config(&cfg);
        let dark = crate::Theme::dark().with_theme_config(&cfg);
        assert_ne!(light.metrics.row_min_height, 80.0);
        assert_eq!(dark.metrics.row_min_height, 80.0);
        assert_eq!(crate::color::to_hex(dark.semantic.card.bg), "#123456");
        assert_eq!(
            crate::Theme::light()
                .with_theme_config(&cfg.with_variation("high-contrast"))
                .metrics
                .row_min_height,
            90.0
        );
    }
    #[test]
    #[should_panic(expected = "fallback")]
    fn author_errors_in_fallbacks_are_rejected() {
        Schema::new().variable(
            "x",
            "n",
            VariableType::Integer {
                min: Some(0),
                max: Some(10),
                step: 2,
                rounds: None,
            },
            Some(Value::Integer(3)),
            "",
        );
    }
}

#[cfg(test)]
mod variation_palette_tests {
    use super::*;
    #[test]
    fn explicit_dark_variation_wins_over_legacy_dark_colors() {
        let cfg = ThemeConfig::builder("test")
            .files(false)
            .app_text("[colors-dark]\ncard-bg=#ff0000\n[dark:colors]\ncard-bg=#00ff00")
            .build();
        let mut palette = Palette::new(Scheme::Dark);
        cfg.apply_to_palette(&mut palette, Scheme::Dark);
        assert_eq!(crate::color::to_hex(palette.card_bg_color), "#00ff00");
    }
}
