//! Binds: the glue between iced events and the headless [primitives](crate::primitives).
//!
//! A bind turns an iced event (press, focus, shortcut) into primitive input. It holds no rules.

pub(crate) mod focus;
pub(crate) mod shortcut_scope;

pub use focus::{FocusScope, focus_return};
pub use shortcut_scope::shortcuts;
