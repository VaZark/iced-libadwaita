//! `AdwNavigationSplitView`, `AdwOverlaySplitView`, `AdwNavigationView`,
//! `.navigation-sidebar`, and `AdwBreakpoint`.

use super::{header_bar, toolbar_view, Element};
use crate::style::{button as sb, container as sc};
use crate::{Icon, Metrics};
use iced::alignment::Vertical;
use iced::widget::{button, column, container, mouse_area, opaque, row, space, stack};
use iced::{Length, Size};

/// `AdwBreakpoint` condition on window width.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Breakpoint {
    max_width: Option<f32>,
    min_width: Option<f32>,
}

impl Breakpoint {
    /// `max-width: <px>`.
    pub fn max_width(px: f32) -> Self {
        Self { max_width: Some(px), min_width: None }
    }
    /// `min-width: <px>`.
    pub fn min_width(px: f32) -> Self {
        Self { max_width: None, min_width: Some(px) }
    }
    /// Whether the breakpoint applies at `size`. Typical use inside
    /// `iced::widget::responsive`.
    pub fn matches(&self, size: Size) -> bool {
        self.max_width.is_none_or(|m| size.width <= m)
            && self.min_width.is_none_or(|m| size.width >= m)
    }
}

/// A sidebar entry (`.navigation-sidebar` row).
pub fn sidebar_row<'a, M: Clone + 'a>(
    icon: Option<Icon>,
    label: impl Into<String>,
    selected: bool,
    on_press: M,
) -> Element<'a, M> {
    let m = Metrics::desktop();
    let mut r = row![].spacing(m.space_m).align_y(Vertical::Center);
    if let Some(i) = icon {
        r = r.push(i.view(m.icon_size));
    }
    // libadwaita ellipsizes; we clip at the row edge so text never paints
    // outside the selection highlight.
    r = r.push(container(super::typography::label(label.into())).width(Length::Fill).clip(true));
    // The selected background fades in and out (`Highlight`); the button keeps hover/press.
    let btn = button(
        container(r)
            .padding([0.0, m.sidebar_row_padding])
            .center_y(Length::Fixed(m.sidebar_row_height))
            .width(Length::Fill),
    )
    .padding(0)
    .width(Length::Fill)
    .style(sb::sidebar_row(false))
    .on_press(on_press);
    super::highlight(btn, selected, m.radius_m, 0.10).into()
}

/// Like [`navigation_sidebar`] but rounds the bottom corners to match a
/// rounded window (pass [`window::State::radius`](crate::window::State::radius)).
pub fn navigation_sidebar_rounded<'a, M: 'a>(
    rows: impl IntoIterator<Item = Element<'a, M>>,
    window_radius: f32,
) -> Element<'a, M> {
    let m = Metrics::desktop();
    container(column(rows).spacing(m.sidebar_spacing).padding(m.sidebar_padding))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_: &crate::Theme| iced::widget::container::Style {
            // Transparent: the split view's pane supplies the color (sidebar or, as a first
            // page, content). Both bottom corners: beside content the right one reveals the pane behind
            // (same color); when the sidebar is the whole window it is a real window corner.
            border: iced::Border::default().rounded(iced::border::bottom(window_radius)),
            ..Default::default()
        })
        .into()
}

/// Stacks sidebar rows with `.navigation-sidebar` spacing/padding.
pub fn navigation_sidebar<'a, M: 'a>(
    rows: impl IntoIterator<Item = Element<'a, M>>,
) -> Element<'a, M> {
    let m = Metrics::desktop();
    container(column(rows).spacing(m.sidebar_spacing).padding(m.sidebar_padding))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(sc::sidebar)
        .into()
}

/// One entry of a [`SidebarList`].
pub struct SidebarItem<M> {
    icon: Option<Icon>,
    title: String,
    subtitle: Option<String>,
    selected: bool,
    on_press: M,
}

