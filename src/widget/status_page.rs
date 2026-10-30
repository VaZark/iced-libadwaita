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
    Card::new(content).into()
}

/// `.card` with properties: padding, width and height.
pub struct Card<'a, M> {
    content: Element<'a, M>,
    padding: iced::Padding,
    width: Length,
    height: Length,
}

impl<'a, M: 'a> Card<'a, M> {
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            content: content.into(),
            padding: Metrics::desktop().card_padding,
            width: Length::Fill,
            height: Length::Shrink,
        }
    }
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = p.into();
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }
}

impl<'a, M: 'a> From<Card<'a, M>> for Element<'a, M> {
    fn from(c: Card<'a, M>) -> Self {
        container(c.content)
            .padding(c.padding)
            .width(c.width)
            .height(c.height)
            .style(sc::card)
            .into()
    }
}

/// A scrolled, clamped page: the body of a window pane. Content is limited to
/// `maximum` wide (default 720) with the standard page padding, in an Adwaita
/// scrolled window. (`AdwPreferencesPage` is the same thing specialised for groups.)
pub struct Page<'a, M> {
    content: Element<'a, M>,
    maximum: f32,
    metrics: Metrics,
}

pub fn page<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Page<'a, M> {
    Page::new(content)
}

impl<'a, M: 'a> Page<'a, M> {
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self { content: content.into(), maximum: 720.0, metrics: Metrics::desktop() }
    }
    /// `AdwClamp:maximum-size`.
    pub fn maximum_size(mut self, px: f32) -> Self {
        self.maximum = px;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: 'a> From<Page<'a, M>> for Element<'a, M> {
    fn from(p: Page<'a, M>) -> Self {
        super::scrolled_window(clamp(
            container(p.content).padding(p.metrics.page_padding).width(Length::Fill),
            p.maximum,
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }
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
            metrics: Metrics::desktop(),
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
