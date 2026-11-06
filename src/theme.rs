//! [`Theme`]: the single theme type every widget in this crate is generic over.
//!
//! It implements all of iced's widget `Catalog` traits, so stock iced widgets
//! (`iced::widget::button`, `text_input`, …) pick up libadwaita styling too.

use crate::metrics::{Metrics, Typography};
use crate::palette::{AccentColor, Palette, Scheme};
use crate::semantic::Semantic;
use crate::shadow::Shadows;
use crate::styles;
use iced::widget::overlay::menu;
use iced::widget::{
    button, checkbox, combo_box, container, pane_grid, pick_list, progress_bar, radio, rule,
    scrollable, slider, svg, text, text_editor, text_input, toggler,
};

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
    pub shadows: Shadows,
    /// Paint nothing behind the app root so a rounded client-side frame can
    /// show transparent corners (the default). Disable for server-side
    /// decorations or non-transparent windows.
    pub transparent_window: bool,
    /// Header bars without a color of their own (`%headerbar-flat`): they take the
    /// color behind them, so window and header read as one continuous surface (the
    /// Nautilus look). Default `false` (libadwaita's `--headerbar-bg-color`). A
    /// bar's own `.flat(..)` overrides this; dialog headers are always flat.
    pub flat_headerbars: bool,
    /// Choose the text color on colored fills (accent, destructive, …) by contrast, like CSS
    /// `color-contrast()`. Default `true`; `false` keeps the palette's fixed `*_fg_color`s.
    pub auto_contrast: bool,
    /// How see-through the surfaces are: 0 opaque (the default), 1 fully clear
    /// ([`Semantic::apply_transparency`]). It needs a transparent window.
    pub transparency: f32,
    /// A color the surfaces are tinted with, its alpha the strength; opt-in, `None` (the default)
    /// is no tint ([`Semantic::apply_tint`]). The surfaces keep their own alpha.
    pub tint: Option<iced::Color>,
    /// The app's theme tokens (see [`theme_config`](crate::theme_config)): its own colors and sizes,
    /// readable from a style function through [`token_color`](Self::token_color) and friends.
    /// Empty until [`with_theme_config`](Self::with_theme_config).
    pub tokens: std::sync::Arc<crate::theme_config::Tokens>,
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

    /// Builds the theme the **desktop config** describes: color scheme and accent (`system`
    /// reads GSettings), density, text scale, shadows, flat header bars, and a transparent
    /// window base when decorations are client-side. This is the intended way to create an
    /// app's theme; building one by hand is an opt-out of the shared desktop behavior.
    pub fn from_config(c: &crate::config::Config) -> Self {
        use crate::config::Pick;
        let scheme = crate::utils::effective_scheme(c, crate::system::color_scheme());
        let mut t = Self::new(scheme);
        let accent = match c.accent {
            Pick::Fixed(a) => Some(a),
            Pick::System => crate::system::accent_color(),
        };
        if let Some(a) = accent {
            t = t.accent(a);
        }
        let shadows = if c.shadows {
            Shadows::default()
                .with_scale(c.shadow_scale)
                .with_intensity(c.shadow_intensity)
        } else {
            Shadows::none()
        };
        t.with_metrics(c.metrics())
            .with_typography(Typography::with_base(
                Typography::default().base * c.text_scale,
            ))
            .with_shadows(shadows)
            .with_flat_headerbars(c.flat_headerbars)
            .with_auto_contrast(c.auto_contrast)
            .with_tint(c.tint)
            .with_transparency(c.transparency)
            .with_transparent_window(c.decorations == crate::config::DecorationMode::Csd)
    }

    pub fn dark() -> Self {
        Self::new(Scheme::Dark)
    }

    /// Builds a theme from a custom palette; the semantic layer is derived.
    pub fn from_palette(scheme: Scheme, palette: Palette) -> Self {
        let mut semantic = Semantic::from_palette(&palette, scheme);
        semantic.apply_contrast();
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
            shadows: Shadows::default(),
            transparent_window: true,
            flat_headerbars: true,
            auto_contrast: true,
            transparency: 0.0,
            tint: None,
            tokens: Default::default(),
            name,
        }
    }

    /// Applies the app's [theme config](crate::theme_config): the palette, metric and typography
    /// tokens that were set (hardcoded, in the app theme, or in the toolkit theme), over what the
    /// desktop config gave. Radii are **not** clamped to the HIG maxima: a different vibe may be
    /// rounder or squarer. The theme also carries every token, so a style function can read the
    /// app's own with [`token_color`](Self::token_color).
    #[must_use]
    pub fn with_theme_config(mut self, cfg: &crate::theme_config::ThemeConfig) -> Self {
        let scheme = self.scheme;
        self = self.with_palette(|p| cfg.apply_to_palette(p, scheme));
        let mut metrics = self.metrics;
        cfg.apply_to_metrics(&mut metrics);
        self.metrics = metrics;
        if let Some(t) = cfg.typography() {
            self.typography = t;
        }
        self.tokens = std::sync::Arc::new(cfg.tokens());
        self
    }

    /// An app theme token that is a color (see [`with_theme_config`](Self::with_theme_config)).
    pub fn token_color(&self, section: &str, key: &str) -> Option<iced::Color> {
        self.tokens.color(section, key)
    }
    /// An app theme token that is a number: a size, a padding, a factor.
    pub fn token_number(&self, section: &str, key: &str) -> Option<f32> {
        self.tokens.number(section, key)
    }
    /// An app theme token that is a bool.
    pub fn token_bool(&self, section: &str, key: &str) -> Option<bool> {
        self.tokens.bool(section, key)
    }
    /// An app theme token that is text.
    pub fn token_text(&self, section: &str, key: &str) -> Option<&str> {
        self.tokens.text(section, key)
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
        self.semantic.apply_tint(self.tint);
        if self.auto_contrast {
            self.semantic.apply_contrast();
        }
        self.semantic.apply_transparency(self.transparency);
        self.iced = build_iced(&self.name, &self.semantic);
        self
    }

    /// Turns CSS-`color-contrast()` text selection on colored fills on or off. Off restores
    /// the palette's fixed foregrounds (libadwaita's white).
    pub fn with_auto_contrast(mut self, on: bool) -> Self {
        self.auto_contrast = on;
        self.with_palette(|_| {})
    }

    /// Makes the surfaces translucent (0 opaque .. 1 clear); see [`Semantic::apply_transparency`].
    pub fn with_transparency(mut self, transparency: f32) -> Self {
        self.transparency = transparency.clamp(0.0, 1.0);
        self.with_palette(|_| {})
    }

    /// Tints the surfaces with a color, its alpha the strength; see [`Semantic::apply_tint`].
    pub fn with_tint(mut self, tint: Option<iced::Color>) -> Self {
        self.tint = tint;
        self.with_palette(|_| {})
    }

    /// Edits semantic tokens directly (card color, opacities, …).
    pub fn with_semantic(mut self, f: impl FnOnce(&mut Semantic)) -> Self {
        let previous = self.semantic.clone();
        f(&mut self.semantic);
        if self.auto_contrast {
            // Recompute defaults after background edits, preserving explicit foreground edits.
            let backdrop = iced::Color {
                a: 1.0,
                ..self.semantic.window.bg
            };
            for (surface, old) in [
                (&mut self.semantic.window, previous.window),
                (&mut self.semantic.view, previous.view),
                (&mut self.semantic.headerbar, previous.headerbar),
                (&mut self.semantic.card, previous.card),
                (&mut self.semantic.sidebar, previous.sidebar),
                (
                    &mut self.semantic.secondary_sidebar,
                    previous.secondary_sidebar,
                ),
                (&mut self.semantic.thumbnail, previous.thumbnail),
                (&mut self.semantic.overview, previous.overview),
                (&mut self.semantic.active_toggle, previous.active_toggle),
                (&mut self.semantic.dialog, previous.dialog),
                (&mut self.semantic.popover, previous.popover),
                (&mut self.semantic.osd, previous.osd),
            ] {
                if surface.fg == old.fg {
                    surface.fg = crate::color::ensure_contrast(
                        surface.fg,
                        crate::color::composite_over(surface.bg, backdrop),
                        4.5,
                    );
                }
            }
            for (swatch, old) in [
                (&mut self.semantic.accent, previous.accent),
                (&mut self.semantic.destructive, previous.destructive),
                (&mut self.semantic.success, previous.success),
                (&mut self.semantic.warning, previous.warning),
                (&mut self.semantic.error, previous.error),
            ] {
                if swatch.fg == old.fg && swatch.bg != old.bg {
                    swatch.fg = crate::color::readable_on(swatch.bg);
                }
            }
        }
        self.iced = build_iced(&self.name, &self.semantic);
        self
    }

    /// Sets layout tokens. Radii are clamped to the HIG maxima
    /// ([`Metrics::MAX_RADIUS_M`] etc.).
    pub fn with_metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = metrics.clamped();
        self
    }

    /// Replaces the shadow settings ([`Shadows::none()`] for a flat look).
    pub fn with_shadows(mut self, shadows: Shadows) -> Self {
        self.shadows = shadows;
        self
    }

    /// Whether the area behind the app root is transparent (default `true`,
    /// needed for rounded window corners). Use `false` with server-side
    /// decorations: the window background is then painted opaque.
    pub fn with_transparent_window(mut self, transparent: bool) -> Self {
        self.transparent_window = transparent;
        self
    }

    /// Makes header bars continuous with the content (see [`Theme::flat_headerbars`]).
    pub fn with_flat_headerbars(mut self, flat: bool) -> Self {
        self.flat_headerbars = flat;
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

    /// `light` when the theme is light, `dark` when it is dark (like CSS `light-dark()`), so one
    /// style function serves both modes: `t.light_dark(Color::WHITE, Color::BLACK)`.
    pub fn light_dark<T>(&self, light: T, dark: T) -> T {
        self.scheme.light_dark(light, dark)
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
            background_color: if self.transparent_window || self.transparency > 0.0 {
                iced::Color::TRANSPARENT
            } else {
                self.semantic.window.bg
            },
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
        Box::new(styles::container::transparent)
    }

    fn style(&self, class: &Self::Class<'_>) -> container::Style {
        class(self)
    }
}

