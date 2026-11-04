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
}

impl<M> SectionHeading<M> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            link: None,
            metrics: Metrics::desktop(),
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
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: Clone + 'a> From<SectionHeading<M>> for Element<'a, M> {
    fn from(h: SectionHeading<M>) -> Self {
        let m = h.metrics;
        let mut title = iced::widget::column![ty::heading(h.title)].spacing(m.space_xs);
        if let Some(s) = h.subtitle {
            title = title.push(ty::caption(s).style(crate::styles::text::dim));
        }
        let mut r = row![title.width(Length::Fill)]
            .spacing(m.space_m)
            .align_y(Alignment::Center);
        match h.link {
            Some((label, msg)) => {
                r = r.push(
                    flat_button(
                        row![ty::body(label), Icon::GoNext.view(m.icon_size - 2.0)]
                            .spacing(m.space_xs)
                            .align_y(Alignment::Center),
                    )
                    .on_press(msg),
                );
            }
            None => r = r.push(space().width(0)),
        }
        r.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_link_keeps_its_message() {
        let h = SectionHeading::new("Recent").link("View all", 7);
        assert_eq!(
            h.link.as_ref().map(|(l, m)| (l.as_str(), *m)),
            Some(("View all", 7))
        );
    }
}
