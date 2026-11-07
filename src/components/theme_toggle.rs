//! A controlled System → Light → Dark → System appearance button.
use crate::{AppearanceIcon, Element, Metrics, Scheme};
use iced::{
    Length,
    widget::{button, container},
};

/// Application appearance preference. System is the default and clears the app override.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}
impl ThemeMode {
    pub fn next(self) -> Self {
        match self {
            Self::System => Self::Light,
            Self::Light => Self::Dark,
            Self::Dark => Self::System,
        }
    }
    pub fn scheme_override(self) -> Option<Scheme> {
        match self {
            Self::System => None,
            Self::Light => Some(Scheme::Light),
            Self::Dark => Some(Scheme::Dark),
        }
    }
    pub fn icon(self) -> AppearanceIcon {
        match self {
            Self::System => AppearanceIcon::System,
            Self::Light => AppearanceIcon::Light,
            Self::Dark => AppearanceIcon::Dark,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }
}
impl From<Option<Scheme>> for ThemeMode {
    fn from(scheme: Option<Scheme>) -> Self {
        match scheme {
            None => Self::System,
            Some(Scheme::Light) => Self::Light,
            Some(Scheme::Dark) => Self::Dark,
        }
    }
}

/// A single icon button displaying the current preference and cycling to the next.
/// Handle its message with `utils::set_scheme_override(mode.scheme_override())`.
pub struct ThemeToggle<'a, M> {
    mode: ThemeMode,
    on_toggle: Option<Box<dyn Fn(ThemeMode) -> M + 'a>>,
    disabled: bool,
    metrics: Metrics,
}
impl<'a, M> ThemeToggle<'a, M> {
    pub fn new(mode: ThemeMode) -> Self {
        Self {
            mode,
            on_toggle: None,
            disabled: false,
            metrics: Metrics::desktop(),
        }
    }
    pub fn on_toggle(mut self, on_toggle: impl Fn(ThemeMode) -> M + 'a) -> Self {
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
    fn press(&self) -> Option<M> {
        if self.disabled {
            None
        } else {
            self.on_toggle.as_ref().map(|f| f(self.mode.next()))
        }
    }
}
impl<'a, M: Clone + 'a> From<ThemeToggle<'a, M>> for Element<'a, M> {
    fn from(toggle: ThemeToggle<'a, M>) -> Self {
        let content = container(toggle.mode.icon().view(toggle.metrics.icon_size + 2.0))
            .center(Length::Fixed(toggle.metrics.button_min_height));
        let control = button(content)
            .padding(0)
            .style(crate::styles::button::flat)
            .on_press_maybe(toggle.press());
        super::tooltip(
            control,
            format!(
                "Appearance: {} · Switch to {}",
                toggle.mode.label(),
                toggle.mode.next().label()
            ),
            super::TooltipPosition::Bottom,
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_to_system_and_cycles_all_three_preferences() {
        assert_eq!(ThemeMode::default(), ThemeMode::System);
        for (current, next) in [
            (ThemeMode::System, ThemeMode::Light),
            (ThemeMode::Light, ThemeMode::Dark),
            (ThemeMode::Dark, ThemeMode::System),
        ] {
            let toggle = ThemeToggle::new(current).on_toggle(|mode| mode);
            assert_eq!(toggle.press(), Some(next));
            assert_eq!(toggle.disabled(true).press(), None);
            assert_eq!(ThemeToggle::<ThemeMode>::new(current).press(), None);
            assert_eq!(ThemeMode::from(current.scheme_override()), current);
        }
    }
    #[test]
    fn appearance_artwork_fits_and_renders_at_header_sizes() {
        for mode in [ThemeMode::System, ThemeMode::Light, ThemeMode::Dark] {
            let tree =
                resvg::usvg::Tree::from_str(mode.icon().source(), &Default::default()).unwrap();
            let bounds = tree.root().abs_stroke_bounding_box();
            assert!(
                bounds.left() >= 0.0
                    && bounds.top() >= 0.0
                    && bounds.right() <= 16.0
                    && bounds.bottom() <= 16.0
            );
            for size in [16, 18, 24, 32] {
                let mut pixels = resvg::tiny_skia::Pixmap::new(size, size).unwrap();
                resvg::render(
                    &tree,
                    resvg::tiny_skia::Transform::from_scale(size as f32 / 16.0, size as f32 / 16.0),
                    &mut pixels.as_mut(),
                );
                assert!(pixels.data().chunks_exact(4).any(|p| p[3] > 0));
            }
        }
    }
}