impl<M> SidebarItem<M> {
    pub fn new(title: impl Into<String>, on_press: M) -> Self {
        Self { icon: None, title: title.into(), subtitle: None, selected: false, on_press }
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    /// A short description. Shown only in [`SidebarStyle::Cards`].
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.subtitle = Some(s.into());
        self
    }
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
}

/// How a [`SidebarList`] presents its items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarStyle {
    /// `.navigation-sidebar`: compact selectable rows on the sidebar color, for a sidebar
    /// beside the content or in a drawer. Subtitles are hidden.
    #[default]
    List,
    /// A page of cards: a padded, scrolling boxed list on the content color, each row with
    /// its icon, title, **subtitle** and a chevron, as in a mobile settings app. Use it for
    /// the sidebar as the first page ([`CollapsedSidebar::FirstPage`]) when collapsed.
    Cards,
}

/// The navigation entries of a sidebar, in either [`SidebarStyle`]. One component, so an
/// app describes its destinations once and the presentation follows the layout.
pub struct SidebarList<'a, M> {
    items: Vec<SidebarItem<M>>,
    style: SidebarStyle,
    window_radius: f32,
    _p: std::marker::PhantomData<&'a ()>,
}

impl<'a, M: Clone + 'a> SidebarList<'a, M> {
    pub fn new(items: impl IntoIterator<Item = SidebarItem<M>>) -> Self {
        Self { items: items.into_iter().collect(), style: SidebarStyle::List, window_radius: 0.0, _p: Default::default() }
    }
    pub fn style(mut self, style: SidebarStyle) -> Self {
        self.style = style;
        self
    }
    /// Window radius for the [`SidebarStyle::List`] bottom corners.
    pub fn window_radius(mut self, r: f32) -> Self {
        self.window_radius = r;
        self
    }
}

impl<'a, M: Clone + 'a> From<SidebarList<'a, M>> for Element<'a, M> {
    fn from(l: SidebarList<'a, M>) -> Self {
        match l.style {
            SidebarStyle::List => navigation_sidebar_rounded(
                l.items.into_iter().map(|i| sidebar_row(i.icon, i.title, i.selected, i.on_press)),
                l.window_radius,
            ),
            SidebarStyle::Cards => {
                let m = Metrics::desktop();
                let rows = l.items.into_iter().map(|i| {
                    let mut r = super::ActionRow::new(i.title).chevron(true).on_activate(i.on_press).metrics(m);
                    if let Some(icon) = i.icon {
                        r = r.prefix(icon.view(m.icon_size));
                    }
                    if let Some(s) = i.subtitle {
                        r = r.subtitle(s);
                    }
                    Element::from(r)
                });
                super::page(super::boxed_list(rows)).metrics(m).into()
            }
        }
    }
}

/// What the sidebar becomes when a [`NavigationSplitView`] collapses (narrow / mobile).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CollapsedSidebar {
    /// The sidebar slides over the content from the start edge with a scrim; the
    /// content stays visible behind it and the drawer never takes the whole screen
    /// (`AdwOverlaySplitView`'s collapsed overlay). Opened/closed by the app
    /// ([`NavigationSplitView::show_sidebar`]). (default)
    #[default]
    Drawer,
    /// The sidebar is the *first page* of a navigation stack, as in a settings app: it
    /// is the landing page, painted in the **content** color (not the sidebar color),
    /// and choosing an item pushes the content page (`show_content`) with a back button.
    /// Beside the content the sidebar keeps its own color.
    FirstPage,
}

impl CollapsedSidebar {
    pub const ALL: [Self; 2] = [Self::Drawer, Self::FirstPage];
}

impl std::fmt::Display for CollapsedSidebar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Drawer => "Drawer",
            Self::FirstPage => "First page",
        })
    }
}

