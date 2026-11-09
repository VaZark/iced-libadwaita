//! `RangeSlider`: a slider with two thumbs that selects a range (`lo..=hi`) of values; the
//! sibling of [`scale`](crate::components::scale), drawn with the same rail and handle styling
//! ([`style::slider`](crate::styles::slider)) with the accent fill between
//! the thumbs.
//!
//! A thin applier of [`primitives::range`](crate::primitives::range): the [`Track`] decides
//! which thumb a press grabs, where a thumb may go (the other thumb stops it, or is pushed, and
//! the minimum gap holds), the step it snaps to and what each key does. The application owns
//! the `(lo, hi)` pair and updates it from `on_change`; without `on_change`, or with
//! [`disabled`](RangeSlider::disabled), the slider is drawn dimmed and ignores input.
//!
//! ```ignore
//! range_slider(0.0..=100.0, (self.lo, self.hi), Message::Range)
//!     .step(5.0)
//!     .min_gap(10.0)
//! ```
//!
//! Each thumb is a focus stop: arrows step it, `PageUp`/`PageDown` move ten steps, `Home` and
//! `End` go to the ends. Pressing on a thumb drags it; pressing on the rail moves the nearest
//! thumb there and keeps dragging it.
//!
//! [`orientation`](RangeSlider::orientation) defaults to horizontal (right increases).
//! Vertical tracks increase upward and use [`height`](RangeSlider::height) for length.
//! Pointer coordinates and grab offsets come from [`primitives::scrub`](crate::primitives::scrub).

use crate::Theme;
use crate::primitives::range::{Collide, Key, Range, Scalar, Thumb, Track};
use crate::primitives::scrub::{Axis, Grab};
use crate::styles::slider::{self, HANDLE_RADIUS};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::operation::{Focusable, Operation};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::keyboard::{self, key::Named};
use iced::widget::slider::Status;
use iced::{
    Background, Border, Color, Element, Event, Length, Point, Rectangle, Shadow, Size, mouse, touch,
};
use std::ops::RangeInclusive;

const R: f32 = HANDLE_RADIUS;
/// Extra height under the rail for the tick marks.
const MARKS: f32 = 10.0;

type OnChange<'a, M, T> = Box<dyn Fn((T, T)) -> M + 'a>;

pub struct RangeSlider<'a, M, T = f32> {
    value: Range<T>,
    track: Track<T>,
    on_change: Option<OnChange<'a, M, T>>,
    marks: Vec<T>,
    disabled: bool,
    width: Length,
    height: Length,
    axis: Axis,
    single: bool,
    on_release: Option<Box<dyn Fn() -> M + 'a>>,
}

/// A `(lo, hi)` slider over `range`; the free-function form of [`RangeSlider::new`].
pub fn range_slider<'a, M>(
    range: RangeInclusive<f32>,
    value: (f32, f32),
    on_change: impl Fn((f32, f32)) -> M + 'a,
) -> RangeSlider<'a, M> {
    RangeSlider::new(range, value, on_change)
}

