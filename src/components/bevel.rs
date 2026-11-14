//! `Bevel`: a two-tone drawn frame around a widget (the raised and sunken borders of classic
//! desktop toolkits).
//!
//! iced draws one border color per quad, so a frame with a light top-left and a dark
//! bottom-right is drawn here, from the geometry in
//! [`primitives::bevel`](crate::primitives::bevel), as 1px quads over the widget's edge. The
//! colors are the theme's `border-light`, `border-dark`, `border-light-inner` and
//! `border-dark-inner` of the section the bevel was made for (`[button.<variant>]`,
//! `[frame.<name>]`), read when it is drawn, so a `[dark:...]` variant applies.
//!
//! The toolkit's own builders use it when a theme asks for it: a variant or frame whose
//! `border-style` is `raised`, `sunken`, `etched`, `bump` or `window` is wrapped, and one that is
//! `solid` or unset is built exactly as before. Use it yourself with [`Bevel::new`] and
//! [`bevel_of`].

use crate::primitives::bevel::{self, BevelStyle};
use crate::{Element, Theme};
use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::{Event, Length, Padding, Point, Rectangle, Size, Vector, mouse};

/// The bevel the installed theme asks of `section` (`frame.field`, `button.default`, …): its
/// `border-style` and `border-width`. `None` for `solid` or unset, which is the toolkit's own
/// border. A bevel's width is 1 or 2 rings (see [`bevel::rings`]); unset it is 2.
pub fn bevel_of(section: &str) -> Option<(BevelStyle, f32)> {
    let cfg = crate::theme_config::current();
    let style = BevelStyle::parse(cfg.text(section, "border-style")?)?;
    Some((style, cfg.number(section, "border-width").unwrap_or(2.0)))
}

/// `content` in the bevel of the frame `[frame.<name>]` when the installed theme gives it a drawn
/// `border-style`; `content` itself, untouched, when it is `solid` or unset. The toolkit's
/// surfaces use it (`window`, `card`, `field`, `well`, `panel`, `popover`, `dialog`), and so
/// can an app for its own panels.
pub fn framed<'a, M: 'a>(content: impl Into<Element<'a, M>>, name: &str) -> Element<'a, M> {
    let section = format!("frame.{name}");
    match bevel_of(&section) {
        Some((style, width)) => Bevel::new(content, section, style, width).into(),
        None => content.into(),
    }
}

/// See the [module](self).
pub struct Bevel<'a, M> {
    content: Element<'a, M>,
    section: String,
    style: BevelStyle,
    rings: u8,
    pressable: bool,
    padding: Padding,
}

struct State {
    pressed: bool,
}

impl<'a, M: 'a> Bevel<'a, M> {
    /// `content` inside a frame of `style` and `border_width` px, colored by the theme tokens of
    /// `section` (the same section [`bevel_of`] was asked for).
    pub fn new(
        content: impl Into<Element<'a, M>>,
        section: impl Into<String>,
        style: BevelStyle,
        border_width: f32,
    ) -> Self {
        Self {
            content: content.into(),
            section: section.into(),
            style,
            rings: bevel::rings(border_width),
            pressable: false,
            padding: Padding::ZERO,
        }
    }
    /// Space between the frame and the content (default none).
    pub fn padding(mut self, p: impl Into<Padding>) -> Self {
        self.padding = p.into();
        self
    }
    /// The frame sinks while the pointer holds the mouse button down on it: a button. Raised
    /// becomes sunken and the other way round.
    pub fn pressable(mut self, pressable: bool) -> Self {
        self.pressable = pressable;
        self
    }
    /// The space the rings take on each side.
    pub fn inset(&self) -> f32 {
        self.rings as f32
    }
    fn total_padding(&self) -> Padding {
        let i = self.inset();
        Padding {
            top: self.padding.top + i,
            right: self.padding.right + i,
            bottom: self.padding.bottom + i,
            left: self.padding.left + i,
        }
    }
}

impl<'a, M: 'a> Widget<M, Theme, iced::Renderer> for Bevel<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State { pressed: false })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = self.content.as_widget().size();
        let padding = self.total_padding();
        layout::padded(limits, size.width, size.height, padding, |limits| {
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
        })
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().expect("a content layout"),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if !self.pressable {
            return;
        }
        let state = tree.state.downcast_mut::<State>();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(layout.bounds()) =>
            {
                state.pressed = true;
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.pressed => {
                state.pressed = false;
                shell.request_redraw();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().expect("a content layout"),
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().expect("a content layout"),
            renderer,
            operation,
        );
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().expect("a content layout"),
            cursor,
            viewport,
        );
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();
        let style = if self.pressable && state.pressed && cursor.is_over(bounds) {
            self.style.pressed()
        } else {
            self.style
        };
        let colors = theme.bevel_colors(&self.section);
        for edge in bevel::edges(style, self.rings, bounds.size()) {
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle::new(
                        Point::new(bounds.x + edge.rect.x, bounds.y + edge.rect.y),
                        edge.rect.size(),
                    ),
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    snap: true,
                },
                colors.tone(edge.tone),
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next()?,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, M: 'a> From<Bevel<'a, M>> for Element<'a, M> {
    fn from(b: Bevel<'a, M>) -> Self {
        iced::Element::new(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_takes_one_or_two_rings_of_space_plus_its_padding() {
        let b = Bevel::<u8>::new(iced::widget::space(), "frame.field", BevelStyle::Sunken, 2.0);
        assert_eq!(b.inset(), 2.0);
        let b = Bevel::<u8>::new(iced::widget::space(), "frame.field", BevelStyle::Sunken, 1.0)
            .padding([3.0, 5.0]);
        let p = b.total_padding();
        assert_eq!((p.top, p.left), (4.0, 6.0));
    }

    #[test]
    fn an_unset_or_solid_theme_asks_for_no_bevel() {
        assert_eq!(bevel_of("frame.field"), None);
        assert_eq!(bevel_of("button.default"), None);
    }
}