impl button::Catalog for Theme {
    type Class<'a> = button::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(styles::button::default)
    }

    fn style(&self, class: &Self::Class<'_>, status: button::Status) -> button::Style {
        class(self, status)
    }
}

impl text_input::Catalog for Theme {
    type Class<'a> = text_input::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(styles::text_input::default)
    }

    fn style(&self, class: &Self::Class<'_>, status: text_input::Status) -> text_input::Style {
        class(self, status)
    }
}

impl checkbox::Catalog for Theme {
    type Class<'a> = checkbox::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(styles::checkbox::default)
    }

    fn style(&self, class: &Self::Class<'_>, status: checkbox::Status) -> checkbox::Style {
        class(self, status)
    }
}

impl radio::Catalog for Theme {
    type Class<'a> = radio::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(styles::radio::default)
    }

    fn style(&self, class: &Self::Class<'_>, status: radio::Status) -> radio::Style {
        class(self, status)
    }
}

impl toggler::Catalog for Theme {
    type Class<'a> = toggler::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(styles::toggler::default)
    }

    fn style(&self, class: &Self::Class<'_>, status: toggler::Status) -> toggler::Style {
        class(self, status)
    }
}

impl slider::Catalog for Theme {
    type Class<'a> = slider::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(styles::slider::default)
    }

    fn style(&self, class: &Self::Class<'_>, status: slider::Status) -> slider::Style {
        class(self, status)
    }
}