impl<'a, M, T: Scalar> RangeSlider<'a, M, T> {
    /// `value` is `(lo, hi)` (put in order and clamped to `range`); `on_change` gets the new
    /// pair whenever a thumb moves. The step is `1.0`.
    pub fn new(
        range: RangeInclusive<T>,
        value: (T, T),
        on_change: impl Fn((T, T)) -> M + 'a,
    ) -> Self {
        let track = Track::new(*range.start(), *range.end()).step(1.0);
        Self {
            value: track.clamp(Range::from(value)),
            track,
            on_change: Some(Box::new(on_change)),
            marks: Vec::new(),
            disabled: false,
            width: Length::Fill,
            height: Length::Fixed(160.0),
            axis: Axis::Horizontal,
            single: false,
            on_release: None,
        }
    }
    /// The grid the thumbs snap to, from the start of the range; `0` is continuous.
    pub fn step(mut self, step: impl Into<f64>) -> Self {
        self.track = self.track.step(step.into());
        self
    }
    /// The smallest allowed `hi - lo`.
    pub fn min_gap(mut self, gap: f32) -> Self {
        self.track = self.track.min_gap(gap as f64);
        self
    }
    /// What a thumb does at the other one: stop (the default) or push it along.
    pub fn collide(mut self, c: Collide) -> Self {
        self.track = self.track.collide(c);
        self
    }
    /// Shorthand for `.collide(Collide::Push)` when `push`.
    pub fn push_thumbs(self, push: bool) -> Self {
        self.collide(if push { Collide::Push } else { Collide::Stop })
    }
    /// Tick marks under the rail at these values.
    pub fn marks(mut self, marks: impl IntoIterator<Item = T>) -> Self {
        self.marks = marks.into_iter().collect();
        self
    }
    /// Ticks at every `every` from the start of the range.
    pub fn marks_every(self, every: f32) -> Self {
        let (min, max) = (self.track.min.to_f64(), self.track.max.to_f64());
        let n = if every > 0.0 {
            ((max - min) / every as f64 + 1e-4) as usize
        } else {
            0
        };
        self.marks((0..=n).map(|i| T::from_f64(min + i as f64 * every as f64)))
    }
    /// Dimmed, and ignores the pointer and the keyboard.
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }

    /// Direction of increasing values: right (horizontal) or up (vertical).
    pub fn orientation(mut self, axis: Axis) -> Self {
        self.axis = axis;
        self
    }
    /// Length of a vertical control (default 160px).
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }
    pub fn on_release(mut self, message: M) -> Self
    where
        M: Clone + 'a,
    {
        self.on_release = Some(Box::new(move || message.clone()));
        self
    }
    pub(crate) fn single(mut self) -> Self {
        self.single = true;
        self
    }
    fn thumbs(&self) -> impl Iterator<Item = Thumb> {
        [Thumb::Lo, Thumb::Hi]
            .into_iter()
            .filter(|t| !self.single || *t == Thumb::Hi)
    }
    fn dimensions(&self) -> Size<Length> {
        match self.axis {
            Axis::Horizontal => Size::new(self.width, Length::Fixed(self.thickness())),
            Axis::Vertical => Size::new(Length::Fixed(self.thickness()), self.height),
        }
    }
    fn enabled(&self) -> bool {
        !self.disabled && self.on_change.is_some()
    }
    fn thickness(&self) -> f32 {
        2.0 * R + if self.marks.is_empty() { 0.0 } else { MARKS }
    }
    /// Low-value endpoint and usable thumb travel.
    fn travel(&self, b: Rectangle) -> ((f32, f32), f32) {
        match self.axis {
            Axis::Horizontal => ((b.x + R, b.y + R), (b.width - 2.0 * R).max(0.0)),
            Axis::Vertical => ((b.x + R, b.y + b.height - R), (b.height - 2.0 * R).max(0.0)),
        }
    }
    fn center(&self, b: Rectangle, v: T) -> Point {
        let (origin, length) = self.travel(b);
        let p = self.axis.point(origin, length, self.track.fraction(v));
        Point::new(p.0, p.1)
    }
    fn value_at(&self, b: Rectangle, p: Point) -> T {
        let (origin, length) = self.travel(b);
        self.track
            .value_at(self.axis.fraction((p.x, p.y), origin, length))
    }
    /// The thumb whose handle `p` is on, the nearer when both are.
    fn thumb_at(&self, b: Rectangle, p: Point) -> Option<Thumb> {
        let d = |t: Thumb| self.center(b, self.value.get(t)).distance(p);
        let hit = |t: Thumb| d(t) <= R + 2.0;
        if self.single {
            return hit(Thumb::Hi).then_some(Thumb::Hi);
        }
        match (hit(Thumb::Lo), hit(Thumb::Hi)) {
            (true, true) => Some(self.track.nearest(self.value, self.value_at(b, p))),
            (true, false) => Some(Thumb::Lo),
            (false, true) => Some(Thumb::Hi),
            _ => None,
        }
    }
}

#[derive(Default)]
struct Focus(bool);

impl Focusable for Focus {
    fn is_focused(&self) -> bool {
        self.0
    }
    fn focus(&mut self) {
        self.0 = true;
    }
    fn unfocus(&mut self) {
        self.0 = false;
    }
}

