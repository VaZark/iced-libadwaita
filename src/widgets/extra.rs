//! Remaining libadwaita gallery widgets, in stateless iced form: the app owns
//! the state (current page, open sheet, …) and reacts to the messages.

use crate::Element;
use crate::components::button::{ToggleGroup, ToggleItem, toggle_group};
use crate::layouts::responsive;
use crate::styles::{button as sb, container as sc, text as st};
use crate::widgets::navigation::Breakpoint;
use crate::{Icon, Metrics, Theme, Typography};
use iced::alignment::Vertical;
use iced::widget::{button, container, row, space, text};
use iced::{Length, Size};

/// `AdwWrapBox`: children flow onto new lines.
pub fn wrap_box<'a, M: 'a>(
    children: impl IntoIterator<Item = Element<'a, M>>,
    spacing: f32,
) -> Element<'a, M> {
    row(children)
        .spacing(spacing)
        .wrap()
        .vertical_spacing(spacing)
        .into()
}

/// `AdwBreakpointBin`: shows `narrow` when the breakpoint matches the
/// available size, else `wide`.
pub fn breakpoint_bin<'a, M: 'a>(
    breakpoint: Breakpoint,
    narrow: impl Fn() -> Element<'a, M> + 'a,
    wide: impl Fn() -> Element<'a, M> + 'a,
) -> Element<'a, M> {
    responsive(move |size: Size| {
        if breakpoint.matches(size) {
            narrow()
        } else {
            wide()
        }
    })
    .into()
}

/// `AdwViewSwitcher`: exclusive icon+label toggles (usually in a header bar).
pub fn view_switcher<'a, T: Clone + PartialEq, M>(
    items: Vec<ToggleItem<'a, T, M>>,
    active: Option<T>,
    on_select: impl Fn(T) -> M + 'a,
) -> ToggleGroup<'a, T, M> {
    toggle_group(items, active, on_select)
}

/// `AdwInlineViewSwitcher`: the same control, meant for use inside content.
pub fn inline_view_switcher<'a, T: Clone + PartialEq, M>(
    items: Vec<ToggleItem<'a, T, M>>,
    active: Option<T>,
    on_select: impl Fn(T) -> M + 'a,
) -> ToggleGroup<'a, T, M> {
    toggle_group(items, active, on_select)
}

/// `AdwViewSwitcherBar`: a switcher in a bottom bar.
pub fn view_switcher_bar<'a, M: Clone + 'a, T: Clone + PartialEq + 'a>(
    items: Vec<ToggleItem<'a, T, M>>,
    active: Option<T>,
    on_select: impl Fn(T) -> M + 'a,
) -> Element<'a, M> {
    let m = Metrics::desktop();
    container(Element::from(toggle_group(items, active, on_select)))
        .padding(m.space_s)
        .center_x(Length::Fill)
        .style(sc::headerbar)
        .into()
}

/// `AdwSidebar` / `AdwViewSwitcherSidebar`: same as
/// [`navigation_sidebar`](crate::widgets::navigation_sidebar).
pub fn sidebar<'a, M: 'a>(rows: impl IntoIterator<Item = Element<'a, M>>) -> Element<'a, M> {
    crate::widgets::navigation_sidebar(rows)
}

/// `AdwShortcutLabel`: renders an accelerator like `Ctrl+Shift+P` as keycaps.
pub fn shortcut_label<'a, M: 'a>(accelerator: &str) -> Element<'a, M> {
    let t = Typography::default();
    let mut r = row![].spacing(4).align_y(Vertical::Center);
    for (i, key) in accelerator.split('+').enumerate() {
        if i > 0 {
            r = r.push(t.caption("+").style(st::dim));
        }
        r = r.push(
            container(t.caption(key.trim().to_string()))
                .padding([2.0, 6.0])
                .style(|th: &Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(crate::color::alpha(
                        th.semantic.window.fg,
                        0.1,
                    ))),
                    border: iced::Border {
                        color: crate::color::alpha(th.semantic.window.fg, 0.15),
                        width: 1.0,
                        radius: th.metrics.radius_s.into(),
                    },
                    ..Default::default()
                }),
        );
    }
    r.into()
}

/// A Nautilus-style path bar: breadcrumb segments with the current one
/// highlighted, selecting one sends its index. Meant as a [`HeaderBar`](crate::widgets::HeaderBar)
/// `title_widget` (it replaces the title; window controls are unaffected). The
/// first segment is shown as the Home icon.
pub fn path_bar<'a, M: Clone + 'a>(
    segments: Vec<String>,
    current: usize,
    on_select: impl Fn(usize) -> M + 'a,
) -> Element<'a, M> {
    let items = segments
        .into_iter()
        .enumerate()
        .map(|(i, label)| {
            if i == 0 {
                ToggleItem::icon_only(i, Icon::Home)
            } else {
                ToggleItem::new(i, label)
            }
        })
        .collect();
    Element::from(toggle_group(items, Some(current), on_select))
}

/// `AdwCarouselIndicatorDots`.
pub fn carousel_indicator_dots<'a, M: Clone + 'a>(
    count: usize,
    index: usize,
    on_navigate: impl Fn(usize) -> M + 'a,
) -> Element<'a, M> {
    indicator(count, index, on_navigate, 8.0, 8.0)
}

/// `AdwCarouselIndicatorLines`.
pub fn carousel_indicator_lines<'a, M: Clone + 'a>(
    count: usize,
    index: usize,
    on_navigate: impl Fn(usize) -> M + 'a,
) -> Element<'a, M> {
    indicator(count, index, on_navigate, 24.0, 3.0)
}

fn indicator<'a, M: Clone + 'a>(
    count: usize,
    index: usize,
    on_navigate: impl Fn(usize) -> M + 'a,
    w: f32,
    h: f32,
) -> Element<'a, M> {
    let mut r = row![].spacing(6).align_y(Vertical::Center);
    for i in 0..count {
        let active = i == index;
        r = r.push(
            button(space().width(w).height(h))
                .padding(0)
                .style(move |t: &Theme, _| iced::widget::button::Style {
                    background: Some(iced::Background::Color(if active {
                        t.semantic.window.fg
                    } else {
                        crate::color::alpha(t.semantic.window.fg, 0.3)
                    })),
                    border: iced::Border::default().rounded(h / 2.0),
                    ..Default::default()
                })
                .on_press(on_navigate(i)),
        );
    }
    container(r).padding(8).into()
}

/// `AdwTabButton`: shows the open tab count, e.g. to open a tab overview.
pub fn tab_button<'a, M: Clone + 'a>(count: usize, on_press: M) -> Element<'a, M> {
    let m = Metrics::desktop();
    button(
        container(text(count.to_string()).size(12))
            .center(Length::Fixed(20.0))
            .style(|t: &Theme| iced::widget::container::Style {
                border: iced::Border {
                    color: t.semantic.window.fg,
                    width: 2.0,
                    radius: t.metrics.radius_s.into(),
                },
                ..Default::default()
            }),
    )
    .padding(m.space_s)
    .style(sb::flat)
    .on_press(on_press)
    .into()
}
