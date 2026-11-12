//! `icon_color_picker`: what a folder, epic or list looks like, in one swatch: its icon in its
//! colour. Pressed, it opens a popover with the icons of [`ICON_CHOICES`] over the colours: a row
//! of suggested ones and a field for any `#rrggbb`. What is picked for the icon is its name (the
//! default folder is none), so only the name need be stored.

use crate::widgets::ColorInput;
use crate::{Element, Icon, ICON_CHOICES, Theme};
use crate::styles::button as sb;
use crate::widgets::{Popover, popover};
use iced::widget::{button, column, container, row, space, svg};
use iced::{Alignment, Background, Border, Color, Length};

use crate::primitives::color_picker::Hsva;

/// How many icons are in a row of the grid.
const PER_ROW: usize = 6;

/// The room one icon or swatch takes, with its padding and spacing.
const CELL: f32 = 36.0;

/// The colours suggested beside the free choice (`#rrggbb`).
pub const SUGGESTED: [&str; 8] = [
    "#8e99a8", "#6aa5e8", "#7fc97f", "#f2d45c", "#f0a35e", "#e0787a", "#a98be0", "#e88fc0",
];

/// The three bytes of a `#rrggbb` that [`SUGGESTED`] holds.
fn rgb8(hex: &str) -> [u8; 3] {
    let v = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0);
    [(v >> 16) as u8, (v >> 8) as u8, v as u8]
}

/// A circle `size` across in `color`, with a ring round it: the one that opens the picker for any
/// colour, set apart from the suggestions.
fn ring<'a, M: 'a>(color: Color, size: f32) -> Element<'a, M> {
    container(space().width(size).height(size))
        .style(move |t: &Theme| container::Style {
            background: Some(Background::Color(color)),
            border: Border {
                color: t.semantic.window.fg,
                width: 2.0,
                radius: (size / 2.0).into(),
            },
            ..container::Style::default()
        })
        .into()
}

/// A filled circle `size` across.
fn dot<'a, M: 'a>(color: Color, size: f32) -> Element<'a, M> {
    container(space().width(size).height(size))
        .style(move |_: &Theme| container::Style {
            background: Some(Background::Color(color)),
            border: Border::default().rounded(size / 2.0),
            ..container::Style::default()
        })
        .into()
}

/// `icon` (a name from [`ICON_CHOICES`], or none for the default) in `color`, `size` across;
/// pressed, it opens the icons over the colours. `typed` is the text the colour picker's own field shows.
/// `on_icon` is given the name chosen (`None` for the default folder); `on_text` the colour
/// field's text as it is typed, and `on_pick` a suggested colour pressed.
pub fn icon_color_picker<'a, M: Clone + 'a>(
    icon: Option<&str>,
    color: Color,
    typed: &'a str,
    size: f32,
    on_icon: impl Fn(Option<&'static str>) -> M + 'a,
    on_text: impl Fn(String) -> M + 'a,
    on_pick: impl Fn(Hsva) -> M + 'a,
) -> Element<'a, M> {
    let trigger = container(
        Icon::from_choice(icon)
            .view(size)
            .style(move |_: &Theme, _| svg::Style { color: Some(color) }),
    )
    .padding(4)
    .center_y(Length::Shrink);

    // The icons, in rows.
    let mut icons = column![].spacing(2);
    let mut line = row![].spacing(2).align_y(Alignment::Center);
    let mut n = 0;
    for (i, c) in ICON_CHOICES.iter().enumerate() {
        // The first is the default folder: choosing it stores nothing.
        let cell: Element<'a, M> = crate::components::tooltip(
            button(
                c.icon
                    .view(18.0)
                    .style(move |_: &Theme, _| svg::Style { color: Some(color) }),
            )
            .padding(8)
            .style(sb::flat)
            .on_press(on_icon((i != 0).then_some(c.name))),
            c.label,
            crate::components::TooltipPosition::Bottom,
        );
        line = line.push(cell);
        n += 1;
        if n % PER_ROW == 0 {
            icons = icons.push(std::mem::replace(
                &mut line,
                row![].spacing(2).align_y(Alignment::Center),
            ));
        }
    }
    if n % PER_ROW != 0 {
        icons = icons.push(line);
    }

    // The colours, at the bottom: the suggestions, then a last circle that opens the picker for
    // any colour.
    let on_pick = std::rc::Rc::new(on_pick);
    let mut colours = row![].spacing(2).align_y(Alignment::Center);
    for hex in SUGGESTED {
        let [r, g, b] = rgb8(hex);
        let c = Color::from_rgb8(r, g, b);
        let pick = on_pick.clone();
        colours = colours.push(
            button(dot(c, 18.0))
                .padding(8)
                .style(sb::flat)
                .on_press(pick(Hsva::from_rgb8(r, g, b))),
        );
    }
    let any = on_pick.clone();
    let [r, g, b] = [color.r, color.g, color.b].map(|v| (v * 255.0).round() as u8);
    colours = colours.push(
        container(
            ColorInput::new(Hsva::from_rgb8(r, g, b), typed, on_text, move |h| any(h))
                .swatches(SUGGESTED)
                .trigger(ring(color, 18.0)),
        )
        .padding(8),
    );
    let content = column![icons, crate::components::divider(), colours.wrap()]
        .spacing(8)
        .width(Length::Fixed(PER_ROW as f32 * CELL));

    Popover::new(trigger, popover([Element::from(content)], 420.0))
        // As wide as the grid and the popover's padding.
        .popover_width(PER_ROW as f32 * CELL + 28.0)
        // For screenshots: open it at once.
        .open_initially(std::env::var_os("ICED_ADAPTIVE_DESIGN_ICON_PICKER_OPEN").is_some())
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_suggestions_read_as_colours() {
        assert_eq!(rgb8("#6aa5e8"), [0x6a, 0xa5, 0xe8]);
        assert!(SUGGESTED.iter().all(|h| h.len() == 7 && h.starts_with('#')));
    }
}
