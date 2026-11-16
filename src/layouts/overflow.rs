//! Explicit natural-content / viewport negotiation.
//!
//! Like CSS `overflow`, [`OverflowMode::Grow`] and [`OverflowMode::Scroll`] choose
//! whether excess content contributes its natural size or lives in a viewport.
//! [`OverflowMode::ScaleDown`] instead follows `object-fit: scale-down`: contain
//! the measured content, preserve its aspect ratio, and never enlarge it.
//!
//! Measurement is unbounded with iced's fluid-length compression enabled, so
//! intrinsic text (including font growth) contributes to the result. A scene's
//! caller-owned minimum supplies geometry for otherwise dimensionless fill
//! content. This is not a way to discover content deliberately clipped or given
//! fixed dimensions by a descendant.

use crate::{Element, Theme};
use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{Operation, Tree, Widget, operation};
use iced::advanced::{Clipboard, Renderer as _, Shell, overlay, renderer};
use iced::{Event, Length, Point, Rectangle, Size, Transformation, Vector, mouse, touch};
use std::any::Any;

/// Policy for content larger than its allocated viewport.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum OverflowMode {
    /// Report natural size, even beyond finite parent limits. A parent may still
    /// clip or reject that request; resizing an OS window is the application's job.
    #[default]
    Grow,
    /// Native iced scrolling, independently on both axes.
    Scroll,
    /// Uniformly contain the content in the viewport, without upscaling.
    ScaleDown,
}

/// A reusable overflow-policy builder. Geometry is supplied by the caller.
///
/// The viewport defaults to shrink sizing, bounded by the parent's limits.
/// Content is top-left aligned. Changing between Scroll and another mode changes
/// the native widget tree and may reset descendant widget-local state.
pub struct Overflow<'a, M> {
    content: Element<'a, M>,
    mode: OverflowMode,
    minimum: Size,
    width: Length,
    height: Length,
}

/// Wrap content with an explicit overflow policy (Grow by default).
pub fn overflow<'a, M>(content: impl Into<Element<'a, M>>) -> Overflow<'a, M> {
    Overflow::new(content)
}

impl<'a, M> Overflow<'a, M> {
    /// Create a natural-size, Grow wrapper.
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            content: content.into(),
            mode: OverflowMode::Grow,
            minimum: Size::ZERO,
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    /// Choose how natural content negotiates with its viewport.
    pub fn mode(mut self, mode: OverflowMode) -> Self {
        self.mode = mode;
        self
    }

    /// Caller/config-owned minimum natural scene size. Intrinsic content can
    /// exceed it. Negative and non-finite dimensions become zero.
    pub fn min_size(mut self, size: Size) -> Self {
        fn dimension(value: f32) -> f32 {
            if value.is_finite() {
                value.max(0.0)
            } else {
                0.0
            }
        }
        self.minimum = Size::new(dimension(size.width), dimension(size.height));
        self
    }

    /// Alias for [`Self::min_size`], useful for fixed-scene designs. This is a
    /// natural-size floor, not a fixed viewport or an upper bound on font growth.
    pub fn content_size(self, size: Size) -> Self {
        self.min_size(size)
    }

    /// Viewport width. Grow treats this as a request, never a natural-size cap.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Viewport height. Grow treats this as a request, never a natural-size cap.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
}

impl<'a, M: 'a> From<Overflow<'a, M>> for Element<'a, M> {
    fn from(value: Overflow<'a, M>) -> Self {
        let natural = Natural {
            content: value.content,
            minimum: value.minimum,
            width: value.width,
            height: value.height,
            scale_down: value.mode == OverflowMode::ScaleDown,
        };
        if value.mode == OverflowMode::Scroll {
            // Use the library's config-owned scrollbar geometry, not native
            // iced presentation defaults. The native widget owns all scrolling.
            crate::layouts::scrolled_window_both(Element::new(Natural {
                width: Length::Shrink,
                height: Length::Shrink,
                ..natural
            }))
            .width(value.width)
            .height(value.height)
            .into()
        } else {
            Element::new(natural)
        }
    }
}

struct Natural<'a, M> {
    content: Element<'a, M>,
    minimum: Size,
    width: Length,
    height: Length,
    scale_down: bool,
}

fn factor(content: Size, viewport: Size) -> f32 {
    let ratio = |natural: f32, available: f32| {
        if natural > 0.0 {
            available / natural
        } else {
            1.0
        }
    };
    ratio(content.width, viewport.width)
        .min(ratio(content.height, viewport.height))
        .clamp(0.0, 1.0)
}

