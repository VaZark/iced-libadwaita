//! MATE 1.26 desktop, Pluma and Desktop User Guide. File name retained for source compatibility.
use crate::{common::*, design::px};
use iad::components as cmp;
use iad::widgets as w;
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

fn edit_menu() -> w::Menu<Msg> {
    native_menu("Edit")
        .section([
            w::MenuItem::action("Undo", Msg::UndoDocument).accel("Ctrl+Z"),
            w::MenuItem::action("Redo", Msg::RedoDocument).accel("Ctrl+Shift+Z"),
        ])
        .section([
            w::MenuItem::action("Select All", Msg::Edit(cmp::TextAction::SelectAll))
                .accel("Ctrl+A"),
        ])
}

fn editor_menu(name: &str, t: &Toy) -> w::Menu<Msg> {
    use w::MenuItem as Item;
    match name {
        "File" => native_menu(name).section([
            Item::action("Open…", Msg::ChooseDocument).accel("Ctrl+O"),
            Item::action("Save", Msg::SaveDocument).accel("Ctrl+S"),
        ]),
        "Edit" => edit_menu(),
        "View" => native_menu(name).section([Item::check(
            "Toolbar",
            t.editor_toolbar,
            Msg::EditorToolbar(!t.editor_toolbar),
        )]),
        "Search" => {
            native_menu(name).section([Item::action("Find…", Msg::Find(true)).accel("Ctrl+F")])
        }
        "Tools" => native_menu(name).section([Item::action(
            "Check Spelling",
            Msg::Status("Spelling unavailable".into()),
        )
        .disabled()]),
        "Documents" => native_menu(name).section([Item::action("index.html", Msg::ShowChild(0))]),
        _ => native_menu(name).section([Item::action("Contents", Msg::ShowChild(1)).accel("F1")]),
    }
}

fn places_menu() -> w::Menu<Msg> {
    native_menu("Places").section(
        [
            ("Home", "~"),
            ("Desktop", "~/Desktop"),
            ("Computer", "/"),
            ("Network", "network:///"),
        ]
        .map(|(name, path)| {
            w::MenuItem::action(
                name,
                Msg::Places(iad::primitives::tree::Event::Click(path.into())),
            )
        }),
    )
}

