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
use crate::components::{icon_button, typography as ty};
use crate::{Icon, Metrics, Theme};
use iced::widget::{container, mouse_area, row};
use iced::{Alignment, Length, mouse};

/// See the [module](self).
pub struct SectionDivider<'a, M> {
    title: String,
    detail: Option<String>,
    open: bool,
    on_toggle: Option<M>,
    action: Option<Element<'a, M>>,
    metrics: Metrics,
}

impl<'a, M> SectionDivider<'a, M> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            detail: None,
            open: true,
            on_toggle: None,
            action: None,
            metrics: Metrics::desktop(),
        }
    }

    /// A dimmed note after the name (a count).
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Whether the section is open: the chevron points up when it is, down when it is folded.
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

    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: Clone + 'a> From<SectionDivider<'a, M>> for Element<'a, M> {
    fn from(d: SectionDivider<'a, M>) -> Self {
        let m = d.metrics;
        let line = container(iced::widget::space())
            .width(Length::Fill)
            .height(Length::Fixed(1.0))
            .style(|t: &Theme| container::Style {
                background: Some(crate::color::alpha(t.semantic.window.fg, 0.12).into()),
                ..Default::default()
            });
        let mut head = row![ty::heading(d.title)]
            .spacing(m.space_m)
            .align_y(Alignment::Center);
        if let Some(detail) = d.detail {
            head = head.push(ty::caption(detail).style(crate::styles::text::dim));
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
            (None, Some(msg)) => icon_button(if d.open { Icon::GoUp } else { Icon::GoDown })
                .on_press(msg)
                .into(),
            (None, None) => iced::widget::space().width(0).into(),
        };
        row![head, end]
            .spacing(m.space_s)
            .align_y(Alignment::Center)
            .into()
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
    }
}
