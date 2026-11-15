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
    view_switcher_bar_with(items, active, on_select, Metrics::desktop().space_s)
}

/// [`view_switcher_bar`] with `padding` around the switcher (default: the theme's `space_s`).
pub fn view_switcher_bar_with<'a, M: Clone + 'a, T: Clone + PartialEq + 'a>(
    items: Vec<ToggleItem<'a, T, M>>,
    active: Option<T>,
    on_select: impl Fn(T) -> M + 'a,
    padding: impl Into<iced::Padding>,
) -> Element<'a, M> {
    view_switcher_bar_with_props(
        items,
        active,
        on_select,
        padding,
        ViewSwitcherBarWidgetProps::default(),
    )
}

/// Native presentation overrides applied after defaults.
pub fn view_switcher_bar_with_props<'a, M: Clone + 'a, T: Clone + PartialEq + 'a>(
    items: Vec<ToggleItem<'a, T, M>>,
    active: Option<T>,
    on_select: impl Fn(T) -> M + 'a,
    padding: impl Into<iced::Padding>,
    props: ViewSwitcherBarWidgetProps,
) -> Element<'a, M> {
    props
        .container
        .apply(
            container(Element::from(
                props.switcher.apply(toggle_group(items, active, on_select)),
            ))
            .padding(padding)
            .center_x(Length::Fill)
            .style(sc::headerbar),
        )
        .into()
}

/// `AdwSidebar` / `AdwViewSwitcherSidebar`: same as
/// [`navigation_sidebar`](crate::widgets::navigation_sidebar).
pub fn sidebar<'a, M: 'a>(rows: impl IntoIterator<Item = Element<'a, M>>) -> Element<'a, M> {
    crate::widgets::navigation_sidebar(rows)
}

/// `AdwShortcutLabel`: renders an accelerator like `Ctrl+Shift+P` as keycaps.
pub fn shortcut_label<'a, M: 'a>(accelerator: &str) -> Element<'a, M> {
    let m = Metrics::desktop();
    shortcut_label_with(accelerator, m.space_xs, [m.space_xxs, m.space_s])
}

/// [`shortcut_label`] with `spacing` between the keycaps and `padding` inside each (default: the
/// theme's `space_xs`, and `space_xxs` by `space_s`).
pub fn shortcut_label_with<'a, M: 'a>(
    accelerator: &str,
    spacing: f32,
    padding: impl Into<iced::Padding>,
) -> Element<'a, M> {
    shortcut_label_with_props(
        accelerator,
        spacing,
        padding,
        ShortcutLabelWidgetProps::default(),
    )
}

/// Native presentation overrides applied after defaults.
pub fn shortcut_label_with_props<'a, M: 'a>(
    accelerator: &str,
    spacing: f32,
    padding: impl Into<iced::Padding>,
    props: ShortcutLabelWidgetProps,
) -> Element<'a, M> {
    let t = Typography::desktop();
    let padding = padding.into();
    let mut r = row![].spacing(spacing).align_y(Vertical::Center);
    for (i, key) in accelerator.split('+').enumerate() {
        if i > 0 {
            r = r.push(props.separator.apply(t.caption("+").style(st::dim)));
        }
        r = r.push(
            props.keycap.apply(
                container(props.label.apply(t.caption(key.trim().to_string())))
                    .padding(padding)
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
            ),
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
    path_bar_with_props(segments, current, on_select, PathBarWidgetProps::default())
}

/// Native presentation overrides applied after defaults.
pub fn path_bar_with_props<'a, M: Clone + 'a>(
    segments: Vec<String>,
    current: usize,
    on_select: impl Fn(usize) -> M + 'a,
    props: PathBarWidgetProps,
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
    Element::from(
        props
            .switcher
            .apply(toggle_group(items, Some(current), on_select)),
    )
}

/// `AdwCarouselIndicatorDots`.
pub fn carousel_indicator_dots<'a, M: Clone + 'a>(
    count: usize,
    index: usize,
    on_navigate: impl Fn(usize) -> M + 'a,
) -> Element<'a, M> {
    carousel_indicator_dots_with_props(
        count,
        index,
        on_navigate,
        CarouselIndicatorWidgetProps::default(),
    )
}

/// `AdwCarouselIndicatorLines`.
pub fn carousel_indicator_lines<'a, M: Clone + 'a>(
    count: usize,
    index: usize,
    on_navigate: impl Fn(usize) -> M + 'a,
) -> Element<'a, M> {
    carousel_indicator_lines_with_props(
        count,
        index,
        on_navigate,
        CarouselIndicatorWidgetProps::default(),
    )
}

fn indicator<'a, M: Clone + 'a>(
    count: usize,
    index: usize,
    on_navigate: impl Fn(usize) -> M + 'a,
    w: f32,
    h: f32,
    props: CarouselIndicatorWidgetProps,
) -> Element<'a, M> {
    let mut r = row![]
        .spacing(crate::Metrics::desktop().space_s)
        .align_y(Vertical::Center);
    for i in 0..count {
        let active = i == index;
        r = r.push(
            props.button.apply(
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
            ),
        );
    }
    props
        .container
        .apply(container(r).padding(crate::Metrics::desktop().inset))
        .into()
}