impl<M> Natural<'_, M> {
    fn transform(&self, layout: Layout<'_>) -> Transformation {
        let origin = layout.position();
        let scale = if self.scale_down {
            factor(
                Size::new(
                    layout.child(0).bounds().width.max(self.minimum.width),
                    layout.child(0).bounds().height.max(self.minimum.height),
                ),
                layout.bounds().size(),
            )
        } else {
            1.0
        };
        Transformation::translate(origin.x, origin.y)
            * Transformation::scale(scale)
            * Transformation::translate(-origin.x, -origin.y)
    }
}

impl<M> Widget<M, Theme, iced::Renderer> for Natural<'_, M> {
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
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
        let child = self.content.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::with_compression(self.minimum, Size::INFINITE, Size::new(true, true)),
        );
        let natural = Size::new(
            child.size().width.max(self.minimum.width),
            child.size().height.max(self.minimum.height),
        );
        let viewport = limits.resolve(self.width, self.height, natural);
        let size = if self.scale_down {
            viewport
        } else {
            Size::new(
                viewport.width.max(natural.width),
                viewport.height.max(natural.height),
            )
        };
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
        let transform = self.transform(layout);
        if transform.scale_factor() == 0.0 {
            return;
        }
        let inverse = transform.inverse();
        let view = layout
            .bounds()
            .intersection(viewport)
            .unwrap_or(Rectangle::new(layout.position(), Size::ZERO));
        if let Event::Touch(touch::Event::FingerPressed { position, .. }) = event {
            if !view.contains(*position) {
                return;
            }
        }
        let cursor = if cursor.is_over(view) {
            cursor * inverse
        } else {
            mouse::Cursor::Unavailable
        };
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            &map_event(event, inverse),
            layout.child(0),
            cursor,
            renderer,
            clipboard,
            shell,
            &(view * inverse),
        );
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
        let transform = self.transform(layout);
        if transform.scale_factor() == 0.0 {
            return;
        }
        let Some(view) = layout.bounds().intersection(viewport) else {
            return;
        };
        let inverse = transform.inverse();
        renderer.with_layer(view, |renderer| {
            renderer.with_transformation(transform, |renderer| {
                self.content.as_widget().draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    layout.child(0),
                    cursor * inverse,
                    &(view * inverse),
                );
            });
        });
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        let transform = self.transform(layout);
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.child(0),
            renderer,
            &mut TransformedOperation {
                operation,
                transform,
            },
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
        let transform = self.transform(layout);
        if transform.scale_factor() == 0.0 {
            return mouse::Interaction::None;
        }
        let Some(view) = layout.bounds().intersection(viewport) else {
            return mouse::Interaction::None;
        };
        if !cursor.is_over(view) {
            return mouse::Interaction::None;
        }
        let inverse = transform.inverse();
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.child(0),
            cursor * inverse,
            &(view * inverse),
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
        let transform = self.transform(layout);
        if transform.scale_factor() == 0.0 {
            return None;
        }
        let inverse = transform.inverse();
        self.content
            .as_widget_mut()
            .overlay(
                &mut tree.children[0],
                layout.child(0),
                renderer,
                &(*viewport * inverse),
                translation * inverse,
            )
            .map(|content| transformed_overlay(content, transform))
    }
}

fn map_event(event: &Event, inverse: Transformation) -> Event {
    match event {
        Event::Mouse(mouse::Event::CursorMoved { position }) => {
            Event::Mouse(mouse::Event::CursorMoved {
                position: *position * inverse,
            })
        }
        Event::Mouse(mouse::Event::WheelScrolled {
            delta: mouse::ScrollDelta::Pixels { x, y },
        }) => {
            let delta = Vector::new(*x, *y) * inverse;
            Event::Mouse(mouse::Event::WheelScrolled {
                delta: mouse::ScrollDelta::Pixels {
                    x: delta.x,
                    y: delta.y,
                },
            })
        }
        Event::Touch(event) => Event::Touch(match *event {
            touch::Event::FingerPressed { id, position } => touch::Event::FingerPressed {
                id,
                position: position * inverse,
            },
            touch::Event::FingerMoved { id, position } => touch::Event::FingerMoved {
                id,
                position: position * inverse,
            },
            touch::Event::FingerLifted { id, position } => touch::Event::FingerLifted {
                id,
                position: position * inverse,
            },
            touch::Event::FingerLost { id, position } => touch::Event::FingerLost {
                id,
                position: position * inverse,
            },
        }),
        _ => event.clone(),
    }
}

