//! MATE 1.26 desktop, Pluma and Desktop User Guide. File name retained for source compatibility.
use crate::{common::*, design::px};
use iad::components as cmp;
use iad::{Icon, Metrics};
use iced::widget::{column, container, row, scrollable, space, stack};
use iced::{Alignment, Length};
use iced_adaptive_design as iad;
pub const TITLE: &str = "MATE 1.26 Desktop";
pub const DOCUMENT: &str = "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n  <meta charset=\"utf-8\">\n  <title>Gallery</title>\n</head>\n<body>\n  <header>\n    <h1>Gallery</h1>\n    <nav>\n      <a href=\"index.html\">Home</a>\n      <a href=\"gallery.html\">Gallery</a>\n    </nav>\n  </header>\n  <main>\n    <h2>Welcome to MATE</h2>\n    <p>A traditional desktop, ready to use.</p>\n  </main>\n</body>\n</html>";
fn g(key: &str) -> f32 {
    px("toy.mate", key)
}

pub fn places() -> Vec<iad::primitives::tree::TreeNode> {
    use iad::primitives::tree::TreeNode as Node;
    vec![
        Node::branch(
            "~",
            "Home",
            vec![
                Node::new("~/Documents", "Documents"),
                Node::new("~/Music", "Music"),
                Node::new("~/Pictures", "Pictures"),
            ],
        ),
        Node::new("~/Desktop", "Desktop"),
        Node::branch(
            "/",
            "File System",
            vec![
                Node::new("/home", "home"),
                Node::new("/etc", "etc"),
                Node::new("/usr", "usr"),
                Node::new("/var", "var"),
            ],
        ),
        Node::new("network:///", "Network"),
    ]
}

fn child_window<'a>(
    index: usize,
    title: &'static str,
    body: El<'a>,
    width: f32,
    height: f32,
    maximized: bool,
) -> El<'a> {
    let m = Metrics::desktop();
    let titlebar = container(
        row![
            space().width(m.space_s),
            caption(title),
            space().width(Length::Fill),
            cmp::flat_button(label("−")).on_press(Msg::Open(index)),
            cmp::flat_button(label("□")).on_press(Msg::Check(4 + index, !maximized)),
            cmp::flat_button(label("×")).on_press(Msg::Open(index)),
        ]
        .align_y(Alignment::Center),
    )
    .height(g("title-height"))
    .width(Length::Fill)
    .style(iad::styles::container::headerbar);
    container(column![titlebar, body])
        .width(if maximized { g("width") } else { width })
        .height(if maximized {
            g("height") - 2.0 * g("panel-height")
        } else {
            height
        })
        .clip(true)
        .style(|theme: &iad::Theme| container::Style {
            background: Some(theme.semantic.window.bg.into()),
            border: iced::Border {
                color: theme.semantic.headerbar_shade,
                width: g("window-border"),
                radius: g("window-radius").into(),
            },
            ..Default::default()
        })
        .into()
}

