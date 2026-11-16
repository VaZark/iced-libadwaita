//! `SectionDivider`: a section's name, a thin line running from it to the far end, and the
//! section's action there. By default the action is the **toggle**: a chevron that folds and
//! unfolds the section, and pressing the name or the line does the same. Use it above a list the
//! user can collapse (a queue's history, a long group).
//!
//! ```ignore
//! SectionDivider::new("History").detail("12 tracks").open(open).on_toggle(Msg::ToggleHistory)
//! ```
//!
//! It draws the header only; the app owns `open` and decides what to show under it (wrap the
//! body in a [`Collapsible`](super::Collapsible) to animate it).

use crate::Element;
use crate::components::{ButtonProps, ContainerProps, TextProps};
use crate::components::{icon_button, typography as ty};
use crate::{Icon, Metrics, Theme};
use iced::widget::{container, mouse_area, row};
use iced::{Alignment, Length, mouse};

/// Initial state for a section whose open state is owned by the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollapseState {
    Expanded,
    Collapsed,
}

/// See the [module](self).
pub struct SectionDivider<'a, M> {
    container_props: Option<ContainerProps>,
    title_props: TextProps,
    detail_props: TextProps,
    line_props: ContainerProps,
    toggle_props: ButtonProps,
    title: String,
    detail: Option<String>,
    open: bool,
    collapsible: Option<bool>,
    chevron: bool,
    trailing: Option<Element<'a, M>>,
    on_toggle: Option<M>,
    action: Option<Element<'a, M>>,
    metrics: Metrics,
    spacing: Option<f32>,
    end_spacing: Option<f32>,
    padding: Option<iced::Padding>,
}

