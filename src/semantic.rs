//! The semantic color layer.
//!
//! Widgets never read [`Palette`] directly. They ask the theme's [`Semantic`]
//! for roles ("the surface of a card", "the accent swatch", "the border
//! color"). Re-skinning is then a matter of overriding a few semantic tokens
//! without touching every named color:
//!
//! ```
//! # use iced_libadwaita::Theme;
//! # use iced::Color;
//! let theme = Theme::dark().with_semantic(|s| {
//!     s.card.bg = Color::from_rgb8(0x2b, 0x2b, 0x35);
//!     s.opacity.disabled = 0.4;
//! });
//! ```

use crate::color::{alpha, lighten};
use crate::palette::{Palette, Scheme};
use iced::Color;

/// A background / foreground pair (`*_bg_color` + `*_fg_color`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Surface {
    pub bg: Color,
    pub fg: Color,
}

/// A status color in its three libadwaita forms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Swatch {
    /// Fill color for buttons / banners (`*_bg_color`).
    pub bg: Color,
    /// Text on top of `bg` (`*_fg_color`).
    pub fg: Color,
    /// Standalone foreground on a view, e.g. an accent label (`*_color`).
    pub color: Color,
}

impl Swatch {
    pub fn surface(self) -> Surface {
        Surface {
            bg: self.bg,
            fg: self.fg,
        }
    }
}

/// Semantic color roles, the equivalent of libadwaita's style classes
/// `.accent`, `.success`, `.warning`, `.error`, `.destructive-action`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Role {
    /// No special meaning; follows the window foreground.
    #[default]
    Neutral,
    Accent,
    Destructive,
    Success,
    Warning,
    Error,
}

/// Alpha values applied on top of the foreground color to produce control
/// backgrounds and state layers, as libadwaita does with `alpha(currentColor, x)`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Opacity {
    /// Resting background of buttons and entries.
    pub control: f32,
    pub control_hover: f32,
    pub control_active: f32,
    /// Hover/press layer for flat buttons and rows.
    pub flat_hover: f32,
    pub flat_active: f32,
    /// `.dim-label`.
    pub dim: f32,
    /// Disabled widgets.
    pub disabled: f32,
    /// Switch / progress trough.
    pub trough: f32,
    /// Border drawn around cards, entries on focus-less state, etc.
    pub border: f32,
    /// Scrollbar slider.
    pub scroller: f32,
}

impl Default for Opacity {
    fn default() -> Self {
        Self {
            control: 0.10,
            control_hover: 0.15,
            control_active: 0.30,
            flat_hover: 0.10,
            flat_active: 0.16,
            dim: 0.55,
            disabled: 0.5,
            trough: 0.15,
            border: 0.15,
            scroller: 0.5,
        }
    }
}

/// Every color role a widget may ask for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Semantic {
    pub scheme: Scheme,

    pub window: Surface,
    pub view: Surface,
    pub headerbar: Surface,
    pub card: Surface,
    pub dialog: Surface,
    pub popover: Surface,
    pub sidebar: Surface,
    pub secondary_sidebar: Surface,
    pub thumbnail: Surface,
    pub overview: Surface,
    /// The active segment of a toggle group / view switcher.
    pub active_toggle: Surface,

    pub accent: Swatch,
    pub destructive: Swatch,
    pub success: Swatch,
    pub warning: Swatch,
    pub error: Swatch,

    /// `--scrollbar-outline-color`: outline that keeps overlay scrollbars visible.
    pub scrollbar_outline: Color,
    /// `--border-color`: the window foreground at `--border-opacity`.
    pub border: Color,
    /// Hairline between boxed-list rows (`--card-shade-color`).
    pub card_shade: Color,
    /// Hairline under bars (`--headerbar-shade-color`).
    pub headerbar_shade: Color,
    /// Line under a `raised-border` bar (`--headerbar-darker-shade-color`).
    pub headerbar_darker_shade: Color,
    /// Hairline beside a sidebar (`--sidebar-border-color`).
    pub sidebar_border: Color,
    /// Popover shade (`--popover-shade-color`).
    pub popover_shade: Color,
    /// Dark scrim used behind dialogs and for shadows (`shade_color`).
    pub shade: Color,
    /// Color of the focus ring.
    pub focus_ring: Color,
    /// Selection highlight in text inputs.
    pub selection: Color,
    /// Colored background for OSD overlays (toasts, floating bars).
    pub osd: Surface,

    pub opacity: Opacity,
}

