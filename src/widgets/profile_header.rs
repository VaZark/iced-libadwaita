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
    spacing: Option<f32>,
    body_spacing: Option<f32>,
    text_spacing: Option<f32>,
    padding: Option<iced::Padding>,
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
            spacing: None,
            body_spacing: None,
            text_spacing: None,
            padding: None,
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
    /// The gap between the image, the text and the top-right slot (default: the theme's
    /// `space_l`, or `space_m` when narrow).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
        self
    }
    /// The gap between the image-and-text row and the controls (default: the theme's `space_m`).
    pub fn body_spacing(mut self, px: f32) -> Self {
        self.body_spacing = Some(px);
        self
    }
    /// The gap between the name, the after-name slot and the description (default: the theme's
    /// `space_s`).
    pub fn text_spacing(mut self, px: f32) -> Self {
        self.text_spacing = Some(px);
        self
    }
    /// The padding around the header (default: the theme's page padding plus 4).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = Some(p.into());
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
            .spacing(h.text_spacing.unwrap_or(m.space_s))
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
            let mut col = column![].spacing(h.spacing.unwrap_or(m.space_m));
            if let Some(end) = h.end {
                col = col.push(row![space::horizontal(), end]);
            }
            col.push(container(h.image).center_x(Length::Fill))
                .push(text)
        } else {
            let mut top = row![h.image, text.width(Length::Fill)]
                .spacing(h.spacing.unwrap_or(m.space_l))
                .align_y(Alignment::Center);
            if let Some(end) = h.end {
                top = top.push(end);
            }
            column![top].spacing(h.body_spacing.unwrap_or(m.space_m))
        };
        if let Some(controls) = h.controls {
            body = body.push(controls);
        }
        container(body)
            .padding(h.padding.unwrap_or(iced::Padding::from(m.page_padding.left + 4.0)))
            .width(Length::Fill)
            .style(move |t: &crate::Theme| {
                h.tint
                    .map(|c| crate::styles::container::wash(c)(t))
                    .unwrap_or_default()
            })
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_parts_take_their_own_spacing_and_padding() {
        let h = ProfileHeader::<()>::new("Name", space(), space())
            .spacing(3.0)
            .body_spacing(5.0)
            .text_spacing(7.0)
            .padding([1.0, 2.0]);
        assert_eq!((h.spacing, h.body_spacing, h.text_spacing), (Some(3.0), Some(5.0), Some(7.0)));
        assert_eq!(h.padding, Some(iced::Padding::from([1.0, 2.0])));
        let _: Element<'_, ()> = h.into();
        let _: Element<'_, ()> = ProfileHeader::<()>::new("N", space(), space()).narrow(true).spacing(2.0).into();
    }
}
