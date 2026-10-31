//! Buttons, `AdwButtonContent`, `AdwSplitButton`, `AdwToggleGroup`.

use super::Element;
use crate::style::button as sb;
use crate::{Icon, Metrics, Theme};
use iced::alignment::Vertical;
use iced::widget::{button, container, row, Button};
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
            metrics: Metrics::desktop(),
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

pub(super) fn sized<'a, M: 'a>(content: impl Into<Element<'a, M>>, m: &Metrics) -> Button<'a, M, Theme> {
    button(
        container(content)
            .center_y(Length::Fixed(m.button_min_height - m.button_padding.y()))
            .center_x(Length::Shrink),
    )
    .padding(m.button_padding)
}

/// Standard button (`.default`).
pub fn raised_button<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Button<'a, M, Theme> {
    sized(content, &Metrics::desktop()).style(sb::default)
}

/// `.flat` button.
pub fn flat_button<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Button<'a, M, Theme> {
    sized(content, &Metrics::desktop()).style(sb::flat)
}

/// `.suggested-action` button.
pub fn suggested_button<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Button<'a, M, Theme> {
    sized(content, &Metrics::desktop()).style(sb::suggested)
}

/// `.destructive-action` button.
pub fn destructive_button<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Button<'a, M, Theme> {
    sized(content, &Metrics::desktop()).style(sb::destructive)
}

/// `.pill` button.
pub fn pill_button<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Button<'a, M, Theme> {
    let m = Metrics::desktop();
    button(content.into()).padding(m.pill_padding).style(sb::pill)
}

/// `.pill.suggested-action` button.
pub fn suggested_pill_button<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
) -> Button<'a, M, Theme> {
    let m = Metrics::desktop();
    button(content.into()).padding(m.pill_padding).style(sb::suggested_pill)
}

/// `GtkProgressBar` with Adwaita's 6px thickness.
pub fn progress_bar<'a>(
    range: std::ops::RangeInclusive<f32>,
    value: f32,
) -> iced::widget::ProgressBar<'a, Theme> {
    iced::widget::progress_bar(range, value).girth(Metrics::desktop().progress_height)
}

/// A square `.circular` icon button (flat by default, like header bar buttons).
pub fn circular_button<'a, M: 'a>(icon: Icon) -> Button<'a, M, Theme> {
    let m = Metrics::desktop();
    button(
        container(icon.view(m.icon_size))
            .center(Length::Fixed(m.button_min_height)),
    )
    .padding(Padding::from(0.0))
    .style(sb::circular_flat)
}

/// A flat icon button with a rounded-rect hover (`.flat` + icon-only).
pub fn icon_button<'a, M: 'a>(icon: Icon) -> Button<'a, M, Theme> {
    let m = Metrics::desktop();
    button(container(icon.view(m.icon_size)).center(Length::Fixed(m.button_min_height)))
        .padding(Padding::from(0.0))
        .style(sb::flat)
}

/// One entry of a [`ToggleGroup`]: a label, an icon, both, or any element.
pub struct ToggleItem<'a, T, M> {
    pub value: T,
    pub label: Option<String>,
    pub icon: Option<Icon>,
    custom: Option<Element<'a, M>>,
}

impl<'a, T, M> ToggleItem<'a, T, M> {
    /// A text segment.
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self { value, label: Some(label.into()), icon: None, custom: None }
    }
    /// An icon-only segment.
    pub fn icon_only(value: T, icon: Icon) -> Self {
        Self { value, label: None, icon: Some(icon), custom: None }
    }
    /// A segment with any content (an avatar, a badge, a row of widgets…).
    pub fn custom(value: T, content: impl Into<Element<'a, M>>) -> Self {
        Self { value, label: None, icon: None, custom: Some(content.into()) }
    }
    /// Adds an icon before the label. When the group is too narrow for icon and
    /// label together, the segment falls back to the icon alone.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// `AdwToggleGroup` / `AdwViewSwitcher`: exclusive, linked segments.
///
/// It respects the space it is given: with [`width`](ToggleGroup::width)`(Fill)`
/// (or when a stacked row asks for 100%) the segments stretch evenly with no
/// dead space; when too wide it shrinks fairly and never overshoots its limits.
pub struct ToggleGroup<'a, T, M> {
    items: Vec<ToggleItem<'a, T, M>>,
    active: Option<T>,
    on_select: Box<dyn Fn(T) -> M + 'a>,
    width: Length,
    homogeneous: bool,
    metrics: Metrics,
}

/// Builds a [`ToggleGroup`].
pub fn toggle_group<'a, T: Clone + PartialEq, M>(
    items: Vec<ToggleItem<'a, T, M>>,
    active: Option<T>,
    on_select: impl Fn(T) -> M + 'a,
) -> ToggleGroup<'a, T, M> {
    ToggleGroup {
        items,
        active,
        on_select: Box::new(on_select),
        width: Length::Shrink,
        homogeneous: false,
        metrics: Metrics::desktop(),
    }
}

impl<'a, T, M> ToggleGroup<'a, T, M> {
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
    /// All segments the same width (`AdwToggleGroup:homogeneous`).
    pub fn homogeneous(mut self, h: bool) -> Self {
        self.homogeneous = h;
        self
    }
    /// `Length::Fill` makes the group fill the available width, each segment
    /// taking an equal share (`flex: 1`) with its content centered.
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
}

impl<'a, T: Clone + PartialEq + 'a, M: 'a> From<ToggleGroup<'a, T, M>> for Element<'a, M> {
    fn from(g: ToggleGroup<'a, T, M>) -> Self {
        let m = g.metrics;
        let active = g.active.as_ref().and_then(|a| g.items.iter().position(|i| &i.value == a));
        let segments = g
            .items
            .into_iter()
            .map(|item| {
                let icon = |ic: Icon| -> Element<'a, M> { ic.view(m.icon_size).into() };
                let (full, compact): (Element<'a, M>, Option<Element<'a, M>>) = match (item.custom, item.icon, item.label) {
                    (Some(c), _, _) => (c, None),
                    (None, Some(ic), Some(l)) => (
                        row![icon(ic), super::typography::label(l)]
                            .spacing(m.space_s)
                            .align_y(Vertical::Center)
                            .into(),
                        Some(icon(ic)),
                    ),
                    (None, Some(ic), None) => (icon(ic), None),
                    (None, None, Some(l)) => (super::typography::label(l).into(), None),
                    (None, None, None) => (iced::widget::space().into(), None),
                };
                (item.value, full, compact)
            })
            .collect();
        super::segmented::Segmented::new(segments, active, g.on_select)
            .width(g.width)
            .homogeneous(g.homogeneous)
            .metrics(m)
            .into()
    }
}
