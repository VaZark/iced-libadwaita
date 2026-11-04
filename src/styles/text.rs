//! Text color classes: `.dim-label`, `.accent`, `.success`, `.warning`,
//! `.error`, `.destructive`.

use crate::Theme;
use iced::widget::text::Style;

fn color(c: iced::Color) -> Style {
    Style { color: Some(c) }
}

pub fn default(_theme: &Theme) -> Style {
    Style::default()
}

/// `.dim-label`.
pub fn dim(theme: &Theme) -> Style {
    color(theme.semantic.dim())
}

/// `.accent`.
pub fn accent(theme: &Theme) -> Style {
    color(theme.semantic.accent.color)
}

/// `.success`.
pub fn success(theme: &Theme) -> Style {
    color(theme.semantic.success.color)
}

/// `.warning`.
pub fn warning(theme: &Theme) -> Style {
    color(theme.semantic.warning.color)
}

/// `.error`.
pub fn error(theme: &Theme) -> Style {
    color(theme.semantic.error.color)
}

/// Destructive action text.
pub fn destructive(theme: &Theme) -> Style {
    color(theme.semantic.destructive.color)
}

/// Text on an accent fill (`accent_fg_color`).
pub fn on_accent(theme: &Theme) -> Style {
    color(theme.semantic.accent.fg)
}
