//! `AdwPreferencesGroup` and `AdwPreferencesPage`.

use crate::Element;
use crate::styles::text as st;
use crate::widgets::{boxed_list, clamp};
use crate::{Metrics, Typography};
use iced::Length;
use iced::widget::{column, space};

/// `AdwPreferencesGroup`: a titled boxed list.
pub struct PreferencesGroup<'a, M> {
    title: Option<String>,
    description: Option<String>,
    header_suffix: Option<Element<'a, M>>,
    rows: Vec<Element<'a, M>>,
    spacing: Option<f32>,
    header_spacing: Option<f32>,
    rows_spacing: Option<f32>,
    padding: iced::Padding,
    metrics: Metrics,
    flat: bool,
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
            spacing: None,
            header_spacing: None,
            rows_spacing: None,
            padding: iced::Padding::ZERO,
            metrics: Metrics::desktop(),
            flat: false,
        }
    }
    /// Show rows directly on the page, without the boxed surface or separators.
    pub fn flat(mut self, flat: bool) -> Self {
        self.flat = flat;
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
    pub fn header_suffix(mut self, e: impl Into<Element<'a, M>>) -> Self {
        self.header_suffix = Some(e.into());
        self
    }
    /// `add`.
    pub fn push(mut self, row: impl Into<Element<'a, M>>) -> Self {
        self.rows.push(row.into());
        self
    }
    /// The gap between the header and the rows (default: the theme's `space_m`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px.max(0.0));
        self
    }
    /// The gap between the title and the description (default: the theme's `space_xs`).
    pub fn header_spacing(mut self, px: f32) -> Self {
        self.header_spacing = Some(px.max(0.0));
        self
    }
    /// For a [`flat`](Self::flat) group: the gap between the rows (default: the theme's
    /// `space_xs`).
    pub fn rows_spacing(mut self, px: f32) -> Self {
        self.rows_spacing = Some(px.max(0.0));
        self
    }
    /// The padding around the whole group (default none).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = p.into();
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
        let t = Typography::desktop();
        let mut head = column![].spacing(g.header_spacing.unwrap_or(g.metrics.space_xs));
        let mut has_text = false;
        if let Some(title) = g.title {
            head = head.push(t.heading(title));
            has_text = true;
        }
        if let Some(d) = g.description {
            head = head.push(t.body(d).style(st::dim));
            has_text = true;
        }
        let mut col = column![]
            .spacing(g.spacing.unwrap_or(g.metrics.space_m))
            .width(Length::Fill);
        let has_head = g.header_suffix.is_some() || has_text;
        if has_head {
            let head: Element<'a, M> = head.width(Length::Fill).into();
            col = col.push(match g.header_suffix {
                Some(s) => crate::layouts::FlexRow::new(head, s)
                    .spacing(g.metrics.space_m)
                    .line_spacing(g.metrics.space_s)
                    .main_basis(g.metrics.row_wrap_basis)
                    .collapse(g.metrics.row_collapse)
                    .into(),
                None => head,
            });
        }
        if !g.rows.is_empty() {
            col = col.push(if g.flat {
                Element::from(
                    column(g.rows)
                        .spacing(g.rows_spacing.unwrap_or(g.metrics.space_xs))
                        .width(Length::Fill),
                )
            } else {
                boxed_list(g.rows)
            });
        }
        if g.padding == iced::Padding::ZERO {
            col.into()
        } else {
            iced::widget::container(col).padding(g.padding).into()
        }
    }
}

/// `AdwPreferencesPage`: scrollable, clamped column of groups.
pub struct PreferencesPage<'a, M> {
    groups: Vec<Element<'a, M>>,
    spacing: Option<f32>,
    padding: Option<iced::Padding>,
    max_width: Option<f32>,
    metrics: Metrics,
}

pub fn preferences_page<'a, M>() -> PreferencesPage<'a, M> {
    PreferencesPage::new()
}

impl<'a, M> PreferencesPage<'a, M> {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            spacing: None,
            padding: None,
            max_width: None,
            metrics: Metrics::desktop(),
        }
    }
    pub fn push(mut self, g: impl Into<Element<'a, M>>) -> Self {
        self.groups.push(g.into());
        self
    }
    /// The gap between groups (default: the theme's `group_spacing`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px.max(0.0));
        self
    }
    /// The padding around the groups (default: the theme's `page_padding`).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }
    /// The widest the content gets (default: the theme's `clamp_maximum`).
    pub fn max_width(mut self, px: f32) -> Self {
        self.max_width = Some(px.max(0.0));
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
            column(p.groups)
                .spacing(p.spacing.unwrap_or(m.group_spacing))
                .width(Length::Fill),
        )
        .padding(p.padding.unwrap_or(m.page_padding));
        crate::layouts::scrolled_window(clamp(body, p.max_width.unwrap_or(m.clamp_maximum)))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

#[allow(dead_code)]
fn _s() -> iced::widget::Space {
    space()
}

#[cfg(test)]
mod spacing_params_tests {
    use super::*;

    #[test]
    fn a_group_and_a_page_take_spacing_and_padding() {
        let g = PreferencesGroup::<u8>::new()
            .title("T")
            .description("D")
            .push(iced::widget::text("row"))
            .spacing(1.0)
            .header_spacing(2.0)
            .rows_spacing(3.0)
            .padding(4.0)
            .flat(true);
        assert_eq!((g.spacing, g.header_spacing, g.rows_spacing), (Some(1.0), Some(2.0), Some(3.0)));
        assert_eq!(g.padding, iced::Padding::from(4.0));
        let el: Element<'_, u8> = g.into();
        let p = PreferencesPage::<u8>::new()
            .push(el)
            .spacing(5.0)
            .padding([1.0, 2.0])
            .max_width(500.0);
        assert_eq!((p.spacing, p.max_width), (Some(5.0), Some(500.0)));
        let _: Element<'_, u8> = p.into();
    }
}