impl progress_bar::Catalog for Theme {
    type Class<'a> = progress_bar::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(styles::progress_bar::default)
    }

    fn style(&self, class: &Self::Class<'_>) -> progress_bar::Style {
        class(self)
    }
}

impl rule::Catalog for Theme {
    type Class<'a> = rule::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(styles::rule::default)
    }

    fn style(&self, class: &Self::Class<'_>) -> rule::Style {
        class(self)
    }
}

impl scrollable::Catalog for Theme {
    type Class<'a> = scrollable::StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(styles::scrollable::default)
    }

    fn style(&self, class: &Self::Class<'_>, status: scrollable::Status) -> scrollable::Style {
        class(self, status)
    }
}

impl pick_list::Catalog for Theme {
    type Class<'a> = pick_list::StyleFn<'a, Self>;

    fn default<'a>() -> <Self as pick_list::Catalog>::Class<'a> {
        Box::new(styles::pick_list::default)
    }

    fn style(
        &self,
        class: &<Self as pick_list::Catalog>::Class<'_>,
        status: pick_list::Status,
    ) -> pick_list::Style {
        class(self, status)
    }
}

impl menu::Catalog for Theme {
    type Class<'a> = menu::StyleFn<'a, Self>;

    fn default<'a>() -> <Self as menu::Catalog>::Class<'a> {
        Box::new(styles::menu::default)
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
        Box::new(styles::text_editor::default)
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
    #[test]
    fn dark_cards_and_selected_toggles_keep_readable_foregrounds() {
        for theme in [
            Theme::dark(),
            Theme::dark().with_tint(Some(Color::from_rgba(0.2, 0.3, 0.4, 0.5))),
            Theme::dark().with_semantic(|s| s.card.bg = crate::color::white(0.12)),
        ] {
            for surface in [theme.semantic.card, theme.semantic.active_toggle] {
                let effective = crate::color::composite_over(surface.bg, theme.semantic.window.bg);
                assert_eq!(surface.fg, Color::WHITE);
                assert!(crate::color::contrast_ratio(surface.fg, effective) >= 4.5);
            }
        }
    }

    #[test]
    fn background_edits_update_inherited_foregrounds_but_preserve_overrides() {
        let changed = Theme::light().with_semantic(|s| {
            s.headerbar.bg = Color::BLACK;
            s.active_toggle.bg = Color::BLACK;
        });
        for surface in [changed.semantic.headerbar, changed.semantic.active_toggle] {
            assert!(crate::color::contrast_ratio(surface.fg, surface.bg) >= 4.5);
        }
        let explicit = Color::from_rgb(0.1, 0.2, 0.3);
        let changed = Theme::light().with_semantic(|s| {
            s.headerbar.bg = Color::BLACK;
            s.headerbar.fg = explicit;
        });
        assert_eq!(changed.semantic.headerbar.fg, explicit);
    }

    #[test]
    fn light_dark_picks_by_scheme() {
        use crate::Scheme;
        assert_eq!(super::Theme::light().light_dark("l", "d"), "l");
        assert_eq!(super::Theme::new(Scheme::Dark).light_dark("l", "d"), "d");
        assert_eq!(Scheme::Dark.light_dark(1, 2), 2);
    }

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
        let t = Theme::light().with_metrics(Metrics {
            radius_m: 2.0,
            ..Metrics::default()
        });
        assert_eq!(t.metrics.radius_m, 2.0);
        assert_eq!(t.metrics.radius_pill, Metrics::default().radius_pill);
    }

