//! Windows 7 All Programs Start menu scene, not an unrelated Explorer window.
use crate::{common::*, design::px};
use iad::components as cmp;
use iad::widgets as w;
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
fn program_menu(t: &Toy, index: usize) -> w::Menu<Msg> {
    native_menu(PROGRAMS[index]).section([
        w::MenuItem::action("Open", Msg::Launch(index)),
        w::MenuItem::check(
            "Pin to Start Menu",
            t.pinned_programs.contains(&index),
            Msg::PinProgram(index),
        )
        .keep_open(false),
    ])
}

fn power_menu() -> w::Menu<Msg> {
    native_menu("Power").section(["Switch user", "Log off", "Lock", "Restart", "Sleep"].map(
        |name| {
            w::MenuItem::action(
                name,
                Msg::Status(format!("{name} preview only — no host action")),
            )
        },
    ))
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
        El::from(w::context_menu(
            button
                .width(Length::Fill)
                .height(g("program-row-height"))
                .padding(0)
                .on_press(Msg::Launch(i)),
            program_menu(t, i),
        ))
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
        w::split_button(
            label("Shut down"),
            Msg::Status("Shutdown preview only — no host action".into()),
            power_menu()
        ),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_context_commands_preserve_row_size_and_pin_the_clicked_program() {
        let mut t = Toy::new();
        let index = PROGRAMS.len() - 1;
        for expected in [true, false] {
            let target = container(space())
                .width(g("programs-width"))
                .height(g("program-row-height"));
            let (size, command) = crate::common::menu_tests::activate(
                w::context_menu(target, program_menu(&t, index)).into(),
                2,
                true,
                1,
            );
            assert_eq!(size.height, g("program-row-height"));
            t.update(command);
            assert_eq!(t.pinned_programs.contains(&index), expected);
            assert!(!t.dialog);
        }
        let target = space()
            .width(g("programs-width"))
            .height(g("program-row-height"));
        let (_, command) = crate::common::menu_tests::activate(
            w::context_menu(target, program_menu(&t, index)).into(),
            2,
            true,
            0,
        );
        t.update(command);
        assert_eq!(t.pick, index);
        assert!(t.status.contains("Word"));
    }

    #[test]
    fn power_alternatives_are_real_menu_commands_without_host_actions() {
        let mut t = Toy::new();
        let (_, command) = crate::common::menu_tests::activate(
            menu_trigger("Power", power_menu(), g("program-row-height")),
            2,
            false,
            3,
        );
        t.update(command);
        assert_eq!(t.status, "Restart preview only — no host action");
        assert!(!t.dialog);
    }
}
