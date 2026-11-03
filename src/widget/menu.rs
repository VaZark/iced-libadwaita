//! Menus and `GtkMenuButton`, with two presentations.
//!
//! * [`MenuPresentation::Dropdown`]: a [`Popover`] anchored to the button (the desktop default),
//!   kept inside the content area it opens from (see [`Popover::bounds`]).
//! * [`MenuPresentation::FullPage`]: the menu takes over the window as a page
//!   with a header (back button + title) and large touch rows, for narrow or
//!   mobile windows.
//!
//! Conventions (see `docs/hig-inferences/design-language/menus.md`):
//! * GNOME HIG: the primary menu button uses `open-menu-symbolic`, sits at the
//!   end of the header bar (above the list when there is a sidebar), is called
//!   "Main Menu", and ends with Preferences, Keyboard Shortcuts, Help, About.
//!   3–12 items, groups separated by hairlines, no nested submenus, no Quit.
//! * Narrow screens: Material swaps menus for bottom sheets on compact screens;
//!   libadwaita's `AdwDialog` switches to a sheet on narrow windows. We follow that with
//!   [`MenuPresentation::for_width`] (mobile mode, below 576px → full page).

use super::popover::{
    Align, Appearance, PAGE_BACK_WIDTH, PAGE_HEADER_HEIGHT, Popover, PopoverBounds, Presentation,
};
use super::{Element, boxed_list, clamp, scrolled_window};
use crate::style::{button as sb, container as sc, text as st};
use crate::{Icon, Metrics, Theme, Typography};
use iced::Length;
use iced::alignment::Vertical;
use iced::widget::{button, column, container, row, space, text, tooltip};

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

/// Which edge of a menu item holds its icon or check mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MenuIconSide {
    /// After the label (and the accelerator): the default.
    #[default]
    End,
    /// Before the label.
    Start,
}

/// How a check item shows whether it is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MenuCheckStyle {
    /// Text after the label, `On` or `Off` (GNOME's quick settings): the default.
    #[default]
    Text,
    /// A check mark, shown while on, at the edge of [`Menu::icon_side`] (GTK's own menus).
    Mark,
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
    icon_side: Option<MenuIconSide>,
    labels: Option<(String, String)>,
    destructive: bool,
    disabled: bool,
    kind: Kind<M>,
}

impl<M> MenuItem<M> {
    /// A command (verb label, header capitalization).
    pub fn action(label: impl Into<String>, on_press: M) -> Self {
        Self {
            label: label.into(),
            accel: None,
            icon: None,
            icon_side: None,
            labels: None,
            destructive: false,
            disabled: false,
            kind: Kind::Action { on_press },
        }
    }
    /// A setting with a check mark (adjective label).
    pub fn check(label: impl Into<String>, checked: bool, on_press: M) -> Self {
        Self {
            label: label.into(),
            accel: None,
            icon: None,
            icon_side: None,
            labels: None,
            destructive: false,
            disabled: false,
            kind: Kind::Check { checked, on_press },
        }
    }
    /// Shown dimmed at the end, e.g. `"Ctrl+Q"`.
    pub fn accel(mut self, accel: impl Into<String>) -> Self {
        self.accel = Some(accel.into());
        self
    }
    /// An icon, at the end of the item unless [`Menu::icon_side`] or
    /// [`icon_side`](Self::icon_side) says otherwise.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    /// The words a check item shows for on and off, in place of `On` and `Off`
    /// (`"Connected"`, `"Disconnected"`). Used by [`MenuCheckStyle::Text`].
    pub fn state_labels(mut self, on: impl Into<String>, off: impl Into<String>) -> Self {
        self.labels = Some((on.into(), off.into()));
        self
    }
    /// Puts this item's icon on `side`, whatever the menu says.
    pub fn icon_side(mut self, side: MenuIconSide) -> Self {
        self.icon_side = Some(side);
        self
    }
    /// Not available right now: dimmed text, no hover or press effect, cannot be activated.
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
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
    icon_side: MenuIconSide,
    check_style: MenuCheckStyle,
}

