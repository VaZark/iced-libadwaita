//! AppShell: a Mantine-style application shell (header, navbar, drawer, footer, main) whose
//! layouts differ in kind. The gallery opens it in a window of its own (Layout page).
//!
//! The parent is [`Adaptive`]: it maps width classes to whole layouts (`Layouts`), each
//! declaring the widths it supports, and runs the ordered `Close(old)` then `Open(new)`
//! transition when the choice changes. Two kinds of trigger feed it, as in Mantine:
//!
//! * **breakpoint triggers**: the width (`Adaptive::resize`), within each layout's range;
//! * **manual triggers**: `Adaptive::pin`, accepted only where the layout supports the width,
//!   plus the burger buttons, which are plain app state.
//!
//! The downstream is [`w::SidebarList`]: the navbar and the drawer's list are the same
//! component, built from the same [`w::SidebarItem`]s in every layout. Its presentation follows the
//! layout that is shown (`SidebarList::follow`); as a parent page it is a page of cards.
//!
//! Two things are independent: what the *sidebar* becomes when collapsed (a drawer or a parent
//! page, `[navigation] mobile-sidebar`) and the *bottom navigation* (a layout choice: Mobile has
//! it in both modes). The *drawer* is a pane that pops up from an edge, by default the right on
//! a desktop and the bottom (a sheet) elsewhere.
//!
//! All state (page, panes open, mobile sidebar mode) lives here in the app, so changing layout
//! loses nothing. The panel in the main area drives the knobs: hysteresis (pixels or a delay),
//! the close/open budgets, and whether the layouts answer their commands (turn it off to watch
//! the coordinator time them out).
//!
//! Start-up switches for screenshots: `APPSHELL_PIN=mobile|tablet|desktop`,
//! `APPSHELL_SIDEBAR_MODE=drawer|parent-page`, `APPSHELL_CONTENT=1`, `APPSHELL_NAV_DRAWER=1`,
//! `APPSHELL_DRAWER=1`, `APPSHELL_EDGE=right|bottom`, `APPSHELL_NAVBAR=0`.

use adw::Icon;
use adw::style;
use adw::transition::{Budgets, Command, Event, Outcome};
use adw::widget::{self as w, typography as ty};
use iced::widget::{column, container, row, scrollable, text};
use iced::{Length, Subscription, Task};
use iced_libadwaita as adw;
use std::time::{Duration, Instant};

use adw::adaptive::{Adaptive, Hysteresis, Layouts, adaptive};
use adw::breakpoints::{Breakpoints, WidthClass};

type Element<'a> = adw::Element<'a, Message>;

const NAVBAR_WIDTH: f32 = 240.0;
const DRAWER_WIDTH: f32 = 260.0;

/// The layouts of this app. Whatever the dev needs: each is built and judged alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    /// Header with a burger, a navbar that is a drawer, a bottom tab bar.
    Mobile,
    /// Header, an in-flow navbar that can be collapsed.
    Tablet,
    /// Header, navbar, aside and a footer.
    Desktop,
}

impl std::fmt::Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Shell::Mobile => "Mobile",
            Shell::Tablet => "Tablet",
            Shell::Desktop => "Desktop",
        })
    }
}

/// The breakpoint triggers. Each layout declares the widths it supports, and those are
/// respected: the width picks the first layout whose range holds it, and a manual trigger may
/// only choose among the others that hold it too (here, where ranges overlap):
///
/// ```text
/// Xs   Sm   Md   Lg   Xl   Xxl
/// Mobile ----
///      Tablet ----
///           Desktop -------------
/// ```
fn layouts() -> Layouts<Shell> {
    Layouts::new()
        .layout(..=WidthClass::Sm, Shell::Mobile)
        .layout(WidthClass::Sm..=WidthClass::Md, Shell::Tablet)
        .layout(WidthClass::Md.., Shell::Desktop)
}

/// The Layout control: follow the width, or pin one layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    Breakpoints,
    Pinned(Shell),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hyst {
    Pixels,
    Delay,
}

/// Which window edges a piece of the shell touches: it rounds its corners and shows the window
/// buttons there.
#[derive(Debug, Clone, Copy)]
struct Touches {
    left: bool,
    right: bool,
}

/// The edge the drawer pops up from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Right,
    Bottom,
}

/// What the drawer lists: the same `SidebarItem` as the navbar, a different list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Overview,
    Activity,
    Notes,
}

