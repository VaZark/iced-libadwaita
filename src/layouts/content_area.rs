//! `content_area`: marks the region of the window that is the page's content (the pane beside
//! the sidebar, between the bars), so a [`Popover`](crate::widgets::Popover) opened from inside it stays
//! inside it instead of sliding under the sidebar or a bar.
//!
//! The widget changes nothing about layout or drawing. While it collects the overlays of what it
//! wraps, it records its own bounds, and a popover being created there reads them. A popover
//! whose trigger is outside every content area (a header bar button, a sidebar row) simply uses
//! the window. [`PopoverBounds::Window`](crate::widgets::PopoverBounds::Window) opts a popover out.
//!
//! The library's own panes are already marked ([`ToolbarView`](crate::widgets::ToolbarView)'s content,
//! the content of [`NavigationSplitView`](crate::widgets::NavigationSplitView) and
//! [`OverlaySplitView`](crate::widgets::OverlaySplitView)); wrap your own content pane with
//! [`content_area`] when you lay a window out yourself.

use crate::Element;
use crate::Theme;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::{Event, Length, Rectangle, Size, Vector, mouse};
use std::cell::Cell;

thread_local! {
    /// The content area the overlays being collected right now belong to.
    static AREA: Cell<Option<Rectangle>> = const { Cell::new(None) };
}

/// The content area of the overlay being created, in window coordinates, if any.
pub(crate) fn current() -> Option<Rectangle> {
    AREA.with(Cell::get)
}

/// See the [module](self).
pub struct ContentArea<'a, M> {
    content: Element<'a, M>,
}

/// Marks `content` as a content area.
pub fn content_area<'a, M>(content: impl Into<Element<'a, M>>) -> ContentArea<'a, M> {
    ContentArea {
        content: content.into(),
    }
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for ContentArea<'a, M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        self.content.as_widget().tag()
    }
    fn state(&self) -> iced::advanced::widget::tree::State {
        self.content.as_widget().state()
    }
    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }
    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
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
        self.content.as_widget_mut().layout(tree, renderer, limits)
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
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
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
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
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
        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        let b = layout.bounds();
        let area = Rectangle {
            x: b.x + translation.x,
            y: b.y + translation.y,
            ..b
        };
        // Popovers are created inside this call: they read the area while it is set.
        let outer = AREA.with(|a| a.replace(Some(area)));
        let overlay =
            self.content
                .as_widget_mut()
                .overlay(tree, layout, renderer, viewport, translation);
        AREA.with(|a| a.set(outer));
        overlay
    }
}

impl<'a, M: 'a> From<ContentArea<'a, M>> for Element<'a, M> {
    fn from(c: ContentArea<'a, M>) -> Self {
        iced::Element::new(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn there_is_no_area_outside_a_content_area() {
        assert_eq!(current(), None);
    }
}