/// `AdwTabButton`: shows the open tab count, e.g. to open a tab overview.
pub fn tab_button<'a, M: Clone + 'a>(count: usize, on_press: M) -> Element<'a, M> {
    tab_button_with_props(count, on_press, TabButtonWidgetProps::default())
}

/// Native presentation overrides applied after defaults.
pub fn tab_button_with_props<'a, M: Clone + 'a>(
    count: usize,
    on_press: M,
    props: TabButtonWidgetProps,
) -> Element<'a, M> {
    let m = Metrics::desktop();
    props
        .button
        .apply(
            button(
                props.container.apply(
                    container(props.label.apply(text(count.to_string()).size(12)))
                        .center(Length::Fixed(20.0))
                        .style(|t: &Theme| iced::widget::container::Style {
                            border: iced::Border {
                                color: t.semantic.window.fg,
                                width: 2.0,
                                radius: t.metrics.radius_s.into(),
                            },
                            ..Default::default()
                        }),
                ),
            )
            .padding(m.space_s)
            .style(sb::flat)
            .on_press(on_press),
        )
        .into()
}

/// Typed presentation for parent-built native children. Defaults preserve the original view.
#[derive(Clone, Default)]
pub struct ViewSwitcherBarWidgetProps {
    pub container: crate::components::ContainerProps,
    pub switcher: crate::components::button::ToggleGroupProps,
}

/// Typed presentation for parent-built native children. Defaults preserve the original view.
#[derive(Clone, Default)]
pub struct ShortcutLabelWidgetProps {
    pub separator: crate::components::TextProps,
    pub keycap: crate::components::ContainerProps,
    pub label: crate::components::TextProps,
}

/// Typed presentation for parent-built native children. Defaults preserve the original view.
#[derive(Clone, Default)]
pub struct TabButtonWidgetProps {
    pub button: crate::components::ButtonProps,
    pub container: crate::components::ContainerProps,
    pub label: crate::components::TextProps,
}

/// Typed presentation for parent-built native children. Defaults preserve the original view.
#[derive(Clone, Default)]
pub struct CarouselIndicatorWidgetProps {
    pub button: crate::components::ButtonProps,
    pub container: crate::components::ContainerProps,
}

/// Carousel indicator with typed native presentation.
pub fn carousel_indicator_dots_with_props<'a, M: Clone + 'a>(
    count: usize,
    index: usize,
    on_navigate: impl Fn(usize) -> M + 'a,
    props: CarouselIndicatorWidgetProps,
) -> Element<'a, M> {
    indicator(count, index, on_navigate, 8.0, 8.0, props)
}

/// Carousel indicator with typed native presentation.
pub fn carousel_indicator_lines_with_props<'a, M: Clone + 'a>(
    count: usize,
    index: usize,
    on_navigate: impl Fn(usize) -> M + 'a,
    props: CarouselIndicatorWidgetProps,
) -> Element<'a, M> {
    indicator(count, index, on_navigate, 24.0, 3.0, props)
}

#[cfg(test)]
mod native_props_tests {
    use super::*;
    use crate::components::{ButtonProps, TextProps};
    use crate::widgets::synced_text::native_props_tests::{assert_override, text_props};

    #[test]
    fn extra_labels_are_native_and_default_wrappers_preserve_layout() {
        assert_override(
            tab_button(7, ()),
            tab_button_with_props(7, (), TabButtonWidgetProps::default()),
            tab_button_with_props(
                7,
                (),
                TabButtonWidgetProps {
                    label: text_props(),
                    container: crate::components::ContainerProps::default()
                        .width(200)
                        .height(60),
                    ..Default::default()
                },
            ),
        );
        assert_override(
            shortcut_label("Ctrl+界"),
            shortcut_label_with_props(
                "Ctrl+界",
                Metrics::desktop().space_xs,
                [Metrics::desktop().space_xxs, Metrics::desktop().space_s],
                ShortcutLabelWidgetProps::default(),
            ),
            shortcut_label_with_props(
                "Ctrl+界",
                Metrics::desktop().space_xs,
                [Metrics::desktop().space_xxs, Metrics::desktop().space_s],
                ShortcutLabelWidgetProps {
                    label: text_props(),
                    ..Default::default()
                },
            ),
        );
    }

    #[test]
    fn indicator_and_tab_button_overrides_keep_navigation_messages() {
        use crate::widgets::synced_text::native_props_tests::click_sized;
        let props = CarouselIndicatorWidgetProps {
            button: ButtonProps::default().width(137).height(29),
            ..Default::default()
        };
        assert_eq!(
            click_sized(carousel_indicator_dots_with_props(
                3,
                0,
                |i| i,
                props.clone()
            )),
            vec![0]
        );
        assert_eq!(
            click_sized(carousel_indicator_lines_with_props(3, 0, |i| i, props)),
            vec![0]
        );
        assert_eq!(
            click_sized(tab_button_with_props(
                7,
                42,
                TabButtonWidgetProps {
                    button: ButtonProps::default().width(137).height(29),
                    label: TextProps::default().size(20),
                    ..Default::default()
                }
            )),
            vec![42]
        );
    }
}

/// Typed presentation for parent-built native children. Defaults preserve the original view.
#[derive(Clone, Default)]
pub struct PathBarWidgetProps {
    pub switcher: crate::components::button::ToggleGroupProps,
}
