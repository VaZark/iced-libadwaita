//! `AdwPreferencesGroup` and `AdwPreferencesPage`.

use super::status_page::CardProps;
use crate::Element;
use crate::components::{ContainerProps, TextProps};
use crate::styles::text as st;
use crate::widgets::clamp;
use crate::{Metrics, Typography};
use iced::Length;
use iced::widget::{column, space};

/// `AdwPreferencesGroup`: a titled boxed list.
pub struct PreferencesGroup<'a, M> {
    card_props: CardProps,
    container_props: Option<ContainerProps>,
    title_props: TextProps,
    description_props: TextProps,
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
    pub fn card_props(mut self, props: CardProps) -> Self {
        self.card_props = props;
        self
    }

    /// An explicit container adds a wrapper only when one is not already present.
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = Some(props);
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn title_props(mut self, props: TextProps) -> Self {
        self.title_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn description_props(mut self, props: TextProps) -> Self {
        self.description_props = props;
        self
    }

    pub fn new() -> Self {
        Self {
            card_props: CardProps::default(),
            container_props: None,
            title_props: TextProps::default(),
            description_props: TextProps::default(),

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
            head = head.push(g.title_props.apply(t.heading(title)));
            has_text = true;
        }
        if let Some(d) = g.description {
            head = head.push(g.description_props.apply(t.body(d).style(st::dim)));
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
                let mut rows = g.rows.into_iter();
                let first = rows.next().expect("nonempty rows");
                let card = rows.fold(crate::widgets::Card::new(first).padding(0), |card, row| {
                    card.push(row)
                });
                g.card_props.apply(card).into()
            });
        }
        if let Some(props) = g.container_props {
            props
                .apply(iced::widget::container(col).padding(g.padding))
                .into()
        } else if g.padding == iced::Padding::ZERO {
            col.into()
        } else {
            iced::widget::container(col).padding(g.padding).into()
        }
    }
}

/// `AdwPreferencesPage`: scrollable, clamped column of groups.
pub struct PreferencesPage<'a, M> {
    container_props: ContainerProps,
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
    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = props;
        self
    }

    pub fn new() -> Self {
        Self {
            container_props: ContainerProps::default(),

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
        let body = p.container_props.apply(
            iced::widget::container(
                column(p.groups)
                    .spacing(p.spacing.unwrap_or(m.group_spacing))
                    .width(Length::Fill),
            )
            .padding(p.padding.unwrap_or(m.page_padding)),
        );
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
        assert_eq!(
            (g.spacing, g.header_spacing, g.rows_spacing),
            (Some(1.0), Some(2.0), Some(3.0))
        );
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

/// Reusable, no-op-by-default presentation for [`PreferencesGroup`].
#[derive(Clone, Default)]
pub struct PreferencesGroupProps {
    pub card_props: Option<CardProps>,
    pub container_props: Option<ContainerProps>,
    pub title_props: Option<TextProps>,
    pub description_props: Option<TextProps>,
}
impl PreferencesGroupProps {
    pub fn card_props(mut self, props: CardProps) -> Self {
        self.card_props = Some(props);
        self
    }

    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = Some(props);
        self
    }

    pub fn title_props(mut self, props: TextProps) -> Self {
        self.title_props = Some(props);
        self
    }
    pub fn description_props(mut self, props: TextProps) -> Self {
        self.description_props = Some(props);
        self
    }
    pub fn apply<'a, M>(&self, widget: PreferencesGroup<'a, M>) -> PreferencesGroup<'a, M> {
        let mut widget = widget;
        if let Some(props) = &self.card_props {
            widget.card_props.merge(props);
        }
        crate::components::props::merge_optional!(widget, self; container_props);
        if let Some(props) = &self.title_props {
            widget.title_props.merge(props);
        }
        if let Some(props) = &self.description_props {
            widget.description_props.merge(props);
        }
        widget
    }
}

/// Reusable, no-op-by-default presentation for [`PreferencesPage`].
#[derive(Clone, Default)]
pub struct PreferencesPageProps {
    pub container_props: Option<ContainerProps>,
}
impl PreferencesPageProps {
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = Some(props);
        self
    }
    pub fn apply<'a, M>(&self, widget: PreferencesPage<'a, M>) -> PreferencesPage<'a, M> {
        let mut widget = widget;
        if let Some(props) = &self.container_props {
            widget.container_props.merge(props);
        }
        widget
    }
}

#[cfg(test)]
mod typed_presentation_tests {
    use super::*;
    use iced::advanced::widget::Tree;
    fn same_tree(a: &Tree, b: &Tree) {
        assert!(a.tag == b.tag);
        assert_eq!(a.children.len(), b.children.len());
        for (a, b) in a.children.iter().zip(&b.children) {
            same_tree(a, b);
        }
    }
    #[test]
    fn reusable_defaults_preserve_geometry_and_tree() {
        let build = || {
            PreferencesGroup::<u8>::new()
                .title("Title")
                .description("Body")
                .push(iced::widget::space())
        };
        let w = build();
        let baseline: Element<'_, u8> = w.into();
        let w = PreferencesGroupProps::default().apply(build());
        let configured: Element<'_, u8> = w.into();
        assert_eq!(baseline.as_widget().size(), configured.as_widget().size());
        same_tree(&Tree::new(&baseline), &Tree::new(&configured));
    }
}
