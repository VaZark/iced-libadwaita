//! Windows XP: "My Computer". A task pane with collapsible groups, an address bar, drives with
//! usage bars, tabs, and a green Start-style button (the theme's `suggested` variant).

use crate::common::*;
use iad::components as cmp;
use iad::widgets as w;
use iad::{Icon, Metrics};
use iced::widget::{column, container, row, space};
use iced::{Alignment, Length};
use iced_adaptive_design as iad;

pub const TITLE: &str = "My Computer";
const DRIVES: [(&str, f32, &str); 3] = [
    ("Local Disk (C:)", 0.62, "15.2 GB free of 40 GB"),
    ("DVD Drive (D:)", 0.0, "No disc"),
    ("Backup (E:)", 0.88, "4.8 GB free of 40 GB"),
];

pub fn view(t: &Toy) -> El<'_> {
    let m = Metrics::desktop();
    let address = row![
        label("Address"),
        cmp::entry("", "My Computer").width(Length::Fill),
        cmp::variant_button(cmp::typography::label("Go"), "suggested").on_press(Msg::Status("Go".into())),
    ]
    .spacing(m.space_s)
    .align_y(Alignment::Center)
    .padding(m.space_s);
    let group = |i: usize, title: &'static str, items: [&'static str; 3]| -> El<'_> {
        let head = cmp::flat_button(heading(title)).width(Length::Fill).on_press(Msg::Open(i));
        let body = column(items.map(|s| El::from(cmp::variant_button(label(s), "link").on_press(Msg::Status(s.into()))))).spacing(m.space_xs);
        w::card(w::Collapsible::new(head, body, t.open[i])).into()
    };
    let task = column![
        group(0, "System Tasks", ["View system information", "Add or remove programs", "Change a setting"]),
        group(1, "Other Places", ["My Network Places", "My Documents", "Control Panel"]),
        group(2, "Details", ["Local Disk (C:)", "File System: NTFS", "Free space 15.2 GB"]),
    ]
    .spacing(m.gap)
    .width(220);
    let drives = row(DRIVES.iter().enumerate().map(|(i, (name, used, note))| {
        let b = column![
            Icon::Folder.view(32.0),
            label(*name),
            cmp::progress_bar(0.0..=1.0, *used),
            caption(*note),
        ]
        .spacing(m.space_xs)
        .width(170);
        let card = w::card(b);
        El::from(cmp::flat_button(card).on_press(Msg::Pick(i)))
    }))
    .spacing(m.gap);
    let tabs = w::Tabs::new(Some(t.tab), Msg::Tab)
        .variant(w::TabsVariant::Outline)
        .tab(w::TabItem::new(0usize, "Hard Disk Drives").panel(move || drives_panel(t)))
        .tab(w::TabItem::new(1usize, "Devices with Removable Storage").panel(|| label("No removable storage")));
    let _ = drives;
    let main = column![Element::from(tabs)].spacing(m.gap).padding(m.inset).width(Length::Fill);
    let start = row![
        cmp::variant_button(cmp::typography::label("start"), "suggested").on_press(Msg::Toast(Some("Start menu".into()))),
        space().width(Length::Fill),
        label("12:00 PM"),
    ]
    .padding(m.space_xs)
    .align_y(Alignment::Center);
    let body = row![container(task).padding(m.inset), main].height(Length::Fill);
    let content = column![El::from(address), El::from(body), El::from(container(start).style(iad::styles::container::headerbar).width(Length::Fill)), status_bar(t.status.clone(), "3 objects")];
    let dialog: El<'_> = w::AlertDialog::new("Safely remove hardware?").body("Windows can now safely remove the device.")
        .response("OK", w::ResponseAppearance::Suggested, Msg::Dialog(false)).into();
    window(t, TITLE, overlays(t, content.into(), dialog))
}

fn drives_panel(t: &Toy) -> El<'_> {
    let m = Metrics::desktop();
    row(DRIVES.iter().enumerate().map(|(i, (name, used, note))| {
        let selected = i == t.pick;
        let b = column![Icon::Folder.view(32.0), label(*name), cmp::progress_bar(0.0..=1.0, *used), caption(*note)]
            .spacing(m.space_xs)
            .width(170);
        let btn = if selected { cmp::raised_button(b) } else { cmp::flat_button(b) };
        El::from(btn.on_press(Msg::Pick(i)))
    }))
    .spacing(m.gap)
    .into()
}
