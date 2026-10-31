//! `AdwCarousel`: paginated, swipeable pages.
//!
//! * **Animated.** The position scrolls with libadwaita's default scroll spring
//!   (damping ratio 1, mass 0.5, stiffness 500; [`presets::sidebar`]).
//! * **Swipeable.** Drag with the mouse; on release the drag velocity is handed to the spring
//!   (`initial-velocity`) and the carousel settles on the nearest page, publishing
//!   `on_navigate`.
//! * **Indicators follow the position.** Dots grow and brighten gradually as their page nears
//!   (`AdwCarouselIndicatorDots`); with lines, the active line glides between pages
//!   (`AdwCarouselIndicatorLines`). Clicking an indicator navigates.
//!
//! State is controlled: the app owns `index` and updates it from `on_navigate`.

use crate::Theme;
use crate::animation::Trigger;
use crate::animation::{Spec, Tween, presets};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::{
    Background, Border, Color, Element, Event, Length, Point, Rectangle, Size, Vector, mouse,
    window,
};
use std::time::Instant;

/// Which indicator a [`Carousel`] draws below its pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CarouselIndicator {
    #[default]
    Dots,
    Lines,
    None,
}

impl CarouselIndicator {
    pub const ALL: [Self; 3] = [Self::Dots, Self::Lines, Self::None];
}

impl std::fmt::Display for CarouselIndicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Dots => "Dots",
            Self::Lines => "Lines",
            Self::None => "None",
        })
    }
}

const INDICATOR_HEIGHT: f32 = 24.0;
const DRAG_THRESHOLD: f32 = 6.0;

pub struct Carousel<'a, M> {
    pages: Vec<Element<'a, M, Theme>>,
    index: usize,
    on_navigate: Box<dyn Fn(usize) -> M + 'a>,
    indicator: CarouselIndicator,
    spacing: f32,
    interactive: bool,
    spec: Spec,
    // layout results
    page_w: f32,
    page_h: f32,
}

/// An animated, swipeable carousel showing page `index`.
pub fn carousel<'a, M>(
    pages: Vec<Element<'a, M, Theme>>,
    index: usize,
    on_navigate: impl Fn(usize) -> M + 'a,
) -> Carousel<'a, M> {
    Carousel {
        pages,
        index,
        on_navigate: Box::new(on_navigate),
        indicator: CarouselIndicator::Dots,
        spacing: 0.0,
        interactive: true,
        spec: Spec::Spring(presets::sidebar()),
        page_w: 0.0,
        page_h: 0.0,
    }
}

impl<'a, M> Carousel<'a, M> {
    pub fn indicator(mut self, i: CarouselIndicator) -> Self {
        self.indicator = i;
        self
    }
    /// Space between pages while scrolling.
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = px;
        self
    }
    /// `interactive`: allow swiping and clicking indicators (default true).
    pub fn interactive(mut self, i: bool) -> Self {
        self.interactive = i;
        self
    }
    pub fn spec(mut self, spec: Spec) -> Self {
        self.spec = spec;
        self
    }

    fn indicator_height(&self) -> f32 {
        if self.indicator == CarouselIndicator::None {
            0.0
        } else {
            INDICATOR_HEIGHT
        }
    }
}

struct State {
    position: Tween,
    shown: f32,
    /// `(cursor x at press, position at press, last x, last time, velocity px/s)`.
    press: Option<Press>,
    dragging: bool,
}

#[derive(Clone, Copy)]
struct Press {
    x0: f32,
    pos0: f32,
    last_x: f32,
    last_t: Instant,
    velocity: f32,
}

