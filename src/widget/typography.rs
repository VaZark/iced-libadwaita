//! Typography classes: `.title-1` … `.title-4`, `.heading`, `.caption`,
//! `.caption-heading`, `.monospace`, `.dim-label`.
//!
//! The free functions use [`Typography::default`]; call the same-named
//! methods on a [`Typography`] to use custom sizes.

use crate::style;
use crate::{Theme, Typography};
use iced::font::{Family, Weight};
use iced::widget::{text, Text};
use iced::Font;

fn weight(w: Weight) -> Font {
    Font {
        weight: w,
        ..Font::DEFAULT
    }
}

impl Typography {
    pub fn title_1<'a>(&self, content: impl text::IntoFragment<'a>) -> Text<'a, Theme> {
        text(content).size(self.title_1).font(weight(Weight::Bold))
    }
    pub fn title_2<'a>(&self, content: impl text::IntoFragment<'a>) -> Text<'a, Theme> {
        text(content).size(self.title_2).font(weight(Weight::Bold))
    }
    pub fn title_3<'a>(&self, content: impl text::IntoFragment<'a>) -> Text<'a, Theme> {
        text(content).size(self.title_3).font(weight(Weight::Bold))
    }
    pub fn title_4<'a>(&self, content: impl text::IntoFragment<'a>) -> Text<'a, Theme> {
        text(content).size(self.title_4).font(weight(Weight::Bold))
    }
    pub fn heading<'a>(&self, content: impl text::IntoFragment<'a>) -> Text<'a, Theme> {
        text(content).size(self.heading).font(weight(Weight::Bold)).wrapping(text::Wrapping::None)
    }
    pub fn body<'a>(&self, content: impl text::IntoFragment<'a>) -> Text<'a, Theme> {
        text(content).size(self.body)
    }
    pub fn caption_heading<'a>(&self, content: impl text::IntoFragment<'a>) -> Text<'a, Theme> {
        text(content).size(self.caption_heading).font(weight(Weight::Bold))
    }
    pub fn caption<'a>(&self, content: impl text::IntoFragment<'a>) -> Text<'a, Theme> {
        text(content).size(self.caption)
    }
    pub fn monospace<'a>(&self, content: impl text::IntoFragment<'a>) -> Text<'a, Theme> {
        text(content).size(self.monospace).font(Font {
            family: Family::Monospace,
            ..Font::DEFAULT
        })
    }
    /// `.dim-label` body text.
    pub fn dim_label<'a>(&self, content: impl text::IntoFragment<'a>) -> Text<'a, Theme> {
        self.body(content).style(style::text::dim)
    }
}

/// A single-line label that never wraps (button, tab, sidebar and header
/// labels). libadwaita ellipsizes; iced cannot, so we keep the full text and
/// let the surrounding layout wrap or scroll instead of squeezing it.
pub fn label<'a>(content: impl text::IntoFragment<'a>) -> Text<'a, Theme> {
    text(content).wrapping(text::Wrapping::None)
}

macro_rules! free {
    ($($name:ident),*) => {$(
        pub fn $name<'a>(content: impl text::IntoFragment<'a>) -> Text<'a, Theme> {
            Typography::default().$name(content)
        }
    )*};
}
free!(title_1, title_2, title_3, title_4, heading, body, caption_heading, caption, monospace, dim_label);
