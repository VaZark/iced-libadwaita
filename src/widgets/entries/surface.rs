//! EntryRow owns one focus outline around its complete control surface.
use crate::{Element, Metrics, Theme};
use iced::advanced::{
    Clipboard, Shell,
    layout::{self, Layout},
    overlay, renderer,
    widget::{Operation, Tree, Widget, tree},
};
use iced::{Event, Length, Rectangle, Size, Vector, mouse};

pub(super) struct EntryRowSurface<'a, M> {
    content: Element<'a, M>,
    ring: bool,
    metrics: Metrics,
}
#[derive(Default)]
struct State {
    focused: bool,
}
#[derive(Default)]
struct Probe(bool);
impl Operation for Probe {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }
    fn focusable(
        &mut self,
        _: Option<&iced::widget::Id>,
        _: Rectangle,
        state: &mut dyn iced::advanced::widget::operation::Focusable,
    ) {
        self.0 |= state.is_focused();
    }
}
impl<'a, M: 'a> EntryRowSurface<'a, M> {
    pub(super) fn new(content: impl Into<Element<'a, M>>, ring: bool, metrics: Metrics) -> Self {
        Self {
            content: content.into(),
            ring,
            metrics,
        }
    }
    fn probe(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &iced::Renderer) -> bool {
        let mut probe = Probe::default();
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut probe);
        probe.0
    }
}
impl<'a, M: 'a> Widget<M, Theme, iced::Renderer> for EntryRowSurface<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
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
        if self.ring {
            let focused = self.probe(tree, layout, renderer);
            let state = tree.state.downcast_mut::<State>();
            if state.focused != focused {
                state.focused = focused;
                shell.request_redraw();
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
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
        if self.ring && tree.state.downcast_ref::<State>().focused {
            use iced::advanced::Renderer as _;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border: iced::Border {
                        color: theme.semantic.accent.bg,
                        width: self.metrics.focus_ring_width,
                        radius: self.metrics.radius_m.into(),
                    },
                    ..Default::default()
                },
                iced::Color::TRANSPARENT,
            );
        }
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
        if self.ring {
            tree.state.downcast_mut::<State>().focused = self.probe(tree, layout, renderer);
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
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, M: 'a> From<EntryRowSurface<'a, M>> for Element<'a, M> {
    fn from(surface: EntryRowSurface<'a, M>) -> Self {
        Element::new(surface)
    }
}
