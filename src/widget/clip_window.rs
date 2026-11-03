//! `ClipWindow`: shows its content through a window of a given height and **translates** it
//! instead of squeezing it.
//!
//! A plain container with a smaller fixed height gives its child less room, so the child
//! **reflows**: a cover shrinks, a row wraps, a column squashes. That is the right answer for a
//! panel that is being resized and the wrong one for a panel that is being *revealed*, where the
//! content should keep its size and slide into view. Chakra's `Collapse`, Mantine's `Collapse`
//! and `GtkRevealer` all measure the content at its natural size and move it; this does the same.
//!
//! * The content is laid out with all the height it wants.
//! * The window is [`height`](ClipWindow::height) tall, and shows the content from its
//!   [`top`](Anchor::Top) (shrinking the window cuts the bottom off) or from its
//!   [`bottom`](Anchor::Bottom) (shrinking cuts the top off: the content is pushed up).
//! * Anything outside the window is clipped, drawn or hit.
//!
//! The height is the application's (animate it with an [`Animation`](crate::primitives::animation)
//! or a [`Reveal`](super::Reveal)); the window just follows.
//!
//! ```ignore
//! ClipWindow::new(queue_list).height(self.reveal * list_height).anchor(Anchor::Bottom)
//! ```
//!
//! Without it, the same effect is a non-scrolling `scrollable` with a hidden scrollbar and
//! `anchor_bottom`, which is how this was first written.

use super::Element;
use crate::Theme;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::{Event, Length, Rectangle, Size, Vector, mouse};

/// Which edge of the content stays put as the window changes height.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Anchor {
    /// The top of the content stays at the top of the window; a smaller window cuts the bottom.
    #[default]
    Top,
    /// The bottom of the content stays at the bottom of the window; a smaller window cuts the top.
    Bottom,
}

impl Anchor {
    /// Where the content starts, relative to the window's top, for a window of `window` px over
    /// content of `content` px.
    pub fn content_top(self, window: f32, content: f32) -> f32 {
        match self {
            Self::Top => 0.0,
            Self::Bottom => window - content,
        }
    }
}

/// See the [module](self).
pub struct ClipWindow<'a, M> {
    content: Element<'a, M>,
    height: f32,
    anchor: Anchor,
    width: Length,
}

impl<'a, M> ClipWindow<'a, M> {
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            content: content.into(),
            height: f32::INFINITY,
            anchor: Anchor::Top,
            width: Length::Fill,
        }
    }
    /// The height of the window in px. By default the content's own height (all of it shows).
    pub fn height(mut self, px: f32) -> Self {
        self.height = px.max(0.0);
        self
    }
    /// Which edge of the content stays fixed (default top).
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for ClipWindow<'a, M> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }
    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let max = limits.max();
        // The content gets the width, and as much height as it asks for.
        let child = self.content.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(max.width, f32::INFINITY)),
        );
        let window = self.height.min(child.size().height);
        let top = self.anchor.content_top(window, child.size().height);
        let width = match self.width {
            Length::Shrink => child.size().width,
            _ => max.width.min(child.size().width.max(max.width)),
        };
        layout::Node::with_children(
            Size::new(width, window),
            vec![child.move_to(iced::Point::new(0.0, top))],
        )
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
        let bounds = layout.bounds();
        // The part of the content outside the window is not there: no pointer reaches it.
        let cursor = if cursor.is_over(bounds) {
            cursor
        } else {
            mouse::Cursor::Unavailable
        };
        let view = bounds.intersection(viewport).unwrap_or(bounds);
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            &view,
        );
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();
        if !cursor.is_over(bounds) {
            return mouse::Interaction::None;
        }
        let view = bounds.intersection(viewport).unwrap_or(bounds);
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            &view,
            renderer,
        )
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
        use iced::advanced::Renderer as _;
        let bounds = layout.bounds();
        let Some(view) = bounds.intersection(viewport) else {
            return;
        };
        // A layer clips what the content draws to the window.
        renderer.with_layer(view, |r| {
            self.content.as_widget().draw(
                &tree.children[0],
                r,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                &view,
            );
        });
    }
}

impl<'a, M: 'a> From<ClipWindow<'a, M>> for Element<'a, M> {
    fn from(c: ClipWindow<'a, M>) -> Self {
        Element::new(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_anchor_says_which_edge_stays() {
        // A 100 px window over 300 px of content.
        assert_eq!(Anchor::Top.content_top(100.0, 300.0), 0.0);
        assert_eq!(Anchor::Bottom.content_top(100.0, 300.0), -200.0);
        // A window as tall as the content shows all of it either way.
        assert_eq!(Anchor::Bottom.content_top(300.0, 300.0), 0.0);
        assert_eq!(Anchor::default(), Anchor::Top);
    }

    #[test]
    fn the_height_is_never_negative() {
        let c: ClipWindow<'_, ()> = ClipWindow::new(iced::widget::space()).height(-5.0);
        assert_eq!(c.height, 0.0);
    }
}
