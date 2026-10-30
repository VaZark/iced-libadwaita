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
    let m = Metrics::default();
    let mut r = row![].spacing(m.space_m).align_y(Vertical::Center);
    if let Some(i) = icon {
        r = r.push(i.view(m.icon_size));
    }
    // libadwaita ellipsizes; we clip at the row edge so text never paints
    // outside the selection highlight.
    r = r.push(container(super::typography::label(label.into())).width(Length::Fill).clip(true));
    button(
        container(r)
            .padding([0.0, m.sidebar_row_padding])
            .center_y(Length::Fixed(m.sidebar_row_height))
            .width(Length::Fill),
    )
    .padding(0)
    .width(Length::Fill)
    .style(sb::sidebar_row(selected))
    .on_press(on_press)
    .into()
}

/// Like [`navigation_sidebar`] but rounds the bottom-left corner to match a
/// rounded window (pass [`window::State::radius`](crate::window::State::radius)).
pub fn navigation_sidebar_rounded<'a, M: 'a>(
    rows: impl IntoIterator<Item = Element<'a, M>>,
    window_radius: f32,
) -> Element<'a, M> {
    let m = Metrics::default();
    container(column(rows).spacing(m.sidebar_spacing).padding(m.sidebar_padding))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |t: &crate::Theme| iced::widget::container::Style {
            border: iced::Border::default().rounded(iced::border::bottom_left(window_radius)),
            ..sc::sidebar(t)
        })
        .into()
}

/// Stacks sidebar rows with `.navigation-sidebar` spacing/padding.
pub fn navigation_sidebar<'a, M: 'a>(
    rows: impl IntoIterator<Item = Element<'a, M>>,
) -> Element<'a, M> {
    let m = Metrics::default();
    container(column(rows).spacing(m.sidebar_spacing).padding(m.sidebar_padding))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(sc::sidebar)
        .into()
}

/// `AdwNavigationSplitView`. When `collapsed` only one pane is shown,
/// selected by `show_content`.
pub struct NavigationSplitView<'a, M> {
    sidebar: Element<'a, M>,
    content: Element<'a, M>,
    sidebar_width: f32,
    fraction: f32,
    min_width: f32,
    max_width: f32,
    collapsed: bool,
    show_content: bool,
}

/// libadwaita's width for a split-view sidebar: `fraction` of the total width
/// clamped to `[min, max]` (defaults 25%, 180sp, 280sp).
pub fn sidebar_width_for(total: f32, fraction: f32, min: f32, max: f32) -> f32 {
    (total * fraction).clamp(min, max)
}

/// The `max-width` below which split views collapse (docs: `400sp`).
pub const COLLAPSE_WIDTH: f32 = 400.0;

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
        }
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
        self.collapsed = width <= COLLAPSE_WIDTH;
        self.sidebar_width = sidebar_width_for(width, self.fraction, self.min_width, self.max_width);
        self
    }
}

impl<'a, M: 'a> From<NavigationSplitView<'a, M>> for Element<'a, M> {
    fn from(v: NavigationSplitView<'a, M>) -> Self {
        if v.collapsed {
            return if v.show_content { v.content } else { v.sidebar };
        }
        row![
            // The sidebar pane carries the sidebar color (`.sidebar-pane`), so a flat
            // header above the list continues it.
            container(v.sidebar)
                .width(Length::Fixed(v.sidebar_width))
                .height(Length::Fill)
                .style(sc::sidebar),
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
            sidebar_width: Metrics::default().sidebar_width,
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

    #[test]
    fn breakpoint_conditions() {
        let bp = Breakpoint::max_width(400.0);
        assert!(bp.matches(Size::new(360.0, 600.0)));
        assert!(bp.matches(Size::new(400.0, 600.0)));
        assert!(!bp.matches(Size::new(401.0, 600.0)));
        assert!(Breakpoint::min_width(500.0).matches(Size::new(500.0, 1.0)));
    }
}
