//! Button styles: `.default`, `.flat`, `.suggested-action`,
//! `.destructive-action`, `.pill`, `.circular`, `.card`, `.osd`, …

use crate::color::{alpha, fade};
use crate::semantic::Role;
use crate::Theme;
use iced::widget::button::{Status, Style};
use iced::{Background, Border, Color, Shadow, Vector};

/// Position of a button inside a `.linked` group (shared corners).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Only,
    Start,
    Middle,
    End,
}

fn border(radius: impl Into<iced::border::Radius>) -> Border {
    Border {
        radius: radius.into(),
        ..Border::default()
    }
}

fn linked_radius(r: f32, pos: Position) -> iced::border::Radius {
    match pos {
        Position::Only => r.into(),
        Position::Start => iced::border::left(r),
        Position::End => iced::border::right(r),
        Position::Middle => 0.0.into(),
    }
}

/// Translucent state-layer button built from `fg` (`alpha(currentColor, x)`).
fn tinted(
    theme: &Theme,
    status: Status,
    fg: Color,
    rest: f32,
    radius: iced::border::Radius,
) -> Style {
    let o = theme.semantic.opacity;
    let (layer, text) = match status {
        Status::Active => (rest, fg),
        Status::Hovered => (
            if rest == 0.0 { o.flat_hover } else { o.control_hover },
            fg,
        ),
        Status::Pressed => (
            if rest == 0.0 { o.flat_active } else { o.control_active },
            fg,
        ),
        Status::Disabled => (rest, fade(fg, o.disabled)),
    };
    Style {
        background: (layer > 0.0).then(|| Background::Color(alpha(fg, layer))),
        text_color: text,
        border: border(radius),
        shadow: Shadow::default(),
        snap: false,
    }
}

/// Solid-fill button (suggested / destructive / opaque).
fn filled(
    theme: &Theme,
    status: Status,
    fill: Color,
    fg: Color,
    radius: iced::border::Radius,
) -> Style {
    let s = &theme.semantic;
    let (bg, text) = match status {
        Status::Active => (fill, fg),
        Status::Hovered => (s.hover_fill(fill), fg),
        Status::Pressed => (s.active_fill(fill), fg),
        Status::Disabled => (fade(fill, s.opacity.disabled), fade(fg, s.opacity.disabled)),
    };
    Style {
        background: Some(Background::Color(bg)),
        text_color: text,
        border: border(radius),
        shadow: Shadow::default(),
        snap: false,
    }
}

/// The standard button: translucent foreground background, 6px radius.
pub fn default(theme: &Theme, status: Status) -> Style {
    tinted(
        theme,
        status,
        theme.semantic.window.fg,
        theme.semantic.opacity.control,
        theme.metrics.radius_m.into(),
    )
}

/// `.raised`: identical to default (kept for API parity with GTK).
pub fn raised(theme: &Theme, status: Status) -> Style {
    default(theme, status)
}

/// `.flat`: transparent until hovered.
pub fn flat(theme: &Theme, status: Status) -> Style {
    tinted(theme, status, theme.semantic.window.fg, 0.0, theme.metrics.radius_m.into())
}

/// `.flat` using the headerbar foreground.
pub fn headerbar(theme: &Theme, status: Status) -> Style {
    tinted(theme, status, theme.semantic.headerbar.fg, 0.0, theme.metrics.radius_m.into())
}

/// `.suggested-action`.
pub fn suggested(theme: &Theme, status: Status) -> Style {
    role(Role::Accent)(theme, status)
}

/// `.destructive-action`.
pub fn destructive(theme: &Theme, status: Status) -> Style {
    role(Role::Destructive)(theme, status)
}

/// `.opaque`: solid accent fill (same as suggested, kept for parity).
pub fn opaque(theme: &Theme, status: Status) -> Style {
    suggested(theme, status)
}

/// Solid button for any [`Role`]. `Role::Neutral` falls back to [`default`].
pub fn role(r: Role) -> impl Fn(&Theme, Status) -> Style + Copy {
    move |theme, status| match r {
        Role::Neutral => default(theme, status),
        _ => {
            let sw = theme.semantic.swatch(r);
            filled(theme, status, sw.bg, sw.fg, theme.metrics.radius_m.into())
        }
    }
}

/// `.pill`: fully rounded standard button.
pub fn pill(theme: &Theme, status: Status) -> Style {
    tinted(
        theme,
        status,
        theme.semantic.window.fg,
        theme.semantic.opacity.control,
        theme.metrics.radius_pill.into(),
    )
}

/// `.pill.suggested-action`.
pub fn suggested_pill(theme: &Theme, status: Status) -> Style {
    let sw = theme.semantic.accent;
    filled(theme, status, sw.bg, sw.fg, theme.metrics.radius_pill.into())
}

/// `.circular`: use with equal width/height.
pub fn circular(theme: &Theme, status: Status) -> Style {
    pill(theme, status)
}