#[derive(Default)]
struct State {
    /// The thumb being dragged, and how far the pointer was from its centre when it was grabbed.
    drag: Option<(Thumb, Grab)>,
    focus: [Focus; 2],
}

impl State {
    fn focus_of(&mut self, t: Thumb) -> &mut Focus {
        &mut self.focus[t as usize]
    }
}

impl<'a, M, T: Scalar> Widget<M, Theme, iced::Renderer> for RangeSlider<'a, M, T> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn size(&self) -> Size<Length> {
        self.dimensions()
    }
    fn layout(
        &mut self,
        _: &mut Tree,
        _: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = self.dimensions();
        layout::atomic(limits, size.width, size.height)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        _: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        if !self.enabled() {
            return;
        }
        let state = tree.state.downcast_mut::<State>();
        let b = layout.bounds();
        for t in self.thumbs() {
            let c = self.center(b, self.value.get(t));
            let area = Rectangle::new(Point::new(c.x - R, c.y - R), Size::new(2.0 * R, 2.0 * R));
            operation.focusable(None, area, state.focus_of(t));
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &iced::Renderer,
        _: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        _: &Rectangle,
    ) {
        if !self.enabled() {
            return;
        }
        let state = tree.state.downcast_mut::<State>();
        let b = layout.bounds();

        // Move `thumb` to the oriented pointer position and report changes.
        let drag_to = |this: &Self, shell: &mut Shell<'_, M>, thumb: Thumb, p: Point| {
            let next = this
                .track
                .move_thumb(this.value, thumb, this.value_at(b, p));
            if next != this.value
                && let Some(f) = &this.on_change
            {
                shell.publish(f((next.lo, next.hi)));
            }
        };

        let press = |this: &Self, state: &mut State, shell: &mut Shell<'_, M>, p: Point| {
            let (thumb, offset) = match this.thumb_at(b, p) {
                Some(t) => {
                    let c = this.center(b, this.value.get(t));
                    (t, Grab::new(this.axis, (p.x, p.y), (c.x, c.y)))
                }
                None => {
                    let t = if this.single {
                        Thumb::Hi
                    } else {
                        this.track.nearest(this.value, this.value_at(b, p))
                    };
                    (t, Grab::default())
                }
            };
            state.drag = Some((thumb, offset));
            state.focus_of(thumb).focus();
            state.focus_of(thumb.other()).unfocus();
            let adjusted = offset.center(this.axis, (p.x, p.y));
            drag_to(this, shell, thumb, Point::new(adjusted.0, adjusted.1));
            shell.capture_event();
            shell.request_redraw();
        };

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                match cursor.position_over(b) {
                    Some(p) => press(self, state, shell, p),
                    None => {
                        if state.focus.iter().any(|f| f.0) {
                            state.focus.iter_mut().for_each(Focus::unfocus);
                            shell.request_redraw();
                        }
                    }
                }
            }
            Event::Touch(touch::Event::FingerPressed { position, .. }) if b.contains(*position) => {
                press(self, state, shell, *position);
            }
            Event::Mouse(mouse::Event::CursorMoved { position })
            | Event::Touch(touch::Event::FingerMoved { position, .. }) => {
                if let Some((thumb, offset)) = state.drag {
                    let adjusted = offset.center(self.axis, (position.x, position.y));
                    drag_to(self, shell, thumb, Point::new(adjusted.0, adjusted.1));
                    shell.capture_event();
                    shell.request_redraw();
                } else if cursor.is_over(b) {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerLifted { .. } | touch::Event::FingerLost { .. })
                if state.drag.is_some() =>
            {
                state.drag = None;
                if let Some(message) = &self.on_release {
                    shell.publish(message());
                }
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                let Some(thumb) = [Thumb::Lo, Thumb::Hi]
                    .into_iter()
                    .find(|t| state.focus[*t as usize].0)
                else {
                    return;
                };
                let k = match key.as_ref() {
                    keyboard::Key::Named(Named::ArrowLeft | Named::ArrowDown) => Key::Dec,
                    keyboard::Key::Named(Named::ArrowRight | Named::ArrowUp) => Key::Inc,
                    keyboard::Key::Named(Named::PageDown) => Key::PageDec,
                    keyboard::Key::Named(Named::PageUp) => Key::PageInc,
                    keyboard::Key::Named(Named::Home) => Key::Home,
                    keyboard::Key::Named(Named::End) => Key::End,
                    _ => return,
                };
                let next = self.track.key(self.value, thumb, k);
                if next != self.value
                    && let Some(f) = &self.on_change
                {
                    shell.publish(f((next.lo, next.hi)));
                }
                shell.capture_event();
                shell.request_redraw();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Rectangle,
        _: &iced::Renderer,
    ) -> mouse::Interaction {
        if !self.enabled() {
            return mouse::Interaction::None;
        }
        let state = tree.state.downcast_ref::<State>();
        if state.drag.is_some() {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::None
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let b = layout.bounds();
        let enabled = self.enabled();
        let k = if enabled {
            1.0
        } else {
            theme.semantic.opacity.disabled
        };
        let dim = |c: Color| crate::color::fade(c, k);
        let base = slider::default(theme, Status::Active);
        let (origin, length) = self.travel(b);

        // Rail: the trough, then the accent fill between the thumbs.
        let rail = base.rail;
        let rail_bounds = |from: Point, to: Point| match self.axis {
            Axis::Horizontal => Rectangle {
                x: from.x,
                y: from.y - rail.width / 2.0,
                width: (to.x - from.x).max(0.0),
                height: rail.width,
            },
            Axis::Vertical => Rectangle {
                x: from.x - rail.width / 2.0,
                y: to.y,
                width: rail.width,
                height: (from.y - to.y).max(0.0),
            },
        };
        let fill = |renderer: &mut iced::Renderer, r: Rectangle, bg: Background| {
            let bg = match bg {
                Background::Color(c) => Background::Color(dim(c)),
                other => other,
            };
            renderer.fill_quad(
                Quad {
                    bounds: r,
                    border: rail.border,
                    shadow: Shadow::default(),
                    snap: false,
                },
                bg,
            );
        };
        let (cl, ch) = (self.center(b, self.value.lo), self.center(b, self.value.hi));
        fill(
            renderer,
            rail_bounds(
                self.center(b, self.track.min),
                self.center(b, self.track.max),
            ),
            rail.backgrounds.1,
        );
        fill(renderer, rail_bounds(cl, ch), rail.backgrounds.0);

        // Marks under the rail.
        let tick = dim(crate::color::alpha(theme.semantic.window.fg, 0.35));
        for m in &self.marks {
            let p = self.axis.point(origin, length, self.track.fraction(*m));
            renderer.fill_quad(
                Quad {
                    bounds: match self.axis {
                        Axis::Horizontal => Rectangle::new(
                            Point::new(p.0 - 0.5, b.y + 2.0 * R + 3.0),
                            Size::new(1.0, 5.0),
                        ),
                        Axis::Vertical => Rectangle::new(
                            Point::new(b.x + 2.0 * R + 3.0, p.1 - 0.5),
                            Size::new(5.0, 1.0),
                        ),
                    },
                    border: Border::default(),
                    shadow: Shadow::default(),
                    snap: true,
                },
                Background::Color(tick),
            );
        }

        // Thumbs: the one in use is drawn last (on top).
        let hovered = if enabled && state.drag.is_none() {
            cursor.position_over(b).and_then(|p| self.thumb_at(b, p))
        } else {
            None
        };
        let status = |t: Thumb| match state.drag {
            Some((d, _)) if d == t => Status::Dragged,
            _ if hovered == Some(t) => Status::Hovered,
            _ => Status::Active,
        };
        let top = state
            .drag
            .map(|(t, _)| t)
            .or(hovered)
            .or_else(|| {
                [Thumb::Lo, Thumb::Hi]
                    .into_iter()
                    .find(|t| state.focus[*t as usize].0)
            })
            .unwrap_or(Thumb::Hi);
        for t in [top.other(), top]
            .into_iter()
            .filter(|t| !self.single || *t == Thumb::Hi)
        {
            let st = slider::default(theme, status(t));
            let c = self.center(b, self.value.get(t));
            if enabled && state.focus[t as usize].0 {
                let ring = theme.metrics.focus_ring_width;
                renderer.fill_quad(
                    Quad {
                        bounds: Rectangle::new(
                            Point::new(c.x - R - ring, c.y - R - ring),
                            Size::new(2.0 * (R + ring), 2.0 * (R + ring)),
                        ),
                        border: Border {
                            radius: (R + ring).into(),
                            width: ring,
                            color: crate::color::alpha(theme.semantic.accent.bg, 0.6),
                        },
                        shadow: Shadow::default(),
                        snap: false,
                    },
                    Background::Color(Color::TRANSPARENT),
                );
            }
            // Opaque even when disabled (a faded handle would show the fill through it): the
            // handle is mixed towards the window instead.
            let bg = match st.handle.background {
                Background::Color(c) => {
                    Background::Color(crate::color::mix(theme.semantic.window.bg, c, k))
                }
                other => other,
            };
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle::new(
                        Point::new(c.x - R, c.y - R),
                        Size::new(2.0 * R, 2.0 * R),
                    ),
                    border: Border {
                        radius: R.into(),
                        width: st.handle.border_width,
                        color: dim(st.handle.border_color),
                    },
                    shadow: Shadow::default(),
                    snap: false,
                },
                bg,
            );
        }
    }
}

