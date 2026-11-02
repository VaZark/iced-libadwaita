//! `Resizable`: a frame of a given size with a drag handle, like a textarea's grip. The content
//! is laid out at the frame's size, so a responsive component inside (a `MetaComponent`, a
//! table, an app shell) can be tried at any width and height without resizing the window.
//!
//! Controlled: the app owns the `Size` and takes it from `on_resize`; the drag rules are
//! [`primitives::resize`](crate::primitives::resize) (min / max, a grid, snapping the width to
//! breakpoints).
//!
//! ```ignore
//! Resizable::new(meta_component, self.frame)        // frame: iced::Size
//!     .on_resize(Msg::Frame)
//!     .constraints(Constraints::new(Size::new(220.0, 120.0), Size::new(900.0, 500.0))
//!         .snap_to(breakpoints, 8.0))
//! ```
//!
//! The frame is clamped to the room its parent gives it, and `on_resize` then reports the size it
//! actually got (once, on the next redraw), so the app's `Size` never claims more than is shown. The handle is the bottom-right corner
//! by default ([`Grip`] for the edges and other corners).

use crate::Theme;
use crate::primitives::resize::{Constraints, Grip, drag};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::{Border, Color, Event, Length, Point, Rectangle, Shadow, Size, Vector, mouse};

/// Side of the corner handle's hit area.
const HIT: f32 = 18.0;
/// Thickness of an edge handle's hit area.
const EDGE: f32 = 8.0;

/// See the [module](self).
pub struct Resizable<'a, M, R = iced::Renderer> {
    content: iced::Element<'a, M, Theme, R>,
    size: Size,
    on_resize: Option<Box<dyn Fn(Size) -> M + 'a>>,
    constraints: Constraints,
    grip: Grip,
    border: bool,
}

impl<'a, M: 'a, R: iced::advanced::Renderer + 'a> Resizable<'a, M, R> {
    pub fn new(content: impl Into<iced::Element<'a, M, Theme, R>>, size: Size) -> Self {
        Self {
            content: content.into(),
            size,
            on_resize: None,
            constraints: Constraints::default(),
            grip: Grip::default(),
            border: true,
        }
    }
    /// Receives the new size while the handle is dragged. Without it the handle is inert.
    pub fn on_resize(mut self, f: impl Fn(Size) -> M + 'a) -> Self {
        self.on_resize = Some(Box::new(f));
        self
    }
    pub fn constraints(mut self, c: Constraints) -> Self {
        self.constraints = c;
        self
    }
    /// Which edge or corner is the handle (default: the bottom-right corner).
    pub fn grip(mut self, grip: Grip) -> Self {
        self.grip = grip;
        self
    }
    /// Draws a hairline frame around the content (default on).
    pub fn border(mut self, on: bool) -> Self {
        self.border = on;
        self
    }
}

#[derive(Default)]
struct State {
    /// The pointer position and the frame size when the drag began.
    drag: Option<(Point, Size)>,
    /// The room the parent gave the frame at the last layout.
    room: Option<Size>,
}

/// The handle's hit area within the frame `bounds`.
fn grip_rect(bounds: Rectangle, grip: Grip) -> Rectangle {
    let (right, bottom, left, top) = (
        bounds.x + bounds.width,
        bounds.y + bounds.height,
        bounds.x,
        bounds.y,
    );
    match grip {
        Grip::BottomRight => {
            Rectangle::new(Point::new(right - HIT, bottom - HIT), Size::new(HIT, HIT))
        }
        Grip::BottomLeft => Rectangle::new(Point::new(left, bottom - HIT), Size::new(HIT, HIT)),
        Grip::TopRight => Rectangle::new(Point::new(right - HIT, top), Size::new(HIT, HIT)),
        Grip::TopLeft => Rectangle::new(Point::new(left, top), Size::new(HIT, HIT)),
        Grip::Right => Rectangle::new(
            Point::new(right - EDGE, top),
            Size::new(EDGE, bounds.height),
        ),
        Grip::Left => Rectangle::new(Point::new(left, top), Size::new(EDGE, bounds.height)),
        Grip::Bottom => Rectangle::new(
            Point::new(left, bottom - EDGE),
            Size::new(bounds.width, EDGE),
        ),
        Grip::Top => Rectangle::new(Point::new(left, top), Size::new(bounds.width, EDGE)),
    }
}

