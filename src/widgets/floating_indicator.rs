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
//! sets another. Supply [`target_shape`](FloatingIndicator::target_shape) to inherit the
//! target's current style, or [`radius`](FloatingIndicator::radius) for explicit corners.
//! Arbitrary children have no discoverable shape and are left unhighlighted by default.

use crate::Theme;
use crate::color::alpha;
use crate::layouts::animation::{Progress, Spec, Trigger as Motion};
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
    target_shape: Option<Box<dyn Fn(&Theme, usize, Rectangle) -> iced::border::Radius + 'a>>,
    target_bounds: Option<Box<dyn Fn(usize, Layout<'_>) -> Rectangle + 'a>>,
}

impl<'a, M, R: iced::advanced::Renderer> FloatingIndicator<'a, M, R> {
    pub fn new(children: Vec<iced::Element<'a, M, Theme, R>>, active: usize) -> Self {
        Self {
            children,
            active,
            spacing: 0.0,
            radius: None,
            color: None,
            target_shape: None,
            target_bounds: None,
        }
    }
    /// The gap between children.
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = px;
        self
    }
    /// Explicit uniform corners, overriding any target shape resolver.
    /// Without this or a known target shape, no highlight is drawn.
    pub fn radius(mut self, px: f32) -> Self {
        self.radius = Some(px);
        self
    }
    /// Resolves the target's four corners from the current draw theme and animated bounds.
    /// Reuse the target's actual style; arbitrary child shapes cannot be introspected.
    pub fn target_shape(
        mut self,
        shape: impl Fn(&Theme, usize, Rectangle) -> iced::border::Radius + 'a,
    ) -> Self {
        self.target_shape = Some(Box::new(shape));
        self
    }

    pub(crate) fn target_bounds(
        mut self,
        bounds: impl Fn(usize, Layout<'_>) -> Rectangle + 'a,
    ) -> Self {
        self.target_bounds = Some(Box::new(bounds));
        self
    }

    fn target(&self, layout: Layout<'_>) -> Option<Rectangle> {
        let child = layout.children().nth(self.active)?;
        Some(
            self.target_bounds
                .as_ref()
                .map_or_else(|| child.bounds(), |bounds| bounds(self.active, child)),
        )
    }

    fn highlight(&self, theme: &Theme, state: &State, layout: Layout<'_>) -> Option<Quad> {
        let target = self.target(layout)?;
        let bounds = if state.placed {
            Rectangle::new(
                Point::new(
                    layout.bounds().x + state.x.shown,
                    layout.bounds().y + state.y.shown,
                ),
                Size::new(state.width.shown, state.height.shown),
            )
        } else {
            target
        };
        let radius = match self.radius {
            Some(radius) => radius.into(),
            None => self.target_shape.as_ref()?(theme, self.active, bounds),
        };
        (bounds.width > 0.0 && bounds.height > 0.0).then_some(Quad {
            bounds,
            border: Border::default().rounded(radius),
            shadow: Shadow::default(),
            snap: false,
        })
    }

    /// The highlight's color (default: the window foreground at 10%).
    pub fn color(mut self, c: Color) -> Self {
        self.color = Some(c);
        self
    }
}

struct State {
    /// The full target rectangle, relative to the widget.
    x: Progress,
    y: Progress,
    width: Progress,
    height: Progress,
    /// Whether the highlight has been placed yet (the first time it appears, it does not slide).
    placed: bool,
}

impl State {
    fn drive(
        &mut self,
        target: Rectangle,
        origin: Point,
        now: std::time::Instant,
        spec: Spec,
    ) -> bool {
        let values = [
            target.x - origin.x,
            target.y - origin.y,
            target.width,
            target.height,
        ];
        let mut moving = false;
        for (progress, value) in [&mut self.x, &mut self.y, &mut self.width, &mut self.height]
            .into_iter()
            .zip(values)
        {
            if self.placed {
                moving |= progress.drive(value, now, spec);
            } else {
                progress.set_immediate(value);
            }
        }
        self.placed = true;
        moving
    }
}

