//! Windows 7 All Programs Start menu scene, not an unrelated Explorer window.
use crate::{common::*, design::px};
use iad::components as cmp;
use iad::{Icon, Metrics};
use iced::widget::{column, container, row, scrollable, space};
use iced::{Alignment, Length};
use iced_adaptive_design as iad;
pub const TITLE: &str = "Windows 7 — Start";
fn g(key: &str) -> f32 {
    px("toy.win7", key)
}
pub const PROGRAMS: &[&str] = &[
    "Google Chrome",
    "Google Drive",
    "Google Earth",
    "Gtk+",
    "Maintenance",
    "Malwarebytes' Anti-Malware",
    "Microsoft Lync",
    "Microsoft Office",
    "Microsoft Access 2010",
    "Microsoft Excel 2010",
    "Microsoft InfoPath Designer 2010",
    "Microsoft InfoPath Filler 2010",
    "Microsoft OneNote 2010",
    "Microsoft Outlook 2010",
    "Microsoft PowerPoint 2010",
    "Microsoft Project 2010",
    "Microsoft Publisher 2010",
    "Microsoft SharePoint Workspace 2010",
    "Microsoft Visio 2010",
    "Microsoft Word 2010",
];
pub fn filtered_programs(query: &str) -> impl Iterator<Item = (usize, &'static str)> + '_ {
    PROGRAMS
        .iter()
        .copied()
        .enumerate()
        .filter(move |(_, name)| matches_query(name, query))
}
pub fn view(t: &Toy) -> El<'_> {
    let m = Metrics::desktop();
    let programs = column(filtered_programs(&t.text).map(|(i, name)| {
        let line = row![Icon::Document.view(m.icon_size), label(name)].spacing(m.space_xs);
        let content = container(line).padding(iced::Padding {
            left: if i >= 8 {
                g("program-indent")
            } else {
                g("program-padding")
            },
            ..iced::Padding::ZERO
        });
        let button = if i == t.pick {
            cmp::variant_button(content, "opaque")
        } else {
            cmp::flat_button(content)
        };
        El::from(
            button
                .width(Length::Fill)
                .height(g("program-row-height"))
                .padding(0)
                .on_press(Msg::Launch(i)),
        )
    }));
    let left = container(column![
        scrollable(programs).height(Length::Fill),
        cmp::flat_button(label("◂   Back"))
            .width(Length::Fill)
            .height(g("program-row-height"))
            .on_press(Msg::Start(false)),
        container(
            cmp::entry("Search programs and files", &t.text)
                .on_input(Msg::Text)
                .width(Length::Fill)
        )
        .padding(m.space_m)
        .height(g("search-height")),
    ])
    .width(g("programs-width"))
    .height(Length::Fill)
    .clip(true)
    .style(iad::styles::container::view);
    let places = column(
        [
            "Documents",
            "Pictures",
            "Music",
            "Recent Items",
            "Computer",
            "Network",
            "Control Panel",
            "Devices and Printers",
            "Default Programs",
        ]
        .map(|name| {
            El::from(
                cmp::variant_button(label(name), "link")
                    .width(Length::Fill)
                    .height(g("place-row-height"))
                    .on_press(Msg::Status(format!("Open {name}"))),
            )
        }),
    );
    let right = column![
        container(
            column![
                Icon::Home.view(g("user-icon-size")),
                label("Thibault, David")
            ]
            .spacing(m.space_s)
            .align_x(Alignment::Center)
        )
        .height(g("user-height"))
        .center_x(Length::Fill),
        places,
        space().height(Length::Fill),
        cmp::raised_button(label("Shut down ▸")).on_press(Msg::Status("Shutdown preview only — no host action".into())),
    ]
    .width(Length::Fill)
    .padding(m.space_s);
    let menu = container(row![left, right])
        .padding(g("menu-padding"))
        .width(g("menu-width"))
        .height(g("menu-height"))
        .style(move |theme: &iad::Theme| container::Style {
            background: Some(theme.surface_background("start-menu", theme.semantic.sidebar.bg)),
            text_color: Some(theme.semantic.sidebar.fg),
            border: iced::Border {
                color: theme.semantic.headerbar.bg,
                width: m.border_width,
                radius: g("menu-radius").into(),
            },
            ..Default::default()
        });
    let taskbar = container(
        row![
            cmp::variant_button(Icon::Home.view(g("desktop-icon-size")), "suggested")
                .width(g("task-button-width"))
                .on_press(Msg::Start(!t.start_open)),
            cmp::flat_button(Icon::Search.view(g("desktop-icon-size")))
                .on_press(Msg::Status("Internet Explorer".into())),
            cmp::flat_button(Icon::Folder.view(g("desktop-icon-size")))
                .on_press(Msg::Status("Windows Explorer".into())),
            space().width(Length::Fill),
            caption(t.status.clone()),
        ]
        .align_y(Alignment::Center)
        .spacing(m.space_m),
    )
    .height(g("taskbar-height"))
    .width(Length::Fill)
    .style(move |theme: &iad::Theme| container::Style {
        background: Some(theme.surface_background("taskbar", theme.semantic.headerbar.bg)),
        ..Default::default()
    });
    let mut desktop = column![space().height(Length::Fill)];
    if t.start_open {
        desktop = desktop.push(menu);
    }
    let content = container(column![desktop.height(Length::Fill), taskbar])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |theme: &iad::Theme| container::Style {
            background: Some(theme.surface_background("desktop", theme.semantic.window.bg)),
            ..Default::default()
        })
        .into();
    content
}
