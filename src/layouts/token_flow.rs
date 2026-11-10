//! Inline token flow, as in a tags entry: pills wrap along the line and the last child, a text
//! field, fills what is left of it.
//!
//! [`flow_with_caret`] draws that field *between* the pills (at a caret position) without
//! moving it in the tree, so its focus, cursor and selection survive edits to the pills. A
//! navigation hook ([`binds::token_nav`](crate::binds::token_nav)) lets the arrow keys at the
//! edges of the field step the caret from group to group.
use crate::Theme;
use crate::binds::token_nav::Navigation;
use iced::advanced::{
    Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer,
    widget::{Operation, Tree},
};
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector};

/// The least room the typing field is given at the end of the line.
const FIELD_WIDTH: f32 = 120.0;

/// Pills in a wrapping line; the last item is the typing field.
pub fn token_flow<'a, M: 'a>(items: Vec<Element<'a, M, Theme>>) -> Element<'a, M, Theme> {
    Element::new(TokenFlow {
        items,
        caret: None,
        compact: FIELD_WIDTH,
        navigation: None,
    })
}

/// Places the last (raw text input) child at the visual insertion point `caret_index`, without
/// reordering its tree or layout index. Out-of-range positions are clamped. While pills follow
/// the field it is `compact` wide, so they stay on its line; at the end it fills the line.
pub fn token_flow_with_caret<'a, M: 'a>(
    items: Vec<Element<'a, M, Theme>>,
    caret_index: usize,
    compact: f32,
    navigation: Navigation<'a, M>,
) -> Element<'a, M, Theme> {
    Element::new(TokenFlow {
        items,
        caret: Some(caret_index),
        compact,
        navigation: Some(navigation),
    })
}

struct TokenFlow<'a, M> {
    items: Vec<Element<'a, M, Theme>>,
    caret: Option<usize>,
    compact: f32,
    navigation: Option<Navigation<'a, M>>,
}

