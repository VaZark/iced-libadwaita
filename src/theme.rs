//! [`Theme`]: the single theme type every widget in this crate is generic over.
//!
//! It implements all of iced's widget `Catalog` traits, so stock iced widgets
//! (`iced::widget::button`, `text_input`, …) pick up libadwaita styling too.

use crate::metrics::{Metrics, Typography};
use crate::palette::{AccentColor, Palette, Scheme};
use crate::semantic::Semantic;
use crate::style;
use iced::widget::{
    button, checkbox, combo_box, container, pane_grid, pick_list, progress_bar, radio,
    rule, scrollable, slider, svg, text, text_editor, text_input, toggler,
};
use iced::widget::overlay::menu;

/// A libadwaita-flavoured iced theme.
///
/// Holds the named [`Palette`], the derived [`Semantic`] layer, and the
/// [`Metrics`] / [`Typography`] tokens.
#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub scheme: Scheme,
    pub palette: Palette,
    pub semantic: Semantic,
    pub metrics: Metrics,
    pub typography: Typography,
    name: String,
    iced: iced::Theme,
}

impl Theme {
    pub fn new(scheme: Scheme) -> Self {
        Self::from_palette(scheme, Palette::new(scheme))
    }

    pub fn light() -> Self {
        Self::new(Scheme::Light)
    }

    pub fn dark() -> Self {
        Self::new(Scheme::Dark)
    }

    /// Builds a theme from a custom palette; the semantic layer is derived.
    pub fn from_palette(scheme: Scheme, palette: Palette) -> Self {
        let semantic = Semantic::from_palette(&palette, scheme);
        let name = match scheme {
            Scheme::Light => "Adwaita Light",
            Scheme::Dark => "Adwaita Dark",
        }
        .to_string();
        Self {
            scheme,
            iced: build_iced(&name, &semantic),
            palette,
            semantic,
            metrics: Metrics::default(),
            typography: Typography::default(),
            name,
        }
    }

    /// Switches the system accent color.
    pub fn accent(self, accent: AccentColor) -> Self {
        let scheme = self.scheme;
        self.with_palette(|p| *p = p.with_accent(accent, scheme))
    }

    /// Edits the palette. The semantic layer is re-derived afterwards, so
    /// apply [`Theme::with_semantic`] edits *after* this.
    pub fn with_palette(mut self, f: impl FnOnce(&mut Palette)) -> Self {
        f(&mut self.palette);
        self.semantic = Semantic::from_palette(&self.palette, self.scheme);
        self.iced = build_iced(&self.name, &self.semantic);
        self
    }

    /// Edits semantic tokens directly (card color, opacities, …).
    pub fn with_semantic(mut self, f: impl FnOnce(&mut Semantic)) -> Self {
        f(&mut self.semantic);
        self.iced = build_iced(&self.name, &self.semantic);
        self
    }

    /// Sets layout tokens. Radii are clamped to the HIG maxima
    /// ([`Metrics::MAX_RADIUS_M`] etc.).
    pub fn with_metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = metrics.clamped();
        self
    }

    pub fn with_typography(mut self, typography: Typography) -> Self {
        self.typography = typography;
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self.iced = build_iced(&self.name, &self.semantic);
        self
    }

    pub fn is_dark(&self) -> bool {
        self.scheme.is_dark()
    }

    /// The equivalent plain `iced::Theme`, for interop with code that needs one.
    pub fn as_iced(&self) -> &iced::Theme {
        &self.iced
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}

fn build_iced(name: &str, s: &Semantic) -> iced::Theme {
    iced::Theme::custom(
        name.to_string(),
        iced::theme::Palette {
            background: s.window.bg,
            text: s.window.fg,
            primary: s.accent.bg,
            success: s.success.bg,
            warning: s.warning.bg,
            danger: s.destructive.bg,
        },
    )
}

impl iced::theme::Base for Theme {
    fn default(preference: iced::theme::Mode) -> Self {
        match preference {
            iced::theme::Mode::Dark => Self::dark(),
            _ => Self::light(),
        }
    }

    fn mode(&self) -> iced::theme::Mode {
        match self.scheme {
            Scheme::Light => iced::theme::Mode::Light,
            Scheme::Dark => iced::theme::Mode::Dark,
        }
    }

    fn base(&self) -> iced::theme::Style {
        iced::theme::Style {
            background_color: self.semantic.window.bg,
            text_color: self.semantic.window.fg,
        }
    }

    fn palette(&self) -> Option<iced::theme::Palette> {
        Some(self.iced.palette())
    }

    fn name(&self) -> &str {
        &self.name
    }
}

// ---------------------------------------------------------------------------
// Catalogs. Each uses iced's `StyleFn` so `.style(|theme, status| ..)` works
// exactly like with `iced::Theme`.
// ---------------------------------------------------------------------------

impl text::Catalog for Theme {
    type Class<'a> = text::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|_| text::Style::default())
    }

    fn style(&self, class: &Self::Class<'_>) -> text::Style {
        class(self)
    }
}

impl container::Catalog for Theme {
    type Class<'a> = container::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style::container::transparent)
    }

    fn style(&self, class: &Self::Class<'_>) -> container::Style {
        class(self)
    }
}

impl button::Catalog for Theme {
    type Class<'a> = button::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style::button::default)
    }

    fn style(&self, class: &Self::Class<'_>, status: button::Status) -> button::Style {
        class(self, status)
    }
}

impl text_input::Catalog for Theme {
    type Class<'a> = text_input::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style::text_input::default)
    }

    fn style(&self, class: &Self::Class<'_>, status: text_input::Status) -> text_input::Style {
        class(self, status)
    }
}

