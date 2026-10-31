//! `AdwAvatar`.

use super::Element;
use crate::style::container as sc;
use iced::widget::{container, text};
use iced::{Color, Length};

/// Background colors used for generated avatars (libadwaita picks from a
/// fixed set by hashing the text).
const COLORS: [u32; 12] = [
    0x62a0ea, 0x57e389, 0xf8e45c, 0xffa348, 0xff7b63, 0xdc8add, 0xc061cb, 0x9a9996, 0x5e5c64,
    0x99c1f1, 0x8ff0a4, 0xf66151,
];

/// `AdwAvatar`: round badge with initials.
pub struct Avatar {
    size: f32,
    text: String,
    show_initials: bool,
    color: Option<Color>,
    text_color: Option<Color>,
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
        }
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
        let hash = a
            .text
            .bytes()
            .fold(5381u32, |h, b| h.wrapping_mul(33).wrapping_add(b as u32));
        let bg = a
            .color
            .unwrap_or_else(|| crate::color::hex(COLORS[hash as usize % COLORS.len()]));
        let fill = crate::color::darken(bg, 0.08);
        let label = if a.show_initials {
            initials(&a.text)
        } else {
            String::new()
        };
        container(text(label).size(a.size * 0.4).font(iced::Font {
            weight: iced::font::Weight::Bold,
            ..iced::Font::DEFAULT
        }))
        .center(Length::Fixed(a.size))
        .style(sc::avatar(
            fill,
            a.text_color
                .unwrap_or_else(|| crate::color::readable_on(fill)),
            a.size,
        ))
        .into()
    }
}
