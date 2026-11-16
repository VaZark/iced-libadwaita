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
//! 2. **The app theme file**, `$XDG_CONFIG_HOME/<app-id>/theme.ron`. The app declares its
//!    tokens and their defaults in a [`Schema`], so the file can be deleted: the app writes a
//!    fresh one (every token commented out with its default and a description).
//! 3. **The toolkit theme file**, `$XDG_CONFIG_HOME/iced-adaptive-design/theme.ron`. The toolkit
//!    declares the tokens it understands ([`Schema::toolkit`]) with their defaults; it too is
//!    regenerated when missing.
//!
//! Below these, an unset token is the **declared default**, and the toolkit's own look is
//! whatever the [desktop config](crate::config) makes of it. A token only changes anything
//! when someone sets it: the generated files are comments, so they document and never pin.
//!
//! # Tokens
//!
//! A token is a typed value ([`Value`]: a color, a number, a gradient, a bool or text) under a section and a
//! key, in a file of [RON](https://github.com/ron-rs/ron): a map of sections to maps of keys, the same
//! shape as the desktop config. Colors are strings, lengths and numbers are numbers, and a
//! gradient is `(angle: 90, stops: [(0.0, "#fff"), (1.0, "#000")])`; see [`Document`].
//!
//! * **Toolkit tokens** (see [`Schema::toolkit`]): `"colors"` and `"light:colors"`/`"dark:colors"` (the palette:
//!   `accent`, `window-bg`, `card-bg`, …; the scheme's variant wins over the base), `"metrics"`
//!   (radii, paddings, heights, spacing) and `"typography"` (`text-scale`, `base-size`).
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
//!     .app_text(r#"{ "cover": { "size": 120 }, "metrics": { "radius-l": 20 } }"#)   // the app theme file's text
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

mod document;
mod gradient;
mod variable;
pub use document::Document;
/// The RON crate a [`Document`] holds its values in, so callers use the same version.
pub use ron;
pub use gradient::Gradient;
pub use variable::{Rounding, VariableType};

use crate::metrics::{Metrics, Typography};
use crate::palette::{AccentColor, Palette, Scheme};
use crate::shadow::{Elevation, Shadows};
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
    /// A linear gradient (`linear-gradient(90deg, #000 0%, #fff 100%)`).
    Gradient(Gradient),
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
    Gradient,
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
            Self::Gradient(_) => Kind::Gradient,
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
            Self::Gradient(g) => g.to_text(),
            Self::Number(n) => n.to_string(),
            Self::Bool(b) => b.to_string(),
            Self::Text(t) => t.clone(),
        }
    }
    /// The value as the RON a file holds (see [`VariableType::from_ron`]): lengths and
    /// percentages are plain numbers, colors are hex strings, a gradient is
    /// `{"angle": .., "stops": [[position, "color"], ..]}`.
    pub fn to_ron(&self) -> ron::Value {
        use ron::Value as R;
        let num = |n: f64| R::Number(ron::Number::from(n));
        match self {
            Self::Integer(n) => R::Number(ron::Number::from(*n)),
            Self::Decimal(n) | Self::Length(n) | Self::Percentage(n) => num(*n),
            Self::Number(n) => num(*n as f64),
            Self::Color(c) => R::String(crate::color::to_hex(*c)),
            Self::Gradient(g) => {
                let mut map = ron::Map::new();
                map.insert(R::String("angle".into()), num(g.angle as f64));
                let stops = g
                    .stops
                    .iter()
                    .map(|(p, c)| {
                        R::Seq(vec![num(*p as f64), R::String(crate::color::to_hex(*c))])
                    })
                    .collect();
                map.insert(R::String("stops".into()), R::Seq(stops));
                R::Map(map)
            }
            Self::Bool(b) => R::Bool(*b),
            Self::Text(t) => R::String(t.clone()),
        }
    }
    /// The value as a theme file holds it (RON): numbers are plain numbers (a length is in px),
    /// colors and text are strings, a gradient is its structure.
    pub fn to_ron_text(&self) -> String {
        let string = |s: &str| ron::ser::to_string(s).unwrap_or_else(|_| format!("{s:?}"));
        match self {
            Self::Integer(n) => n.to_string(),
            Self::Decimal(n) | Self::Length(n) | Self::Percentage(n) => ron_number(*n),
            Self::Number(n) => ron_number(*n as f64),
            Self::Color(c) => string(&crate::color::to_hex(*c)),
            Self::Gradient(g) => g.to_ron_text(),
            Self::Bool(b) => b.to_string(),
            Self::Text(t) => string(t),
        }
    }
}

/// A float as RON writes it: always with a decimal point or an exponent, so it reads back as a
/// float, and with no floating-point noise (`24.0`, `0.5`).
fn ron_number(n: f64) -> String {
    let mut t = format!("{n}");
    if !t.contains(['.', 'e', 'E']) {
        t.push_str(".0");
    }
    t
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
    /// The declared widget variants, `(widget, name)`.
    variants: Vec<(String, String)>,
}

/// The variants the toolkit provides, per widget: each is a named set of style tokens in the
/// section `[<widget>.<variant>]` (see [`Schema::variant`]). An app opts out of them with
/// [`Builder::default_variants`].
pub const DEFAULT_VARIANTS: &[(&str, &[&str])] = &[
    (
        "button",
        &[
            "default",
            "raised",
            "flat",
            "headerbar",
            "suggested",
            "destructive",
            "opaque",
            "pill",
            "suggested-pill",
            "link",
            "osd",
            "card",
        ],
    ),
    (
        "frame",
        &[
            "window",
            "card",
            "field",
            "well",
            "panel",
            "popover",
            "dialog",
            "tab",
            "check",
            "radio",
            "progress",
            "tooltip",
            "menu",
            "scrollthumb",
            "scrollbutton",
            "statusbar",
        ],
    ),
    ("tree", &["default"]),
    ("tree-row", &["default"]),
    ("navigation", &["default", "sm", "md", "lg"]),
    ("toolbar", &["flat", "raised", "raised-border"]),
    ("tabs", &["default", "outline", "pills", "floating"]),
];

/// Which of the toolkit's [default variants](DEFAULT_VARIANTS) an app keeps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DefaultVariants {
    /// All of them (the default).
    #[default]
    All,
    /// None: the app declares every variant it uses. A widget asked for a variant that is not
    /// declared draws its neutral base style.
    None,
    /// Only the defaults of these widgets (`"button"`, …).
    Only(&'static [&'static str]),
    /// The defaults of every widget but these.
    Without(&'static [&'static str]),
}

impl DefaultVariants {
    /// Whether the defaults of `widget` are kept.
    pub fn keeps(self, widget: &str) -> bool {
        match self {
            Self::All => true,
            Self::None => false,
            Self::Only(list) => list.contains(&widget),
            Self::Without(list) => !list.contains(&widget),
        }
    }
    fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "all" => Some(Self::All),
            "none" => Some(Self::None),
            _ => None,
        }
    }
}

/// What a variant key holds.
#[derive(Clone, Copy)]
enum KeyTy {
    Color,
    /// A length in px, up to the maximum.
    Length(f64),
    Gradient,
    /// One of these strings.
    Enum(&'static [&'static str]),
}

/// A style key a widget's variants understand. Every key is optional; an unset key is what the
/// widget's built-in look computes.
#[derive(Clone, Copy)]
struct VariantKey {
    key: &'static str,
    ty: KeyTy,
    doc: &'static str,
}

/// The values of a `border-style` key. `solid` is the toolkit's own border; the others are drawn
/// by [`Bevel`](crate::components::Bevel).
pub const BORDER_STYLES: &[&str] = &["solid", "raised", "sunken", "etched", "bump", "window"];

/// The values of a frame's `glyph` key: the mark drawn in a check or radio box.
pub const GLYPHS: &[&str] = &["tick", "cross", "dot", "square", "none"];
/// The values of `border-edges`: which edges of a bevel are drawn (a tab has no bottom edge).
pub const BORDER_EDGES: &[&str] = &["all", "no-bottom", "no-top", "no-left", "no-right"];
/// The values of `[scrollbar] track-style`.
pub const TRACK_STYLES: &[&str] = &["plain", "dither"];
/// The values of `[progress] style`.
pub const PROGRESS_STYLES: &[&str] = &["smooth", "blocks"];
/// The values of `[window-decorations] glyph-set`: the artwork of the window buttons.
pub const GLYPH_SETS: &[&str] = &["symbolic", "classic"];
/// The values of `[window-decorations] button-fill`.
pub const BUTTON_FILLS: &[&str] = &["none", "circle"];
/// The values of `[typography] disabled-style`.
pub const DISABLED_STYLES: &[&str] = &["fade", "emboss"];
/// The values of `[menu] separator`.
pub const MENU_SEPARATORS: &[&str] = &["line", "etched", "none"];

macro_rules! key {
    ($key:literal, color, $doc:literal) => {
        VariantKey {
            key: $key,
            ty: KeyTy::Color,
            doc: $doc,
        }
    };
    ($key:literal, length($max:literal), $doc:literal) => {
        VariantKey {
            key: $key,
            ty: KeyTy::Length($max),
            doc: $doc,
        }
    };
    ($key:literal, gradient, $doc:literal) => {
        VariantKey {
            key: $key,
            ty: KeyTy::Gradient,
            doc: $doc,
        }
    };
    ($key:literal, border_style, $doc:literal) => {
        VariantKey {
            key: $key,
            ty: KeyTy::Enum(BORDER_STYLES),
            doc: $doc,
        }
    };
    ($key:literal, glyph, $doc:literal) => {
        VariantKey {
            key: $key,
            ty: KeyTy::Enum(GLYPHS),
            doc: $doc,
        }
    };
    ($key:literal, edges, $doc:literal) => {
        VariantKey {
            key: $key,
            ty: KeyTy::Enum(BORDER_EDGES),
            doc: $doc,
        }
    };
}

/// The bevel keys shared by every widget that can be drawn with a two-tone frame.
macro_rules! bevel_keys {
    () => {
        [
            key!("border-style", border_style, "solid (the toolkit's border) or a drawn bevel: raised, sunken, etched, bump, window"),
            key!("border-light", color, "Bevel: the light outer edge (top and left of a raised frame)"),
            key!("border-dark", color, "Bevel: the dark outer edge (bottom and right of a raised frame)"),
            key!("border-light-inner", color, "Bevel: the light inner edge"),
            key!("border-dark-inner", color, "Bevel: the dark inner edge"),
        ]
    };
}

const BUTTON_KEYS: &[VariantKey] = &{
    let bevel = bevel_keys!();
    [
        key!("bg", color, "Background fill"),
        key!("fg", color, "Text and icon color"),
        key!("border", color, "Border color"),
        key!("border-width", length(16.0), "Border width in px"),
        key!("radius", length(1000.0), "Corner radius in px"),
        key!(
            "hover-bg",
            color,
            "Background while hovered (derived from bg when unset)"
        ),
        key!(
            "active-bg",
            color,
            "Background while pressed (derived from bg when unset)"
        ),
        key!(
            "hover-fg",
            color,
            "Text color while hovered (a menu bar's highlighted item)"
        ),
        key!("active-fg", color, "Text color while pressed"),
        key!(
            "border-default",
            color,
            "A solid 1px ring around a bevelled button, outside the bevel: the classic default-button outline"
        ),
        key!(
            "press-offset",
            length(8.0),
            "How far (px, right and down) the content moves while a bevelled button is pressed"
        ),
        key!("bg-gradient", gradient, "Background gradient (over bg)"),
        key!(
            "hover-bg-gradient",
            gradient,
            "Gradient while hovered (derived from bg-gradient when unset)"
        ),
        key!(
            "active-bg-gradient",
            gradient,
            "Gradient while pressed (derived from bg-gradient when unset)"
        ),
        bevel[0],
        bevel[1],
        bevel[2],
        bevel[3],
        bevel[4],
    ]
};

const FRAME_KEYS: &[VariantKey] = &{
    let bevel = bevel_keys!();
    [
        key!(
            "bg",
            color,
            "Fill of the framed surface (fields: the entry's background; unset: the toolkit's own)"
        ),
        key!(
            "fg",
            color,
            "Text color on the framed surface (tooltips, menus)"
        ),
        key!(
            "border",
            color,
            "Color of a solid 1px border (a tooltip's black outline)"
        ),
        key!(
            "size",
            length(64.0),
            "Size in px of a drawn mark: a check box, a radio box (unset: the classic 13 and 12)"
        ),
        key!(
            "glyph",
            glyph,
            "The mark drawn when checked: tick, cross, dot, square or none"
        ),
        key!(
            "border-edges",
            edges,
            "Which edges of the bevel are drawn: all, or all but one (a tab has no bottom edge)"
        ),
        key!(
            "border-width",
            length(16.0),
            "Frame width in px (a bevel takes 1 or 2)"
        ),
        key!(
            "radius",
            length(1000.0),
            "Corner radius in px (solid frames)"
        ),
        bevel[0],
        bevel[1],
        bevel[2],
        bevel[3],
        bevel[4],
    ]
};

fn variant_keys(widget: &str) -> &'static [VariantKey] {
    match widget {
        "button" => BUTTON_KEYS,
        "frame" => FRAME_KEYS,
        _ => &[],
    }
}

fn variant_key_type(k: &VariantKey) -> VariableType {
    match k.ty {
        KeyTy::Color => VariableType::Color,
        KeyTy::Length(max) => VariableType::Length {
            min: Some(0.0),
            max: Some(max),
        },
        KeyTy::Gradient => VariableType::Gradient,
        KeyTy::Enum(values) => VariableType::String {
            values: values.iter().map(|v| v.to_string()).collect(),
            accepts_random: false,
        },
    }
}

/// The surfaces a `[gradients]` token can paint, and what each is.
pub const GRADIENT_SURFACES: &[(&str, &str)] = &[
    ("window", "The window background"),
    ("view", "Content views and text areas"),
    ("headerbar", "Header bars and toolbars"),
    (
        "titlebar",
        "The title bar of a client-side decorated window (over headerbar)",
    ),
    ("sidebar", "The sidebar pane"),
    ("card", "Cards and boxed lists"),
    ("popover", "Popovers and menus"),
    ("dialog", "Dialogs"),
    ("progress", "The filled part of a progress bar"),
];

/// The enum values of `[window-decorations] titlebar-align`.
pub const TITLEBAR_ALIGNS: &[&str] = &["start", "center", "end"];
/// The enum values of `[window-decorations] titlebar-font-weight`.
pub const TITLEBAR_WEIGHTS: &[&str] = &["normal", "bold"];
/// The enum values of `[metrics] focus-style`.
pub const FOCUS_STYLES: &[&str] = &["ring", "dotted", "none"];

fn enum_type(values: &[&str]) -> VariableType {
    VariableType::String {
        values: values.iter().map(|v| v.to_string()).collect(),
        accepts_random: false,
    }
}