pub fn find_matches(document: &str, query: &str) -> usize {
    if query.is_empty() {
        0
    } else {
        document
            .to_lowercase()
            .matches(&query.to_lowercase())
            .count()
    }
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
    .map(|name| menu_trigger(name, editor_menu(name, t), g("menu-height"))))
    .height(g("menu-height"))
    .align_y(Alignment::Center);
    let editor_tools = row![
        cmp::flat_button(row![Icon::Folder.view(m.icon_size), label("Open")].spacing(m.space_s))
            .on_press(Msg::ChooseDocument),
        cmp::flat_button(row![Icon::Document.view(m.icon_size), label("Save")].spacing(m.space_s))
            .on_press(Msg::SaveDocument),
        cmp::flat_button(label("Undo")).on_press(Msg::UndoDocument),
        cmp::flat_button(label("Redo")).on_press(Msg::RedoDocument),
        cmp::flat_button(Icon::Search.view(m.icon_size)).on_press(Msg::Find(true)),
    ]
    .spacing(m.space_s)
    .height(g("toolbar-height"))
    .align_y(Alignment::Center);
    let editor_tools: El<'_> = if t.editor_toolbar {
        editor_tools.into()
    } else {
        space().into()
    };
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
    let editor: El<'_> = w::context_menu(editor, edit_menu()).into();
    let mut editor_body = column![editor].height(Length::Fill);
    if t.find_open {
        editor_body = editor_body.push(
            row![
                cmp::entry("Find in document", &t.find_query)
                    .on_input(Msg::FindQuery)
                    .width(Length::Fill),
                caption(format!(
                    "{} matches (case-insensitive)",
                    find_matches(&t.document.text(), &t.find_query)
                )),
                cmp::flat_button(label("Close")).on_press(Msg::Find(false)),
            ]
            .spacing(m.space_s)
            .align_y(Alignment::Center),
        );
    }
    let pluma = child_window(
        0,
        "index.html — Pluma",
        column![
            menus,
            editor_tools,
            container(label("index.html  ×"))
                .height(g("tab-height"))
                .padding([0.0, m.space_xs]),
            editor_body,
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
    let desktop_menu = native_menu("Desktop").section([
        w::MenuItem::action("Show Desktop", Msg::ShowDesktop),
        w::MenuItem::action("Desktop User Guide", Msg::ShowChild(1)),
    ]);
    let mut desktop = stack![w::context_menu(
        container(icons)
            .padding([m.space_s, 0.0])
            .width(Length::Fill)
            .height(Length::Fill),
        desktop_menu,
    )];
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
            menu_trigger(
                "Applications",
                native_menu("Applications").section([
                    w::MenuItem::action("Text Editor", Msg::ShowChild(0)),
                    w::MenuItem::action("Desktop User Guide", Msg::ShowChild(1)),
                ]),
                g("panel-height")
            ),
            menu_trigger("Places", places_menu(), g("panel-height")),
            menu_trigger(
                "System",
                native_menu("System").section([
                    w::MenuItem::action("Help", Msg::ShowChild(1)),
                    w::MenuItem::action(
                        "Preferences",
                        Msg::Status("System preferences unavailable in this preview".into())
                    )
                    .disabled(),
                ]),
                g("panel-height")
            ),
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
    let dialog = w::AlertDialog::new("Open document")
        .body("Choose a bundled document. No host files are read or written.")
        .extra_child(
            column![
                cmp::flat_button(label("index.html (bundled sample)"))
                    .on_press(Msg::LoadDocument(DOCUMENT.into())),
                cmp::flat_button(label("Last saved preview"))
                    .on_press(Msg::LoadDocument(t.saved_document.clone())),
            ]
            .spacing(m.space_s),
        )
        .response("Cancel", w::ResponseAppearance::Default, Msg::Dialog(false))
        .into();
    overlays(t, content, dialog)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::menu_tests::activate;

    #[test]
    fn editor_context_undo_and_redo_use_document_history_without_a_modal() {
        let mut t = Toy::new();
        let original = t.document.text();
        t.update(Msg::Edit(cmp::TextAction::Edit(
            iced::widget::text_editor::Edit::Insert('X'),
        )));
        let changed = t.document.text();
        for row in [0, 1] {
            let target = space().width(g("editor-width")).height(g("editor-height"));
            let (size, command) =
                activate(w::context_menu(target, edit_menu()).into(), 3, true, row);
            assert_eq!(size, iced::Size::new(g("editor-width"), g("editor-height")));
            t.update(command);
            assert_eq!(
                t.document.text(),
                if row == 0 { &original } else { &changed }.as_str()
            );
            assert!(!t.dialog);
        }
    }

    #[test]
    fn places_and_search_text_menus_keep_compact_geometry_and_independent_queries() {
        let mut t = Toy::new();
        let (size, command) = activate(
            menu_trigger("Places", places_menu(), g("panel-height")),
            3,
            false,
            1,
        );
        assert_eq!(size.height, g("panel-height"));
        t.update(command);
        assert!(t.status.contains("~/Desktop"));
        assert!(!t.dialog);
        t.text = "help search".into();
        let (size, command) = activate(
            menu_trigger("Search", editor_menu("Search", &t), g("menu-height")),
            3,
            false,
            0,
        );
        assert_eq!(size.height, g("menu-height"));
        t.update(command);
        t.update(Msg::FindQuery("GALLERY".into()));
        assert!(t.find_open);
        assert_eq!(t.text, "help search");
        assert!(find_matches(DOCUMENT, &t.find_query) > 0);
        assert_eq!(find_matches(DOCUMENT, ""), 0);
        assert_eq!(find_matches("a.b aXb", "a.b"), 1);
        let (_, command) = activate(
            menu_trigger("View", editor_menu("View", &t), g("menu-height")),
            3,
            false,
            0,
        );
        t.update(command);
        assert!(!t.editor_toolbar);
        let _ = view(&t);
    }

    #[test]
    fn file_open_is_a_document_picker_not_a_menu_substitute_and_load_is_undoable() {
        let mut t = Toy::new();
        let (_, command) = activate(
            menu_trigger("File", editor_menu("File", &t), g("menu-height")),
            3,
            false,
            0,
        );
        t.update(command);
        assert!(t.dialog);
        let _ = view(&t);
        let original = t.document.text();
        t.update(Msg::LoadDocument("preview document".into()));
        assert!(!t.dialog);
        assert_eq!(t.document.text(), "preview document");
        t.update(Msg::UndoDocument);
        assert_eq!(t.document.text(), original);
        t.update(Msg::ShowDesktop);
        assert!(!t.open[0] && !t.open[1]);
        t.update(Msg::ShowChild(1));
        t.update(Msg::ShowChild(1));
        assert!(t.open[1]);
        assert_eq!(t.active_child, 1);
    }
}
