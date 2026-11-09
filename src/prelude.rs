//! One import for building a layout: every component, the theme types, and the layout
//! primitives (`row`, `column`, `container`, `text`, `space`, `responsive`).
//!
//! ```ignore
//! use iced_adaptive_design::prelude::*;
//!
//! fn view(&self) -> Element<'_, Msg> {
//!     toolbar_view(
//!         page(
//!             preferences_group()
//!                 .title("General")
//!                 .push(SwitchEntry::new("Notifications", self.notifications, Msg::Notifications)),
//!         ),
//!     )
//!     .top_bar(header_bar().title("Settings"))
//!     .into()
//! }
//! ```

pub use crate::adaptive::{Adaptive, Hysteresis, Layouts, OutOfRange, adaptive};
pub use crate::binds::*;
pub use crate::breakpoints::WidthClass;
pub use crate::components::chart;
pub use crate::components::typography::{
    self, body, caption, heading, title_1, title_2, title_3, title_4,
};
pub use crate::components::*;
pub use crate::layouts::*;
pub use crate::metas::*;
pub use crate::widgets::*;
pub use crate::{
    AccentColor, CollapseBehavior, Icon, IconSource, Metrics, Role, Scheme, Shadows, Theme,
    Typography, config,
};
pub use iced::widget::{column, container, row, space, text};
