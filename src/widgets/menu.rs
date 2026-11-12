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
//!   3–12 items, groups separated by hairlines, no cascading submenus, no Quit. A submenu that
//!   slides in place of its parent ([`MenuItem::submenu`]) is not a cascade and is supported.
//! * Narrow screens: Material swaps menus for bottom sheets on compact screens;
//!   libadwaita's `AdwDialog` switches to a sheet on narrow windows. We follow that with
//!   [`MenuPresentation::for_width`] (mobile mode, below 576px → full page).

use crate::Element;
use crate::layouts::scrolled_window;
use crate::styles::{button as sb, container as sc, text as st};
use crate::widgets::menu_pages::{MenuPages, NavRow, Request, plan};
use crate::widgets::popover::{
    Align, Appearance, PAGE_BACK_WIDTH, PAGE_HEADER_HEIGHT, Popover, PopoverBounds, Presentation,
};
use crate::widgets::{boxed_list, clamp};
use crate::{Icon, IconSource, Metrics, Theme, Typography};
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
    pub const FULL_PAGE_MAX_WIDTH: f32 = crate::breakpoints::MOBILE_MAX_WIDTH;

    /// Picks a presentation for the window width: full page when narrow (at or below
    /// the configured mobile breakpoint).
    pub fn for_width(width: f32) -> Self {
        if crate::breakpoints::is_mobile(width) {
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
    Action {
        on_press: M,
    },
    Check {
        checked: bool,
        on_press: M,
    },
    /// A boolean setting shown as a switch after the label; toggling it leaves the menu open.
    Switch {
        checked: bool,
        on_press: M,
    },
    /// Opens another page of the menu in place of this one.
    Submenu {
        menu: Box<Menu<M>>,
    },
}

/// One menu entry: an action, a check item or a submenu.
pub struct MenuItem<M> {
    label: String,
    accel: Option<String>,
    icon: Option<IconSource>,
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
    /// A boolean setting as `label … [switch]`. Unlike [`check`](Self::check) it leaves the menu
    /// open when toggled, so several can be changed in one visit.
    pub fn switch(label: impl Into<String>, checked: bool, on_press: M) -> Self {
        Self {
            label: label.into(),
            accel: None,
            icon: None,
            icon_side: None,
            labels: None,
            destructive: false,
            disabled: false,
            kind: Kind::Switch { checked, on_press },
        }
    }
    /// A row with a chevron that opens `menu` in place of this one: its items slide in from the
    /// end and a back row at their top slides them away again. Not a cascade beside the parent
    /// (the HIG forbids that); it is GTK's sliding submenu. The title of `menu` is the back
    /// row's label. In the full-page presentation and in [`Menu::sheet`], where there is nothing
    /// to slide, a submenu's items follow under its label instead.
    pub fn submenu(label: impl Into<String>, menu: Menu<M>) -> Self {
        Self {
            label: label.into(),
            accel: None,
            icon: None,
            icon_side: None,
            labels: None,
            destructive: false,
            disabled: false,
            kind: Kind::Submenu {
                menu: Box::new(menu),
            },
        }
    }
    /// Shown dimmed at the end, e.g. `"Ctrl+Q"`.
    pub fn accel(mut self, accel: impl Into<String>) -> Self {
        self.accel = Some(accel.into());
        self
    }
    /// An icon (a system [`Icon`] or your own SVG, see [`IconSource`]), at the end of the item unless [`Menu::icon_side`] or
    /// [`icon_side`](Self::icon_side) says otherwise.
    pub fn icon(mut self, icon: impl Into<IconSource>) -> Self {
        self.icon = Some(icon.into());
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
    /// The menu as the content of a bottom sheet (put it in a [`Dock`](crate::layouts::Dock)): the title
    /// and an optional dimmed line under it, then the large touch rows. The sheet closes on a
    /// choice by the app mapping the messages, as with any menu.
    pub fn sheet<'a>(&self, subtitle: Option<String>) -> Element<'a, M>
    where
        M: Clone + 'a,
    {
        let m = Metrics::desktop();
        let t = Typography::default();
        let mut head = column![t.heading(self.title.clone())];
        if let Some(s) = subtitle {
            head = head.push(t.caption(s).style(st::dim));
        }
        container(column![
            container(head).padding(m.page_padding),
            container(touch_lists(self)).padding(m.page_padding),
        ])
        .width(Length::Fill)
        .into()
    }
    /// The accelerator and message of every enabled item that has an accelerator, so a
    /// [`Shortcuts`](crate::shortcut::Shortcuts) can make the shown shortcuts work.
    pub fn accelerators(&self) -> Vec<(String, M)>
    where
        M: Clone,
    {
        self.all_items()
            .into_iter()
            .filter(|i| !i.disabled)
            .filter_map(|i| Some((i.accel.clone()?, message(i)?)))
            .collect()
    }
    /// Every item, those of submenus (at any depth) included.
    fn all_items(&self) -> Vec<&MenuItem<M>> {
        let mut out = Vec::new();
        for item in self.sections.iter().flatten() {
            out.push(item);
            if let Kind::Submenu { menu } = &item.kind {
                out.extend(menu.all_items());
            }
        }
        out
    }
    /// The menus this one's submenu rows open, in order.
    fn subs(&self) -> Vec<&Menu<M>> {
        self.sections
            .iter()
            .flatten()
            .filter_map(|i| match &i.kind {
                Kind::Submenu { menu } => Some(&**menu),
                _ => None,
            })
            .collect()
    }
    fn has_submenu(&self) -> bool {
        !self.subs().is_empty()
    }
    /// Total number of items (the HIG recommends 3–12).
    pub fn len(&self) -> usize {
        self.sections.iter().map(Vec::len).sum()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// How wide a menu's popover is.
const POPOVER_WIDTH: f32 = 260.0;

/// `GtkMenuButton`: an icon button that opens a [`Menu`].
pub struct MenuButton<M> {
    icon: IconSource,
    menu: Menu<M>,
    presentation: MenuPresentation,
    tooltip: Option<String>,
    window_radius: f32,
    open_initially: bool,
    open_page: usize,
    bounds: Option<PopoverBounds>,
}

/// An icon button opening `menu`.
pub fn menu_button<M>(icon: impl Into<IconSource>, menu: Menu<M>) -> MenuButton<M> {
    MenuButton {
        icon: icon.into(),
        menu,
        presentation: MenuPresentation::Dropdown,
        tooltip: None,
        window_radius: 0.0,
        open_initially: false,
        open_page: 0,
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
    /// With [`open_initially`](Self::open_initially), start on a submenu's page rather than the
    /// root, sliding in as if it had been chosen. Pages are numbered depth first: the root is 0,
    /// its first submenu 1, that submenu's first submenu 2, then the root's second submenu.
    pub fn open_on_page(mut self, page: usize) -> Self {
        self.open_page = page;
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
        // A submenu row ends in a chevron, so the other rows keep a slot there to line up.
        if matches!(i.kind, Kind::Submenu { .. }) {
            s.trail = true;
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
                            t.semantic.dim()
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
        .as_ref()
        .map(|i| (item.icon_side.unwrap_or(side), i.view(m.icon_size).into()));

    let mut lbl = crate::components::typography::label(item.label.clone()).width(Length::Fill);
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

    // A switch is only drawn: the row's button takes the press (it has no handler of its own, so
    // it would look disabled, hence the style).
    let switch = match item.kind {
        Kind::Switch { checked, .. } => Some(
            iced::widget::toggler(checked)
                .size(m.icon_size + 4.0)
                .style(move |t: &Theme, s| {
                    use iced::widget::toggler::Status;
                    let s = match s {
                        Status::Disabled { is_toggled } if !disabled => {
                            Status::Active { is_toggled }
                        }
                        s => s,
                    };
                    crate::styles::toggler::default(t, s)
                }),
        ),
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
    if matches!(item.kind, Kind::Submenu { .. }) {
        trail.push(Icon::GoNext.view(m.icon_size).into());
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
        .extend(state.map(Element::from))
        .extend(switch.map(Element::from));
    r = r.extend(trail);
    r.into()
}

/// What choosing the item sends. A submenu row sends nothing: it opens a page.
fn message<M: Clone>(item: &MenuItem<M>) -> Option<M> {
    match &item.kind {
        Kind::Action { on_press }
        | Kind::Check { on_press, .. }
        | Kind::Switch { on_press, .. } => Some(on_press.clone()),
        Kind::Submenu { .. } => None,
    }
}

/// `b` sending what choosing `item` sends. A switch keeps the popover open while it does.
fn pressed<'a, M: Clone + 'a>(
    b: iced::widget::Button<'a, M, Theme>,
    item: &MenuItem<M>,
) -> iced::widget::Button<'a, M, Theme> {
    match (item.disabled, message(item)) {
        (false, Some(m)) if matches!(item.kind, Kind::Switch { .. }) => {
            b.on_press_with(move || {
                crate::widgets::popover::stay_open();
                m.clone()
            })
        }
        (false, Some(m)) => b.on_press(m),
        _ => b,
    }
}

/// The popover for `menu`, as a dropdown shows it.
pub(crate) fn popover_content<'a, M: Clone + 'a>(
    menu: &Menu<M>,
    max_height: f32,
) -> Element<'a, M> {
    crate::widgets::popover(dropdown_items(menu, None), max_height)
}

/// A hairline between groups of rows.
fn separator<'a, M: 'a>() -> Element<'a, M> {
    let m = Metrics::desktop();
    container(
        container(space().height(1))
            .width(Length::Fill)
            .style(sc::popover_separator),
    )
    .padding([m.space_xs, 0.0])
    .into()
}

/// The rows of a dropdown. `pages` says where this menu's submenu rows lead: the page each one
/// opens, in order, and the [`Request`] they tell. Without it a submenu row is inert.
fn dropdown_items<'a, M: Clone + 'a>(
    menu: &Menu<M>,
    pages: Option<(&[usize], &Request)>,
) -> Vec<Element<'a, M>> {
    let m = Metrics::desktop();
    let slots = slots(menu);
    let mut out: Vec<Element<'a, M>> = Vec::new();
    let mut next_submenu = 0;
    for (si, section) in menu.sections.iter().enumerate() {
        if si > 0 {
            out.push(separator());
        }
        for item in section {
            let content = container(row_content(
                item,
                menu.icon_side,
                menu.check_style,
                slots,
                false,
            ))
            .padding([0.0, m.space_m])
            .center_y(Length::Fixed(32.0));
            if let Kind::Submenu { .. } = &item.kind {
                let target = pages.and_then(|(children, _)| children.get(next_submenu).copied());
                next_submenu += 1;
                out.push(match (target, pages, item.disabled) {
                    (Some(t), Some((_, request)), false) => {
                        NavRow::new(content.width(Length::Fill), t, request).into()
                    }
                    // Nowhere to go (or switched off): a row that does nothing.
                    _ => button(content)
                        .padding(0)
                        .width(Length::Fill)
                        .style(sb::menu_item(false))
                        .into(),
                });
                continue;
            }
            out.push(
                pressed(
                    button(content)
                        .padding(0)
                        .width(Length::Fill)
                        .style(sb::menu_item(false)),
                    item,
                )
                .into(),
            );
        }
    }
    out
}

/// The back row at the top of a subpage: an arrow and the submenu's title, sliding back to
/// `parent`.
fn back_row<'a, M: Clone + 'a>(menu: &Menu<M>, parent: usize, request: &Request) -> Element<'a, M> {
    let m = Metrics::desktop();
    let t = Typography::default();
    NavRow::new(
        container(
            row![
                Icon::GoPrevious.view(m.icon_size),
                t.heading(menu.title.clone()).wrapping(text::Wrapping::None)
            ]
            .spacing(m.space_m)
            .align_y(Vertical::Center),
        )
        .padding([0.0, m.space_m])
        .width(Length::Fill)
        .center_y(Length::Fixed(32.0)),
        parent,
        request,
    )
    .into()
}

/// The content of a dropdown whose menu has submenus: one page per menu, sliding as rows ask.
fn pages_content<'a, M: Clone + 'a>(menu: &Menu<M>, width: f32, initial: usize) -> Element<'a, M> {
    fn collect<'m, M>(m: &'m Menu<M>, out: &mut Vec<&'m Menu<M>>) {
        out.push(m);
        for s in m.subs() {
            collect(s, out);
        }
    }
    let request = Request::default();
    let numbering = plan(menu, &|m: &Menu<M>| m.subs());
    let mut menus = Vec::new();
    collect(menu, &mut menus);
    let pages = menus
        .iter()
        .zip(&numbering)
        .map(|(m, (parent, children))| {
            let mut rows: Vec<Element<'a, M>> = Vec::new();
            if let Some(p) = parent {
                rows.push(back_row(m, *p, &request));
                rows.push(separator());
            }
            rows.extend(dropdown_items(m, Some((children, &request))));
            // Just the rows, scrolling if they are many: the surface is drawn once, around the
            // pages, so they slide inside one card rather than as cards side by side.
            crate::layouts::scrolled_window(column(rows).spacing(2).width(Length::Fill)).into()
        })
        .collect();
    let m = Metrics::desktop();
    let pad = iced::Padding::from(m.popover_padding);
    let inner = width - pad.left - pad.right;
    container(MenuPages::new(pages, &request, inner, initial))
        .padding(m.popover_padding)
        .max_height(360.0)
        .style(sc::popover)
        .into()
}

/// Large touch rows, one boxed list per section: the body of the full-page menu and of
/// [`Menu::sheet`].
fn touch_lists<'a, M: Clone + 'a>(menu: &Menu<M>) -> iced::widget::Column<'a, M, Theme> {
    let m = Metrics::desktop();
    let t = Typography::default();
    let slots = slots(menu);
    let mut lists = column![].spacing(m.space_l);
    for section in &menu.sections {
        let rows: Vec<Element<'a, M>> = section
            .iter()
            .filter(|i| !matches!(i.kind, Kind::Submenu { .. }))
            .map(|item| -> Element<'a, M> {
                let row_button = button(
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
                .style(sb::list_row);
                pressed(row_button, item).into()
            })
            .collect();
        if !rows.is_empty() {
            lists = lists.push(boxed_list(rows));
        }
        // There is no page to slide to here, so a submenu's items follow under its label.
        for item in section {
            if let Kind::Submenu { menu: sub } = &item.kind {
                lists = lists.push(
                    column![
                        t.caption(item.label.clone()).style(st::dim),
                        touch_lists(sub)
                    ]
                    .spacing(m.space_s),
                );
            }
        }
    }
    lists
}

fn page<'a, M: Clone + 'a>(menu: &Menu<M>, window_radius: f32) -> Element<'a, M> {
    let m = Metrics::desktop();
    let t = Typography::default();

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

    let lists = touch_lists(menu);

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
            MenuPresentation::Dropdown if b.menu.has_submenu() => (
                pages_content(&b.menu, POPOVER_WIDTH, b.open_page),
                Presentation::Popover,
            ),
            MenuPresentation::Dropdown => (
                crate::widgets::popover(dropdown_items(&b.menu, None), 360.0),
                Presentation::Popover,
            ),
            MenuPresentation::FullPage => (page(&b.menu, b.window_radius), Presentation::Page),
        };
        let mut dd = Popover::new(trigger, content)
            .appearance(Appearance::Flat)
            .padding(0)
            .presentation(presentation)
            .align(Align::End)
            .popover_width(POPOVER_WIDTH)
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
    fn a_submenu_row_opens_a_page_and_sends_nothing() {
        let sort: Menu<u8> = Menu::new("Sort").section([
            MenuItem::check("Title", true, 1).accel("Ctrl+1"),
            MenuItem::check("Newest", false, 2),
        ]);
        let root: Menu<u8> = Menu::new("Menu")
            .section([MenuItem::action("Settings", 9)])
            .section([MenuItem::submenu("Sort", sort)]);
        assert!(root.has_submenu());
        assert_eq!(root.subs().len(), 1);
        assert_eq!(root.subs()[0].title, "Sort");
        // The submenu row is one row of the parent; what is in it is not counted there.
        assert_eq!(root.len(), 2);
        let row = &root.sections[1][0];
        assert_eq!(message(row), None);
        assert_eq!(message(&root.sections[0][0]), Some(9));
        // The shortcut of an item inside the submenu still works.
        assert_eq!(root.accelerators(), vec![("Ctrl+1".to_string(), 1)]);
        let plain: Menu<u8> = Menu::new("m").section([MenuItem::action("A", 1)]);
        assert!(!plain.has_submenu());
    }

    #[test]
    fn submenus_nest_to_any_depth() {
        let deep: Menu<u8> = Menu::new("c").section([MenuItem::action("Leaf", 3).accel("Ctrl+L")]);
        let mid: Menu<u8> = Menu::new("b").section([MenuItem::submenu("C", deep)]);
        let root: Menu<u8> = Menu::new("a").section([MenuItem::submenu("B", mid)]);
        assert_eq!(root.accelerators(), vec![("Ctrl+L".to_string(), 3)]);
        assert_eq!(root.all_items().len(), 3);
        let numbering = plan(&root, &|m: &Menu<u8>| m.subs());
        assert_eq!(
            numbering,
            vec![(None, vec![1]), (Some(0), vec![2]), (Some(1), vec![])]
        );
    }

    #[test]
    fn a_submenu_row_gives_the_other_rows_a_slot_for_its_chevron() {
        let m: Menu<()> = Menu::new("m").section([
            MenuItem::action("Plain", ()),
            MenuItem::submenu("More", Menu::new("More")),
        ]);
        let s = slots(&m);
        assert!(s.trail && !s.lead);
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
