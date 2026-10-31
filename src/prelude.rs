//! One import for building a layout: every component, the theme types, and the layout
//! primitives (`row`, `column`, `container`, `text`, `space`, `responsive`).
//!
//! ```ignore
//! use iced_libadwaita::prelude::*;
//!
//! fn view(&self) -> Element<'_, Msg> {
//!     toolbar_view(
//!         page(
//!             preferences_group()
//!                 .title("General")
//!                 .push(SwitchRow::new("Dark Style", self.dark, Msg::Dark)),
//!         ),
//!     )
//!     .top_bar(header_bar().title("Settings"))
//!     .into()
//! }
//! ```

pub use crate::widget::typography::{
    self, body, caption, heading, title_1, title_2, title_3, title_4,
};
pub use crate::widget::*;
pub use crate::{
    AccentColor, CollapseBehavior, Icon, Metrics, Role, Scheme, Shadows, Theme, Typography, config,
};
pub use iced::widget::{column, container, responsive, row, space, text};
