//! `Pagination`: previous / next and numbered page buttons with ellipses, after Mantine's
//! `Pagination` and Bootstrap's pagination. The numbers and the window of pages are
//! [`primitives::pagination`](crate::primitives::pagination); this widget is its look.
//!
//! Controlled: the app owns a [`primitives::pagination::Pagination`](State) and updates it
//! from `on_change(page)` (1-based).
//!
//! ```ignore
//! use iced_libadwaita::primitives::pagination::Pagination as Paging;
//! Pagination::new(&self.paging, Msg::Page)      // self.paging.goto(page) in update
//!     .with_edges(true)                         // first / last
//!     .siblings(1).boundaries(1)
//! ```
//!
//! * the active page is filled with the accent; previous / next (and first / last with
//!   [`with_edges`](Pagination::with_edges)) are disabled at the ends;
//! * every button is the same square, wide enough for the largest number shown, so the pager
//!   does not jitter as the page changes;
//! * the pager is centered in its row ([`align`](Pagination::align) to change it);
//! * [`label`] is the caption "Showing 21-40 of 187"; [`page_size_dropdown`] picks the page
//!   size. Compose them in a footer (see the gallery's data page).

use super::Element;
use crate::primitives::pagination::{PageItem, Pagination as State};
use crate::style::button as sb;
use crate::{Icon, Metrics, Theme};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, container, row, text};
use iced::{Length, Padding};
use std::rc::Rc;

/// How big the buttons are (Mantine's `size`, reduced to three).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PaginationSize {
    Small,
    #[default]
    Medium,
    Large,
}

/// See the [module](self).
pub struct Pagination<'a, M> {
    state: State,
    on_change: Rc<dyn Fn(usize) -> M + 'a>,
    siblings: usize,
    boundaries: usize,
    edges: bool,
    controls: bool,
    size: PaginationSize,
    disabled: bool,
    align: Horizontal,
    metrics: Metrics,
}

impl<'a, M: 'a> Pagination<'a, M> {
    /// A pager for `state` (copied), sending the page the user chose.
    pub fn new(state: &State, on_change: impl Fn(usize) -> M + 'a) -> Self {
        Self {
            state: *state,
            on_change: Rc::new(on_change),
            siblings: 1,
            boundaries: 1,
            edges: false,
            controls: true,
            size: PaginationSize::Medium,
            disabled: false,
            align: Horizontal::Center,
            metrics: Metrics::desktop(),
        }
    }
    /// Pages shown on each side of the active one (default 1).
    pub fn siblings(mut self, n: usize) -> Self {
        self.siblings = n;
        self
    }
    /// Pages always shown at each end (default 1).
    pub fn boundaries(mut self, n: usize) -> Self {
        self.boundaries = n;
        self
    }
    /// First and last buttons beside previous and next (Mantine's `withEdges`).
    pub fn with_edges(mut self, edges: bool) -> Self {
        self.edges = edges;
        self
    }
    /// Previous and next buttons (default true; Mantine's `withControls`).
    pub fn with_controls(mut self, controls: bool) -> Self {
        self.controls = controls;
        self
    }
    pub fn size(mut self, size: PaginationSize) -> Self {
        self.size = size;
        self
    }
    /// Nothing can be pressed (the active page keeps its fill, dimmed).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Where the pager sits in the row it fills (default centered).
    pub fn align(mut self, align: Horizontal) -> Self {
        self.align = align;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }

    /// Side of a button.
    fn side(&self) -> f32 {
        let base = self.metrics.button_min_height;
        match self.size {
            PaginationSize::Small => (base * 0.8).round(),
            PaginationSize::Medium => base,
            PaginationSize::Large => (base * 1.25).round(),
        }
    }
    fn text_size(&self) -> f32 {
        match self.size {
            PaginationSize::Small => 12.0,
            PaginationSize::Medium => 14.0,
            PaginationSize::Large => 16.0,
        }
    }
}

impl<'a, M: Clone + 'a> From<Pagination<'a, M>> for Element<'a, M> {
    fn from(p: Pagination<'a, M>) -> Self {
        let items = p.state.items(p.siblings, p.boundaries);
        // Wide enough for the largest number: all buttons share one width.
        let digits = items
            .iter()
            .filter_map(|i| match i {
                PageItem::Page(n) => Some(n.to_string().len()),
                PageItem::Ellipsis => None,
            })
            .max()
            .unwrap_or(1);
        let h = p.side();
        let w = h.max(digits as f32 * p.text_size() * 0.62 + 2.0 * p.metrics.space_m);
        let cell = |content: Element<'a, M>| {
            container(content)
                .center_x(Length::Fixed(w))
                .center_y(Length::Fixed(h))
        };
        let icon = |icon: Icon, enabled: bool| -> Element<'a, M> {
            icon.view(p.metrics.icon_size)
                .style(move |t: &Theme, _| iced::widget::svg::Style {
                    color: Some(t.semantic.icon_solid(!enabled)),
                })
                .into()
        };
        let control = |ic: Icon, target: Option<usize>| -> Element<'a, M> {
            let enabled = target.is_some() && !p.disabled;
            let b = button(cell(icon(ic, enabled)))
                .padding(Padding::ZERO)
                .style(sb::default);
            let on_change = p.on_change.clone();
            match target.filter(|_| !p.disabled) {
                Some(page) => b.on_press_with(move || on_change(page)),
                None => b,
            }
            .into()
        };

        let mut r = row![].spacing(p.metrics.space_xs).align_y(Vertical::Center);
        if p.edges {
            r = r.push(control(Icon::GoFirst, p.state.first_page()));
        }
        if p.controls {
            r = r.push(control(Icon::GoPrevious, p.state.prev_page()));
        }
        for item in items {
            r = r.push(match item {
                PageItem::Page(n) => {
                    let active = n == p.state.page();
                    let label = text(n.to_string())
                        .size(p.text_size())
                        .wrapping(text::Wrapping::None);
                    let b = button(cell(label.into()))
                        .padding(Padding::ZERO)
                        .style(if active { sb::suggested } else { sb::default });
                    let on_change = p.on_change.clone();
                    if p.disabled {
                        b
                    } else {
                        b.on_press_with(move || on_change(n))
                    }
                    .into()
                }
                PageItem::Ellipsis => Element::from(cell(
                    text("\u{2026}")
                        .size(p.text_size())
                        .style(crate::style::text::dim)
                        .into(),
                )),
            });
        }
        if p.controls {
            r = r.push(control(Icon::GoNext, p.state.next_page()));
        }
        if p.edges {
            r = r.push(control(Icon::GoLast, p.state.last_page()));
        }
        container(r).width(Length::Fill).align_x(p.align).into()
    }
}

/// "Showing 21-40 of 187": a dim caption for a footer.
pub fn label<'a, M: 'a>(state: &State) -> Element<'a, M> {
    super::typography::caption(state.label())
        .style(crate::style::text::dim)
        .into()
}

/// One entry of [`page_size_dropdown`]: displays as "20 per page".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageSize(pub usize);

impl std::fmt::Display for PageSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} per page", self.0)
    }
}

/// The page-size picker: a dropdown over `options`, sending the chosen size.
pub fn page_size_dropdown<'a, M: Clone + 'a>(
    options: &'a [PageSize],
    current: usize,
    on_select: impl Fn(usize) -> M + 'a,
) -> super::Popover<'a, M> {
    super::dropdown(options, Some(PageSize(current)), move |s| on_select(s.0))
}
