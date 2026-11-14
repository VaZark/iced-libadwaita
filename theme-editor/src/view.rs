//! The window: a header bar, the themes list, the tokens and the live preview.
//!
//! The shell follows the organiser apps: a `NavigationSplitView` with the list on the side and a
//! `toolbar_view` for the content, boxed groups of rows, toasts for what happened, shortcuts shown
//! in the menu, and an empty state where there is nothing to list.

use crate::{Editor, Msg, Variant};
use iad::components::{self as cmp, TooltipPosition};
use iad::shortcut::Shortcuts;
use iad::widgets as w;
use iad::{Element, Icon, Role};
use iced::widget::{column, container, responsive, row, scrollable, space, text};
use iced::{Alignment, Length};
use iced_adaptive_design as iad;

/// Below this window width the themes list is a drawer.
const SIDEBAR_COLLAPSES: f32 = 820.0;
/// From this content width the preview sits beside the tokens; narrower, it replaces them.
const PREVIEW_BESIDE: f32 = 940.0;
/// Below this width the sections are a dropdown above the tokens, not a list beside them.
const SECTIONS_BESIDE: f32 = 620.0;

impl Editor {
    pub(crate) fn view(&self) -> Element<'_, Msg> {
        let chrome = iad::windows::Chrome::new(self.win);
        let body: Element<'_, Msg> = responsive(move |size| self.shell(size.width)).into();
        let framed = chrome.frame(body, Msg::Window);
        let menu = self.main_menu();
        let framed = Shortcuts::new()
            .from_menu(&menu)
            .bind("Ctrl+Y", Msg::Redo)
            .scope(framed);
        match &self.toast {
            Some(n) => {
                let mut toast = w::Toast::new(n.text.clone())
                    .timeout(if n.error { 7000 } else { 3500 })
                    .on_dismiss(Msg::DismissToast);
                if n.error {
                    toast = toast.status(Role::Error);
                }
                w::toast_stack(framed, vec![toast])
            }
            None => w::toast_stack(framed, Vec::new()),
        }
    }

    // ---------------------------------------------------------------- menus

    /// The main menu: file actions, edit, registering, folders. Its accelerators work too.
    pub(crate) fn main_menu(&self) -> w::Menu<Msg> {
        let registered = match (self.registry(), &self.path) {
            (Some(dir), Some(file)) => Some((
                crate::files::is_registered(dir, file),
                !crate::files::links_to(dir, file).is_empty(),
                crate::files::default_target(dir) == Some(crate::files::canon(file)),
            )),
            _ => None,
        };
        let item = |label: &str, msg: Msg, accel: Option<&str>, enabled: bool| {
            let mut i = w::MenuItem::action(label, msg);
            if let Some(a) = accel {
                i = i.accel(a);
            }
            if !enabled {
                i = i.disabled();
            }
            i
        };
        w::Menu::new("Menu")
            .section([
                item("New", Msg::New, Some("Ctrl+N"), true),
                item("Open…", Msg::Open, Some("Ctrl+O"), true),
            ])
            .section([
                item("Save", Msg::Save, Some("Ctrl+S"), true),
                item("Save as…", Msg::SaveAs, Some("Ctrl+Shift+S"), true),
            ])
            .section([
                item("Undo", Msg::Undo, Some("Ctrl+Z"), !self.undo.is_empty()),
                item("Redo", Msg::Redo, Some("Ctrl+Shift+Z"), !self.redo.is_empty()),
            ])
            .section([
                item(
                    "Register for every app",
                    Msg::Register,
                    None,
                    registered.is_some_and(|(is, _, _)| !is),
                ),
                item("Unregister", Msg::Unregister, None, registered.is_some_and(|(_, l, _)| l)),
                item(
                    "Use as the default theme",
                    Msg::SetDefault,
                    None,
                    registered.is_some_and(|(_, _, d)| !d),
                ),
            ])
            .section([
                item("Add folder…", Msg::AddFolder, None, true),
                item("Refresh the list", Msg::Refresh, Some("F5"), true),
            ])
    }

    /// "Try it out": the gallery, then a toy window per design language.
    fn try_menu(&self, m: iad::Metrics) -> Element<'_, Msg> {
        let mut items: Vec<Element<'_, Msg>> = vec![
            w::menu_item("Gallery (every component)", false, Msg::TryOut("gallery".into())),
        ];
        items.extend(self.toys.iter().map(|t| {
            w::menu_item(format!("{} window", t.label), false, Msg::TryOut(t.name.clone()))
        }));
        let trigger = container(
            row![
                cmp::typography::label("Try it out"),
                Icon::GoDown.view(m.icon_size)
            ]
            .spacing(m.space_s)
            .align_y(Alignment::Center),
        )
        .padding(m.button_padding)
        .center_y(Length::Fixed(m.button_min_height))
        .style(|t: &iad::Theme| iced::widget::container::Style {
            background: Some(iad::color::alpha(t.semantic.window.fg, t.semantic.opacity.control).into()),
            border: iced::Border {
                radius: t.metrics.radius_m.into(),
                ..Default::default()
            },
            text_color: Some(t.semantic.window.fg),
            ..Default::default()
        });
        w::Popover::new(trigger, w::popover(items, 360.0))
            .popover_width(260.0)
            .into()
    }

    // ---------------------------------------------------------------- shell

    fn shell(&self, width: f32) -> Element<'_, Msg> {
        let m = iad::Metrics::desktop();
        let chrome = iad::windows::Chrome::new(self.win);
        let collapsed = width < SIDEBAR_COLLAPSES;
        let radius = chrome.radius();

        let side_head = w::header_bar()
            .chrome(&chrome.edges(true, false), Msg::Window)
            .title("Themes");
        let side = w::toolbar_view(self.files_pane(m))
            .top_bar(side_head)
            .bottom_bar(
                row![
                    cmp::flat_button(cmp::typography::label("New")).on_press(Msg::New),
                    cmp::flat_button(cmp::typography::label("Open…")).on_press(Msg::Open),
                    space().width(Length::Fill),
                    cmp::tooltip(
                        cmp::icon_button(Icon::FolderOpen).on_press(Msg::AddFolder),
                        "Add a folder of themes",
                        TooltipPosition::Top,
                    ),
                ]
                .spacing(m.space_xs)
                .padding(m.space_xs)
                .align_y(Alignment::Center),
            );

        let content_width = if collapsed { width } else { width * 0.74 };
        let content = w::toolbar_view(self.main_pane(content_width, m)).top_bar(self.header(collapsed, &chrome, m));

        w::NavigationSplitView::new(side, content)
            .adapt(width)
            .collapsed(collapsed)
            .show_sidebar(self.drawer)
            .on_close_drawer(Msg::Drawer(false))
            .window_radius(radius)
            .into()
    }

    /// The header: the file, undo, the actions and the menus.
    fn header(&self, collapsed: bool, chrome: &iad::windows::Chrome, m: iad::Metrics) -> Element<'_, Msg> {
        let name = self
            .path
            .as_ref()
            .and_then(|p| p.file_name())
            .map_or("Untitled".to_string(), |n| n.to_string_lossy().into_owned());
        let path = self
            .path
            .as_ref()
            .map_or("not saved yet".to_string(), |p| p.display().to_string());
        let mut bar = w::header_bar()
            .chrome(&chrome.edges(collapsed, true), Msg::Window)
            .title(format!("{}{name}", self.dot()))
            .subtitle(path);
        if collapsed {
            bar = bar.pack_start(cmp::tooltip(
                cmp::icon_button(Icon::Sidebar).on_press(Msg::Drawer(true)),
                "The themes",
                TooltipPosition::Bottom,
            ));
        }
        bar = bar
            .pack_end(
                w::primary_menu(self.main_menu())
                    .tooltip("Menu")
                    .window_radius(chrome.radius()),
            )
            .pack_end(self.try_menu(m))
            .pack_end(
                cmp::raised_button(cmp::typography::label("Save as")).on_press(Msg::SaveAs),
            )
            .pack_end(
                cmp::raised_button(cmp::typography::label("Save"))
                    .on_press_maybe((self.dirty() || self.path.is_none()).then_some(Msg::Save)),
            );
        bar.into()
    }

    // ------------------------------------------------------------ the list

    /// The theme files: a filter, then the registered ones, the folders and the recent files.
    fn files_pane(&self, m: iad::Metrics) -> Element<'_, Msg> {
        let filter = cmp::entry("Filter themes", self.files_filter.as_str())
            .left_section(iad::icon::EntryIcon::Search.view(16.0))
            .on_input(Msg::FilesFilter);
        let q = self.files_filter.trim().to_lowercase();
        let open = self.path.as_deref().map(crate::files::canon);
        let dot = self.dot();
        let mut list = column![].spacing(m.space_xxs).width(Length::Fill);
        let mut group = "";
        let mut shown = 0;
        for preset in &self.presets {
            if !q.is_empty()
                && !preset.name.to_lowercase().contains(&q)
                && !preset.path.display().to_string().to_lowercase().contains(&q)
            {
                continue;
            }
            shown += 1;
            if preset.group != group {
                group = preset.group;
                list = list.push(
                    text(group)
                        .size(m.space_m + 1.0)
                        .style(iad::styles::text::dim),
                );
            }
            let active = open.as_deref() == Some(crate::files::canon(&preset.path).as_path());
            let mut title = row![text(format!("{}{}", if active { dot } else { "" }, preset.name))]
                .spacing(m.space_xs);
            if preset.is_default {
                title = title.push(space().width(Length::Fill)).push(
                    text("default")
                        .size(m.space_m)
                        .style(iad::styles::text::dim),
                );
            }
            let mut label = column![title].width(Length::Fill);
            if let Some(target) = &preset.target {
                label = label.push(
                    text(format!("→ {}", target.display()))
                        .size(m.space_m - 1.0)
                        .style(iad::styles::text::dim),
                );
            }
            let button = if active {
                cmp::raised_button(label)
            } else {
                cmp::flat_button(label)
            };
            list = list.push(cmp::tooltip(
                button.on_press(Msg::Pick(preset.path.clone())).width(Length::Fill),
                preset.path.display().to_string(),
                TooltipPosition::Right,
            ));
        }
        let body: Element<'_, Msg> = if shown == 0 {
            w::status_page()
                .icon(Icon::Folder.view(48.0))
                .title(if self.presets.is_empty() { "No themes yet" } else { "Nothing matches" })
                .description(if self.presets.is_empty() {
                    "Open a theme file or start a new one. Save as registers it for every app."
                } else {
                    "No theme has that name or path."
                })
                .into()
        } else {
            scrollable(container(list).padding([0.0, m.space_s])).height(Length::Fill).into()
        };
        column![container(filter).padding([m.space_xs, m.space_s]), body]
            .spacing(m.space_xs)
            .height(Length::Fill)
            .into()
    }

    // ------------------------------------------------------------ the tokens

    /// The controls, then the tokens, then (wide enough) the preview.
    fn main_pane(&self, width: f32, m: iad::Metrics) -> Element<'_, Msg> {
        let beside = width >= PREVIEW_BESIDE;
        let controls = self.controls(width, beside, m);
        let tokens = self.tokens_area(if beside { width * 0.58 } else { width }, m);
        let body: Element<'_, Msg> = if beside {
            row![
                container(tokens).width(Length::FillPortion(3)),
                container(self.preview_pane(m)).width(Length::FillPortion(2)),
            ]
            .spacing(m.space_m)
            .height(Length::Fill)
            .into()
        } else if self.show_preview {
            self.preview_pane(m)
        } else {
            tokens
        };
        column![controls, body]
            .spacing(m.space_s)
            .padding(m.space_s)
            .height(Length::Fill)
            .into()
    }

    /// Variant, search, "modified only", copy from base, and the preview switch when it is hidden.
    fn controls(&self, _width: f32, beside: bool, m: iad::Metrics) -> Element<'_, Msg> {
        let variants = cmp::toggle_group(
            Variant::ALL
                .iter()
                .map(|v| cmp::ToggleItem::new(*v, v.label()))
                .collect(),
            Some(self.variant),
            Msg::Variant,
        );
        let search = cmp::entry("Search tokens", self.search.as_str())
            .left_section(iad::icon::EntryIcon::Search.view(16.0))
            .on_input(Msg::Search)
            .width(Length::Fill);
        let mut bar = row![
            variants,
            search,
            row![
                text("Modified only").size(m.space_m + 1.0),
                cmp::switch(self.modified_only).on_toggle(Msg::ModifiedOnly),
            ]
            .spacing(m.space_xs)
            .align_y(Alignment::Center),
        ]
        .spacing(m.space_m)
        .align_y(Alignment::Center);
        if self.variant != Variant::Base {
            bar = bar.push(cmp::tooltip(
                cmp::flat_button(cmp::typography::label("Copy from Base")).on_press(Msg::CopyFromBase),
                "Copy what the base sets into this variant, where it sets nothing",
                TooltipPosition::Bottom,
            ));
        }
        if !beside {
            bar = bar.push(
                row![
                    text("Preview").size(m.space_m + 1.0),
                    cmp::switch(self.show_preview).on_toggle(Msg::ShowPreview),
                ]
                .spacing(m.space_xs)
                .align_y(Alignment::Center),
            );
        }
        bar.into()
    }

    /// The sections (with how many tokens each sets) and the page of tokens.
    fn tokens_area(&self, width: f32, m: iad::Metrics) -> Element<'_, Msg> {
        let page = scrollable(
            container(self.page(m))
                .padding([m.space_s, m.space_m])
                .max_width(m.clamp_maximum + 160.0),
        )
        .width(Length::Fill)
        .height(Length::Fill);
        if width < SECTIONS_BESIDE {
            let dropdown = w::dropdown(&self.sections, Some(self.section.clone()), Msg::Section);
            return column![container(dropdown).padding([0.0, m.space_s]), page]
                .spacing(m.space_xs)
                .into();
        }
        let list = column(self.sections.iter().map(|s| {
            let n = self.set_count(s);
            let mut label = row![text(s.as_str())].spacing(m.space_xs).width(Length::Fill);
            if n > 0 {
                label = label
                    .push(space().width(Length::Fill))
                    .push(text(n.to_string()).size(m.space_m).style(iad::styles::text::dim));
            }
            let button = if *s == self.section && self.search.trim().is_empty() {
                cmp::raised_button(label)
            } else {
                cmp::flat_button(label)
            };
            button.on_press(Msg::Section(s.clone())).width(Length::Fill).into()
        }))
        .spacing(m.space_xxs)
        .width(Length::Fixed(172.0));
        row![scrollable(list).height(Length::Fill), page]
            .spacing(m.space_s)
            .height(Length::Fill)
            .into()
    }

    // ------------------------------------------------------------ the preview

    fn preview_pane(&self, m: iad::Metrics) -> Element<'_, Msg> {
        let schemes = cmp::toggle_group(
            vec![
                cmp::ToggleItem::new(false, "Light"),
                cmp::ToggleItem::new(true, "Dark"),
            ],
            Some(self.preview_dark),
            Msg::PreviewDark,
        );
        let frame = container(self.preview.view())
            .width(Length::Fill)
            .height(Length::Fill)
            .clip(true)
            .style(|t: &iad::Theme| iced::widget::container::Style {
                border: iced::Border {
                    color: t.semantic.window.fg.scale_alpha(0.2),
                    width: 1.0,
                    radius: 8.0.into(),
                },
                ..Default::default()
            });
        column![
            row![
                text("Live preview").size(m.space_m + 2.0),
                space().width(Length::Fill),
                schemes,
            ]
            .align_y(Alignment::Center),
            frame,
        ]
        .spacing(m.space_xs)
        .height(Length::Fill)
        .into()
    }
}