fn interaction(grip: Grip) -> mouse::Interaction {
    match grip {
        Grip::Right | Grip::Left => mouse::Interaction::ResizingHorizontally,
        Grip::Bottom | Grip::Top => mouse::Interaction::ResizingVertically,
        Grip::BottomRight | Grip::TopLeft => mouse::Interaction::ResizingDiagonallyDown,
        Grip::BottomLeft | Grip::TopRight => mouse::Interaction::ResizingDiagonallyUp,
    }
}

impl<'a, M: 'a, R: iced::advanced::Renderer + 'a> Widget<M, Theme, R> for Resizable<'a, M, R> {
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
        Size::new(Length::Shrink, Length::Shrink)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &R, limits: &layout::Limits) -> layout::Node {
        let room = limits.max();
        tree.state.downcast_mut::<State>().room = Some(room);
        let size = Size::new(
            self.size.width.min(room.width),
            self.size.height.min(room.height),
        );
        let child = self.content.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(size, size),
        );
        layout::Node::with_children(size, vec![child])
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
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<State>();
        if self.on_resize.is_some() {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                    if cursor.is_over(grip_rect(bounds, self.grip)) =>
                {
                    if let Some(p) = cursor.position() {
                        state.drag = Some((p, bounds.size()));
                        shell.capture_event();
                        shell.request_redraw();
                        return;
                    }
                }
                Event::Mouse(mouse::Event::CursorMoved { .. }) if state.drag.is_some() => {
                    // The pointer from `cursor`, as at the press: a scrollable parent shifts it
                    // by its scroll offset, which the raw event position does not include, so
                    // mixing the two made the first move look like a huge drag.
                    let Some(position) = cursor.position() else {
                        shell.capture_event();
                        return;
                    };
                    let (from, start) = state.drag.expect("dragging");
                    let mut c = self.constraints.clone();
                    if let Some(room) = state.room {
                        c.max =
                            Size::new(c.max.width.min(room.width), c.max.height.min(room.height));
                    }
                    let size = drag(
                        start,
                        Vector::new(position.x - from.x, position.y - from.y),
                        self.grip,
                        &c,
                    );
                    if size != bounds.size()
                        && let Some(f) = &self.on_resize
                    {
                        shell.publish(f(size));
                    }
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                    if state.drag.is_some() =>
                {
                    state.drag = None;
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
                _ => {}
            }
        }
        // Clamped by the parent: tell the app the size that is really shown.
        if let Event::Window(iced::window::Event::RedrawRequested(_)) = event
            && state.drag.is_none()
            && let Some(f) = &self.on_resize
            && ((bounds.width - self.size.width).abs() > 0.5
                || (bounds.height - self.size.height).abs() > 0.5)
        {
            shell.publish(f(bounds.size()));
        }
        // While the handle is held the content does not see the pointer.
        if state.drag.is_some() {
            return;
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout.children().next().expect("a child layout"),
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
        renderer: &R,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        if self.on_resize.is_some()
            && (state.drag.is_some() || cursor.is_over(grip_rect(layout.bounds(), self.grip)))
        {
            return interaction(self.grip);
        }
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().expect("a child layout"),
            cursor,
            viewport,
            renderer,
        )
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
        let bounds = layout.bounds();
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().expect("a child layout"),
            cursor,
            viewport,
        );
        let ink = |alpha: f32| Color {
            a: alpha,
            ..theme.semantic.window.fg
        };
        if self.border {
            renderer.fill_quad(
                Quad {
                    bounds,
                    border: Border {
                        color: ink(0.22),
                        width: 1.0,
                        radius: 0.0.into(),
                    },
                    shadow: Shadow::default(),
                    snap: false,
                },
                Color::TRANSPARENT,
            );
        }
        if self.on_resize.is_none() {
            return;
        }
        let active = tree.state.downcast_ref::<State>().drag.is_some()
            || cursor.is_over(grip_rect(bounds, self.grip));
        let dot = ink(if active { 0.7 } else { 0.4 });
        let r = grip_rect(bounds, self.grip);
        // Dots: a triangle in the corner (a textarea's grip) or a short run along an edge.
        let mut put = |x: f32, y: f32| {
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle::new(Point::new(x, y), Size::new(2.0, 2.0)),
                    border: Border::default().rounded(1.0),
                    shadow: Shadow::default(),
                    snap: false,
                },
                dot,
            );
        };
        let (right, bottom) = (r.x + r.width, r.y + r.height);
        match self.grip {
            Grip::BottomRight => {
                for row in 0..3 {
                    for col in 0..3 {
                        if row + col >= 2 {
                            put(
                                right - 5.0 - col as f32 * 4.0,
                                bottom - 5.0 - row as f32 * 4.0,
                            );
                        }
                    }
                }
            }
            Grip::BottomLeft => {
                for row in 0..3 {
                    for col in 0..3 {
                        if row + col >= 2 {
                            put(
                                r.x + 3.0 + col as f32 * 4.0,
                                bottom - 5.0 - row as f32 * 4.0,
                            );
                        }
                    }
                }
            }
            Grip::TopRight => {
                for row in 0..3 {
                    for col in 0..3 {
                        if row + col >= 2 {
                            put(right - 5.0 - col as f32 * 4.0, r.y + 3.0 + row as f32 * 4.0);
                        }
                    }
                }
            }
            Grip::TopLeft => {
                for row in 0..3 {
                    for col in 0..3 {
                        if row + col >= 2 {
                            put(r.x + 3.0 + col as f32 * 4.0, r.y + 3.0 + row as f32 * 4.0);
                        }
                    }
                }
            }
            Grip::Right | Grip::Left => {
                let x = if self.grip == Grip::Right {
                    right - 5.0
                } else {
                    r.x + 3.0
                };
                let mid = r.y + r.height / 2.0;
                for i in -1..=1 {
                    put(x, mid + i as f32 * 5.0 - 1.0);
                }
            }
            Grip::Bottom | Grip::Top => {
                let y = if self.grip == Grip::Bottom {
                    bottom - 5.0
                } else {
                    r.y + 3.0
                };
                let mid = r.x + r.width / 2.0;
                for i in -1..=1 {
                    put(mid + i as f32 * 5.0 - 1.0, y);
                }
            }
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &R,
        operation: &mut dyn Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().expect("a child layout"),
            renderer,
            operation,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &R,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, R>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next()?,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, M: 'a, R: iced::advanced::Renderer + 'a> From<Resizable<'a, M, R>>
    for iced::Element<'a, M, Theme, R>
{
    fn from(r: Resizable<'a, M, R>) -> Self {
        iced::Element::new(r)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::widget::Tree;
    use iced::widget::space;

    type E = iced::Element<'static, (), Theme, ()>;

    fn lay(frame: Size, room: Size) -> Size {
        let content: E = space().width(Length::Fill).height(Length::Fill).into();
        let mut el: E = Resizable::<(), ()>::new(content, frame).into();
        let mut tree = Tree::new(&el);
        el.as_widget_mut()
            .layout(&mut tree, &(), &layout::Limits::new(Size::ZERO, room))
            .size()
    }

    #[test]
    fn the_frame_is_laid_out_at_its_size() {
        assert_eq!(
            lay(Size::new(320.0, 200.0), Size::new(900.0, 700.0)),
            Size::new(320.0, 200.0)
        );
    }

    #[test]
    fn the_frame_never_exceeds_the_room_its_parent_gives_it() {
        assert_eq!(
            lay(Size::new(1200.0, 900.0), Size::new(800.0, 600.0)),
            Size::new(800.0, 600.0)
        );
    }

    #[test]
    fn the_handle_sits_in_the_chosen_corner_or_edge() {
        let b = Rectangle::new(Point::new(10.0, 20.0), Size::new(300.0, 200.0));
        let br = grip_rect(b, Grip::BottomRight);
        assert_eq!((br.x + br.width, br.y + br.height), (310.0, 220.0));
        let r = grip_rect(b, Grip::Right);
        assert_eq!((r.x + r.width, r.height), (310.0, 200.0));
        assert!(grip_rect(b, Grip::TopLeft).contains(Point::new(12.0, 22.0)));
    }

    /// Drags the corner handle of a 300x200 frame by `by` in a parent scrolled by `scrolled`:
    /// a scrollable shifts the `cursor` it passes down by its scroll offset, while the raw
    /// position carried by the event is not shifted. Returns the sizes the frame asked for.
    fn drag_scrolled(by: Vector, scrolled: Vector) -> Vec<Size> {
        use iced::advanced::clipboard;
        type S = iced::Element<'static, Size, Theme, ()>;
        let content: S = space().width(Length::Fill).height(Length::Fill).into();
        let mut el: S = Resizable::<Size, ()>::new(content, Size::new(300.0, 200.0))
            .on_resize(|s| s)
            .into();
        let mut tree = Tree::new(&el);
        let node = el.as_widget_mut().layout(
            &mut tree,
            &(),
            &layout::Limits::new(Size::ZERO, Size::new(900.0, 700.0)),
        );
        let layout = Layout::new(&node);
        let mut messages: Vec<Size> = Vec::new();
        let mut shell = Shell::new(&mut messages);
        let view = Rectangle::new(Point::ORIGIN, Size::new(900.0, 700.0));
        let mut send = |shell: &mut Shell<'_, Size>, e: Event, cursor: Point| {
            el.as_widget_mut().update(
                &mut tree,
                &e,
                layout,
                mouse::Cursor::Available(cursor),
                &(),
                &mut clipboard::Null,
                shell,
                &view,
            );
        };
        let press = Point::new(295.0, 195.0);
        send(
            &mut shell,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            press,
        );
        let cursor = Point::new(press.x + by.x, press.y + by.y);
        send(
            &mut shell,
            Event::Mouse(mouse::Event::CursorMoved {
                position: Point::new(cursor.x + scrolled.x, cursor.y + scrolled.y),
            }),
            cursor,
        );
        drop(shell);
        messages
    }

    #[test]
    fn a_drag_in_a_vertically_scrolled_parent_is_not_thrown_by_the_offset() {
        // The reported bug: with the page scrolled 400 px down, a 20 px drag down collapsed the
        // frame to its minimum height and it never grew again.
        let got = drag_scrolled(Vector::new(0.0, 20.0), Vector::new(0.0, 400.0));
        assert_eq!(got, vec![Size::new(300.0, 220.0)]);
    }

    #[test]
    fn the_reported_collapse_a_page_scrolled_so_the_raw_position_is_smaller() {
        // Raw event y = cursor y - 400: the old code saw a 380 px drag up from a 20 px drag down
        // and clamped the frame to its minimum height.
        let got = drag_scrolled(Vector::new(0.0, 20.0), Vector::new(0.0, -400.0));
        assert_eq!(got, vec![Size::new(300.0, 220.0)]);
        let got = drag_scrolled(Vector::new(20.0, 0.0), Vector::new(-250.0, 0.0));
        assert_eq!(got, vec![Size::new(320.0, 200.0)]);
    }

    #[test]
    fn a_drag_in_a_horizontally_scrolled_parent_is_not_thrown_by_the_offset() {
        let got = drag_scrolled(Vector::new(30.0, 0.0), Vector::new(250.0, 0.0));
        assert_eq!(got, vec![Size::new(330.0, 200.0)]);
    }

    #[test]
    fn a_drag_in_a_parent_scrolled_both_ways_changes_both_axes_correctly() {
        let got = drag_scrolled(Vector::new(-40.0, 25.0), Vector::new(180.0, 400.0));
        assert_eq!(got, vec![Size::new(260.0, 225.0)]);
        // Shrinking is right too (a drag up and left), not a collapse to the minimum.
        let got = drag_scrolled(Vector::new(-60.0, -50.0), Vector::new(120.0, 300.0));
        assert_eq!(got, vec![Size::new(240.0, 150.0)]);
    }

    #[test]
    fn without_scrolling_the_drag_is_unchanged() {
        let got = drag_scrolled(Vector::new(10.0, 15.0), Vector::ZERO);
        assert_eq!(got, vec![Size::new(310.0, 215.0)]);
    }
}
