//! `AdwToolbarView` and `GtkActionBar`.

use super::Element;
use crate::style::container as sc;
use crate::Metrics;
use iced::alignment::Vertical;
use iced::widget::{column, container, row, space};
use iced::Length;

fn sep<'a, M: 'a>() -> Element<'a, M> {
    container(space().height(1))
        .width(Length::Fill)
        .style(sc::separator)
        .into()
}

/// `AdwToolbarView`: top bars, content, bottom bars, with hairlines between.
pub struct ToolbarView<'a, M> {
    top: Vec<Element<'a, M>>,
    bottom: Vec<Element<'a, M>>,
    content: Element<'a, M>,
    separators: bool,
}

pub fn toolbar_view<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> ToolbarView<'a, M> {
    ToolbarView::new(content)
}

impl<'a, M: 'a> ToolbarView<'a, M> {
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            top: Vec::new(),
            bottom: Vec::new(),
            content: content.into(),
            separators: true,
        }
    }
    /// `add_top_bar`.
    pub fn top_bar(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.top.push(e.into());
        self
    }
    /// `add_bottom_bar`.
    pub fn bottom_bar(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.bottom.push(e.into());
        self
    }
    /// `top-bar-style` raised (true, default) or flat (false).
    pub fn separators(mut self, s: bool) -> Self {
        self.separators = s;
        self
    }
}

impl<'a, M: 'a> From<ToolbarView<'a, M>> for Element<'a, M> {
    fn from(t: ToolbarView<'a, M>) -> Self {
        let mut col = column![];
        for b in t.top {
            col = col.push(b);
        }
        if t.separators {
            col = col.push(sep());
        }
        col = col.push(container(t.content).width(Length::Fill).height(Length::Fill));
        if t.separators && !t.bottom.is_empty() {
            col = col.push(sep());
        }
        for b in t.bottom {
            col = col.push(b);
        }
        col.width(Length::Fill).height(Length::Fill).into()
    }
}

/// `GtkActionBar`.
pub struct ActionBar<'a, M> {
    start: Vec<Element<'a, M>>,
    center: Option<Element<'a, M>>,
    end: Vec<Element<'a, M>>,
    metrics: Metrics,
}

pub fn action_bar<'a, M>() -> ActionBar<'a, M> {
    ActionBar::new()
}

impl<'a, M> ActionBar<'a, M> {
    pub fn new() -> Self {
        Self {
            start: Vec::new(),
            center: None,
            end: Vec::new(),
            metrics: Metrics::default(),
        }
    }
    pub fn pack_start(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.start.push(e.into());
        self
    }
    pub fn pack_end(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.end.insert(0, e.into());
        self
    }
    pub fn center(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.center = Some(e.into());
        self
    }
}

impl<'a, M: 'a> Default for ActionBar<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, M: 'a> From<ActionBar<'a, M>> for Element<'a, M> {
    fn from(a: ActionBar<'a, M>) -> Self {
        let m = a.metrics;
        let side = |v: Vec<Element<'a, M>>, h| {
            container(row(v).spacing(m.space_s).align_y(Vertical::Center))
                .width(Length::FillPortion(1))
                .align_x(h)
        };
        container(
            row![
                side(a.start, iced::alignment::Horizontal::Left),
                a.center.unwrap_or_else(|| space().into()),
                side(a.end, iced::alignment::Horizontal::Right),
            ]
            .align_y(Vertical::Center)
            .spacing(m.space_m),
        )
        .padding(m.space_s)
        .width(Length::Fill)
        .style(sc::headerbar)
        .into()
    }
}
