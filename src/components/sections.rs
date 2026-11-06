//! Content for an input's `left_section` / `right_section` slots (after Mantine's
//! `leftSection` / `rightSection`): a dim icon and a small flat icon button that fit inside an
//! [`Entry`](crate::components::Entry). Any other element can be a section too (spinner, badge, label).

use crate::Element;
use crate::{Icon, Metrics, Theme};
use iced::widget::{button, container, svg};
use iced::{Border, Length};

/// Side of a section's square slot (and of a [`section_button`]).
pub fn section_size(m: &Metrics) -> f32 {
    m.icon_size + 2.0 * m.space_xs
}

/// Semantic tint with real alpha, preserving the effective background.
fn tint(t: &Theme, active: bool, disabled: bool) -> iced::Color {
    let s = &t.semantic;
    let (fg, level) = if active {
        (s.accent.color, 1.0)
    } else {
        (s.window.fg, s.opacity.dim)
    };
    crate::color::fade(fg, level * if disabled { s.opacity.disabled } else { 1.0 })
}

/// A dim, non-interactive icon sized to sit inside an entry.
pub struct SectionIcon {
    icon: Icon,
    disabled: bool,
    metrics: Metrics,
}

pub fn section_icon(icon: Icon) -> SectionIcon {
    SectionIcon {
        icon,
        disabled: false,
        metrics: Metrics::desktop(),
    }
}

impl SectionIcon {
    /// Dim further, to match a disabled entry.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: 'a> From<SectionIcon> for Element<'a, M> {
    fn from(s: SectionIcon) -> Self {
        let disabled = s.disabled;
        container(
            s.icon
                .view(s.metrics.icon_size)
                .style(move |t: &Theme, _| svg::Style {
                    color: Some(tint(t, false, disabled)),
                }),
        )
        .center(Length::Fixed(section_size(&s.metrics)))
        .into()
    }
}

/// A flat circular icon button sized to fit inside an entry: a clear "x", a filter button.
/// `active` highlights it in the accent color (e.g. a filter is applied).
pub struct SectionButton<'a, M> {
    icon: Icon,
    on_press: Option<M>,
    active: bool,
    disabled: bool,
    metrics: Metrics,
    _p: std::marker::PhantomData<&'a ()>,
}

pub fn section_button<'a, M>(icon: Icon, on_press: M) -> SectionButton<'a, M> {
    SectionButton {
        icon,
        on_press: Some(on_press),
        active: false,
        disabled: false,
        metrics: Metrics::desktop(),
        _p: Default::default(),
    }
}

impl<'a, M> SectionButton<'a, M> {
    /// Highlighted (accent tint): the thing this button controls is on.
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
    /// Dim to match a disabled entry; a disabled button ignores presses.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
    /// The button's look with no press handler, for a trigger that handles the press itself
    /// (a [`Popover`](crate::widgets::Popover) anchored to it).
    pub fn face(icon: Icon, active: bool) -> Element<'a, M>
    where
        M: 'a,
    {
        let m = Metrics::desktop();
        container(
            icon.view(m.icon_size - 2.0)
                .style(move |t: &Theme, _| svg::Style {
                    color: Some(tint(t, active, false)),
                }),
        )
        .center(Length::Fixed(section_size(&m)))
        .style(move |t: &Theme| face_style(t, active))
        .into()
    }
}

fn face_style(t: &Theme, active: bool) -> container::Style {
    container::Style {
        background: active
            .then(|| iced::Background::Color(crate::color::alpha(t.semantic.accent.color, 0.18))),
        border: Border::default().rounded(t.metrics.radius_pill),
        ..Default::default()
    }
}

impl<'a, M: Clone + 'a> From<SectionButton<'a, M>> for Element<'a, M> {
    fn from(b: SectionButton<'a, M>) -> Self {
        let (active, disabled) = (b.active, b.disabled);
        let size = section_size(&b.metrics);
        let icon = container(
            b.icon
                .view(b.metrics.icon_size - 2.0)
                .style(move |t: &Theme, _| svg::Style {
                    color: Some(tint(t, active, disabled)),
                }),
        )
        .center(Length::Fixed(size));
        button(icon)
            .padding(0)
            .style(move |t: &Theme, status| {
                let hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);
                let mut st = button::Style {
                    background: None,
                    border: Border::default().rounded(t.metrics.radius_pill),
                    ..Default::default()
                };
                if active {
                    st.background = Some(iced::Background::Color(crate::color::alpha(
                        t.semantic.accent.color,
                        if hovered { 0.28 } else { 0.18 },
                    )));
                } else if hovered && !disabled {
                    st.background = Some(iced::Background::Color(crate::color::alpha(
                        t.semantic.window.fg,
                        0.1,
                    )));
                }
                st
            })
            .on_press_maybe(if disabled { None } else { b.on_press })
            .into()
    }
}
