//! One pass-through widget for binds that only watch, decorate or gate their content.
//!
//! A [`Behavior`] adds a little state and a few hooks; [`Wrap`] forwards everything else
//! (children, layout, events, drawing, operations, overlays) to the content once, so a bind
//! does not repeat that plumbing.
use super::focus::{FocusAt, Probe};
use crate::{Element, Theme};
use iced::advanced::{
    Clipboard, Shell,
    layout::{self, Layout},
    overlay, renderer,
    widget::{Operation, Tree, Widget, tree},
};
use iced::{Event, Length, Rectangle, Size, Vector, mouse};

/// The wrapped content, as a hook may inspect it: which of its widgets are focusable and focused,
/// and moving focus among them.
pub(crate) struct Content<'c, 'a, M> {
    element: &'c mut Element<'a, M>,
    tree: &'c mut Tree,
    layout: Layout<'c>,
    renderer: &'c iced::Renderer,
}
impl<M> Content<'_, '_, M> {
    pub fn contains_focus(&mut self) -> bool {
        self.scan(false).focused.is_some()
    }
    /// The focusable widgets inside, in tree order, with their ids.
    pub fn probe(&mut self) -> Probe {
        self.scan(true)
    }
    fn scan(&mut self, ids: bool) -> Probe {
        let mut probe = Probe::new(ids);
        self.element.as_widget_mut().operate(
            self.tree,
            self.layout,
            self.renderer,
            &mut probe,
        );
        probe
    }
    /// Focuses the `index`-th focusable widget and unfocuses the others.
    pub fn focus(&mut self, index: usize) {
        self.element.as_widget_mut().operate(
            self.tree,
            self.layout,
            self.renderer,
            &mut FocusAt::new(index),
        );
    }
}

/// What a wrapper adds to its content. Every hook has a do-nothing default.
pub(crate) trait Behavior<M> {
    type State: 'static;
    fn state(&self) -> Self::State;
    /// Called on every diff, to reset `state` when the wrapper is reused for something new.
    fn diff(&self, _state: &mut Self::State) {}
    /// A hidden wrapper takes no space and ignores events, drawing and operations.
    fn hidden(&self, _state: &Self::State) -> bool {
        false
    }
    /// Runs before the content sees an event. Return `true` to keep the event from it.
    fn before_update(
        &mut self,
        _state: &mut Self::State,
        _event: &Event,
        _shell: &mut Shell<'_, M>,
        _content: &mut Content<'_, '_, M>,
    ) -> bool {
        false
    }
    fn after_update(
        &mut self,
        _state: &mut Self::State,
        _event: &Event,
        _shell: &mut Shell<'_, M>,
        _content: &mut Content<'_, '_, M>,
    ) {
    }
    fn after_operate(&mut self, _state: &mut Self::State, _content: &mut Content<'_, '_, M>) {}
    fn after_draw(
        &self,
        _state: &Self::State,
        _renderer: &mut iced::Renderer,
        _theme: &Theme,
        _layout: Layout<'_>,
    ) {
    }
}

pub(crate) struct Wrap<'a, M, B> {
    pub content: Element<'a, M>,
    pub behavior: B,
}

impl<'a, M: 'a, B: Behavior<M> + 'a> Wrap<'a, M, B> {
    pub fn new(content: impl Into<Element<'a, M>>, behavior: B) -> Self {
        Self {
            content: content.into(),
            behavior,
        }
    }
}

impl<'a, M: 'a, B: Behavior<M> + 'a> From<Wrap<'a, M, B>> for Element<'a, M> {
    fn from(wrap: Wrap<'a, M, B>) -> Self {
        Element::new(wrap)
    }
}

fn hidden<'a, M, B: Behavior<M>>(wrap: &Wrap<'a, M, B>, tree: &Tree) -> bool {
    wrap.behavior.hidden(tree.state.downcast_ref::<B::State>())
}

impl<'a, M: 'a, B: Behavior<M> + 'a> Widget<M, Theme, iced::Renderer> for Wrap<'a, M, B> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<B::State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(self.behavior.state())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        self.behavior.diff(tree.state.downcast_mut::<B::State>());
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
        if hidden(self, tree) {
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
        let Tree {
            state, children, ..
        } = tree;
        let state = state.downcast_mut::<B::State>();
        if self.behavior.hidden(state) {
            return;
        }
        let mut content = Content {
            element: &mut self.content,
            tree: &mut children[0],
            layout,
            renderer,
        };
        if self
            .behavior
            .before_update(state, event, shell, &mut content)
        {
            return;
        }
        content.element.as_widget_mut().update(
            content.tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        self.behavior
            .after_update(state, event, shell, &mut content);
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if hidden(self, tree) {
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
        if hidden(self, tree) {
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
        self.behavior.after_draw(
            tree.state.downcast_ref::<B::State>(),
            renderer,
            theme,
            layout,
        );
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        let Tree {
            state, children, ..
        } = tree;
        let state = state.downcast_mut::<B::State>();
        if self.behavior.hidden(state) {
            return;
        }
        self.content
            .as_widget_mut()
            .operate(&mut children[0], layout, renderer, operation);
        let mut content = Content {
            element: &mut self.content,
            tree: &mut children[0],
            layout,
            renderer,
        };
        self.behavior.after_operate(state, &mut content);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        if hidden(self, tree) {
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