impl Default for State {
    fn default() -> Self {
        Self {
            x: Progress::new(0.0),
            y: Progress::new(0.0),
            width: Progress::new(0.0),
            height: Progress::new(0.0),
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
            && let Some(target) = self.target(layout)
        {
            let state = tree.state.downcast_mut::<State>();
            if state.drive(
                target,
                layout.bounds().position(),
                *now,
                Motion::Selection.spec(Spec::STYLE),
            ) {
                shell.request_redraw();
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
        let state = tree.state.downcast_ref::<State>();
        if let Some(quad) = self.highlight(theme, state, layout) {
            renderer.fill_quad(
                quad,
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
    fn unknown_shape_still_forwards_events_operations_mouse_and_overlays() {
        use std::{cell::Cell, rc::Rc};
        struct Probe(Rc<Cell<u8>>);
        impl Widget<(), Theme, ()> for Probe {
            fn size(&self) -> Size<Length> {
                Size::new(Length::Shrink, Length::Shrink)
            }
            fn layout(&mut self, _: &mut Tree, _: &(), _: &layout::Limits) -> layout::Node {
                layout::Node::new(Size::new(40.0, 20.0))
            }
            fn draw(
                &self,
                _: &Tree,
                _: &mut (),
                _: &Theme,
                _: &renderer::Style,
                _: Layout<'_>,
                _: mouse::Cursor,
                _: &Rectangle,
            ) {
                self.0.set(self.0.get() | 1);
            }
            fn update(
                &mut self,
                _: &mut Tree,
                _: &Event,
                _: Layout<'_>,
                _: mouse::Cursor,
                _: &(),
                _: &mut dyn Clipboard,
                shell: &mut Shell<'_, ()>,
                _: &Rectangle,
            ) {
                self.0.set(self.0.get() | 2);
                shell.publish(());
            }
            fn operate(&mut self, _: &mut Tree, _: Layout<'_>, _: &(), _: &mut dyn Operation) {
                self.0.set(self.0.get() | 4);
            }
            fn mouse_interaction(
                &self,
                _: &Tree,
                _: Layout<'_>,
                _: mouse::Cursor,
                _: &Rectangle,
                _: &(),
            ) -> mouse::Interaction {
                self.0.set(self.0.get() | 8);
                mouse::Interaction::Pointer
            }
            fn overlay<'b>(
                &'b mut self,
                _: &'b mut Tree,
                _: Layout<'b>,
                _: &(),
                _: &Rectangle,
                translation: Vector,
            ) -> Option<overlay::Element<'b, (), Theme, ()>> {
                assert_eq!(translation, Vector::new(3.0, 7.0));
                self.0.set(self.0.get() | 16);
                None
            }
        }
        struct Noop;
        impl Operation for Noop {
            fn traverse(&mut self, _: &mut dyn FnMut(&mut dyn Operation)) {}
        }
        let calls = Rc::new(Cell::new(0));
        let mut element: E =
            FloatingIndicator::new(vec![iced::Element::new(Probe(calls.clone()))], 0).into();
        let mut tree = Tree::new(&element);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(100.0, 100.0));
        let node = element.as_widget_mut().layout(
            &mut tree,
            &(),
            &layout::Limits::new(Size::ZERO, viewport.size()),
        );
        let layout = Layout::new(&node);
        let mut messages = vec![];
        element.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(mouse::Event::CursorMoved {
                position: Point::new(2.0, 2.0),
            }),
            layout,
            mouse::Cursor::Unavailable,
            &(),
            &mut iced::advanced::clipboard::Null,
            &mut Shell::new(&mut messages),
            &viewport,
        );
        assert_eq!(messages, vec![()]);
        element.as_widget().draw(
            &tree,
            &mut (),
            &Theme::light(),
            &renderer::Style::default(),
            layout,
            mouse::Cursor::Unavailable,
            &viewport,
        );
        element
            .as_widget_mut()
            .operate(&mut tree, layout, &(), &mut Noop);
        assert_eq!(
            element.as_widget().mouse_interaction(
                &tree,
                layout,
                mouse::Cursor::Unavailable,
                &viewport,
                &()
            ),
            mouse::Interaction::Pointer
        );
        assert!(
            element
                .as_widget_mut()
                .overlay(&mut tree, layout, &(), &viewport, Vector::new(3.0, 7.0))
                .is_none()
        );
        assert_eq!(calls.get(), 31);
    }

    fn unequal_targets() -> layout::Node {
        layout::Node::with_children(
            Size::new(140.0, 60.0),
            vec![
                layout::Node::new(Size::new(40.0, 20.0)).move_to(Point::new(0.0, 12.0)),
                layout::Node::new(Size::new(80.0, 60.0)).move_to(Point::new(60.0, 0.0)),
            ],
        )
        .move_to(Point::new(15.0, 25.0))
    }

    #[test]
    fn full_selected_rectangle_not_row_height_before_and_after_redraw() {
        let node = unequal_targets();
        let layout = Layout::new(&node);
        let indicator = FloatingIndicator::<(), ()>::new(vec![], 0).radius(0.0);
        let theme = Theme::light();
        let mut state = State::default();
        let target = layout.children().next().unwrap().bounds();
        assert_eq!(
            indicator.highlight(&theme, &state, layout).unwrap().bounds,
            target
        );
        state.drive(
            target,
            layout.bounds().position(),
            std::time::Instant::now(),
            Spec::STYLE,
        );
        assert_eq!(
            indicator.highlight(&theme, &state, layout).unwrap().bounds,
            target
        );
        assert_eq!(
            target,
            Rectangle::new(Point::new(15.0, 37.0), Size::new(40.0, 20.0))
        );
    }

    #[test]
    fn current_theme_square_pill_asymmetric_and_explicit_radius() {
        let node = unequal_targets();
        let layout = Layout::new(&node);
        let indicator =
            FloatingIndicator::<(), ()>::new(vec![], 0).target_shape(|theme, _, bounds| {
                if theme.metrics.radius_m == 0.0 {
                    0.0.into()
                } else if theme.metrics.radius_m == 1.0 {
                    (bounds.height / 2.0).into()
                } else {
                    iced::border::Radius {
                        top_left: theme.metrics.radius_m,
                        top_right: 0.0,
                        bottom_right: 3.0,
                        bottom_left: 7.0,
                    }
                }
            });
        let state = State::default();
        let mut theme = Theme::light();
        for (metric, expected) in [
            (0.0, 0.0.into()),
            (1.0, 10.0.into()),
            (
                9.0,
                iced::border::Radius {
                    top_left: 9.0,
                    top_right: 0.0,
                    bottom_right: 3.0,
                    bottom_left: 7.0,
                },
            ),
        ] {
            theme.metrics.radius_m = metric;
            assert_eq!(
                indicator
                    .highlight(&theme, &state, layout)
                    .unwrap()
                    .border
                    .radius,
                expected
            );
        }
        let explicit = indicator.radius(4.0);
        assert_eq!(
            explicit
                .highlight(&theme, &state, layout)
                .unwrap()
                .border
                .radius,
            4.0.into()
        );
        let unknown = FloatingIndicator::<(), ()>::new(vec![], 0);
        assert!(unknown.highlight(&theme, &state, layout).is_none());
        let missing = FloatingIndicator::<(), ()>::new(vec![], 9).radius(4.0);
        assert!(missing.highlight(&theme, &state, layout).is_none());
    }

    #[test]
    fn intermediate_animation_interpolates_every_rectangle_dimension() {
        use crate::layouts::animation::Easing;
        use std::time::{Duration, Instant};
        let node = unequal_targets();
        let layout = Layout::new(&node);
        let mut indicator = FloatingIndicator::<(), ()>::new(vec![], 0)
            .target_shape(|_, _, bounds| (bounds.height / 2.0).into());
        let mut state = State::default();
        let now = Instant::now();
        let spec = Spec::Timed(Duration::from_millis(100), Easing::Linear);
        state.drive(
            indicator.target(layout).unwrap(),
            layout.bounds().position(),
            now,
            spec,
        );
        indicator.active = 1;
        let target = indicator.target(layout).unwrap();
        state.drive(target, layout.bounds().position(), now, spec);
        state.drive(
            target,
            layout.bounds().position(),
            now + Duration::from_millis(50),
            spec,
        );
        let quad = indicator
            .highlight(&Theme::light(), &state, layout)
            .unwrap();
        assert_eq!(
            quad.bounds,
            Rectangle::new(Point::new(45.0, 31.0), Size::new(60.0, 40.0))
        );
        assert_eq!(quad.border.radius, 20.0.into());
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