/// Indicator geometry: `(pitch between item centers, item size along x)`.
fn indicator_slots(kind: CarouselIndicator, _n: usize) -> (f32, f32) {
    match kind {
        CarouselIndicator::Dots => (14.0, 8.0),
        CarouselIndicator::Lines => (40.0, 35.0),
        CarouselIndicator::None => (0.0, 0.0),
    }
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for Carousel<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State {
            position: Tween::new(self.index as f32),
            shown: self.index as f32,
            press: None,
            dragging: false,
        })
    }
    fn children(&self) -> Vec<Tree> {
        self.pages.iter().map(Tree::new).collect()
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.pages.iter().map(|p| p.as_widget()).collect::<Vec<_>>());
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let max = limits.max();
        let w = if max.width.is_finite() {
            max.width
        } else {
            320.0
        };
        let total_h = if max.height.is_finite() {
            max.height
        } else {
            180.0
        };
        let h = (total_h - self.indicator_height()).max(0.0);
        self.page_w = w;
        self.page_h = h;
        let nodes = self
            .pages
            .iter_mut()
            .zip(&mut tree.children)
            .enumerate()
            .map(|(i, (page, t))| {
                let node = page.as_widget_mut().layout(
                    t,
                    renderer,
                    &layout::Limits::new(Size::new(w, h), Size::new(w, h)),
                );
                node.move_to(Point::new(i as f32 * (w + self.spacing), 0.0))
            })
            .collect();
        layout::Node::with_children(Size::new(w, total_h), nodes)
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
        let n = self.pages.len();
        if n == 0 {
            return;
        }
        let bounds = layout.bounds();
        let pitch = self.page_w + self.spacing;
        let state = tree.state.downcast_mut::<State>();
        let max_pos = (n - 1) as f32;

        // Animate the scroll position toward the current page (unless the user holds it).
        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            if !state.dragging {
                state.position.set_target(
                    self.index.min(n - 1) as f32,
                    *now,
                    Trigger::Carousel.spec(self.spec),
                );
            }
            let v = state.position.tick(*now);
            if (v - state.shown).abs() > f32::EPSILON || state.position.is_animating() {
                state.shown = v;
                shell.request_redraw();
            }
        }

        let page_area = Rectangle {
            height: self.page_h,
            ..bounds
        };
        let over_pages = cursor.position().is_some_and(|p| page_area.contains(p));

        if self.interactive {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if over_pages => {
                    let x = cursor.position().map(|p| p.x).unwrap_or(0.0);
                    state.press = Some(Press {
                        x0: x,
                        pos0: state.shown,
                        last_x: x,
                        last_t: Instant::now(),
                        velocity: 0.0,
                    });
                }
                Event::Mouse(mouse::Event::CursorMoved { position }) => {
                    if let Some(mut pr) = state.press {
                        let dx = position.x - pr.x0;
                        if !state.dragging && dx.abs() > DRAG_THRESHOLD {
                            state.dragging = true;
                        }
                        if state.dragging {
                            // Rubber-band past the ends.
                            let raw = pr.pos0 - dx / pitch;
                            let pos = if raw < 0.0 {
                                raw * 0.35
                            } else if raw > max_pos {
                                max_pos + (raw - max_pos) * 0.35
                            } else {
                                raw
                            };
                            state.position.set_immediate(pos);
                            state.shown = pos;
                            let now = Instant::now();
                            let dt = now.duration_since(pr.last_t).as_secs_f32().max(1e-3);
                            // Smoothed velocity in pages per second (screen motion → page motion).
                            let inst = -(position.x - pr.last_x) / pitch / dt;
                            pr.velocity = pr.velocity * 0.6 + inst * 0.4;
                            pr.last_x = position.x;
                            pr.last_t = now;
                            state.press = Some(pr);
                            shell.capture_event();
                            shell.request_redraw();
                        }
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                    if state.dragging {
                        let pr = state.press.unwrap();
                        // Project a little along the velocity, then snap to the nearest page.
                        let target = (state.shown + pr.velocity * 0.12)
                            .round()
                            .clamp(0.0, max_pos) as usize;
                        let now = Instant::now();
                        state.position.fling_to(
                            target as f32,
                            pr.velocity,
                            now,
                            Trigger::Carousel.spec(self.spec),
                        );
                        state.dragging = false;
                        state.press = None;
                        if target != self.index {
                            shell.publish((self.on_navigate)(target));
                        }
                        shell.capture_event();
                        shell.request_redraw();
                        return;
                    }
                    state.press = None;
                }
                _ => {}
            }
            // Indicator clicks.
            if self.indicator != CarouselIndicator::None
                && let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
                && let Some(p) = cursor.position()
                && let Some(i) = indicator_hit(self.indicator, n, bounds, self.page_h, p)
            {
                if i != self.index {
                    shell.publish((self.on_navigate)(i));
                }
                shell.capture_event();
                return;
            }
        }

        if state.dragging {
            return;
        }
        // Forward events to the current page when the carousel is at rest.
        let settled = (state.shown - state.shown.round()).abs() < 0.001;
        if settled {
            let i = (state.shown.round() as usize).min(n - 1);
            let off = i as f32 * pitch;
            let shift = |p: Point| Point::new(p.x + off, p.y);
            let cur = match cursor {
                mouse::Cursor::Available(p) => mouse::Cursor::Available(shift(p)),
                other => other,
            };
            let ev = match event {
                Event::Mouse(mouse::Event::CursorMoved { position }) => {
                    Event::Mouse(mouse::Event::CursorMoved {
                        position: shift(*position),
                    })
                }
                other => other.clone(),
            };
            let page_layout = layout.children().nth(i).unwrap();
            self.pages[i].as_widget_mut().update(
                &mut tree.children[i],
                &ev,
                page_layout,
                cur,
                renderer,
                clipboard,
                shell,
                viewport,
            );
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
        let n = self.pages.len();
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();
        if state.dragging {
            return mouse::Interaction::Grabbing;
        }
        if let Some(p) = cursor.position() {
            if self.indicator != CarouselIndicator::None
                && indicator_hit(self.indicator, n, bounds, self.page_h, p).is_some()
            {
                return mouse::Interaction::Pointer;
            }
            if self.interactive && bounds.contains(p) && n > 1 {
                return mouse::Interaction::Grab;
            }
        }
        mouse::Interaction::None
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
        let n = self.pages.len();
        if n == 0 {
            return;
        }
        let bounds = layout.bounds();
        let pos = tree.state.downcast_ref::<State>().shown;
        let pitch = self.page_w + self.spacing;
        let off = pos * pitch;
        let page_area = Rectangle {
            height: self.page_h,
            ..bounds
        };

        renderer.with_layer(page_area, |r| {
            // Slide everything left by `off`; shift the viewport back so clipping still
            // matches the (translated) page.
            let vp = Rectangle {
                x: viewport.x + off,
                ..*viewport
            };
            r.with_translation(Vector::new(-off, 0.0), |r| {
                for (i, (page, node)) in self.pages.iter().zip(layout.children()).enumerate() {
                    if (i as f32 - pos).abs() < 1.0 + self.spacing / self.page_w.max(1.0) {
                        page.as_widget().draw(
                            &tree.children[i],
                            r,
                            theme,
                            style,
                            node,
                            cursor,
                            &vp,
                        );
                    }
                }
            });
        });

        // Indicator: follows the animated position.
        if self.indicator != CarouselIndicator::None {
            let fg = theme.semantic.window.fg;
            let (pitch_i, size) = indicator_slots(self.indicator, n);
            let span = (n as f32 - 1.0) * pitch_i;
            let cx = bounds.x + bounds.width / 2.0 - span / 2.0;
            let cy = bounds.y + self.page_h + INDICATOR_HEIGHT / 2.0;
            let quad = |renderer: &mut iced::Renderer, b: Rectangle, radius: f32, c: Color| {
                renderer.fill_quad(
                    Quad {
                        bounds: b,
                        border: Border::default().rounded(radius),
                        shadow: Default::default(),
                        snap: false,
                    },
                    Background::Color(c),
                );
            };
            match self.indicator {
                CarouselIndicator::Dots => {
                    for i in 0..n {
                        // 0 = far, 1 = the current page: grows and brightens gradually.
                        let near = (1.0 - (pos - i as f32).abs()).clamp(0.0, 1.0);
                        let d = size * (0.75 + 0.25 * near);
                        let alpha = 0.3 + 0.6 * near;
                        let x = cx + i as f32 * pitch_i;
                        quad(
                            renderer,
                            Rectangle::new(Point::new(x - d / 2.0, cy - d / 2.0), Size::new(d, d)),
                            d / 2.0,
                            crate::color::alpha(fg, alpha),
                        );
                    }
                }
                CarouselIndicator::Lines => {
                    let h = 3.0;
                    for i in 0..n {
                        let x = cx + i as f32 * pitch_i;
                        quad(
                            renderer,
                            Rectangle::new(
                                Point::new(x - size / 2.0, cy - h / 2.0),
                                Size::new(size, h),
                            ),
                            h / 2.0,
                            crate::color::alpha(fg, 0.3),
                        );
                    }
                    // The active line glides with the position.
                    let x = cx + pos.clamp(0.0, (n - 1) as f32) * pitch_i;
                    quad(
                        renderer,
                        Rectangle::new(
                            Point::new(x - size / 2.0, cy - h / 2.0),
                            Size::new(size, h),
                        ),
                        h / 2.0,
                        fg,
                    );
                }
                CarouselIndicator::None => {}
            }
        }
    }
}

