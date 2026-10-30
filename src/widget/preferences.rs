//! `AdwPreferencesGroup` and `AdwPreferencesPage`.

use super::{boxed_list, clamp, Element};
use crate::style::text as st;
use crate::{Metrics, Typography};
use iced::widget::{column, space};
use iced::Length;

/// `AdwPreferencesGroup`: a titled boxed list.
pub struct PreferencesGroup<'a, M> {
    title: Option<String>,
    description: Option<String>,
    header_suffix: Option<Element<'a, M>>,
    rows: Vec<Element<'a, M>>,
    metrics: Metrics,
}

pub fn preferences_group<'a, M>() -> PreferencesGroup<'a, M> {
    PreferencesGroup::new()
}

impl<'a, M> PreferencesGroup<'a, M> {
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            header_suffix: None,
            rows: Vec::new(),
            metrics: Metrics::default(),
        }
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn description(mut self, t: impl Into<String>) -> Self {
        self.description = Some(t.into());
        self
    }
    pub fn header_suffix(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.header_suffix = Some(e.into());
        self
    }
    /// `add`.
    pub fn push(mut self, row: impl Into<Element<'a, M>>) -> Self {
        self.rows.push(row.into());
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: 'a> Default for PreferencesGroup<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, M: 'a> From<PreferencesGroup<'a, M>> for Element<'a, M> {
    fn from(g: PreferencesGroup<'a, M>) -> Self {
        let t = Typography::default();
        let mut head = column![].spacing(g.metrics.space_xs);
        let mut has_text = false;
        if let Some(title) = g.title {
            head = head.push(t.heading(title));
            has_text = true;
        }
        if let Some(d) = g.description {
            head = head.push(t.body(d).style(st::dim));
            has_text = true;
        }
        let mut col = column![].spacing(g.metrics.space_m).width(Length::Fill);
        let has_head = g.header_suffix.is_some() || has_text;
        if has_head {
            let head: Element<'a, M> = head.width(Length::Fill).into();
            col = col.push(match g.header_suffix {
                Some(s) => super::FlexRow::new(head, s)
                    .spacing(g.metrics.space_m)
                    .line_spacing(g.metrics.space_s)
                    .main_basis(g.metrics.row_wrap_basis)
                    .collapse(g.metrics.row_collapse)
                    .into(),
                None => head,
            });
        }
        if !g.rows.is_empty() {
            col = col.push(boxed_list(g.rows));
        }
        col.into()
    }
}

/// `AdwPreferencesPage`: scrollable, clamped column of groups.
pub struct PreferencesPage<'a, M> {
    groups: Vec<Element<'a, M>>,
    metrics: Metrics,
}

pub fn preferences_page<'a, M>() -> PreferencesPage<'a, M> {
    PreferencesPage::new()
}

impl<'a, M> PreferencesPage<'a, M> {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            metrics: Metrics::default(),
        }
    }
    pub fn push(mut self, g: impl Into<Element<'a, M>>) -> Self {
        self.groups.push(g.into());
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: 'a> Default for PreferencesPage<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, M: 'a> From<PreferencesPage<'a, M>> for Element<'a, M> {
    fn from(p: PreferencesPage<'a, M>) -> Self {
        let m = p.metrics;
        let body = iced::widget::container(
            column(p.groups).spacing(m.group_spacing).width(Length::Fill),
        )
        .padding(m.page_padding);
        super::scrolled_window(clamp(body, m.clamp_maximum))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

#[allow(dead_code)]
fn _s() -> iced::widget::Space {
    space()
}