impl<M> Widget<M, Theme, iced::Renderer> for TokenFlow<'_, M> {
    fn children(&self) -> Vec<Tree> {
        self.items.iter().map(Tree::new).collect()
    }
    fn diff(&self, tree: &mut Tree) {
        if self.caret.is_some() && !self.items.is_empty() {
            let input = tree.children.pop();
            tree.diff_children(&self.items[..self.items.len() - 1]);
            let item = self.items.last().unwrap();
            let mut input = input.unwrap_or_else(|| Tree::new(item));
            input.diff(item.as_widget());
            tree.children.push(input);
        } else {
            tree.diff_children(&self.items);
        }
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let width = limits.max().width;
        let mut nodes = vec![layout::Node::new(Size::ZERO); self.items.len()];
        let (mut x, mut y, mut line_height) = (0.0_f32, 0.0_f32, 28.0_f32);
        let last = self.items.len().saturating_sub(1);
        let caret = self.caret.unwrap_or(last).min(last);
        let has_items = !self.items.is_empty();
        let order = (0..caret)
            .chain(std::iter::once(last).filter(|_| has_items))
            .chain(caret..last);
        for index in order {
            let item = &mut self.items[index];
            let state = &mut tree.children[index];
            let room = if caret == last {
                FIELD_WIDTH
            } else {
                self.compact
            };
            if index == last && x > 0.0 && width - x < room.min(width) {
                x = 0.0;
                y += line_height + 4.0;
                line_height = 28.0;
            }
            let available = if index == last && caret == last {
                // At the end the field takes the rest of the line.
                (width - x).max(0.0)
            } else if index == last {
                // Between pills it is only as wide as it needs to be to type in, so the pills
                // after it stay on the line and wrap only where they would not fit.
                (width - x).clamp(0.0, room.min(width))
            } else {
                width
            };
            let child_limits = layout::Limits::new(Size::ZERO, Size::new(available, f32::INFINITY));
            let mut node = item.as_widget_mut().layout(state, renderer, &child_limits);
            if index != last && x > 0.0 && x + node.size().width > width {
                x = 0.0;
                y += line_height + 4.0;
                line_height = 28.0;
            }
            let height = node.size().height;
            node.move_to_mut(Point::new(x, y + (28.0 - height).max(0.0) / 2.0));
            x += node.size().width + 4.0;
            line_height = line_height.max(height);
            nodes[index] = node;
        }
        layout::Node::with_children(Size::new(width, y + line_height), nodes)
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |op| {
            for ((item, state), layout) in self
                .items
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
            {
                item.as_widget_mut().operate(state, layout, renderer, op);
            }
        });
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
        if let Some(message) = self.navigation.as_ref().and_then(|navigate| {
            tree.children
                .last()
                .and_then(|input| navigate(input, event))
        }) {
            shell.publish(message);
            shell.capture_event();
            return;
        }
        for ((item, state), layout) in self
            .items
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            item.as_widget_mut().update(
                state, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
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
        self.items
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((item, state), layout)| {
                item.as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
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
        let Some(viewport) = layout.bounds().intersection(viewport) else {
            return;
        };
        for ((item, state), layout) in self.items.iter().zip(&tree.children).zip(layout.children())
        {
            item.as_widget()
                .draw(state, renderer, theme, style, layout, cursor, &viewport);
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
        iced::advanced::overlay::from_children(
            &mut self.items,
            tree,
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
    use crate::binds::token_nav::{InputState, between_groups};
    use iced::advanced::renderer::Headless;

    fn key_event(key: iced::keyboard::key::Named, modifiers: iced::keyboard::Modifiers) -> Event {
        use iced::keyboard::{Key, Location, key::Physical};
        Event::Keyboard(iced::keyboard::Event::KeyPressed {
            key: Key::Named(key),
            modified_key: Key::Named(key),
            physical_key: Physical::Unidentified(iced::keyboard::key::NativeCode::Unidentified),
            location: Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        })
    }

    fn caret_flow(count: usize, caret: usize) -> Element<'static, (), Theme> {
        let mut items: Vec<Element<'static, (), Theme>> = (0..count)
            .map(|_| iced::widget::space().width(80).height(22).into())
            .collect();
        items.push(
            iced::widget::text_input("", "a\u{301}b")
                .on_input(|_| ())
                .into(),
        );
        token_flow_with_caret(
            items,
            caret,
            120.0,
            crate::binds::token_nav::between_groups("a\u{301}b", Some(()), Some(())),
        )
    }

    #[test]
    fn the_field_is_compact_between_pills_and_fills_the_line_at_the_end() {
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        for (caret, x, y) in [
            (0, 0.0, 0.0),
            (1, 84.0, 0.0),
            (2, 168.0, 0.0),
            (99, 168.0, 0.0),
        ] {
            let mut element = caret_flow(2, caret);
            let mut tree = Tree::new(&element);
            let node = element.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(400.0, 200.0)),
            );
            let input = node.children()[2].bounds();
            if caret >= 2 {
                // at the end the field fills the line
                assert_eq!((input.x, input.y, input.width), (x, y, 400.0 - x));
            } else {
                // between pills it is compact, and the pills after it stay on its line
                assert_eq!((input.x, input.y, input.width), (x, y, 120.0));
                for item in &node.children()[caret..2] {
                    assert!(item.bounds().y < input.y + input.height, "same line");
                    assert!(item.bounds().x >= input.x + input.width);
                }
            }
        }
    }

    #[test]
    fn a_compact_field_leaves_no_gap_between_the_pills() {
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let items: Vec<Element<'static, (), Theme>> = vec![
            iced::widget::space().width(80).height(22).into(),
            iced::widget::space().width(80).height(22).into(),
            iced::widget::text_input("", "").on_input(|_| ()).into(),
        ];
        let mut element =
            token_flow_with_caret(items, 1, 16.0, between_groups("", None::<()>, None));
        let mut tree = Tree::new(&element);
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(400.0, 200.0)),
        );
        let b: Vec<_> = node.children().iter().map(|n| n.bounds()).collect();
        assert_eq!(b[2].width, 16.0);
        assert_eq!(b[1].x, b[2].x + 16.0 + 4.0);
    }

    #[test]
    fn input_focus_and_cursor_survive_group_count_and_caret_changes() {
        let element = caret_flow(2, 1);
        let mut tree = Tree::new(&element);
        let state = tree
            .children
            .last_mut()
            .unwrap()
            .state
            .downcast_mut::<InputState>();
        state.focus();
        state.move_cursor_to(1);
        for count in [4, 0, 1, 3] {
            let element = caret_flow(count, count / 2);
            tree.diff(element.as_widget());
            assert_eq!(tree.children.len(), count + 1);
            let state = tree
                .children
                .last()
                .unwrap()
                .state
                .downcast_ref::<InputState>();
            assert!(state.is_focused());
            assert_eq!(
                state
                    .cursor()
                    .state(&iced::widget::text_input::Value::new("ab")),
                iced::widget::text_input::cursor::State::Index(1)
            );
        }
    }

    #[test]
    fn navigation_publishes_and_captures_without_losing_focus() {
        use iced::keyboard::{Modifiers, key::Named};
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let mut element = token_flow_with_caret(
            vec![iced::widget::text_input("", "ab").on_input(|_| 0).into()],
            0,
            120.0,
            between_groups("ab", Some(1), None),
        );
        let mut tree = Tree::new(&element);
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(400.0, 200.0)),
        );
        tree.children[0].state.downcast_mut::<InputState>().focus();
        tree.children[0]
            .state
            .downcast_mut::<InputState>()
            .move_cursor_to_front();
        for (key, expected, captured) in [
            (Named::ArrowLeft, vec![1], true),
            (Named::ArrowRight, vec![], true),
        ] {
            let mut messages = Vec::new();
            let mut shell = Shell::new(&mut messages);
            element.as_widget_mut().update(
                &mut tree,
                &key_event(key, Modifiers::empty()),
                Layout::new(&node),
                mouse::Cursor::Unavailable,
                &renderer,
                &mut iced::advanced::clipboard::Null,
                &mut shell,
                &Rectangle::with_size(Size::new(400.0, 200.0)),
            );
            assert_eq!(shell.is_event_captured(), captured);
            assert_eq!(messages, expected);
            assert!(
                tree.children[0]
                    .state
                    .downcast_ref::<InputState>()
                    .is_focused()
            );
        }
        // The right arrow was handled by the input itself, not by navigation.
        assert_eq!(
            tree.children[0]
                .state
                .downcast_ref::<InputState>()
                .cursor()
                .state(&iced::widget::text_input::Value::new("ab")),
            iced::widget::text_input::cursor::State::Index(1)
        );
    }

    #[test]
    fn typing_field_fills_the_last_pill_line_and_wraps_when_needed() {
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        for (width, expected_x, expected_y) in [(400.0, 168.0, 0.0), (250.0, 0.0, 32.0)] {
            let mut element = token_flow::<()>(vec![
                iced::widget::space().width(80).height(22).into(),
                iced::widget::space().width(80).height(22).into(),
                iced::widget::space().width(Length::Fill).height(28).into(),
            ]);
            let mut tree = Tree::new(&element);
            let node = element.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, Size::new(width, 200.0)),
            );
            let field = node.children().last().unwrap().bounds();
            assert_eq!(field.x, expected_x);
            assert_eq!(field.y, expected_y);
            assert_eq!(field.width, width - expected_x);
            assert_eq!(node.size().height, expected_y + 28.0);
        }
    }

    #[test]
    fn narrow_fields_wrap_pills_without_overlapping_the_typing_field() {
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let mut element = token_flow::<()>(vec![
            iced::widget::space().width(100).height(22).into(),
            iced::widget::space().width(100).height(22).into(),
            iced::widget::space().width(Length::Fill).height(28).into(),
        ]);
        let mut tree = Tree::new(&element);
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(140.0, 200.0)),
        );
        let children = node.children();
        assert!(children[1].bounds().y >= children[0].bounds().y + children[0].size().height);
        assert!(children[2].bounds().y >= children[1].bounds().y + children[1].size().height);
        assert_eq!(children[2].size().width, 140.0);
    }
}