/// `AdwNavigationSplitView`. Collapsed, it shows either a drawer over the content
/// or one pane at a time; see [`CollapsedSidebar`].
pub struct NavigationSplitView<'a, M> {
    sidebar: Element<'a, M>,
    content: Element<'a, M>,
    sidebar_width: f32,
    fraction: f32,
    min_width: f32,
    max_width: f32,
    collapsed: bool,
    show_content: bool,
    window_radius: f32,
    collapsed_sidebar: CollapsedSidebar,
    drawer_open: bool,
    on_close_drawer: Option<M>,
}

/// libadwaita's width for a split-view sidebar: `fraction` of the total width
/// clamped to `[min, max]` (defaults 25%, 180sp, 280sp).
pub fn sidebar_width_for(total: f32, fraction: f32, min: f32, max: f32) -> f32 {
    (total * fraction).clamp(min, max)
}

/// The default `max-width` below which split views collapse (docs: `400sp`).
pub const COLLAPSE_WIDTH: f32 = 400.0;

/// The collapse width in effect: the desktop config's `[navigation] collapse-width`.
pub fn collapse_width() -> f32 {
    crate::config::current().collapse_width
}

impl<'a, M: 'a> NavigationSplitView<'a, M> {
    pub fn new(sidebar: impl Into<Element<'a, M>>, content: impl Into<Element<'a, M>>) -> Self {
        Self {
            sidebar: sidebar.into(),
            content: content.into(),
            sidebar_width: 240.0,
            fraction: 0.25,
            min_width: 180.0,
            max_width: 280.0,
            collapsed: false,
            show_content: true,
            window_radius: 0.0,
            // The desktop config decides (default: drawer); `.collapsed_sidebar(..)` is an opt-out.
            collapsed_sidebar: crate::config::current().mobile_sidebar,
            drawer_open: false,
            on_close_drawer: None,
        }
    }
    /// What the sidebar becomes when collapsed: [`CollapsedSidebar::Drawer`]
    /// (default) or [`CollapsedSidebar::FirstPage`].
    pub fn collapsed_sidebar(mut self, mode: CollapsedSidebar) -> Self {
        self.collapsed_sidebar = mode;
        self
    }
    /// Whether the drawer is open (only used with [`CollapsedSidebar::Drawer`]).
    pub fn show_sidebar(mut self, open: bool) -> Self {
        self.drawer_open = open;
        self
    }
    /// Message sent when the scrim behind the open drawer is clicked.
    pub fn on_close_drawer(mut self, msg: M) -> Self {
        self.on_close_drawer = Some(msg);
        self
    }
    /// Rounds the sidebar pane's outer corners to match a rounded window
    /// ([`window::State::radius`](crate::window::State::radius)). The pane paints
    /// the sidebar color, so without this it would cover the window's corners.
    pub fn window_radius(mut self, r: f32) -> Self {
        self.window_radius = r;
        self
    }
    /// Fixed sidebar width (overrides the fraction rules).
    pub fn sidebar_width(mut self, w: f32) -> Self {
        self.sidebar_width = w;
        self
    }
    pub fn sidebar_width_fraction(mut self, f: f32) -> Self {
        self.fraction = f;
        self
    }
    pub fn min_sidebar_width(mut self, w: f32) -> Self {
        self.min_width = w;
        self
    }
    pub fn max_sidebar_width(mut self, w: f32) -> Self {
        self.max_width = w;
        self
    }
    pub fn collapsed(mut self, c: bool) -> Self {
        self.collapsed = c;
        self
    }
    pub fn show_content(mut self, s: bool) -> Self {
        self.show_content = s;
        self
    }
    /// Applies libadwaita's adaptive rules for a window of `width`: collapse at
    /// `max-width: 400sp` and size the sidebar by fraction/min/max.
    pub fn adapt(mut self, width: f32) -> Self {
        self.collapsed = width <= collapse_width();
        self.sidebar_width = if self.collapsed && self.collapsed_sidebar == CollapsedSidebar::Drawer {
            // A drawer uses the maximum sidebar width when it fits, but always leaves a
            // strip of content (and the scrim) visible: never the whole screen.
            self.max_width.min(width * 0.85)
        } else {
            sidebar_width_for(width, self.fraction, self.min_width, self.max_width)
        };
        self
    }
}

