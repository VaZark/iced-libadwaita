//! `Pixels`: a [`PixelMap`](crate::primitives::pixmap::PixelMap) drawn at one logical pixel per
//! map pixel, in the theme's colors.
//!
//! The classic glyphs (check marks, scrollbar arrows, window-button symbols) and the round radio
//! box are pixel patterns. `Pixels` draws one as a few runs of 1px quads. `L`, `D`, `l` and `d`
//! take the bevel colors of the theme section it was made for (`frame.check`, `button.default`,
//! …), `F` the section's `bg` (or the view background), and `X` the ink: the text color of the
//! widget it sits in, or a color given with [`ink`](Pixels::ink).

use crate::primitives::pixmap::{Pixel, PixelMap};
use crate::{Element, Theme};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Tree, Widget};
use iced::advanced::Renderer as _;
use iced::{Color, Length, Point, Rectangle, Size, mouse};

/// See the [module](self).
pub struct Pixels {
    map: PixelMap,
    section: String,
    ink: Option<Color>,
    disabled: bool,
}

impl Pixels {
    /// `map`, colored by the tokens of `section`.
    pub fn new(map: PixelMap, section: impl Into<String>) -> Self {
        Self {
            map,
            section: section.into(),
            ink: None,
            disabled: false,
        }
    }
    /// A fixed ink instead of the surrounding text color.
    pub fn ink(mut self, color: Color) -> Self {
        self.ink = Some(color);
        self
    }
    /// Draws the ink in the section's `border-dark-inner` (the classic disabled gray).
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Its size in logical pixels.
    pub fn size_px(&self) -> Size {
        self.map.size()
    }
}

impl<M> Widget<M, Theme, iced::Renderer> for Pixels {
    fn size(&self) -> Size<Length> {
        let s = self.map.size();
        Size::new(Length::Fixed(s.width), Length::Fixed(s.height))
    }

    fn layout(&mut self, _tree: &mut Tree, _renderer: &iced::Renderer, _limits: &layout::Limits) -> layout::Node {
        layout::Node::new(self.map.size())
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let colors = theme.bevel_colors(&self.section);
        let fill = theme
            .token_color(&self.section, "bg")
            .unwrap_or(theme.semantic.view.bg);
        let ink = if self.disabled {
            colors.dark_inner
        } else {
            self.ink.unwrap_or(style.text_color)
        };
        for run in self.map.runs() {
            let color = match run.pixel {
                Pixel::Light => colors.light,
                Pixel::Dark => colors.dark,
                Pixel::LightInner => colors.light_inner,
                Pixel::DarkInner => colors.dark_inner,
                Pixel::Fill => fill,
                Pixel::Ink => ink,
            };
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle::new(
                        Point::new(bounds.x + run.x as f32, bounds.y + run.y as f32),
                        Size::new(run.width as f32, 1.0),
                    ),
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    snap: true,
                },
                color,
            );
        }
    }
}

impl<'a, M: 'a> From<Pixels> for Element<'a, M> {
    fn from(p: Pixels) -> Self {
        iced::Element::new(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::pixmap;

    #[test]
    fn it_is_as_big_as_its_map() {
        let p = Pixels::new(pixmap::check_box(13), "frame.check");
        assert_eq!(p.size_px(), Size::new(13.0, 13.0));
        let _: Element<'_, ()> = p.into();
    }
}
