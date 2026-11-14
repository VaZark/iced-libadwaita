//! `SectionHeading`: a heading over a section with an optional link at the far end, "View all ›".
//! Home pages, explore pages, any list that is a preview of a longer one.
//!
//! ```ignore
//! SectionHeading::new("Recently played").link("View all", Msg::OpenHistory)
//! ```

use crate::Element;
use crate::components::{flat_button, typography as ty};
use crate::{Icon, Metrics};
use iced::widget::{row, space};
use iced::{Alignment, Length};

/// See the [module](self).
pub struct SectionHeading<M> {
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
    pub fn new(title: impl Into<String>) -> Self {
        Self {
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
        let mut title = iced::widget::column![ty::heading(h.title)].spacing(h.title_spacing.unwrap_or(m.space_xs));
        if let Some(s) = h.subtitle {
            title = title.push(ty::caption(s).style(crate::styles::text::dim));
        }
        let mut r = row![title.width(Length::Fill)]
            .spacing(h.spacing.unwrap_or(m.space_m))
            .align_y(Alignment::Center);
        match h.link {
            Some((label, msg)) => {
                r = r.push(
                    flat_button(
                        row![ty::body(label), Icon::GoNext.view(m.icon_size - 2.0)]
                            .spacing(h.link_spacing.unwrap_or(m.space_xs))
                            .align_y(Alignment::Center),
                    )
                    .on_press(msg),
                );
            }
            None => r = r.push(space().width(0)),
        }
        match h.padding {
            Some(p) => iced::widget::container(r).padding(p).into(),
            None => r.into(),
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
        assert_eq!((h.spacing, h.title_spacing, h.link_spacing), (Some(2.0), Some(3.0), Some(4.0)));
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