impl<'a, M: Clone + 'a> From<NavigationSplitView<'a, M>> for Element<'a, M> {
    fn from(v: NavigationSplitView<'a, M>) -> Self {
        let r = v.window_radius;
        // The sidebar pane carries the sidebar color (`.sidebar-pane`), so a flat header
        // above the list continues it. It sits at the window edge, so it rounds the
        // window's corners: left only when beside the content or as a drawer, all four
        // when it is the whole (collapsed, first-page) window.
        let pane = move |radius: iced::border::Radius| {
            move |t: &crate::Theme| iced::widget::container::Style {
                border: iced::Border::default().rounded(radius),
                ..sc::sidebar(t)
            }
        };
        let left = iced::border::Radius { top_left: r, top_right: 0.0, bottom_right: 0.0, bottom_left: r };
        if v.collapsed {
            if v.collapsed_sidebar != CollapsedSidebar::Drawer {
                // The sidebar is the first page; the content is pushed over it (and popped
                // back) with the navigation slide.
                let first: Element<'a, M> = container(v.sidebar).width(Length::Fill).height(Length::Fill).into();
                return super::push_stack(first, v.content, v.show_content).window_radius(r).into();
            }
            // Drawer: the content is always there; the sidebar slides over it. The layer
            // is always mounted so it animates open *and* closed (spring, scrim fade).
            let content: Element<'a, M> = container(v.content).width(Length::Fill).height(Length::Fill).into();
            let drawer = container(v.sidebar)
                .width(Length::Fixed(v.sidebar_width))
                .height(Length::Fill)
                .style(move |t: &crate::Theme| {
                    let shadow = t.shadows.dialog().shadow;
                    iced::widget::container::Style {
                        border: iced::Border::default().rounded(left),
                        shadow,
                        ..sc::sidebar(t)
                    }
                });
            let mut layer = super::drawer_layer(drawer, v.drawer_open, v.sidebar_width).window_radius(r);
            if let Some(m) = v.on_close_drawer {
                layer = layer.on_close(m);
            }
            return stack![content, Element::from(layer)].into();
        }
        row![
            container(v.sidebar)
                .width(Length::Fixed(v.sidebar_width))
                .height(Length::Fill)
                .style(pane(left)),
            container(space().width(1)).height(Length::Fill).style(sc::sidebar_separator),
            container(v.content).width(Length::Fill).height(Length::Fill),
        ]
        .into()
    }
}

/// `AdwOverlaySplitView`. When `collapsed`, the sidebar floats over the
/// content with a scrim (click it to send `on_close`).
pub struct OverlaySplitView<'a, M> {
    sidebar: Element<'a, M>,
    content: Element<'a, M>,
    sidebar_width: f32,
    collapsed: bool,
    show_sidebar: bool,
    on_close: Option<M>,
}

impl<'a, M: 'a> OverlaySplitView<'a, M> {
    pub fn new(sidebar: impl Into<Element<'a, M>>, content: impl Into<Element<'a, M>>) -> Self {
        Self {
            sidebar: sidebar.into(),
            content: content.into(),
            sidebar_width: Metrics::desktop().sidebar_width,
            collapsed: false,
            show_sidebar: true,
            on_close: None,
        }
    }
    pub fn sidebar_width(mut self, w: f32) -> Self {
        self.sidebar_width = w;
        self
    }
    pub fn collapsed(mut self, c: bool) -> Self {
        self.collapsed = c;
        self
    }
    pub fn show_sidebar(mut self, s: bool) -> Self {
        self.show_sidebar = s;
        self
    }
    pub fn on_close(mut self, m: M) -> Self {
        self.on_close = Some(m);
        self
    }
}

