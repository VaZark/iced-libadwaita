//! Styles for form controls and misc iced widgets.


pub mod text_input {
    use crate::color::{alpha, fade};
    use crate::Theme;
    use iced::widget::text_input::{Status, Style};
    use iced::{Background, Border};

    /// Entry: translucent fill, accent ring on focus.
    pub fn default(theme: &Theme, status: Status) -> Style {
        let s = &theme.semantic;
        let o = s.opacity;
        let fg = s.window.fg;
        let (fill, border) = match status {
            Status::Active => (o.control, Border::default()),
            Status::Hovered => (o.control_hover, Border::default()),
            Status::Focused { .. } => (
                o.control,
                Border {
                    color: s.accent.bg,
                    width: theme.metrics.focus_ring_width,
                    radius: 0.0.into(),
                },
            ),
            Status::Disabled => (o.control, Border::default()),
        };
        let disabled = status == Status::Disabled;
        Style {
            background: Background::Color(alpha(fg, fill)),
            border: Border {
                radius: theme.metrics.radius_m.into(),
                ..border
            },
            icon: s.dim(),
            placeholder: s.dim(),
            value: if disabled { fade(fg, o.disabled) } else { fg },
            selection: s.selection,
        }
    }

    /// Entry marked `.error`.
    pub fn error(theme: &Theme, status: Status) -> Style {
        let mut st = default(theme, status);
        st.border.color = theme.semantic.error.color;
        st.border.width = theme.metrics.focus_ring_width;
        st
    }

    /// Entry marked `.warning`.
    pub fn warning(theme: &Theme, status: Status) -> Style {
        let mut st = default(theme, status);
        st.border.color = theme.semantic.warning.color;
        st.border.width = theme.metrics.focus_ring_width;
        st
    }

    /// Entry marked `.success`.
    pub fn success(theme: &Theme, status: Status) -> Style {
        let mut st = default(theme, status);
        st.border.color = theme.semantic.success.color;
        st.border.width = theme.metrics.focus_ring_width;
        st
    }

    /// Entry embedded in a row (no fill; the row provides the surface).
    pub fn embedded(theme: &Theme, status: Status) -> Style {
        let mut st = default(theme, status);
        st.background = Background::Color(iced::Color::TRANSPARENT);
        st.border = Border::default();
        st
    }
}

pub mod text_editor {
    use crate::Theme;
    use iced::widget::text_editor::{Status, Style};
    use iced::{Background, Border};

    /// Multi-line entry: same translucent fill and focus ring as an entry.
    pub fn default(theme: &Theme, status: Status) -> Style {
        let s = &theme.semantic;
        let focused = matches!(status, Status::Focused { .. });
        Style {
            background: Background::Color(crate::color::alpha(s.window.fg, s.opacity.control)),
            border: Border {
                color: if focused { s.accent.bg } else { iced::Color::TRANSPARENT },
                width: if focused { theme.metrics.focus_ring_width } else { 0.0 },
                radius: theme.metrics.radius_m.into(),
            },
            placeholder: s.dim(),
            value: s.window.fg,
            selection: s.selection,
        }
    }
}

pub mod checkbox {
    use crate::color::{alpha, fade};
    use crate::Theme;
    use iced::widget::checkbox::{Status, Style};
    use iced::{Background, Border};

    pub fn default(theme: &Theme, status: Status) -> Style {
        let s = &theme.semantic;
        let o = s.opacity;
        let (checked, hovered, disabled) = match status {
            Status::Active { is_checked } => (is_checked, false, false),
            Status::Hovered { is_checked } => (is_checked, true, false),
            Status::Disabled { is_checked } => (is_checked, false, true),
        };
        let (bg, icon) = if checked {
            let b = if hovered { s.hover_fill(s.accent.bg) } else { s.accent.bg };
            (b, s.accent.fg)
        } else {
            let a = if hovered { o.control_hover } else { o.control };
            (alpha(s.window.fg, a + 0.05), s.window.fg)
        };
        let k = if disabled { o.disabled } else { 1.0 };
        Style {
            background: Background::Color(fade(bg, k)),
            icon_color: fade(icon, k),
            border: Border {
                color: if checked { iced::Color::TRANSPARENT } else { alpha(s.window.fg, 0.25 * k) },
                width: if checked { 0.0 } else { theme.metrics.border_width },
                radius: theme.metrics.radius_s.into(),
            },
            text_color: Some(fade(s.window.fg, k)),
        }
    }
}

