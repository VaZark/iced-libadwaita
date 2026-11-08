//! Size-aware content that survives animation-only relayouts.
//!
//! Rebuilding interactive descendants after their redraw update loses widget-local
//! presentation state: native buttons then draw as disabled until the next redraw.
//! Build once per view and available size, while still laying out the child each time.
use crate::{Element, Theme};
use iced::advanced::{
    Clipboard, Shell,
    layout::{self, Layout},
    overlay, renderer,
    widget::{Operation, Tree, Widget},
};
use iced::{Event, Length, Rectangle, Size, Vector, mouse};

pub struct Responsive<'a, M> {
    view: Box<dyn Fn(Size) -> Element<'a, M> + 'a>,
    content: Element<'a, M>,
    available: Option<Size>,
    width: Length,
    height: Length,
}

pub fn responsive<'a, M: 'a>(view: impl Fn(Size) -> Element<'a, M> + 'a) -> Responsive<'a, M> {
    Responsive::new(view)
}
impl<'a, M: 'a> Responsive<'a, M> {
    pub fn new(view: impl Fn(Size) -> Element<'a, M> + 'a) -> Self {
        Self {
            view: Box::new(view),
            content: iced::widget::space().into(),
            available: None,
            width: Length::Fill,
            height: Length::Fill,
        }
    }
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}
impl<'a, M: 'a> Widget<M, Theme, iced::Renderer> for Responsive<'a, M> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        // New views build and diff during their first layout. A cached instance
        // can also be attached to a new widget tree without losing its child.
        if self.available.is_some() {
            tree.diff_children(std::slice::from_ref(&self.content));
        }
    }
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width).height(self.height);
        let available = limits.max();
        if self.available != Some(available) {
            self.content = (self.view)(available);
            tree.diff_children(std::slice::from_ref(&self.content));
            self.available = Some(available);
        }
        let child =
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &limits.loose());
        let size = limits.resolve(self.width, self.height, child.size());
        layout::Node::with_children(size, vec![child])
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
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
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
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
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
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
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
            .operate(&mut tree.children[0], layout, renderer, operation);
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
            layout.children().next().unwrap(),
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, M: 'a> From<Responsive<'a, M>> for Element<'a, M> {
    fn from(responsive: Responsive<'a, M>) -> Self {
        Element::new(responsive)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::{Renderer, renderer::Headless};
    use std::{cell::Cell, rc::Rc};

    #[test]
    fn relayout_keeps_button_and_icon_foregrounds_in_both_schemes() {
        let mut renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let bounds = Rectangle::new(iced::Point::ORIGIN, Size::new(100.0, 100.0));
        let limits = layout::Limits::new(Size::ZERO, bounds.size());
        for theme in [Theme::light(), Theme::dark()] {
            let builds = Rc::new(Cell::new(0));
            let count = builds.clone();
            let mut widget = responsive(move |_| {
                count.set(count.get() + 1);
                iced::widget::button(crate::Icon::Delete.view(24.0))
                    .on_press(())
                    .style(crate::styles::button::flat)
                    .into()
            });
            let mut tree = Tree::empty();
            let node = widget.layout(&mut tree, &renderer, &limits);
            widget.update(
                &mut tree,
                &Event::Window(iced::window::Event::RedrawRequested(
                    std::time::Instant::now(),
                )),
                Layout::new(&node),
                mouse::Cursor::Unavailable,
                &renderer,
                &mut iced::advanced::clipboard::Null,
                &mut Shell::new(&mut vec![]),
                &bounds,
            );
            let mut capture = |widget: &Responsive<'_, ()>, tree: &Tree, node: &layout::Node| {
                renderer.reset(bounds);
                widget.draw(
                    tree,
                    &mut renderer,
                    &theme,
                    &renderer::Style {
                        text_color: theme.semantic.window.fg,
                    },
                    Layout::new(node),
                    mouse::Cursor::Unavailable,
                    &bounds,
                );
                renderer.screenshot(Size::new(100, 100), 1.0, theme.semantic.window.bg)
            };
            let before = capture(&widget, &tree, &node);
            drop(capture);
            let node = widget.layout(&mut tree, &renderer, &limits);
            renderer.reset(bounds);
            widget.draw(
                &tree,
                &mut renderer,
                &theme,
                &renderer::Style {
                    text_color: theme.semantic.window.fg,
                },
                Layout::new(&node),
                mouse::Cursor::Unavailable,
                &bounds,
            );
            let after = renderer.screenshot(Size::new(100, 100), 1.0, theme.semantic.window.bg);
            assert_eq!(
                before, after,
                "a relayout must not dim an enabled button's icon"
            );
            assert_eq!(builds.get(), 1);
            widget.layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(120.0, 100.0)),
            );
            assert_eq!(
                builds.get(),
                2,
                "resizing still rebuilds size-aware content"
            );
        }
    }
}
