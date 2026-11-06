//! A composable profile: image, name and description, with optional caller-owned slots.
//! No controls, search, metadata, playback behavior or background wash are supplied implicitly.
use crate::components::typography as ty;
use crate::{Element, Metrics};
use iced::widget::{column, container, row, space};
use iced::{Alignment, Color, Length};
pub struct ProfileHeader<'a, M> {
    name: String,
    image: Element<'a, M>,
    description: Element<'a, M>,
    after: Option<Element<'a, M>>,
    end: Option<Element<'a, M>>,
    controls: Option<Element<'a, M>>,
    narrow: bool,
    tint: Option<Color>,
    metrics: Metrics,
}
impl<'a, M: 'a> ProfileHeader<'a, M> {
    pub fn new(
        name: impl Into<String>,
        image: impl Into<Element<'a, M>>,
        description: impl Into<Element<'a, M>>,
    ) -> Self {
        Self {
            name: name.into(),
            image: image.into(),
            description: description.into(),
            after: None,
            end: None,
            controls: None,
            narrow: false,
            tint: None,
            metrics: Metrics::desktop(),
        }
    }
    pub fn after_name(mut self, content: impl Into<Element<'a, M>>) -> Self {
        self.after = Some(content.into());
        self
    }
    pub fn top_right(mut self, content: impl Into<Element<'a, M>>) -> Self {
        self.end = Some(content.into());
        self
    }
    pub fn controls(mut self, content: impl Into<Element<'a, M>>) -> Self {
        self.controls = Some(content.into());
        self
    }
    pub fn narrow(mut self, narrow: bool) -> Self {
        self.narrow = narrow;
        self
    }
    pub fn tint(mut self, tint: Color) -> Self {
        self.tint = Some(tint);
        self
    }
    pub fn metrics(mut self, metrics: Metrics) -> Self {
        self.metrics = metrics;
        self
    }
}
impl<'a, M: 'a> From<ProfileHeader<'a, M>> for Element<'a, M> {
    fn from(h: ProfileHeader<'a, M>) -> Self {
        let m = h.metrics;
        let alignment = if h.narrow {
            Alignment::Center
        } else {
            Alignment::Start
        };
        let mut text = column![ty::title_2(h.name).width(Length::Fill).align_x(alignment)]
            .spacing(m.space_s)
            .align_x(alignment)
            .width(Length::Fill);
        if let Some(after) = h.after {
            text = text.push(container(after).width(Length::Fill).align_x(alignment));
        }
        text = text.push(
            container(h.description)
                .width(Length::Fill)
                .align_x(alignment),
        );
        let mut body = if h.narrow {
            let mut col = column![].spacing(m.space_m);
            if let Some(end) = h.end {
                col = col.push(row![space::horizontal(), end]);
            }
            col.push(container(h.image).center_x(Length::Fill))
                .push(text)
        } else {
            let mut top = row![h.image, text.width(Length::Fill)]
                .spacing(m.space_l)
                .align_y(Alignment::Center);
            if let Some(end) = h.end {
                top = top.push(end);
            }
            column![top].spacing(m.space_m)
        };
        if let Some(controls) = h.controls {
            body = body.push(controls);
        }
        container(body)
            .padding(m.page_padding.left + 4.0)
            .width(Length::Fill)
            .style(move |t: &crate::Theme| {
                h.tint
                    .map(|c| crate::styles::container::wash(c)(t))
                    .unwrap_or_default()
            })
            .into()
    }
}
