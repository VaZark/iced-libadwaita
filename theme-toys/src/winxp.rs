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

pub fn item_name(index: usize) -> Option<&'static str> {
    SECTIONS
        .iter()
        .flat_map(|(_, names)| names.iter().copied())
        .nth(index)
}

fn item_menu(index: usize) -> w::Menu<Msg> {
    let name = item_name(index).unwrap_or("My Computer");
    native_menu(name).section([
        w::MenuItem::action("Open", Msg::OpenXpItem(index)),
        w::MenuItem::action("Properties", Msg::XpProperties(Some(index))),
    ])
}

fn menubar_menu(name: &str, t: &Toy) -> w::Menu<Msg> {
    use w::MenuItem as Item;
    match name {
        "File" if item_name(t.pick).is_some() => item_menu(t.pick),
        "File" => {
            native_menu(name).section([Item::action("Open", Msg::OpenXpItem(t.pick)).disabled()])
        }
        "Edit" => native_menu(name)
            .section([Item::action("Select All", Msg::SelectAllXp).accel("Ctrl+A")]),
        "View" => native_menu(name).section([
            Item::check("Folders", t.open[3], Msg::Open(3)),
            Item::action(
                "Refresh",
                Msg::Status("My Computer refreshed (preview)".into()),
            ),
        ]),
        "Favorites" => {
            native_menu(name).section([Item::action("Shared Documents", Msg::OpenXpItem(0))])
        }
        "Tools" => native_menu(name).section([Item::action(
            "Folder Options…",
            Msg::Status("Folder options are not implemented in this preview".into()),
        )
        .disabled()]),
        _ => {
            native_menu(name).section([Item::action("About My Computer", Msg::XpProperties(None))])
        }
    }
}

pub fn view(t: &Toy) -> El<'_> {
    let m = Metrics::desktop();
    let menus = row(["File", "Edit", "View", "Favorites", "Tools", "Help"]
        .map(|name| menu_trigger(name, menubar_menu(name, t), g("menu-height"))))
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
        El::from(
            cmp::flat_button(
                row![Icon::Folder.view(g("command-icon-size")), label("Folders")]
                    .spacing(m.space_s)
            )
            .on_press(Msg::Open(3))
        )
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
            let button = if id == t.pick || t.xp_all_selected {
                cmp::variant_button(content, "opaque")
            } else {
                cmp::flat_button(content)
            };
            El::from(w::context_menu(
                button
                    .width(g("item-width"))
                    .height(g("item-height"))
                    .on_press(Msg::Pick(id)),
                item_menu(id),
            ))
        }));
        El::from(column![title_bar, items])
    }));
    let sidebar: El<'_> = if t.open[3] {
        column(
            (0..SECTIONS.iter().map(|(_, names)| names.len()).sum()).filter_map(|i| {
                item_name(i).map(|name| {
                    El::from(cmp::flat_button(label(name)).on_press(Msg::OpenXpItem(i)))
                })
            }),
        )
        .spacing(m.space_s)
        .into()
    } else {
        tasks.into()
    };
    let body = row![
        container(sidebar)
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
    let dialog = w::AlertDialog::new(if t.xp_properties.is_some() {
        "Properties"
    } else {
        "System information"
    })
    .body(t.xp_properties.and_then(item_name).map_or_else(
        || "My Computer — safe preview. No programs are installed or removed.".to_string(),
        |name| {
            format!(
                "{name}\nBundled preview item. Host disk properties are not queried or changed."
            )
        },
    ))
    .response("OK", w::ResponseAppearance::Default, Msg::Dialog(false))
    .into();
    let mut content = column![menus, toolbar, address, body];
    if t.status != "Ready" {
        content = content.push(status_bar(t.status.clone(), "Preview only"));
    }
    window(t, TITLE, overlays(t, content.into(), dialog))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::menu_tests::activate;

    #[test]
    fn context_open_uses_the_target_not_the_prior_selection_and_keeps_item_geometry() {
        let mut t = Toy::new();
        t.pick = 0;
        let size = iced::Size::new(g("item-width"), g("item-height"));
        let target = space().width(size.width).height(size.height);
        let (actual, command) = activate(w::context_menu(target, item_menu(3)).into(), 1, true, 0);
        assert_eq!(actual, size);
        assert!(matches!(command, Msg::OpenXpItem(3)));
        t.update(command);
        assert_eq!(t.pick, 3);
        assert_eq!(t.text, "CD Drive (D:)");
        assert!(!t.dialog);
        assert_eq!(item_name(usize::MAX), None);
        let target = space().width(size.width).height(size.height);
        let (_, command) = activate(w::context_menu(target, item_menu(4)).into(), 1, true, 1);
        t.update(command);
        assert!(t.dialog);
        assert_eq!(t.xp_properties, Some(4));
        assert_eq!(t.pick, 3, "Properties must not act on the old selection");
        let _ = view(&t);
    }

    #[test]
    fn text_view_menu_toggles_folders_at_the_scene_menubar_height() {
        let mut t = Toy::new();
        let (size, command) = activate(
            menu_trigger("View", menubar_menu("View", &t), g("menu-height")),
            1,
            false,
            0,
        );
        assert_eq!(size.height, g("menu-height"));
        t.update(command);
        assert!(t.open[3]);
        assert!(!t.dialog);
        let _ = view(&t);
        t.update(Msg::SelectAllXp);
        assert!(t.xp_all_selected);
        t.update(Msg::Pick(2));
        assert!(!t.xp_all_selected);
    }
}