pub mod radio {
    use crate::color::{alpha, fade};
    use crate::Theme;
    use iced::widget::radio::{Status, Style};
    use iced::Background;

    pub fn default(theme: &Theme, status: Status) -> Style {
        let s = &theme.semantic;
        let (selected, hovered) = match status {
            Status::Active { is_selected } => (is_selected, false),
            Status::Hovered { is_selected } => (is_selected, true),
        };
        let bg = if selected {
            if hovered { s.hover_fill(s.accent.bg) } else { s.accent.bg }
        } else {
            alpha(s.window.fg, if hovered { s.opacity.control_hover } else { s.opacity.control } + 0.05)
        };
        Style {
            background: Background::Color(bg),
            dot_color: s.accent.fg,
            border_width: if selected { 0.0 } else { theme.metrics.border_width },
            border_color: fade(s.window.fg, 0.25),
            text_color: Some(s.window.fg),
        }
    }
}

pub mod toggler {
    use crate::color::{alpha, fade};
    use crate::Theme;
    use iced::widget::toggler::{Status, Style};
    use iced::{Background, Color};

    /// `GtkSwitch`: accent track when on, translucent track when off, white knob.
    pub fn default(theme: &Theme, status: Status) -> Style {
        let s = &theme.semantic;
        let (on, hovered, disabled) = match status {
            Status::Active { is_toggled } => (is_toggled, false, false),
            Status::Hovered { is_toggled } => (is_toggled, true, false),
            Status::Disabled { is_toggled } => (is_toggled, false, true),
        };
        let track = if on {
            if hovered { s.hover_fill(s.accent.bg) } else { s.accent.bg }
        } else {
            alpha(s.window.fg, s.opacity.trough + if hovered { 0.05 } else { 0.0 })
        };
        let knob = if s.is_dark() && !on { Color::from_rgb8(0xee, 0xee, 0xee) } else { Color::WHITE };
        let k = if disabled { s.opacity.disabled } else { 1.0 };
        Style {
            background: Background::Color(fade(track, k)),
            background_border_width: 0.0,
            background_border_color: Color::TRANSPARENT,
            foreground: Background::Color(fade(knob, k)),
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
            text_color: Some(fade(s.window.fg, k)),
            border_radius: Some(theme.metrics.radius_pill.into()),
            padding_ratio: 0.14,
        }
    }
}

pub mod slider {
    use crate::color::alpha;
    use crate::Theme;
    use iced::widget::slider::{Handle, HandleShape, Rail, Status, Style};
    use iced::{Background, Border, Color};

    /// `GtkScale`: thin accent progress, round white handle.
    pub fn default(theme: &Theme, status: Status) -> Style {
        let s = &theme.semantic;
        let handle = match status {
            Status::Active => Color::WHITE,
            Status::Hovered => Color::from_rgb(0.97, 0.97, 0.97),
            Status::Dragged => Color::from_rgb(0.92, 0.92, 0.92),
        };
        Style {
            rail: Rail {
                backgrounds: (
                    Background::Color(s.accent.bg),
                    Background::Color(alpha(s.window.fg, s.opacity.trough)),
                ),
                width: 6.0,
                border: Border::default().rounded(3.0),
            },
            handle: Handle {
                shape: HandleShape::Circle { radius: 9.0 },
                background: Background::Color(handle),
                border_width: 1.0,
                border_color: alpha(Color::BLACK, 0.2),
            },
        }
    }
}

pub mod progress_bar {
    use crate::color::alpha;
    use crate::Theme;
    use iced::widget::progress_bar::Style;
    use iced::{Background, Border};

    /// `GtkProgressBar`.
    pub fn default(theme: &Theme) -> Style {
        let s = &theme.semantic;
        Style {
            background: Background::Color(alpha(s.window.fg, s.opacity.trough)),
            bar: Background::Color(s.accent.bg),
            border: Border::default().rounded(theme.metrics.radius_pill),
        }
    }

    /// `GtkLevelBar`-like success bar.
    pub fn success(theme: &Theme) -> Style {
        let mut st = default(theme);
        st.bar = Background::Color(theme.semantic.success.bg);
        st
    }