impl<'a, M> SectionDivider<'a, M> {
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
    pub fn detail_props(mut self, props: TextProps) -> Self {
        self.detail_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn line_props(mut self, props: ContainerProps) -> Self {
        self.line_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn toggle_props(mut self, props: ButtonProps) -> Self {
        self.toggle_props = props;
        self
    }

    pub fn new(title: impl Into<String>) -> Self {
        Self {
            container_props: None,
            title_props: TextProps::default(),
            detail_props: TextProps::default(),
            line_props: ContainerProps::default(),
            toggle_props: ButtonProps::default(),

            title: title.into(),
            detail: None,
            open: true,
            collapsible: None,
            chevron: true,
            trailing: None,
            on_toggle: None,
            action: None,
            metrics: Metrics::desktop(),
            spacing: None,
            end_spacing: None,
            padding: None,
        }
    }

    /// Whether presses toggle the caller-owned section. With no callback it remains a heading.
    pub fn collapsible(mut self, enabled: bool) -> Self {
        self.collapsible = Some(enabled);
        self
    }
    pub fn default_collapse_state(mut self, state: CollapseState) -> Self {
        self.open = state == CollapseState::Expanded;
        self
    }
    pub fn chevron(mut self, show: bool) -> Self {
        self.chevron = show;
        self
    }
    /// Extra actions beside the collapse control, without replacing it.
    pub fn trailing(mut self, action: impl Into<Element<'a, M>>) -> Self {
        self.trailing = Some(action.into());
        self
    }
    /// A dimmed note after the name (a count).
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Whether the section is open: the chevron points down when it is, and back towards the name, the way the line comes from, when it is folded.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// What pressing the chevron, the name or the line sends. Without it the divider is only a
    /// heading.
    pub fn on_toggle(mut self, message: M) -> Self {
        self.on_toggle = Some(message);
        self
    }

    /// Replaces the chevron with an action of your own at the far end; the name and the line
    /// still toggle when [`on_toggle`](Self::on_toggle) is set.
    pub fn action(mut self, action: impl Into<Element<'a, M>>) -> Self {
        self.action = Some(action.into());
        self
    }

    /// The gap between the name, its detail and the line (default: the theme's `space_m`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
        self
    }
    /// The gap between the line and the chevron, action or trailing slot (default: the theme's
    /// `space_s`).
    pub fn end_spacing(mut self, px: f32) -> Self {
        self.end_spacing = Some(px);
        self
    }
    /// Padding around the divider (default: none).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }

    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: Clone + 'a> From<SectionDivider<'a, M>> for Element<'a, M> {
    fn from(mut d: SectionDivider<'a, M>) -> Self {
        if !d.collapsible.unwrap_or(d.on_toggle.is_some()) {
            d.on_toggle = None;
        }
        let m = d.metrics;
        let line = d.line_props.apply(
            container(iced::widget::space())
                .width(Length::Fill)
                .height(Length::Fixed(1.0))
                .style(|t: &Theme| container::Style {
                    background: Some(crate::color::alpha(t.semantic.window.fg, 0.12).into()),
                    ..Default::default()
                }),
        );
        let mut head = row![d.title_props.apply(ty::heading(d.title))]
            .spacing(d.spacing.unwrap_or(m.space_m))
            .align_y(Alignment::Center);
        if let Some(detail) = d.detail {
            head = head.push(
                d.detail_props
                    .apply(ty::caption(detail).style(crate::styles::text::dim)),
            );
        }
        let head = head.push(line).width(Length::Fill);
        let head: Element<'a, M> = match d.on_toggle.clone() {
            Some(msg) => mouse_area(head)
                .on_press(msg)
                .interaction(mouse::Interaction::Pointer)
                .into(),
            None => head.into(),
        };
        let end: Element<'a, M> = match (d.action, d.on_toggle) {
            (Some(action), _) => action,
            (None, Some(msg)) if d.chevron => d
                .toggle_props
                .apply(
                    icon_button(if d.open {
                        Icon::GoDown
                    } else {
                        Icon::GoPrevious
                    })
                    .on_press(msg),
                )
                .into(),
            (None, _) => iced::widget::space().width(0).into(),
        };
        let mut row = row![head, end]
            .spacing(d.end_spacing.unwrap_or(m.space_s))
            .align_y(Alignment::Center);
        if let Some(trailing) = d.trailing {
            row = row.push(trailing);
        }
        match (d.padding, d.container_props) {
            (padding, Some(props)) => props
                .apply(container(row).padding(padding.unwrap_or(iced::Padding::ZERO)))
                .into(),
            (Some(p), None) => container(row).padding(p).into(),
            (None, None) => row.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_builds_with_the_default_toggle_a_custom_action_or_none() {
        let _: Element<'_, u8> = SectionDivider::new("History")
            .detail("3 tracks")
            .open(false)
            .on_toggle(1)
            .into();
        let _: Element<'_, u8> = SectionDivider::new("Up next")
            .on_toggle(1)
            .action(icon_button(Icon::Delete).on_press(2))
            .into();
        let _: Element<'_, u8> = SectionDivider::new("Plain").into();
        let d = SectionDivider::<u8>::new("Spaced")
            .spacing(2.0)
            .end_spacing(3.0)
            .padding([1.0, 4.0]);
        assert_eq!((d.spacing, d.end_spacing), (Some(2.0), Some(3.0)));
        let _: Element<'_, u8> = d.into();
    }
}

/// Reusable, no-op-by-default presentation for [`SectionDivider`].
#[derive(Clone, Default)]
pub struct SectionDividerProps {
    pub container_props: Option<ContainerProps>,
    pub title_props: Option<TextProps>,
    pub detail_props: Option<TextProps>,
    pub line_props: Option<ContainerProps>,
    pub toggle_props: Option<ButtonProps>,
}
impl SectionDividerProps {
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = Some(props);
        self
    }

    pub fn title_props(mut self, props: TextProps) -> Self {
        self.title_props = Some(props);
        self
    }
    pub fn detail_props(mut self, props: TextProps) -> Self {
        self.detail_props = Some(props);
        self
    }
    pub fn line_props(mut self, props: ContainerProps) -> Self {
        self.line_props = Some(props);
        self
    }
    pub fn toggle_props(mut self, props: ButtonProps) -> Self {
        self.toggle_props = Some(props);
        self
    }
    pub fn apply<'a, M>(&self, widget: SectionDivider<'a, M>) -> SectionDivider<'a, M> {
        let mut widget = widget;
        crate::components::props::merge_optional!(widget, self; container_props);
        if let Some(props) = &self.title_props {
            widget.title_props.merge(props);
        }
        if let Some(props) = &self.detail_props {
            widget.detail_props.merge(props);
        }
        if let Some(props) = &self.line_props {
            widget.line_props.merge(props);
        }
        if let Some(props) = &self.toggle_props {
            widget.toggle_props.merge(props);
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
        let build = || SectionDivider::new("Title").detail("Body").on_toggle(7u8);
        let w = build();
        let baseline: Element<'_, u8> = w.into();
        let w = SectionDividerProps::default().apply(build());
        let configured: Element<'_, u8> = w.into();
        assert_eq!(baseline.as_widget().size(), configured.as_widget().size());
        same_tree(&Tree::new(&baseline), &Tree::new(&configured));
    }
}
