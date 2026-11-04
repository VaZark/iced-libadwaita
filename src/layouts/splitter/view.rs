//! `Splitter`: panes along one axis with a draggable handle between each pair, after Mantine's
//! [Splitter](https://mantine.dev/core/splitter/). The sizing rules (fixed and flexible panes,
//! limits, collapsing, redistribution) are the [`splitter`](crate::layouts::splitter::rule)
//! primitive; this lays the panes out, draws the handles and turns a drag into events.
//!
//! Controlled: the app owns one [`SplitterState`] and applies the one message the widget speaks:
//!
//! ```ignore
//! // state: panes with their sizes and limits
//! SplitterState::new(vec![
//!     Pane::fixed(240.0).min(Limit::Px(160.0)).collapsible(true),   // a sidebar
//!     Pane::flex(),
//! ])
//!
//! // view
//! Splitter::new(&self.split).pane(sidebar).pane(content).on_event(Msg::Split)
//!
//! // update
//! Msg::Split(event) => { self.split.apply(event); }
//! // from code: self.split.toggle(0)
//! ```
//!
//! Panes are the widget's children, in order, one per pane of the state. The handle is a
//! hairline with a small grip ([`with_handle`](Splitter::with_handle) hides the grip; the line
//! stays and still drags), [`line_size`](Splitter::line_size) its thickness. Splitters nest: put
//! one in a pane. Give it a bounded height (and width); it fills what it is given. Moving a
//! handle with the keyboard is not included: iced has no focus model to hang it on.

use crate::Theme;
use crate::color::alpha;
use crate::layouts::splitter::rule::{Event, SplitterState};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::{Border, Event as IcedEvent, Length, Point, Rectangle, Shadow, Size, Vector, mouse};

/// The width of a handle's grab area, centered on its line.
const HIT: f32 = 9.0;
/// The grip: the thumb's length along the handle and its thickness.
const GRIP_LENGTH: f32 = 28.0;
const GRIP_THICKNESS: f32 = 4.0;

/// Which way the panes sit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SplitterOrientation {
    /// Side by side, with vertical handles.
    #[default]
    Horizontal,
    /// Stacked, with horizontal handles.
    Vertical,
}

/// See the [module](self).
pub struct Splitter<'a, M, R = iced::Renderer> {
    state: &'a SplitterState,
    panes: Vec<iced::Element<'a, M, Theme, R>>,
    orientation: SplitterOrientation,
    line: f32,
    grip: bool,
    on_event: Option<Box<dyn Fn(Event) -> M + 'a>>,
    width: Length,
    height: Length,
}

impl<'a, M, R: iced::advanced::Renderer> Splitter<'a, M, R> {
    pub fn new(state: &'a SplitterState) -> Self {
        Self {
            state,
            panes: Vec::new(),
            orientation: SplitterOrientation::default(),
            line: 1.0,
            grip: true,
            on_event: None,
            width: Length::Fill,
            height: Length::Fill,
        }
    }
    /// The next pane's content.
    pub fn pane(mut self, content: impl Into<iced::Element<'a, M, Theme, R>>) -> Self {
        self.panes.push(content.into());
        self
    }
    pub fn orientation(mut self, o: SplitterOrientation) -> Self {
        self.orientation = o;
        self
    }
    /// The thickness of the line between panes (default 1 px).
    pub fn line_size(mut self, px: f32) -> Self {
        self.line = px.max(0.0);
        self
    }
    /// Whether the handles show a grip (default true). The line stays and still drags.
    pub fn with_handle(mut self, on: bool) -> Self {
        self.grip = on;
        self
    }
    /// A handle moved, or a pane collapsed: apply it with [`SplitterState::apply`].
    pub fn on_event(mut self, f: impl Fn(Event) -> M + 'a) -> Self {
        self.on_event = Some(Box::new(f));
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }

    fn vertical(&self) -> bool {
        self.orientation == SplitterOrientation::Vertical
    }
}

#[derive(Default)]
struct State {
    /// The handle being dragged and where on it the pointer took hold (along the main axis).
    drag: Option<(usize, f32)>,
    /// Pixels already published since the last layout: events in one frame share a layout, so
    /// each delta is measured against where the handle will be, not where it was drawn.
    pending: f32,
}

