//! The libadwaita named-color palette.
//!
//! Field names match the `@define-color` names used by libadwaita's
//! stylesheet (e.g. `accent_bg_color`, `headerbar_fg_color`), so porting
//! custom CSS is mostly a rename. Values approximate libadwaita 1.6.

use crate::color::{black, hex, white};
use iced::Color;

/// Light or dark appearance (libadwaita's `AdwColorScheme` after resolution).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Scheme {
    #[default]
    Light,
    Dark,
}

impl Scheme {
    pub fn is_dark(self) -> bool {
        self == Scheme::Dark
    }
}

/// System accent colors (`AdwAccentColor`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AccentColor {
    #[default]
    Blue,
    Teal,
    Green,
    Yellow,
    Orange,
    Red,
    Pink,
    Purple,
    Slate,
}

impl AccentColor {
    pub const ALL: [AccentColor; 9] = [
        Self::Blue,
        Self::Teal,
        Self::Green,
        Self::Yellow,
        Self::Orange,
        Self::Red,
        Self::Pink,
        Self::Purple,
        Self::Slate,
    ];

    /// The `accent_bg_color` for this accent (like `adw_accent_color_to_rgba`).
    pub fn to_color(self) -> Color {
        hex(match self {
            Self::Blue => 0x3584e4,
            Self::Teal => 0x2190a4,
            Self::Green => 0x3a944a,
            Self::Yellow => 0xc88800,
            Self::Orange => 0xed5b00,
            Self::Red => 0xe62d42,
            Self::Pink => 0xd56199,
            Self::Purple => 0x9141ac,
            Self::Slate => 0x6f8396,
        })
    }

    /// The `accent_color` (accent used as foreground on a view) for a scheme
    /// (like `adw_accent_color_to_standalone_rgba`).
    pub fn to_standalone_color(self, scheme: Scheme) -> Color {
        hex(match (self, scheme) {
            (Self::Blue, Scheme::Light) => 0x1c71d8,
            (Self::Blue, Scheme::Dark) => 0x78aeed,
            (Self::Teal, Scheme::Light) => 0x007184,
            (Self::Teal, Scheme::Dark) => 0x7bdff4,
            (Self::Green, Scheme::Light) => 0x15772e,
            (Self::Green, Scheme::Dark) => 0x8de6b1,
            (Self::Yellow, Scheme::Light) => 0x905400,
            (Self::Yellow, Scheme::Dark) => 0xf8e359,
            (Self::Orange, Scheme::Light) => 0xbe3d00,
            (Self::Orange, Scheme::Dark) => 0xffa95a,
            (Self::Red, Scheme::Light) => 0xc01c28,
            (Self::Red, Scheme::Dark) => 0xff888c,
            (Self::Pink, Scheme::Light) => 0xa2326c,
            (Self::Pink, Scheme::Dark) => 0xffa0d2,
            (Self::Purple, Scheme::Light) => 0x8939a4,
            (Self::Purple, Scheme::Dark) => 0xdc8add,
            (Self::Slate, Scheme::Light) => 0x4f6272,
            (Self::Slate, Scheme::Dark) => 0xbed6ff,
        })
    }
}

/// The libadwaita named colors. Every field is overridable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub accent_bg_color: Color,
    pub accent_fg_color: Color,
    pub accent_color: Color,

    pub destructive_bg_color: Color,
    pub destructive_fg_color: Color,
    pub destructive_color: Color,

    pub success_bg_color: Color,
    pub success_fg_color: Color,
    pub success_color: Color,

    pub warning_bg_color: Color,
    pub warning_fg_color: Color,
    pub warning_color: Color,

    pub error_bg_color: Color,
    pub error_fg_color: Color,
    pub error_color: Color,

    pub window_bg_color: Color,
    pub window_fg_color: Color,

    pub view_bg_color: Color,
    pub view_fg_color: Color,

    pub headerbar_bg_color: Color,
    pub headerbar_fg_color: Color,
    pub headerbar_border_color: Color,
    pub headerbar_shade_color: Color,

    pub card_bg_color: Color,
    pub card_fg_color: Color,
    pub card_shade_color: Color,

    pub dialog_bg_color: Color,
    pub dialog_fg_color: Color,

    pub popover_bg_color: Color,
    pub popover_fg_color: Color,

    pub sidebar_bg_color: Color,
    pub sidebar_fg_color: Color,
    pub sidebar_border_color: Color,
    pub secondary_sidebar_bg_color: Color,
    pub secondary_sidebar_fg_color: Color,

    pub thumbnail_bg_color: Color,
    pub thumbnail_fg_color: Color,

    pub shade_color: Color,
    pub scrollbar_outline_color: Color,
}

impl Palette {
    pub fn new(scheme: Scheme) -> Self {
        match scheme {
            Scheme::Light => Self::light(),
            Scheme::Dark => Self::dark(),
        }
    }

