//! The widget that listens for keyboard shortcuts.
//!
//! [`shortcuts`] wraps any content and, while it is in the view, publishes the message bound to
//! a key press ([`Shortcuts`]). It needs no subscription and no `update` arm in the app: the
//! message just arrives. A focused widget has precedence: when a text input (or an open popover)
//! handled the key, the shortcut does not fire.

use crate::shortcut::{KeyPress, Shortcuts};
use crate::Theme;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{tree, Operation, Tree, Widget};
use iced::advanced::{overlay, Clipboard, Shell};
use iced::{keyboard, mouse, Element, Event, Length, Rectangle, Size, Vector};

/// `content` with `shortcuts` active.
pub fn shortcuts<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M, Theme>>,
    shortcuts: Shortcuts<M>,
) -> Element<'a, M, Theme> {
    Element::new(ShortcutScope { content: content.into(), shortcuts })
}

struct ShortcutScope<'a, M> {
    content: Element<'a, M, Theme>,
    shortcuts: Shortcuts<M>,
}

impl<'a, M: Clone> Widget<M, Theme, iced::Renderer> for ShortcutScope<'a, M> {
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }
    fn state(&self) -> tree::State {
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
    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
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
        self.content.as_widget_mut().update(tree, event, layout, cursor, renderer, clipboard, shell, viewport);
        if let Event::Keyboard(keyboard::Event::KeyPressed { key, modified_key, physical_key, modifiers, .. }) = event
            && !shell.is_event_captured()
        {
            let press = KeyPress {
                key: key.clone(),
                modified_key: modified_key.clone(),
                physical_key: *physical_key,
                modifiers: *modifiers,
                captured: false,
            };
            if let Some(message) = self.shortcuts.handle(&press) {
                shell.publish(message);
                shell.capture_event();
            }
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
        self.content.as_widget().mouse_interaction(tree, layout, cursor, viewport, renderer)
    }
    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &iced::Renderer, operation: &mut dyn Operation) {
        self.content.as_widget_mut().operate(tree, layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(tree, layout, renderer, viewport, translation)
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
        self.content.as_widget().draw(tree, renderer, theme, style, layout, cursor, viewport);
    }
}