impl<'a, M: 'a, T: Scalar + 'a> From<RangeSlider<'a, M, T>> for Element<'a, M, Theme> {
    fn from(s: RangeSlider<'a, M, T>) -> Self {
        Element::new(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::{clipboard, renderer::Headless};

    #[test]
    fn single_and_range_controls_follow_orientation_and_preserve_grabs() {
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("software renderer");
        for axis in [Axis::Horizontal, Axis::Vertical] {
            for single in [false, true] {
                let mut control =
                    RangeSlider::new(0.0..=100.0, (if single { 0.0 } else { 20.0 }, 60.0), |v| v)
                        .orientation(axis)
                        .width(220.0)
                        .height(220.0)
                        .step(5.0);
                control.single = single;
                let mut tree = Tree::new(&control as &dyn Widget<_, _, _>);
                let node = control.layout(
                    &mut tree,
                    &renderer,
                    &layout::Limits::new(Size::ZERO, Size::new(220.0, 220.0)),
                );
                let layout = Layout::new(&node);
                let bounds = layout.bounds();
                let center = control.center(bounds, 60.0);
                let grab = match axis {
                    Axis::Horizontal => Point::new(center.x + 3.0, center.y),
                    Axis::Vertical => Point::new(center.x, center.y - 3.0),
                };
                let target = control.center(bounds, 80.0);
                let target = match axis {
                    Axis::Horizontal => Point::new(target.x + 3.0, target.y),
                    Axis::Vertical => Point::new(target.x, target.y - 3.0),
                };
                let mut messages = vec![];
                for (event, point) in [
                    (mouse::Event::ButtonPressed(mouse::Button::Left), grab),
                    (mouse::Event::CursorMoved { position: target }, target),
                    (mouse::Event::ButtonReleased(mouse::Button::Left), target),
                ] {
                    control.update(
                        &mut tree,
                        &Event::Mouse(event),
                        layout,
                        mouse::Cursor::Available(point),
                        &renderer,
                        &mut clipboard::Null,
                        &mut Shell::new(&mut messages),
                        &bounds,
                    );
                }
                assert_eq!(
                    messages,
                    vec![(if single { 0.0 } else { 20.0 }, 80.0)],
                    "{axis:?}, single={single}"
                );
                assert!(tree.state.downcast_ref::<State>().drag.is_none());

                // A rail press must move the high thumb even near the minimum for single controls.
                if single {
                    let point = control.center(bounds, 10.0);
                    messages.clear();
                    control.update(
                        &mut tree,
                        &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                        layout,
                        mouse::Cursor::Available(point),
                        &renderer,
                        &mut clipboard::Null,
                        &mut Shell::new(&mut messages),
                        &bounds,
                    );
                    assert_eq!(messages, vec![(0.0, 10.0)]);
                }
            }
        }
    }
}