// Preserve traversal/state semantics while reporting all geometry in the same
// coordinates as the rendered result (including nested operation traversal).
struct TransformedOperation<'a> {
    operation: &'a mut dyn Operation,
    transform: Transformation,
}

impl Operation for TransformedOperation<'_> {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        let transform = self.transform;
        self.operation.traverse(&mut |operation| {
            operate(&mut TransformedOperation {
                operation,
                transform,
            })
        });
    }
    fn container(&mut self, id: Option<&iced::advanced::widget::Id>, bounds: Rectangle) {
        self.operation.container(id, bounds * self.transform);
    }
    fn focusable(
        &mut self,
        id: Option<&iced::advanced::widget::Id>,
        bounds: Rectangle,
        state: &mut dyn operation::Focusable,
    ) {
        self.operation.focusable(id, bounds * self.transform, state);
    }
    fn scrollable(
        &mut self,
        id: Option<&iced::advanced::widget::Id>,
        bounds: Rectangle,
        content: Rectangle,
        translation: Vector,
        state: &mut dyn operation::Scrollable,
    ) {
        self.operation.scrollable(
            id,
            bounds * self.transform,
            content * self.transform,
            translation * self.transform,
            state,
        );
    }
    fn text_input(
        &mut self,
        id: Option<&iced::advanced::widget::Id>,
        bounds: Rectangle,
        state: &mut dyn operation::TextInput,
    ) {
        self.operation
            .text_input(id, bounds * self.transform, state);
    }
    fn text(&mut self, id: Option<&iced::advanced::widget::Id>, bounds: Rectangle, text: &str) {
        self.operation.text(id, bounds * self.transform, text);
    }
    fn custom(
        &mut self,
        id: Option<&iced::advanced::widget::Id>,
        bounds: Rectangle,
        state: &mut dyn Any,
    ) {
        self.operation.custom(id, bounds * self.transform, state);
    }
    fn finish(&self) -> operation::Outcome<()> {
        self.operation.finish()
    }
}

struct TransformedOverlay<'a, M> {
    content: overlay::Element<'a, M, Theme, iced::Renderer>,
    transform: Transformation,
}

fn transformed_overlay<'a, M: 'a>(
    content: overlay::Element<'a, M, Theme, iced::Renderer>,
    transform: Transformation,
) -> overlay::Element<'a, M, Theme, iced::Renderer> {
    overlay::Element::new(Box::new(TransformedOverlay { content, transform }))
}

impl<M> overlay::Overlay<M, Theme, iced::Renderer> for TransformedOverlay<'_, M> {
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        // Keep a raw-coordinate child beneath the visual root. The runtime sees
        // visual bounds; the child overlay always sees its original layout.
        let edge = Point::new(bounds.width, bounds.height) * self.transform.inverse();
        let raw = self
            .content
            .as_overlay_mut()
            .layout(renderer, Size::new(edge.x.max(0.0), edge.y.max(0.0)));
        let visual = raw.bounds() * self.transform;
        let position = raw.bounds().position() - Vector::new(visual.x, visual.y);
        layout::Node::with_children(visual.size(), vec![raw.move_to(position)])
            .move_to(visual.position())
    }
    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        renderer.with_transformation(self.transform, |renderer| {
            self.content.as_overlay().draw(
                renderer,
                theme,
                style,
                layout.child(0),
                cursor * self.transform.inverse(),
            );
        });
    }
    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
    ) {
        let inverse = self.transform.inverse();
        self.content.as_overlay_mut().update(
            &map_event(event, inverse),
            layout.child(0),
            cursor * inverse,
            renderer,
            clipboard,
            shell,
        );
    }
    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_overlay().mouse_interaction(
            layout.child(0),
            cursor * self.transform.inverse(),
            renderer,
        )
    }
    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content.as_overlay_mut().operate(
            layout.child(0),
            renderer,
            &mut TransformedOperation {
                operation,
                transform: self.transform,
            },
        );
    }
    fn overlay<'a>(
        &'a mut self,
        layout: Layout<'a>,
        renderer: &iced::Renderer,
    ) -> Option<overlay::Element<'a, M, Theme, iced::Renderer>> {
        let transform = self.transform;
        self.content
            .as_overlay_mut()
            .overlay(layout.child(0), renderer)
            .map(|content| transformed_overlay(content, transform))
    }
    fn index(&self) -> f32 {
        self.content.as_overlay().index()
    }
}