    pub fn warning(theme: &Theme) -> Style {
        let mut st = default(theme);
        st.bar = Background::Color(theme.semantic.warning.bg);
        st
    }

    pub fn error(theme: &Theme) -> Style {
        let mut st = default(theme);
        st.bar = Background::Color(theme.semantic.error.bg);
        st
    }
}

pub mod rule {
    use crate::Theme;
    use iced::widget::rule::{FillMode, Style};

    /// `GtkSeparator`.
    pub fn default(theme: &Theme) -> Style {
        Style {
            color: theme.semantic.border,
            radius: 0.0.into(),
            fill_mode: FillMode::Full,
            snap: true,
        }
    }
}

pub mod scrollable {
    use crate::color::alpha;
    use crate::Theme;
    use iced::widget::scrollable::{Rail, Scroller, Status, Style};
    use iced::{Background, Border};

    /// Adwaita's overlay scrollbar (libadwaita `_scrolling.scss`):
    /// * slider: `currentColor` at 20% at rest, 40% hovered, 60% dragged;
    /// * trough: `currentColor` at 10%, shown only while the scrollbar is hovered
    ///   or dragged (`.hovering`);
    /// * a 1px outline in `--scrollbar-outline-color` (35% at rest, 60% when
    ///   hovered) keeps the slider visible on any content;
    /// * fully rounded.
    ///
    /// Sizes come from `Metrics` via [`crate::widget::scrollbar`].
    pub fn default(theme: &Theme, status: Status) -> Style {
        let s = &theme.semantic;
        let (hovered, dragged) = match status {
            Status::Active { .. } => (false, false),
            Status::Hovered { .. } => (true, false),
            Status::Dragged { .. } => (true, true),
        };
        let level = if dragged { 0.60 } else if hovered { 0.40 } else { 0.20 };
        let outline = if hovered { 0.60 } else { 0.35 };
        let radius = theme.metrics.radius_pill;
        let rail = Rail {
            background: hovered.then(|| Background::Color(alpha(s.window.fg, 0.10))),
            border: Border::default().rounded(radius),
            scroller: Scroller {
                background: Background::Color(alpha(s.window.fg, level)),
                border: Border {
                    color: alpha(s.scrollbar_outline, outline),
                    width: 1.0,
                    radius: radius.into(),
                },
            },
        };
        let mut style = iced::widget::scrollable::default(theme.as_iced(), match status {
            Status::Active { is_horizontal_scrollbar_disabled, is_vertical_scrollbar_disabled } => Status::Active {
                is_horizontal_scrollbar_disabled,
                is_vertical_scrollbar_disabled,
            },
            other => other,
        });
        style.container = iced::widget::container::Style::default();
        style.vertical_rail = rail;
        style.horizontal_rail = rail;
        style.gap = None;
        style
    }
}

pub mod pick_list {
    use crate::color::{alpha, fade};
    use crate::Theme;
    use iced::widget::pick_list::{Status, Style};
    use iced::{Background, Border};

    /// Dropdown button (`GtkDropDown`).
    pub fn default(theme: &Theme, status: Status) -> Style {
        let s = &theme.semantic;
        let o = s.opacity;
        let fg = s.window.fg;
        let layer = match status {
            Status::Active => o.control,
            Status::Hovered => o.control_hover,
            Status::Opened { .. } => o.control_active,
        };
        Style {
            text_color: fg,
            placeholder_color: s.dim(),
            handle_color: fade(fg, 0.8),
            background: Background::Color(alpha(fg, layer)),
            border: Border::default().rounded(theme.metrics.radius_m),
        }
    }

    /// Dropdown without background (inside rows).
    pub fn flat(theme: &Theme, status: Status) -> Style {
        let mut st = default(theme, status);
        if status == Status::Active {
            st.background = Background::Color(iced::Color::TRANSPARENT);
        }
        st
    }
}

pub mod menu {
    use crate::Theme;
    use iced::widget::overlay::menu::Style;
    use iced::Background;

    pub fn default(theme: &Theme) -> Style {
        let s = &theme.semantic;
        Style {
            background: Background::Color(s.popover.bg),
            border: theme.shadows.popover().border(theme.metrics.radius_popover),
            text_color: s.popover.fg,
            selected_text_color: s.accent.fg,
            selected_background: Background::Color(s.accent.bg),
            shadow: theme.shadows.popover().shadow,
        }
    }
}
