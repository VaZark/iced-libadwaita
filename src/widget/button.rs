//! Buttons, `AdwButtonContent`, `AdwSplitButton`, `AdwToggleGroup`.

use super::Element;
use crate::style::button as sb;
use crate::style::button::Position;
use crate::{Icon, Metrics, Theme};
use iced::alignment::Vertical;
use iced::widget::{button, container, row, space, Button};
use iced::{Length, Padding};

/// `AdwButtonContent`: an optional icon next to a label.
pub struct ButtonContent<'a, M> {
    label: String,
    icon: Option<Icon>,
    metrics: Metrics,
    _p: std::marker::PhantomData<&'a M>,
}

impl<'a, M> ButtonContent<'a, M> {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
            metrics: Metrics::default(),
            _p: std::marker::PhantomData,
        }
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: 'a> From<ButtonContent<'a, M>> for Element<'a, M> {
    fn from(c: ButtonContent<'a, M>) -> Self {
        let mut r = row![].spacing(c.metrics.space_s).align_y(Vertical::Center);
        if let Some(i) = c.icon {
            r = r.push(i.view(c.metrics.icon_size));
        }
        r.push(super::typography::label(c.label)).into()
    }
}

fn sized<'a, M: 'a>(content: impl Into<Element<'a, M>>, m: &Metrics) -> Button<'a, M, Theme> {
    button(
        container(content)
            .center_y(Length::Fixed(m.button_min_height - m.button_padding.y()))
            .center_x(Length::Shrink),
    )
    .padding(m.button_padding)
}

/// Standard button (`.default`).
pub fn raised_button<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Button<'a, M, Theme> {
    sized(content, &Metrics::default()).style(sb::default)
}

/// `.flat` button.
pub fn flat_button<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Button<'a, M, Theme> {
    sized(content, &Metrics::default()).style(sb::flat)
}

/// `.suggested-action` button.
pub fn suggested_button<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Button<'a, M, Theme> {
    sized(content, &Metrics::default()).style(sb::suggested)
}

/// `.destructive-action` button.
pub fn destructive_button<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Button<'a, M, Theme> {
    sized(content, &Metrics::default()).style(sb::destructive)
}

/// `.pill` button.
pub fn pill_button<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Button<'a, M, Theme> {
    let m = Metrics::default();
    button(content.into()).padding(m.pill_padding).style(sb::pill)
}

/// `.pill.suggested-action` button.
pub fn suggested_pill_button<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
) -> Button<'a, M, Theme> {
    let m = Metrics::default();
    button(content.into()).padding(m.pill_padding).style(sb::suggested_pill)
}

/// `GtkProgressBar` with Adwaita's 6px thickness.
pub fn progress_bar<'a>(
    range: std::ops::RangeInclusive<f32>,
    value: f32,
) -> iced::widget::ProgressBar<'a, Theme> {
    iced::widget::progress_bar(range, value).girth(Metrics::default().progress_height)
}

/// A square `.circular` icon button (flat by default, like header bar buttons).
pub fn circular_button<'a, M: 'a>(icon: Icon) -> Button<'a, M, Theme> {
    let m = Metrics::default();
    button(
        container(icon.view(m.icon_size))
            .center(Length::Fixed(m.button_min_height)),
    )
    .padding(Padding::from(0.0))
    .style(sb::circular_flat)
}

/// A flat icon button with a rounded-rect hover (`.flat` + icon-only).
pub fn icon_button<'a, M: 'a>(icon: Icon) -> Button<'a, M, Theme> {
    let m = Metrics::default();
    button(container(icon.view(m.icon_size)).center(Length::Fixed(m.button_min_height)))
        .padding(Padding::from(0.0))
        .style(sb::flat)
}

/// `AdwSplitButton`: a main action plus a dropdown arrow, visually linked.
pub fn split_button<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M>>,
    on_press: M,
    on_menu: M,
) -> Element<'a, M> {
    let m = Metrics::default();
    row![
        sized(content, &m).style(sb::linked(Position::Start)).on_press(on_press),
        container(space().width(1).height(Length::Fill)).style(crate::style::container::bar_separator),
        button(container(Icon::GoDown.view(m.icon_size)).center_y(Length::Fixed(m.button_min_height - m.button_padding.y())))
            .padding(m.button_padding)
            .style(sb::linked(Position::End))
            .on_press(on_menu),
    ]
    .height(Length::Shrink)
    .into()
}

/// One entry of a [`ToggleGroup`].
pub struct ToggleItem<T> {
    pub value: T,
    pub label: Option<String>,
    pub icon: Option<Icon>,
}

impl<T> ToggleItem<T> {
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: Some(label.into()),
            icon: None,
        }
    }
    pub fn icon_only(value: T, icon: Icon) -> Self {
        Self {
            value,
            label: None,
            icon: Some(icon),
        }
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// `AdwToggleGroup` / `AdwViewSwitcher`: exclusive, linked toggle buttons.
pub struct ToggleGroup<'a, T, M> {
    items: Vec<ToggleItem<T>>,
    active: Option<T>,
    on_select: Box<dyn Fn(T) -> M + 'a>,
    metrics: Metrics,
}

/// Builds a [`ToggleGroup`].
pub fn toggle_group<'a, T: Clone + PartialEq, M>(
    items: Vec<ToggleItem<T>>,
    active: Option<T>,
    on_select: impl Fn(T) -> M + 'a,
) -> ToggleGroup<'a, T, M> {
    ToggleGroup {
        items,
        active,
        on_select: Box::new(on_select),
        metrics: Metrics::default(),
    }
}

impl<'a, T, M> ToggleGroup<'a, T, M> {
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, T: Clone + PartialEq + 'a, M: Clone + 'a> From<ToggleGroup<'a, T, M>> for Element<'a, M> {
    fn from(g: ToggleGroup<'a, T, M>) -> Self {
        let n = g.items.len();
        let m = g.metrics;
        let mut r = row![].spacing(0).align_y(Vertical::Center);
        for (i, item) in g.items.into_iter().enumerate() {
            let pos = match (i, n) {
                (_, 1) => Position::Only,
                (0, _) => Position::Start,
                (i, n) if i + 1 == n => Position::End,
                _ => Position::Middle,
            };
            let active = g.active.as_ref() == Some(&item.value);
            let mut content = row![].spacing(m.space_s).align_y(Vertical::Center);
            if let Some(ic) = item.icon {
                content = content.push(ic.view(m.icon_size));
            }
            if let Some(l) = item.label {
                content = content.push(super::typography::label(l));
            }
            let msg = (g.on_select)(item.value);
            r = r.push(
                sized(content, &m)
                    .style(sb::toggle(active, pos))
                    .on_press(msg),
            );
        }
        container(r)
            .padding(2)
            .style(|t: &Theme| iced::widget::container::Style {
                background: Some(iced::Background::Color(crate::color::alpha(
                    t.semantic.window.fg,
                    t.semantic.opacity.control,
                ))),
                border: iced::Border::default().rounded(if t.metrics.radius_m > 0.0 { t.metrics.radius_m + 2.0 } else { 0.0 }),
                ..Default::default()
            })
            .into()
    }
}