    /// The default light palette (blue accent).
    pub fn light() -> Self {
        let fg = black(0.8);
        let accent = AccentColor::Blue;
        Self {
            accent_bg_color: accent.to_color(),
            accent_fg_color: Color::WHITE,
            accent_color: accent.to_standalone_color(Scheme::Light),

            destructive_bg_color: hex(0xe01b24),
            destructive_fg_color: Color::WHITE,
            destructive_color: hex(0xc01c28),

            success_bg_color: hex(0x2ec27e),
            success_fg_color: Color::WHITE,
            success_color: hex(0x1b8553),

            warning_bg_color: hex(0xe5a50a),
            warning_fg_color: black(0.8),
            warning_color: hex(0xae7b03),

            error_bg_color: hex(0xe01b24),
            error_fg_color: Color::WHITE,
            error_color: hex(0xc01c28),

            window_bg_color: hex(0xfafafa),
            window_fg_color: fg,

            view_bg_color: Color::WHITE,
            view_fg_color: fg,

            headerbar_bg_color: hex(0xebebeb),
            headerbar_fg_color: fg,
            headerbar_border_color: black(0.12),
            headerbar_shade_color: black(0.07),

            card_bg_color: Color::WHITE,
            card_fg_color: fg,
            card_shade_color: black(0.07),

            dialog_bg_color: hex(0xfafafa),
            dialog_fg_color: fg,

            popover_bg_color: Color::WHITE,
            popover_fg_color: fg,

            sidebar_bg_color: hex(0xebebeb),
            sidebar_fg_color: fg,
            sidebar_border_color: black(0.07),
            secondary_sidebar_bg_color: hex(0xf3f3f3),
            secondary_sidebar_fg_color: fg,

            thumbnail_bg_color: Color::WHITE,
            thumbnail_fg_color: fg,

            shade_color: black(0.07),
            scrollbar_outline_color: Color::WHITE,
        }
    }

    /// The default dark palette (blue accent).
    pub fn dark() -> Self {
        let fg = Color::WHITE;
        let accent = AccentColor::Blue;
        Self {
            accent_bg_color: accent.to_color(),
            accent_fg_color: Color::WHITE,
            accent_color: accent.to_standalone_color(Scheme::Dark),

            destructive_bg_color: hex(0xc01c28),
            destructive_fg_color: Color::WHITE,
            destructive_color: hex(0xff7b63),

            success_bg_color: hex(0x26a269),
            success_fg_color: Color::WHITE,
            success_color: hex(0x8ff0a4),

            warning_bg_color: hex(0xcd9309),
            warning_fg_color: black(0.8),
            warning_color: hex(0xf8e45c),

            error_bg_color: hex(0xc01c28),
            error_fg_color: Color::WHITE,
            error_color: hex(0xff7b63),

            window_bg_color: hex(0x242424),
            window_fg_color: fg,

            view_bg_color: hex(0x1e1e1e),
            view_fg_color: fg,

            headerbar_bg_color: hex(0x303030),
            headerbar_fg_color: fg,
            headerbar_border_color: white(0.12),
            headerbar_shade_color: black(0.36),

            card_bg_color: white(0.08),
            card_fg_color: fg,
            card_shade_color: black(0.36),

            dialog_bg_color: hex(0x383838),
            dialog_fg_color: fg,

            popover_bg_color: hex(0x383838),
            popover_fg_color: fg,

            sidebar_bg_color: hex(0x303030),
            sidebar_fg_color: fg,
            sidebar_border_color: white(0.07),
            secondary_sidebar_bg_color: hex(0x2a2a2a),
            secondary_sidebar_fg_color: fg,

            thumbnail_bg_color: hex(0x383838),
            thumbnail_fg_color: fg,

            shade_color: black(0.36),
            scrollbar_outline_color: black(0.5),
        }
    }

    /// Replaces the accent triple with a system accent, as
    /// `AdwStyleManager` does when the user picks an accent color.
    pub fn with_accent(mut self, accent: AccentColor, scheme: Scheme) -> Self {
        self.accent_bg_color = accent.to_color();
        self.accent_fg_color = Color::WHITE;
        self.accent_color = accent.to_standalone_color(scheme);
        self
    }

    /// Sets a custom accent background; the standalone accent is derived
    /// by mixing towards black (light) or white (dark).
    pub fn with_custom_accent(mut self, bg: Color, scheme: Scheme) -> Self {
        self.accent_bg_color = bg;
        self.accent_fg_color = Color::WHITE;
        self.accent_color = match scheme {
            Scheme::Light => crate::color::darken(bg, 0.2),
            Scheme::Dark => crate::color::lighten(bg, 0.35),
        };
        self
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::light()
    }
}
