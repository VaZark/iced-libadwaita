//! # iced-adaptive-design (`iad`)
//!
//! An adaptive design system for [iced](https://iced.rs): a theme, components and behavior
//! primitives that follow the GNOME HIG. Import it as `use iced_adaptive_design as iad;`.
//!
//! * [`Theme`]: one theme type implementing every iced widget catalog, with a
//!   named [`Palette`], a [`Semantic`] color layer, [`Metrics`] and
//!   [`Typography`] tokens.
//! * [`style`]: style functions mirroring libadwaita's CSS classes.
//! * [`widget`]: components named after their libadwaita counterparts (`HeaderBar`, `ActionRow`, `PreferencesGroup`,
//!   `StatusPage`, `AlertDialog`, …).

pub mod adaptive;
pub mod breakpoints;
pub mod color;
pub mod components;
pub mod config;
pub mod config_history;
pub mod dev;
pub mod icon;
pub mod lookup;
pub mod metrics;
pub mod palette;
pub mod prelude;
pub mod primitives;
pub mod semantic;
pub mod shadow;
pub mod shortcut;
pub mod style;
pub mod system;
pub mod theme;
pub mod theme_config;
pub mod widget;
pub mod window;

/// The animation engine, a [primitive](primitives::animation); kept at its old path.
pub use primitives::animation;
/// The ordered-transition coordinator, a [primitive](primitives::transition); kept at its old path.
pub use primitives::transition;

pub use icon::{Icon, IconSource};
pub use metrics::{CollapseBehavior, Metrics, Typography};
pub use palette::{AccentColor, Palette, Scheme};
pub use semantic::{Opacity, Role, Semantic, Surface, Swatch};
pub use shadow::{Elevation, Layer, Resolved, Shadows};
pub use system::{DecorationLayout, WindowButton};
pub use theme::Theme;
pub use widget::Element;
