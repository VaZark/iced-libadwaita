//! `AdwStatusPage`, `AdwClamp`, `AdwBin`, and `.card`.

use super::Element;
use crate::style::{container as sc, text as st};
use crate::{Metrics, Typography};
use iced::alignment::Horizontal;
use iced::widget::{column, container};
use iced::Length;

/// `AdwClamp`: limits the width of `content`, centering it.
pub fn clamp<'a, M: 'a>(content: impl Into<Element<'a, M>>, maximum: f32) -> Element<'a, M> {
    container(container(content).max_width(maximum).width(Length::Fill))
        .center_x(Length::Fill)
        .into()
}

/// `AdwBin`: a single-child container.
pub fn bin<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    container(content).into()
}

/// `.card`: padded card surface.
pub fn card<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    container(content)
        .padding(Metrics::default().card_padding)
        .width(Length::Fill)
        .style(sc::card)
        .into()
}

/// `AdwStatusPage`.
pub struct StatusPage<'a, M> {
    icon: Option<Element<'a, M>>,
    title: Option<String>,
    description: Option<String>,
    child: Option<Element<'a, M>>,
    compact: bool,
    metrics: Metrics,
}

pub fn status_page<'a, M>() -> StatusPage<'a, M> {
    StatusPage::new()
}

impl<'a, M> StatusPage<'a, M> {
    pub fn new() -> Self {
        Self {
            icon: None,
            title: None,
            description: None,
            child: None,
            compact: false,
            metrics: Metrics::default(),
        }
    }
    /// `icon-name` (any element, typically an [`Icon`](crate::Icon) view).
    pub fn icon(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.icon = Some(e.into());
        self
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn description(mut self, t: impl Into<String>) -> Self {
        self.description = Some(t.into());
        self
    }
    pub fn child(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.child = Some(e.into());
        self
    }
    /// `.compact`: smaller title, tighter spacing.
    pub fn compact(mut self, c: bool) -> Self {
        self.compact = c;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: 'a> Default for StatusPage<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, M: 'a> From<StatusPage<'a, M>> for Element<'a, M> {
    fn from(s: StatusPage<'a, M>) -> Self {
        let m = s.metrics;
        let t = Typography::default();
        let mut col = column![]
            .spacing(m.space_m)
            .align_x(Horizontal::Center)
            .width(Length::Fill);
        if let Some(i) = s.icon {
            col = col.push(i);
        }
        if let Some(title) = s.title {
            col = col.push(if s.compact { t.title_2(title) } else { t.title_1(title) });
        }
        if let Some(d) = s.description {
            col = col.push(t.body(d).style(st::dim).align_x(Horizontal::Center));
        }
        if let Some(c) = s.child {
            col = col.push(c);
        }
        container(clamp(container(col).padding(m.space_xl), 400.0))
            .center(Length::Fill)
            .into()
    }
}
