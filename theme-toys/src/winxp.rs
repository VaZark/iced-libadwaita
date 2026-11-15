//! Luna My Computer, matching the supplied maximized 1024px reference (without desktop taskbar).
use crate::{common::*, design::px};
use iad::components as cmp;
use iad::widgets as w;
use iad::{Icon, Metrics};
use iced::widget::{column, container, row, space};
use iced::{Alignment, Length};
use iced_adaptive_design as iad;
pub const TITLE: &str = "My Computer";
fn g(key: &str) -> f32 {
    px("toy.winxp", key)
}
pub const SECTIONS: &[(&str, &[&str])] = &[
    (
        "Files Stored on This Computer",
        &["Shared Documents", "FarazHashmi’s Documents"],
    ),
    ("Hard Disk Drives", &["WindowsXp (C:)"]),
    ("Devices with Removable Storage", &["CD Drive (D:)"]),
    ("Network Drives", &["material on 'vboxsrv' (E:)"]),
];

pub fn view(t: &Toy) -> El<'_> {
    let m = Metrics::desktop();
    let menus = row(
        ["File", "Edit", "View", "Favorites", "Tools", "Help"].map(|name| {
            El::from(cmp::flat_button(label(name)).on_press(Msg::Status(format!("{name} menu"))))
        }),
    )
    .height(g("menu-height"))
    .align_y(Alignment::Center);
    let tool = |icon: Icon, name: &'static str| {
        El::from(
            cmp::flat_button(
                row![icon.view(g("command-icon-size")), label(name)].spacing(m.space_s),
            )
            .on_press(Msg::Status(name.into())),
        )
    };
    let toolbar = row![
        tool(Icon::GoPrevious, "Back"),
        tool(Icon::GoNext, ""),
        tool(Icon::GoUp, ""),
        tool(Icon::Search, "Search"),
        tool(Icon::Folder, "Folders")
    ]
    .spacing(m.space_s)
    .height(g("toolbar-height"))
    .align_y(Alignment::Center);
    let address = row![
        label("Address"),
        cmp::entry("My Computer", &t.text)
            .on_input(Msg::Text)
            .width(Length::Fill),
        cmp::variant_button(label("Go"), "suggested").on_press(Msg::Status("My Computer".into()))
    ]
    .spacing(m.space_s)
    .height(g("address-height"))
    .align_y(Alignment::Center);
    let group = |index: usize, title: &'static str, links: &[&'static str]| -> El<'_> {
        let head = cmp::flat_button(row![
            heading(title),
            space().width(Length::Fill),
            label(if t.open[index] { "⌃" } else { "⌄" })
        ])
        .width(Length::Fill)
        .on_press(Msg::Open(index));
        let body = container(
            column(links.iter().map(|name| {
                El::from(
                    cmp::variant_button(label(*name), "link")
                        .padding(0)
                        .height(g("task-row-height"))
                        .on_press(Msg::Status((*name).into())),
                )
            }))
            .spacing(m.space_xxs),
        )
        .padding(g("task-body-padding"));
        container(w::Collapsible::new(head, body, t.open[index]))
            .width(Length::Fill)
            .style(iad::styles::container::card)
            .into()
    };
    let tasks = column![
        group(
            0,
            "System Tasks",
            &[
                "View system information",
                "Add or remove programs",
                "Change a setting"
            ]
        ),
        group(
            1,
            "Other Places",
            &[
                "My Network Places",
                "My Documents",
                "Shared Documents",
                "Control Panel"
            ]
        ),
        group(2, "Details", &["My Computer", "System Folder"]),
    ]
    .spacing(g("task-gap"));
    let mut index = 0;
    let groups = column(SECTIONS.iter().map(|(title, names)| {
        let title_bar = container(heading(*title))
            .height(g("section-height"))
            .padding([m.space_xs, m.space_m])
            .width(Length::Fill);
        let items = row(names.iter().map(|name| {
            let id = index;
            index += 1;
            let content = row![Icon::Folder.view(g("item-icon-size")), label(*name)]
                .spacing(m.space_s)
                .align_y(Alignment::Center);
            let button = if id == t.pick {
                cmp::variant_button(content, "opaque")
            } else {
                cmp::flat_button(content)
            };
            El::from(
                button
                    .width(g("item-width"))
                    .height(g("item-height"))
                    .on_press(Msg::Pick(id)),
            )
        }));
        El::from(column![title_bar, items])
    }));
    let body = row![
        container(tasks)
            .width(g("sidebar-width"))
            .height(Length::Fill)
            .padding(g("task-padding"))
            .clip(true)
            .style(iad::styles::container::sidebar),
        container(iced::widget::scrollable(groups))
            .width(Length::Fill)
            .height(Length::Fill)
            .clip(true)
            .style(iad::styles::container::view),
    ]
    .height(Length::Fill);
    let dialog = w::AlertDialog::new("System information")
        .body("This is a safe preview. No programs are installed or removed.")
        .response("OK", w::ResponseAppearance::Default, Msg::Dialog(false))
        .into();
    let mut content = column![menus, toolbar, address, body];
    if t.status != "Ready" {
        content = content.push(status_bar(t.status.clone(), "Preview only"));
    }
    window(t, TITLE, overlays(t, content.into(), dialog))
}
