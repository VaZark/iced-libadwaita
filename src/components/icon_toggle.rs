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

/// Owned presentation overrides; default values preserve the component defaults.
#[derive(Clone, Default)]
pub struct IconToggleButtonProps {
    pub tooltip_props: super::tooltip::TooltipProps,
    pub button_props: super::props::ButtonProps,
}
impl IconToggleButtonProps {
    pub fn tooltip_props(mut self, props: super::tooltip::TooltipProps) -> Self {
        self.tooltip_props = props;
        self
    }
    pub fn button_props(mut self, props: super::props::ButtonProps) -> Self {
        self.button_props = props;
        self
    }
    pub fn apply<'a, M: 'a>(&self, mut widget: IconToggleButton<'a, M>) -> IconToggleButton<'a, M> {
        widget.button_props.merge(&self.button_props);
        widget.tooltip_props.merge(&self.tooltip_props);
        widget
    }
}

#[cfg(test)]
mod aggregate_merge_tests {
    use super::*;
    use crate::components::{ButtonProps, TextProps, TooltipProps};

    #[test]
    fn nested_tooltip_fields_and_callbacks_survive_partial_overrides() {
        let tooltip = TooltipProps::default()
            .gap(9.0)
            .padding(7)
            .style(|_| iced::widget::container::Style::default())
            .label_props(TextProps::default().width(123).size(19));
        let widget = IconToggleButton::<bool>::new(crate::Icon::Search, true)
            .tooltip("Retained")
            .tooltip_props(tooltip.clone())
            .button_props(ButtonProps::default().width(83).clip(true));
        let widget = IconToggleButtonProps::default().apply(widget);
        let widget = IconToggleButtonProps::default()
            .button_props(ButtonProps::default().clip(false))
            .tooltip_props(
                TooltipProps::default()
                    .gap(0.0)
                    .label_props(TextProps::default().size(21)),
            )
            .apply(widget);
        assert_eq!(widget.button_props.width, Some(iced::Length::Fixed(83.0)));
        assert_eq!(widget.button_props.clip, Some(false));
        assert_eq!(widget.tooltip_props.gap, Some(0.0));
        assert_eq!(widget.tooltip_props.padding, tooltip.padding);
        assert_eq!(
            widget.tooltip_props.label_props.width,
            tooltip.label_props.width
        );
        assert_eq!(
            widget.tooltip_props.label_props.size,
            Some(iced::Pixels(21.0))
        );
        assert!(std::sync::Arc::ptr_eq(
            widget.tooltip_props.container_style.as_ref().unwrap(),
            tooltip.container_style.as_ref().unwrap()
        ));
        assert_eq!(widget.tooltip.as_deref(), Some("Retained"));
        assert!(widget.active);
    }
}

/// See the [module](self).
pub struct IconToggleButton<'a, M> {
    tooltip_props: super::tooltip::TooltipProps,
    icon: IconSource,
    active: bool,
    size: f32,
    disabled: bool,
    tooltip: Option<String>,
    on_toggle: Option<Rc<dyn Fn(bool) -> M + 'a>>,
    button_props: super::props::ButtonProps,
}

impl<'a, M: 'a> IconToggleButton<'a, M> {
    pub fn tooltip_props(mut self, props: super::tooltip::TooltipProps) -> Self {
        self.tooltip_props = props;
        self
    }
    pub fn button_props(mut self, props: super::props::ButtonProps) -> Self {
        self.button_props = props;
        self
    }

    pub fn new(icon: impl Into<IconSource>, active: bool) -> Self {
        Self {
            tooltip_props: Default::default(),
            button_props: super::props::ButtonProps::default(),
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
            b.button_props
                .apply(suggested_button(icon).on_press_maybe(press))
                .into()
        } else {
            b.button_props
                .apply(flat_button(icon).on_press_maybe(press))
                .into()
        };
        match b.tooltip {
            Some(t) => super::tooltip::tooltip_with_props(
                button,
                t,
                super::TooltipPosition::Bottom,
                b.tooltip_props,
            ),
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
