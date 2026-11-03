//! Style functions, the equivalent of libadwaita's CSS classes.
//!
//! Each submodule is named after the iced widget it styles and exposes plain
//! `fn(&Theme[, Status]) -> Style` functions that plug straight into
//! `.style(..)`:
//!
//! ```ignore
//! button("Save").style(iad::style::button::suggested)      // .suggested-action
//! button("Delete").style(iad::style::button::destructive)  // .destructive-action
//! container(x).style(iad::style::container::card)          // .card
//! text("Hint").style(iad::style::text::dim)                // .dim-label
//! ```

pub mod button;
pub mod container;
mod controls;
pub mod text;

pub use controls::{
    checkbox, menu, pick_list, progress_bar, radio, rule, scrollable, slider, text_editor,
    text_input, toggler,
};