/// The main-axis range of handle `i` (the line between pane `i` and pane `i + 1`).
fn handle_span(layout: Layout<'_>, i: usize, vertical: bool) -> Option<(f32, f32)> {
    let mut kids = layout.children();
    let a = kids.nth(i)?.bounds();
    let b = kids.next()?.bounds();
    Some(if vertical {
        (a.y + a.height, b.y)
    } else {
        (a.x + a.width, b.x)
    })
}

/// The grab area of handle `i` as a rectangle.
fn handle_rect(layout: Layout<'_>, i: usize, vertical: bool, line: f32) -> Option<Rectangle> {
    let (from, to) = handle_span(layout, i, vertical)?;
    let center = from + (to - from).max(line) / 2.0;
    let b = layout.bounds();
    Some(if vertical {
        Rectangle::new(Point::new(b.x, center - HIT / 2.0), Size::new(b.width, HIT))
    } else {
        Rectangle::new(
            Point::new(center - HIT / 2.0, b.y),
            Size::new(HIT, b.height),
        )
    })
}

impl<'a, M: 'a, R: iced::advanced::Renderer + 'a> Widget<M, Theme, R> for Splitter<'a, M, R> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn children(&self) -> Vec<Tree> {
        self.panes.iter().map(Tree::new).collect()
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.panes);
    }
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &R, limits: &layout::Limits) -> layout::Node {
        tree.state.downcast_mut::<State>().pending = 0.0;
        let size = limits.resolve(self.width, self.height, Size::ZERO);
        let vertical = self.vertical();
        let (main, cross) = if vertical {
            (size.height, size.width)
        } else {
            (size.width, size.height)
        };
        let n = self.panes.len();
        let lines = n.saturating_sub(1) as f32 * self.line;
        let px = self.state.resolve((main - lines).max(0.0));
        let mut at = 0.0;
        let mut nodes = Vec::with_capacity(n);
        for (i, (pane, t)) in self.panes.iter_mut().zip(&mut tree.children).enumerate() {
            let p = px.get(i).copied().unwrap_or(0.0);
            let exact = if vertical {
                Size::new(cross, p)
            } else {
                Size::new(p, cross)
            };
            let node = pane
                .as_widget_mut()
                .layout(t, renderer, &layout::Limits::new(exact, exact))
                .move_to(if vertical {
                    Point::new(0.0, at)
                } else {
                    Point::new(at, 0.0)
                });
            at += p + self.line;
            nodes.push(node);
        }
        layout::Node::with_children(size, nodes)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &IcedEvent,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &R,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        let vertical = self.vertical();
        let b = layout.bounds();
        let n = self.panes.len();
        let panes_total = (if vertical { b.height } else { b.width }
            - n.saturating_sub(1) as f32 * self.line)
            .max(0.0);
        let main = |p: Point| if vertical { p.y } else { p.x };
        let state = tree.state.downcast_mut::<State>();

        if let Some((handle, grab)) = state.drag {
            match event {
                IcedEvent::Mouse(mouse::Event::CursorMoved { .. }) => {
                    // The pointer from `cursor`: a scrollable parent shifts it by its offset.
                    if let (Some(p), Some((from, to))) =
                        (cursor.position(), handle_span(layout, handle, vertical))
                    {
                        let center = from + (to - from).max(self.line) / 2.0;
                        let delta = (main(p) - grab) - (center + state.pending);
                        if delta.abs() >= 0.5
                            && let Some(f) = &self.on_event
                        {
                            state.pending += delta;
                            shell.publish(f(Event::Resize {
                                handle,
                                delta,
                                total: panes_total,
                            }));
                        }
                    }
                    shell.request_redraw();
                }
                IcedEvent::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    state.drag = None;
                    shell.request_redraw();
                }
                _ => {}
            }
            shell.capture_event();
            return;
        }

        if self.on_event.is_some()
            && let IcedEvent::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
            && let Some(p) = cursor.position()
            && let Some(i) = (0..n.saturating_sub(1)).find(|&i| {
                handle_rect(layout, i, vertical, self.line).is_some_and(|r| r.contains(p))
            })
            && let Some((from, to)) = handle_span(layout, i, vertical)
        {
            let center = from + (to - from).max(self.line) / 2.0;
            state.drag = Some((i, main(p) - center));
            state.pending = 0.0;
            shell.capture_event();
            shell.request_redraw();
            return;
        }

        for ((pane, t), l) in self
            .panes
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            pane.as_widget_mut()
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
        let vertical = self.vertical();
        let resize = if vertical {
            mouse::Interaction::ResizingVertically
        } else {
            mouse::Interaction::ResizingHorizontally
        };
        if tree.state.downcast_ref::<State>().drag.is_some() {
            return resize;
        }
        if self.on_event.is_some()
            && (0..self.panes.len().saturating_sub(1)).any(|i| {
                handle_rect(layout, i, vertical, self.line).is_some_and(|r| cursor.is_over(r))
            })
        {
            return resize;
        }
        self.panes
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((pane, t), l)| {
                pane.as_widget()
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
        let vertical = self.vertical();
        let drag = tree.state.downcast_ref::<State>().drag.map(|(i, _)| i);
        // Each pane is clipped to its own bounds, so a collapsed or narrow one never spills.
        for ((pane, t), l) in self.panes.iter().zip(&tree.children).zip(layout.children()) {
            renderer.with_layer(l.bounds(), |renderer| {
                pane.as_widget()
                    .draw(t, renderer, theme, style, l, cursor, viewport);
            });
        }
        let b = layout.bounds();
        for i in 0..self.panes.len().saturating_sub(1) {
            let Some((from, to)) = handle_span(layout, i, vertical) else {
                continue;
            };
            let hot = drag == Some(i)
                || (drag.is_none()
                    && handle_rect(layout, i, vertical, self.line)
                        .is_some_and(|r| cursor.is_over(r)));
            let color = if hot {
                theme.semantic.accent.bg
            } else {
                theme.semantic.border
            };
            let thickness = (to - from).max(self.line);
            let line = if vertical {
                Rectangle::new(Point::new(b.x, from), Size::new(b.width, thickness))
            } else {
                Rectangle::new(Point::new(from, b.y), Size::new(thickness, b.height))
            };
            if self.line > 0.0 {
                renderer.fill_quad(
                    Quad {
                        bounds: line,
                        border: Border::default(),
                        shadow: Shadow::default(),
                        snap: true,
                    },
                    color,
                );
            }
            if self.grip {
                let c = line.center();
                let (w, h) = if vertical {
                    (GRIP_LENGTH, GRIP_THICKNESS)
                } else {
                    (GRIP_THICKNESS, GRIP_LENGTH)
                };
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(
                            Point::new(c.x - w / 2.0, c.y - h / 2.0),
                            Size::new(w, h),
                        ),
                        border: Border::default().rounded(GRIP_THICKNESS / 2.0),
                        shadow: Shadow::default(),
                        snap: false,
                    },
                    if hot {
                        theme.semantic.accent.bg
                    } else {
                        alpha(theme.semantic.window.fg, 0.35)
                    },
                );
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
        for ((pane, t), l) in self
            .panes
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            pane.as_widget_mut().operate(t, l, renderer, operation);
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
            .panes
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
            .filter_map(|((pane, t), l)| {
                pane.as_widget_mut()
                    .overlay(t, l, renderer, viewport, translation)
            })
            .collect();
        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

impl<'a, M: 'a, R: iced::advanced::Renderer + 'a> From<Splitter<'a, M, R>>
    for iced::Element<'a, M, Theme, R>
{
    fn from(s: Splitter<'a, M, R>) -> Self {
        iced::Element::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layouts::splitter::rule::{Limit, Pane};
    use iced::advanced::clipboard;
    use iced::widget::space;

    type E = iced::Element<'static, Event, Theme, ()>;

    fn element(state: &'static SplitterState, vertical: bool) -> E {
        let mut s = Splitter::<Event, ()>::new(state)
            .on_event(|e| e)
            .orientation(if vertical {
                SplitterOrientation::Vertical
            } else {
                SplitterOrientation::Horizontal
            });
        for _ in 0..state.len() {
            s = s.pane(space().width(Length::Fill).height(Length::Fill));
        }
        s.into()
    }

    fn lay(el: &mut E, tree: &mut Tree) -> layout::Node {
        el.as_widget_mut().layout(
            tree,
            &(),
            &layout::Limits::new(Size::ZERO, Size::new(501.0, 300.0)),
        )
    }

    #[test]
    fn panes_are_laid_out_at_their_resolved_sizes_with_a_line_between() {
        let state: &'static SplitterState = Box::leak(Box::new(SplitterState::new(vec![
            Pane::fixed(200.0),
            Pane::flex(),
            Pane::flex(),
        ])));
        let mut el = element(state, false);
        let mut tree = Tree::new(&el);
        let node = lay(&mut el, &mut tree);
        // 501 wide, two 1 px lines: 499 for the panes, 200 fixed and 149.5 each.
        let widths: Vec<f32> = Layout::new(&node)
            .children()
            .map(|c| c.bounds().width)
            .collect();
        assert!(
            (widths[0] - 200.0).abs() < 0.01 && (widths[1] - 149.5).abs() < 0.01,
            "{widths:?}"
        );
        let xs: Vec<f32> = Layout::new(&node)
            .children()
            .map(|c| c.bounds().x)
            .collect();
        assert!(
            (xs[1] - 201.0).abs() < 0.01,
            "a line after the first pane: {xs:?}"
        );
    }

    #[test]
    fn a_vertical_splitter_stacks_its_panes() {
        let state: &'static SplitterState = Box::leak(Box::new(SplitterState::new(vec![
            Pane::weight(1.0),
            Pane::weight(2.0),
        ])));
        let mut el = element(state, true);
        let mut tree = Tree::new(&el);
        let node = lay(&mut el, &mut tree);
        let heights: Vec<f32> = Layout::new(&node)
            .children()
            .map(|c| c.bounds().height)
            .collect();
        assert!(
            (heights[0] - 99.67).abs() < 0.1 && (heights[1] - 199.33).abs() < 0.1,
            "{heights:?}"
        );
        assert!(
            Layout::new(&node)
                .children()
                .all(|c| (c.bounds().width - 501.0).abs() < 0.01)
        );
    }

    /// Presses on the first handle, moves the pointer `by` px along the main axis and returns
    /// what was published.
    fn drag(state: &'static SplitterState, by: f32) -> Vec<Event> {
        let mut el = element(state, false);
        let mut tree = Tree::new(&el);
        let node = lay(&mut el, &mut tree);
        let layout = Layout::new(&node);
        let handle_x = {
            let first = layout.children().next().unwrap().bounds();
            first.x + first.width + 0.5
        };
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        let view = Rectangle::new(Point::ORIGIN, Size::new(501.0, 300.0));
        let mut send = |shell: &mut Shell<'_, Event>, e: IcedEvent, at: Point| {
            el.as_widget_mut().update(
                &mut tree,
                &e,
                layout,
                mouse::Cursor::Available(at),
                &(),
                &mut clipboard::Null,
                shell,
                &view,
            );
        };
        let press = Point::new(handle_x, 50.0);
        send(
            &mut shell,
            IcedEvent::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            press,
        );
        let to = Point::new(handle_x + by, 50.0);
        send(
            &mut shell,
            IcedEvent::Mouse(mouse::Event::CursorMoved { position: to }),
            to,
        );
        // A second move in the same frame (same layout) must not double-count.
        send(
            &mut shell,
            IcedEvent::Mouse(mouse::Event::CursorMoved { position: to }),
            to,
        );
        drop(shell);
        messages
    }

    #[test]
    fn dragging_a_handle_publishes_the_distance_once() {
        let state: &'static SplitterState = Box::leak(Box::new(SplitterState::new(vec![
            Pane::flex(),
            Pane::flex(),
        ])));
        let got = drag(state, 40.0);
        assert_eq!(
            got.len(),
            1,
            "a repeated move in the same frame adds nothing: {got:?}"
        );
        let Event::Resize {
            handle,
            delta,
            total,
        } = got[0]
        else {
            panic!("a resize")
        };
        assert_eq!(handle, 0);
        assert!((delta - 40.0).abs() < 0.5, "{delta}");
        assert!(
            (total - 500.0).abs() < 0.01,
            "the panes share what the line leaves: {total}"
        );
    }

    #[test]
    fn pressing_away_from_a_handle_does_not_start_a_drag() {
        let state: &'static SplitterState = Box::leak(Box::new(SplitterState::new(vec![
            Pane::flex(),
            Pane::flex(),
        ])));
        let mut el = element(state, false);
        let mut tree = Tree::new(&el);
        let node = lay(&mut el, &mut tree);
        let layout = Layout::new(&node);
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        let view = Rectangle::new(Point::ORIGIN, Size::new(501.0, 300.0));
        for e in [
            IcedEvent::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            IcedEvent::Mouse(mouse::Event::CursorMoved {
                position: Point::new(100.0, 50.0),
            }),
        ] {
            el.as_widget_mut().update(
                &mut tree,
                &e,
                layout,
                mouse::Cursor::Available(Point::new(100.0, 50.0)),
                &(),
                &mut clipboard::Null,
                &mut shell,
                &view,
            );
        }
        drop(shell);
        assert!(messages.is_empty());
        let _ = Limit::Px(0.0);
    }
}
