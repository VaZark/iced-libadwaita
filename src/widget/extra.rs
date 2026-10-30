//! Remaining libadwaita gallery widgets, in stateless iced form: the app owns
//! the state (current page, open sheet, …) and reacts to the messages.

use super::{header_bar, toolbar_view, Element, EntryRow};
use crate::style::{button as sb, container as sc, text as st};
use crate::widget::button::{toggle_group, ToggleGroup, ToggleItem};
use crate::widget::navigation::Breakpoint;
use crate::{DecorationLayout, Icon, Metrics, Theme, Typography, WindowButton};
use iced::alignment::Vertical;
use iced::widget::{
    button, column, container, mouse_area, opaque, responsive, row, space, stack, text,
};
use iced::{Length, Size};

/// `AdwWindow` / `AdwApplicationWindow`: fills the window with the window
/// background (the title bar is yours: put a [`HeaderBar`](super::HeaderBar) in).
pub fn window<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(sc::window)
        .into()
}

/// `AdwPasswordEntryRow`.
pub fn password_entry_row<'a, M>(
    title: impl Into<String>,
    value: impl Into<String>,
) -> EntryRow<'a, M> {
    EntryRow::new(title, value).password(true)
}

/// `AdwWrapBox`: children flow onto new lines.
pub fn wrap_box<'a, M: 'a>(
    children: impl IntoIterator<Item = Element<'a, M>>,
    spacing: f32,
) -> Element<'a, M> {
    row(children).spacing(spacing).wrap().vertical_spacing(spacing).into()
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
/// [`navigation_sidebar`](super::navigation_sidebar).
pub fn sidebar<'a, M: 'a>(rows: impl IntoIterator<Item = Element<'a, M>>) -> Element<'a, M> {
    super::navigation_sidebar(rows)
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

/// `AdwShortcutsDialog` content: sections of `(label, accelerator)`.
pub fn shortcuts_dialog<'a, M: Clone + 'a>(
    sections: Vec<(String, Vec<(String, String)>)>,
    on_close: M,
) -> Element<'a, M> {
    let m = Metrics::desktop();
    let t = Typography::default();
    let mut col = column![].spacing(m.space_l);
    for (title, items) in sections {
        let mut list = column![t.heading(title)].spacing(m.space_s);
        for (label, accel) in items {
            list = list.push(
                row![
                    container(t.body(label)).width(Length::Fill),
                    shortcut_label(&accel)
                ]
                .align_y(Vertical::Center),
            );
        }
        col = col.push(list);
    }
    dialog_frame("Keyboard Shortcuts", col.into(), on_close, 480.0, 420.0)
}

/// `AdwPreferencesDialog`: a dialog with a header bar wrapping a page.
pub fn preferences_dialog<'a, M: Clone + 'a>(
    title: impl Into<String>,
    page: impl Into<Element<'a, M>>,
    on_close: M,
) -> Element<'a, M> {
    dialog_frame(&title.into(), page.into(), on_close, 640.0, 520.0)
}

fn dialog_frame<'a, M: Clone + 'a>(
    title: &str,
    body: Element<'a, M>,
    on_close: M,
    width: f32,
    height: f32,
) -> Element<'a, M> {
    // `AdwHeaderBar` in a dialog: the dialog title and only a close button, on the
    // side the system puts it (end by default). The header is flat so it shares the
    // dialog's background.
    let close_at_start = DecorationLayout::detect().start.contains(&WindowButton::Close);
    let close = |at: bool| if at { vec![WindowButton::Close] } else { vec![] };
    let layout = DecorationLayout { start: close(close_at_start), end: close(!close_at_start) };
    let bar = header_bar()
        .title(title.to_string())
        // Flat: the header flows into the dialog body instead of having its own color.
        .flat(true)
        .window_controls(super::WindowControls {
            on_close: Some(on_close),
            layout: Some(layout),
            ..Default::default()
        });
    let content = container(body).padding(Metrics::desktop().space_l);
    container(toolbar_view(super::scrolled_window(content)).top_bar(bar))
        .width(Length::Fill)
        .height(Length::Fill)
        .max_width(width)
        .max_height(height)
        .clip(true)
        .style(sc::dialog)
        .into()
}

/// A Nautilus-style path bar: breadcrumb segments with the current one
/// highlighted, selecting one sends its index. Meant as a [`HeaderBar`](super::HeaderBar)
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

/// `AdwTabBar`: a row of tabs, each optionally closable.
pub fn tab_bar<'a, M: Clone + 'a>(
    tabs: Vec<String>,
    active: usize,
    on_select: impl Fn(usize) -> M + 'a,
    on_close: Option<impl Fn(usize) -> M + 'a>,
) -> Element<'a, M> {
    let m = Metrics::desktop();
    let mut r = row![].spacing(m.space_xs).padding(m.space_xs).align_y(Vertical::Center);
    for (i, title) in tabs.into_iter().enumerate() {
        let mut label = row![container(super::typography::label(title)).width(Length::Fill)]
            .spacing(m.space_s)
            .align_y(Vertical::Center);
        if let Some(close) = &on_close {
            label = label.push(
                button(Icon::WindowClose.view(12.0))
                    .padding(m.space_xs)
                    .style(sb::circular_flat)
                    .on_press(close(i)),
            );
        }
        r = r.push(
            button(label)
                .padding([m.space_xs, m.space_m])
                .width(Length::FillPortion(1))
                .style(sb::flat_toggle(i == active))
                .on_press(on_select(i)),
        );
    }
    container(r).width(Length::Fill).style(sc::headerbar).into()
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

/// `AdwBottomSheet`: slides a panel over the bottom of `content` when `open`.
pub fn bottom_sheet<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M>>,
    sheet: impl Into<Element<'a, M>>,
    open: bool,
    on_close: M,
) -> Element<'a, M> {
    bottom_sheet_in_window(content, sheet, open, on_close, 0.0)
}

/// Like [`bottom_sheet`], for a window with rounded corners: the scrim and the
/// panel's bottom corners take the window radius so nothing paints outside it.
pub fn bottom_sheet_in_window<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M>>,
    sheet: impl Into<Element<'a, M>>,
    open: bool,
    on_close: M,
    window_radius: f32,
) -> Element<'a, M> {
    if !open {
        return content.into();
    }
    let m = Metrics::desktop();
    let catcher = mouse_area(container(space()).width(Length::Fill).height(Length::Fill)).on_press(on_close);
    let panel = container(sheet.into())
        .padding(m.space_l)
        .width(Length::Fill)
        .style(move |t: &Theme| iced::widget::container::Style {
            border: iced::Border {
                radius: iced::border::Radius {
                    top_left: t.metrics.radius_l,
                    top_right: t.metrics.radius_l,
                    bottom_right: window_radius,
                    bottom_left: window_radius,
                },
                ..Default::default()
            },
            ..sc::dialog(t)
        });
    // Slides up on the sheet spring with a fading scrim (`AdwBottomSheet`).
    let layer = stack![
        opaque(catcher),
        container(opaque(panel)).height(Length::Fill).align_y(Vertical::Bottom)
    ];
    stack![
        content.into(),
        Element::from(super::entrance(layer, iced::Vector::new(0.0, 160.0)).scrim(window_radius))
    ]
    .into()
}
