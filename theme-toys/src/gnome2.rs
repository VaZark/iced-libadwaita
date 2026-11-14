//! GNOME 2: a Nautilus-style file manager. A menu bar, a toolbar, a tree side pane, notebook tabs,
//! a file list, a status bar and a preferences dialog.

use crate::common::*;
use iad::components as cmp;
use iad::widgets as w;
use iad::{Icon, Metrics};
use iced::widget::{column, container, row, space};
use iced::{Alignment, Length};
use iced_adaptive_design as iad;

pub const TITLE: &str = "Home - File Browser";
const MENUS: [&str; 6] = ["File", "Edit", "View", "Go", "Bookmarks", "Help"];
const FILES: [&str; 6] = ["Desktop", "Documents", "Music", "Pictures", "Videos", "examples.desktop"];

pub fn view(t: &Toy) -> El<'_> {
    let m = Metrics::desktop();
    let menubar = row(MENUS.map(|n| El::from(cmp::flat_button(label(n)).on_press(Msg::Status(n.into()))))).spacing(m.space_xxs);
    let tool = |icon: Icon, text: &'static str| El::from(cmp::flat_button(row![icon.view(16.0), label(text)].spacing(m.space_xs)).on_press(Msg::Status(text.into())));
    let toolbar = row![tool(Icon::GoPrevious, "Back"), tool(Icon::GoNext, "Forward"), tool(Icon::GoUp, "Up"), tool(Icon::Home, "Home"), El::from(cmp::flat_button(row![Icon::Settings.view(16.0), label("Preferences")].spacing(m.space_xs)).on_press(Msg::Dialog(true)))]
        .spacing(m.space_xs);
    let list = column(FILES.iter().enumerate().map(|(i, n)| {
        let line = row![Icon::Folder.view(16.0), label(*n)].spacing(m.space_s).align_y(Alignment::Center);
        let b = if i == t.pick { cmp::raised_button(line) } else { cmp::flat_button(line) };
        El::from(b.width(Length::Fill).on_press(Msg::Pick(i)))
    }));
    let tabs = w::Tabs::new(Some(t.tab), Msg::Tab)
        .variant(w::TabsVariant::Outline)
        .tab(w::TabItem::new(0usize, "Home").panel(move || pane(column(FILES.iter().enumerate().map(|(i, n)| {
            let line = row![Icon::Folder.view(16.0), label(*n)].spacing(8).align_y(Alignment::Center);
            let b = if i == t.pick { cmp::raised_button(line) } else { cmp::flat_button(line) };
            El::from(b.width(Length::Fill).on_press(Msg::Pick(i)))
        })))))
        .tab(w::TabItem::new(1usize, "Notes").panel(|| label("A second notebook page.")));
    let _ = list;
    let body = row![container(nav_tree(t)).width(190).padding(m.space_s), container(tabs).width(Length::Fill).padding(m.space_s)].height(Length::Fill);
    let content = column![El::from(menubar), El::from(toolbar), El::from(body), status_bar(t.status.clone(), format!("{} items", FILES.len()))].spacing(m.space_xxs);
    let prefs = column![
        cmp::check_button("Show hidden files", t.checks[0]).on_toggle(|b| Msg::Check(0, b)),
        cmp::check_button("Single click to open", t.checks[1]).on_toggle(|b| Msg::Check(1, b)),
        cmp::check_button("Show thumbnails", t.checks[2]).on_toggle(|b| Msg::Check(2, b)),
    ]
    .spacing(m.space_s);
    let dialog: El<'_> = w::AlertDialog::new("File Management Preferences")
        .extra_child(prefs)
        .response("Close", w::ResponseAppearance::Default, Msg::Dialog(false))
        .into();
    let _ = space();
    window(t, TITLE, overlays(t, content.into(), dialog))
}
