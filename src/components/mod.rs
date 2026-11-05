//! Components: themed, standalone UI elements that carry no information on their own.
//!
//! Each pulls [primitives](crate::primitives) (rules) and [binds](crate::binds) (event glue) and
//! composes them with its look. Widgets and third-party widgets are built from these.
//!
//! * [`chart`]: the drawing pieces of the chart widgets (axis, legend, tooltip, series colors).

pub(crate) mod avatar;
pub(crate) mod badge;
pub(crate) mod button;
pub mod chart;
pub(crate) mod controls;
pub(crate) mod divider;
pub(crate) mod elevated;
pub(crate) mod entry;
pub(crate) mod icon_toggle;
pub(crate) mod search_bar;
pub(crate) mod sections;
pub(crate) mod spectrum;
pub(crate) mod spinner;
pub(crate) mod switch;
pub(crate) mod tooltip;
pub mod typography;

pub use avatar::{Avatar, AvatarShape, avatar};
pub use badge::{Badge, Chip, ChipSize, RoundBadge};
pub use button::{
    ButtonContent, SuggestedCircularButton, ToggleGroup, ToggleItem, circular_button,
    destructive_button, flat_button, icon_button, pill_button, progress_bar, raised_button,
    suggested_button, suggested_circular_button, suggested_pill_button, toggle_group,
};
pub use controls::{
    check_button, color_swatch, link_button, radio_button, scale, status_swatch, tag,
};
pub use divider::{Divider, DividerTone, DividerVariant, LabelPosition, divider};
pub use elevated::{Elevated, card_shadow};
pub use entry::{Entry, EntryState, TextView, entry, password_entry, text_view};
pub use iced::widget::text_editor::{Action as TextAction, Content as TextContent};
pub use icon_toggle::IconToggleButton;
pub use search_bar::{SearchBar, search_bar};
pub use sections::{SectionButton, SectionIcon, section_button, section_icon, section_size};
pub use spectrum::{SpectrumBars, spectrum_bars};
pub use spinner::{Spinner, spinner};
pub use switch::{Switch, switch};
pub use tooltip::{TooltipPosition, tooltip};
