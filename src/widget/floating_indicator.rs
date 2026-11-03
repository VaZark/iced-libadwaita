//! `FloatingIndicator`: a row of children with a highlight that slides behind the active one,
//! after Mantine's [FloatingIndicator](https://mantine.dev/core/floating-indicator/) (its tabs'
//! "custom variants"). The children are the app's own widgets, usually flat buttons; the
//! widget only places them in a row and draws the highlight, which eases to the active child's
//! place with the theme's `selection` motion (instant under reduced motion).
//!
//! ```ignore
//! FloatingIndicator::new(
//!     vec![
//!         flat_button(label("Preview")).on_press(Msg::Tab(0)).into(),
//!         flat_button(label("Code")).on_press(Msg::Tab(1)).into(),
//!     ],
//!     self.tab,                 // controlled: the app owns the active index
//! )
//! ```
//!
//! The highlight is the window foreground at 10% by default; [`color`](FloatingIndicator::color)
//! sets another and [`radius`](FloatingIndicator::radius) its corners.

use crate::Theme;
use crate::color::alpha;
use crate::primitives::animation::{Progress, Spec, Trigger as Motion};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::{Border, Color, Event, Length, Point, Rectangle, Shadow, Size, Vector, mouse};

/// See the [module](self).
pub struct FloatingIndicator<'a, M, R = iced::Renderer> {
    children: Vec<iced::Element<'a, M, Theme, R>>,
    active: usize,
    spacing: f32,
    radius: Option<f32>,
    color: Option<Color>,
}

impl<'a, M, R: iced::advanced::Renderer> FloatingIndicator<'a, M, R> {
    pub fn new(children: Vec<iced::Element<'a, M, Theme, R>>, active: usize) -> Self {
        Self {
            children,
            active,
            spacing: 0.0,
            radius: None,
            color: None,
        }
    }
    /// The gap between children.
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = px;
        self
    }
    /// The highlight's corner radius (default: the theme's control radius).
    pub fn radius(mut self, px: f32) -> Self {
        self.radius = Some(px);
        self
    }
    /// The highlight's color (default: the window foreground at 10%).
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}

struct State {
    /// The highlight's left edge and width, relative to the widget.
    x: Progress,
    width: Progress,
    /// Whether the highlight has been placed yet (the first time it appears, it does not slide).
    placed: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            x: Progress::new(0.0),
            width: Progress::new(0.0),
            placed: false,
        }
    }
}

impl<'a, M: 'a, R: iced::advanced::Renderer + 'a> Widget<M, Theme, R>
    for FloatingIndicator<'a, M, R>
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children);
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &R, limits: &layout::Limits) -> layout::Node {
        let child_limits =
            layout::Limits::new(Size::ZERO, Size::new(f32::INFINITY, limits.max().height));
        let mut x = 0.0;
        let mut height: f32 = 0.0;
        let mut nodes = Vec::with_capacity(self.children.len());
        for (i, (child, t)) in self.children.iter_mut().zip(&mut tree.children).enumerate() {
            if i > 0 {
                x += self.spacing;
            }
            let node = child
                .as_widget_mut()
                .layout(t, renderer, &child_limits)
                .move_to(Point::new(x, 0.0));
            x += node.size().width;
            height = height.max(node.size().height);
            nodes.push(node);
        }
        layout::Node::with_children(Size::new(x, height), nodes)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &R,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        // The highlight eases toward the active child.
        if let Event::Window(iced::window::Event::RedrawRequested(now)) = event
            && let Some(target) = layout.children().nth(self.active)
        {
            let origin = layout.bounds().x;
            let (tx, tw) = (target.bounds().x - origin, target.bounds().width);
            let state = tree.state.downcast_mut::<State>();
            if state.placed {
                let spec = Motion::Selection.spec(Spec::STYLE);
                let moving = state.x.drive(tx, *now, spec) | state.width.drive(tw, *now, spec);
                if moving {
                    shell.request_redraw();
                }
            } else {
                state.x.set_immediate(tx);
                state.width.set_immediate(tw);
                state.placed = true;
            }
        }
        for ((child, t), l) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child
                .as_widget_mut()
                .update(t, event, l, cursor, renderer, clipboard, shell, viewport);
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &R,
    ) -> mouse::Interaction {
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((child, t), l)| {
                child
                    .as_widget()
                    .mouse_interaction(t, l, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut R,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let b = layout.bounds();
        let state = tree.state.downcast_ref::<State>();
        let (x, w) = if state.placed {
            (state.x.shown, state.width.shown)
        } else {
            // Before the first redraw: straight under the active child.
            layout
                .children()
                .nth(self.active)
                .map_or((0.0, 0.0), |c| (c.bounds().x - b.x, c.bounds().width))
        };
        if w > 0.0 {
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle::new(Point::new(b.x + x, b.y), Size::new(w, b.height)),
                    border: Border::default()
                        .rounded(self.radius.unwrap_or(theme.metrics.radius_m)),
                    shadow: Shadow::default(),
                    snap: false,
                },
                self.color
                    .unwrap_or_else(|| alpha(theme.semantic.window.fg, 0.10)),
            );
        }
        for ((child, t), l) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
        {
            child
                .as_widget()
                .draw(t, renderer, theme, style, l, cursor, viewport);
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &R,
        operation: &mut dyn Operation,
    ) {
        for ((child, t), l) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().operate(t, l, renderer, operation);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &R,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, R>> {
        let overlays: Vec<_> = self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|((child, t), l)| {
                child
                    .as_widget_mut()
                    .overlay(t, l, renderer, viewport, translation)
            })
            .collect();
        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

impl<'a, M: 'a, R: iced::advanced::Renderer + 'a> From<FloatingIndicator<'a, M, R>>
    for iced::Element<'a, M, Theme, R>
{
    fn from(f: FloatingIndicator<'a, M, R>) -> Self {
        iced::Element::new(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::widget::Tree;
    use iced::widget::space;

    type E = iced::Element<'static, (), Theme, ()>;

    fn row(widths: &[f32], spacing: f32) -> (E, layout::Node) {
        let children: Vec<E> = widths
            .iter()
            .map(|w| space().width(*w).height(20).into())
            .collect();
        let mut el: E = FloatingIndicator::<(), ()>::new(children, 0)
            .spacing(spacing)
            .into();
        let mut tree = Tree::new(&el);
        let node = el.as_widget_mut().layout(
            &mut tree,
            &(),
            &layout::Limits::new(Size::ZERO, Size::new(500.0, 100.0)),
        );
        (el, node)
    }

    #[test]
    fn children_sit_in_a_row_with_the_spacing() {
        let (_, node) = row(&[40.0, 60.0, 30.0], 8.0);
        assert_eq!(node.size(), Size::new(40.0 + 60.0 + 30.0 + 16.0, 20.0));
        let xs: Vec<f32> = Layout::new(&node)
            .children()
            .map(|c| c.bounds().x)
            .collect();
        assert_eq!(xs, vec![0.0, 48.0, 116.0]);
    }
}
