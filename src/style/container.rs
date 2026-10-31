//! Container styles: surfaces and `.card`, `.osd`, `.boxed-list`, …

use crate::color::alpha;
use crate::semantic::{Role, Surface};
use crate::Theme;
use iced::widget::container::Style;
use iced::{Background, Border, Color, Shadow};

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

/// Dialog body (`AdwDialog` floating sheet): window radius, layered shadow.
pub fn dialog(theme: &Theme) -> Style {
    let r = theme.shadows.dialog();
    Style {
        border: r.border(theme.metrics.radius_window),
        shadow: r.shadow,
        ..surface(theme.semantic.dialog, theme.metrics.radius_window)
    }
}

/// Popover / menu surface: popover radius (15), layered shadow.
pub fn popover(theme: &Theme) -> Style {
    let r = theme.shadows.popover();
    Style {
        border: r.border(theme.metrics.radius_popover),
        shadow: r.shadow,
        ..surface(theme.semantic.popover, theme.metrics.radius_popover)
    }
}

/// `.card`: card radius, layered shadow with a 1px ring.
pub fn card(theme: &Theme) -> Style {
    let r = theme.shadows.card();
    Style {
        border: r.border(theme.metrics.radius_l),
        shadow: r.shadow,
        ..surface(theme.semantic.card, theme.metrics.radius_l)
    }
}

/// `.card` surface without its shadow: the ring and radius only. Pair it with
/// [`card_shadow`](crate::widget::card_shadow), which draws each shadow layer as its own
/// rounded shape (iced has one shadow per quad and no spread).
pub fn card_surface(theme: &Theme) -> Style {
    Style {
        // No ring: `card_shadow` draws the ring as part of the shadow (a hard 1px line over
        // the shadow beneath it reads as an outline).
        border: iced::Border::default().rounded(theme.metrics.radius_l),
        ..surface(theme.semantic.card, theme.metrics.radius_l)
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
    let r = theme.shadows.osd();
    Style {
        border: r.border(theme.metrics.radius_pill),
        shadow: r.shadow,
        ..surface(theme.semantic.osd, theme.metrics.radius_pill)
    }
}

/// Scrim behind modal dialogs: `--shade-color` at double alpha
/// (`floating-sheet > dimming`), so 14% in light and 50% in dark.
pub fn backdrop(theme: &Theme) -> Style {
    let c = theme.semantic.shade;
    Style {
        background: Some(Background::Color(Color { a: (c.a * 2.0).min(1.0), ..c })),
        ..Style::default()
    }
}

/// The scrim clipped to a rounded window: same dim as [`backdrop`] with the window's
/// corner radius, so it never paints into the transparent corners outside the
/// rounded window (iced can only clip rectangles).
pub fn backdrop_in_window(radius: f32) -> impl Fn(&Theme) -> Style + Copy {
    move |theme| Style {
        border: Border::default().rounded(radius),
        ..backdrop(theme)
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
pub fn avatar(bg: Color, fg: Color, size: f32) -> impl Fn(&Theme) -> Style {
    move |_| Style {
        text_color: Some(fg),
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

/// Hairline between boxed-list rows (`--card-shade-color`).
pub fn card_separator(theme: &Theme) -> Style {
    Style {
        background: Some(Background::Color(theme.semantic.card_shade)),
        ..Style::default()
    }
}

/// Hairline under/over bars (`--headerbar-shade-color`).
pub fn bar_separator(theme: &Theme) -> Style {
    Style {
        background: Some(Background::Color(theme.semantic.headerbar_shade)),
        ..Style::default()
    }
}

/// Hairline between sections of a popover menu (`--popover-shade-color`).
pub fn popover_separator(theme: &Theme) -> Style {
    Style {
        background: Some(Background::Color(theme.semantic.popover_shade)),
        ..Style::default()
    }
}

/// Hairline beside a sidebar (`--sidebar-border-color`).
pub fn sidebar_separator(theme: &Theme) -> Style {
    Style {
        background: Some(Background::Color(theme.semantic.sidebar_border)),
        ..Style::default()
    }
}

/// `.thumbnail` surface.
pub fn thumbnail(theme: &Theme) -> Style {
    surface(theme.semantic.thumbnail, theme.metrics.radius_m)
}
