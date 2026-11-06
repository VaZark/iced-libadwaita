//! A controlled light/dark theme toggle for application headers.
//!
//! It displays the current scheme and requests the opposite one when pressed.
//! Apply the requested scheme with [`crate::utils::set_scheme_override`] to override
//! desktop appearance for this process; pass `None` to resume following the system.

use crate::{Element, IconSource, Metrics, Scheme};
use iced::{
    Length,
    widget::{button, container},
};

/// A sun/moon icon toggle with no label or background highlight.
///
/// ```ignore
/// ThemeToggle::new(self.scheme).on_toggle(Msg::Scheme)
/// // In update: utils::set_scheme_override(Some(scheme));
/// ```
pub struct ThemeToggle<'a, M> {
    scheme: Scheme,
    on_toggle: Option<Box<dyn Fn(Scheme) -> M + 'a>>,
    disabled: bool,
    metrics: Metrics,
}

impl<'a, M> ThemeToggle<'a, M> {
    pub fn new(scheme: Scheme) -> Self {
        Self {
            scheme,
            on_toggle: None,
            disabled: false,
            metrics: Metrics::desktop(),
        }
    }
    /// Request the other scheme on activation.
    pub fn on_toggle(mut self, on_toggle: impl Fn(Scheme) -> M + 'a) -> Self {
        self.on_toggle = Some(Box::new(on_toggle));
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = metrics;
        self
    }
    fn requested_scheme(&self) -> Scheme {
        if self.scheme.is_dark() {
            Scheme::Light
        } else {
            Scheme::Dark
        }
    }
    fn press(&self) -> Option<M> {
        if self.disabled {
            None
        } else {
            self.on_toggle.as_ref().map(|f| f(self.requested_scheme()))
        }
    }
}

impl<'a, M: Clone + 'a> From<ThemeToggle<'a, M>> for Element<'a, M> {
    fn from(toggle: ThemeToggle<'a, M>) -> Self {
        let press = toggle.press();
        let dark = toggle.scheme.is_dark();
        let hint = if dark {
            "Switch to light mode"
        } else {
            "Switch to dark mode"
        };
        let icon = IconSource::symbolic(if dark {
            "weather-clear-night"
        } else {
            "weather-clear"
        });
        let content = container(icon.view(toggle.metrics.icon_size + 2.0).weight(0.3))
            .center(Length::Fixed(toggle.metrics.button_min_height));
        let control = button(content)
            .padding(0)
            .style(|theme, status| {
                let mut style = crate::styles::button::flat(theme, status);
                style.background = None;
                style
            })
            .on_press_maybe(press);
        super::tooltip(control, hint, super::TooltipPosition::Bottom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn requests_the_opposite_scheme_and_respects_disabled_and_read_only_states() {
        let light = ThemeToggle::new(Scheme::Light).on_toggle(|s| s);
        assert_eq!(light.press(), Some(Scheme::Dark));
        let dark = ThemeToggle::new(Scheme::Dark).on_toggle(|s| s);
        assert_eq!(dark.press(), Some(Scheme::Light));
        assert_eq!(dark.disabled(true).press(), None);
        assert_eq!(ThemeToggle::<Scheme>::new(Scheme::Light).press(), None);
    }
}
