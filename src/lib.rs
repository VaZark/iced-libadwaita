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

pub mod color;
pub mod icon;
pub mod metrics;
pub mod palette;
pub mod semantic;
pub mod style;
pub mod theme;
pub mod widget;

pub use icon::Icon;
pub use metrics::{Metrics, Typography};
pub use palette::{AccentColor, Palette, Scheme};
pub use semantic::{Opacity, Role, Semantic, Surface, Swatch};
pub use theme::Theme;
pub use widget::Element;