pub fn view(t: &Toy) -> El<'_> {
    let m = Metrics::desktop();
    let menus = row([
        "File",
        "Edit",
        "View",
        "Search",
        "Tools",
        "Documents",
        "Help",
    ]
    .map(|name| {
        El::from(
            cmp::flat_button(label(name))
                .padding([0.0, m.space_xs])
                .on_press(Msg::Status(format!("{name} menu"))),
        )
    }))
    .height(g("menu-height"))
    .align_y(Alignment::Center);
    let editor_tools = row![
        cmp::flat_button(row![Icon::Folder.view(m.icon_size), label("Open")].spacing(m.space_s))
            .on_press(Msg::Status("Requires core text-trigger/context menu support".into())),
        cmp::flat_button(row![Icon::Document.view(m.icon_size), label("Save")].spacing(m.space_s))
            .on_press(Msg::SaveDocument),
        cmp::flat_button(label("Undo")).on_press(Msg::UndoDocument),
        cmp::flat_button(label("Redo")).on_press(Msg::RedoDocument),
        cmp::flat_button(Icon::Search.view(m.icon_size)).on_press(Msg::Status("Requires core text-trigger/context menu support".into())),
    ]
    .spacing(m.space_s)
    .height(g("toolbar-height"))
    .align_y(Alignment::Center);
    let editor: El<'_> = cmp::text_view(&t.document)
        .on_action(Msg::Edit)
        .editor_props(
            cmp::TextEditorProps::default()
                .size(g("editor-font-size"))
                .style(|theme, status| {
                    let mut style = iad::styles::text_editor::default(theme, status);
                    style.background = theme.semantic.view.bg.into();
                    style
                }),
        )
        .height(Length::Fill)
        .into();
    let pluma = child_window(
        0,
        "index.html — Pluma",
        column![
            menus,
            editor_tools,
            container(label("index.html  ×"))
                .height(g("tab-height"))
                .padding([0.0, m.space_xs]),
            editor,
            status_bar("HTML   ▾    Tab Width: 4", t.status.clone()),
        ]
        .height(Length::Fill)
        .into(),
        g("editor-width"),
        g("editor-height"),
        t.checks[4],
    );
    let help_tools = row![
        cmp::flat_button(Icon::GoPrevious.view(m.icon_size)).on_press(Msg::Section(0)),
        cmp::flat_button(Icon::GoNext.view(m.icon_size)).on_press(Msg::Section(1)),
        space().width(Length::Fill),
        cmp::entry("Search help", &t.text).on_input(Msg::Text),
    ]
    .height(g("toolbar-height"))
    .align_y(Alignment::Center);
    let mut article = column![
        iced::widget::text(if t.section == 0 { "The Desktop" } else { "Working with Windows" })
            .size(g("heading-size")),
        label("The desktop lies behind all other components on your visible desktop. When no windows are visible, the desktop is that part of the screen between the top and bottom panels."),
        label("The desktop has the following special icons:"),
        label("• The Computer icon gives you access to CDs, removable media and the entire filesystem (also known as the root filesystem)."),
        label("• Home opens your personal folder, where your documents and other files are stored."),
        label("• Trash contains files that you have deleted. You can empty it or restore files."),
        label("• When you insert a CD, USB flash drive or other removable media, an icon appears on your desktop."),
        label("As you work, the desktop may become obscured by windows. To view the desktop, minimize all windows or use the Show Desktop button on the bottom panel."),
        label("You can also press Ctrl + Alt + D to show the desktop. Press the keys again to restore your windows."),
        label("You can create folders and files on the desktop. Right-click an empty area to see the desktop menu, or drag a file from your Home folder."),
        label("To arrange desktop items, select them and choose the arrangement you prefer. Your personal settings are stored in your Home folder."),
    ].spacing(m.space_m);
    if !t.text.trim().is_empty() {
        article = column![heading("Search results")].spacing(m.space_m);
        for (section, title) in ["The Desktop", "Working with Windows"].iter().enumerate() {
            if matches_query(title, &t.text) {
                article =
                    article.push(cmp::flat_button(label(*title)).on_press(Msg::Section(section)));
            }
        }
    }
    let help = child_window(
        1,
        "Desktop User Guide",
        column![
            help_tools,
            container(scrollable(article))
                .padding(g("document-padding"))
                .height(Length::Fill)
                .style(iad::styles::container::view),
        ]
        .into(),
        g("help-width"),
        g("help-height"),
        t.checks[5],
    );
    let icons = column(
        ["Computer", "Home", "Trash", "Pictures", "Downloads"]
            .iter()
            .enumerate()
            .map(|(i, name)| {
                El::from(
                    cmp::variant_button(
                        column![Icon::Folder.view(g("desktop-icon-size")), label(*name)]
                            .spacing(m.space_xs)
                            .align_x(Alignment::Center),
                        "link",
                    )
                    .width(g("desktop-item-width"))
                    .height(g("desktop-item-height"))
                    .on_press(Msg::Places(
                        iad::primitives::tree::Event::Click(
                            ["/", "~", "trash:///", "~/Pictures", "~/Downloads"][i].into(),
                        ),
                    )),
                )
            }),
    );
    let mut desktop = stack![container(icons).padding([m.space_s, 0.0])];
    let mut previews: [Option<El<'_>>; 2] = [
        t.open[0].then(|| {
            container(pluma)
                .padding(iced::Padding {
                    left: if t.checks[4] { 0.0 } else { g("editor-x") },
                    top: if t.checks[4] { 0.0 } else { g("editor-y") },
                    ..iced::Padding::ZERO
                })
                .into()
        }),
        t.open[1].then(|| {
            container(help)
                .padding(iced::Padding {
                    left: if t.checks[5] { 0.0 } else { g("help-x") },
                    top: if t.checks[5] { 0.0 } else { g("help-y") },
                    ..iced::Padding::ZERO
                })
                .into()
        }),
    ];
    for index in [1 - t.active_child, t.active_child] {
        if let Some(preview) = previews[index].take() {
            desktop = desktop.push(preview);
        }
    }
    let desktop = desktop.height(Length::Fill).width(Length::Fill);
    let top = container(
        row![
            cmp::flat_button(label("Applications")).on_press(Msg::Status("Applications".into())),
            cmp::flat_button(label("Places")).on_press(Msg::Status("Requires core text-trigger/context menu support".into())),
            cmp::flat_button(label("System")).on_press(Msg::Status("System".into())),
            space().width(Length::Fill),
            label("Tue Aug 24, 21:21"),
        ]
        .align_y(Alignment::Center),
    )
    .height(g("panel-height"))
    .width(Length::Fill)
    .style(iad::styles::container::card);
    let bottom = container(
        row![
            label("[MATE Terminal]"),
            cmp::flat_button(label("index.html — Pluma"))
                .padding(0)
                .on_press(Msg::Open(0)),
            cmp::flat_button(label("Desktop User Guide"))
                .padding(0)
                .on_press(Msg::Open(1)),
            space().width(Length::Fill)
        ]
        .spacing(m.space_l),
    )
    .height(g("panel-height"))
    .width(Length::Fill)
    .style(iad::styles::container::card);
    let content = container(column![top, desktop, bottom])
        .height(Length::Fill)
        .width(Length::Fill)
        .style(|theme: &iad::Theme| container::Style {
            background: Some(theme.surface_background("desktop", theme.semantic.window.bg)),
            ..Default::default()
        })
        .into();
    content
}
