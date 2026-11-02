//! `Badge` and `Chip`: small tinted labels.
//!
//! A [`Badge`] is a read-only status or count (`3`, `New`, `99+`). A [`Chip`] is a tag the user
//! can act on: pressing it, removing it (a trailing ✕), or toggling it as selected. Both take
//! their colors from a [`Role`], so they follow the theme's semantic swatches. Not libadwaita
//! widgets; the pattern behind `.tag`-like pills in GNOME apps (see [`tag`](super::tag)).

use super::Element;
use crate::style::container as sc;
use crate::{Icon, Metrics, Role, Theme};
use iced::alignment::Vertical;
use iced::widget::{button, container, row, text};
use iced::{Length, Padding};

/// A read-only pill: a label or a count, tinted by a [`Role`] (or solid with [`solid`](Self::solid)).
pub struct Badge {
    label: String,
    role: Role,
    solid: bool,
    metrics: Metrics,
}

impl Badge {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            role: Role::Neutral,
            solid: false,
            metrics: Metrics::desktop(),
        }
    }
    /// A count; above `max` it reads `max+` (`99+`).
    pub fn count(n: u32, max: u32) -> Self {
        Self::new(if n > max {
            format!("{max}+")
        } else {
            n.to_string()
        })
    }
    pub fn role(mut self, role: Role) -> Self {
        self.role = role;
        self
    }
    /// A solid fill with contrasting text instead of a tint (notification counts).
    pub fn solid(mut self, solid: bool) -> Self {
        self.solid = solid;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: 'a> From<Badge> for Element<'a, M> {
    fn from(b: Badge) -> Self {
        let c = container(
            text(b.label)
                .size(b.metrics.space_m + 1.0)
                .wrapping(text::Wrapping::None),
        )
        .padding(Padding::from([1.0, b.metrics.space_s + 1.0]))
        .center_y(Length::Shrink);
        if b.solid {
            c.style(sc::role(b.role)).into()
        } else {
            c.style(sc::tinted(b.role)).into()
        }
    }
}

/// An actionable pill: optional icon, a label, and optionally a remove button.
pub struct Chip<'a, M> {
    label: String,
    icon: Option<Icon>,
    role: Role,
    selected: bool,
    disabled: bool,
    on_press: Option<M>,
    on_remove: Option<M>,
    metrics: Metrics,
    _p: std::marker::PhantomData<&'a ()>,
}

impl<'a, M> Chip<'a, M> {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            role: Role::Neutral,
            selected: false,
            disabled: false,
            on_press: None,
            on_remove: None,
            metrics: Metrics::desktop(),
            _p: Default::default(),
        }
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn role(mut self, role: Role) -> Self {
        self.role = role;
        self
    }
    /// A selected chip is solid; an unselected one is tinted.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    /// A disabled chip is dimmed and ignores presses.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Pressing the chip body.
    pub fn on_press(mut self, m: M) -> Self {
        self.on_press = Some(m);
        self
    }
    /// Shows a trailing ✕ that sends `m`.
    pub fn on_remove(mut self, m: M) -> Self {
        self.on_remove = Some(m);
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

/// Background and text colors of a chip.
fn chip_colors(
    t: &Theme,
    role: Role,
    selected: bool,
    disabled: bool,
) -> (iced::Color, iced::Color) {
    let (mut bg, mut fg) = if selected {
        let s = t.semantic.swatch(role).surface();
        (s.bg, s.fg)
    } else {
        let c = sc::tinted(role)(t);
        let bg = match c.background {
            Some(iced::Background::Color(bg)) => bg,
            _ => iced::Color::TRANSPARENT,
        };
        (bg, c.text_color.unwrap_or(t.semantic.window.fg))
    };
    if disabled {
        bg = crate::color::fade(bg, t.semantic.opacity.disabled);
        fg = crate::color::fade(fg, t.semantic.opacity.disabled);
    }
    (bg, fg)
}

impl<'a, M: Clone + 'a> From<Chip<'a, M>> for Element<'a, M> {
    fn from(c: Chip<'a, M>) -> Self {
        let m = c.metrics;
        let (role, selected, disabled) = (c.role, c.selected, c.disabled);
        let icon_size = m.icon_size - 2.0;
        let tint = move |t: &Theme| chip_colors(t, role, selected, disabled).1;
        // Inside the pill the buttons are transparent; hover is a faint layer of the text color.
        let inner = move |t: &Theme, status: button::Status| {
            let fg = tint(t);
            let hovered =
                !disabled && matches!(status, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: hovered.then(|| iced::Background::Color(crate::color::alpha(fg, 0.15))),
                text_color: fg,
                border: iced::Border::default().rounded(t.metrics.radius_pill),
                ..Default::default()
            }
        };
        let mut body = row![].spacing(m.space_s - 2.0).align_y(Vertical::Center);
        if let Some(i) = c.icon {
            body =
                body.push(
                    i.view(icon_size)
                        .style(move |t: &Theme, _| iced::widget::svg::Style {
                            color: Some(tint(t)),
                        }),
                );
        }
        body = body.push(
            text(c.label)
                .size(m.space_m + 1.0)
                .wrapping(text::Wrapping::None),
        );
        let main = button(body)
            .padding(Padding::from([2.0, m.space_m - 1.0]))
            .style(inner)
            .on_press_maybe(c.on_press.filter(|_| !disabled));
        let mut pill = row![main].align_y(Vertical::Center);
        if let Some(rm) = c.on_remove {
            pill = pill.push(
                button(
                    Icon::WindowClose
                        .view(icon_size)
                        .style(move |t: &Theme, _| iced::widget::svg::Style {
                            color: Some(tint(t)),
                        }),
                )
                .padding(Padding::from([2.0, 4.0]))
                .style(inner)
                .on_press_maybe((!disabled).then_some(rm)),
            );
        }
        container(pill)
            .style(move |t: &Theme| {
                let (bg, fg) = chip_colors(t, role, selected, disabled);
                iced::widget::container::Style {
                    text_color: Some(fg),
                    background: Some(iced::Background::Color(bg)),
                    border: iced::Border::default().rounded(t.metrics.radius_pill),
                    ..Default::default()
                }
            })
            .into()
    }
}
