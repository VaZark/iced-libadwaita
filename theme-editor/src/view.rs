//! Section-first editor with a theme picker and external gallery/toy launchers.
//!
//! The shell follows the organiser apps: a `NavigationSplitView` with sections on the side and a
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

/// Below this window width the section list is a drawer.
const SIDEBAR_COLLAPSES: f32 = 820.0;
/// Editor-local overflow policy; shared dropdown defaults remain unchanged.
pub(crate) fn theme_item_props(max_width: f32) -> w::DropdownItemProps {
    w::DropdownItemProps::default()
        .label_props(
            cmp::TextProps::default()
                .overflow(cmp::TextOverflow::Wrap)
                .width(Length::Fill),
        )
        .container_props(
            cmp::ContainerProps::default()
                .width(Length::Fill)
                .max_width(max_width),
        )
        .button_props(cmp::ButtonProps::default().width(Length::Fill))
}

impl Editor {
    pub(crate) fn view(&self) -> Element<'_, Msg> {
        let chrome = iad::windows::Chrome::new(self.win);
        let body: Element<'_, Msg> = responsive(move |size| self.shell(size.width)).into();
        let body = if self.theme_picker {
            w::dialog(
                body,
                self.theme_picker_view(),
                Some(Msg::ThemePicker(false)),
            )
        } else {
            body
        };
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
                item("Choose theme…", Msg::ThemePicker(true), None, true),
            ])
            .section([
                item("Save", Msg::Save, Some("Ctrl+S"), true),
                item("Save as…", Msg::SaveAs, Some("Ctrl+Shift+S"), true),
            ])
            .section([
                item("Undo", Msg::Undo, Some("Ctrl+Z"), !self.undo.is_empty()),
                item(
                    "Redo",
                    Msg::Redo,
                    Some("Ctrl+Shift+Z"),
                    !self.redo.is_empty(),
                ),
            ])
            .section([
                item(
                    "Register for every app",
                    Msg::Register,
                    None,
                    registered.is_some_and(|(is, _, _)| !is),
                ),
                item(
                    "Unregister",
                    Msg::Unregister,
                    None,
                    registered.is_some_and(|(_, l, _)| l),
                ),
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
        let actions = std::iter::once(
            w::SplitAction::new("gallery".to_string(), "Open Gallery").icon(Icon::Play),
        )
        .chain(self.toys.iter().map(|t| {
            w::SplitAction::new(t.name.clone(), format!("Open {} window", t.label))
                .icon(Icon::Window)
        }));
        w::SplitButtonAction::new(
            actions,
            self.launch_target.clone(),
            Msg::LaunchTarget,
            Msg::TryOut,
        )
        .metrics(m)
        .label_props(
            cmp::TextProps::default()
                .overflow(cmp::TextOverflow::Wrap)
                .width(Length::Fixed(m.sidebar_width)),
        )
        .option_props(theme_item_props(m.sidebar_width))
        .into()
    }

    fn theme_picker_view(&self) -> Element<'_, Msg> {
        responsive(move |size| self.theme_picker_at(size)).into()
    }

    pub(crate) fn theme_picker_at(&self, size: iced::Size) -> Element<'_, Msg> {
        let m = iad::Metrics::desktop();
        let width = m.clamp_maximum.min((size.width - 2.0 * m.space_m).max(0.0));
        let height = m
            .clamp_maximum
            .min((size.height - 2.0 * m.space_m).max(0.0));
        let picker = container(
            column![
                row![
                    text("Choose a theme"),
                    space().width(Length::Fill),
                    cmp::icon_button(Icon::WindowClose).on_press(Msg::ThemePicker(false))
                ]
                .align_y(Alignment::Center),
                self.files_pane(m, (width - m.page_padding.x()).max(0.0)),
                row![
                    cmp::flat_button(cmp::typography::label("New")).on_press(Msg::New),
                    cmp::flat_button(cmp::typography::label("Open…")).on_press(Msg::Open),
                    cmp::flat_button(cmp::typography::label("Add folder…"))
                        .on_press(Msg::AddFolder),
                    cmp::icon_button(Icon::Reset).on_press(Msg::Refresh),
                ]
                .spacing(m.space_s)
                .wrap(),
            ]
            .spacing(m.space_m),
        )
        .padding(m.page_padding)
        .width(Length::Fixed(width))
        .height(Length::Fixed(height))
        .clip(true)
        .style(iad::styles::container::card);
        container(picker)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
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
            .title("Sections");
        let side = w::toolbar_view(self.sections_pane(m)).top_bar(side_head);

        let content_width = if collapsed { width } else { width * 0.74 };
        let content = w::toolbar_view(self.main_pane(content_width, collapsed, m))
            .top_bar(self.header(collapsed, &chrome, m));

        w::NavigationSplitView::new(side, content)
            .adapt(width)
            .collapsed(collapsed)
            .show_sidebar(self.drawer)
            .on_close_drawer(Msg::Drawer(false))
            .window_radius(radius)
            .into()
    }

    /// Closed theme picker uses the same local overflow policy as its options.
    pub(crate) fn theme_selector(&self, width: f32) -> Element<'_, Msg> {
        let name = self.path.as_ref().map_or_else(
            || "Untitled".to_string(),
            |path| {
                self.presets
                    .iter()
                    .find(|preset| crate::files::canon(&preset.path) == crate::files::canon(path))
                    .map_or_else(
                        || {
                            path.file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .into_owned()
                        },
                        |preset| preset.name.clone(),
                    )
            },
        );
        let label = theme_item_props(width)
            .label
            .apply(cmp::typography::label(format!("{}{}", self.dot(), name)));
        container(cmp::flat_button(label).on_press(Msg::ThemePicker(true)))
            .width(Length::Fixed(width))
            .max_width(width)
            .into()
    }

    /// The header: the file, undo, the actions and the menus.
    fn header(
        &self,
        collapsed: bool,
        chrome: &iad::windows::Chrome,
        m: iad::Metrics,
    ) -> Element<'_, Msg> {
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
                "Show sections",
                TooltipPosition::Bottom,
            ));
        }
        if !collapsed {
            bar = bar.pack_start(self.try_menu(m));
        }
        bar = bar
            .pack_end(
                w::primary_menu(self.main_menu())
                    .tooltip("Menu")
                    .window_radius(chrome.radius()),
            )
            .pack_end(self.theme_selector(m.sidebar_width))
            .pack_end(
                cmp::raised_button(
                    row![
                        Icon::SaveAs.view(m.icon_size),
                        cmp::typography::label("Save as")
                    ]
                    .spacing(m.space_s)
                    .align_y(Alignment::Center),
                )
                .on_press(Msg::SaveAs),
            )
            .pack_end(
                cmp::raised_button(
                    row![Icon::Save.view(m.icon_size), cmp::typography::label("Save")]
                        .spacing(m.space_s)
                        .align_y(Alignment::Center),
                )
                .on_press_maybe((self.dirty() || self.path.is_none()).then_some(Msg::Save)),
            );
        bar.into()
    }

    // ------------------------------------------------------------ the list

    /// The theme files: a filter, then the registered ones, the folders and the recent files.
    pub(crate) fn theme_option(
        &self,
        preset: &crate::files::Preset,
        width: f32,
    ) -> Element<'_, Msg> {
        let active = self
            .path
            .as_deref()
            .map(crate::files::canon)
            .is_some_and(|path| path == crate::files::canon(&preset.path));
        let path = match &preset.target {
            Some(target) => format!("{} → {}", preset.path.display(), target.display()),
            None => preset.path.display().to_string(),
        };
        let label = format!(
            "{}{}{}\n{path}",
            if active { self.dot() } else { "" },
            preset.name,
            if preset.is_default { " · default" } else { "" }
        );
        w::menu_item_with_props(
            label,
            active,
            Msg::Pick(preset.path.clone()),
            theme_item_props(width),
        )
    }

    fn files_pane(&self, m: iad::Metrics, width: f32) -> Element<'_, Msg> {
        let filter = cmp::entry("Filter themes", self.files_filter.as_str())
            .left_section(iad::icon::EntryIcon::Search.view(16.0))
            .on_input(Msg::FilesFilter)
            .width(Length::Fill);
        let q = self.files_filter.trim().to_lowercase();

        let mut list = column![].spacing(m.space_xxs).width(Length::Fill);
        let mut group = "";
        let mut shown = 0;
        for preset in &self.presets {
            if !q.is_empty()
                && !preset.name.to_lowercase().contains(&q)
                && !preset
                    .path
                    .display()
                    .to_string()
                    .to_lowercase()
                    .contains(&q)
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
            list = list.push(self.theme_option(preset, (width - 2.0 * m.space_s).max(0.0)));
        }

        let body: Element<'_, Msg> = if shown == 0 {
            w::status_page()
                .icon(Icon::Folder.view(48.0))
                .title(if self.presets.is_empty() {
                    "No themes yet"
                } else {
                    "Nothing matches"
                })
                .description(if self.presets.is_empty() {
                    "Open a theme file or start a new one. Save as registers it for every app."
                } else {
                    "No theme has that name or path."
                })
                .into()
        } else {
            scrollable(container(list).padding([0.0, m.space_s]))
                .height(Length::Fill)
                .into()
        };
        column![container(filter).padding([m.space_xs, m.space_s]), body]
            .spacing(m.space_xs)
            .height(Length::Fill)
            .into()
    }

    // ------------------------------------------------------------ the tokens

    /// Token controls and editing, with an independent external-preview scheme selector.
    fn main_pane(&self, width: f32, collapsed: bool, m: iad::Metrics) -> Element<'_, Msg> {
        let controls = self.controls(width, m);
        let body = self.tokens_area(m);
        let scheme = cmp::toggle_group(
            crate::toys::LaunchScheme::ALL
                .iter()
                .map(|s| cmp::ToggleItem::new(*s, s.label()))
                .collect(),
            Some(self.launch_scheme),
            Msg::LaunchScheme,
        );
        let mut launcher = row![text("Launch appearance"), scheme]
            .spacing(m.space_s)
            .align_y(Alignment::Center);
        if collapsed {
            launcher = launcher.push(self.try_menu(m));
        }
        let launcher = launcher.wrap();
        column![controls, body, launcher]
            .spacing(m.space_s)
            .padding(m.space_s)
            .height(Length::Fill)
            .into()
    }

    /// Scheme token variation, search, modified-only filter, and copy from base.
    fn controls(&self, _width: f32, m: iad::Metrics) -> Element<'_, Msg> {
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
        let overrides = if self.variant == Variant::Base {
            cmp::switch(false)
        } else {
            cmp::switch(self.overrides_only).on_toggle(Msg::OverridesOnly)
        };
        let mut bar = row![
            row![
                text("Modified only").size(m.space_m + 1.0),
                cmp::switch(self.modified_only).on_toggle(Msg::ModifiedOnly),
            ]
            .spacing(m.space_xs)
            .align_y(Alignment::Center),
            cmp::tooltip(
                row![text("Overrides only").size(m.space_m + 1.0), overrides]
                    .spacing(m.space_xs).align_y(Alignment::Center),
                if self.variant == Variant::Base {
                    "Choose Light or Dark to show only tokens explicitly set in that scheme."
                } else {
                    "Only values explicitly set in the active scheme; inherited base values are hidden."
                },
                TooltipPosition::Bottom,
            ),
        ]
        .spacing(m.space_m)
        .align_y(Alignment::Center);
        if self.variant != Variant::Base {
            bar = bar.push(cmp::tooltip(
                cmp::flat_button(cmp::typography::label("Copy from Base"))
                    .on_press(Msg::CopyFromBase),
                "Copy what the base sets into this variant, where it sets nothing",
                TooltipPosition::Bottom,
            ));
        }
        column![
            row![variants, search]
                .spacing(m.space_m)
                .align_y(Alignment::Center),
            bar.wrap()
        ]
        .spacing(m.space_s)
        .into()
    }

    /// The token page occupies the content pane, without an embedded preview.
    fn tokens_area(&self, m: iad::Metrics) -> Element<'_, Msg> {
        let page = scrollable(
            container(self.page(m))
                .padding([m.space_s, m.space_m])
                .max_width(m.clamp_maximum + 160.0),
        )
        .id(iced::widget::Id::new(crate::tokens::TOKENS_SCROLL))
        .width(Length::Fill)
        .height(Length::Fill);
        page.into()
    }

    fn sections_pane(&self, m: iad::Metrics) -> Element<'_, Msg> {
        let list = column(self.section_choices().map(|s| {
            let n = self.set_count(s);
            let mut label = row![text(crate::tokens::section_label(s))]
                .spacing(m.space_xs)
                .width(Length::Fill);
            if n > 0 {
                label = label.push(space().width(Length::Fill)).push(
                    text(n.to_string())
                        .size(m.space_m)
                        .style(iad::styles::text::dim),
                );
            }
            let button = if s == self.effective_section() {
                cmp::raised_button(label)
            } else {
                cmp::flat_button(label)
            };
            button
                .on_press(Msg::Section(s.to_string()))
                .width(Length::Fill)
                .into()
        }))
        .spacing(m.space_xxs)
        .width(Length::Fill);
        scrollable(list).height(Length::Fill).into()
    }
}