impl Semantic {
    /// Derives the semantic layer from a palette.
    pub fn from_palette(p: &Palette, scheme: Scheme) -> Self {
        let s = |bg, fg| Surface { bg, fg };
        let w = |bg, fg, color| Swatch { bg, fg, color };
        Self {
            scheme,
            window: s(p.window_bg_color, p.window_fg_color),
            view: s(p.view_bg_color, p.view_fg_color),
            headerbar: s(p.headerbar_bg_color, p.headerbar_fg_color),
            card: s(p.card_bg_color, p.card_fg_color),
            dialog: s(p.dialog_bg_color, p.dialog_fg_color),
            popover: s(p.popover_bg_color, p.popover_fg_color),
            sidebar: s(p.sidebar_bg_color, p.sidebar_fg_color),
            secondary_sidebar: s(p.secondary_sidebar_bg_color, p.secondary_sidebar_fg_color),
            thumbnail: s(p.thumbnail_bg_color, p.thumbnail_fg_color),
            overview: s(p.overview_bg_color, p.overview_fg_color),
            active_toggle: s(p.active_toggle_bg_color, p.active_toggle_fg_color),

            accent: w(p.accent_bg_color, p.accent_fg_color, p.accent_color),
            destructive: w(
                p.destructive_bg_color,
                p.destructive_fg_color,
                p.destructive_color,
            ),
            success: w(p.success_bg_color, p.success_fg_color, p.success_color),
            warning: w(p.warning_bg_color, p.warning_fg_color, p.warning_color),
            error: w(p.error_bg_color, p.error_fg_color, p.error_color),

            scrollbar_outline: p.scrollbar_outline_color,
            border: alpha(p.window_fg_color, Opacity::default().border),
            card_shade: p.card_shade_color,
            headerbar_shade: p.headerbar_shade_color,
            headerbar_darker_shade: p.headerbar_darker_shade_color,
            sidebar_border: p.sidebar_border_color,
            popover_shade: p.popover_shade_color,
            shade: p.shade_color,
            focus_ring: alpha(p.accent_bg_color, 0.5),
            selection: alpha(p.accent_bg_color, 0.3),
            osd: Surface {
                bg: Color::from_rgba(0.0, 0.0, 0.0, 0.65),
                fg: Color::WHITE,
            },

            opacity: Opacity::default(),
        }
    }

    /// Chooses the text color on every status fill (`accent`, `destructive`, `success`,
    /// `warning`, `error`) with the CSS `color-contrast()` rule: light or dark, whichever
    /// contrasts more with the fill ([`crate::color::readable_on`]). Replaces the palette's
    /// fixed `*_fg_color`s; [`Theme::with_auto_contrast`](crate::Theme::with_auto_contrast)
    /// turns it off, and a component's own `text_color` overrides it.
    pub fn apply_contrast(&mut self) {
        for sw in [
            &mut self.accent,
            &mut self.destructive,
            &mut self.success,
            &mut self.warning,
            &mut self.error,
        ] {
            sw.fg = crate::color::readable_on(sw.bg);
        }
    }

    pub fn is_dark(&self) -> bool {
        self.scheme.is_dark()
    }

    /// Returns the swatch for a role. [`Role::Neutral`] yields a swatch built
    /// from the window foreground.
    pub fn swatch(&self, role: Role) -> Swatch {
        match role {
            Role::Neutral => Swatch {
                bg: alpha(self.window.fg, self.opacity.control),
                fg: self.window.fg,
                color: self.window.fg,
            },
            Role::Accent => self.accent,
            Role::Destructive => self.destructive,
            Role::Success => self.success,
            Role::Warning => self.warning,
            Role::Error => self.error,
        }
    }

    /// Hover variant of a solid fill (like libadwaita's `lighten` on hover
    /// in light mode and a white overlay in dark mode).
    pub fn hover_fill(&self, c: Color) -> Color {
        // `background-image: image(color-mix(in srgb, currentColor 10%, transparent))`
        lighten(c, 0.10)
    }

    /// Pressed variant of a solid fill.
    pub fn active_fill(&self, c: Color) -> Color {
        // `background-image: image(RGB(0 0 6 / 20%))`
        crate::color::mix(c, crate::color::ink(1.0), 0.20)
    }

    /// `alpha(window_fg, a)`, the basis of translucent state layers.
    pub fn layer(&self, a: f32) -> Color {
        alpha(self.window.fg, a)
    }

    /// Dim foreground as an opaque color (foreground mixed into the window
    /// background). Use for SVG tints, which ignore alpha.
    pub fn dim_solid(&self) -> Color {
        crate::color::mix(self.window.bg, self.window.fg, self.opacity.dim)
    }

    /// Foreground at dim-label opacity.
    pub fn dim(&self) -> Color {
        alpha(self.window.fg, self.opacity.dim)
    }
}
