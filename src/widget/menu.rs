//! Menus and `GtkMenuButton`, with two presentations.
//!
//! * [`MenuPresentation::Dropdown`]: an anchored popover (the desktop default).
//! * [`MenuPresentation::FullPage`]: the menu takes over the window as a page
//!   with a header (back button + title) and large touch rows, for narrow or
//!   mobile windows.
//!
//! Conventions (see `docs/design-language/menus.md`):
//! * GNOME HIG: the primary menu button uses `open-menu-symbolic`, sits at the
//!   end of the header bar (above the list when there is a sidebar), is called
//!   "Main Menu", and ends with Preferences, Keyboard Shortcuts, Help, About.
//!   3–12 items, groups separated by hairlines, no nested submenus, no Quit.
//! * Narrow screens: Material swaps menus for bottom sheets on compact screens;
//!   libadwaita's `AdwDialog` switches to a sheet at `max-width: 450`. We follow
//!   that with [`MenuPresentation::for_width`] (≤ 450 → full page).

use super::dropdown::{Align, Appearance, Dropdown, Presentation, PAGE_BACK_WIDTH, PAGE_HEADER_HEIGHT};
use super::{boxed_list, clamp, scrolled_window, Element};
use crate::style::{button as sb, container as sc, text as st};
use crate::{Icon, Metrics, Theme, Typography};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, space, text, tooltip};
use iced::Length;

/// How an opened menu is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MenuPresentation {
    /// Anchored popover below the button.
    #[default]
    Dropdown,
    /// Full-window page with a back button and large rows.
    FullPage,
}

impl MenuPresentation {
    /// The presentation the **desktop config** asks for at this width: `[menus] presentation`
    /// (`auto` follows [`for_width`](Self::for_width)). Apps should use this; forcing a mode
    /// is an opt-out (`Config::opt_out`).
    pub fn resolve(width: f32) -> Self {
        use crate::config::MenuPresentationSetting as S;
        match crate::config::current().menu_presentation {
            S::Auto => Self::for_width(width),
            S::Dropdown => Self::Dropdown,
            S::FullPage => Self::FullPage,
        }
    }

    /// The window width at or below which phones/narrow windows should use the
    /// full-page presentation (the `AdwDialog` bottom-sheet threshold).
    pub const FULL_PAGE_MAX_WIDTH: f32 = crate::window::MOBILE_MAX_WIDTH;

    /// Picks a presentation for the window width: full page when narrow (at or below
    /// the configured mobile breakpoint).
    pub fn for_width(width: f32) -> Self {
        if crate::window::is_mobile(width) {
            Self::FullPage
        } else {
            Self::Dropdown
        }
    }
}

enum Kind<M> {
    Action { on_press: M },
    Check { checked: bool, on_press: M },
}

/// One menu entry: an action or a check item.
pub struct MenuItem<M> {
    label: String,
    accel: Option<String>,
    icon: Option<Icon>,
    destructive: bool,
    kind: Kind<M>,
}

