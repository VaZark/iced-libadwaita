//! Windows 95: the "Display Properties" dialog. Tabs, a preview monitor, a list box, radio
//! buttons, check boxes, sunken fields, OK / Cancel / Apply and a status bar. Every bevel, color
//! and metric comes from the theme.

use crate::common::*;
use crate::design::px;
fn g(key: &str) -> f32 {
    px("toy.win95", key)
}
use iad::components as cmp;
use iad::widgets as w;
use iad::{Icon, Metrics};
use iced::widget::{column, container, row, space};
use iced::{Alignment, Length};
use iced_adaptive_design as iad;

pub const TITLE: &str = "Display Properties";
const WALLPAPERS: [&str; 5] = ["(None)", "Arcade", "Argyle", "Black Thatch", "Blue Rivets"];
const MODES: [&str; 3] = ["Center", "Tile", "Stretch"];

pub fn view(t: &Toy) -> El<'_> {
    let m = Metrics::desktop();
    let tabs = w::Tabs::new(Some(t.tab), Msg::Tab)
        .panel_spacing(g("tab-panel-gap"))
        .label_props(cmp::TextProps::default().size(iad::Typography::desktop().body))
        .padding([0.0, m.space_s])
        .variant(w::TabsVariant::Default)
        .tab(w::TabItem::new(0usize, "Themes").panel(move || appearance(t)))
        .tab(w::TabItem::new(1usize, "Desktop").panel(move || background(t)))
        .tab(w::TabItem::new(2usize, "Screen Saver").panel(move || screen_saver(t)))
        .tab(w::TabItem::new(3usize, "Appearance").panel(move || appearance(t)))
        .tab(w::TabItem::new(4usize, "Settings").panel(move || appearance(t)));
    let buttons = row![
        space().width(Length::Fill),
        cmp::raised_button(label("OK"))
            .on_press(Msg::Apply)
            .width(g("command-width")),
        cmp::raised_button(label("Cancel"))
            .on_press(Msg::CancelDisplay)
            .width(g("command-width")),
        cmp::raised_button(label("Apply"))
            .on_press(Msg::Apply)
            .width(g("command-width")),
    ]
    .spacing(m.space_s);
    let page = column![Element::from(tabs), El::from(buttons)]
        .spacing(m.space_m)
        .padding(m.inset);
    let dialog: El<'_> = if t.section == 1 {
        w::AlertDialog::new("Choose background")
            .body("Bundled preview backgrounds; no files are changed.")
            .extra_child(column(WALLPAPERS.iter().enumerate().map(|(i, name)| {
                El::from(
                    cmp::flat_button(label(*name))
                        .width(Length::Fill)
                        .on_press(Msg::Pick(i)),
                )
            })))
            .response("Close", w::ResponseAppearance::Default, Msg::Dialog(false))
            .into()
    } else {
        w::AlertDialog::new("Cancel changes?")
            .body("Your changes to the display settings will be lost.")
            .response(
                "Keep editing",
                w::ResponseAppearance::Default,
                Msg::Dialog(false),
            )
            .response("Discard", w::ResponseAppearance::Destructive, Msg::Discard)
            .into()
    };
    let content = column![El::from(page), space().height(Length::Fill)].into();
    window(t, TITLE, overlays(t, content, dialog))
}

fn color_menu(t: &Toy) -> w::Menu<Msg> {
    native_menu("Background color").section([
        w::MenuItem::select("Blue", !t.checks[5], Msg::Check(5, false)).keep_open(false),
        w::MenuItem::select("Green", t.checks[5], Msg::Check(5, true)).keep_open(false),
    ])
}

fn background(t: &Toy) -> El<'_> {
    let m = Metrics::desktop();
    let nodes: Vec<_> = WALLPAPERS
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let b = cmp::flat_button(label(*n))
                .padding([0.0, m.space_s])
                .height(g("list-row-height"))
                .width(Length::Fill)
                .on_press(Msg::Pick(i));
            if i == t.pick {
                cmp::variant_button(label(*n), "opaque")
                    .padding([0.0, m.space_s])
                    .height(g("list-row-height"))
                    .width(Length::Fill)
                    .on_press(Msg::Pick(i))
            } else {
                b
            }
        })
        .map(El::from)
        .collect();
    let list = cmp::framed(
        container(iced::widget::scrollable(column(nodes)))
            .height(g("list-height"))
            .padding(g("list-inset"))
            .width(Length::Fill)
            .style(iad::styles::container::view),
        "field",
    );
    let display = column![
        cmp::raised_button(label("Browse…"))
            .width(Length::Fill)
            .on_press(Msg::BrowseWallpaper),
        space().height(m.space_s),
        label("Position:"),
        w::dropdown(&MODES, Some(MODES[t.radio.min(2)]), |s| Msg::Radio(
            MODES.iter().position(|mode| *mode == s).unwrap_or(0)
        )),
        label("Color:"),
        menu_trigger(
            if t.checks[5] { "Green ▾" } else { "Blue ▾" },
            color_menu(t),
            m.button_min_height,
        ),
    ]
    .spacing(m.space_xs)
    .width(g("group-width"));
    let controls = row![
        column![
            label("Background:"),
            list,
            cmp::raised_button(label("Customize Desktop…")).on_press(Msg::Toast(Some(
                "Desktop icon preferences are a preview only".into()
            )))
        ]
        .spacing(m.space_s)
        .width(Length::Fill),
        display,
    ]
    .spacing(m.gap)
    .align_y(Alignment::Start);
    container(
        column![
            container(monitor(t))
                .width(Length::Fill)
                .center_x(Length::Fill),
            controls,
        ]
        .spacing(g("preview-gap")),
    )
    .padding(g("panel-padding"))
    .width(Length::Fill)
    .into()
}