    #[test]
    fn icon_tint_is_opaque_and_readable_in_both_schemes() {
        fn lum(c: iced::Color) -> f32 {
            0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b
        }
        for t in [Theme::light(), Theme::dark()] {
            let s = &t.semantic;
            let on = s.icon_solid(false);
            let off = s.icon_solid(true);
            assert_eq!(on.a, 1.0);
            // The enabled tint is the foreground: far from the background in either scheme
            // (a black default would vanish on the dark one).
            assert!((lum(on) - lum(s.window.bg)).abs() > 0.5);
            // Disabled sits between the background and the foreground.
            assert!((lum(off) - lum(s.window.bg)).abs() < (lum(on) - lum(s.window.bg)).abs());
        }
    }

    #[test]
    fn accent_flows_into_semantic_layer() {
        let t = Theme::dark().accent(AccentColor::Green);
        assert_eq!(t.semantic.accent.bg, AccentColor::Green.to_color());
        assert_eq!(
            t.semantic.accent.color,
            AccentColor::Green.to_standalone_color(Scheme::Dark)
        );
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
        assert_ne!(
            Theme::light().semantic.window.bg,
            Theme::dark().semantic.window.bg
        );
        assert!(Theme::dark().is_dark());
    }
}

#[cfg(test)]
mod shadow_and_scrollbar_tests {
    use super::*;
    use crate::Shadows;
    use iced::widget::scrollable::Status;

    #[test]
    fn shadows_are_a_theme_setting() {
        let on = Theme::light();
        let off = Theme::light().with_shadows(Shadows::none());
        assert!(on.shadows.card().ring.is_some());
        assert!(off.shadows.card().ring.is_none());
        let flat = crate::styles::container::card(&off);
        assert_eq!(flat.shadow.color.a, 0.0);
        assert_eq!(flat.border.width, 0.0);
        let raised = crate::styles::container::card(&on);
        assert!(raised.shadow.blur_radius > 0.0 && raised.border.width == 1.0);
    }

