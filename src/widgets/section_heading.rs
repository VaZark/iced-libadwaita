//! `SectionHeading`: a heading over a section with an optional link at the far end, "View all ›".
//! Home pages, explore pages, any list that is a preview of a longer one.
//!
//! ```ignore
//! SectionHeading::new("Recently played").link("View all", Msg::OpenHistory)
//! ```

use crate::Element;
use crate::components::{ButtonProps, ContainerProps, TextProps};
use crate::components::{flat_button, typography as ty};
use crate::{Icon, Metrics};
use iced::widget::{row, space};
use iced::{Alignment, Length};

/// See the [module](self).
pub struct SectionHeading<M> {
    container_props: Option<ContainerProps>,
    title_props: TextProps,
    subtitle_props: TextProps,
    action_label_props: TextProps,
    action_props: ButtonProps,
    title: String,
    subtitle: Option<String>,
    link: Option<(String, M)>,
    metrics: Metrics,
    spacing: Option<f32>,
    title_spacing: Option<f32>,
    link_spacing: Option<f32>,
    padding: Option<iced::Padding>,
}

impl<M> SectionHeading<M> {
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
    pub fn subtitle_props(mut self, props: TextProps) -> Self {
        self.subtitle_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn action_label_props(mut self, props: TextProps) -> Self {
        self.action_label_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn action_props(mut self, props: ButtonProps) -> Self {
        self.action_props = props;
        self
    }

    pub fn new(title: impl Into<String>) -> Self {
        Self {
            container_props: None,
            title_props: TextProps::default(),
            subtitle_props: TextProps::default(),
            action_label_props: TextProps::default(),
            action_props: ButtonProps::default(),

            title: title.into(),
            subtitle: None,
            link: None,
            metrics: Metrics::desktop(),
            spacing: None,
            title_spacing: None,
            link_spacing: None,
            padding: None,
        }
    }
    /// A dimmed line under the title.
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }
    /// The link at the far end: its label and what pressing it sends. A chevron follows the label.
    pub fn link(mut self, label: impl Into<String>, on_press: M) -> Self {
        self.link = Some((label.into(), on_press));
        self
    }
    /// The gap between the title and the link (default: the theme's `space_m`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
        self
    }
    /// The gap between the title and its subtitle (default: the theme's `space_xs`).
    pub fn title_spacing(mut self, px: f32) -> Self {
        self.title_spacing = Some(px);
        self
    }
    /// The gap between the link's label and its chevron (default: the theme's `space_xs`).
    pub fn link_spacing(mut self, px: f32) -> Self {
        self.link_spacing = Some(px);
        self
    }
    /// Padding around the heading (default: none).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: Clone + 'a> From<SectionHeading<M>> for Element<'a, M> {
    fn from(h: SectionHeading<M>) -> Self {
        let m = h.metrics;
        let mut title = iced::widget::column![h.title_props.apply(ty::heading(h.title))]
            .spacing(h.title_spacing.unwrap_or(m.space_xs));
        if let Some(s) = h.subtitle {
            title = title.push(
                h.subtitle_props
                    .apply(ty::caption(s).style(crate::styles::text::dim)),
            );
        }
        let mut r = row![title.width(Length::Fill)]
            .spacing(h.spacing.unwrap_or(m.space_m))
            .align_y(Alignment::Center);
        match h.link {
            Some((label, msg)) => {
                r = r.push(
                    h.action_props.apply(
                        flat_button(
                            row![
                                h.action_label_props.apply(ty::body(label)),
                                Icon::GoNext.view(m.icon_size - 2.0)
                            ]
                            .spacing(h.link_spacing.unwrap_or(m.space_xs))
                            .align_y(Alignment::Center),
                        )
                        .on_press(msg),
                    ),
                );
            }
            None => r = r.push(space().width(0)),
        }
        match (h.padding, h.container_props) {
            (padding, Some(props)) => props
                .apply(iced::widget::container(r).padding(padding.unwrap_or(iced::Padding::ZERO)))
                .into(),
            (Some(p), None) => iced::widget::container(r).padding(p).into(),
            (None, None) => r.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_parts_take_their_own_spacing_and_padding() {
        let h = SectionHeading::new("T")
            .subtitle("s")
            .link("All", 1)
            .spacing(2.0)
            .title_spacing(3.0)
            .link_spacing(4.0)
            .padding(5.0);
        assert_eq!(
            (h.spacing, h.title_spacing, h.link_spacing),
            (Some(2.0), Some(3.0), Some(4.0))
        );
        let _: Element<'_, u8> = h.into();
    }

    #[test]
    fn the_link_keeps_its_message() {
        let h = SectionHeading::new("Recent").link("View all", 7);
        assert_eq!(
            h.link.as_ref().map(|(l, m)| (l.as_str(), *m)),
            Some(("View all", 7))
        );
    }
}

/// Reusable, no-op-by-default presentation for [`SectionHeading`].
#[derive(Clone, Default)]
pub struct SectionHeadingProps {
    pub container_props: Option<ContainerProps>,
    pub title_props: Option<TextProps>,
    pub subtitle_props: Option<TextProps>,
    pub action_label_props: Option<TextProps>,
    pub action_props: Option<ButtonProps>,
}
impl SectionHeadingProps {
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = Some(props);
        self
    }

    pub fn title_props(mut self, props: TextProps) -> Self {
        self.title_props = Some(props);
        self
    }
    pub fn subtitle_props(mut self, props: TextProps) -> Self {
        self.subtitle_props = Some(props);
        self
    }
    pub fn action_label_props(mut self, props: TextProps) -> Self {
        self.action_label_props = Some(props);
        self
    }
    pub fn action_props(mut self, props: ButtonProps) -> Self {
        self.action_props = Some(props);
        self
    }
    pub fn apply<M>(&self, widget: SectionHeading<M>) -> SectionHeading<M> {
        let widget = if let Some(props) = &self.container_props {
            widget.container_props(props.clone())
        } else {
            widget
        };
        let widget = if let Some(props) = &self.title_props {
            widget.title_props(props.clone())
        } else {
            widget
        };
        let widget = if let Some(props) = &self.subtitle_props {
            widget.subtitle_props(props.clone())
        } else {
            widget
        };
        let widget = if let Some(props) = &self.action_label_props {
            widget.action_label_props(props.clone())
        } else {
            widget
        };
        let widget = if let Some(props) = &self.action_props {
            widget.action_props(props.clone())
        } else {
            widget
        };
        widget
    }
}

#[cfg(test)]
mod typed_presentation_tests {
    #[test]
    fn container_is_explicit_and_link_is_retained() {
        let heading = SectionHeadingProps::default()
            .action_props(ButtonProps::default().padding(17))
            .container_props(ContainerProps::default().width(123).height(45))
            .apply(SectionHeading::new("Heading").link("All", 7u8));
        assert_eq!(heading.link, Some(("All".into(), 7)));
        let element: Element<'_, u8> = heading.into();
        assert_eq!(
            element.as_widget().size(),
            iced::Size::new(Length::Fixed(123.0), Length::Fixed(45.0))
        );
    }

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
            SectionHeading::new("Title")
                .subtitle("Body")
                .link("All", 7u8)
        };
        let w = build();
        let baseline: Element<'_, u8> = w.into();
        let w = SectionHeadingProps::default().apply(build());
        let configured: Element<'_, u8> = w.into();
        assert_eq!(baseline.as_widget().size(), configured.as_widget().size());
        same_tree(&Tree::new(&baseline), &Tree::new(&configured));
    }
}
