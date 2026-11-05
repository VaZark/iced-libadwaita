//! `IconToggleButton`: one icon that is on or off. Off it is a flat icon button; on it is the
//! accent (suggested) one. Shuffle, repeat, a favorite, a sidebar toggle.
//!
//! `toggle_group` picks one of several and `Switch` is a labeled setting; this is the single
//! icon toggle.
//!
//! ```ignore
//! IconToggleButton::new(Icon::Shuffle, self.shuffle).on_toggle(Msg::Shuffle)   // gets the new state
//! ```

use crate::Element;
use crate::components::{flat_button, suggested_button};
use crate::{IconSource, Metrics};
use std::rc::Rc;

/// See the [module](self).
pub struct IconToggleButton<'a, M> {
    icon: IconSource,
    active: bool,
    size: f32,
    disabled: bool,
    tooltip: Option<String>,
    on_toggle: Option<Rc<dyn Fn(bool) -> M + 'a>>,
}

impl<'a, M: 'a> IconToggleButton<'a, M> {
    pub fn new(icon: impl Into<IconSource>, active: bool) -> Self {
        Self {
            icon: icon.into(),
            active,
            size: Metrics::desktop().icon_size,
            disabled: false,
            tooltip: None,
            on_toggle: None,
        }
    }
    /// The icon size in px (default the metrics' icon size).
    pub fn size(mut self, px: f32) -> Self {
        self.size = px;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// A hint shown on hover.
    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        self.tooltip = Some(text.into());
        self
    }
    /// Pressed: the state the button would take.
    pub fn on_toggle(mut self, f: impl Fn(bool) -> M + 'a) -> Self {
        self.on_toggle = Some(Rc::new(f));
        self
    }
}

impl<'a, M: Clone + 'a> From<IconToggleButton<'a, M>> for Element<'a, M> {
    fn from(b: IconToggleButton<'a, M>) -> Self {
        let icon = b.icon.view(b.size);
        let press = if b.disabled {
            None
        } else {
            b.on_toggle.as_ref().map(|f| f(!b.active))
        };
        let button: Element<'a, M> = if b.active {
            suggested_button(icon).on_press_maybe(press).into()
        } else {
            flat_button(icon).on_press_maybe(press).into()
        };
        match b.tooltip {
            Some(t) => crate::components::tooltip(button, t, super::TooltipPosition::Bottom),
            None => button,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Icon;

    #[test]
    fn a_toggle_asks_for_the_opposite_state() {
        let on = IconToggleButton::new(Icon::Star, true).on_toggle(|s| s);
        assert_eq!(on.on_toggle.as_ref().map(|f| f(!on.active)), Some(false));
        let off = IconToggleButton::new(Icon::Star, false).on_toggle(|s| s);
        assert_eq!(off.on_toggle.as_ref().map(|f| f(!off.active)), Some(true));
    }
}