impl<M> MenuItem<M> {
    /// A command (verb label, header capitalization).
    pub fn action(label: impl Into<String>, on_press: M) -> Self {
        Self { label: label.into(), accel: None, icon: None, destructive: false, kind: Kind::Action { on_press } }
    }
    /// A setting with a check mark (adjective label).
    pub fn check(label: impl Into<String>, checked: bool, on_press: M) -> Self {
        Self { label: label.into(), accel: None, icon: None, destructive: false, kind: Kind::Check { checked, on_press } }
    }
    /// Shown dimmed at the end, e.g. `"Ctrl+Q"`.
    pub fn accel(mut self, accel: impl Into<String>) -> Self {
        self.accel = Some(accel.into());
        self
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    /// Colors the label with the destructive color.
    pub fn destructive(mut self) -> Self {
        self.destructive = true;
        self
    }
}

/// A menu: a title (shown in the full-page header) and sections of items.
/// Sections are separated by hairlines; don't mix action and check items in one.
pub struct Menu<M> {
    title: String,
    sections: Vec<Vec<MenuItem<M>>>,
}

impl<M> Menu<M> {
    pub fn new(title: impl Into<String>) -> Self {
        Self { title: title.into(), sections: Vec::new() }
    }
    /// Adds a group of items.
    pub fn section(mut self, items: impl IntoIterator<Item = MenuItem<M>>) -> Self {
        self.sections.push(items.into_iter().collect());
        self
    }
    /// Total number of items (the HIG recommends 3–12).
    pub fn len(&self) -> usize {
        self.sections.iter().map(Vec::len).sum()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// `GtkMenuButton`: an icon button that opens a [`Menu`].
pub struct MenuButton<M> {
    icon: Icon,
    menu: Menu<M>,
    presentation: MenuPresentation,
    tooltip: Option<String>,
    window_radius: f32,
    open_initially: bool,
}

/// An icon button opening `menu`.
pub fn menu_button<M>(icon: Icon, menu: Menu<M>) -> MenuButton<M> {
    MenuButton {
        icon,
        menu,
        presentation: MenuPresentation::Dropdown,
        tooltip: None,
        window_radius: 0.0,
        open_initially: false,
    }
}

/// The primary menu button: `open-menu-symbolic`, tooltip "Main Menu".
pub fn primary_menu<M>(menu: Menu<M>) -> MenuButton<M> {
    menu_button(Icon::OpenMenu, menu).tooltip("Main Menu")
}

impl<M> MenuButton<M> {
    /// [`MenuPresentation::Dropdown`] or [`MenuPresentation::FullPage`].
    pub fn presentation(mut self, p: MenuPresentation) -> Self {
        self.presentation = p;
        self
    }
    pub fn tooltip(mut self, t: impl Into<String>) -> Self {
        self.tooltip = Some(t.into());
        self
    }
    /// Rounds the full page's corners to match a rounded window.
    pub fn window_radius(mut self, r: f32) -> Self {
        self.window_radius = r;
        self
    }
    /// Start opened (screenshots, demos).
    pub fn open_initially(mut self, open: bool) -> Self {
        self.open_initially = open;
        self
    }
}

/// Builds one row's content: `[leading] label … [accel]`.
fn row_content<'a, M: 'a>(item: &MenuItem<M>, any_check: bool) -> Element<'a, M> {
    let m = Metrics::desktop();
    let t = Typography::default();
    let (text_label, destructive, accel, icon) =
        (item.label.clone(), item.destructive, item.accel.clone(), item.icon);
    let mut r = row![].spacing(m.space_m).align_y(Vertical::Center);
    if any_check {
        let checked = matches!(item.kind, Kind::Check { checked: true, .. });
        r = r.push(if checked {
            Element::from(Icon::Check.view(m.icon_size))
        } else {
            space().width(m.icon_size).into()
        });
    }
    if let Some(i) = icon {
        r = r.push(i.view(m.icon_size));
    }
    let mut lbl = super::typography::label(text_label).width(Length::Fill);
    if destructive {
        lbl = lbl.style(st::destructive);
    }
    r = r.push(lbl);
    if let Some(a) = accel {
        r = r.push(t.caption(a).wrapping(text::Wrapping::None).style(st::dim));
    }
    r.into()
}

fn message<M: Clone>(item: &MenuItem<M>) -> M {
    match &item.kind {
        Kind::Action { on_press } | Kind::Check { on_press, .. } => on_press.clone(),
    }
}

fn dropdown_items<'a, M: Clone + 'a>(menu: &Menu<M>) -> Vec<Element<'a, M>> {
    let m = Metrics::desktop();
    let any_check = menu.sections.iter().flatten().any(|i| matches!(i.kind, Kind::Check { .. }));
    let mut out: Vec<Element<'a, M>> = Vec::new();
    for (si, section) in menu.sections.iter().enumerate() {
        if si > 0 {
            out.push(
                container(
                    container(space().height(1)).width(Length::Fill).style(sc::popover_separator),
                )
                .padding([m.space_xs, 0.0])
                .into(),
            );
        }
        for item in section {
            out.push(
                button(
                    container(row_content(item, any_check))
                        .padding([0.0, m.space_m])
                        .center_y(Length::Fixed(32.0)),
                )
                .padding(0)
                .width(Length::Fill)
                .style(sb::menu_item(false))
                .on_press(message(item))
                .into(),
            );
        }
    }
    out
}

fn page<'a, M: Clone + 'a>(menu: &Menu<M>, window_radius: f32) -> Element<'a, M> {
    let m = Metrics::desktop();
    let t = Typography::default();
    let any_check = menu.sections.iter().flatten().any(|i| matches!(i.kind, Kind::Check { .. }));

