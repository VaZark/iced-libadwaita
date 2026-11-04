//! `GtkDropDown` and the items of a popover list: a [`Popover`] whose trigger is a button
//! (label + chevron) and whose content is a `.menu` list below it: rounded surface, 6px padding,
//! items with a hover layer and a trailing check on the selected one.

use crate::Element;
use crate::styles::button as sb;
use crate::widgets::popover::{Popover, popover};
use crate::{Icon, Metrics, Theme};
use iced::Length;
use iced::alignment::Vertical;
use iced::widget::{button, container, row, space, text};

/// A menu item for a dropdown popover: label with a trailing check when selected. The
/// selected item is the active one, in the accent color.
pub fn menu_item<'a, M: Clone + 'a>(
    label: impl Into<String>,
    selected: bool,
    on_press: M,
) -> Element<'a, M> {
    menu_item_inner(label.into(), selected, Some(on_press))
}

/// A menu item that cannot be chosen: dimmed text, no hover or press effect, no message.
/// (Distinct from selected, which is accent-colored.)
pub fn menu_item_disabled<'a, M: Clone + 'a>(
    label: impl Into<String>,
    selected: bool,
) -> Element<'a, M> {
    menu_item_inner(label.into(), selected, None)
}

fn menu_item_inner<'a, M: Clone + 'a>(
    label: String,
    selected: bool,
    on_press: Option<M>,
) -> Element<'a, M> {
    let m = Metrics::desktop();
    let mut r = row![
        text(label)
            .wrapping(text::Wrapping::None)
            .width(Length::Fill)
    ]
    .spacing(m.space_m)
    .align_y(Vertical::Center);
    r = r.push(if selected {
        // The check follows the text color (accent when active, dimmed when disabled).
        let disabled = on_press.is_none();
        Element::from(Icon::Check.view(m.icon_size).style(move |t: &Theme, _| {
            iced::widget::svg::Style {
                color: Some(if disabled {
                    crate::color::fade(t.semantic.accent.color, t.semantic.opacity.disabled)
                } else {
                    t.semantic.accent.color
                }),
            }
        }))
    } else {
        space().width(m.icon_size).into()
    });
    button(
        container(r)
            .padding([0.0, m.space_m])
            .center_y(Length::Fixed(32.0)),
    )
    .padding(0)
    .width(Length::Fill)
    .style(sb::menu_item(selected))
    .on_press_maybe(on_press)
    .into()
}

/// A `GtkDropDown` that is insensitive: its value and arrow are dimmed and it cannot open.
pub fn dropdown_disabled<'a, T, M>(selected: Option<&T>) -> Popover<'a, M>
where
    T: ToString,
    M: Clone + 'a,
{
    let m = Metrics::desktop();
    let current = selected.map(ToString::to_string).unwrap_or_default();
    let trigger = container(
        row![
            text(current)
                .wrapping(text::Wrapping::None)
                .style(crate::styles::text::dim),
            Icon::GoDown
                .view(m.icon_size)
                .style(|t: &Theme, _| iced::widget::svg::Style {
                    color: Some(crate::color::fade(
                        t.semantic.window.fg,
                        t.semantic.opacity.disabled
                    )),
                })
        ]
        .spacing(m.space_s)
        .align_y(Vertical::Center),
    )
    .center_y(Length::Fixed(m.button_min_height - m.button_padding.y()));
    Popover::new(trigger, popover(Vec::<Element<'a, M>>::new(), 0.0)).disabled(true)
}

/// The shared core of [`dropdown`] and [`MultiSelect`](crate::widgets::MultiSelect): one
/// [`menu_item`] per option, checked by `is_selected`, sending `on_press(option)`.
pub(crate) fn option_items<'a, T, M>(
    options: &'a [T],
    is_selected: impl Fn(&T) -> bool,
    on_press: impl Fn(T) -> M,
) -> Vec<Element<'a, M>>
where
    T: ToString + Clone + 'a,
    M: Clone + 'a,
{
    options
        .iter()
        .map(|o| menu_item(o.to_string(), is_selected(o), on_press(o.clone())))
        .collect()
}

/// `GtkDropDown` over a slice of options.
pub fn dropdown<'a, T, M>(
    options: &'a [T],
    selected: Option<T>,
    on_select: impl Fn(T) -> M + 'a,
) -> Popover<'a, M>
where
    T: ToString + PartialEq + Clone + 'a,
    M: Clone + 'a,
{
    let m = Metrics::desktop();
    let current = selected.as_ref().map(|s| s.to_string()).unwrap_or_default();
    let trigger = container(
        row![
            text(current).wrapping(text::Wrapping::None),
            Icon::GoDown.view(m.icon_size)
        ]
        .spacing(m.space_s)
        .align_y(Vertical::Center),
    )
    .center_y(Length::Fixed(m.button_min_height - m.button_padding.y()));
    let items = option_items(options, |o| selected.as_ref() == Some(o), on_select);
    Popover::new(trigger, popover(items, 280.0))
}