impl<M> Menu<M> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            sections: Vec::new(),
            icon_side: MenuIconSide::End,
            check_style: MenuCheckStyle::Text,
        }
    }
    /// The edge that holds the items' icons (and check marks, with [`MenuCheckStyle::Mark`]): [`MenuIconSide::End`] by default,
    /// [`MenuIconSide::Start`] to opt in to the leading position. An item can override its
    /// icon with [`MenuItem::icon_side`].
    pub fn icon_side(mut self, side: MenuIconSide) -> Self {
        self.icon_side = side;
        self
    }
    /// How check items show their state: text after the label ([`MenuCheckStyle::Text`], the
    /// default) or a check mark ([`MenuCheckStyle::Mark`]).
    pub fn check_style(mut self, style: MenuCheckStyle) -> Self {
        self.check_style = style;
        self
    }
    /// Adds a group of items.
    pub fn section(mut self, items: impl IntoIterator<Item = MenuItem<M>>) -> Self {
        self.sections.push(items.into_iter().collect());
        self
    }
    /// The accelerator and message of every enabled item that has an accelerator, so a
    /// [`Shortcuts`](crate::shortcut::Shortcuts) can make the shown shortcuts work.
    pub fn accelerators(&self) -> Vec<(String, M)>
    where
        M: Clone,
    {
        self.sections
            .iter()
            .flatten()
            .filter(|i| !i.disabled)
            .filter_map(|i| i.accel.clone().map(|a| (a, message(i))))
            .collect()
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
    bounds: Option<PopoverBounds>,
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
        bounds: None,
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
    /// What the menu's popover is kept inside: the content area the button is in, or the whole
    /// window. Without it the popover's default applies (`[popovers] bounds`).
    pub fn bounds(mut self, bounds: PopoverBounds) -> Self {
        self.bounds = Some(bounds);
        self
    }
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

/// Whether some item of the menu has a mark (a check or an icon) at each edge. Rows without one
/// keep a blank slot there only if another row has a mark, so labels line up and an edge no
/// item uses takes no room.
#[derive(Clone, Copy)]
struct Slots {
    lead: bool,
    trail: bool,
}

fn slots<M>(menu: &Menu<M>) -> Slots {
    let mut s = Slots {
        lead: false,
        trail: false,
    };
    for i in menu.sections.iter().flatten() {
        let mut mark = |side| match side {
            MenuIconSide::Start => s.lead = true,
            MenuIconSide::End => s.trail = true,
        };
        if matches!(i.kind, Kind::Check { .. }) && menu.check_style == MenuCheckStyle::Mark {
            mark(menu.icon_side);
        }
        if i.icon.is_some() {
            mark(i.icon_side.unwrap_or(menu.icon_side));
        }
    }
    s
}

/// Builds one row's content: `label … [accel] [mark]` (or `[mark] label … [accel]` for the
/// start side). The mark is the check and the icon, sharing one slot per edge.
fn row_content<'a, M: 'a>(
    item: &MenuItem<M>,
    side: MenuIconSide,
    style: MenuCheckStyle,
    slots: Slots,
    touch: bool,
) -> Element<'a, M> {
    let m = Metrics::desktop();
    let t = Typography::default();
    // The accelerator is shown only when it is available (see `shortcut::hint`).
    let accel = crate::shortcut::hint(item.accel.as_deref(), !item.disabled, touch);
    let disabled = item.disabled;

    // Check marks follow the menu's side; an icon may be moved by its item. A check item shows
    // its mark only while checked, but its slot is always there.
    let check: Option<(MenuIconSide, Element<'a, M>)> =
        (style == MenuCheckStyle::Mark && matches!(item.kind, Kind::Check { .. })).then(|| {
            let on = matches!(item.kind, Kind::Check { checked: true, .. });
            let el: Element<'a, M> = if on {
                Icon::Check
                    .view(m.icon_size)
                    .style(move |t: &Theme, _| iced::widget::svg::Style {
                        color: Some(if disabled {
                            t.semantic.dim_solid()
                        } else {
                            t.semantic.popover.fg
                        }),
                    })
                    .into()
            } else {
                space().width(m.icon_size).into()
            };
            (side, el)
        });
    let icon: Option<(MenuIconSide, Element<'a, M>)> = item
        .icon
        .map(|i| (item.icon_side.unwrap_or(side), i.view(m.icon_size).into()));

    let mut lbl = super::typography::label(item.label.clone()).width(Length::Fill);
    if item.destructive {
        lbl = lbl.style(st::destructive);
    }
    if item.disabled {
        lbl = lbl.style(st::dim);
    }
    let accel = accel.map(|a| t.caption(a).wrapping(text::Wrapping::None).style(st::dim));
    // A check item's state as words, after the accelerator and before any trailing mark.
    let state = match (&item.kind, style) {
        (Kind::Check { checked, .. }, MenuCheckStyle::Text) => {
            let (on, off) = item
                .labels
                .clone()
                .unwrap_or_else(|| ("On".into(), "Off".into()));
            let word = if *checked { on } else { off };
            let w = t.caption(word).wrapping(text::Wrapping::None);
            Some(if *checked { w } else { w.style(st::dim) })
        }
        _ => None,
    };

    let (mut lead, mut trail): (Vec<Element<'a, M>>, Vec<Element<'a, M>>) =
        (Vec::new(), Vec::new());
    for (to, el) in check.into_iter().chain(icon) {
        match to {
            MenuIconSide::Start => lead.push(el),
            MenuIconSide::End => trail.push(el),
        }
    }
    let blank = || -> Element<'a, M> { space().width(m.icon_size).into() };
    if lead.is_empty() && slots.lead {
        lead.push(blank());
    }
    if trail.is_empty() && slots.trail {
        trail.push(blank());
    }
    let mut r = row![].spacing(m.space_m).align_y(Vertical::Center);
    r = r
        .extend(lead)
        .push(lbl)
        .extend(accel.map(Element::from))
        .extend(state.map(Element::from));
    r = r.extend(trail);
    r.into()
}

fn message<M: Clone>(item: &MenuItem<M>) -> M {
    match &item.kind {
        Kind::Action { on_press } | Kind::Check { on_press, .. } => on_press.clone(),
    }
}

/// The popover for `menu`, as a dropdown shows it.
pub(super) fn popover_content<'a, M: Clone + 'a>(
    menu: &Menu<M>,
    max_height: f32,
) -> Element<'a, M> {
    super::popover(dropdown_items(menu), max_height)
}