    // Header: back arrow (hit area handled by the overlay) + centered title.
    let header = container(
        row![
            container(Icon::GoPrevious.view(m.icon_size + 2.0))
                .center(Length::Fixed(PAGE_BACK_WIDTH))
                .height(Length::Fixed(PAGE_HEADER_HEIGHT)),
            container(t.heading(menu.title.clone()).wrapping(text::Wrapping::None))
                .center_x(Length::Fill)
                .center_y(Length::Fixed(PAGE_HEADER_HEIGHT)),
            space().width(PAGE_BACK_WIDTH),
        ]
        .align_y(Vertical::Center),
    )
    .width(Length::Fill)
    // Flat, like a dialog header: it shares the page background.
    .style(|th: &Theme| iced::widget::container::Style {
        text_color: Some(th.semantic.window.fg),
        ..Default::default()
    });

    // Large touch rows, one boxed list per section.
    let mut lists = column![].spacing(m.space_l);
    for section in &menu.sections {
        let rows = section.iter().map(|item| -> Element<'a, M> {
            button(
                container(row_content(item, any_check))
                    .padding(m.row_padding)
                    .center_y(Length::Fixed(m.row_min_height)),
            )
            .padding(0)
            .width(Length::Fill)
            .style(sb::list_row)
            .on_press(message(item))
            .into()
        });
        lists = lists.push(boxed_list(rows));
    }

    let body = scrolled_window(clamp(
        container(lists).padding(m.page_padding).width(Length::Fill),
        m.clamp_maximum,
    ))
    .width(Length::Fill)
    .height(Length::Fill);

    container(column![header, body])
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |th: &Theme| iced::widget::container::Style {
            border: iced::Border::default().rounded(window_radius),
            ..sc::window(th)
        })
        .into()
}

impl<'a, M: Clone + 'a> From<MenuButton<M>> for Element<'a, M> {
    fn from(b: MenuButton<M>) -> Self {
        let m = Metrics::desktop();
        let trigger = container(b.icon.view(m.icon_size))
            .center(Length::Fixed(m.button_min_height));
        let (content, presentation) = match b.presentation {
            MenuPresentation::Dropdown => (
                super::popover(dropdown_items(&b.menu), 360.0),
                Presentation::Popover,
            ),
            MenuPresentation::FullPage => (page(&b.menu, b.window_radius), Presentation::Page),
        };
        let dd = Dropdown::custom(trigger, content)
            .appearance(Appearance::Flat)
            .padding(0)
            .presentation(presentation)
            .align(Align::End)
            .popover_width(260.0)
            .open_initially(b.open_initially);
        match b.tooltip {
            Some(t) => tooltip(Element::from(dd), text(t).size(12), tooltip::Position::Bottom)
                .style(sc::osd)
                .padding(m.space_xs)
                .into(),
            None => dd.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_follows_width() {
        assert_eq!(MenuPresentation::for_width(360.0), MenuPresentation::FullPage);
        assert_eq!(MenuPresentation::for_width(450.0), MenuPresentation::FullPage);
        assert_eq!(MenuPresentation::for_width(451.0), MenuPresentation::Dropdown);
        assert_eq!(MenuPresentation::default(), MenuPresentation::Dropdown);
    }

    #[test]
    fn menu_counts_items_across_sections() {
        let m: Menu<()> = Menu::new("Main Menu")
            .section([MenuItem::check("Dark Style", false, ())])
            .section([
                MenuItem::action("Preferences", ()).accel("Ctrl+,"),
                MenuItem::action("About", ()),
            ]);
        assert_eq!(m.len(), 3);
        assert!(!m.is_empty());
    }
}