impl checkbox::Catalog for Theme {
    type Class<'a> = checkbox::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style::checkbox::default)
    }

    fn style(&self, class: &Self::Class<'_>, status: checkbox::Status) -> checkbox::Style {
        class(self, status)
    }
}

impl radio::Catalog for Theme {
    type Class<'a> = radio::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style::radio::default)
    }

    fn style(&self, class: &Self::Class<'_>, status: radio::Status) -> radio::Style {
        class(self, status)
    }
}

impl toggler::Catalog for Theme {
    type Class<'a> = toggler::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style::toggler::default)
    }

    fn style(&self, class: &Self::Class<'_>, status: toggler::Status) -> toggler::Style {
        class(self, status)
    }
}

impl slider::Catalog for Theme {
    type Class<'a> = slider::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style::slider::default)
    }

    fn style(&self, class: &Self::Class<'_>, status: slider::Status) -> slider::Style {
        class(self, status)
    }
}

impl progress_bar::Catalog for Theme {
    type Class<'a> = progress_bar::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style::progress_bar::default)
    }

    fn style(&self, class: &Self::Class<'_>) -> progress_bar::Style {
        class(self)
    }
}

impl rule::Catalog for Theme {
    type Class<'a> = rule::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style::rule::default)
    }

    fn style(&self, class: &Self::Class<'_>) -> rule::Style {
        class(self)
    }
}

impl scrollable::Catalog for Theme {
    type Class<'a> = scrollable::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style::scrollable::default)
    }

    fn style(&self, class: &Self::Class<'_>, status: scrollable::Status) -> scrollable::Style {
        class(self, status)
    }
}

impl pick_list::Catalog for Theme {
    type Class<'a> = pick_list::StyleFn<'a, Self>;

    fn default<'a>() -> <Self as pick_list::Catalog>::Class<'a> {
        Box::new(style::pick_list::default)
    }

    fn style(&self, class: &<Self as pick_list::Catalog>::Class<'_>, status: pick_list::Status) -> pick_list::Style {
        class(self, status)
    }
}

impl menu::Catalog for Theme {
    type Class<'a> = menu::StyleFn<'a, Self>;

    fn default<'a>() -> <Self as menu::Catalog>::Class<'a> {
        Box::new(style::menu::default)
    }

    fn style(&self, class: &<Self as menu::Catalog>::Class<'_>) -> menu::Style {
        class(self)
    }
}

impl combo_box::Catalog for Theme {}

// Widgets without bespoke libadwaita styling delegate to iced's own theme,
// built from this theme's palette, so they stay readable and consistent.

impl text_editor::Catalog for Theme {
    type Class<'a> = text_editor::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(style::text_editor::default)
    }

    fn style(&self, class: &Self::Class<'_>, status: text_editor::Status) -> text_editor::Style {
        class(self, status)
    }
}

impl pane_grid::Catalog for Theme {
    type Class<'a> = pane_grid::StyleFn<'a, Self>;

    fn default<'a>() -> <Self as pane_grid::Catalog>::Class<'a> {
        Box::new(|t| pane_grid::default(t.as_iced()))
    }

    fn style(&self, class: &<Self as pane_grid::Catalog>::Class<'_>) -> pane_grid::Style {
        class(self)
    }
}

impl svg::Catalog for Theme {
    type Class<'a> = svg::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(|t, _| svg::Style {
            color: Some(t.semantic.window.fg),
        })
    }

    fn style(&self, class: &Self::Class<'_>, status: svg::Status) -> svg::Style {
        class(self, status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Metrics;
    use iced::Color;

    #[test]
    fn radii_are_capped_at_hig_values() {
        let t = Theme::light().with_metrics(Metrics {
            radius_s: 40.0,
            radius_m: 40.0,
            radius_l: 40.0,
            ..Metrics::default()
        });
        assert_eq!(t.metrics.radius_s, Metrics::MAX_RADIUS_S);
        assert_eq!(t.metrics.radius_m, Metrics::MAX_RADIUS_M);
        assert_eq!(t.metrics.radius_l, Metrics::MAX_RADIUS_L);
        // Smaller values pass through, and the pill radius is untouched.
        let t = Theme::light().with_metrics(Metrics { radius_m: 2.0, ..Metrics::default() });
        assert_eq!(t.metrics.radius_m, 2.0);
        assert_eq!(t.metrics.radius_pill, Metrics::default().radius_pill);
    }

    #[test]
    fn accent_flows_into_semantic_layer() {
        let t = Theme::dark().accent(AccentColor::Green);
        assert_eq!(t.semantic.accent.bg, AccentColor::Green.to_color());
        assert_eq!(t.semantic.accent.color, AccentColor::Green.to_standalone_color(Scheme::Dark));
    }

    #[test]
    fn semantic_override_survives_and_palette_edit_rederives() {
        let c = Color::from_rgb(0.1, 0.2, 0.3);
        let t = Theme::light().with_semantic(|s| s.card.bg = c);
        assert_eq!(t.semantic.card.bg, c);
        assert_eq!(t.palette.card_bg_color, Palette::light().card_bg_color);
        let t = t.with_palette(|p| p.card_bg_color = Color::BLACK);
        assert_eq!(t.semantic.card.bg, Color::BLACK);
    }

    #[test]
    fn light_and_dark_differ() {
        assert_ne!(Theme::light().semantic.window.bg, Theme::dark().semantic.window.bg);
        assert!(Theme::dark().is_dark());
    }
}
