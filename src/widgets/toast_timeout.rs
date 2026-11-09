//! Widget-owned toast expiry: no application subscription is needed.
use crate::{Element, Theme};
use iced::advanced::{
    Clipboard, Shell,
    layout::{self, Layout},
    overlay, renderer,
    widget::{Operation, Tree, Widget, tree},
};
use iced::{Event, Length, Rectangle, Size, Vector, mouse};
use std::time::{Duration, Instant};

pub(super) struct Timeout<'a, M> {
    pub content: Element<'a, M>,
    pub key: (String, u32),
    pub on_dismiss: Option<M>,
}
struct State {
    key: (String, u32),
    deadline: Option<Instant>,
    fired: bool,
}
impl State {
    fn new(key: (String, u32)) -> Self {
        Self {
            key,
            deadline: None,
            fired: false,
        }
    }
}
impl<'a, M: Clone + 'a> Widget<M, Theme, iced::Renderer> for Timeout<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.key.clone()))
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State>();
        if state.key != self.key {
            *state = State::new(self.key.clone());
        }
        tree.diff_children(&[self.content.as_widget()]);
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
        if tree.state.downcast_ref::<State>().fired {
            return layout::Node::new(Size::ZERO);
        }
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
        let state = tree.state.downcast_mut::<State>();
        if state.fired {
            return;
        }
        if self.key.1 > 0 {
            let now = match event {
                Event::Window(iced::window::Event::RedrawRequested(now)) => *now,
                _ => Instant::now(),
            };
            let deadline = *state
                .deadline
                .get_or_insert_with(|| now + Duration::from_millis(self.key.1 as u64));
            if now >= deadline {
                state.fired = true;
                if let Some(message) = &self.on_dismiss {
                    shell.publish(message.clone());
                }
                shell.invalidate_layout();
                shell.request_redraw();
                return;
            }
            shell.request_redraw_at(deadline);
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
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
        if tree.state.downcast_ref::<State>().fired {
            return mouse::Interaction::None;
        }
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
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
        if tree.state.downcast_ref::<State>().fired {
            return;
        }
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
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
        if tree.state.downcast_ref::<State>().fired {
            return;
        }
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
        if tree.state.downcast_ref::<State>().fired {
            return None;
        }
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::{clipboard, renderer::Headless};

    #[test]
    fn expiry_is_exact_once_and_can_be_disabled_or_restarted() {
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let limits = layout::Limits::new(Size::ZERO, Size::new(400.0, 200.0));
        let viewport = Rectangle::new(iced::Point::ORIGIN, Size::new(400.0, 200.0));
        for timeout in [0u32, 75, 3000] {
            for dismiss in [None, Some(1u8)] {
                let mut widget = Timeout {
                    content: iced::widget::space().width(100).height(30).into(),
                    key: ("Toast".into(), timeout),
                    on_dismiss: dismiss,
                };
                let mut tree = Tree::new(&widget as &dyn Widget<_, _, _>);
                let node = widget.layout(&mut tree, &renderer, &limits);
                let start = Instant::now();
                let mut messages = vec![];
                for elapsed in [
                    0,
                    timeout.saturating_sub(1),
                    timeout,
                    timeout.saturating_add(100),
                ] {
                    widget.update(
                        &mut tree,
                        &Event::Window(iced::window::Event::RedrawRequested(
                            start + Duration::from_millis(elapsed as u64),
                        )),
                        Layout::new(&node),
                        mouse::Cursor::Unavailable,
                        &renderer,
                        &mut clipboard::Null,
                        &mut Shell::new(&mut messages),
                        &viewport,
                    );
                    assert_eq!(
                        tree.state.downcast_ref::<State>().fired,
                        timeout > 0 && elapsed >= timeout
                    );
                }
                assert_eq!(
                    messages,
                    if timeout > 0 {
                        dismiss.into_iter().collect()
                    } else {
                        vec![]
                    }
                );
                if timeout > 0 {
                    assert_eq!(
                        widget.layout(&mut tree, &renderer, &limits).size(),
                        Size::ZERO
                    );
                    widget.key.0 = "Replacement".into();
                    widget.diff(&mut tree);
                    assert!(!tree.state.downcast_ref::<State>().fired);
                    assert!(tree.state.downcast_ref::<State>().deadline.is_none());
                    assert_eq!(
                        widget.layout(&mut tree, &renderer, &limits).size(),
                        Size::new(100.0, 30.0)
                    );
                }
            }
        }
    }
}
