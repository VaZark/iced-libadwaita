//! `AdwAvatar`, with an optional icon in place of the initials and a choice of shape: the round
//! badge of a person, a playlist, a cover.

use crate::Element;
use crate::IconSource;
use iced::widget::{container, text};
use iced::{Color, Length};

/// The outline of an [`Avatar`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AvatarShape {
    /// A circle (libadwaita's avatar).
    #[default]
    Circle,
    /// A square with rounded corners: a cover, an app icon.
    Rounded,
    /// A square.
    Square,
}

impl AvatarShape {
    /// The corner radius for an avatar of `size`.
    fn radius(self, size: f32) -> f32 {
        match self {
            Self::Circle => size / 2.0,
            Self::Rounded => size * 0.22,
            Self::Square => 0.0,
        }
    }
}

/// `AdwAvatar`: round badge with initials.
pub struct Avatar {
    size: f32,
    text: String,
    show_initials: bool,
    color: Option<Color>,
    text_color: Option<Color>,
    icon: Option<IconSource>,
    shape: AvatarShape,
}

pub fn avatar(size: f32, text: impl Into<String>) -> Avatar {
    Avatar::new(size, text)
}

impl Avatar {
    pub fn new(size: f32, text: impl Into<String>) -> Self {
        Self {
            size,
            text: text.into(),
            show_initials: true,
            color: None,
            text_color: None,
            icon: None,
            shape: AvatarShape::Circle,
        }
    }
    /// An icon (a built-in [`Icon`](crate::Icon) or your own SVG) in place of the initials. The
    /// background color still comes from the text, so the same name keeps the same color.
    pub fn icon(mut self, icon: impl Into<IconSource>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    /// The outline: a circle (default), a rounded square or a square.
    pub fn shape(mut self, shape: AvatarShape) -> Self {
        self.shape = shape;
        self
    }
    pub fn show_initials(mut self, show: bool) -> Self {
        self.show_initials = show;
        self
    }
    /// Hard override for the initials' color; by default it is chosen by contrast with the
    /// background (CSS `color-contrast()`).
    pub fn text_color(mut self, c: Color) -> Self {
        self.text_color = Some(c);
        self
    }
    /// Overrides the generated background color.
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}

fn initials(s: &str) -> String {
    let mut words = s.split_whitespace();
    let first = words.next().and_then(|w| w.chars().next());
    let last = words.last().and_then(|w| w.chars().next());
    first
        .into_iter()
        .chain(last)
        .flat_map(|c| c.to_uppercase())
        .collect()
}

impl<'a, M: 'a> From<Avatar> for Element<'a, M> {
    fn from(a: Avatar) -> Self {
        let bg = a.color.unwrap_or_else(|| crate::color::from_seed(&a.text));
        let fill = crate::color::darken(bg, 0.08);
        let ink = a
            .text_color
            .unwrap_or_else(|| crate::color::readable_on(fill));
        let radius = a.shape.radius(a.size);
        let content: Element<'a, M> = match &a.icon {
            Some(icon) => icon
                .view(a.size * 0.5)
                .style(move |_: &crate::Theme, _| iced::widget::svg::Style { color: Some(ink) })
                .into(),
            None => {
                let label = if a.show_initials {
                    initials(&a.text)
                } else {
                    String::new()
                };
                text(label)
                    .size(a.size * 0.4)
                    .font(iced::Font {
                        weight: iced::font::Weight::Bold,
                        ..iced::Font::DEFAULT
                    })
                    .into()
            }
        };
        container(content)
            .center(Length::Fixed(a.size))
            .style(move |_: &crate::Theme| iced::widget::container::Style {
                text_color: Some(ink),
                background: Some(iced::Background::Color(fill)),
                border: iced::Border::default().rounded(radius),
                ..Default::default()
            })
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes_round_by_size() {
        assert_eq!(AvatarShape::Circle.radius(40.0), 20.0);
        assert_eq!(AvatarShape::Square.radius(40.0), 0.0);
        assert!(
            AvatarShape::Rounded.radius(40.0) > 0.0 && AvatarShape::Rounded.radius(40.0) < 20.0
        );
    }

    #[test]
    fn the_same_name_keeps_its_color() {
        assert_eq!(
            crate::color::from_seed("Road trip"),
            crate::color::from_seed("Road trip")
        );
    }
}
