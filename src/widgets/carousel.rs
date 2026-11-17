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
use crate::layouts::animation::Trigger;
use crate::layouts::animation::{Progress, Spec, presets};
use crate::layouts::boundary::{self, SurfaceShape};
use crate::layouts::motion::draw_translated;
use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Operation, Tree, Widget, tree};
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
    /// How long a page change takes; the default spring is stretched to it.
    pub fn duration(mut self, d: std::time::Duration) -> Self {
        self.spec = self.spec.with_duration(d);
        self
    }

    fn page_area(&self, layout: Layout<'_>) -> Rectangle {
        Rectangle {
            height: self.page_h,
            ..layout.bounds()
        }
    }

    fn resting_page(&self, state: &State) -> Option<usize> {
        (!state.dragging
            && !self.pages.is_empty()
            && (state.progress.shown - state.progress.shown.round()).abs() < 0.001)
            .then(|| (state.progress.shown.round() as usize).min(self.pages.len() - 1))
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
    /// The scroll position in pages, and the value shown this frame (`progress.shown`).
    progress: Progress,
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

// The draw path uses motion::draw_translated. Events and hit tests must enter
// that same child coordinate system, including every inherited surface.
fn in_page_coordinates<T>(offset: f32, f: impl FnOnce() -> T) -> T {
    fn enter<T>(shapes: &[SurfaceShape], offset: f32, f: impl FnOnce() -> T) -> T {
        if let Some((shape, rest)) = shapes.split_first() {
            let mut shape = *shape;
            shape.bounds.x += offset;
            boundary::with_surface(shape, || enter(rest, offset, f))
        } else {
            f()
        }
    }
    let inherited = boundary::current_boundary();
    boundary::with_portal(|| enter(&inherited.shapes, offset, f))
}

fn page_cursor(cursor: mouse::Cursor, visible: Rectangle, offset: f32) -> mouse::Cursor {
    let cursor = boundary::current_boundary().cursor(cursor);
    if cursor.is_over(visible) {
        cursor + Vector::new(offset, 0.0)
    } else {
        mouse::Cursor::Unavailable
    }
}

fn page_event(event: &Event, offset: f32) -> Event {
    let shift = |position: Point| position + Vector::new(offset, 0.0);
    match event {
        Event::Mouse(mouse::Event::CursorMoved { position }) => {
            Event::Mouse(mouse::Event::CursorMoved {
                position: shift(*position),
            })
        }
        Event::Touch(event) => {
            use iced::touch::Event::*;
            Event::Touch(match *event {
                FingerPressed { id, position } => FingerPressed {
                    id,
                    position: shift(position),
                },
                FingerMoved { id, position } => FingerMoved {
                    id,
                    position: shift(position),
                },
                FingerLifted { id, position } => FingerLifted {
                    id,
                    position: shift(position),
                },
                FingerLost { id, position } => FingerLost {
                    id,
                    position: shift(position),
                },
            })
        }
        _ => event.clone(),
    }
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
            progress: Progress::new(self.index as f32),
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
            let moving = if state.dragging {
                state.progress.step(*now)
            } else {
                state.progress.drive(
                    self.index.min(n - 1) as f32,
                    *now,
                    Trigger::Carousel.spec(self.spec),
                )
            };
            if moving {
                shell.request_redraw();
            }
        }

        let page_area = Rectangle {
            height: self.page_h,
            ..bounds
        };
        let visible = page_area.intersection(viewport).unwrap_or_default();
        let cursor = boundary::current_boundary().cursor(cursor);
        let over_pages = cursor.is_over(visible);

        let was_dragging = state.dragging;
        if self.interactive {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) if over_pages => {
                    let x = cursor.position().map(|p| p.x).unwrap_or(0.0);
                    state.press = Some(Press {
                        x0: x,
                        pos0: state.progress.shown,
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
                            state.progress.set_immediate(pos);
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
                        let target = (state.progress.shown + pr.velocity * 0.12)
                            .round()
                            .clamp(0.0, max_pos) as usize;
                        let now = Instant::now();
                        state.progress.fling_to(
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
                && let Some(p) = cursor.position_over(*viewport)
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
            if !was_dragging && let Some(press) = state.press {
                // The page saw the original press. Cancel it when the carousel
                // takes the gesture, otherwise buttons remain pressed after a swipe.
                let i = press.pos0.round().clamp(0.0, max_pos) as usize;
                let off = state.progress.shown * pitch;
                let view = Rectangle {
                    x: visible.x + off,
                    ..visible
                };
                let mut messages = Vec::new();
                let mut cancellation = Shell::new(&mut messages);
                in_page_coordinates(off, || {
                    self.pages[i].as_widget_mut().update(
                        &mut tree.children[i],
                        &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                        layout.children().nth(i).unwrap(),
                        mouse::Cursor::Unavailable,
                        renderer,
                        clipboard,
                        &mut cancellation,
                        &view,
                    )
                });
                shell.merge(cancellation, |message| message);
            }
            return;
        }
        // Intermediate pages remain inert; releases still reach a resting page
        // with an unavailable cursor so existing child gestures can terminate.
        if let Some(i) = self.resting_page(state) {
            let off = state.progress.shown * pitch;
            let cur = page_cursor(cursor, visible, off);
            let ev = page_event(event, off);
            if let Event::Touch(iced::touch::Event::FingerPressed { position, .. }) = event
                && (!visible.contains(*position)
                    || !boundary::current_boundary().contains(*position))
            {
                return;
            }
            let view = Rectangle {
                x: visible.x + off,
                ..visible
            };
            in_page_coordinates(off, || {
                self.pages[i].as_widget_mut().update(
                    &mut tree.children[i],
                    &ev,
                    layout.children().nth(i).unwrap(),
                    cur,
                    renderer,
                    clipboard,
                    shell,
                    &view,
                );
            });
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
        let n = self.pages.len();
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();
        if state.dragging {
            return mouse::Interaction::Grabbing;
        }
        let cursor = boundary::current_boundary().cursor(cursor);
        if let Some(p) = cursor.position_over(*viewport) {
            if self.interactive
                && self.indicator != CarouselIndicator::None
                && indicator_hit(self.indicator, n, bounds, self.page_h, p).is_some()
            {
                return mouse::Interaction::Pointer;
            }
            let visible = self
                .page_area(layout)
                .intersection(viewport)
                .unwrap_or_default();
            if let Some(i) = self.resting_page(state) {
                let off = state.progress.shown * (self.page_w + self.spacing);
                let cur = page_cursor(cursor, visible, off);
                let view = Rectangle {
                    x: visible.x + off,
                    ..visible
                };
                let interaction = in_page_coordinates(off, || {
                    self.pages[i].as_widget().mouse_interaction(
                        &tree.children[i],
                        layout.children().nth(i).unwrap(),
                        cur,
                        &view,
                        renderer,
                    )
                });
                if interaction != mouse::Interaction::None {
                    return interaction;
                }
            }
            if self.interactive && visible.contains(p) && n > 1 {
                return mouse::Interaction::Grab;
            }
        }
        mouse::Interaction::None
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        let state = tree.state.downcast_ref::<State>();
        if let Some(i) = self.resting_page(state) {
            let off = state.progress.shown * (self.page_w + self.spacing);
            in_page_coordinates(off, || {
                self.pages[i].as_widget_mut().operate(
                    &mut tree.children[i],
                    layout.children().nth(i).unwrap(),
                    renderer,
                    operation,
                )
            });
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
        let state = tree.state.downcast_ref::<State>();
        let i = self.resting_page(state)?;
        let visible = self.page_area(layout).intersection(viewport)?;
        let off = state.progress.shown * (self.page_w + self.spacing);
        self.pages[i].as_widget_mut().overlay(
            &mut tree.children[i],
            layout.children().nth(i)?,
            renderer,
            &visible,
            translation - Vector::new(off, 0.0),
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
        let n = self.pages.len();
        if n == 0 {
            return;
        }
        let bounds = layout.bounds();
        let pos = tree.state.downcast_ref::<State>().progress.shown;
        let pitch = self.page_w + self.spacing;
        let off = pos * pitch;
        let page_area = Rectangle {
            height: self.page_h,
            ..bounds
        };

        if let Some(visible) = page_area.intersection(viewport) {
            let cur = page_cursor(cursor, visible, off);
            renderer.with_layer(visible, |r| {
                // Slide everything left by `off`, clipping to the part of the pages that is visible
                // (in the pages' own coordinates), like every sliding child.
                draw_translated(r, page_area, viewport, Vector::new(-off, 0.0), |r, vp| {
                    for (i, (page, node)) in self.pages.iter().zip(layout.children()).enumerate() {
                        if (i as f32 - pos).abs() < 1.0 + self.spacing / self.page_w.max(1.0) {
                            page.as_widget().draw(
                                &tree.children[i],
                                r,
                                theme,
                                style,
                                node,
                                cur,
                                vp,
                            );
                        }
                    }
                });
            });
        }

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
    use crate::components::{ContainerProps, TooltipPosition, TooltipProps, tooltip_with_props};
    use crate::widgets::Card;
    use iced::advanced::{clipboard, renderer::Headless};

    fn renderer() -> iced::Renderer {
        iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("software renderer")
    }

    fn viewport() -> Rectangle {
        Rectangle::with_size(Size::new(160.0, 100.0))
    }

    fn card_bounds() -> Rectangle {
        Rectangle::new(Point::new(20.0, 20.0), Size::new(120.0, 60.0))
    }

    fn card(
        content: impl Into<Element<'static, usize, Theme>>,
        radius: iced::border::Radius,
    ) -> Element<'static, usize, Theme> {
        Card::new(content)
            .padding(0)
            .width(120)
            .height(60)
            .container_props(ContainerProps::default().style(move |_| {
                iced::widget::container::Style {
                    border: Border::default().rounded(radius),
                    ..Default::default()
                }
            }))
            .into()
    }

    fn prepare(
        element: &mut Element<'_, usize, Theme>,
        r: &iced::Renderer,
    ) -> (Tree, layout::Node) {
        let mut tree = Tree::new(element.as_widget());
        let node = element
            .as_widget_mut()
            .layout(
                &mut tree,
                r,
                &layout::Limits::new(card_bounds().size(), card_bounds().size()),
            )
            .move_to(card_bounds().position());
        assert_eq!(Layout::new(&node).bounds(), card_bounds());
        (tree, node)
    }

    fn carousel_state(tree: &mut Tree) -> &mut State {
        if tree.tag == tree::Tag::of::<State>() {
            return tree.state.downcast_mut::<State>();
        }
        // Card owns several real wrappers; locate the carousel without coupling
        // these regressions to their tree depth.
        fn contains(tree: &Tree) -> bool {
            tree.tag == tree::Tag::of::<State>() || tree.children.iter().any(contains)
        }
        carousel_state(
            tree.children
                .iter_mut()
                .find(|child| contains(child))
                .expect("carousel tree"),
        )
    }

    fn draw(
        element: &Element<'_, usize, Theme>,
        tree: &Tree,
        node: &layout::Node,
        r: &mut iced::Renderer,
    ) -> Vec<u8> {
        let mut theme = Theme::light();
        theme.shadows.enabled = false;
        r.reset(viewport());
        element.as_widget().draw(
            tree,
            r,
            &theme,
            &renderer::Style::default(),
            Layout::new(node),
            mouse::Cursor::Unavailable,
            &viewport(),
        );
        assert!(boundary::current_boundary().shapes.is_empty());
        r.screenshot(Size::new(160, 100), 1.0, Color::BLACK)
    }

    fn pixel(image: &[u8], x: usize, y: usize) -> &[u8] {
        &image[(y * 160 + x) * 4..(y * 160 + x + 1) * 4]
    }

    struct OversizedPage;
    impl Widget<usize, Theme, iced::Renderer> for OversizedPage {
        fn size(&self) -> Size<Length> {
            Size::new(Length::Fill, Length::Fill)
        }
        fn layout(
            &mut self,
            _: &mut Tree,
            _: &iced::Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            layout::Node::new(limits.max())
        }
        fn draw(
            &self,
            _: &Tree,
            r: &mut iced::Renderer,
            _: &Theme,
            _: &renderer::Style,
            layout: Layout<'_>,
            _: mouse::Cursor,
            _: &Rectangle,
        ) {
            // Deliberately ignore the viewport, as an overflowing opaque child can.
            r.fill_quad(
                Quad {
                    bounds: Rectangle {
                        x: layout.bounds().x - 200.0,
                        y: layout.bounds().y - 200.0,
                        width: 600.0,
                        height: 460.0,
                    },
                    snap: false,
                    ..Default::default()
                },
                Color::from_rgb(1.0, 0.0, 0.0),
            );
        }
    }

    // Independent oracle for the final Card styles below. Boundary uses
    // conservative whole-pixel bands, so all four pixel corners must fit.
    fn inside_card_pixel(x: usize, y: usize, radii: [f32; 4]) -> bool {
        [(0.0, 0.0), (1.0, 0.0), (0.0, 1.0), (1.0, 1.0)]
            .into_iter()
            .all(|(dx, dy)| {
                let x = x as f32 + dx - 20.0;
                let y = y as f32 + dy - 20.0;
                if !(0.0..=120.0).contains(&x) || !(0.0..=60.0).contains(&y) {
                    return false;
                }
                [
                    (radii[0], 0.0, 0.0),
                    (radii[1], 120.0, 0.0),
                    (radii[2], 0.0, 60.0),
                    (radii[3], 120.0, 60.0),
                ]
                .into_iter()
                .all(|(radius, cx, cy)| {
                    let dx = (x - cx).abs();
                    let dy = (y - cy).abs();
                    dx >= radius
                        || dy >= radius
                        || (dx - radius).powi(2) + (dy - radius).powi(2) <= radius.powi(2)
                })
            })
    }

    #[test]
    fn rendered_actual_card_clips_oversized_carousel_pages_at_rest_and_in_motion() {
        let mut r = renderer();
        for (radius, normalized) in [
            (0.0.into(), [0.0; 4]),
            (999.0.into(), [30.0; 4]),
            (
                iced::border::Radius {
                    top_left: 25.0,
                    top_right: 0.0,
                    bottom_left: 8.0,
                    bottom_right: 19.0,
                },
                [25.0, 0.0, 8.0, 19.0],
            ),
        ] {
            for spacing in [0.0, 9.0] {
                let mut element = card(
                    carousel(
                        vec![Element::new(OversizedPage), Element::new(OversizedPage)],
                        0,
                        |i| i,
                    )
                    .indicator(CarouselIndicator::None)
                    .spacing(spacing),
                    radius,
                );
                let (mut tree, node) = prepare(&mut element, &r);
                for progress in [0.0, 0.25, 0.5, 0.75, 1.0] {
                    carousel_state(&mut tree).progress.set_immediate(progress);
                    let image = draw(&element, &tree, &node, &mut r);
                    assert_eq!(
                        pixel(&image, 80, 50),
                        &[255, 0, 0, 255],
                        "content must render"
                    );
                    for y in 0..100 {
                        for x in 0..160 {
                            let expected: &[u8] = if inside_card_pixel(x, y, normalized) {
                                &[255, 0, 0, 255]
                            } else {
                                &[0, 0, 0, 255]
                            };
                            assert_eq!(
                                pixel(&image, x, y),
                                expected,
                                "pixel ({x},{y}), radius={radius:?}, progress={progress}, spacing={spacing}"
                            );
                        }
                    }
                }
            }
        }
    }

    fn update(
        element: &mut Element<'_, usize, Theme>,
        tree: &mut Tree,
        node: &layout::Node,
        r: &iced::Renderer,
        event: Event,
        position: Point,
    ) -> Vec<usize> {
        let mut messages = Vec::new();
        element.as_widget_mut().update(
            tree,
            &event,
            Layout::new(node),
            mouse::Cursor::Available(position),
            r,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
            &viewport(),
        );
        messages
    }

    fn click(
        element: &mut Element<'_, usize, Theme>,
        tree: &mut Tree,
        node: &layout::Node,
        r: &iced::Renderer,
        position: Point,
    ) -> Vec<usize> {
        let mut messages = Vec::new();
        for event in [
            mouse::Event::ButtonPressed(mouse::Button::Left),
            mouse::Event::ButtonReleased(mouse::Button::Left),
        ] {
            messages.extend(update(
                element,
                tree,
                node,
                r,
                Event::Mouse(event),
                position,
            ));
        }
        messages
    }

    fn button_page(index: usize) -> Element<'static, usize, Theme> {
        iced::widget::button(iced::widget::text(format!("Page {index}")))
            .width(Length::Fill)
            .height(Length::Fill)
            .on_press(index)
            .into()
    }

    #[test]
    fn actual_card_forwards_only_resting_page_buttons_and_cancels_swipes() {
        let mut r = renderer();
        let mut element = card(
            carousel(vec![button_page(0), button_page(1)], 1, |i| 10 + i)
                .indicator(CarouselIndicator::None)
                .spacing(9.0),
            999.0.into(),
        );
        let (mut tree, node) = prepare(&mut element, &r);
        draw(&element, &tree, &node, &mut r); // resolve the final Card radius
        let center = Point::new(80.0, 50.0);
        let corner = Point::new(20.5, 20.5);
        assert_eq!(click(&mut element, &mut tree, &node, &r, center), vec![1]);
        assert!(click(&mut element, &mut tree, &node, &r, corner).is_empty());
        assert_eq!(
            element.as_widget().mouse_interaction(
                &tree,
                Layout::new(&node),
                mouse::Cursor::Available(center),
                &viewport(),
                &r
            ),
            mouse::Interaction::Pointer
        );
        assert_eq!(
            element.as_widget().mouse_interaction(
                &tree,
                Layout::new(&node),
                mouse::Cursor::Available(corner),
                &viewport(),
                &r
            ),
            mouse::Interaction::None
        );
        carousel_state(&mut tree).progress.set_immediate(0.5);
        assert!(click(&mut element, &mut tree, &node, &r, center).is_empty());
        carousel_state(&mut tree).progress.set_immediate(0.0);
        assert_eq!(click(&mut element, &mut tree, &node, &r, center), vec![0]);
        update(
            &mut element,
            &mut tree,
            &node,
            &r,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            center,
        );
        let moved = Point::new(45.0, 50.0);
        assert!(
            update(
                &mut element,
                &mut tree,
                &node,
                &r,
                Event::Mouse(mouse::Event::CursorMoved { position: moved }),
                moved
            )
            .is_empty()
        );
        let release = update(
            &mut element,
            &mut tree,
            &node,
            &r,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
            moved,
        );
        assert!(
            release.iter().all(|message| *message >= 10),
            "swipe must not activate page button"
        );
        carousel_state(&mut tree).progress.set_immediate(0.0);
        assert!(
            update(
                &mut element,
                &mut tree,
                &node,
                &r,
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                center
            )
            .is_empty()
        );
        assert_eq!(click(&mut element, &mut tree, &node, &r, center), vec![0]);
    }

    #[derive(Default)]
    struct PageText(Vec<String>);
    impl Operation for PageText {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn text(&mut self, _: Option<&iced::advanced::widget::Id>, _: Rectangle, text: &str) {
            self.0.push(text.to_owned());
        }
    }

    #[test]
    fn actual_card_operates_only_on_resting_carousel_page() {
        let r = renderer();
        let mut element = card(
            carousel(vec![button_page(0), button_page(1)], 0, |i| i)
                .indicator(CarouselIndicator::None),
            30.0.into(),
        );
        let (mut tree, node) = prepare(&mut element, &r);
        for (progress, expected) in [(0.0, vec!["Page 0"]), (1.0, vec!["Page 1"]), (0.5, vec![])] {
            carousel_state(&mut tree).progress.set_immediate(progress);
            let mut operation = PageText::default();
            element
                .as_widget_mut()
                .operate(&mut tree, Layout::new(&node), &r, &mut operation);
            assert_eq!(operation.0, expected, "progress={progress}");
        }
    }

    #[test]
    fn actual_card_forwards_translated_tooltip_portal_outside_rounded_surface() {
        let mut r = renderer();
        let tooltip_page = tooltip_with_props(
            button_page(1),
            "Page tooltip",
            TooltipPosition::Top,
            TooltipProps::default()
                .delay(std::time::Duration::ZERO)
                .gap(2.0)
                .padding(0)
                .style(|_| iced::widget::container::Style {
                    background: Some(Color::from_rgb(0.0, 1.0, 0.0).into()),
                    ..Default::default()
                }),
        );
        let mut element = card(
            carousel(vec![button_page(0), tooltip_page], 1, |i| i)
                .indicator(CarouselIndicator::None)
                .spacing(9.0),
            999.0.into(),
        );
        let (mut tree, node) = prepare(&mut element, &r);
        draw(&element, &tree, &node, &mut r);
        let center = Point::new(80.0, 50.0);
        update(
            &mut element,
            &mut tree,
            &node,
            &r,
            Event::Mouse(mouse::Event::CursorMoved { position: center }),
            center,
        );
        {
            let mut popup = element
                .as_widget_mut()
                .overlay(&mut tree, Layout::new(&node), &r, &viewport(), Vector::ZERO)
                .expect("resting page tooltip forwarded through Card");
            let popup_node = popup.as_overlay_mut().layout(&r, viewport().size());
            r.reset(viewport());
            // A portal must clear even an ambient parent boundary at draw time.
            boundary::with_surface(SurfaceShape::new(card_bounds(), 30.0), || {
                popup.as_overlay().draw(
                    &mut r,
                    &Theme::light(),
                    &renderer::Style::default(),
                    Layout::new(&popup_node),
                    mouse::Cursor::Available(center),
                );
            });
            let image = r.screenshot(Size::new(160, 100), 1.0, Color::BLACK);
            assert!(
                (0..20).any(|y| (20..140).any(|x| pixel(&image, x, y) == [0, 255, 0, 255])),
                "tooltip must render above Card, at its translated on-screen anchor"
            );
            assert!(
                (0..20).all(|y| (140..160).all(|x| pixel(&image, x, y) == [0, 0, 0, 255])),
                "tooltip must not retain the second page's unscrolled x coordinate"
            );
        }
        carousel_state(&mut tree).progress.set_immediate(0.5);
        assert!(
            element
                .as_widget_mut()
                .overlay(&mut tree, Layout::new(&node), &r, &viewport(), Vector::ZERO)
                .is_none(),
            "moving pages must not expose portals"
        );
    }

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