    #[test]
    fn scrollbar_slider_levels_match_adwaita() {
        use iced::Background;
        let t = Theme::light();
        let alpha = |s: Status| match crate::styles::scrollable::default(&t, s)
            .vertical_rail
            .scroller
            .background
        {
            Background::Color(c) => c.a / t.semantic.window.fg.a,
            _ => unreachable!(),
        };
        let rest = Status::Active {
            is_horizontal_scrollbar_disabled: false,
            is_vertical_scrollbar_disabled: false,
        };
        let hover = Status::Hovered {
            is_horizontal_scrollbar_hovered: false,
            is_vertical_scrollbar_hovered: true,
            is_horizontal_scrollbar_disabled: false,
            is_vertical_scrollbar_disabled: false,
        };
        let drag = Status::Dragged {
            is_horizontal_scrollbar_dragged: false,
            is_vertical_scrollbar_dragged: true,
            is_horizontal_scrollbar_disabled: false,
            is_vertical_scrollbar_disabled: false,
        };
        assert!((alpha(rest) - 0.20).abs() < 1e-4);
        assert!((alpha(hover) - 0.40).abs() < 1e-4);
        assert!((alpha(drag) - 0.60).abs() < 1e-4);
        // The trough only shows while hovered.
        assert!(
            crate::styles::scrollable::default(&t, rest)
                .vertical_rail
                .background
                .is_none()
        );
        assert!(
            crate::styles::scrollable::default(&t, hover)
                .vertical_rail
                .background
                .is_some()
        );
    }

    #[test]
    fn radii_follow_adwaita_scss() {
        let m = Metrics::default();
        assert_eq!(
            (m.radius_m, m.radius_l, m.radius_popover, m.radius_window),
            (9.0, 12.0, 15.0, 15.0)
        );
    }
}

#[cfg(test)]
mod header_tests {
    use super::*;

    #[test]
    fn header_bars_follow_libadwaita_unless_made_continuous() {
        assert!(Theme::light().flat_headerbars);
        assert!(!Theme::light().with_flat_headerbars(false).flat_headerbars);
    }

    #[test]
    fn text_on_status_fills_is_chosen_by_contrast() {
        use crate::color::{contrast_ratio, readable_on};
        for accent in AccentColor::ALL {
            for scheme in [Scheme::Light, Scheme::Dark] {
                let t = Theme::new(scheme).accent(accent);
                let a = t.semantic.accent;
                assert_eq!(a.fg, readable_on(a.bg), "{accent:?}");
                assert!(
                    contrast_ratio(a.bg, a.fg) >= 3.0,
                    "{accent:?} {scheme:?} unreadable"
                );
            }
        }
        // A bright fill gets dark text; turning it off restores the palette's white.
        let t = Theme::light().accent(AccentColor::Yellow);
        assert_eq!(t.semantic.accent.fg, crate::color::DARK_TEXT);
        let off = t.with_auto_contrast(false);
        assert_eq!(off.semantic.accent.fg, iced::Color::WHITE);
    }

    #[test]
    fn transparency_scales_surface_alpha_and_keeps_tints_translucent() {
        let solid = Theme::light();
        let glass = Theme::light().with_transparency(0.4);
        let (a, b) = (solid.semantic.window.bg, glass.semantic.window.bg);
        assert_eq!((a.r, a.g, a.b), (b.r, b.g, b.b), "the color is unchanged");
        assert!((b.a - 0.6).abs() < 1e-5);
        // Floating surfaces take half, foregrounds none.
        assert!((glass.semantic.popover.bg.a - 0.8).abs() < 1e-5);
        assert_eq!(glass.semantic.window.fg, solid.semantic.window.fg);
        // A hover or press tint of a translucent fill stays as translucent.
        assert!((glass.semantic.hover_fill(b).a - 0.6).abs() < 1e-5);
        assert!((glass.semantic.active_fill(b).a - 0.6).abs() < 1e-5);
        // Re-deriving the palette keeps the transparency; 0 restores the solid theme.
        assert!(
            (glass
                .clone()
                .accent(crate::AccentColor::Teal)
                .semantic
                .window
                .bg
                .a
                - 0.6)
                .abs()
                < 1e-5
        );
        assert_eq!(glass.with_transparency(0.0).semantic.window.bg, a);
    }

    #[test]
    fn a_tint_moves_the_color_towards_its_own_color_and_keeps_the_alpha() {
        let plain = Theme::light().with_transparency(0.5);
        let red = iced::Color::from_rgba(1.0, 0.0, 0.0, 0.3);
        let tinted = Theme::light().with_transparency(0.5).with_tint(Some(red));
        let (a, b) = (plain.semantic.window.bg, tinted.semantic.window.bg);
        assert_eq!(a.a, b.a, "transparency is kept");
        assert!(b.r > a.r && b.g < a.g, "towards red, not the accent");
        assert_eq!(
            Theme::light().with_tint(None).semantic.window.bg,
            Theme::light().semantic.window.bg
        );
    }
}