impl<'a, M: Clone + 'a> From<OverlaySplitView<'a, M>> for Element<'a, M> {
    fn from(v: OverlaySplitView<'a, M>) -> Self {
        if !v.show_sidebar {
            return v.content;
        }
        let side = container(v.sidebar).width(Length::Fixed(v.sidebar_width)).height(Length::Fill);
        if !v.collapsed {
            return row![side, container(space().width(1)).height(Length::Fill).style(sc::sidebar_separator), v.content].into();
        }
        let mut scrim = mouse_area(container(space()).width(Length::Fill).height(Length::Fill).style(sc::backdrop));
        if let Some(m) = v.on_close {
            scrim = scrim.on_press(m);
        }
        stack![v.content, opaque(scrim), opaque(side)].into()
    }
}

/// One page of a navigation stack.
pub struct NavigationPage<'a, M> {
    pub title: String,
    pub content: Element<'a, M>,
}

impl<'a, M> NavigationPage<'a, M> {
    pub fn new(title: impl Into<String>, content: impl Into<Element<'a, M>>) -> Self {
        Self { title: title.into(), content: content.into() }
    }
}

/// `AdwNavigationView`: shows the top page with a header bar and a back
/// button (sending `on_pop`) when the stack has more than one page.
pub fn navigation_view<'a, M: Clone + 'a>(
    mut pages: Vec<NavigationPage<'a, M>>,
    on_pop: M,
) -> Element<'a, M> {
    let depth = pages.len();
    let Some(top) = pages.pop() else {
        return space().into();
    };
    let mut bar = header_bar().title(top.title);
    if depth > 1 {
        bar = bar.pack_start(super::circular_button(Icon::GoPrevious).on_press(on_pop));
    }
    toolbar_view(top.content).top_bar(bar).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sidebar_width_follows_docs_rules() {
        // 25% of the width, clamped to 180..280.
        assert_eq!(sidebar_width_for(600.0, 0.25, 180.0, 280.0), 180.0);
        assert_eq!(sidebar_width_for(1000.0, 0.25, 180.0, 280.0), 250.0);
        assert_eq!(sidebar_width_for(2000.0, 0.25, 180.0, 280.0), 280.0);
    }

    fn split(mode: CollapsedSidebar) -> NavigationSplitView<'static, ()> {
        NavigationSplitView::new(space(), space()).collapsed_sidebar(mode)
    }

    #[test]
    fn drawer_is_the_default_collapsed_mode() {
        assert_eq!(CollapsedSidebar::default(), CollapsedSidebar::Drawer);
        assert_eq!(split(CollapsedSidebar::Drawer).collapsed_sidebar, CollapsedSidebar::Drawer);
    }

    #[test]
    fn drawer_never_takes_the_whole_screen() {
        // At 380px the drawer uses the max sidebar width; at 300px it is capped to 85%.
        let d = split(CollapsedSidebar::Drawer).adapt(380.0);
        assert!(d.collapsed && (d.sidebar_width - 280.0).abs() < 1e-3);
        let d = split(CollapsedSidebar::Drawer).adapt(300.0);
        assert!((d.sidebar_width - 255.0).abs() < 1e-3 && d.sidebar_width < 300.0);
        // Not collapsed: the usual fraction/min/max rule.
        let wide = split(CollapsedSidebar::Drawer).adapt(1000.0);
        assert!(!wide.collapsed && (wide.sidebar_width - 250.0).abs() < 1e-3);
    }

    #[test]
    fn full_page_uses_the_regular_width_rule() {
        let f = split(CollapsedSidebar::FirstPage).adapt(300.0);
        assert!(f.collapsed && (f.sidebar_width - 180.0).abs() < 1e-3);
    }

    #[test]
    fn breakpoint_conditions() {
        let bp = Breakpoint::max_width(400.0);
        assert!(bp.matches(Size::new(360.0, 600.0)));
        assert!(bp.matches(Size::new(400.0, 600.0)));
        assert!(!bp.matches(Size::new(401.0, 600.0)));
        assert!(Breakpoint::min_width(500.0).matches(Size::new(500.0, 1.0)));
    }
}