/// Rounded CRT casing, recessed glass, neck and foot; no raster artwork required.
fn monitor(t: &Toy) -> El<'_> {
    let wallpaper = t.pick.min(WALLPAPERS.len() - 1);
    let motif = |width: f32, height: f32| {
        container(space().width(width).height(height)).style(move |theme: &iad::Theme| {
            container::Style {
                background: Some(theme.semantic.accent.bg.into()),
                border: iced::Border {
                    radius: g("screen-radius").into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
    };
    let picture: El<'_> = if wallpaper == 0 {
        space().into()
    } else if t.radio == 1 {
        column(
            (0..(g("screen-height") / (g("tile-height") + g("tile-gap"))) as usize).map(|_| {
                row(
                    (0..(g("screen-width") / (g("tile-width") + g("tile-gap"))) as usize)
                        .map(|_| El::from(motif(g("tile-width"), g("tile-height")))),
                )
                .spacing(g("tile-gap"))
                .into()
            }),
        )
        .spacing(g("tile-gap"))
        .into()
    } else {
        motif(
            if t.radio == 2 {
                g("screen-width")
            } else {
                g("picture-width")
            },
            if t.radio == 2 {
                g("screen-height")
            } else {
                g("picture-height")
            },
        )
        .into()
    };
    let green = t.checks[5];
    let glass = container(picture)
        .center_x(g("screen-width"))
        .center_y(g("screen-height"))
        .clip(true)
        .style(move |theme: &iad::Theme| container::Style {
            background: Some(if green {
                theme.semantic.success.bg.into()
            } else {
                theme
                    .token_color("frame.well", "bg")
                    .unwrap_or(theme.semantic.view.bg)
                    .into()
            }),
            border: iced::Border {
                radius: g("screen-radius").into(),
                ..Default::default()
            },
            ..Default::default()
        });
    let bezel = container(
        column![
            container(cmp::framed(glass, "well")).center_x(Length::Fill),
            row![space().width(Length::Fill), caption("●")].padding([0.0, g("indicator-padding")]),
        ]
        .spacing(g("indicator-gap")),
    )
    .width(g("monitor-width"))
    .padding(g("bezel-padding"))
    .style(|theme: &iad::Theme| container::Style {
        background: Some(theme.semantic.window.bg.into()),
        border: iced::Border {
            color: theme.semantic.window.fg.scale_alpha(0.65),
            width: g("border-width"),
            radius: g("bezel-radius").into(),
        },
        ..Default::default()
    });
    column![
        bezel,
        cmp::framed(
            container(space().width(g("neck-width")).height(g("neck-height"))),
            "panel"
        ),
        cmp::framed(
            container(space().width(g("foot-width")).height(g("foot-height"))),
            "panel"
        ),
    ]
    .width(g("monitor-width"))
    .align_x(Alignment::Center)
    .into()
}

fn radio<'a>(t: &'a Toy, i: usize, text: &'static str) -> El<'a> {
    cmp::themed_radio(text, i, Some(t.radio), Msg::Radio)
}

fn screen_saver(t: &Toy) -> El<'_> {
    let m = Metrics::desktop();
    column![
        label("Screen Saver"),
        w::dropdown(
            &["(None)", "Flying Windows", "Mystify", "3D Pipes"],
            Some(t.drops[0].as_str())
                .filter(|s| ["(None)", "Flying Windows", "Mystify", "3D Pipes"].contains(s))
                .or(Some("(None)")),
            |s| Msg::Drop(0, s.to_string())
        ),
        cmp::check_button("Password protected", t.checks[0]).on_toggle(|b| Msg::Check(0, b)),
        row![
            label("Wait:"),
            cmp::entry("", t.text.as_str())
                .on_input(Msg::Text)
                .width(g("wait-width")),
            label("minutes")
        ]
        .spacing(m.space_s)
        .align_y(Alignment::Center),
        cmp::themed_progress(0.0..=1.0, t.progress),
    ]
    .spacing(m.space_m)
    .into()
}

fn appearance(t: &Toy) -> El<'_> {
    let m = Metrics::desktop();
    column![
        label("Item:"),
        radio(t, 0, "Normal windows"),
        radio(t, 1, "Large windows"),
        cmp::scale(0.0..=100.0, t.sliders[0], |v| Msg::Slide(0, v)),
        cmp::check_button("Use large icons", t.checks[1]).on_toggle(|b| Msg::Check(1, b)),
        cmp::check_button("Show window contents while dragging", t.checks[2])
            .on_toggle(|b| Msg::Check(2, b)),
        row![
            Icon::Settings.view(m.icon_size),
            label("Window metrics follow the theme")
        ]
        .spacing(m.space_s),
    ]
    .spacing(m.space_m)
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_menu_is_an_explicit_choice_not_a_blind_toggle_or_dialog() {
        let mut t = Toy::new();
        for row in [1, 1, 0] {
            let (_, command) = crate::common::menu_tests::activate(
                menu_trigger(
                    "Color",
                    color_menu(&t),
                    Metrics::desktop().button_min_height,
                ),
                0,
                false,
                row,
            );
            t.update(command);
            assert_eq!(t.checks[5], row == 1);
            assert!(!t.dialog);
        }
        t.update(Msg::Apply);
        t.update(Msg::Check(5, true));
        t.update(Msg::Discard);
        assert!(!t.checks[5]);
    }
}