/// Declares the tokens that let a theme take on another design language: gradients, window
/// frame, font, focus.
fn declare_design_tokens(mut s: Schema) -> Schema {
    for (key, doc) in GRADIENT_SURFACES {
        s = s.variable(
            "gradients",
            key,
            VariableType::Gradient,
            None,
            &format!("{doc}: a gradient over the flat color (unset: flat)"),
        );
    }
    let len = |max: f64| VariableType::Length {
        min: Some(0.0),
        max: Some(max),
    };
    s = s.variable(
        "window-decorations",
        "frame-border-width",
        len(32.0),
        None,
        "Width of the frame around a client-side decorated window, in px",
    );
    s = s.variable(
        "window-decorations",
        "frame-border",
        VariableType::Color,
        None,
        "Color of the window frame (a solid frame)",
    );
    s = s.variable(
        "window-decorations",
        "frame-border-style",
        enum_type(BORDER_STYLES),
        None,
        "solid, or a bevel: raised, sunken, etched, bump, window (colors from [frame.window])",
    );
    s = s.variable(
        "window-decorations",
        "titlebar-align",
        enum_type(TITLEBAR_ALIGNS),
        None,
        "Where the window title sits: start, center or end",
    );
    s = s.variable(
        "window-decorations",
        "titlebar-font-weight",
        enum_type(TITLEBAR_WEIGHTS),
        None,
        "Weight of the window title: normal or bold",
    );
    s = s.variable("window-decorations", "flat-headerbars", VariableType::Bool, None, "Header bars with no background of their own (libadwaita's flat bars); false for a title bar with a color or gradient. Over the desktop's [appearance] flat-headerbars");
    s = s.variable(
        "window-decorations",
        "titlebar-icon",
        VariableType::Bool,
        None,
        "Show the application icon at the start of the title bar",
    );
    for action in ["minimize", "maximize", "restore", "close"] {
        for (part, doc) in [
            ("bg", "Background"),
            ("fg", "Foreground"),
            ("border", "Border color"),
        ] {
            s = s.variable(
                "window-decorations", &format!("{action}-{part}"), VariableType::Color, None,
                &format!("{doc} of the {action} window control (unset: button style; restore falls back to maximize)"),
            );
        }
    }
    s = s.variable(
        "window-decorations",
        "button-style",
        VariableType::String {
            values: vec![],
            accepts_random: true,
        },
        None,
        "The button variant of the window buttons (default: flat)",
    );
    s = s.variable(
        "typography",
        "font-family",
        VariableType::Font,
        None,
        "The text font family (unset: the system default)",
    );
    s = s.variable(
        "typography",
        "monospace-font-family",
        VariableType::Font,
        None,
        "The monospace font family",
    );
    for (section, key, default, max) in [
        ("tree", "level-offset", 20.0, 256.0),
        ("tree", "handle-padding", 8.0, 256.0),
        ("tree", "expander-icon-inset", 2.0, 64.0),
        ("tree", "grip-width", 32.0, 256.0),
        ("tree", "drag-threshold", 4.0, 256.0),
        ("tree", "checkbox-size-extra", 2.0, 64.0),
        ("tree", "checkbox-glyph-inset", 4.0, 64.0),
        ("tree", "checkbox-radius", 4.0, 64.0),
        ("tree", "checkbox-border-width", 1.5, 32.0),
        ("tree", "connector-width", 1.0, 32.0),
        ("tree", "drop-border-width", 1.5, 32.0),
        ("tree", "drop-line-width", 2.0, 32.0),
        ("tree", "drop-line-radius", 1.0, 64.0),
        ("tree", "drag-border-width", 2.0, 32.0),
        ("tree", "drag-clip-margin", 24.0, 256.0),
        ("tree", "drag-shadow-offset", 2.0, 256.0),
        ("tree-row", "separator-width", 1.0, 32.0),
        ("navigation", "indent", 14.0, 256.0),
        ("navigation", "initial-sidebar-width", 240.0, 4096.0),
        ("navigation", "initial-drawer-width", 280.0, 4096.0),
        ("navigation", "separator-width", 1.0, 32.0),
    ] {
        s = s.variable(
            section,
            key,
            len(max),
            Some(Value::Length(default)),
            &format!("{section} {key}, in px"),
        );
    }
    s = s.variable(
        "tree",
        "row-height",
        VariableType::Length {
            min: Some(1.0),
            max: Some(512.0),
        },
        Some(Value::Length(32.0)),
        "Height of a tree row, in px (must be positive)",
    );
    for (section, key, doc) in [
        (
            "tree",
            "icon-size",
            "Tree icon size (unset: icon-size metric)",
        ),
        (
            "tree",
            "icon-padding",
            "Tree icon horizontal padding (unset: space-xs metric)",
        ),
        (
            "tree",
            "label-padding",
            "Tree label horizontal padding (unset: space-s metric)",
        ),
        (
            "tree",
            "checkbox-padding",
            "Tree checkbox horizontal padding (unset: space-xs metric)",
        ),
        (
            "tree",
            "row-radius",
            "Tree row radius (unset: radius-s metric)",
        ),
        (
            "tree-row",
            "indent",
            "Child indentation (unset: expander-indent metric)",
        ),
        (
            "tree-row",
            "handle-width",
            "Expander handle width (unset: icon-size plus 8px)",
        ),
        (
            "tree-row",
            "handle-padding-left",
            "Expander left padding (unset: row-padding left)",
        ),
        (
            "tree-row",
            "glyph-size",
            "Expander glyph size (unset: icon-size minus 2px, at least zero)",
        ),
    ] {
        s = s.variable(section, key, len(256.0), None, doc);
    }
    for (section, key, default, min, max, doc) in [
        (
            "tree",
            "drag-scale",
            1.01,
            0.001,
            4.0,
            "Scale of the dragged tree row",
        ),
        (
            "tree-row",
            "children-opacity",
            0.03,
            0.0,
            1.0,
            "Opacity of the child group background",
        ),
        (
            "navigation",
            "selection-opacity",
            0.10,
            0.0,
            1.0,
            "Opacity of the navigation selection background",
        ),
    ] {
        s = s.variable(
            section,
            key,
            VariableType::Decimal {
                no_decimals: 3,
                min: Some(min),
                max: Some(max),
                step: None,
                rounds: None,
            },
            Some(Value::Decimal(default)),
            doc,
        );
    }
    s = s.variable(
        "theme-toggle",
        "icon-size",
        VariableType::Length {
            min: Some(1.0),
            max: Some(128.0),
        },
        None,
        "Appearance icon size, in px (unset: inherited metrics icon-size)",
    );
    for (section, key, max) in [
        ("tabs", "close-icon-size", 128.0),
        ("tabs", "indicator-height", 32.0),
        ("tabs", "border-width", 32.0),
        ("tabs", "separator-width", 32.0),
        ("tabs", "focus-inset", 64.0),
        ("tabs", "panel-padding", 512.0),
        ("toolbar", "shadow-height", 128.0),
        ("toolbar", "separator-width", 32.0),
        ("color-picker", "width", 4096.0),
        ("color-input", "popover-width", 4096.0),
        ("number-input", "width", 4096.0),
    ] {
        s = s.variable(
            section,
            key,
            len(max),
            None,
            &format!("{section} {key}, in px (unset: widget default)"),
        );
    }
    s = s.variable(
        "toolbar",
        "shadow-opacity",
        VariableType::Decimal {
            no_decimals: 3,
            min: Some(0.0),
            max: Some(1.0),
            step: None,
            rounds: None,
        },
        None,
        "Toolbar shadow opacity (unset: widget default)",
    );
    for key in ["row-height", "row-padding", "gap", "inset"] {
        s = s.variable(
            "navigation",
            key,
            len(512.0),
            None,
            "Navigation row geometry in px (unset: density metrics)",
        );
    }
    for widget in ["tree", "tree-row", "navigation", "tabs", "toolbar"] {
        s = s.variable(widget, "breakpoint-rules", VariableType::String {
            values: vec![], accepts_random: true,
        }, None, "Comma-separated minimum-width class:variant rules, e.g. xs:compact,md:wide; variants must be declared for this widget (unset: no automatic selection)");
    }
    for key in ["sm", "md", "lg", "xl", "xxl"] {
        s = s.variable(
            "breakpoints",
            key,
            VariableType::Length { min: Some(1.0), max: Some(16384.0) },
            None,
            "Minimum width of this breakpoint class, in px (unset: desktop scale); the combined scale must be strictly increasing",
        );
    }
    // Pixel-exact classic looks.
    s = s.variable(
        "progress",
        "style",
        enum_type(PROGRESS_STYLES),
        None,
        "smooth (a filled bar) or blocks (a row of separate blocks, the classic progress bar)",
    );
    s = s.variable(
        "progress",
        "block-width",
        len(64.0),
        None,
        "Width of one block of a blocks progress bar, in px",
    );
    s = s.variable(
        "progress",
        "block-gap",
        len(32.0),
        None,
        "Gap between blocks, in px",
    );
    s = s.variable(
        "progress",
        "block-inset",
        len(16.0),
        None,
        "Space between the trough's frame and the blocks, in px",
    );
    s = s.variable(
        "scrollbar",
        "arrows",
        VariableType::Bool,
        None,
        "Arrow buttons at both ends of a scrollbar (drawn by ClassicScroll)",
    );
    s = s.variable(
        "scrollbar",
        "track-style",
        enum_type(TRACK_STYLES),
        None,
        "The track: plain, or dither (a 1px checkerboard of track-light and track-dark)",
    );
    s = s.variable(
        "scrollbar",
        "track-light",
        VariableType::Color,
        None,
        "First color of a dithered track",
    );
    s = s.variable(
        "scrollbar",
        "track-dark",
        VariableType::Color,
        None,
        "Second color of a dithered track",
    );
    s = s.variable(
        "scrollbar",
        "width",
        len(32.0),
        None,
        "Width of a drawn scrollbar, in px (also the arrow buttons' size)",
    );
    for (key, default, description) in [
        (
            "popover-width",
            260.0,
            "Preferred width of menu popovers, in logical px",
        ),
        (
            "popover-max-height",
            360.0,
            "Maximum height of menu popovers, in logical px",
        ),
    ] {
        s = s.variable(
            "menu",
            key,
            VariableType::Length {
                min: Some(1.0),
                max: Some(4096.0),
            },
            Some(Value::Length(default)),
            description,
        );
    }
    s = s.variable(
        "menu",
        "item-height",
        len(64.0),
        None,
        "Height of a menu bar or popup menu item, in px",
    );
    s = s.variable(
        "menu",
        "highlight-bg",
        VariableType::Color,
        None,
        "Background of the highlighted menu item",
    );
    s = s.variable(
        "menu",
        "highlight-fg",
        VariableType::Color,
        None,
        "Text color of the highlighted menu item",
    );
    s = s.variable(
        "menu",
        "separator",
        enum_type(MENU_SEPARATORS),
        None,
        "A menu separator: line, etched (a dark and a light line) or none",
    );
    s = s.variable(
        "tooltip",
        "bg",
        VariableType::Color,
        None,
        "Background of a tooltip (unset: the on-screen-display look)",
    );
    s = s.variable(
        "tooltip",
        "fg",
        VariableType::Color,
        None,
        "Text color of a tooltip",
    );
    s = s.variable(
        "tabs",
        "height",
        len(64.0),
        None,
        "Height of an unselected tab of a classic tab control, in px (unset: the button height)",
    );
    s = s.variable("tabs", "raise", len(16.0), None, "How much taller (px) the selected tab of a classic tab control is, and how far it overlaps the page's frame");
    s = s.variable(
        "typography",
        "disabled-style",
        enum_type(DISABLED_STYLES),
        None,
        "Disabled text: fade, or emboss (grey over a white 1px offset copy)",
    );
    s = s.variable(
        "window-decorations",
        "glyph-set",
        enum_type(GLYPH_SETS),
        None,
        "The artwork of the window buttons: symbolic icons, or classic pixel glyphs",
    );
    s = s.variable("window-decorations", "close-gap", len(32.0), None, "The gap in px before the close button (over button-gap): classic title bars keep minimize and maximize together and the close button apart");
    s = s.variable(
        "window-decorations",
        "button-fill",
        enum_type(BUTTON_FILLS),
        None,
        "The window buttons at rest: `none` is the bare glyph (a fill shows only under the pointer), `circle` a filled disc behind it",
    );
    s = s.variable("window-decorations", "button-corner-clearance", len(32.0), None, "The least distance in px between a window button's outer corner and the rounded window corner (default: half the space above the button); the side inset grows to keep it, whatever the window radius");
    s = s.variable("window-decorations", "button-separation", len(64.0), None, "Extra space in px between the window buttons and the title bar's own actions, so the two groups read as separate");
    s = s.variable("window-decorations", "button-inset", len(32.0), None, "Extra space in px between the outermost window button and the title bar's edge, over the title bar padding (libadwaita: the 5 px of padding around each circular button, so the buttons keep clear of the rounded corner)");
    s = s.variable(
        "window-decorations",
        "button-width",
        len(64.0),
        None,
        "Width of a window button in px (unset: button-size)",
    );
    s = s.variable(
        "window-decorations",
        "button-height",
        len(64.0),
        None,
        "Height of a window button in px (unset: button-size)",
    );
    s = s.variable(
        "colors",
        "focus-ring",
        VariableType::Color,
        None,
        "The focus indicator color (unset: the accent at half strength)",
    );
    s = s.variable(
        "metrics",
        "focus-style",
        enum_type(FOCUS_STYLES),
        None,
        "How focus is drawn: ring, dotted (a 1px dotted rectangle) or none",
    );
    s
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
    /// The typed value a file's RON gives `[section] key`, or what was wrong with it.
    fn value_from_ron(&self, section: &str, key: &str, v: &ron::Value) -> Result<Value, String> {
        let spec = self
            .get(section, key)
            .ok_or_else(|| "not a declared token".to_string())?;
        if key == "breakpoint-rules" {
            let ron::Value::String(raw) = v else {
                return Err("expected a string of breakpoint rules, like \"xs:compact,md:default\"".into());
            };
            crate::breakpoints::Rules::parse(raw, |name| {
                self.variants
                    .iter()
                    .any(|(widget, variant)| widget == section && variant == name)
            })
            .map_err(|e| e.to_string())?;
        }
        spec.ty.from_ron(v)
    }

    /// Validate a theme file's RON against exactly this schema, without reading or writing files.
    pub fn validate(&self, text: &str) -> ValidationReport {
        let mut report = ValidationReport::default();
        let doc = match Document::parse(text) {
            Ok(doc) => doc,
            Err(error) => {
                report.errors.push(error);
                return report;
            }
        };
        for (section, key, v) in doc.entries() {
            let base = if let Some((variation, base)) = section.split_once(':') {
                if !self.variation_keys().any(|k| k == variation) {
                    report.unknown.push((section.into(), key.into()));
                    continue;
                }
                base
            } else {
                section
            };
            if self.get(base, key).is_none() {
                report.unknown.push((section.into(), key.into()));
            } else if let Err(why) = self.value_from_ron(base, key, v) {
                report.errors.push(format!("[{section}] {key}: {why}"));
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
        let ty =
            VariableType::untyped(default.kind()).expect("typed variables use Schema::variable");
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
    /// Declares the **variant** `name` of `widget`: a named set of style tokens in the section
    /// `[<widget>.<name>]` that a widget selects with `.variant(name)`. Declare a new name to add
    /// a variant, or a name the toolkit already provides to replace it (the tokens given become
    /// its declared defaults; a theme file can still change them).
    ///
    /// For a widget the toolkit knows (`"button"`: `bg`, `fg`, `border`, `border-width`, `radius`,
    /// `hover-bg`, `active-bg`) every key is declared, typed, and unset keys keep the built-in look.
    /// Tree, tree-row, navigation, tabs and toolbar variants inherit the types of their
    /// base-section geometry tokens, but not their defaults: omitted overrides stay unset.
    /// For any other widget only the keys given are declared, typed by their value.
    ///
    /// # Panics
    /// If `widget` or `name` is empty or contains anything but ASCII letters, digits, `-` and `_`.
    pub fn variant(mut self, widget: &str, name: &str, tokens: &[(&str, Value)]) -> Self {
        let ok = |s: &str| {
            !s.is_empty()
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        };
        assert!(
            ok(widget) && ok(name),
            "invalid variant name {widget}.{name}"
        );
        let section = format!("{widget}.{name}");
        let known = variant_keys(widget);
        let geometry = matches!(
            widget,
            "tree" | "tree-row" | "navigation" | "tabs" | "toolbar"
        );
        let mut base = if geometry {
            let mut base = declare_design_tokens(Self::new());
            for spec in self.specs.iter().filter(|s| s.section == widget) {
                base = base.variable(
                    widget,
                    &spec.key,
                    spec.ty.clone(),
                    spec.default.clone(),
                    &spec.doc,
                );
            }
            base.specs
                .into_iter()
                .filter(|s| s.section == widget)
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        base.retain(|s| s.key != "breakpoint-rules");
        for spec in &base {
            self = self.variable(&section, &spec.key, spec.ty.clone(), None, &spec.doc);
        }
        for k in known {
            self = self.variable(&section, k.key, variant_key_type(k), None, k.doc);
        }
        for (key, value) in tokens {
            assert!(
                !geometry || *key != "breakpoint-rules",
                "breakpoint-rules is a base-only token"
            );
            if let Some(spec) = base.iter().find(|s| s.key == *key) {
                self = self.variable(
                    &section,
                    key,
                    spec.ty.clone(),
                    Some(value.clone()),
                    &spec.doc,
                );
                continue;
            }
            let (ty, doc) = match known.iter().find(|k| k.key == *key) {
                Some(k) => (variant_key_type(k), k.doc),
                None => (
                    match value.kind() {
                        Kind::Length => VariableType::Length { min: None, max: None },
                        Kind::Percentage => VariableType::Percentage {
                            min: None,
                            max: None,
                            rounds: None,
                        },
                        kind => VariableType::untyped(kind).expect(
                            "variant tokens are colors, lengths, percentages, numbers, bools or text",
                        ),
                    },
                    "",
                ),
            };
            self = self.variable(&section, key, ty, Some(value.clone()), doc);
        }
        let entry = (widget.to_string(), name.to_string());
        if !self.variants.contains(&entry) {
            self.variants.push(entry);
        }
        self
    }
    /// The declared variants, `(widget, name)`.
    pub fn variants_declared(&self) -> &[(String, String)] {
        &self.variants
    }
    /// This schema without the toolkit's default variants that `keep` does not keep.
    fn without_default_variants(mut self, keep: DefaultVariants) -> Self {
        for (widget, names) in DEFAULT_VARIANTS {
            if keep.keeps(widget) {
                continue;
            }
            for name in *names {
                let section = format!("{widget}.{name}");
                self.specs.retain(|s| s.section != section);
                self.variants.retain(|(w, n)| !(w == widget && n == name));
            }
        }
        self
    }
    /// This schema with `other`'s tokens added (`other` wins a clash).
    pub fn with(mut self, other: &Schema) -> Self {
        self = self.variations(other.variations.clone());
        for entry in &other.variants {
            if !self.variants.contains(entry) {
                self.variants.push(entry.clone());
            }
        }
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
        // The accent drives a family of colors, so it is not a plain palette field.
        s = s.push(
            "colors",
            "accent",
            Value::Color(base.accent_bg_color),
            "The accent color (derives its text and standalone variants)",
        );
        for (key, doc, get, _) in COLORS {
            s = s.push("colors", key, Value::Color(get(&base)), doc);
        }
        s = s.boolean(
            "colors",
            "auto-contrast",
            false,
            "Pick light or dark text on colored fills by contrast, instead of the theme's -fg colors",
        );
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
        s = declare_design_tokens(s);
        for (widget, names) in DEFAULT_VARIANTS {
            for name in *names {
                s = s.variant(widget, name, &[]);
            }
        }
        s = s.variable(
            "metrics",
            "circular-button-size",
            VariableType::Length {
                min: Some(8.0),
                max: Some(128.0),
            },
            None,
            "Diameter of a circular button (unset: the button height)",
        );
        s = s.variable(
            "theme",
            "default-variants",
            VariableType::String {
                values: ["all", "none"].map(String::from).to_vec(),
                accepts_random: false,
            },
            Some(Value::Text("all".into())),
            "Keep (all) or drop (none) the toolkit's default widget variants",
        );
        s = s.text("theme", "name", "", "A display name for the theme");
        s = s.text("theme", "key", "", "A stable identifier for the theme");
        s = s.variable(
            "theme",
            "mode",
            VariableType::String {
                values: ["inherit", "override", "optout"].map(String::from).to_vec(),
                accepts_random: false,
            },
            Some(Value::Text("inherit".into())),
            "How the app relates to the desktop's look: inherit, override or optout",
        );
        s = s.text(
            "window-decorations",
            "button-layout",
            "appmenu:close",
            "Window buttons and their side: `start:end`, e.g. `:minimize,maximize,close`",
        );
        s = s.variable(
            "window-decorations",
            "button-size",
            VariableType::Length {
                min: Some(8.0),
                max: Some(64.0),
            },
            Some(Value::Length(24.0)),
            "Window button edge in px",
        );
        s = s.variable(
            "window-decorations",
            "button-gap",
            VariableType::Length {
                min: Some(0.0),
                max: Some(32.0),
            },
            Some(Value::Length(4.0)),
            "Space between window buttons in px",
        );
        s = s.variable(
            "window-decorations",
            "button-icon-size",
            VariableType::Length {
                min: Some(8.0),
                max: Some(48.0),
            },
            Some(Value::Length(16.0)),
            "Window button icon size in px",
        );
        s = s.variable(
            "window-decorations",
            "button-radius",
            VariableType::Percentage {
                min: Some(0.0),
                max: Some(50.0),
                rounds: None,
            },
            Some(Value::Percentage(50.0)),
            "Window button corner radius as a share of its size (0 square, 50 circle)",
        );
        for (key, doc) in [
            ("titlebar-double-click", "Double-clicking the title bar"),
            ("titlebar-middle-click", "Middle-clicking the title bar"),
            ("titlebar-right-click", "Right-clicking the title bar"),
        ] {
            s = s.variable(
                "window-decorations",
                key,
                VariableType::String {
                    values: ["toggle-maximize", "menu", "minimize", "none"]
                        .map(String::from)
                        .to_vec(),
                    accepts_random: false,
                },
                Some(Value::Text(
                    if key.ends_with("double-click") {
                        "toggle-maximize"
                    } else if key.ends_with("right-click") {
                        "menu"
                    } else {
                        "none"
                    }
                    .into(),
                )),
                doc,
            );
        }
        let sh = Shadows::bare();
        for (key, doc, e) in [
            ("card", "Cards and boxed lists", &sh.card),
            ("popover", "Popovers and menus", &sh.popover),
            ("dialog", "Floating dialogs", &sh.dialog),
            ("osd", "On-screen-display surfaces (toasts)", &sh.osd),
        ] {
            s = s.push(
                "shadows",
                key,
                Value::Text(e.to_css()),
                &format!("{doc}: CSS box-shadow list"),
            );
        }
        let ratio = Typography::default();
        for (key, doc, _) in TYPE_RATIOS {
            let value = match *key {
                "title-1" => ratio.title_1,
                "title-2" => ratio.title_2,
                "title-3" => ratio.title_3,
                "title-4" => ratio.title_4,
                "heading" => ratio.heading,
                "caption-heading" => ratio.caption_heading,
                "caption" => ratio.caption,
                "small" => ratio.small,
                _ => ratio.monospace,
            } / ratio.base;
            s = s.push("typography", key, Value::Number(value), doc);
        }
        s.push(
            "typography",
            "base-size",
            Value::Number(t.base),
            "The body text size in px, before the scale",
        )
    }

    /// The text of a file for this schema: every token commented out with its default and a
    /// description, so the file documents without pinning anything. Deleting a layer's file and
    /// asking for it again gives this. It is valid RON: uncomment a `"key": value,` line to set it.
    pub fn template(&self, title: &str) -> String {
        let mut out = format!(
            "// {title}\n// Generated from the declared defaults. Delete this file to get a fresh one.\n// Uncomment a line to change it; commented lines are the defaults and change nothing.\n{{\n"
        );
        let line = |s: &Spec| {
            format!(
                "        // {}\n        // Expects {}.\n        // \"{}\": {},\n",
                s.doc.replace('\n', "\n        // "),
                s.ty.expectation(),
                s.key,
                s.default
                    .as_ref()
                    .map(Value::to_ron_text)
                    .unwrap_or_else(|| "/* unset (no fallback) */".into())
            )
        };
        let mut sections: Vec<&str> = Vec::new();
        for s in &self.specs {
            if !sections.contains(&s.section.as_str()) {
                sections.push(&s.section);
            }
        }
        for section in &sections {
            out.push_str(&format!("    \"{section}\": {{\n"));
            for s in self.specs.iter().filter(|s| s.section == *section) {
                out.push_str(&line(s));
            }
            out.push_str("    },\n");
        }
        for variation in self.variation_keys() {
            for section in &sections_for_schema(self) {
                out.push_str(&format!(
                    "    // Overrides \"{section}\" while {variation} is active.\n    \"{variation}:{section}\": {{}},\n"
                ));
            }
        }
        out.push_str("}\n");
        out
    }
}
/// A one-entry RON document for tests: `ron_one("tabs", "radius", "4")`. `value` is RON text.
#[cfg(test)]
pub(crate) fn ron_one(section: &str, key: &str, value: &str) -> String {
    format!("{{ {section:?}: {{ {key:?}: {value} }} }}")
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

type TypeSet = fn(&mut Typography, f32);

/// The text-size ratio tokens (multiples of the base size): key, description, write.
const TYPE_RATIOS: &[(&str, &str, TypeSet)] = &[
    ("title-1", "Title 1 size, relative to the base", |t, v| {
        t.title_1 = v
    }),
    ("title-2", "Title 2 size, relative to the base", |t, v| {
        t.title_2 = v
    }),
    ("title-3", "Title 3 size, relative to the base", |t, v| {
        t.title_3 = v
    }),
    ("title-4", "Title 4 size, relative to the base", |t, v| {
        t.title_4 = v
    }),
    ("heading", "Heading size, relative to the base", |t, v| {
        t.heading = v
    }),
    (
        "caption-heading",
        "Caption heading size, relative to the base",
        |t, v| t.caption_heading = v,
    ),
    ("caption", "Caption size, relative to the base", |t, v| {
        t.caption = v
    }),
    (
        "monospace",
        "Monospace size, relative to the base",
        |t, v| t.monospace = v,
    ),
    (
        "small",
        "Labels, chips, tags and badges text size, relative to the base",
        |t, v| t.small = v,
    ),
];

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
    "primary", "Primary actions (suggested buttons); independent of the accent", primary_bg_color;
    "primary-fg", "Text on primary fills", primary_fg_color;
    "primary-text", "Primary color as text or an icon", primary_color;
    "success", "Success color", success_bg_color;
    "warning", "Warning color", warning_bg_color;
    "destructive", "Destructive action color", destructive_bg_color;
    "error", "Error color", error_bg_color;
    "accent-fg", "Text on accent fills", accent_fg_color;
    "accent-text", "Accent as text or an icon on a neutral surface", accent_color;
    "destructive-fg", "Text on destructive fills", destructive_fg_color;
    "destructive-text", "Destructive color as text", destructive_color;
    "success-fg", "Text on success fills", success_fg_color;
    "success-text", "Success color as text", success_color;
    "warning-fg", "Text on warning fills", warning_fg_color;
    "warning-text", "Warning color as text", warning_color;
    "error-fg", "Text on error fills", error_fg_color;
    "error-text", "Error color as text", error_color;
    "headerbar-border", "Header bar border", headerbar_border_color;
    "headerbar-shade", "Header bar bottom shade", headerbar_shade_color;
    "headerbar-darker-shade", "Header bar darker shade", headerbar_darker_shade_color;
    "card-shade", "Card outline shade", card_shade_color;
    "popover-shade", "Popover outline shade", popover_shade_color;
    "sidebar-border", "Sidebar border", sidebar_border_color;
    "secondary-sidebar-bg", "The secondary sidebar", secondary_sidebar_bg_color;
    "secondary-sidebar-fg", "Text on the secondary sidebar", secondary_sidebar_fg_color;
    "thumbnail-bg", "Thumbnails", thumbnail_bg_color;
    "thumbnail-fg", "Text on thumbnails", thumbnail_fg_color;
    "overview-bg", "Overview surfaces", overview_bg_color;
    "overview-fg", "Text on overview surfaces", overview_fg_color;
    "active-toggle-bg", "The active segment of a toggle group", active_toggle_bg_color;
    "active-toggle-fg", "Text on the active toggle", active_toggle_fg_color;
    "shade", "Generic shade (dividers, outlines)", shade_color;
    "scrollbar-outline", "Scrollbar outline", scrollbar_outline_color;
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
    "gap", "Gap between the parts of a composed widget", |m| m.gap, |m, v| m.gap = v;
    "inset", "Padding around a composed widget's panel", |m| m.inset, |m, v| m.inset = v;
    "code-padding", "Padding of a code block", |m| m.code_padding, |m, v| m.code_padding = v;
    "inline-padding-x", "Horizontal padding of inline code", |m| m.inline_padding_x, |m, v| m.inline_padding_x = v;
    "empty-padding-y", "Vertical padding of an empty state", |m| m.empty_padding_y, |m, v| m.empty_padding_y = v;
    "small-button-padding-y", "Compact button vertical padding", |m| m.small_button_padding.top, |m, v| { m.small_button_padding.top = v; m.small_button_padding.bottom = v };
    "small-button-padding-x", "Compact button horizontal padding", |m| m.small_button_padding.left, |m, v| { m.small_button_padding.left = v; m.small_button_padding.right = v };
    "space-xxs", "Tiniest gap", |m| m.space_xxs, |m, v| m.space_xxs = v;
    "space-xxl", "Hugest gap", |m| m.space_xxl, |m, v| m.space_xxl = v;
    "radius-pill", "Pill buttons and circular widgets", |m| m.radius_pill, |m, v| m.radius_pill = v;
    "pill-padding-y", "Pill button vertical padding", |m| m.pill_padding.top, |m, v| { m.pill_padding.top = v; m.pill_padding.bottom = v };
    "pill-padding-x", "Pill button horizontal padding", |m| m.pill_padding.left, |m, v| { m.pill_padding.left = v; m.pill_padding.right = v };
    "focus-ring-width", "Focus ring width", |m| m.focus_ring_width, |m, v| m.focus_ring_width = v;
    "switch-size", "Switch height", |m| m.switch_size, |m, v| m.switch_size = v;
    "checkbox-size", "Checkbox size", |m| m.checkbox_size, |m, v| m.checkbox_size = v;
    "scrollbar-slider-width", "Visible scrollbar slider width", |m| m.scrollbar_slider_width, |m, v| m.scrollbar_slider_width = v;
    "scrollbar-margin", "Gap between scrollbar slider and edge", |m| m.scrollbar_margin, |m, v| m.scrollbar_margin = v;
    "scrollbar-gutter", "Room between content and a scrollbar beside it", |m| m.scrollbar_gutter, |m, v| m.scrollbar_gutter = v;
    "progress-height", "Progress bar thickness", |m| m.progress_height, |m, v| m.progress_height = v;
    "list-spacing", "Space between list rows", |m| m.list_spacing, |m, v| m.list_spacing = v;
    "headerbar-padding-y", "Header bar vertical padding", |m| m.headerbar_padding.top, |m, v| { m.headerbar_padding.top = v; m.headerbar_padding.bottom = v };
    "headerbar-padding-x", "Header bar horizontal padding", |m| m.headerbar_padding.left, |m, v| { m.headerbar_padding.left = v; m.headerbar_padding.right = v };
    "toolbar-spacing", "Space between toolbar items", |m| m.toolbar_spacing, |m, v| m.toolbar_spacing = v;
    "dialog-width", "Default dialog width", |m| m.dialog_width, |m, v| m.dialog_width = v;
    "sidebar-padding", "Padding around the sidebar list", |m| m.sidebar_padding, |m, v| m.sidebar_padding = v;
    "sidebar-spacing", "Gap between sidebar rows", |m| m.sidebar_spacing, |m, v| m.sidebar_spacing = v;
    "sidebar-row-padding", "Horizontal padding inside a sidebar row", |m| m.sidebar_row_padding, |m, v| m.sidebar_row_padding = v;
}

// ------------------------------------------------------------ the layers

/// Where a token's value came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    /// Never set: the declared default (and, for a toolkit token, whatever the desktop config
    /// makes of it).
    Default,
    /// The bundled base or toolkit theme file found by [`theme_path`].
    Theme,
    /// The app theme file.
    App,
    /// Set in code. Discouraged.
    Hardcoded,
}

impl Source {
    pub fn label(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Theme => "theme file",
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

/// The directories theme files are installed in, most specific first: `$XDG_DATA_HOME/iad/themes`
/// (default `~/.local/share/iad/themes`), then `iad/themes` in each of `$XDG_DATA_DIRS`
/// (default `/usr/local/share:/usr/share`). A desktop or a distribution installs its themes in
/// the latter; the user's own go in the former.
pub fn theme_dirs() -> Vec<PathBuf> {
    let home = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")));
    let dirs = std::env::var("XDG_DATA_DIRS")
        .ok()
        .filter(|d| !d.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    home.into_iter()
        .chain(
            dirs.split(':')
                .filter(|d| !d.is_empty())
                .map(PathBuf::from)
                .filter(|p| p.is_absolute()),
        )
        .map(|d| d.join("iad/themes"))
        .collect()
}

/// Which theme file applies: `IAD_THEME=<path>` if it names a readable file (`IAD_THEME=none`
/// skips external theme discovery), else the first `default.ron` in [`theme_dirs`].
/// This selects only the external layer: a bundled [`Builder::base_theme`] still applies.
/// Without either layer, the toolkit uses its bare defaults.
pub fn theme_path() -> Option<PathBuf> {
    pick_theme(std::env::var("IAD_THEME").ok().as_deref(), &theme_dirs())
}

// DECISION (user, 2026-11): `IAD_THEME=none` means no theme at all, like HTML without CSS: bare
// neutral presentation, no discovered theme. It never disables application behavior or events.
// Refinement: it suppresses discovery only; an explicit `Builder::base_theme` still applies.
// Revisit if a "no base either" switch is needed.
fn pick_theme(env: Option<&str>, dirs: &[PathBuf]) -> Option<PathBuf> {
    match env.map(str::trim).filter(|v| !v.is_empty()) {
        Some("none") => return None,
        Some(path) => {
            let path = PathBuf::from(path);
            if path.is_file() {
                return Some(path);
            }
            eprintln!(
                "iced-adaptive-design: IAD_THEME={} is not a readable file; looking further",
                path.display()
            );
        }
        None => {}
    }
    dirs.iter()
        .map(|d| d.join("default.ron"))
        .find(|p| p.is_file())
}

/// Where the theme file of the app `app_id` is.
pub fn app_path(app_id: &str) -> Option<PathBuf> {
    config_home().map(|h| h.join(app_id).join("theme.ron"))
}

/// How an app relates to the look the desktop provides.
///
/// * **`Inherit`** (default): the app supplies no look. It gets the installed theme file (or the
///   bare look) and follows the desktop config and the portal's light/dark.
/// * **`OverrideValue`**: inherit everything, but the app sets some tokens in its theme file or
///   in code. Allowed, **discouraged** (it can fight the user's theme) and reported on stderr at
///   [`install`], listing each token.
/// * **`OptOut`**: the app uses only its own look and never consults the desktop: no theme file
///   lookup, no `desktop.ron`, no portal scheme. For apps with their own identity (a music
///   player). It bundles its look in its own theme text or hardcoded tokens.
///
/// Declared with [`Builder::inheritance`] or `[theme] mode=inherit|override|optout` in the app
/// theme file (code wins). When undeclared it is inferred: tokens set by the app mean
/// `OverrideValue`, none mean `Inherit`; `OptOut` is never inferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Inheritance {
    #[default]
    Inherit,
    OverrideValue,
    OptOut,
}

impl Inheritance {
    fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "inherit" => Some(Self::Inherit),
            "override" => Some(Self::OverrideValue),
            "optout" => Some(Self::OptOut),
            _ => None,
        }
    }
}

/// Builds a [`ThemeConfig`]. See the [module](self).
pub struct Builder {
    app_id: String,
    schema: Schema,
    base_theme: Option<String>,
    toolkit_text: Option<String>,
    app_text: Option<String>,
    hardcoded: Vec<(String, String, Value)>,
    files: bool,
    regenerate: bool,
    variation: Option<String>,
    inheritance: Option<Inheritance>,
    default_variants: Option<DefaultVariants>,
}

impl Builder {
    fn new(app_id: &str) -> Self {
        Self {
            app_id: app_id.to_string(),
            schema: Schema::new(),
            base_theme: None,
            toolkit_text: None,
            app_text: None,
            hardcoded: Vec::new(),
            files: true,
            regenerate: true,
            variation: None,
            inheritance: None,
            default_variants: None,
        }
    }
    /// Declares how the app relates to the desktop's look (see [`Inheritance`]). Over the
    /// `[theme] mode` of the app file.
    pub fn inheritance(mut self, inheritance: Inheritance) -> Self {
        self.inheritance = Some(inheritance);
        self
    }
    /// Which of the toolkit's [default widget variants](DEFAULT_VARIANTS) the app keeps. Over the
    /// `[theme] default-variants` token of the app file. The default is all of them.
    pub fn default_variants(mut self, keep: DefaultVariants) -> Self {
        self.default_variants = Some(keep);
        self
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
    /// A bundled base theme, validated below the selected toolkit and app theme layers.
    /// It remains active when `IAD_THEME` is unset or `none`, discovery finds no file,
    /// files are disabled, or automatic desktop inheritance is opted out.
    /// Prefer `include_str!` with a bundled `.ron` file (recommended: `libadwaita.ron`).
    pub fn base_theme(mut self, text: impl Into<String>) -> Self {
        self.base_theme = Some(text.into());
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
        let app_text = self.read_layer(
            &self.app_text,
            app_path(&self.app_id),
            &self.schema,
            &format!("{} theme: this app's look", self.app_id),
        );
        let app_doc = Document::parse(&app_text).unwrap_or_default();
        let keep = self.default_variants.unwrap_or_else(|| {
            app_doc
                .text("theme", "default-variants")
                .and_then(DefaultVariants::parse)
                .unwrap_or_default()
        });
        let toolkit = Schema::toolkit().without_default_variants(keep);
        let schema = toolkit.clone().with(&self.schema);
        let declared = self
            .inheritance
            .or_else(|| app_doc.text("theme", "mode").and_then(Inheritance::parse));
        let opt_out = declared == Some(Inheritance::OptOut);
        let toolkit_text = match (&self.toolkit_text, self.files && !opt_out) {
            (Some(t), _) => t.clone(),
            (None, true) => theme_path()
                .map(|p| {
                    std::fs::read_to_string(&p).unwrap_or_else(|e| {
                        eprintln!("iced-adaptive-design: theme {}: {e}", p.display());
                        String::new()
                    })
                })
                .unwrap_or_default(),
            (None, false) => String::new(),
        };
        let mut cfg = ThemeConfig {
            values: BTreeMap::new(),
            schema,
            warnings: Vec::new(),
            unknown: Vec::new(),
            hardcoded: Vec::new(),
            variation: None,
            inheritance: Inheritance::Inherit,
            inherited: BTreeMap::new(),
            denied: Vec::new(),
        };
        for s in cfg.schema.specs().to_vec() {
            cfg.values
                .insert((s.section, s.key), (s.default, Source::Default));
        }
        // Lowest priority first, so a later layer overwrites an earlier one.
        for (text, source) in [
            (self.base_theme.as_deref().unwrap_or(""), Source::Theme),
            (toolkit_text.as_str(), Source::Theme),
            (app_text.as_str(), Source::App),
        ] {
            if source == Source::App {
                cfg.inherited = cfg.values.clone();
            }
            match Document::parse(text) {
                Ok(doc) => {
                    for (section, key, v) in doc.entries() {
                        cfg.set_from_ron(section, key, v, source);
                    }
                }
                Err(error) => cfg.warnings.push(format!("{}: {error}", source.label())),
            }
        }
        for (section, key, value) in self.hardcoded {
            let checked = cfg
                .schema
                .get(&section, &key)
                .and_then(|spec| spec.ty.from_value(&value));
            match checked {
                Some(value) => {
                    cfg.values
                        .insert((section.clone(), key.clone()), (Some(value), Source::Hardcoded));
                    cfg.hardcoded.push((section, key));
                }
                None => cfg.warnings.push(format!(
                    "[{section}] {key}: invalid or undeclared hardcoded value; ignored"
                )),
            }
        }
        if let Some(variation) = self.variation {
            cfg = cfg.with_variation(&variation);
        }
        cfg.inheritance = declared.unwrap_or_else(|| {
            if cfg.overrides().is_empty() {
                Inheritance::Inherit
            } else {
                Inheritance::OverrideValue
            }
        });

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
    inheritance: Inheritance,
    /// The values before the app's layers (theme file and defaults only).
    inherited: BTreeMap<(String, String), (Option<Value>, Source)>,
    /// App tokens the desktop policy refused (`[policy] allow-app-overrides=false`).
    denied: Vec<(String, String)>,
}

impl ThemeConfig {
    /// Starts a config for the app `app_id` (the directory of its theme file).
    pub fn builder(app_id: &str) -> Builder {
        Builder::new(app_id)
    }

    fn set_from_ron(&mut self, section: &str, key: &str, v: &ron::Value, source: Source) {
        let base = if let Some((variation, base)) = section.split_once(':') {
            if !self.schema.variation_keys().any(|key| key == variation) {
                self.unknown.push((section.into(), key.into()));
                return;
            }
            base
        } else {
            section
        };
        if self.schema.get(base, key).is_none() {
            self.unknown.push((section.into(), key.into()));
            return;
        }
        match self.schema.value_from_ron(base, key, v) {
            Ok(value) => {
                self.values
                    .insert((section.into(), key.into()), (Some(value), source));
            }
            Err(why) => self.warnings.push(format!(
                "[{section}] {key}: {why}; next valid lower layer or declared fallback used"
            )),
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
    /// How the app relates to the desktop's look.
    pub fn inheritance(&self) -> Inheritance {
        self.inheritance
    }
    /// The tokens the app itself sets (in its theme file or in code), over what the desktop
    /// provides. `[theme]` metadata does not count.
    pub fn overrides(&self) -> Vec<(String, String)> {
        self.values
            .iter()
            .filter(|((section, _), (value, source))| {
                value.is_some()
                    && matches!(source, Source::App | Source::Hardcoded)
                    && section.split_once(':').map_or(section.as_str(), |(_, b)| b) != "theme"
            })
            .map(|((section, key), _)| (section.clone(), key.clone()))
            .collect()
    }
    /// This config as the desktop policy allows it. With `allow` (the desktop's
    /// `[policy] allow-app-overrides`) it is unchanged; without, an `OverrideValue` app's tokens
    /// (file and hardcoded) are dropped back to what the desktop provides and listed in
    /// [`denied`](Self::denied). `OptOut` and `Inherit` apps are not overriding anything.
    #[must_use]
    pub fn for_policy(&self, allow: bool) -> Self {
        if allow || self.inheritance != Inheritance::OverrideValue {
            return self.clone();
        }
        let mut c = self.clone();
        c.denied = self.overrides();
        c.values = self.inherited.clone();
        c.hardcoded.clear();
        c.inheritance = Inheritance::Inherit;
        c
    }
    /// The app tokens the desktop policy refused.
    pub fn denied(&self) -> &[(String, String)] {
        &self.denied
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

    /// The `[shadows]` tokens that were set, applied to `shadows`. A malformed list is ignored.
    pub fn apply_to_shadows(&self, shadows: &mut Shadows) {
        for (key, slot) in [
            ("card", &mut shadows.card),
            ("popover", &mut shadows.popover),
            ("dialog", &mut shadows.dialog),
            ("osd", &mut shadows.osd),
        ] {
            if self.is_set("shadows", key)
                && let Some(e) = self.text("shadows", key).and_then(Elevation::parse)
            {
                *slot = e;
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
    /// `[colors]`, then `[<variation>:colors]` over it (`[dark:colors]` on a dark scheme).
    pub fn apply_to_palette(&self, palette: &mut Palette, scheme: Scheme) {
        if self.variation.is_none() {
            return self
                .clone()
                .with_variation(if scheme.is_dark() { "dark" } else { "light" })
                .apply_to_palette(palette, scheme);
        }
        let variation_section = format!("{}:colors", self.variation.as_deref().unwrap());
        let sections = ["colors", variation_section.as_str()];
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

    /// The text sizes the `[typography]` tokens imply, or `None` when none was set. The sizes
    /// are `base-size * text-scale * <ratio>`; unset ratios keep [`Typography`]'s.
    pub fn typography(&self) -> Option<Typography> {
        let any = TYPE_RATIOS
            .iter()
            .map(|(k, ..)| *k)
            .chain(["text-scale", "base-size"])
            .any(|k| self.is_set("typography", k));
        if !any {
            return None;
        }
        let base = self.number("typography", "base-size")?;
        let scale = self.number("typography", "text-scale")?;
        let mut t = Typography::with_base(base * scale);
        for (key, _, set) in TYPE_RATIOS {
            if self.is_set("typography", key)
                && let Some(r) = self.number("typography", key)
            {
                set(&mut t, base * scale * r);
            }
        }
        Some(t)
    }

    /// Every resolved token, for a [`Theme`](crate::Theme) to carry (see
    /// [`Theme::token_color`](crate::Theme::token_color)).
    pub fn tokens(&self) -> Tokens {
        Tokens {
            values: self
                .entries()
                .map(|(spec, value, _)| ((spec.section.clone(), spec.key.clone()), value.clone()))
                .collect(),
            variants: Some(self.schema.variants.iter().cloned().collect()),
        }
    }
}

/// A widget variant resolved from a theme: a named set of typed style tokens (see
/// [`Schema::variant`]). Every accessor is `None` for a key the theme leaves unset, and the widget
/// then keeps its built-in look for that key.
#[derive(Debug, Clone)]
pub struct ResolvedVariant<'a> {
    exists: bool,
    section: String,
    tokens: &'a Tokens,
}

impl ResolvedVariant<'_> {
    /// Whether the variant is declared (by the toolkit's defaults or by the app). A widget asked
    /// for one that is not draws its neutral base style.
    pub fn exists(&self) -> bool {
        self.exists
    }
    /// A color token (`bg`, `fg`, `border`, …).
    pub fn color(&self, key: &str) -> Option<Color> {
        self.tokens.color(&self.section, key)
    }
    /// A length token in px (`border-width`, `radius`, …).
    pub fn length(&self, key: &str) -> Option<f32> {
        self.tokens.number(&self.section, key)
    }
    /// A bool token.
    pub fn bool(&self, key: &str) -> Option<bool> {
        self.tokens.bool(&self.section, key)
    }
    /// A text or enum-string token.
    pub fn text(&self, key: &str) -> Option<&str> {
        self.tokens.text(&self.section, key)
    }
    /// A gradient token (`bg-gradient`, …).
    pub fn gradient(&self, key: &str) -> Option<&Gradient> {
        self.tokens.gradient(&self.section, key)
    }
}

/// The resolved tokens a [`Theme`](crate::Theme) carries, so a style function can read an app's
/// own colors and sizes from the theme it is given.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Tokens {
    values: BTreeMap<(String, String), Value>,
    /// The declared variants, `(widget, name)`; `None` when no theme config was applied (the
    /// toolkit's default variants are then taken to exist).
    variants: Option<std::collections::BTreeSet<(String, String)>>,
}

impl Tokens {
    pub fn get(&self, section: &str, key: &str) -> Option<&Value> {
        self.values.get(&(section.to_string(), key.to_string()))
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
    /// A gradient token (`[gradients] titlebar`, `bg-gradient` of a variant, …).
    pub fn gradient(&self, section: &str, key: &str) -> Option<&Gradient> {
        match self.get(section, key)? {
            Value::Gradient(g) => Some(g),
            _ => None,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    /// A numeric variant override, falling back to the base widget token. Undeclared
    /// variants are neutral; an unset base token remains `None` for metric inheritance.
    pub fn variant_number(&self, widget: &str, name: &str, key: &str) -> Option<f32> {
        let variant = self.variant(widget, name);
        variant
            .exists()
            .then(|| variant.length(key))
            .flatten()
            .or_else(|| self.number(widget, key))
    }

    /// The variant `name` of `widget`: whether it is declared, and its tokens.
    pub fn variant(&self, widget: &str, name: &str) -> ResolvedVariant<'_> {
        let exists = match &self.variants {
            Some(set) => set.contains(&(widget.to_string(), name.to_string())),
            None => DEFAULT_VARIANTS
                .iter()
                .any(|(w, names)| *w == widget && names.contains(&name)),
        };
        ResolvedVariant {
            exists,
            section: format!("{widget}.{name}"),
            tokens: self,
        }
    }
}

// ----------------------------------------------------------------- global

static CURRENT: RwLock<Option<Arc<ThemeConfig>>> = RwLock::new(None);

impl ThemeConfig {
    /// The selected external theme ([`theme_path`]) over the declared defaults, with no
    /// bundled base or app file. For a bundled base use [`Builder::base_theme`] or
    /// [`Config::load_with_base_theme`](crate::config::Config::load_with_base_theme).
    pub fn discover() -> Self {
        Self::builder("").app_text("").build()
    }
}

/// Installs `config` as the process-wide theme config and reports, on stderr, any token hardcoded
/// in code, any warning and any unknown token, so none is silent.
pub fn install(config: ThemeConfig) {
    if !config.hardcoded.is_empty() {
        eprintln!(
            "iced-adaptive-design: this app hardcodes theme tokens {:?} (discouraged: put them in the app theme)",
            config.hardcoded
        );
    }
    if config.inheritance == Inheritance::OverrideValue {
        eprintln!(
            "iced-adaptive-design: this app overrides theme tokens {:?} (override-value; discouraged: it can fight the user's theme)",
            config.overrides()
        );
    }
    if !config.denied.is_empty() {
        eprintln!(
            "iced-adaptive-design: the desktop policy forbids app overrides; ignored theme tokens {:?}",
            config.denied
        );
    }
    for w in &config.warnings {
        eprintln!("iced-adaptive-design: theme: {w}");
    }
    *CURRENT.write().unwrap() = Some(Arc::new(config));
}

/// The built-in value of the `[colors]` token `key` for `scheme` (the bare palette, before any
/// theme file), or `None` when `key` is not a palette token. The declared defaults of
/// [`Schema::toolkit`] are scheme-neutral, so this is how to ask for a dark default.
pub fn palette_default(scheme: Scheme, key: &str) -> Option<Color> {
    let p = Palette::new(scheme);
    if key == "accent" {
        return Some(p.accent_bg_color);
    }
    COLORS
        .iter()
        .find(|(k, ..)| *k == key)
        .map(|(_, _, get, _)| get(&p))
}

/// The diameter of a circular button in px (`[metrics] circular-button-size`), or `None` when the
/// theme leaves it unset (the button height applies).
pub fn circular_button_size() -> Option<f32> {
    current().number("metrics", "circular-button-size")
}

/// A font family name as a `&'static str`, which `iced::font::Family::Name` needs: each distinct
/// name is leaked once.
fn intern(name: &str) -> &'static str {
    static NAMES: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());
    let mut names = NAMES.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(n) = names.iter().find(|n| **n == name) {
        return n;
    }
    let leaked: &'static str = Box::leak(name.to_string().into_boxed_str());
    names.push(leaked);
    leaked
}

/// The font for a family name (see [`font`]).
pub(crate) fn font_named(name: &str) -> iced::Font {
    iced::Font {
        family: iced::font::Family::Name(intern(name)),
        ..iced::Font::DEFAULT
    }
}

/// The theme's text font (`[typography] font-family`), `None` when unset (the system default).
pub fn font() -> Option<iced::Font> {
    let cfg = current();
    let name = cfg.text("typography", "font-family")?;
    Some(iced::Font {
        family: iced::font::Family::Name(intern(name)),
        ..iced::Font::DEFAULT
    })
}

/// The theme's monospace font (`[typography] monospace-font-family`), `None` when unset.
pub fn monospace_font() -> Option<iced::Font> {
    let cfg = current();
    let name = cfg.text("typography", "monospace-font-family")?;
    Some(iced::Font {
        family: iced::font::Family::Name(intern(name)),
        ..iced::Font::DEFAULT
    })
}

/// The font an application sets as its default so every text, including text the toolkit does
/// not style, uses the theme's family: `iced::application(..).default_font(theme_config::default_font())`.
/// The system default when the theme sets none.
pub fn default_font() -> iced::Font {
    font().unwrap_or(iced::Font::DEFAULT)
}

/// The frame around a client-side decorated window, from `[window-decorations] frame-border-*`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WindowFrame {
    /// `frame-border-width` in px (always above 0 here).
    pub width: f32,
    /// A drawn bevel (`frame-border-style` raised, window, …, colored by `[frame.window]`), or
    /// `None` for a solid frame in `frame-border`.
    pub bevel: Option<crate::primitives::bevel::BevelStyle>,
}

/// The window frame the theme asks for, `None` when `frame-border-width` is unset or 0 (the
/// toolkit's own 1px outline).
pub fn window_frame() -> Option<WindowFrame> {
    let cfg = current();
    let width = cfg
        .number("window-decorations", "frame-border-width")
        .filter(|w| *w > 0.0)?;
    let bevel = cfg
        .text("window-decorations", "frame-border-style")
        .and_then(crate::primitives::bevel::BevelStyle::parse);
    Some(WindowFrame { width, bevel })
}

/// Where the window title sits (`[window-decorations] titlebar-align`): `"start"`, `"center"` (default) or
/// `"end"`.
pub fn titlebar_align() -> &'static str {
    match current().text("window-decorations", "titlebar-align") {
        Some("start") => "start",
        Some("end") => "end",
        _ => "center",
    }
}

/// Whether the window title is bold (`[window-decorations] titlebar-font-weight`): unset is the toolkit's
/// own bold heading.
pub fn titlebar_bold() -> bool {
    current().text("window-decorations", "titlebar-font-weight") != Some("normal")
}

/// Whether the title bar shows the application icon at its start (`[window-decorations] titlebar-icon`).
pub fn titlebar_icon() -> bool {
    current().bool("window-decorations", "titlebar-icon").unwrap_or(false)
}

/// The button variant of the window buttons (`[window-decorations] button-style`), `None` when unset (the
/// toolkit's own round flat buttons).
pub fn window_button_style() -> Option<String> {
    current()
        .text("window-decorations", "button-style")
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// Optional per-action window control color. Restore inherits maximize per color;
/// absent colors leave the selected button style unchanged.
pub(crate) fn window_control_color(tokens: &Tokens, action: &str, part: &str) -> Option<Color> {
    tokens
        .color("window-decorations", &format!("{action}-{part}"))
        .or_else(|| {
            (action == "restore")
                .then(|| tokens.color("window-decorations", &format!("maximize-{part}")))
                .flatten()
        })
}

/// The window button edge in px (`[window-decorations] button-size`).
pub fn window_button_size() -> f32 {
    current().number("window-decorations", "button-size").unwrap_or(24.0)
}

/// The window button size as `(width, height)` in px: `[window-decorations] button-width` and
/// `button-height`, each defaulting to [`window_button_size`].
pub fn window_button_dims() -> (f32, f32) {
    let size = window_button_size();
    let cfg = current();
    (
        cfg.number("window-decorations", "button-width").unwrap_or(size),
        cfg.number("window-decorations", "button-height").unwrap_or(size),
    )
}

/// Whether the window buttons are the classic pixel glyphs (`[window-decorations] glyph-set=classic`).
pub fn classic_window_glyphs() -> bool {
    current().text("window-decorations", "glyph-set") == Some("classic")
}

/// The gap before the close button, when the theme sets one (`[window-decorations] close-gap`).
pub fn window_close_gap() -> Option<f32> {
    current().number("window-decorations", "close-gap")
}

/// Whether the window buttons are drawn as a filled disc at rest (`[window-decorations] button-fill=circle`);
/// by default they are the bare glyph, with a fill only under the pointer.
pub fn window_buttons_filled() -> bool {
    current().text("window-decorations", "button-fill") == Some("circle")
}

/// The least distance in px between a window button's corner and the rounded window corner
/// (`[window-decorations] button-corner-clearance`), when the theme sets one.
pub fn window_button_corner_clearance() -> Option<f32> {
    current().number("window-decorations", "button-corner-clearance")
}

/// The extra space in px between the window buttons and the title bar's own actions
/// (`[window-decorations] button-separation`). Default 0.
pub fn window_button_separation() -> f32 {
    current().number("window-decorations", "button-separation").unwrap_or(0.0)
}

/// The extra space in px between the outermost window button and the title bar's edge
/// (`[window-decorations] button-inset`), over the title bar padding. Default 0.
pub fn window_button_inset() -> f32 {
    current().number("window-decorations", "button-inset").unwrap_or(0.0)
}

/// The space between window buttons in px (`[window-decorations] button-gap`).
pub fn window_button_gap() -> f32 {
    current().number("window-decorations", "button-gap").unwrap_or(4.0)
}

/// The window button icon size in px (`[window-decorations] button-icon-size`).
pub fn window_button_icon_size() -> f32 {
    current()
        .number("window-decorations", "button-icon-size")
        .unwrap_or(16.0)
}

/// The window button corner radius in px: `[window-decorations] button-radius` percent of the button size.
pub fn window_button_radius() -> f32 {
    window_button_size() * current().number("window-decorations", "button-radius").unwrap_or(50.0) / 100.0
}

/// What a title-bar click does (`[window-decorations] titlebar-<click>-click`): `toggle-maximize`, `menu`,
/// `minimize` or `none`. `click` is `double`, `middle` or `right`.
pub fn titlebar_action(click: &str) -> String {
    current()
        .text("window-decorations", &format!("titlebar-{click}-click"))
        .unwrap_or("none")
        .to_string()
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
            r##"{
    "cover": {
        "size": 100,
    },
    "metrics": {
        "radius-l": 2,
    },
}
"##,
            r##"{
    "cover": {
        "size": 120,
    },
}
"##,
        );
        assert_eq!(c.number("cover", "size"), Some(120.0));
        assert_eq!(c.source("cover", "size"), Source::App);
        // Only the toolkit file set this one.
        assert_eq!(c.number("metrics", "radius-l"), Some(2.0));
        assert_eq!(c.source("metrics", "radius-l"), Source::Theme);
    }

    #[test]
    fn hardcoded_beats_everything_and_is_reported() {
        let c = ThemeConfig::builder("test")
            .schema(schema())
            .toolkit_text(r##"{
    "cover": {
        "size": 100,
    },
}
"##)
            .app_text(r##"{
    "cover": {
        "size": 120,
    },
}
"##)
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
            r##"{
    "cover": {
        "size": 100,
    },
}
"##,
            r##"{
    "cover": {
        "size": "wide",
        "background": "nope",
    },
}
"##,
        );
        assert_eq!(c.number("cover", "size"), Some(100.0));
        assert_eq!(c.source("cover", "size"), Source::Theme);
        assert_eq!(c.source("cover", "background"), Source::Default);
        assert_eq!(c.warnings().len(), 2);
    }

    #[test]
    fn unknown_tokens_are_listed_not_used() {
        let c = build("", r##"{
    "cover": {
        "colour": "#fff",
    },
    "elsewhere": {
        "x": 1,
    },
}
"##);
        assert_eq!(c.unknown().len(), 2);
        assert!(c.warnings().is_empty());
    }

    #[test]
    fn a_template_documents_every_token_and_pins_nothing() {
        let t = schema().template("Test theme");
        assert!(t.contains("\"cover\": {") && t.contains("// \"size\": 88.0,") && t.contains("// Cover edge"));
        assert!(t.contains("// \"background\": \"#9c4ad1\",") && t.contains("// \"rounded\": true,"));
        // Read back, it sets nothing: every line is a comment.
        let c = build("", &t);
        assert!(c.entries().all(|(_, _, s)| s == Source::Default));
        assert!(c.warnings().is_empty() && c.unknown().is_empty());
    }

    #[test]
    fn window_control_colors_are_optional_validated_and_restore_inherits_per_color() {
        let empty = ThemeConfig::builder("x").files(false).build();
        assert_eq!(window_control_color(&empty.tokens(), "close", "bg"), None);
        for action in ["minimize", "maximize", "restore", "close"] {
            for part in ["bg", "fg", "border"] {
                let key = format!("{action}-{part}");
                let valid = Schema::toolkit().validate(&ron_one("window-decorations", &key, "\"#123456\""));
                assert!(valid.errors.is_empty() && valid.unknown.is_empty());
                let invalid =
                    Schema::toolkit().validate(&ron_one("window-decorations", &key, "\"not-a-color\""));
                assert!(!invalid.errors.is_empty());
            }
        }
        let cfg = ThemeConfig::builder("x").files(false)
            .toolkit_text(r##"{
    "window-decorations": {
        "maximize-bg": "#123456",
        "maximize-fg": "#ffffff",
        "restore-fg": "#000000",
        "close-bg": "#ff0000",
    },
}
"##)
            .app_text(r##"{
    "window-decorations": {
        "close-bg": "invalid",
    },
}
"##)
            .build();
        let tokens = cfg.tokens();
        assert_eq!(
            window_control_color(&tokens, "restore", "bg"),
            cfg.color("window-decorations", "maximize-bg")
        );
        assert_eq!(
            window_control_color(&tokens, "restore", "fg"),
            cfg.color("window-decorations", "restore-fg")
        );
        assert_eq!(window_control_color(&tokens, "minimize", "bg"), None);
        assert_eq!(
            window_control_color(&tokens, "close", "bg"),
            cfg.color("window-decorations", "close-bg")
        );
        assert_eq!(cfg.warnings().len(), 1);
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
        assert!(app.exists());
        assert!(std::fs::read_to_string(&app).unwrap().contains("// \"size\": 88.0,"));
        // The user edits it, then deletes it: the app writes a fresh one.
        std::fs::write(&app, r##"{
    "cover": {
        "size": 200,
    },
}
"##).unwrap();
        assert_eq!(build().number("cover", "size"), Some(200.0));
        std::fs::remove_file(&app).unwrap();
        assert_eq!(build().number("cover", "size"), Some(88.0));
        assert!(app.exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_theme_lookup_prefers_the_env_then_the_data_dirs_then_nothing() {
        let root = std::env::temp_dir().join(format!("iad-theme-lookup-{}", std::process::id()));
        let (a, b) = (root.join("a"), root.join("b"));
        for d in [&a, &b] {
            std::fs::create_dir_all(d).unwrap();
        }
        let file = root.join("custom.conf");
        std::fs::write(&file, "").unwrap();
        std::fs::write(b.join("default.ron"), "").unwrap();
        let dirs = [a.clone(), b.clone()];
        assert_eq!(pick_theme(None, &dirs), Some(b.join("default.ron")));
        std::fs::write(a.join("default.ron"), "").unwrap();
        assert_eq!(pick_theme(None, &dirs), Some(a.join("default.ron")));
        assert_eq!(pick_theme(file.to_str(), &dirs), Some(file.clone()));
        assert_eq!(pick_theme(Some("none"), &dirs), None);
        // An unreadable env path falls through to the data dirs.
        assert_eq!(
            pick_theme(Some("/nonexistent/x.conf"), &dirs),
            Some(a.join("default.ron"))
        );
        assert_eq!(pick_theme(None, &[root.join("empty")]), None);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn metric_tokens_apply_unclamped_and_only_when_set() {
        let c = build("", r##"{
    "metrics": {
        "radius-l": 30,
        "card-padding": 20,
    },
}
"##);
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
            r##"{
    "colors": {
        "card-bg": "#ff0000",
    },
    "dark:colors": {
        "card-bg": "#00ff00",
    },
}
"##,
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
        let t = build("", r##"{
    "typography": {
        "text-scale": 1.25,
    },
}
"##)
            .typography()
            .unwrap();
        assert!((t.base - Typography::default().base * 1.25).abs() < 1e-4);
    }

    #[test]
    fn the_accent_token_sets_a_custom_accent() {
        let c = build("", r##"{
    "colors": {
        "accent": "#e01b24",
    },
}
"##);
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
            r##"{
    "metrics": {
        "radius-l": 40,
    },
    "cover": {
        "background": "#102030",
        "size": 96,
    },
}
"##,
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
        let c = build("", r##"{
    "metrics": {
        "card-padding": 30,
        "row-min-height": 70,
    },
}
"##);
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
        let c = build("", r##"{
    "colors": {
        "card-bg": "#123456",
    },
}
"##);
        let theme = crate::Theme::light().with_theme_config(&c);
        assert_eq!(crate::color::to_hex(theme.semantic.card.bg), "#123456");
    }
}

#[cfg(test)]
mod tree_navigation_schema_tests {
    use super::*;

    #[test]
    fn breakpoint_thresholds_are_optional_validated_and_documented() {
        let schema = Schema::toolkit();
        let template = schema.template("Breakpoint thresholds");
        assert!(template.contains("\"breakpoints\": {"));
        assert!(schema.validate(&template).is_valid());
        assert!(schema.validate(&template).unknown.is_empty());
        for key in ["sm", "md", "lg", "xl", "xxl"] {
            let spec = schema.get("breakpoints", key).unwrap();
            assert_eq!(spec.default, None);
            assert_eq!(
                spec.ty,
                VariableType::Length {
                    min: Some(1.0),
                    max: Some(16384.0)
                }
            );
            assert!(template.contains(&format!("// \"{key}\":")));
            let valid = ron_one("breakpoints", key, "800");
            let report = schema.validate(&valid);
            assert!(report.is_valid());
            assert!(report.unknown.is_empty());
            assert_eq!(build("", "").number("breakpoints", key), None);
            assert_eq!(
                build("", &valid).tokens().number("breakpoints", key),
                Some(800.0)
            );
            // A length is a plain number of px: a string, zero, a negative, NaN and too big are not.
            for raw in ["0", "-1", "NaN", "inf", "16385", "\"800px\""] {
                let invalid = ron_one("breakpoints", key, raw);
                let report = schema.validate(&invalid);
                assert!(!report.is_valid(), "{invalid}");
                assert!(report.unknown.is_empty());
                let cfg = build("", &invalid);
                assert_eq!(cfg.number("breakpoints", key), None);
                assert!(!cfg.warnings().is_empty());
                assert!(cfg.unknown().is_empty());
                assert_eq!(
                    build(&valid, &invalid).number("breakpoints", key),
                    Some(800.0)
                );
            }
            for raw in ["1", "16384"] {
                assert!(
                    schema
                        .validate(&ron_one("breakpoints", key, raw))
                        .is_valid()
                );
            }
        }
    }

    #[test]
    fn breakpoint_rules_validate_and_cascade_without_variant_recursion() {
        let schema = Schema::toolkit();
        for widget in ["tree", "tree-row", "navigation", "tabs", "toolbar"] {
            let name = if widget == "toolbar" {
                "flat"
            } else {
                "default"
            };
            assert_eq!(
                schema.get(widget, "breakpoint-rules").unwrap().default,
                None
            );
            assert!(
                schema
                    .get(&format!("{widget}.{name}"), "breakpoint-rules")
                    .is_none()
            );
            let valid = ron_one(widget, "breakpoint-rules", &format!("\"xs:{name}\""));
            assert!(schema.validate(&valid).is_valid());
            for rules in [
                "md",
                "bad:default",
                "md:missing",
                "xs:bad name",
                "xs:flat,xs:flat",
            ] {
                let invalid = ron_one(widget, "breakpoint-rules", &format!("{rules:?}"));
                assert!(!schema.validate(&invalid).is_valid(), "{invalid}");
                let cfg = build(&valid, &invalid);
                assert_eq!(
                    cfg.text(widget, "breakpoint-rules"),
                    Some(format!("xs:{name}").as_str())
                );
                assert!(!cfg.warnings().is_empty());
                assert_eq!(
                    crate::breakpoints::variant_for_tokens(
                        &cfg.tokens(),
                        widget,
                        &crate::breakpoints::Breakpoints::default(),
                        800.0,
                        "neutral"
                    ),
                    name
                );
                assert_eq!(build("", &invalid).text(widget, "breakpoint-rules"), None);
            }
        }
        for (widget, names) in [
            ("toolbar", vec!["flat", "raised", "raised-border"]),
            ("tabs", vec!["default", "outline", "pills", "floating"]),
        ] {
            for name in names {
                assert!(
                    schema
                        .variants_declared()
                        .contains(&(widget.into(), name.into()))
                );
            }
        }
        for key in ["row-height", "row-padding", "gap", "inset"] {
            assert_eq!(schema.get("navigation", key).unwrap().default, None);
            assert!(schema.get("navigation.sm", key).is_some());
        }
        let custom = Schema::new().variant("tabs", "compact", &[]);
        let cfg = ThemeConfig::builder("breakpoint-tests").files(false).schema(custom)
            .toolkit_text(r##"{
    "tabs": {
        "breakpoint-rules": "xs:default",
    },
    "dark:tabs": {
        "breakpoint-rules": "xs:compact",
    },
}
"##)
            .app_text(r##"{
    "light:tabs": {
        "breakpoint-rules": "md:outline",
    },
    "dark:tabs": {
        "breakpoint-rules": "xs:missing",
    },
}
"##)
            .build();
        assert_eq!(
            cfg.clone()
                .with_variation("light")
                .text("tabs", "breakpoint-rules"),
            Some("md:outline")
        );
        assert_eq!(
            cfg.with_variation("dark").text("tabs", "breakpoint-rules"),
            Some("xs:compact")
        );
        let opted_out = ThemeConfig::builder("breakpoint-tests")
            .files(false)
            .default_variants(DefaultVariants::None)
            .toolkit_text("")
            .app_text(r##"{
    "tabs": {
        "breakpoint-rules": "xs:outline",
    },
}
"##)
            .build();
        assert_eq!(opted_out.text("tabs", "breakpoint-rules"), None);
    }

    #[test]
    fn geometry_variants_are_optional_typed_and_cascade_over_base() {
        for (widget, key) in [
            ("tree", "row-height"),
            ("tree-row", "indent"),
            ("navigation", "indent"),
            ("tabs", "close-icon-size"),
            ("toolbar", "shadow-height"),
        ] {
            let schema = Schema::new().variant(widget, "custom", &[]);
            let section = format!("{widget}.custom");
            let spec = schema.get(&section, key).unwrap();
            assert_eq!(spec.default, None);
            assert!(matches!(spec.ty, VariableType::Length { .. }));
            assert!(
                !schema
                    .validate(&ron_one(&section, key, "-1"))
                    .is_valid()
            );
            let cfg = ThemeConfig::builder("geometry-variants")
                .files(false)
                .schema(schema)
                .toolkit_text(format!(
                    "{{ {widget:?}: {{ {key:?}: 10 }}, {section:?}: {{ {key:?}: 12 }}, \"dark:{section}\": {{ {key:?}: 18 }} }}"
                ))
                .app_text(format!(
                    "{{ \"light:{section}\": {{ {key:?}: 14 }}, \"dark:{section}\": {{ {key:?}: NaN }} }}"
                ))
                .build();
            let light = cfg.clone().with_variation("light").tokens();
            let dark = cfg.with_variation("dark").tokens();
            assert_eq!(light.variant_number(widget, "custom", key), Some(14.0));
            assert_eq!(dark.variant_number(widget, "custom", key), Some(18.0));
            assert_eq!(dark.variant_number(widget, "undeclared", key), Some(10.0));
            assert_eq!(dark.variant_number(widget, "custom", "missing"), None);
            let empty = ThemeConfig::builder("geometry-variants")
                .files(false)
                .schema(Schema::new().variant(widget, "empty", &[]))
                .toolkit_text(ron_one(widget, key, "16"))
                .app_text("")
                .build()
                .tokens();
            assert_eq!(empty.variant_number(widget, "empty", key), Some(16.0));
        }
        let schema = Schema::toolkit();
        for (widget, names) in [
            ("tree", vec!["default"]),
            ("tree-row", vec!["default"]),
            ("navigation", vec!["default", "sm", "md", "lg"]),
        ] {
            for name in names {
                assert!(
                    schema
                        .variants_declared()
                        .contains(&(widget.into(), name.into()))
                );
                assert!(
                    schema
                        .specs()
                        .iter()
                        .filter(|s| s.section == format!("{widget}.{name}"))
                        .all(|s| s.default.is_none())
                );
            }
        }
    }

    #[test]
    fn pending_optional_geometry_tokens_have_bounded_types() {
        let schema = Schema::toolkit();
        for (section, key, max) in [
            ("tabs", "close-icon-size", 128.0),
            ("tabs", "indicator-height", 32.0),
            ("tabs", "border-width", 32.0),
            ("tabs", "separator-width", 32.0),
            ("tabs", "focus-inset", 64.0),
            ("toolbar", "shadow-height", 128.0),
            ("toolbar", "separator-width", 32.0),
            ("color-picker", "width", 4096.0),
            ("color-input", "popover-width", 4096.0),
            ("number-input", "width", 4096.0),
        ] {
            let spec = schema.get(section, key).unwrap();
            assert_eq!(spec.default, None);
            assert_eq!(
                spec.ty,
                VariableType::Length {
                    min: Some(0.0),
                    max: Some(max)
                }
            );
        }
        let opacity = schema.get("toolbar", "shadow-opacity").unwrap();
        assert_eq!(opacity.default, None);
        for raw in ["0", "0.5", "1"] {
            assert!(opacity.ty.parse(raw).is_some());
        }
        for raw in ["-1", "1.001", "NaN", "inf"] {
            assert!(opacity.ty.parse(raw).is_none());
        }
    }

    fn build(toolkit: &str, app: &str) -> ThemeConfig {
        ThemeConfig::builder("tree-navigation-test")
            .files(false)
            .toolkit_text(toolkit)
            .app_text(app)
            .build()
    }

    #[test]
    fn defaults_and_optional_metric_inheritance_reach_resolvers() {
        let cfg = build("", "");
        for (section, entries) in [
            (
                "tree",
                vec![
                    ("level-offset", 20.0),
                    ("row-height", 32.0),
                    ("handle-padding", 8.0),
                    ("expander-icon-inset", 2.0),
                    ("grip-width", 32.0),
                    ("drag-threshold", 4.0),
                    ("checkbox-size-extra", 2.0),
                    ("checkbox-glyph-inset", 4.0),
                    ("checkbox-radius", 4.0),
                    ("checkbox-border-width", 1.5),
                    ("connector-width", 1.0),
                    ("drop-border-width", 1.5),
                    ("drop-line-width", 2.0),
                    ("drop-line-radius", 1.0),
                    ("drag-border-width", 2.0),
                    ("drag-scale", 1.01),
                    ("drag-clip-margin", 24.0),
                    ("drag-shadow-offset", 2.0),
                ],
            ),
            (
                "tree-row",
                vec![("separator-width", 1.0), ("children-opacity", 0.03)],
            ),
            (
                "navigation",
                vec![
                    ("indent", 14.0),
                    ("initial-sidebar-width", 240.0),
                    ("initial-drawer-width", 280.0),
                    ("separator-width", 1.0),
                    ("selection-opacity", 0.10),
                ],
            ),
        ] {
            for (key, default) in entries {
                assert_eq!(cfg.number(section, key), Some(default), "{section}.{key}");
                assert_eq!(cfg.tokens().number(section, key), Some(default));
            }
        }
        for (section, keys) in [
            (
                "tree",
                vec![
                    "icon-size",
                    "icon-padding",
                    "label-padding",
                    "checkbox-padding",
                    "row-radius",
                ],
            ),
            (
                "tree-row",
                vec![
                    "indent",
                    "handle-width",
                    "handle-padding-left",
                    "glyph-size",
                ],
            ),
        ] {
            for key in keys {
                assert!(cfg.schema().get(section, key).is_some());
                assert_eq!(cfg.number(section, key), None);
                assert!(!cfg.is_set(section, key));
                let inherited = build(
                    &ron_one(section, key, "12"),
                    &ron_one(section, key, "NaN"),
                );
                assert_eq!(inherited.number(section, key), Some(12.0));
            }
        }
    }

    #[test]
    fn every_geometry_token_rejects_invalid_values_and_falls_back() {
        let schema = Schema::toolkit();
        for spec in schema.specs().iter().filter(|s| {
            ["tree", "tree-row", "navigation"].contains(&s.section.as_str())
                && s.key != "breakpoint-rules"
        }) {
            let length = matches!(spec.ty, VariableType::Length { .. });
            let valid = if length { "2" } else { "0.5" };
            for raw in if length {
                ["-1", "NaN", "inf", "100000"]
            } else {
                ["-1", "NaN", "inf", "100000"]
            } {
                let text = ron_one(&spec.section, &spec.key, raw);
                assert!(!schema.validate(&text).is_valid(), "{text}");
                let cfg = build("", &text);
                assert_eq!(
                    cfg.get(&spec.section, &spec.key),
                    spec.default.as_ref(),
                    "{text}"
                );
                let lower = ron_one(&spec.section, &spec.key, valid);
                let cfg = build(&lower, &text);
                assert_eq!(
                    cfg.number(&spec.section, &spec.key),
                    Some(if length { 2.0 } else { 0.5 })
                );
            }
            let zero = "0";
            let text = ron_one(&spec.section, &spec.key, zero);
            assert_eq!(
                schema.validate(&text).is_valid(),
                !(spec.section == "tree"
                    && ["row-height", "drag-scale"].contains(&spec.key.as_str())),
                "{text}"
            );
        }
        for section_key in [
            ("tree-row", "children-opacity"),
            ("navigation", "selection-opacity"),
        ] {
            assert!(
                !schema
                    .validate(&ron_one(section_key.0, section_key.1, "1.001"))
                    .is_valid()
            );
            assert!(
                schema
                    .validate(&ron_one(section_key.0, section_key.1, "1"))
                    .is_valid()
            );
        }
        assert!(
            schema
                .validate(&schema.template("Tree and navigation"))
                .is_valid()
        );
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
            r##"{
    "panel": {
        "gap": 12,
        "optional": "yes",
    },
}
"##,
            r##"{
    "panel": {
        "gap": 500,
        "optional": "no",
    },
}
"##,
        );
        assert_eq!(cfg.get("panel", "gap"), Some(&Value::Length(12.0)));
        assert_eq!(cfg.text("panel", "optional"), Some("yes"));
        assert_eq!(cfg.warnings().len(), 2);
        let cfg = build(
            r##"{
    "panel": {
        "gap": "no",
        "optional": "no",
    },
}
"##,
            r##"{
    "panel": {
        "gap": 500,
        "optional": "no",
    },
}
"##,
        );
        assert_eq!(cfg.get("panel", "gap"), Some(&Value::Length(8.0)));
        assert_eq!(cfg.get("panel", "optional"), None);
    }
    #[test]
    fn variations_always_override_base_and_can_be_switched_without_losing_values() {
        let cfg = build(
            r##"{
    "dark:panel": {
        "gap": 20,
    },
}
"##,
            r##"{
    "panel": {
        "gap": 10,
    },
    "high-contrast:panel": {
        "gap": 30,
    },
}
"##,
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
            r##"{
    "panel": {
        "gap": 12,
        "optional": "yes",
    },
    "dark:panel": {
        "gap": "no",
        "optional": "no",
    },
}
"##,
        )
        .with_variation("dark");
        assert_eq!(cfg.get("panel", "gap"), Some(&Value::Length(12.0)));
        assert_eq!(cfg.text("panel", "optional"), Some("yes"));
        assert_eq!(cfg.tokens().text("panel", "optional"), Some("yes"));
        let cfg = build(r##"{
    "dark:panel": {
        "gap": 24,
    },
}
"##, r##"{
    "dark:panel": {
        "gap": "no",
    },
}
"##).with_variation("dark");
        assert_eq!(cfg.get("panel", "gap"), Some(&Value::Length(24.0)));
    }
    #[test]
    fn templates_validate_and_include_variations_and_optional_fallbacks() {
        let schema = schema();
        let template = schema.template("Example");
        assert!(template.contains("\"dark:panel\": {") && template.contains("\"high-contrast:panel\": {"));
        assert!(template.contains("// \"gap\": 8.0,") && template.contains("no fallback"));
        assert!(schema.validate(&template).is_valid());
        let report =
            schema.validate(r##"{
    "panel": {
        "gap": "1em",
        "unknown": "yes",
    },
    "typo:panel": {
        "gap": 20,
    },
}
"##);
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.unknown.len(), 2);
        // Malformed RON is one error that says where.
        let broken = schema.validate("{ \"panel\": { \"gap\": } }");
        assert_eq!(broken.errors.len(), 1);
        assert!(broken.errors[0].contains("1:"), "{:?}", broken.errors);
    }
    #[test]
    fn themes_automatically_cascade_light_dark_and_respect_custom_variations() {
        let cfg = build(
            "",
            r##"{
    "dark:metrics": {
        "row-min-height": 80,
    },
    "dark:colors": {
        "card-bg": "#123456",
    },
    "high-contrast:metrics": {
        "row-min-height": 90,
    },
}
"##,
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

/// The libadwaita look, read from `themes/libadwaita.ron`, for tests that assert its values.
#[cfg(test)]
pub(crate) fn libadwaita() -> ThemeConfig {
    ThemeConfig::builder("libadwaita-test")
        .files(false)
        .app_text(include_str!("../themes/libadwaita.ron"))
        .build()
}

/// The libadwaita metrics.
#[cfg(test)]
pub(crate) fn libadwaita_metrics() -> Metrics {
    let mut m = Metrics::default();
    libadwaita().apply_to_metrics(&mut m);
    m
}

#[cfg(test)]
mod libadwaita_theme_tests {
    use super::*;

    #[test]
    fn every_shipped_theme_validates_against_the_toolkit_schema() {
        for (name, text) in [
            ("libadwaita", include_str!("../themes/libadwaita.ron")),
            ("cosmic", include_str!("../themes/cosmic.ron")),
            ("solarized", include_str!("../themes/solarized.ron")),
            ("terminal", include_str!("../themes/terminal.ron")),
            ("bubblegum", include_str!("../themes/bubblegum.ron")),
            ("synthwave", include_str!("../themes/synthwave.ron")),
            ("win95", include_str!("../themes/win95.ron")),
            ("winxp", include_str!("../themes/winxp.ron")),
            ("win7", include_str!("../themes/win7.ron")),
            ("mate", include_str!("../themes/mate.ron")),
            ("gnome2", include_str!("../themes/gnome2.ron")),
        ] {
            // RON keeps only the last of two sections with the same name, silently: a section
            // appears once per file.
            let mut sections: Vec<&str> = text
                .lines()
                .filter_map(|l| l.strip_prefix("    \"")?.split_once("\": {"))
                .map(|(name, _)| name)
                .collect();
            let all = sections.len();
            sections.sort_unstable();
            sections.dedup();
            assert_eq!(all, sections.len(), "{name} repeats a section");
            let report = Schema::toolkit().validate(text);
            assert!(report.errors.is_empty(), "{name}: {:?}", report.errors);
            assert!(report.unknown.is_empty(), "{name}: {:?}", report.unknown);
            let cfg = ThemeConfig::builder("t")
                .files(false)
                .toolkit_text(text)
                .app_text("")
                .build();
            assert!(cfg.warnings().is_empty(), "{name}: {:?}", cfg.warnings());
        }
    }

    #[test]
    fn the_libadwaita_file_is_valid_and_carries_libadwaita_values() {
        let cfg = libadwaita();
        assert!(cfg.warnings().is_empty(), "{:?}", cfg.warnings());
        assert!(cfg.unknown().is_empty(), "{:?}", cfg.unknown());
        let m = libadwaita_metrics();
        assert_eq!(
            (m.radius_m, m.radius_l, m.radius_popover, m.radius_window),
            (9.0, 12.0, 15.0, 15.0)
        );
        let mut light = Palette::light();
        cfg.apply_to_palette(&mut light, Scheme::Light);
        assert_eq!(light.window_bg_color, crate::color::hex(0xfafafb));
        assert_eq!(light.sidebar_bg_color, crate::color::hex(0xebebed));
        let mut dark = Palette::dark();
        cfg.apply_to_palette(&mut dark, Scheme::Dark);
        assert_eq!(dark.window_bg_color, crate::color::hex(0x222226));
        assert_eq!(dark.dialog_bg_color, crate::color::hex(0x36363a));
        assert_eq!(dark.card_bg_color.a, 0.08);
        let mut shadows = Shadows::bare();
        cfg.apply_to_shadows(&mut shadows);
        assert!(shadows.card().ring.is_some());
        assert_eq!(cfg.text("window-decorations", "button-layout"), Some("appmenu:close"));
        assert_eq!(cfg.number("window-decorations", "button-size"), Some(24.0));
    }
}

#[cfg(test)]
mod inheritance_tests {
    use super::*;

    fn cfg(app: &str) -> ThemeConfig {
        ThemeConfig::builder("test")
            .files(false)
            .app_text(app)
            .build()
    }

    #[test]
    fn the_mode_is_inferred_from_what_the_app_sets() {
        assert_eq!(cfg("").inheritance(), Inheritance::Inherit);
        assert_eq!(
            cfg(r##"{
    "theme": {
        "name": "x",
        "key": "x",
    },
}
"##).inheritance(),
            Inheritance::Inherit,
            "metadata is not an override"
        );
        let c = cfg(r##"{
    "metrics": {
        "radius-m": 3,
    },
}
"##);
        assert_eq!(c.inheritance(), Inheritance::OverrideValue);
        assert_eq!(
            c.overrides(),
            vec![("metrics".to_string(), "radius-m".to_string())]
        );
        let c = cfg(r##"{
    "dark:colors": {
        "card-bg": "#000000",
    },
}
"##);
        assert_eq!(c.inheritance(), Inheritance::OverrideValue);
    }

    #[test]
    fn the_file_declares_the_mode_and_code_wins() {
        assert_eq!(
            cfg(r##"{
    "theme": {
        "mode": "optout",
    },
}
"##).inheritance(),
            Inheritance::OptOut
        );
        assert_eq!(
            cfg(r##"{
    "theme": {
        "mode": "override",
    },
}
"##).inheritance(),
            Inheritance::OverrideValue
        );
        assert_eq!(
            cfg(r##"{
    "theme": {
        "mode": "bogus",
    },
}
"##).inheritance(),
            Inheritance::Inherit,
            "an invalid mode is not a mode"
        );
        let c = ThemeConfig::builder("test")
            .files(false)
            .app_text(r##"{
    "theme": {
        "mode": "optout",
    },
}
"##)
            .inheritance(Inheritance::Inherit)
            .build();
        assert_eq!(c.inheritance(), Inheritance::Inherit);
    }

    #[test]
    fn opting_out_discards_the_desktop_files_and_the_portal_scheme() {
        let desktop = crate::config::Config::from_ron(
            r##"{
    "appearance": {
        "color-scheme": "dark",
        "text-scale": 1.5,
    },
}
"##,
        );
        let out = desktop
            .clone()
            .with_theme_config(&cfg(r##"{
    "theme": {
        "mode": "optout",
    },
}
"##));
        assert!(out.desktop_opt_out);
        assert_eq!(out.text_scale, 1.0);
        let inherit = desktop.with_theme_config(&cfg(""));
        assert!(!inherit.desktop_opt_out);
        assert_eq!(inherit.text_scale, 1.5);
        // No fixed scheme, so the portal would decide for an inheriting app; not for this one.
        assert_eq!(
            crate::utils::effective_scheme(&out, Some(Scheme::Dark)),
            Scheme::Light
        );
    }
}

#[cfg(test)]
mod policy_tests {
    use super::*;

    fn app(text: &str) -> ThemeConfig {
        ThemeConfig::builder("test")
            .files(false)
            .app_text(text)
            .build()
    }

    #[test]
    fn a_policy_that_forbids_overrides_drops_the_apps_tokens_and_lists_them() {
        let c = app(r##"{
    "metrics": {
        "radius-m": 3,
    },
}
"##);
        assert_eq!(c.number("metrics", "radius-m"), Some(3.0));
        let kept = c.for_policy(true);
        assert_eq!(kept.number("metrics", "radius-m"), Some(3.0));
        let denied = c.for_policy(false);
        assert_eq!(
            denied.number("metrics", "radius-m"),
            Some(Metrics::default().radius_m)
        );
        assert_eq!(
            denied.denied(),
            [("metrics".to_string(), "radius-m".to_string())]
        );
        assert_eq!(denied.inheritance(), Inheritance::Inherit);
        assert!(denied.overrides().is_empty());
    }

    #[test]
    fn the_policy_leaves_inherit_and_optout_apps_alone() {
        let opt = app(r##"{
    "theme": {
        "mode": "optout",
    },
    "metrics": {
        "radius-m": 3,
    },
}
"##);
        assert_eq!(
            opt.for_policy(false).number("metrics", "radius-m"),
            Some(3.0)
        );
        assert!(opt.for_policy(false).denied().is_empty());
        let inherit = app("");
        assert!(inherit.for_policy(false).denied().is_empty());
    }

    #[test]
    fn the_config_applies_the_policy_to_the_apps_theme() {
        let mut desktop = crate::config::Config::default();
        desktop.allow_app_overrides = false;
        let out = desktop.with_theme_config(&app(r##"{
    "metrics": {
        "radius-m": 3,
    },
}
"##));
        assert_eq!(out.metrics().radius_m, Metrics::default().radius_m);
        let free =
            crate::config::Config::default().with_theme_config(&app(r##"{
    "metrics": {
        "radius-m": 3,
    },
}
"##));
        assert_eq!(free.metrics().radius_m, 3.0);
    }

    #[test]
    fn a_click_maps_to_an_action_the_window_allows() {
        use crate::windows::{Action, Chrome, State};
        let full = Chrome::new(State::default()).csd_override(true);
        assert!(matches!(
            Action::titlebar("double", &full),
            Some(Action::ToggleMaximize)
        ));
        assert!(matches!(
            Action::titlebar("right", &full),
            Some(Action::Menu)
        ));
        assert!(Action::titlebar("middle", &full).is_none());
        let dialog = full.close_only();
        assert!(Action::titlebar("double", &dialog).is_none());
    }
}

#[cfg(test)]
mod primary_and_contrast_tests {
    use super::*;

    fn themed(text: &str) -> crate::Theme {
        let cfg = ThemeConfig::builder("t")
            .files(false)
            .toolkit_text(text)
            .app_text("")
            .build();
        crate::Theme::light().with_theme_config(&cfg)
    }

    #[test]
    fn suggested_buttons_use_the_primary_color_not_the_accent() {
        let t = themed(r##"{
    "colors": {
        "accent": "#ff0000",
        "primary": "#00aa00",
        "primary-fg": "#fffffe",
    },
}
"##);
        let style = crate::styles::button::suggested(&t, iced::widget::button::Status::Active);
        assert_eq!(
            style.background,
            Some(iced::Background::Color(crate::color::hex(0x00aa00)))
        );
        assert_eq!(t.semantic.accent.bg, crate::color::hex(0xff0000));
        // A desktop accent change leaves the primary alone.
        let t = t.accent(crate::AccentColor::Teal);
        assert_eq!(t.semantic.primary.bg, crate::color::hex(0x00aa00));
    }

    #[test]
    fn text_on_fills_is_the_themes_fg_until_it_opts_in_to_contrast() {
        let fixed = themed(r##"{
    "colors": {
        "warning": "#ffee00",
        "warning-fg": "#ffffff",
    },
}
"##);
        assert_eq!(fixed.semantic.warning.fg, iced::Color::WHITE);
        let auto = themed(r##"{
    "colors": {
        "warning": "#ffee00",
        "warning-fg": "#ffffff",
        "auto-contrast": true,
    },
}
"##);
        assert_eq!(auto.semantic.warning.fg, crate::color::DARK_TEXT);
    }
}

#[cfg(test)]
mod variant_tests {
    use super::*;
    use crate::styles::button as sb;
    use iced::widget::button::Status;

    fn build(b: Builder) -> crate::Theme {
        crate::Theme::light().with_theme_config(&b.files(false).build())
    }

    /// A color as the hex it prints, so a 1-ulp difference between parsing and `hex()` is moot.
    fn hx(c: Option<Color>) -> Option<String> {
        c.map(crate::color::to_hex)
    }

    fn bg(s: iced::widget::button::Style) -> Option<String> {
        hx(match s.background {
            Some(iced::Background::Color(c)) => Some(c),
            _ => None,
        })
    }

    fn hex(rgb: u32) -> Option<String> {
        Some(crate::color::to_hex(crate::color::hex(rgb)))
    }

    fn brand() -> Schema {
        Schema::new().variant(
            "button",
            "brand",
            &[
                ("bg", Value::Color(crate::color::hex(0xaa0000))),
                ("radius", Value::Length(3.0)),
            ],
        )
    }

    #[test]
    fn the_toolkit_declares_the_default_button_variants_with_typed_keys() {
        let schema = Schema::toolkit();
        for name in DEFAULT_VARIANTS[0].1 {
            let section = format!("button.{name}");
            assert_eq!(
                schema.get(&section, "bg").unwrap().ty,
                VariableType::Color,
                "{name}"
            );
            assert!(matches!(
                schema.get(&section, "radius").unwrap().ty,
                VariableType::Length { .. }
            ));
            assert!(schema.get(&section, "bg").unwrap().default.is_none());
        }
        // A bad value is reported, not applied.
        let r = schema.validate(r##"{
    "button.flat": {
        "bg": "nope",
        "radius": 3,
    },
}
"##);
        assert_eq!(r.errors.len(), 1, "{:?}", r.errors);
    }

    #[test]
    fn an_app_adds_a_variant_and_a_theme_file_restyles_it() {
        let t = build(
            ThemeConfig::builder("t")
                .schema(brand())
                .app_text(r##"{
    "button.brand": {
        "fg": "#112233",
    },
}
"##),
        );
        let v = t.variant("button", "brand");
        assert!(v.exists());
        assert_eq!(hx(v.color("bg")), hex(0xaa0000), "declared default");
        assert_eq!(hx(v.color("fg")), hex(0x112233), "from the file");
        let style = sb::variant("brand")(&t, Status::Active);
        assert_eq!(bg(style), hex(0xaa0000));
        assert_eq!(crate::color::to_hex(style.text_color), "#112233");
        assert_eq!(style.border.radius, 3.0.into());
        // Hovering derives the fill from the same token.
        assert_eq!(
            bg(sb::variant("brand")(&t, Status::Hovered)),
            hx(Some(t.semantic.hover_fill(crate::color::hex(0xaa0000))))
        );
    }

    #[test]
    fn a_theme_overrides_one_key_of_a_default_variant_and_the_rest_stays_built_in() {
        let plain = crate::Theme::light();
        let t = build(
            ThemeConfig::builder("t")
                .toolkit_text(r##"{
    "button.suggested": {
        "bg": "#00aa00",
    },
}
"##)
                .app_text(""),
        );
        let green = crate::color::hex(0x00aa00);
        let gh = |c: Color| Some(crate::color::to_hex(c));
        assert_eq!(bg(sb::suggested(&t, Status::Active)), gh(green));
        assert_eq!(
            bg(sb::suggested(&t, Status::Pressed)),
            gh(t.semantic.active_fill(green))
        );
        // The text color is still the palette's, and other variants are untouched.
        assert_eq!(
            sb::suggested(&t, Status::Active).text_color,
            sb::suggested(&plain, Status::Active).text_color
        );
        assert_eq!(
            bg(sb::destructive(&t, Status::Active)),
            bg(sb::destructive(&plain, Status::Active))
        );
        // A radius alone leaves a flat button flat.
        let t = build(
            ThemeConfig::builder("t")
                .toolkit_text(r##"{
    "button.flat": {
        "radius": 3,
    },
}
"##)
                .app_text(""),
        );
        let flat = sb::flat(&t, Status::Active);
        assert!(flat.background.is_none());
        assert_eq!(flat.border.radius, 3.0.into());
    }

    #[test]
    fn without_a_theme_config_the_default_variants_exist() {
        let t = crate::Theme::light();
        assert!(t.variant("button", "suggested").exists());
        assert!(!t.variant("button", "nope").exists());
        assert!(!t.variant("chip", "suggested").exists());
    }

    #[test]
    fn opting_out_of_the_defaults_leaves_the_apps_own_and_draws_the_rest_neutral() {
        let t = build(
            ThemeConfig::builder("t")
                .schema(brand())
                .default_variants(DefaultVariants::None)
                .app_text(""),
        );
        assert!(!t.variant("button", "suggested").exists());
        assert!(t.variant("button", "brand").exists());
        let neutral = sb::default(&t, Status::Active);
        assert_eq!(
            bg(sb::variant("suggested")(&t, Status::Active)),
            bg(neutral)
        );
        assert_eq!(
            bg(sb::variant("typo")(&t, Status::Hovered)),
            bg(sb::default(&t, Status::Hovered))
        );
        assert_eq!(bg(sb::variant("brand")(&t, Status::Active)), hex(0xaa0000));
        // The file can ask for it too, and a file naming a dropped variant reports it as unknown.
        let cfg = ThemeConfig::builder("t")
            .files(false)
            .app_text(r##"{
    "theme": {
        "default-variants": "none",
    },
    "button.suggested": {
        "bg": "#00aa00",
    },
}
"##)
            .build();
        assert!(
            !cfg.schema()
                .variants_declared()
                .iter()
                .any(|(_, n)| n == "suggested")
        );
        assert!(cfg.unknown().iter().any(|(s, _)| s == "button.suggested"));
    }

    #[test]
    fn some_widgets_keep_their_defaults_and_others_drop_them() {
        assert!(DefaultVariants::All.keeps("button"));
        assert!(!DefaultVariants::None.keeps("button"));
        assert!(DefaultVariants::Only(&["button"]).keeps("button"));
        assert!(!DefaultVariants::Only(&["chip"]).keeps("button"));
        assert!(!DefaultVariants::Without(&["button"]).keeps("button"));
        let t = build(
            ThemeConfig::builder("t")
                .default_variants(DefaultVariants::Without(&["button"]))
                .app_text(""),
        );
        assert!(!t.variant("button", "flat").exists());
    }

    #[test]
    fn an_app_replaces_a_default_variant_by_declaring_its_name() {
        let schema = Schema::new().variant(
            "button",
            "suggested",
            &[("bg", Value::Color(crate::color::hex(0x0000aa)))],
        );
        let t = build(ThemeConfig::builder("t").schema(schema).app_text(""));
        assert_eq!(bg(sb::suggested(&t, Status::Active)), hex(0x0000aa));
    }
}

#[cfg(test)]
mod design_language_tests {
    use super::*;
    use crate::components::bevel_of;
    use crate::primitives::bevel::BevelStyle;
    use iced::widget::button::Status;

    const WIN95: &str = r##"{
    "theme": {
        "name": "Test 95",
    },
    "colors": {
        "window-bg": "#c0c0c0",
        "window-fg": "#000000",
        "focus-ring": "#000000",
    },
    "gradients": {
        "titlebar": (angle: 90.0, stops: [(0.0, "#000080"), (1.0, "#1084d0")]),
    },
    "window-decorations": {
        "frame-border-width": 4,
        "frame-border": "#c0c0c0",
        "frame-border-style": "window",
        "titlebar-align": "start",
        "titlebar-font-weight": "bold",
        "titlebar-icon": true,
        "button-style": "win95",
    },
    "typography": {
        "font-family": "Tahoma",
        "monospace-font-family": "Fixedsys",
    },
    "metrics": {
        "focus-style": "dotted",
    },
    "button.default": {
        "bg": "#c0c0c0",
        "border-style": "raised",
        "border-width": 2,
        "border-light": "#ffffff",
        "border-dark": "#000000",
        "border-light-inner": "#dfdfdf",
        "border-dark-inner": "#808080",
        "bg-gradient": (angle: 180.0, stops: [(0.0, "#ffffff"), (1.0, "#c0c0c0")]),
    },
    "frame.field": {
        "border-style": "sunken",
    },
}
"##;

    fn cfg(text: &str) -> ThemeConfig {
        ThemeConfig::builder("t")
            .files(false)
            .toolkit_text(text)
            .app_text("")
            .build()
    }

    #[test]
    fn every_design_token_is_declared_and_a_crude_95_validates() {
        let report = Schema::toolkit().validate(WIN95);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert!(report.unknown.is_empty(), "{:?}", report.unknown);
        let c = cfg(WIN95);
        assert!(c.warnings().is_empty(), "{:?}", c.warnings());
        assert!(c.unknown().is_empty(), "{:?}", c.unknown());
    }

    #[test]
    fn invalid_design_values_are_refused() {
        for bad in [
            r##"{
    "button.default": {
        "border-style": "groove",
    },
}
"##,
            r##"{
    "typography": {
        "font-family": "",
    },
}
"##,
            r##"{
    "typography": {
        "font-family": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    },
}
"##,
            r##"{
    "window-decorations": {
        "titlebar-align": "middle",
    },
}
"##,
            r##"{
    "window-decorations": {
        "titlebar-font-weight": "heavy",
    },
}
"##,
            r##"{
    "window-decorations": {
        "frame-border-width": 900,
    },
}
"##,
            r##"{
    "metrics": {
        "focus-style": "wavy",
    },
}
"##,
            r##"{
    "gradients": {
        "titlebar": (angle: 90.0, stops: [(0.0, "#000")]),
    },
}
"##,
            r##"{
    "button.default": {
        "bg-gradient": "#ffffff",
    },
}
"##,
        ] {
            let c = cfg(bad);
            assert!(!c.warnings().is_empty(), "{bad:?} was accepted");
        }
    }

    #[test]
    fn a_theme_carries_gradients_fonts_focus_and_bevel_colors() {
        let t = crate::Theme::light().with_theme_config(&cfg(WIN95));
        let g = t.gradient("titlebar").unwrap();
        assert_eq!((g.angle, g.stops.len()), (90.0, 2));
        assert!(t.gradient("card").is_none());
        assert!(matches!(
            t.surface_background("titlebar", iced::Color::WHITE),
            iced::Background::Gradient(_)
        ));
        assert!(matches!(
            t.surface_background("card", iced::Color::WHITE),
            iced::Background::Color(_)
        ));
        assert_eq!(
            t.font().map(|f| f.family),
            Some(iced::font::Family::Name("Tahoma"))
        );
        assert_eq!(
            t.monospace_font().map(|f| f.family),
            Some(iced::font::Family::Name("Fixedsys"))
        );
        assert_eq!(t.focus_width(), 1.0);
        assert_eq!(t.semantic.focus_ring, crate::color::hex(0x000000));
        let b = t.bevel_colors("button.default");
        assert_eq!(
            (b.light, b.dark_inner),
            (crate::color::hex(0xffffff), crate::color::hex(0x808080))
        );
        // An unset bevel section has the classic gray-frame colors.
        assert_eq!(
            t.bevel_colors("frame.nothing").dark_inner,
            crate::color::hex(0x808080)
        );
    }

    #[test]
    fn without_design_tokens_nothing_changes() {
        let t = crate::Theme::light();
        assert!(t.gradient("titlebar").is_none() && t.font().is_none());
        assert_eq!(t.focus_width(), t.metrics.focus_ring_width);
    }

    #[test]
    fn a_button_variant_with_a_gradient_fills_with_it_and_derives_its_states() {
        let t = crate::Theme::light().with_theme_config(&cfg(WIN95));
        let style = crate::styles::button::variant("default");
        let bg = |s| style(&t, s).background;
        assert!(matches!(
            bg(Status::Active),
            Some(iced::Background::Gradient(_))
        ));
        assert_ne!(
            bg(Status::Active),
            bg(Status::Hovered),
            "hover derives from the gradient"
        );
        assert_ne!(bg(Status::Hovered), bg(Status::Pressed));
        assert!(matches!(
            bg(Status::Disabled),
            Some(iced::Background::Gradient(_))
        ));
        // The bevel is the border: no solid one is drawn under it.
        assert_eq!(style(&t, Status::Active).border.width, 0.0);
    }

    #[test]
    fn a_variant_without_a_gradient_or_bevel_is_as_before() {
        let plain = crate::Theme::light();
        let themed = crate::Theme::light().with_theme_config(&cfg(r##"{
    "button.flat": {
        "bg": "#ff0000",
    },
}
"##));
        let a = crate::styles::button::variant("default")(&plain, Status::Active);
        let b = crate::styles::button::variant("default")(&themed, Status::Active);
        assert_eq!(a.background, b.background);
        assert_eq!(a.border.width, b.border.width);
    }

    /// The installed theme config is process-wide: one test at a time sets it, then puts it back.
    static GLOBAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn the_installed_theme_decides_which_builders_make_a_bevel_and_which_window_frame_is_drawn() {
        let _guard = GLOBAL.lock().unwrap_or_else(|e| e.into_inner());
        let before = (*current()).clone();
        assert_eq!(bevel_of("button.default"), None);
        assert_eq!(window_frame(), None);
        install(cfg(WIN95));
        assert_eq!(bevel_of("button.default"), Some((BevelStyle::Raised, 2.0)));
        assert_eq!(bevel_of("frame.field"), Some((BevelStyle::Sunken, 2.0)));
        assert_eq!(bevel_of("button.flat"), None);
        let f = window_frame().unwrap();
        assert_eq!((f.width, f.bevel), (4.0, Some(BevelStyle::Window)));
        assert_eq!(titlebar_align(), "start");
        assert!(titlebar_bold() && titlebar_icon());
        assert_eq!(window_button_style().as_deref(), Some("win95"));
        assert_eq!(
            font().map(|f| f.family),
            Some(iced::font::Family::Name("Tahoma"))
        );
        // The builders produce elements either way.
        let _: crate::Element<'_, u8> =
            crate::components::raised_button(iced::widget::space()).into();
        let _: crate::Element<'_, u8> = crate::components::framed(iced::widget::space(), "field");
        install(before);
        assert_eq!(bevel_of("button.default"), None);
        assert!(font().is_none());
    }
}