fn dropdown_items<'a, M: Clone + 'a>(menu: &Menu<M>) -> Vec<Element<'a, M>> {
    let m = Metrics::desktop();
    let slots = slots(menu);
    let mut out: Vec<Element<'a, M>> = Vec::new();
    for (si, section) in menu.sections.iter().enumerate() {
        if si > 0 {
            out.push(
                container(
                    container(space().height(1))
                        .width(Length::Fill)
                        .style(sc::popover_separator),
                )
                .padding([m.space_xs, 0.0])
                .into(),
            );
        }
        for item in section {
            out.push(
                button(
                    container(row_content(
                        item,
                        menu.icon_side,
                        menu.check_style,
                        slots,
                        false,
                    ))
                    .padding([0.0, m.space_m])
                    .center_y(Length::Fixed(32.0)),
                )
                .padding(0)
                .width(Length::Fill)
                .style(sb::menu_item(false))
                .on_press_maybe((!item.disabled).then(|| message(item)))
                .into(),
            );
        }
    }
    out
}

fn page<'a, M: Clone + 'a>(menu: &Menu<M>, window_radius: f32) -> Element<'a, M> {
    let m = Metrics::desktop();
    let t = Typography::default();
    let slots = slots(menu);

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
                container(row_content(
                    item,
                    menu.icon_side,
                    menu.check_style,
                    slots,
                    true,
                ))
                .padding(m.row_padding)
                .center_y(Length::Fixed(m.row_min_height)),
            )
            .padding(0)
            .width(Length::Fill)
            .style(sb::list_row)
            .on_press_maybe((!item.disabled).then(|| message(item)))
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
        let trigger =
            container(b.icon.view(m.icon_size)).center(Length::Fixed(m.button_min_height));
        let (content, presentation) = match b.presentation {
            MenuPresentation::Dropdown => (
                super::popover(dropdown_items(&b.menu), 360.0),
                Presentation::Popover,
            ),
            MenuPresentation::FullPage => (page(&b.menu, b.window_radius), Presentation::Page),
        };
        let mut dd = Popover::new(trigger, content)
            .appearance(Appearance::Flat)
            .padding(0)
            .presentation(presentation)
            .align(Align::End)
            .popover_width(260.0)
            .open_initially(b.open_initially);
        if let Some(bounds) = b.bounds {
            dd = dd.bounds(bounds);
        }
        let button: Element<'a, M> = match b.tooltip {
            Some(t) => tooltip(
                Element::from(dd),
                text(t).size(12),
                tooltip::Position::Bottom,
            )
            .style(sc::osd)
            .padding(m.space_xs)
            .into(),
            None => dd.into(),
        };
        // The menu registers the accelerators it shows, so they work while it is in the view.
        crate::shortcut::Shortcuts::new()
            .from_menu(&b.menu)
            .scope(button)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presentation_follows_width() {
        assert_eq!(
            MenuPresentation::for_width(360.0),
            MenuPresentation::FullPage
        );
        assert_eq!(
            MenuPresentation::for_width(575.0),
            MenuPresentation::FullPage
        );
        assert_eq!(
            MenuPresentation::for_width(576.0),
            MenuPresentation::Dropdown
        );
        assert_eq!(MenuPresentation::default(), MenuPresentation::Dropdown);
    }

    #[test]
    fn icons_are_at_the_end_unless_asked() {
        let m: Menu<()> = Menu::new("Menu");
        assert_eq!(m.icon_side, MenuIconSide::End);
        let m = m.icon_side(MenuIconSide::Start);
        assert_eq!(m.icon_side, MenuIconSide::Start);
        let item = MenuItem::action("Edit", ()).icon(Icon::Check);
        assert_eq!(item.icon_side, None);
        assert_eq!(
            item.icon_side(MenuIconSide::Start).icon_side,
            Some(MenuIconSide::Start)
        );
    }

    #[test]
    fn an_edge_nobody_uses_takes_no_room() {
        let icons: Menu<()> = Menu::new("m").icon_side(MenuIconSide::Start).section([
            MenuItem::action("Edit", ()).icon(Icon::Edit),
            MenuItem::action("Plain", ()),
        ]);
        let s = slots(&icons);
        assert!(s.lead && !s.trail);
        // Icons and a check share the one slot at their edge.
        let both = icons
            .check_style(MenuCheckStyle::Mark)
            .section([MenuItem::check("Compact", true, ())]);
        let s = slots(&both);
        assert!(s.lead && !s.trail);
        let end: Menu<()> = Menu::new("m")
            .check_style(MenuCheckStyle::Mark)
            .section([MenuItem::check("Compact", true, ())]);
        let s = slots(&end);
        assert!(!s.lead && s.trail);
        // As text (the default) a check item needs no mark slot.
        let text: Menu<()> = Menu::new("m").section([MenuItem::check("Compact", true, ())]);
        assert!(!slots(&text).lead && !slots(&text).trail);
        assert_eq!(text.check_style, MenuCheckStyle::Text);
        let none: Menu<()> = Menu::new("m").section([MenuItem::action("Plain", ())]);
        assert!(!slots(&none).lead && !slots(&none).trail);
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