impl Section {
    const ALL: [Section; 3] = [Section::Overview, Section::Activity, Section::Notes];

    fn label(self) -> &'static str {
        match self {
            Section::Overview => "Overview",
            Section::Activity => "Activity",
            Section::Notes => "Notes",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Dashboard,
    Inbox,
    Settings,
}

impl Page {
    const ALL: [Page; 3] = [Page::Dashboard, Page::Inbox, Page::Settings];

    fn label(self) -> &'static str {
        match self {
            Page::Dashboard => "Dashboard",
            Page::Inbox => "Inbox",
            Page::Settings => "Settings",
        }
    }

    fn blurb(self) -> &'static str {
        match self {
            Page::Dashboard => "Layouts, triggers and transitions",
            Page::Inbox => "Messages waiting for you",
            Page::Settings => "Preferences for this app",
        }
    }

    fn icon(self) -> Icon {
        match self {
            Page::Dashboard => Icon::Home,
            Page::Inbox => Icon::Search,
            Page::Settings => Icon::Settings,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    Resized(f32),
    Tick(Instant),
    Transition(Event<Shell>),
    Page(Page),
    Section(Section),
    Back,
    MobileSidebar(w::CollapsedSidebar),
    /// Window chrome with client-side decorations; the parent app performs it on this window.
    Window(adw::window::Action),
    /// The window's maximized state, to draw the frame (the gallery queries it on resize).
    Maximized(bool),
    // Manual triggers owned by the app.
    /// The navbar as a drawer (Mobile).
    ToggleNavDrawer,
    ToggleNavbar,
    /// The drawer that pops up from an edge.
    ToggleDrawer,
    /// `None`: by layout (right on a desktop, bottom elsewhere).
    DrawerEdge(Option<Edge>),
    // The Layout control.
    Trigger(Trigger),
    Hysteresis(Hyst),
    Answer(bool),
    CloseMs(f32),
    OpenMs(f32),
}

pub struct AppShell {
    maximized: bool,
    adaptive: Adaptive<Shell>,
    width: f32,
    hyst: Hyst,
    // Layouts hold only visual state; this is the app's.
    page: Page,
    section: Section,
    /// What the sidebar becomes on Mobile: `[navigation] mobile-sidebar`, here switchable.
    mobile_sidebar: w::CollapsedSidebar,
    /// Parent page mode: whether the content page is pushed over the sidebar page.
    show_content: bool,
    /// The navbar as a drawer (Mobile).
    nav_drawer_open: bool,
    navbar_open: bool,
    /// The drawer popping up from an edge, and which edge (`None`: by layout).
    drawer_open: bool,
    drawer_edge: Option<Edge>,
    // Test knobs.
    answer: bool,
    close_ms: f32,
    open_ms: f32,
    log: Vec<String>,
}

impl AppShell {
    /// The shell for a window `width` px wide. Its decorations follow the toolkit config, like
    /// every window.
    pub fn new(width: f32) -> (Self, Task<Message>) {
        let mut app = Self {
            maximized: false,
            adaptive: Adaptive::new(layouts()),
            width,
            hyst: Hyst::Pixels,
            page: Page::Dashboard,
            section: Section::Overview,
            mobile_sidebar: match std::env::var("APPSHELL_SIDEBAR_MODE").as_deref() {
                Ok("parent-page") => w::CollapsedSidebar::ParentPage,
                Ok("drawer") => w::CollapsedSidebar::Drawer,
                _ => adw::config::current().mobile_sidebar,
            },
            show_content: std::env::var_os("APPSHELL_CONTENT").is_some(),
            nav_drawer_open: std::env::var_os("APPSHELL_NAV_DRAWER").is_some(),
            navbar_open: std::env::var("APPSHELL_NAVBAR").as_deref() != Ok("0"),
            drawer_open: std::env::var_os("APPSHELL_DRAWER").is_some(),
            drawer_edge: match std::env::var("APPSHELL_EDGE").as_deref() {
                Ok("right") => Some(Edge::Right),
                Ok("bottom") => Some(Edge::Bottom),
                _ => None,
            },
            answer: true,
            close_ms: 150.0,
            open_ms: 150.0,
            log: Vec::new(),
        };
        let cmds = app.adaptive.resize_with(
            &Breakpoints::current(),
            app.budgets(),
            width,
            Instant::now(),
        );
        let mut tasks = vec![app.dispatch(cmds)];
        // `APPSHELL_PIN=mobile|tablet|desktop` pins a layout from the start (if its range
        // holds the width).
        if let Ok(v) = std::env::var("APPSHELL_PIN") {
            let pin = [Shell::Mobile, Shell::Tablet, Shell::Desktop]
                .into_iter()
                .find(|s| s.to_string().eq_ignore_ascii_case(&v));
            tasks.push(Task::done(Message::Trigger(
                pin.map_or(Trigger::Breakpoints, Trigger::Pinned),
            )));
        }
        (app, Task::batch(tasks))
    }

    /// What the Layout control shows: the adaptive's own pin, which it drops when a resize
    /// leaves the pinned layout's range.
    fn trigger(&self) -> Trigger {
        self.adaptive
            .pinned()
            .map_or(Trigger::Breakpoints, |s| Trigger::Pinned(*s))
    }

    fn budgets(&self) -> Budgets {
        Budgets {
            close: Duration::from_millis(self.close_ms as u64),
            open: Duration::from_millis(self.open_ms as u64),
        }
    }

    /// Hands commands to the layouts. A layout that animates would start now and send its
    /// event when done; these answer at once, or not at all when "Layouts answer" is off.
    fn dispatch(&mut self, cmds: Vec<Command<Shell>>) -> Task<Message> {
        let mut tasks = Vec::new();
        for c in cmds {
            let (verb, towards) = match &c {
                Command::Close { closing_for, .. } => ("Close", closing_for),
                Command::Open { opening_for, .. } => ("Open", opening_for),
            };
            let line = format!(
                "{verb} {} within {} ms{}",
                c.target(),
                c.within().as_millis(),
                towards.map_or(String::new(), |t| format!(" (for {t})")),
            );
            self.note(line);
            if self.answer {
                tasks.push(Task::done(Message::Transition(c.done())));
            }
        }
        for r in self.adaptive.take_records() {
            let how = match r.outcome {
                Outcome::Done => "done",
                Outcome::TimedOut => "TIMED OUT",
            };
            self.note(format!("  {:?} {} {how}", r.kind, r.target));
        }
        Task::batch(tasks)
    }

    fn note(&mut self, line: String) {
        self.log.push(line);
        if self.log.len() > 10 {
            self.log.remove(0);
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        let now = Instant::now();
        let cmds = match message {
            Message::Resized(width) => {
                self.width = width;
                self.adaptive
                    .resize_with(&Breakpoints::current(), self.budgets(), width, now)
            }
            Message::Tick(now) => self.adaptive.tick(now),
            Message::Transition(e) => self.adaptive.report(e, now),
            Message::Page(p) => {
                self.page = p;
                self.nav_drawer_open = false;
                self.show_content = true;
                Vec::new()
            }
            Message::Window(_) => Vec::new(), // performed by the parent app
            Message::Maximized(m) => {
                self.maximized = m;
                Vec::new()
            }
            Message::Back => {
                self.show_content = false;
                Vec::new()
            }
            Message::MobileSidebar(m) => {
                self.mobile_sidebar = m;
                self.nav_drawer_open = false;
                self.show_content = false;
                Vec::new()
            }
            Message::Section(sec) => {
                self.section = sec;
                Vec::new()
            }
            Message::ToggleNavDrawer => {
                self.nav_drawer_open = !self.nav_drawer_open;
                Vec::new()
            }
            Message::ToggleDrawer => {
                self.drawer_open = !self.drawer_open;
                Vec::new()
            }
            Message::DrawerEdge(e) => {
                self.drawer_edge = e;
                Vec::new()
            }
            Message::ToggleNavbar => {
                self.navbar_open = !self.navbar_open;
                Vec::new()
            }
            Message::Trigger(t) => {
                self.nav_drawer_open = false;
                let pin = match t {
                    Trigger::Breakpoints => None,
                    Trigger::Pinned(s) => Some(s),
                };
                match self
                    .adaptive
                    .pin_with(&Breakpoints::current(), self.budgets(), pin, now)
                {
                    Ok(cmds) => cmds,
                    Err(e) => {
                        let width = self.width as u32;
                        self.note(format!("refused: {pin:?}: {e} ({width} px)"));
                        Vec::new()
                    }
                }
            }
            Message::Hysteresis(h) => {
                self.hyst = h;
                let setting = match h {
                    Hyst::Pixels => Hysteresis::default(),
                    Hyst::Delay => Hysteresis::Delay(adw::adaptive::DELAY),
                };
                self.adaptive = std::mem::replace(&mut self.adaptive, Adaptive::new(layouts()))
                    .hysteresis(setting);
                Vec::new()
            }
            Message::Answer(a) => {
                self.answer = a;
                Vec::new()
            }
            Message::CloseMs(v) => {
                self.close_ms = v;
                Vec::new()
            }
            Message::OpenMs(v) => {
                self.open_ms = v;
                Vec::new()
            }
        };
        self.dispatch(cmds)
    }

    /// Ticks while a transition or a hysteresis delay is pending. Resizes arrive as
    /// [`Message::Resized`], routed by the window they belong to.
    pub fn subscription(&self) -> Subscription<Message> {
        self.adaptive.subscription(Message::Tick)
    }

    // ---- sections ---------------------------------------------------------------------
    //
    // The navbar and aside are full height; the header belongs to the content area between
    // them. So the navbar's toggle sits at the start of the header, the aside's at its end, and
    // both ride the content area's edges as the panes slide in and out.

    /// The window's decorations, for the part of it that touches `edges`.
    fn chrome(&self, edges: Touches) -> adw::window::Chrome {
        adw::window::Chrome::new(adw::window::State {
            maximized: self.maximized,
            fullscreen: false,
        })
        .edges(edges.left, edges.right)
    }

    /// A header bar for a piece of the shell touching `edges`: the window buttons, drag area
    /// and rounded corners the toolkit's decorations give it.
    fn bar(&self, edges: Touches) -> w::HeaderBar<'_, Message> {
        w::header_bar().chrome(&self.chrome(edges), Message::Window)
    }

    /// `base` with the bottom corners at `edges` rounded to the window frame.
    fn rounded_bottom(
        &self,
        base: fn(&adw::Theme) -> iced::widget::container::Style,
        edges: Touches,
    ) -> impl Fn(&adw::Theme) -> iced::widget::container::Style + 'static {
        let r = self.chrome(edges).radius();
        move |t| iced::widget::container::Style {
            border: iced::Border::default().rounded(iced::border::Radius {
                top_left: 0.0,
                top_right: 0.0,
                bottom_right: if edges.right { r } else { 0.0 },
                bottom_left: if edges.left { r } else { 0.0 },
            }),
            ..base(t)
        }
    }

    /// The content area's header: the pane toggles at its ends, window buttons where it meets
    /// the window edge.
    fn header(
        &self,
        start: Option<(Icon, Message)>,
        end: Option<Message>,
        edges: Touches,
    ) -> Element<'_> {
        let mut bar = self.bar(edges).title("AppShell").subtitle(
            self.adaptive
                .shown()
                .last()
                .map_or("-".into(), |s| s.to_string()),
        );
        if let Some((icon, m)) = start {
            bar = bar.pack_start(w::icon_button(icon).on_press(m));
        }
        if let Some(m) = end {
            bar = bar.pack_end(w::icon_button(Icon::Settings).on_press(m));
        }
        bar.into()
    }

    /// The destinations, described once: every layout renders these same items.
    fn destinations(&self) -> Vec<w::SidebarItem<Message>> {
        Page::ALL
            .map(|p| {
                w::SidebarItem::new(p.label(), Message::Page(p))
                    .icon(p.icon())
                    .subtitle(p.blurb())
                    .selected(self.page == p)
            })
            .into()
    }

    /// A sidebar pane: its own header bar over a `SidebarList`, full height. The list's
    /// presentation follows the layout, the items do not change.
    fn sidebar<'a>(
        &'a self,
        title: &'static str,
        items: Vec<w::SidebarItem<Message>>,
        edges: Touches,
    ) -> Element<'a> {
        let list = w::SidebarList::new(items).follow(&self.adaptive, presentation);
        // The list is transparent: the pane supplies the sidebar color.
        container(column![
            Element::from(self.bar(edges).title(title)),
            Element::from(list),
        ])
        .style(self.rounded_bottom(style::container::sidebar, edges))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn navbar(&self) -> Element<'_> {
        self.sidebar(
            "AppShell",
            self.destinations(),
            Touches {
                left: true,
                right: false,
            },
        )
    }

    fn tab_bar(&self, edges: Touches) -> Element<'_> {
        // `flat_button` has a minimum height, so the icon over the label grows it.
        let tabs = Page::ALL.map(|p| {
            w::flat_button(
                column![p.icon().view(16.0), ty::caption(p.label())]
                    .spacing(2)
                    .align_x(iced::Alignment::Center)
                    .width(Length::Fill),
            )
            .on_press(Message::Page(p))
            .width(Length::Fill)
            .into()
        });
        container(row(tabs).padding(4))
            .style(self.rounded_bottom(style::container::headerbar, edges))
            .width(Length::Fill)
            .into()
    }

    /// The edge the drawer pops up from in `layout`: the right on a desktop, the bottom
    /// elsewhere, unless the Drawer edge control says otherwise.
    fn edge(&self, layout: Shell) -> Edge {
        self.drawer_edge.unwrap_or(match layout {
            Shell::Desktop => Edge::Right,
            _ => Edge::Bottom,
        })
    }

    /// Whether the right window edge is free for the window buttons and corners: not while the
    /// drawer is open from the right.
    fn right_free(&self, layout: Shell) -> bool {
        !(self.drawer_open && self.edge(layout) == Edge::Right)
    }

    /// What the drawer lists: the same `SidebarItem` as the navbar, other items.
    fn drawer_items(&self) -> Vec<w::SidebarItem<Message>> {
        Section::ALL
            .map(|sec| {
                w::SidebarItem::new(sec.label(), Message::Section(sec))
                    .selected(self.section == sec)
            })
            .into()
    }

    /// The drawer from the right edge: a full-height sidebar with its own header bar.
    fn drawer_pane(&self) -> Element<'_> {
        self.sidebar(
            "On this page",
            self.drawer_items(),
            Touches {
                left: false,
                right: true,
            },
        )
    }

    /// The drawer from the bottom: a sheet, a title over the same list. A `SidebarList` fills its
    /// height, so the sheet is given the height its rows need.
    fn drawer_sheet(&self) -> Element<'_> {
        let items = self.drawer_items();
        let rows = items.len() as f32;
        let layout = self
            .adaptive
            .shown()
            .last()
            .copied()
            .unwrap_or(Shell::Mobile);
        let (_, pad) = presentation(&layout);
        let m = adw::Metrics::default();
        let height =
            rows * pad.row_height(&m) + (rows - 1.0) * pad.gap(&m) + 2.0 * pad.inset(&m) + 8.0;
        let list = w::SidebarList::new(items).follow(&self.adaptive, presentation);
        column![
            ty::title_4("On this page"),
            container(Element::from(list)).height(Length::Fixed(height)),
        ]
        .spacing(8)
        .padding(12)
        .into()
    }

    /// `content` with the drawer for `layout`: a pane that slides in beside it from the right,
    /// or a sheet over it from the bottom.
    fn with_drawer<'a>(&'a self, layout: Shell, content: Element<'a>) -> Element<'a> {
        match self.edge(layout) {
            Edge::Right => w::adaptive_split(
                self.drawer_pane(),
                content,
                self.drawer_open,
                DRAWER_WIDTH,
                DRAWER_WIDTH,
            )
            .end(true)
            .into(),
            Edge::Bottom => w::bottom_sheet_in_window(
                content,
                self.drawer_sheet(),
                self.drawer_open,
                Message::ToggleDrawer,
                self.chrome(Touches {
                    left: true,
                    right: true,
                })
                .radius(),
            ),
        }
    }

    fn footer(&self, edges: Touches) -> Element<'_> {
        container(ty::caption(format!(
            "Footer · {} px · {:?}",
            self.width as u32,
            WidthClass::of(self.width)
        )))
        .padding([6, 12])
        .style(self.rounded_bottom(style::container::headerbar, edges))
        .width(Length::Fill)
        .into()
    }

    /// `edges`: the window edges its bottom corners touch (none when a bottom bar is below).
    fn main_area(&self, edges: Touches) -> Element<'_> {
        let pin = Trigger::Pinned;
        let layout_items = vec![
            w::ToggleItem::new(Trigger::Breakpoints, "Auto"),
            w::ToggleItem::new(pin(Shell::Mobile), "Mobile"),
            w::ToggleItem::new(pin(Shell::Tablet), "Tablet"),
            w::ToggleItem::new(pin(Shell::Desktop), "Desktop"),
        ];
        let hyst_items = vec![
            w::ToggleItem::new(Hyst::Pixels, "16 px"),
            w::ToggleItem::new(Hyst::Delay, "200 ms"),
        ];
        let log = self.log.iter().fold(column![].spacing(2), |c, l| {
            c.push(text(l.clone()).size(12).font(iced::Font::MONOSPACE))
        });
        let body = column![
            ty::title_2(self.page.label()),
            text(
                "The page is app state: it survives every layout change. Resize the window, \
                 use the header buttons, or pick a layout below."
            ),
            w::preferences_group()
                .title("Triggers")
                .description(format!(
                    "Auto follows the breakpoints. A layout can be pinned only where its range \
                     holds the width; here: {}.",
                    self.adaptive
                        .allowed_at(&Breakpoints::current(), self.width)
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
                .push(w::action_row("Layout").suffix(w::toggle_group(
                    layout_items,
                    Some(self.trigger()),
                    Message::Trigger
                )))
                .push(
                    w::action_row("Mobile sidebar")
                        .subtitle("[navigation] mobile-sidebar, on the Mobile layout")
                        .suffix(w::toggle_group(
                            vec![
                                w::ToggleItem::new(w::CollapsedSidebar::Drawer, "Drawer"),
                                w::ToggleItem::new(w::CollapsedSidebar::ParentPage, "Parent page"),
                            ],
                            Some(self.mobile_sidebar),
                            Message::MobileSidebar,
                        )),
                )
                .push(
                    w::action_row("Drawer edge")
                        .subtitle("Where the drawer pops up from")
                        .suffix(w::toggle_group(
                            vec![
                                w::ToggleItem::new(None, "Auto"),
                                w::ToggleItem::new(Some(Edge::Right), "Right"),
                                w::ToggleItem::new(Some(Edge::Bottom), "Bottom"),
                            ],
                            Some(self.drawer_edge),
                            Message::DrawerEdge,
                        )),
                )
                .push(
                    w::action_row("Hysteresis")
                        .subtitle("Resize near a threshold")
                        .suffix(w::toggle_group(
                            hyst_items,
                            Some(self.hyst),
                            Message::Hysteresis
                        )),
                ),
            w::preferences_group()
                .title("Transition")
                .push(
                    w::SwitchRow::new("Layouts answer", self.answer, Message::Answer)
                        .subtitle("Off: the coordinator times every step out"),
                )
                .push(
                    w::SliderRow::new("Close budget", self.close_ms, 0.0..=600.0, Message::CloseMs)
                        .subtitle(format!("{} ms", self.close_ms as u32)),
                )
                .push(
                    w::SliderRow::new("Open budget", self.open_ms, 0.0..=600.0, Message::OpenMs)
                        .subtitle(format!("{} ms", self.open_ms as u32)),
                ),
            w::preferences_group()
                .title("Log")
                .description(format!(
                    "Width {} px, class {:?}, phase {:?}",
                    self.width as u32,
                    WidthClass::of(self.width),
                    self.adaptive.phase()
                ))
                .push(container(log).padding(12)),
        ]
        .spacing(16)
        .padding(16)
        .max_width(720);
        // Centered in the content area, so it stays put when a pane slides in or out beside it.
        container(scrollable(container(body).center_x(Length::Fill)).width(Length::Fill))
            .style(self.rounded_bottom(style::container::view, edges))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// The content area: header, main, and an optional bottom bar. `edges`: the window edges the
    /// content's bottom corners touch.
    fn content<'a>(&'a self, header: Element<'a>, bottom: Bottom, edges: Touches) -> Element<'a> {
        let none = Touches {
            left: false,
            right: false,
        };
        match bottom {
            Bottom::None => column![header, self.main_area(edges)].into(),
            Bottom::Tabs => column![header, self.main_area(none), self.tab_bar(edges)].into(),
            Bottom::Footer => column![header, self.main_area(none), self.footer(edges)].into(),
        }
    }

    // ---- layouts ----------------------------------------------------------------------

    /// Mobile: what the sidebar becomes is `[navigation] mobile-sidebar`. The bottom navigation
    /// is the layout's, whichever it is.
    fn mobile(&self) -> Element<'_> {
        match self.mobile_sidebar {
            w::CollapsedSidebar::Drawer => self.mobile_drawer(),
            w::CollapsedSidebar::ParentPage => self.mobile_parent_page(),
        }
    }

    /// The navbar is a drawer over the content, full height; the burger opens it.
    fn mobile_drawer(&self) -> Element<'_> {
        let edges = Touches {
            left: true,
            right: self.right_free(Shell::Mobile),
        };
        let content = self.content(
            self.header(
                Some((Icon::Sidebar, Message::ToggleNavDrawer)),
                Some(Message::ToggleDrawer),
                edges,
            ),
            Bottom::Tabs,
            edges,
        );
        w::adaptive_split(
            self.navbar(),
            self.with_drawer(Shell::Mobile, content),
            false,
            NAVBAR_WIDTH,
            NAVBAR_WIDTH,
        )
        .drawer_open(self.nav_drawer_open)
        .on_close(Message::ToggleNavDrawer)
        .window_radius(self.chrome(edges).radius())
        .into()
    }

    /// The sidebar is the parent page, a page of cards (the same `SidebarItem`s); choosing one
    /// pushes the content over it, with a back button. The bottom navigation stays under both
    /// pages: it is not part of this mode.
    fn mobile_parent_page(&self) -> Element<'_> {
        let all = Touches {
            left: true,
            right: self.right_free(Shell::Mobile),
        };
        let none = Touches {
            left: false,
            right: false,
        };
        let parent = column![
            self.bar(all).title("AppShell"),
            container(Element::from(
                w::SidebarList::new(self.destinations())
                    .style(w::SidebarStyle::Cards)
                    .padding(w::SidebarPadding::Lg)
            ))
            .style(self.rounded_bottom(style::container::view, none))
            .width(Length::Fill)
            .height(Length::Fill),
        ];
        let content = self.content(
            self.header(
                Some((Icon::GoPrevious, Message::Back)),
                Some(Message::ToggleDrawer),
                all,
            ),
            Bottom::None,
            none,
        );
        column![
            w::push_stack(
                parent,
                self.with_drawer(Shell::Mobile, content),
                self.show_content
            )
            .window_radius(0.0),
            self.tab_bar(all),
        ]
        .into()
    }

    /// The navbar sits beside the content and slides away with the burger.
    fn tablet(&self) -> Element<'_> {
        let edges = Touches {
            left: !self.navbar_open,
            right: self.right_free(Shell::Tablet),
        };
        let content = self.content(
            self.header(
                Some((Icon::Sidebar, Message::ToggleNavbar)),
                Some(Message::ToggleDrawer),
                edges,
            ),
            Bottom::None,
            edges,
        );
        w::adaptive_split(
            self.navbar(),
            self.with_drawer(Shell::Tablet, content),
            self.navbar_open,
            NAVBAR_WIDTH,
            NAVBAR_WIDTH,
        )
        .into()
    }

    /// Navbar and content; the drawer pops up from the right.
    fn desktop(&self) -> Element<'_> {
        let edges = Touches {
            left: !self.navbar_open,
            right: self.right_free(Shell::Desktop),
        };
        let content = self.content(
            self.header(
                Some((Icon::Sidebar, Message::ToggleNavbar)),
                Some(Message::ToggleDrawer),
                edges,
            ),
            Bottom::Footer,
            edges,
        );
        w::adaptive_split(
            self.navbar(),
            self.with_drawer(Shell::Desktop, content),
            self.navbar_open,
            NAVBAR_WIDTH,
            NAVBAR_WIDTH,
        )
        .into()
    }

    pub fn view(&self) -> Element<'_> {
        let body = adaptive(&self.adaptive)
            .layout(Shell::Mobile, || self.mobile())
            .layout(Shell::Tablet, || self.tablet())
            .layout(Shell::Desktop, || self.desktop());
        // The frame draws the window background, rounded corners, outline and resize edges (with
        // client-side decorations), or just the background.
        self.chrome(Touches {
            left: true,
            right: true,
        })
        .frame(body, Message::Window)
    }
}

/// How a sidebar list is drawn in each layout: roomy rows for touch, dense rows where there is a
/// pointer and space. The items do not change.
fn presentation(layout: &Shell) -> (w::SidebarStyle, w::SidebarPadding) {
    match layout {
        Shell::Mobile => (w::SidebarStyle::List, w::SidebarPadding::Lg),
        Shell::Tablet => (w::SidebarStyle::List, w::SidebarPadding::Md),
        Shell::Desktop => (w::SidebarStyle::List, w::SidebarPadding::Sm),
    }
}

/// What sits under the main area.
enum Bottom {
    None,
    Tabs,
    Footer,
}
