//! # iced-adaptive-design (`iad`)
//!
//! An adaptive design system for [iced](https://iced.rs): a theme, components and behavior
//! primitives that follow the GNOME HIG. Import it as `use iced_adaptive_design as iad;`.
//!
//! * [`Theme`]: one theme type implementing every iced widget catalog, with a
//!   named [`Palette`], a [`Semantic`] color layer, [`Metrics`] and
//!   [`Typography`] tokens.
//! * [`styles`]: style functions mirroring libadwaita's CSS classes.
//!
//! The rest is in layers, each importing only the ones below it (DESIGN.md, "Layers"):
//!
//! * [`primitives`]: headless rules and state machines. No widgets, no look.
//! * [`layouts`]: where things go and how they flow: grid, flex, overlay, dock, split, motion.
//! * [`binds`]: iced events turned into primitive input (focus, shortcuts).
//! * [`components`]: themed atoms with no information of their own.
//! * [`metas`]: choose between, or join, whole widgets.
//! * [`widgets`]: what the user sees and uses, named after their libadwaita counterparts (`HeaderBar`,
//!   `ActionRow`, `PreferencesGroup`, `StatusPage`, `AlertDialog`, …).
//! * [`windows`]: window state, chrome and frame logic.

pub mod adaptive;
pub mod binds;
pub mod breakpoints;
pub mod color;
pub mod components;
pub mod config;
pub mod config_history;
pub mod dev;
pub mod icon;
pub mod layouts;
pub mod lookup;
pub mod metas;
pub mod metrics;
pub mod palette;
pub mod platform;
pub mod prelude;
pub mod primitives;
pub mod semantic;
pub mod shadow;
pub mod shortcut;
pub mod styles;
pub mod system;
pub mod theme;
pub mod theme_config;
pub mod utils;
pub mod widgets;
pub mod windows;

/// The animation engine, a [primitive](layouts::animation); kept at its old path.
pub use layouts::animation;
/// The ordered-transition coordinator, a [primitive](layouts::transition); kept at its old path.
pub use layouts::transition;

pub use icon::{AppearanceIcon, ChevronIcon, EntryIcon, Icon, IconSource, IconView, WindowIcon};
pub use metrics::{CollapseBehavior, Metrics, Typography};
pub use palette::{AccentColor, Palette, Scheme};
pub use semantic::{Opacity, Role, Semantic, Surface, Swatch};
pub use shadow::{Elevation, Layer, Resolved, Shadows};
pub use system::{DecorationLayout, WindowButton};
pub use theme::Theme;
/// Shorthand for an iced element using this crate's [`Theme`].
pub type Element<'a, Message> = iced::Element<'a, Message, Theme>;
