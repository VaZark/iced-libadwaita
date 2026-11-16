//! The named-color palette.
//!
//! Field names match the `@define-color` names used by libadwaita's
//! stylesheet (e.g. `accent_bg_color`, `headerbar_fg_color`), so porting
//! custom CSS is mostly a rename. The built-in values are the bare baseline (black on white);
//! the libadwaita values are in `themes/libadwaita.conf`.

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

    /// `light` in the light scheme, `dark` in the dark one (like CSS `light-dark()`).
    pub fn light_dark<T>(self, light: T, dark: T) -> T {
        if self.is_dark() { dark } else { light }
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
    /// Any color, as `0xRRGGBB`. The ones above are the desktop's; this is for an app or a user
    /// that picks its own (a [`ColorPicker`](crate::widgets::ColorPicker)).
    Custom(u32),
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

    /// A custom accent from 8-bit red, green and blue.
    pub fn custom(r: u8, g: u8, b: u8) -> Self {
        Self::Custom(((r as u32) << 16) | ((g as u32) << 8) | b as u32)
    }

    /// Reads an accent: a name (`teal`, any case) or a hex color (`#3584e4`).
    pub fn parse(text: &str) -> Option<Self> {
        let t = text.trim();
        if let Some(a) = Self::ALL
            .into_iter()
            .find(|a| format!("{a:?}").eq_ignore_ascii_case(t))
        {
            return Some(a);
        }
        let h = t.strip_prefix('#')?;
        (h.len() == 6)
            .then(|| u32::from_str_radix(h, 16).ok())
            .flatten()
            .map(Self::Custom)
    }

    /// The text form: a name for the desktop's accents, `#rrggbb` for a custom one.
    pub fn key(self) -> String {
        match self {
            Self::Custom(c) => format!("#{c:06x}"),
            named => format!("{named:?}").to_lowercase(),
        }
    }

    /// The `accent_bg_color` for this accent (like `adw_accent_color_to_rgba`).
    pub fn to_color(self) -> Color {
        hex(match self {
            Self::Custom(c) => c,
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
        // A custom accent has no hand-made pair: darken it for a light view, lighten it for a
        // dark one, the way the named pairs relate to their fill.
        if let Self::Custom(c) = self {
            let base = hex(c);
            let mix = |to: f32, k: f32| Color {
                r: base.r + (to - base.r) * k,
                g: base.g + (to - base.g) * k,
                b: base.b + (to - base.b) * k,
                a: 1.0,
            };
            return match scheme {
                Scheme::Light => mix(0.0, 0.25),
                Scheme::Dark => mix(1.0, 0.45),
            };
        }
        hex(match (self, scheme) {
            (Self::Blue, Scheme::Light) => 0x0461be,
            (Self::Blue, Scheme::Dark) => 0x81d0ff,
            (Self::Teal, Scheme::Light) => 0x007184,
            (Self::Teal, Scheme::Dark) => 0x7bdff4,
            (Self::Green, Scheme::Light) => 0x15772e,
            (Self::Green, Scheme::Dark) => 0x8de698,
            (Self::Yellow, Scheme::Light) => 0x905300,
            (Self::Yellow, Scheme::Dark) => 0xffc057,
            (Self::Orange, Scheme::Light) => 0xb62200,
            (Self::Orange, Scheme::Dark) => 0xff9c5b,
            (Self::Red, Scheme::Light) => 0xc00023,
            (Self::Red, Scheme::Dark) => 0xff888c,
            (Self::Pink, Scheme::Light) => 0xa2326c,
            (Self::Pink, Scheme::Dark) => 0xffa0d8,
            (Self::Purple, Scheme::Light) => 0x8939a4,
            (Self::Purple, Scheme::Dark) => 0xfba7ff,
            (Self::Slate, Scheme::Light) => 0x526678,
            (Self::Slate, Scheme::Dark) => 0xbbd1e5,
            (Self::Custom(_), _) => unreachable!("handled above"),
        })
    }
}

/// The libadwaita named colors. Every field is overridable.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub accent_bg_color: Color,
    pub accent_fg_color: Color,
    pub accent_color: Color,

    /// The color of primary actions (Bootstrap's `$primary`, Mantine's `primaryColor`): suggested
    /// buttons and the like. Independent of the accent, which the user's desktop may change.
    pub primary_bg_color: Color,
    pub primary_fg_color: Color,
    pub primary_color: Color,

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
    pub headerbar_darker_shade_color: Color,

    pub card_bg_color: Color,
    pub card_fg_color: Color,
    pub card_shade_color: Color,

    pub dialog_bg_color: Color,
    pub dialog_fg_color: Color,

    pub popover_bg_color: Color,
    pub popover_fg_color: Color,
    pub popover_shade_color: Color,

    pub sidebar_bg_color: Color,
    pub sidebar_fg_color: Color,
    pub sidebar_border_color: Color,
    pub secondary_sidebar_bg_color: Color,
    pub secondary_sidebar_fg_color: Color,

    pub thumbnail_bg_color: Color,
    pub thumbnail_fg_color: Color,

    pub overview_bg_color: Color,
    pub overview_fg_color: Color,

    pub active_toggle_bg_color: Color,
    pub active_toggle_fg_color: Color,

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

    /// The bare light palette: black on white, grey rules, plain blue accent. Like a browser's
    /// default stylesheet it has no opinion; a theme file gives the look.
    pub fn light() -> Self {
        let fg = Color::BLACK;
        let rule = black(0.54);
        let accent = AccentColor::Custom(0x0066cc);
        Self {
            accent_bg_color: accent.to_color(),
            accent_fg_color: Color::WHITE,
            accent_color: accent.to_color(),

            primary_bg_color: Color::BLACK,
            primary_fg_color: Color::WHITE,
            primary_color: Color::BLACK,

            destructive_bg_color: hex(0xc00000),
            destructive_fg_color: Color::WHITE,
            destructive_color: hex(0xc00000),

            success_bg_color: hex(0x007000),
            success_fg_color: Color::WHITE,
            success_color: hex(0x007000),

            warning_bg_color: hex(0xffcc00),
            warning_fg_color: fg,
            warning_color: hex(0x805800),

            error_bg_color: hex(0xc00000),
            error_fg_color: Color::WHITE,
            error_color: hex(0xc00000),

            window_bg_color: Color::WHITE,
            window_fg_color: fg,

            view_bg_color: Color::WHITE,
            view_fg_color: fg,

            headerbar_bg_color: Color::WHITE,
            headerbar_fg_color: fg,
            headerbar_border_color: rule,
            headerbar_shade_color: rule,
            headerbar_darker_shade_color: rule,

            card_bg_color: Color::WHITE,
            card_fg_color: fg,
            card_shade_color: rule,

            dialog_bg_color: Color::WHITE,
            dialog_fg_color: fg,

            popover_bg_color: Color::WHITE,
            popover_fg_color: fg,
            popover_shade_color: rule,

            sidebar_bg_color: Color::WHITE,
            sidebar_fg_color: fg,
            sidebar_border_color: rule,
            secondary_sidebar_bg_color: Color::WHITE,
            secondary_sidebar_fg_color: fg,

            thumbnail_bg_color: Color::WHITE,
            thumbnail_fg_color: fg,

            overview_bg_color: Color::WHITE,
            overview_fg_color: fg,

            active_toggle_bg_color: black(0.1),
            active_toggle_fg_color: fg,

            shade_color: rule,
            scrollbar_outline_color: Color::WHITE,
        }
    }

    /// The bare dark palette: white on black, the light one inverted.
    pub fn dark() -> Self {
        let fg = Color::WHITE;
        let rule = white(0.54);
        let accent = AccentColor::Custom(0x6cb6ff);
        Self {
            accent_bg_color: accent.to_color(),
            accent_fg_color: Color::BLACK,
            accent_color: accent.to_color(),

            primary_bg_color: Color::WHITE,
            primary_fg_color: Color::BLACK,
            primary_color: Color::WHITE,

            destructive_bg_color: hex(0xff6b6b),
            destructive_fg_color: Color::BLACK,
            destructive_color: hex(0xff6b6b),

            success_bg_color: hex(0x5fd35f),
            success_fg_color: Color::BLACK,
            success_color: hex(0x5fd35f),

            warning_bg_color: hex(0xffcc00),
            warning_fg_color: Color::BLACK,
            warning_color: hex(0xffcc00),

            error_bg_color: hex(0xff6b6b),
            error_fg_color: Color::BLACK,
            error_color: hex(0xff6b6b),

            window_bg_color: Color::BLACK,
            window_fg_color: fg,

            view_bg_color: Color::BLACK,
            view_fg_color: fg,

            headerbar_bg_color: Color::BLACK,
            headerbar_fg_color: fg,
            headerbar_border_color: rule,
            headerbar_shade_color: rule,
            headerbar_darker_shade_color: rule,

            card_bg_color: Color::BLACK,
            card_fg_color: fg,
            card_shade_color: rule,

            dialog_bg_color: Color::BLACK,
            dialog_fg_color: fg,

            popover_bg_color: Color::BLACK,
            popover_fg_color: fg,
            popover_shade_color: rule,

            sidebar_bg_color: Color::BLACK,
            sidebar_fg_color: fg,
            sidebar_border_color: rule,
            secondary_sidebar_bg_color: Color::BLACK,
            secondary_sidebar_fg_color: fg,

            thumbnail_bg_color: Color::BLACK,
            thumbnail_fg_color: fg,

            overview_bg_color: Color::BLACK,
            overview_fg_color: fg,

            active_toggle_bg_color: white(0.2),
            active_toggle_fg_color: fg,

            shade_color: rule,
            scrollbar_outline_color: Color::BLACK,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::hex;

    #[test]
    fn the_built_in_palette_is_black_on_white_and_its_inverse() {
        let l = Palette::light();
        assert_eq!(l.window_bg_color, Color::WHITE);
        assert_eq!(l.window_fg_color, Color::BLACK);
        let d = Palette::dark();
        assert_eq!(d.window_bg_color, Color::BLACK);
        assert_eq!(d.window_fg_color, Color::WHITE);
        // The named accents are still available for a theme to pick.
        assert_eq!(
            AccentColor::Blue.to_standalone_color(Scheme::Light),
            hex(0x0461be)
        );
    }
}

#[cfg(test)]
mod accent_tests {
    use super::*;

    #[test]
    fn a_custom_accent_is_its_own_color() {
        let a = AccentColor::custom(0x35, 0x84, 0xe4);
        assert_eq!(a, AccentColor::Custom(0x3584e4));
        assert_eq!(a.to_color(), AccentColor::Blue.to_color());
        assert_eq!(a.key(), "#3584e4");
    }

    #[test]
    fn names_and_hex_both_parse() {
        assert_eq!(AccentColor::parse("Teal"), Some(AccentColor::Teal));
        assert_eq!(
            AccentColor::parse(" #9141ac "),
            Some(AccentColor::Custom(0x9141ac))
        );
        assert_eq!(AccentColor::parse("#9141a"), None);
        assert_eq!(AccentColor::parse("#gggggg"), None);
        assert_eq!(AccentColor::parse("mauve"), None);
        assert_eq!(AccentColor::Teal.key(), "teal");
        for a in AccentColor::ALL {
            assert_eq!(AccentColor::parse(&a.key()), Some(a), "{a:?} round-trips");
        }
    }

    #[test]
    fn the_on_view_color_of_a_custom_accent_follows_the_scheme() {
        let a = AccentColor::Custom(0x3584e4);
        let (light, dark) = (
            a.to_standalone_color(Scheme::Light),
            a.to_standalone_color(Scheme::Dark),
        );
        let lum = |c: Color| c.r + c.g + c.b;
        assert!(lum(light) < lum(a.to_color()), "darker on a light view");
        assert!(lum(dark) > lum(a.to_color()), "lighter on a dark view");
    }
}
