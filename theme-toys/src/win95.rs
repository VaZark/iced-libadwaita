//! Windows 95: the "Display Properties" dialog. Tabs, a preview monitor, a list box, radio
//! buttons, check boxes, sunken fields, OK / Cancel / Apply and a status bar. Every bevel, color
//! and metric comes from the theme.

use crate::common::*;
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
        .variant(w::TabsVariant::Outline)
        .tab(w::TabItem::new(0usize, "Background").panel(move || background(t)))
        .tab(w::TabItem::new(1usize, "Screen Saver").panel(move || screen_saver(t)))
        .tab(w::TabItem::new(2usize, "Appearance").panel(move || appearance(t)));
    let buttons = row![
        space().width(Length::Fill),
        cmp::raised_button(cmp::typography::label("OK")).on_press(Msg::Status("OK".into())).width(75),
        cmp::raised_button(cmp::typography::label("Cancel")).on_press(Msg::Dialog(true)).width(75),
        cmp::raised_button(cmp::typography::label("Apply")).on_press(Msg::Toast(Some("Settings applied".into()))).width(75),
    ]
    .spacing(m.space_s);
    let page = column![Element::from(tabs), El::from(buttons)].spacing(m.space_m).padding(m.inset);
    let dialog: El<'_> = w::AlertDialog::new("Cancel changes?")
        .body("Your changes to the display settings will be lost.")
        .response("Keep editing", w::ResponseAppearance::Default, Msg::Dialog(false))
        .response("Discard", w::ResponseAppearance::Destructive, Msg::Dialog(false))
        .into();
    let content = column![El::from(page.height(Length::Fill)), status_bar(t.status.clone(), "Display")].into();
    window(t, TITLE, overlays(t, content, dialog))
}

fn background(t: &Toy) -> El<'_> {
    let m = Metrics::desktop();
    let monitor = container(
        column![
            container(label(WALLPAPERS[t.pick.min(4)])).padding(m.space_l),
            space().height(Length::Fill),
            iad::Icon::Home.view(16.0),
        ]
        .align_x(Alignment::Center),
    )
    .width(200)
    .height(130)
    .padding(m.space_m)
    .style(iad::styles::container::card);
    let nodes: Vec<_> = WALLPAPERS
        .iter()
        .enumerate()
        .map(|(i, n)| {
            let b = cmp::flat_button(label(*n)).width(Length::Fill).on_press(Msg::Pick(i));
            if i == t.pick { cmp::raised_button(label(*n)).width(Length::Fill).on_press(Msg::Pick(i)) } else { b }
        })
        .map(El::from)
        .collect();
    let list = container(column(nodes)).padding(m.space_xs).width(Length::Fill).style(iad::styles::container::card);
    let display = column![
        label("Display:"),
        radio(t, 0, MODES[0]),
        radio(t, 1, MODES[1]),
        radio(t, 2, MODES[2]),
    ]
    .spacing(m.space_xs);
    row![
        column![Element::from(monitor), heading("Pattern")].spacing(m.space_s),
        column![label("Wallpaper"), Element::from(list), El::from(display)].spacing(m.space_s).width(Length::Fill),
    ]
    .spacing(m.gap)
    .into()
}

fn radio<'a>(t: &'a Toy, i: usize, text: &'static str) -> El<'a> {
    cmp::radio_button(text, i, Some(t.radio), Msg::Radio).into()
}

fn screen_saver(t: &Toy) -> El<'_> {
    let m = Metrics::desktop();
    column![
        label("Screen Saver"),
        w::dropdown(&["(None)", "Flying Windows", "Mystify", "3D Pipes"], Some("Mystify"), |s| Msg::Drop(0, s.to_string())),
        cmp::check_button("Password protected", t.checks[0]).on_toggle(|b| Msg::Check(0, b)),
        row![label("Wait:"), cmp::entry("", t.text.as_str()).on_input(Msg::Text).width(80), label("minutes")]
            .spacing(m.space_s)
            .align_y(Alignment::Center),
        cmp::progress_bar(0.0..=1.0, t.progress),
    ]
    .spacing(m.space_m)
    .into()
}

fn appearance(t: &Toy) -> El<'_> {
    let m = Metrics::desktop();
    column![
        label("Item:"),
        cmp::scale(0.0..=100.0, t.sliders[0], |v| Msg::Slide(0, v)),
        cmp::check_button("Use large icons", t.checks[1]).on_toggle(|b| Msg::Check(1, b)),
        cmp::check_button("Show window contents while dragging", t.checks[2]).on_toggle(|b| Msg::Check(2, b)),
        row![Icon::Settings.view(16.0), label("Window metrics follow the theme")].spacing(m.space_s),
    ]
    .spacing(m.space_m)
    .into()
}
