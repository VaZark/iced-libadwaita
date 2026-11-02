//! `RangeSlider`: a slider with two thumbs that selects a range (`lo..=hi`) of values; the
//! sibling of [`scale`](super::scale), drawn with the same rail and handle styling
//! ([`style::slider`](crate::style::slider)) with the accent fill between
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

use crate::Theme;
use crate::primitives::range::{Collide, Key, Range, Thumb, Track};
use crate::style::slider::{self, HANDLE_RADIUS};
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

type OnChange<'a, M> = Box<dyn Fn((f32, f32)) -> M + 'a>;

pub struct RangeSlider<'a, M> {
    value: Range<f32>,
    track: Track<f32>,
    on_change: Option<OnChange<'a, M>>,
    marks: Vec<f32>,
    disabled: bool,
    width: Length,
}

/// A `(lo, hi)` slider over `range`; the free-function form of [`RangeSlider::new`].
pub fn range_slider<'a, M>(
    range: RangeInclusive<f32>,
    value: (f32, f32),
    on_change: impl Fn((f32, f32)) -> M + 'a,
) -> RangeSlider<'a, M> {
    RangeSlider::new(range, value, on_change)
}

impl<'a, M> RangeSlider<'a, M> {
    /// `value` is `(lo, hi)` (put in order and clamped to `range`); `on_change` gets the new
    /// pair whenever a thumb moves. The step is `1.0`.
    pub fn new(
        range: RangeInclusive<f32>,
        value: (f32, f32),
        on_change: impl Fn((f32, f32)) -> M + 'a,
    ) -> Self {
        let track = Track::new(*range.start(), *range.end()).step(1.0);
        Self {
            value: track.clamp(Range::from(value)),
            track,
            on_change: Some(Box::new(on_change)),
            marks: Vec::new(),
            disabled: false,
            width: Length::Fill,
        }
    }
    /// The grid the thumbs snap to, from the start of the range; `0` is continuous.
    pub fn step(mut self, step: f32) -> Self {
        self.track = self.track.step(step as f64);
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
    pub fn marks(mut self, marks: impl IntoIterator<Item = f32>) -> Self {
        self.marks = marks.into_iter().collect();
        self
    }
    /// Ticks at every `every` from the start of the range.
    pub fn marks_every(self, every: f32) -> Self {
        let (min, max) = (self.track.min, self.track.max);
        let n = if every > 0.0 {
            ((max - min) / every + 1e-4) as usize
        } else {
            0
        };
        self.marks((0..=n).map(|i| min + i as f32 * every))
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

    fn enabled(&self) -> bool {
        !self.disabled && self.on_change.is_some()
    }
    fn height(&self) -> f32 {
        2.0 * R + if self.marks.is_empty() { 0.0 } else { MARKS }
    }
    /// The horizontal travel of a thumb's centre.
    fn travel(b: Rectangle) -> (f32, f32) {
        (b.x + R, (b.width - 2.0 * R).max(1.0))
    }
    fn center(&self, b: Rectangle, v: f32) -> Point {
        let (x0, w) = Self::travel(b);
        Point::new(x0 + self.track.fraction(v) * w, b.y + R)
    }
    /// The value under a pointer `x`, without snapping to a thumb.
    fn value_at(&self, b: Rectangle, x: f32) -> f32 {
        let (x0, w) = Self::travel(b);
        self.track.value_at((x - x0) / w)
    }
    /// The thumb whose handle `p` is on, the nearer when both are.
    fn thumb_at(&self, b: Rectangle, p: Point) -> Option<Thumb> {
        let d = |t: Thumb| self.center(b, self.value.get(t)).distance(p);
        let hit = |t: Thumb| d(t) <= R + 2.0;
        match (hit(Thumb::Lo), hit(Thumb::Hi)) {
            (true, true) => Some(self.track.nearest(self.value, self.value_at(b, p.x))),
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
    drag: Option<(Thumb, f32)>,
    focus: [Focus; 2],
}

impl State {
    fn focus_of(&mut self, t: Thumb) -> &mut Focus {
        &mut self.focus[t as usize]
    }
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for RangeSlider<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Fixed(self.height()))
    }
    fn layout(
        &mut self,
        _: &mut Tree,
        _: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, Length::Fixed(self.height()))
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
        for t in [Thumb::Lo, Thumb::Hi] {
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

        // Move `thumb` to the pointer at `x` and report the pair when it changed.
        let drag_to = |this: &Self, shell: &mut Shell<'_, M>, thumb: Thumb, x: f32| {
            let next = this
                .track
                .move_thumb(this.value, thumb, this.value_at(b, x));
            if next != this.value
                && let Some(f) = &this.on_change
            {
                shell.publish(f((next.lo, next.hi)));
            }
        };

        let press = |this: &Self, state: &mut State, shell: &mut Shell<'_, M>, p: Point| {
            let (thumb, offset) = match this.thumb_at(b, p) {
                Some(t) => (t, p.x - this.center(b, this.value.get(t)).x),
                None => {
                    let t = this.track.nearest(this.value, this.value_at(b, p.x));
                    (t, 0.0)
                }
            };
            state.drag = Some((thumb, offset));
            state.focus_of(thumb).focus();
            state.focus_of(thumb.other()).unfocus();
            drag_to(this, shell, thumb, p.x - offset);
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
                    drag_to(self, shell, thumb, position.x - offset);
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
        let (x0, w) = Self::travel(b);
        let y = b.y + R;

        // Rail: the trough, then the accent fill between the thumbs.
        let rail = base.rail;
        let rail_bounds = |from: f32, to: f32| Rectangle {
            x: from,
            y: y - rail.width / 2.0,
            width: (to - from).max(0.0),
            height: rail.width,
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
        let (cl, ch) = (
            self.center(b, self.value.lo).x,
            self.center(b, self.value.hi).x,
        );
        fill(
            renderer,
            rail_bounds(b.x, b.x + b.width),
            rail.backgrounds.1,
        );
        fill(renderer, rail_bounds(cl, ch), rail.backgrounds.0);

        // Marks under the rail.
        let tick = dim(crate::color::alpha(theme.semantic.window.fg, 0.35));
        for m in &self.marks {
            let x = x0 + self.track.fraction(*m) * w;
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle::new(
                        Point::new(x - 0.5, b.y + 2.0 * R + 3.0),
                        Size::new(1.0, 5.0),
                    ),
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
        for t in [top.other(), top] {
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

impl<'a, M: 'a> From<RangeSlider<'a, M>> for Element<'a, M, Theme> {
    fn from(s: RangeSlider<'a, M>) -> Self {
        Element::new(s)
    }
}
