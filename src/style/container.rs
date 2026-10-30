//! Container styles: surfaces and `.card`, `.osd`, `.boxed-list`, …

use crate::color::alpha;
use crate::semantic::{Role, Surface};
use crate::Theme;
use iced::widget::container::Style;
use iced::{Background, Border, Color, Shadow, Vector};

fn surface(s: Surface, radius: f32) -> Style {
    Style {
        text_color: Some(s.fg),
        background: Some(Background::Color(s.bg)),
        border: Border::default().rounded(radius),
        shadow: Shadow::default(),
        snap: false,
    }
}

/// No background; text color is inherited.
pub fn transparent(_theme: &Theme) -> Style {
    Style::default()
}

/// `window` background.
pub fn window(theme: &Theme) -> Style {
    surface(theme.semantic.window, 0.0)
}

/// `.view`: lists, text areas.
pub fn view(theme: &Theme) -> Style {
    surface(theme.semantic.view, 0.0)
}

/// `headerbar` / `.toolbar` background.
pub fn headerbar(theme: &Theme) -> Style {
    surface(theme.semantic.headerbar, 0.0)
}

/// `.navigation-sidebar` pane background.
pub fn sidebar(theme: &Theme) -> Style {
    surface(theme.semantic.sidebar, 0.0)
}

/// Secondary sidebar pane (three-pane layouts).
pub fn secondary_sidebar(theme: &Theme) -> Style {
    surface(theme.semantic.secondary_sidebar, 0.0)
}

/// Dialog body (`AdwDialog`).
pub fn dialog(theme: &Theme) -> Style {
    Style {
        border: Border {
            color: alpha(theme.semantic.dialog.fg, 0.1),
            width: theme.metrics.border_width,
            radius: theme.metrics.radius_l.into(),
        },
        shadow: Shadow {
            color: alpha(Color::BLACK, 0.35),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 18.0,
        },
        ..surface(theme.semantic.dialog, theme.metrics.radius_l)
    }
}

/// Popover / menu surface.
pub fn popover(theme: &Theme) -> Style {
    Style {
        border: Border {
            color: alpha(theme.semantic.popover.fg, 0.1),
            width: theme.metrics.border_width,
            radius: theme.metrics.radius_l.into(),
        },
        shadow: Shadow {
            color: alpha(Color::BLACK, 0.25),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..surface(theme.semantic.popover, theme.metrics.radius_l)
    }
}

/// `.card`.
pub fn card(theme: &Theme) -> Style {
    let s = &theme.semantic;
    Style {
        border: Border {
            color: alpha(s.card.fg, 0.08),
            width: theme.metrics.border_width,
            radius: theme.metrics.radius_l.into(),
        },
        shadow: Shadow {
            color: s.shade,
            offset: Vector::new(0.0, 1.0),
            blur_radius: 3.0,
        },
        ..surface(s.card, theme.metrics.radius_l)
    }
}

/// `.boxed-list`: like a card, rows are separated by the caller.
pub fn boxed_list(theme: &Theme) -> Style {
    card(theme)
}

/// `.osd`: dark translucent floating surface.
pub fn osd(theme: &Theme) -> Style {
    surface(theme.semantic.osd, theme.metrics.radius_l)
}

/// `AdwToast`.
pub fn toast(theme: &Theme) -> Style {
    Style {
        shadow: Shadow {
            color: alpha(Color::BLACK, 0.3),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
        ..surface(theme.semantic.osd, theme.metrics.radius_pill)
    }
}

/// Scrim behind modal dialogs.
pub fn backdrop(theme: &Theme) -> Style {
    Style {
        background: Some(Background::Color(alpha(
            Color::BLACK,
            if theme.is_dark() { 0.5 } else { 0.3 },
        ))),
        ..Style::default()
    }
}

/// `AdwBanner`: accent fill with accent foreground.
pub fn banner(theme: &Theme) -> Style {
    surface(theme.semantic.accent.surface(), 0.0)
}

/// Solid fill for a [`Role`] (`.accent` background, `.error` background, …).
pub fn role(r: Role) -> impl Fn(&Theme) -> Style + Copy {
    move |theme| {
        let sw = theme.semantic.swatch(r);
        surface(sw.surface(), theme.metrics.radius_m)
    }
}

/// Translucent tint of a [`Role`] with a colored foreground (inline badges,
/// `.tag`-like chips).
pub fn tinted(r: Role) -> impl Fn(&Theme) -> Style + Copy {
    move |theme| {
        let sw = theme.semantic.swatch(r);
        Style {
            text_color: Some(sw.color),
            background: Some(Background::Color(alpha(sw.bg, 0.18))),
            border: Border::default().rounded(theme.metrics.radius_pill),
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

/// Circular colored badge (used by `Avatar`).
pub fn avatar(bg: Color, size: f32) -> impl Fn(&Theme) -> Style {
    move |_| Style {
        text_color: Some(Color::WHITE),
        background: Some(Background::Color(bg)),
        border: Border::default().rounded(size / 2.0),
        shadow: Shadow::default(),
        snap: false,
    }
}

/// A 1px-high / wide hairline in the border color. Use with a fixed
/// `height(1)` or `width(1)` container.
pub fn separator(theme: &Theme) -> Style {
    Style {
        background: Some(Background::Color(theme.semantic.border)),
        ..Style::default()
    }
}

/// `.thumbnail` surface.
pub fn thumbnail(theme: &Theme) -> Style {
    surface(theme.semantic.thumbnail, theme.metrics.radius_m)
}
