//! # iced-libadwaita
//!
//! libadwaita-style theming and widgets for [iced](https://iced.rs).
//!
//! * [`Theme`]: one theme type implementing every iced widget catalog, with a
//!   named [`Palette`], a [`Semantic`] color layer, [`Metrics`] and
//!   [`Typography`] tokens.
//! * [`style`]: style functions mirroring libadwaita's CSS classes.
//! * [`widget`]: Adw-named widgets (`HeaderBar`, `ActionRow`, `PreferencesGroup`,
//!   `StatusPage`, `AlertDialog`, …).

pub mod animation;
pub mod breakpoints;
pub mod catalog;
pub mod color;
pub mod config;
pub mod icon;
pub mod metrics;
pub mod palette;
pub mod prelude;
pub mod semantic;
pub mod shadow;
pub mod shortcut;
pub mod style;
pub mod system;
pub mod theme;
pub mod widget;
pub mod window;

pub use icon::Icon;
pub use metrics::{CollapseBehavior, Metrics, Typography};
pub use palette::{AccentColor, Palette, Scheme};
pub use shadow::{Elevation, Layer, Resolved, Shadows};
pub use semantic::{Opacity, Role, Semantic, Surface, Swatch};
pub use system::{DecorationLayout, WindowButton};
pub use theme::Theme;
pub use widget::Element;