/// `.circular.flat`.
pub fn circular_flat(theme: &Theme, status: Status) -> Style {
    tinted(theme, status, theme.semantic.window.fg, 0.0, theme.metrics.radius_pill.into())
}

/// `.card`: a clickable card surface.
pub fn card(theme: &Theme, status: Status) -> Style {
    let s = &theme.semantic;
    let bg = match status {
        Status::Hovered => crate::color::mix(s.card.bg, s.card.fg, 0.04),
        Status::Pressed => crate::color::mix(s.card.bg, s.card.fg, 0.08),
        _ => s.card.bg,
    };
    Style {
        background: Some(Background::Color(bg)),
        text_color: if status == Status::Disabled {
            fade(s.card.fg, s.opacity.disabled)
        } else {
            s.card.fg
        },
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
        snap: false,
    }
}

/// `.osd`: dark translucent button for overlays.
pub fn osd(theme: &Theme, status: Status) -> Style {
    let osd = theme.semantic.osd;
    let layer = match status {
        Status::Hovered => 0.15,
        Status::Pressed => 0.25,
        _ => 0.0,
    };
    Style {
        background: Some(Background::Color(crate::color::mix(osd.bg, osd.fg, layer))),
        text_color: osd.fg,
        border: border(theme.metrics.radius_pill),
        shadow: Shadow::default(),
        snap: false,
    }
}

/// `.link`: accent-colored text that underlines on hover (underline omitted;
/// iced buttons cannot underline children).
pub fn link(theme: &Theme, status: Status) -> Style {
    let c = theme.semantic.accent.color;
    Style {
        background: None,
        text_color: match status {
            Status::Disabled => fade(c, theme.semantic.opacity.disabled),
            Status::Hovered | Status::Pressed => crate::color::mix(c, theme.semantic.window.fg, 0.2),
            Status::Active => c,
        },
        border: border(theme.metrics.radius_s),
        shadow: Shadow::default(),
        snap: false,
    }
}

/// A row inside a boxed list (`row.activatable`): flat with a subtle layer.
pub fn list_row(theme: &Theme, status: Status) -> Style {
    let fg = theme.semantic.card.fg;
    let o = theme.semantic.opacity;
    let layer = match status {
        Status::Hovered => o.flat_hover * 0.5,
        Status::Pressed => o.flat_active * 0.6,
        _ => 0.0,
    };
    Style {
        background: (layer > 0.0).then(|| Background::Color(alpha(fg, layer))),
        text_color: if status == Status::Disabled { fade(fg, o.disabled) } else { fg },
        border: border(0.0),
        shadow: Shadow::default(),
        snap: false,
    }
}

/// A row in a `.navigation-sidebar`; `selected` shows the accent-tinted pill.
pub fn sidebar_row(selected: bool) -> impl Fn(&Theme, Status) -> Style + Copy {
    move |theme, status| {
        let fg = theme.semantic.sidebar.fg;
        let o = theme.semantic.opacity;
        let layer = match (selected, status) {
            (true, Status::Hovered) => 0.14,
            (true, _) => 0.10,
            (false, Status::Hovered) => o.flat_hover * 0.6,
            (false, Status::Pressed) => o.flat_active * 0.6,
            _ => 0.0,
        };
        Style {
            background: (layer > 0.0).then(|| Background::Color(alpha(fg, layer))),
            text_color: fg,
            border: border(theme.metrics.radius_m),
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

/// A toggle button (`AdwToggle`, view switcher). `active` highlights it.
pub fn toggle(active: bool, pos: Position) -> impl Fn(&Theme, Status) -> Style + Copy {
    move |theme, status| {
        let o = theme.semantic.opacity;
        let fg = theme.semantic.window.fg;
        let r = linked_radius(theme.metrics.radius_m, pos);
        if active {
            let sw = theme.semantic.accent;
            let mut s = filled(theme, status, sw.bg, sw.fg, r);
            if status == Status::Disabled {
                s.text_color = fade(sw.fg, o.disabled);
            }
            s
        } else {
            tinted(theme, status, fg, o.control, r)
        }
    }
}

/// Standard button with linked-group corners (`.linked`).
pub fn linked(pos: Position) -> impl Fn(&Theme, Status) -> Style + Copy {
    move |theme, status| {
        let mut s = default(theme, status);
        s.border.radius = linked_radius(theme.metrics.radius_m, pos);
        s
    }
}

/// Flat button that reads as pressed when `active` (menu buttons, tabs).
pub fn flat_toggle(active: bool) -> impl Fn(&Theme, Status) -> Style + Copy {
    move |theme, status| {
        let mut s = flat(theme, status);
        if active && status == Status::Active {
            s.background = Some(Background::Color(alpha(
                theme.semantic.window.fg,
                theme.semantic.opacity.flat_hover,
            )));
        }
        s
    }
}