/// Which indicator item (if any) the point `p` is over.
fn indicator_hit(
    kind: CarouselIndicator,
    n: usize,
    bounds: Rectangle,
    page_h: f32,
    p: Point,
) -> Option<usize> {
    if n == 0 || kind == CarouselIndicator::None {
        return None;
    }
    let (pitch, _) = indicator_slots(kind, n);
    let area = Rectangle {
        y: bounds.y + page_h,
        height: INDICATOR_HEIGHT,
        ..bounds
    };
    if !area.contains(p) {
        return None;
    }
    let span = (n as f32 - 1.0) * pitch;
    let cx = bounds.x + bounds.width / 2.0 - span / 2.0;
    let i = ((p.x - cx) / pitch).round();
    (i >= 0.0 && (i as usize) < n && (p.x - (cx + i * pitch)).abs() <= pitch / 2.0)
        .then_some(i as usize)
}

impl<'a, M: 'a> From<Carousel<'a, M>> for Element<'a, M, Theme> {
    fn from(c: Carousel<'a, M>) -> Self {
        Element::new(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indicator_hit_tests_items() {
        let b = Rectangle::new(Point::new(0.0, 0.0), Size::new(200.0, 124.0));
        // 3 dots, pitch 14, centered: centers at 86, 100, 114; row y in 100..124.
        assert_eq!(
            indicator_hit(
                CarouselIndicator::Dots,
                3,
                b,
                100.0,
                Point::new(100.0, 110.0)
            ),
            Some(1)
        );
        assert_eq!(
            indicator_hit(
                CarouselIndicator::Dots,
                3,
                b,
                100.0,
                Point::new(86.0, 110.0)
            ),
            Some(0)
        );
        assert_eq!(
            indicator_hit(
                CarouselIndicator::Dots,
                3,
                b,
                100.0,
                Point::new(100.0, 50.0)
            ),
            None
        );
        assert_eq!(
            indicator_hit(
                CarouselIndicator::None,
                3,
                b,
                100.0,
                Point::new(100.0, 110.0)
            ),
            None
        );
    }
}
