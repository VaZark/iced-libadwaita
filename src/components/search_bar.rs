//! `SearchBar`: the pill-shaped search field of a music app: a magnifier inside at the left, the
//! text, and a close cross at the right. For finding something now, not for settings: the app
//! decides whether the text is kept.

use crate::Element;
use crate::color::alpha;
use crate::components::icon_button;
use crate::{Icon, Metrics, Theme};
use iced::widget::{container, row, text_input};
use iced::{Alignment, Border, Length};

/// See the [module](self).
pub struct SearchBar<'a, M> {
    value: &'a str,
    placeholder: String,
    on_input: Box<dyn Fn(String) -> M + 'a>,
    on_close: Option<M>,
    id: Option<iced::widget::Id>,
    width: Length,
    spacing: Option<f32>,
    padding: Option<iced::Padding>,
    input_padding: Option<iced::Padding>,
    metrics: Metrics,
}

/// A search bar showing `value` and sending each edit through `on_input`.
pub fn search_bar<'a, M>(value: &'a str, on_input: impl Fn(String) -> M + 'a) -> SearchBar<'a, M> {
    SearchBar {
        value,
        placeholder: "Search".into(),
        on_input: Box::new(on_input),
        on_close: None,
        id: None,
        width: Length::Fill,
        spacing: None,
        padding: None,
        input_padding: None,
        metrics: Metrics::desktop(),
    }
}

impl<'a, M> SearchBar<'a, M> {
    /// The gap between the icon, the field and the close button (default `space_s`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
        self
    }
    /// The padding of the pill (default `space_xxs` above and below, `space_m` at the sides).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }
    /// The padding inside the text field (default `space_s`).
    pub fn input_padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.input_padding = Some(p.into());
        self
    }
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }
    /// The cross at the right: what pressing it sends (the app clears the text and closes the bar).
    pub fn on_close(mut self, message: M) -> Self {
        self.on_close = Some(message);
        self
    }
    /// An id, so the bar can take focus as it appears (see [`FocusScope`](crate::binds::focus)).
    pub fn id(mut self, id: impl Into<iced::widget::Id>) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

impl<'a, M: Clone + 'a> From<SearchBar<'a, M>> for Element<'a, M> {
    fn from(b: SearchBar<'a, M>) -> Self {
        let m = b.metrics;
        let mut input = text_input(&b.placeholder, b.value)
            .on_input(b.on_input)
            .padding(b.input_padding.unwrap_or(iced::Padding::from(m.space_s)))
            .width(Length::Fill)
            .style(|theme: &Theme, status| {
                // The pill draws the field: the input itself is bare.
                let mut s = crate::styles::text_input::default(theme, status);
                s.background = iced::Background::Color(iced::Color::TRANSPARENT);
                s.border = Border::default();
                s
            });
        if let Some(id) = b.id {
            input = input.id(id);
        }
        let mut r = row![Icon::Search.view(m.icon_size), input]
            .spacing(b.spacing.unwrap_or(m.space_s))
            .align_y(Alignment::Center);
        if let Some(close) = b.on_close {
            r = r.push(icon_button(Icon::WindowClose).on_press(close));
        }
        container(r)
            .padding(
                b.padding
                    .unwrap_or(iced::Padding::from([m.space_xxs, m.space_m])),
            )
            .width(b.width)
            .style(|theme: &Theme| {
                let fg = theme.semantic.window.fg;
                container::Style {
                    background: Some(alpha(fg, 0.09).into()),
                    border: Border {
                        color: alpha(fg, 0.14),
                        width: 1.0,
                        radius: 999.0.into(),
                    },
                    ..Default::default()
                }
            })
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_builds_with_and_without_a_close_cross() {
        let _: Element<'_, u8> = search_bar("", |_| 1).into();
        let _: Element<'_, u8> = search_bar("luna", |_| 1)
            .placeholder("What do you want to play?")
            .on_close(2)
            .into();
    }
}

#[cfg(test)]
mod spacing_params {
    use super::*;

    #[test]
    fn search_bar_gaps_and_padding_are_parameters() {
        let b = search_bar("", |_: String| ()).spacing(1.0).padding(2.0).input_padding(3.0);
        assert_eq!((b.spacing, b.padding), (Some(1.0), Some(iced::Padding::from(2.0))));
        let _: Element<'_, ()> = b.into();
    }
}
