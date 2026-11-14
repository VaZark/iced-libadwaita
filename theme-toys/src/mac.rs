//! Classic Mac OS / Aqua: a System Preferences window. A toolbar of icon buttons, a source list,
//! a segmented control, sliders, switches, a pop-up menu and a sheet.

use crate::common::*;
use iad::components as cmp;
use iad::widgets as w;
use iad::{Icon, Metrics};
use iced::widget::{column, container, row, space};
use iced::{Alignment, Length};
use iced_adaptive_design as iad;

pub const TITLE: &str = "System Preferences";
const PANES: [&str; 3] = ["General", "Desktop", "Dock"];
const SCROLL: [&str; 3] = ["Automatically", "When scrolling", "Always"];

pub fn view(t: &Toy) -> El<'_> {
    let m = Metrics::desktop();
    let toolbar = row![
        cmp::icon_button(Icon::GoPrevious).on_press(Msg::Status("Back".into())),
        cmp::icon_button(Icon::GoNext).on_press(Msg::Status("Forward".into())),
        space().width(Length::Fill),
        heading("General"),
        space().width(Length::Fill),
        cmp::search_bar(t.text.as_str(), Msg::Text).placeholder("Search").width(180),
    ]
    .spacing(m.space_s)
    .align_y(Alignment::Center)
    .padding(m.space_s);
    let source = column(PANES.iter().enumerate().map(|(i, n)| {
        let b = cmp::flat_button(row![Icon::Settings.view(16.0), label(*n)].spacing(m.space_s));
        let b = if i == t.section { cmp::raised_button(row![Icon::Settings.view(16.0), label(*n)].spacing(m.space_s)) } else { b };
        El::from(b.width(Length::Fill).on_press(Msg::Section(i)))
    }))
    .spacing(m.space_xxs)
    .width(170);
    let seg = cmp::toggle_group(
        vec![cmp::ToggleItem::new(0usize, "Blue"), cmp::ToggleItem::new(1usize, "Graphite")],
        Some(t.radio.min(1)),
        Msg::Radio,
    );
    let pane_view = column![
        row![label("Appearance:"), Element::from(seg)].spacing(m.gap).align_y(Alignment::Center),
        row![label("Show scroll bars:"), w::dropdown(&SCROLL, SCROLL.iter().copied().find(|s| *s == t.drops[0]).or(Some(SCROLL[0])), |s| Msg::Drop(0, s.to_string()))].spacing(m.gap).align_y(Alignment::Center),
        row![label("Dock size:"), cmp::scale(0.0..=100.0, t.sliders[0], |v| Msg::Slide(0, v))].spacing(m.gap).align_y(Alignment::Center),
        row![label("Magnification:"), cmp::scale(0.0..=100.0, t.sliders[1], |v| Msg::Slide(1, v))].spacing(m.gap).align_y(Alignment::Center),
        row![label("Animate opening applications"), space().width(Length::Fill), cmp::switch(t.checks[0]).on_toggle(|b| Msg::Check(0, b))].align_y(Alignment::Center),
        row![label("Automatically hide the Dock"), space().width(Length::Fill), cmp::switch(t.checks[1]).on_toggle(|b| Msg::Check(1, b))].align_y(Alignment::Center),
        cmp::raised_button(cmp::typography::label("Restart…")).on_press(Msg::Dialog(true)),
    ]
    .spacing(m.gap);
    let body = row![container(source).padding(m.space_s), container(pane_view).padding(m.inset).width(Length::Fill)].height(Length::Fill);
    let content = column![El::from(toolbar), El::from(body), status_bar(t.status.clone(), "")];
    let dialog: El<'_> = w::AlertDialog::new("Restart now?").body("Changes take effect after you restart.")
        .response("Later", w::ResponseAppearance::Default, Msg::Dialog(false))
        .response("Restart", w::ResponseAppearance::Suggested, Msg::Dialog(false)).into();
    window(t, TITLE, overlays(t, content.into(), dialog))
}
