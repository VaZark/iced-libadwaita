//! `Badge` and `Chip`: small tinted labels.
//!
//! A [`Badge`] is a read-only status or count (`3`, `New`, `99+`). A [`Chip`] is a tag the user
//! can act on: pressing it, removing it (a trailing ✕), or toggling it as selected. Both take
//! their colors from a [`Role`], so they follow the theme's semantic swatches. Not libadwaita
//! widgets; the pattern behind `.tag`-like pills in GNOME apps (see [`tag`](super::tag)).

use super::Element;
use crate::style::container as sc;
use crate::{Icon, IconSource, Metrics, Role, Theme};
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

/// A round badge around any content: a count, an icon, a glyph of your own. Sits between
/// [`Badge`] (a label pill) and [`Avatar`](super::Avatar) (a person). Tinted by a [`Role`], or
/// solid with [`solid`](Self::solid).
///
/// ```ignore
/// RoundBadge::new(Icon::Check.view(12.0)).role(Role::Success).solid(true)
/// ```
pub struct RoundBadge<'a, M> {
    content: Element<'a, M>,
    role: Role,
    color: Option<iced::Color>,
    solid: bool,
    size: f32,
}

impl<'a, M: 'a> RoundBadge<'a, M> {
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            content: content.into(),
            role: Role::Neutral,
            color: None,
            solid: false,
            size: 20.0,
        }
    }
    pub fn role(mut self, role: Role) -> Self {
        self.role = role;
        self
    }
    /// A color of your own (a playlist's color), in place of the [`role`](Self::role)'s.
    pub fn color(mut self, color: iced::Color) -> Self {
        self.color = Some(color);
        self
    }
    /// A solid fill with contrasting content instead of a tint.
    pub fn solid(mut self, solid: bool) -> Self {
        self.solid = solid;
        self
    }
    /// The diameter in px (default 20).
    pub fn size(mut self, px: f32) -> Self {
        self.size = px.max(1.0);
        self
    }
}

impl<'a, M: 'a> From<RoundBadge<'a, M>> for Element<'a, M> {
    fn from(b: RoundBadge<'a, M>) -> Self {
        let (role, solid, size, custom) = (b.role, b.solid, b.size, b.color);
        container(b.content)
            .center(Length::Fixed(size))
            .style(move |t: &crate::Theme| {
                let base = match (custom, solid) {
                    (Some(c), true) => iced::widget::container::Style {
                        text_color: Some(crate::color::readable_on(c)),
                        background: Some(iced::Background::Color(c)),
                        ..Default::default()
                    },
                    (Some(c), false) => iced::widget::container::Style {
                        text_color: Some(c),
                        background: Some(iced::Background::Color(crate::color::alpha(c, 0.18))),
                        ..Default::default()
                    },
                    (None, true) => sc::role(role)(t),
                    (None, false) => sc::tinted(role)(t),
                };
                iced::widget::container::Style {
                    border: iced::Border::default().rounded(size / 2.0),
                    ..base
                }
            })
            .into()
    }
}

/// How big a [`Chip`] is: the text, the icon and the padding together.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ChipSize {
    /// About 18 px tall: tags in a dense list.
    Small,
    /// About 22 px tall: the default.
    #[default]
    Medium,
    /// About 28 px tall: a tag that is the subject, as on a detail page.
    Large,
}

impl ChipSize {
    /// `(text size, icon size, vertical padding, horizontal padding)` for `m`.
    fn dims(self, m: &Metrics) -> (f32, f32, f32, f32) {
        match self {
            Self::Small => (11.0, 10.0, 1.0, 8.0),
            Self::Medium => (m.space_m + 1.0, m.icon_size - 2.0, 2.0, m.space_m - 1.0),
            Self::Large => (15.0, 16.0, 4.0, 14.0),
        }
    }
}

/// An actionable pill: optional icon, a label, and optionally a remove button.
pub struct Chip<'a, M> {
    label: String,
    icon: Option<IconSource>,
    color: Option<iced::Color>,
    on_color: Option<iced::Color>,
    size: ChipSize,
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
            color: None,
            on_color: None,
            size: ChipSize::Medium,
            role: Role::Neutral,
            selected: false,
            disabled: false,
            on_press: None,
            on_remove: None,
            metrics: Metrics::desktop(),
            _p: Default::default(),
        }
    }
    /// An icon: a built-in [`Icon`] or your own SVG ([`IconSource`]).
    pub fn icon(mut self, icon: impl Into<IconSource>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    /// The text and icon color of a selected chip that has its own [`color`](Self::color), in
    /// place of the one picked for contrast (white on every tag, for a consistent look).
    pub fn on_color(mut self, color: iced::Color) -> Self {
        self.on_color = Some(color);
        self
    }
    /// The size: [`ChipSize::Small`], `Medium` (default) or `Large`.
    pub fn size(mut self, size: ChipSize) -> Self {
        self.size = size;
        self
    }
    /// A color of your own (a tag's color), in place of the [`role`](Self::role)'s: the chip is
    /// tinted with it, or solid in it when selected, and its icon and text follow.
    pub fn color(mut self, color: iced::Color) -> Self {
        self.color = Some(color);
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
    custom: Option<(iced::Color, Option<iced::Color>)>,
    selected: bool,
    disabled: bool,
) -> (iced::Color, iced::Color) {
    let (mut bg, mut fg) = if let Some((c, on)) = custom {
        if selected {
            (c, on.unwrap_or_else(|| crate::color::readable_on(c)))
        } else {
            // The text is the chip's own color, pulled away from the page if it is too close
            // to it to read (a dark tag color on a dark page).
            let surface = t.semantic.window.bg;
            let tinted = crate::color::mix(surface, c, 0.18);
            (
                crate::color::alpha(c, 0.18),
                crate::color::ensure_contrast(c, tinted, 4.5),
            )
        }
    } else if selected {
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
        let custom = c.color.map(|col| (col, c.on_color));
        let (text_size, icon_size, pad_y, pad_x) = c.size.dims(&m);
        let tint = move |t: &Theme| chip_colors(t, role, custom, selected, disabled).1;
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
        if let Some(i) = &c.icon {
            body =
                body.push(
                    i.view(icon_size)
                        .style(move |t: &Theme, _| iced::widget::svg::Style {
                            color: Some(tint(t)),
                        }),
                );
        }
        body = body.push(text(c.label).size(text_size).wrapping(text::Wrapping::None));
        let main = button(body)
            .padding(Padding::from([pad_y, pad_x]))
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
                let (bg, fg) = chip_colors(t, role, custom, selected, disabled);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chips_grow_with_their_size() {
        let m = Metrics::desktop();
        let (ts, is, py, _) = ChipSize::Small.dims(&m);
        let (tm, im, pm, _) = ChipSize::Medium.dims(&m);
        let (tl, il, pl, _) = ChipSize::Large.dims(&m);
        assert!(ts < tm && tm < tl && is < im && im < il && py < pm && pm < pl);
        assert_eq!(ChipSize::default(), ChipSize::Medium);
    }

    #[test]
    fn a_round_badge_keeps_its_own_color() {
        let b: RoundBadge<'_, ()> = RoundBadge::new(iced::widget::space())
            .color(iced::Color::from_rgb(1.0, 0.0, 0.0))
            .size(30.0);
        assert!(b.color.is_some() && b.size == 30.0);
    }
}
