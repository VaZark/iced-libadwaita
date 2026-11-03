//! `ColorPicker` and `ColorInput`, after Mantine's
//! [ColorPicker](https://mantine.dev/core/color-picker/) and
//! [ColorInput](https://mantine.dev/core/color-input/). The color math (hue, saturation, value,
//! alpha, text forms, parsing) is the [`color_picker`](crate::primitives::color_picker)
//! primitive; this draws the controls and turns the pointer into a new [`Hsva`].
//!
//! Controlled: the app owns the color and takes it from `on_change`.
//!
//! ```ignore
//! ColorPicker::new(self.color, Msg::Color)           // a saturation/value square and a hue slider
//!     .alpha(true)                                    // and an alpha slider
//!     .swatches(["#e01b24", "#ff7800", "#f5c211", "#33d17a", "#3584e4", "#9141ac"])
//!     .format(Format::Hex)                            // the readout under it
//!
//! ColorInput::new(self.color, &self.text, Msg::Text, Msg::Color)   // an entry and a popover picker
//! ```
//!
//! A drag keeps working outside the control. Keyboard nudging of the thumbs is not included:
//! iced has no focus model to hang it on.

use super::{Element, Popover, popover, wrap_box};
use crate::Theme;
use crate::primitives::color_picker::{Format, Hsva};
use crate::widget::typography as ty;
use crate::{Metrics, style::button as sb};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::gradient::Linear;
use iced::widget::{button, column, container, row, space};
use iced::{
    Background, Border, Color, Event, Length, Point, Radians, Rectangle, Shadow, Size, mouse,
};

/// The thickness of a slider track.
const TRACK: f32 = 12.0;
/// The thumb's diameter.
const THUMB: f32 = 16.0;

fn to_color(c: Hsva) -> Color {
    let (r, g, b, a) = c.to_rgba();
    Color { r, g, b, a }
}

/// A gradient across a track: left to right, or top to bottom when `down`. iced measures the
/// angle from "up" clockwise: 0 runs bottom to top, a quarter turn left to right, half a turn
/// top to bottom.
fn gradient(down: bool, stops: &[(f32, Color)]) -> Background {
    let mut g = Linear::new(Radians(if down {
        std::f32::consts::PI
    } else {
        std::f32::consts::FRAC_PI_2
    }));
    for (offset, color) in stops {
        g = g.add_stop(*offset, *color);
    }
    Background::Gradient(g.into())
}

/// Which control a pointer is on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Part {
    Square,
    Hue,
    Alpha,
}

/// The rectangles of the three controls inside `bounds`, for a width and whether alpha shows.
fn parts(bounds: Rectangle, with_alpha: bool, gap: f32) -> [Option<Rectangle>; 3] {
    let square_h = (bounds.height
        - TRACK * (1.0 + with_alpha as u8 as f32)
        - gap * (1.0 + with_alpha as u8 as f32))
        .max(40.0);
    let square = Rectangle::new(bounds.position(), Size::new(bounds.width, square_h));
    let hue = Rectangle::new(
        Point::new(bounds.x, square.y + square_h + gap),
        Size::new(bounds.width, TRACK),
    );
    let alpha = with_alpha.then(|| {
        Rectangle::new(
            Point::new(bounds.x, hue.y + TRACK + gap),
            Size::new(bounds.width, TRACK),
        )
    });
    [Some(square), Some(hue), alpha]
}

/// The controls: a saturation / value square, a hue slider and, optionally, an alpha slider.
struct Surface<'a, M> {
    value: Hsva,
    with_alpha: bool,
    width: Length,
    on_change: Option<Box<dyn Fn(Hsva) -> M + 'a>>,
    metrics: Metrics,
}

#[derive(Default)]
struct State {
    drag: Option<Part>,
}

impl<M> Surface<'_, M> {
    fn gap(&self) -> f32 {
        self.metrics.space_m
    }
    /// The color the pointer at `p` makes while it holds `part`.
    fn pick(&self, part: Part, p: Point, bounds: Rectangle) -> Hsva {
        let rects = parts(bounds, self.with_alpha, self.gap());
        let frac = |x: f32, from: f32, len: f32| ((x - from) / len.max(1.0)).clamp(0.0, 1.0);
        match part {
            Part::Square => {
                let r = rects[0].expect("a square");
                self.value
                    .at(frac(p.x, r.x, r.width), frac(p.y, r.y, r.height))
            }
            Part::Hue => {
                let r = rects[1].expect("a hue slider");
                self.value.with_hue(frac(p.x, r.x, r.width) * 360.0)
            }
            Part::Alpha => {
                let r = rects[2].expect("an alpha slider");
                self.value.with_alpha(frac(p.x, r.x, r.width))
            }
        }
    }
    fn part_at(&self, p: Point, bounds: Rectangle) -> Option<Part> {
        let rects = parts(bounds, self.with_alpha, self.gap());
        // The sliders' grab areas are a little taller than their tracks.
        let grab = |r: Rectangle| r.expand(4.0);
        if rects[0]?.contains(p) {
            Some(Part::Square)
        } else if grab(rects[1]?).contains(p) {
            Some(Part::Hue)
        } else if rects[2].is_some_and(|r| grab(r).contains(p)) {
            Some(Part::Alpha)
        } else {
            None
        }
    }
}

impl<'a, M: 'a> Widget<M, Theme, iced::Renderer> for Surface<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn layout(
        &mut self,
        _: &mut Tree,
        _: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let width = match self.width {
            Length::Fixed(w) => w,
            _ => limits.max().width.min(320.0),
        };
        // A square a bit wider than it is tall, then the sliders.
        let tracks = 1.0 + self.with_alpha as u8 as f32;
        let square = (width * 0.62).clamp(110.0, 220.0);
        let height = square + tracks * (TRACK + self.gap());
        layout::Node::new(limits.resolve(
            Length::Fixed(width),
            Length::Fixed(height),
            Size::new(width, height),
        ))
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
        let Some(on_change) = &self.on_change else {
            return;
        };
        let bounds = layout.bounds();
        let state = tree.state.downcast_mut::<State>();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                if let Some(p) = cursor.position()
                    && let Some(part) = self.part_at(p, bounds)
                {
                    state.drag = Some(part);
                    shell.publish(on_change(self.pick(part, p, bounds)));
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let (Some(part), Some(p)) = (state.drag, cursor.position()) {
                    shell.publish(on_change(self.pick(part, p, bounds)));
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.drag.is_some() =>
            {
                state.drag = None;
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
        if self.on_change.is_none() {
            return mouse::Interaction::None;
        }
        if tree.state.downcast_ref::<State>().drag.is_some() {
            return mouse::Interaction::Grabbing;
        }
        match cursor
            .position()
            .and_then(|p| self.part_at(p, layout.bounds()))
        {
            Some(Part::Square) => mouse::Interaction::Crosshair,
            Some(_) => mouse::Interaction::Pointer,
            None => mouse::Interaction::None,
        }
    }

    fn draw(
        &self,
        _: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        _: &renderer::Style,
        layout: Layout<'_>,
        _: mouse::Cursor,
        _: &Rectangle,
    ) {
        use iced::advanced::Renderer as _;
        let [square, hue, alpha_r] = parts(layout.bounds(), self.with_alpha, self.gap());
        let radius = theme.metrics.radius_s;
        let ring = Border {
            color: theme.semantic.border,
            width: 1.0,
            radius: radius.into(),
        };
        let quad = |bounds: Rectangle, border: Border| Quad {
            bounds,
            border,
            shadow: Shadow::default(),
            snap: false,
        };
        let full = Hsva::new(self.value.h, 1.0, 1.0, 1.0);

        // The square: white to the hue across, then transparent to black down.
        if let Some(r) = square {
            renderer.fill_quad(
                quad(r, Border::default().rounded(radius)),
                gradient(false, &[(0.0, Color::WHITE), (1.0, to_color(full))]),
            );
            renderer.fill_quad(
                quad(r, ring),
                gradient(true, &[(0.0, Color::TRANSPARENT), (1.0, Color::BLACK)]),
            );
            let (x, y) = self.value.position();
            self.thumb(
                renderer,
                Point::new(r.x + x * r.width, r.y + y * r.height),
                to_color(Hsva {
                    a: 1.0,
                    ..self.value
                }),
            );
        }
        // The hue slider: the six hues round the wheel.
        if let Some(r) = hue {
            let stops: Vec<(f32, Color)> = (0..=6)
                .map(|i| {
                    (
                        i as f32 / 6.0,
                        to_color(Hsva::new(i as f32 * 60.0, 1.0, 1.0, 1.0)),
                    )
                })
                .collect();
            renderer.fill_quad(quad(r, ring), gradient(false, &stops));
            self.thumb(
                renderer,
                Point::new(r.x + self.value.h / 360.0 * r.width, r.y + r.height / 2.0),
                to_color(full),
            );
        }
        // The alpha slider: a checkerboard under the color fading in.
        if let Some(r) = alpha_r {
            let cell = TRACK / 2.0;
            let (light, dark) = (Color::WHITE, Color::from_rgb(0.8, 0.8, 0.8));
            let (cols, rows) = ((r.width / cell).ceil() as usize, 2);
            for row in 0..rows {
                for col in 0..cols {
                    let x = r.x + col as f32 * cell;
                    renderer.fill_quad(
                        quad(
                            Rectangle::new(
                                Point::new(x, r.y + row as f32 * cell),
                                Size::new(cell.min(r.x + r.width - x), cell),
                            ),
                            Border::default(),
                        ),
                        if (row + col) % 2 == 0 { light } else { dark },
                    );
                }
            }
            let c = to_color(Hsva {
                a: 1.0,
                ..self.value
            });
            renderer.fill_quad(
                quad(r, ring),
                gradient(false, &[(0.0, Color { a: 0.0, ..c }), (1.0, c)]),
            );
            self.thumb(
                renderer,
                Point::new(r.x + self.value.a * r.width, r.y + r.height / 2.0),
                to_color(self.value),
            );
        }
    }
}

impl<M> Surface<'_, M> {
    /// A round thumb with a white ring, filled with the color under it.
    fn thumb(&self, renderer: &mut iced::Renderer, at: Point, fill: Color) {
        use iced::advanced::Renderer as _;
        let bounds = Rectangle::new(
            Point::new(at.x - THUMB / 2.0, at.y - THUMB / 2.0),
            Size::new(THUMB, THUMB),
        );
        renderer.fill_quad(
            Quad {
                bounds,
                border: Border {
                    color: Color::WHITE,
                    width: 2.0,
                    radius: (THUMB / 2.0).into(),
                },
                shadow: Shadow {
                    color: Color {
                        a: 0.35,
                        ..Color::BLACK
                    },
                    offset: iced::Vector::new(0.0, 1.0),
                    blur_radius: 3.0,
                },
                snap: false,
            },
            fill,
        );
    }
}

impl<'a, M: 'a> From<Surface<'a, M>> for Element<'a, M> {
    fn from(s: Surface<'a, M>) -> Self {
        iced::Element::new(s)
    }
}

/// A color picker. See the [module](self).
pub struct ColorPicker<'a, M> {
    value: Hsva,
    on_change: Option<std::rc::Rc<dyn Fn(Hsva) -> M + 'a>>,
    alpha: bool,
    swatches: Vec<Hsva>,
    format: Format,
    readout: bool,
    width: Length,
    metrics: Metrics,
}

impl<'a, M: Clone + 'a> ColorPicker<'a, M> {
    pub fn new(value: Hsva, on_change: impl Fn(Hsva) -> M + 'a) -> Self {
        Self {
            value,
            on_change: Some(std::rc::Rc::new(on_change)),
            alpha: false,
            swatches: Vec::new(),
            format: Format::Hex,
            readout: true,
            width: Length::Fixed(240.0),
            metrics: Metrics::desktop(),
        }
    }
    /// A slider for the color's alpha (with the readout in a form that carries it).
    pub fn alpha(mut self, on: bool) -> Self {
        self.alpha = on;
        self
    }
    /// Colors to pick from in one press: hex or any form [`Hsva::parse`] reads. Ones that do
    /// not parse are left out.
    pub fn swatches<S: AsRef<str>>(mut self, swatches: impl IntoIterator<Item = S>) -> Self {
        self.swatches = swatches
            .into_iter()
            .filter_map(|s| Hsva::parse(s.as_ref()))
            .collect();
        self
    }
    /// The form of the readout under the controls (default hex).
    pub fn format(mut self, f: Format) -> Self {
        self.format = f;
        self
    }
    /// Whether the readout (a swatch and the value as text) shows (default true).
    pub fn readout(mut self, on: bool) -> Self {
        self.readout = on;
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

/// A square of one color, with a hairline.
fn chip<'a, M: 'a>(c: Hsva, size: f32) -> Element<'a, M> {
    container(space())
        .width(size)
        .height(size)
        .style(move |t: &Theme| iced::widget::container::Style {
            background: Some(Background::Color(to_color(c))),
            border: Border {
                color: t.semantic.border,
                width: 1.0,
                radius: t.metrics.radius_s.into(),
            },
            ..Default::default()
        })
        .into()
}

impl<'a, M: Clone + 'a> From<ColorPicker<'a, M>> for Element<'a, M> {
    fn from(p: ColorPicker<'a, M>) -> Self {
        let m = p.metrics;
        let change = p.on_change.clone();
        let surface = Surface {
            value: p.value,
            with_alpha: p.alpha,
            width: p.width,
            on_change: change
                .clone()
                .map(|f| Box::new(move |c| f(c)) as Box<dyn Fn(Hsva) -> M + 'a>),
            metrics: m,
        };
        let mut col = column![Element::from(surface)].spacing(m.space_m);
        if !p.swatches.is_empty() {
            let chips: Vec<Element<'a, M>> = p
                .swatches
                .iter()
                .map(|&c| {
                    let on_press = change.as_ref().map(|f| f(c));
                    button(chip(c, 24.0))
                        .padding(0)
                        .style(sb::flat)
                        .on_press_maybe(on_press)
                        .into()
                })
                .collect();
            col = col.push(wrap_box(chips, m.space_xs));
        }
        if p.readout {
            let format = if p.alpha && !p.format.has_alpha() {
                match p.format {
                    Format::Hex => Format::Hexa,
                    Format::Rgb => Format::Rgba,
                    Format::Hsl => Format::Hsla,
                    Format::Hsv => Format::Hsva,
                    f => f,
                }
            } else {
                p.format
            };
            col = col.push(
                row![chip(p.value, 20.0), ty::monospace(p.value.format(format))]
                    .spacing(m.space_s)
                    .align_y(iced::Alignment::Center),
            );
        }
        container(col).width(p.width).into()
    }
}

/// An entry for a color, with the picker in a popover behind a swatch. See the
/// [module](self).
pub struct ColorInput<'a, M> {
    value: Hsva,
    text: &'a str,
    on_input: Box<dyn Fn(String) -> M + 'a>,
    on_change: std::rc::Rc<dyn Fn(Hsva) -> M + 'a>,
    placeholder: String,
    alpha: bool,
    swatches: Vec<Hsva>,
    format: Format,
    metrics: Metrics,
}

impl<'a, M: Clone + 'a> ColorInput<'a, M> {
    /// `text` is what the entry shows (the app keeps it, so typing is not interrupted); `value`
    /// is the color. `on_input` gets the typed text, `on_change` a color picked in the popover:
    /// parse the text with [`Hsva::parse`] and format a picked color with [`Hsva::format`].
    pub fn new(
        value: Hsva,
        text: &'a str,
        on_input: impl Fn(String) -> M + 'a,
        on_change: impl Fn(Hsva) -> M + 'a,
    ) -> Self {
        Self {
            value,
            text,
            on_input: Box::new(on_input),
            on_change: std::rc::Rc::new(on_change),
            placeholder: "Pick a color".into(),
            alpha: false,
            swatches: Vec::new(),
            format: Format::Hex,
            metrics: Metrics::desktop(),
        }
    }
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }
    pub fn alpha(mut self, on: bool) -> Self {
        self.alpha = on;
        self
    }
    pub fn swatches<S: AsRef<str>>(mut self, swatches: impl IntoIterator<Item = S>) -> Self {
        self.swatches = swatches
            .into_iter()
            .filter_map(|s| Hsva::parse(s.as_ref()))
            .collect();
        self
    }
    pub fn format(mut self, f: Format) -> Self {
        self.format = f;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: Clone + 'a> From<ColorInput<'a, M>> for Element<'a, M> {
    fn from(i: ColorInput<'a, M>) -> Self {
        let m = i.metrics;
        let change = i.on_change.clone();
        let picker = ColorPicker::new(i.value, move |c| change(c))
            .alpha(i.alpha)
            .format(i.format)
            .readout(false)
            .width(240.0)
            .metrics(m);
        let picker = if i.swatches.is_empty() {
            picker
        } else {
            ColorPicker {
                swatches: i.swatches,
                ..picker
            }
        };
        let swatch = Popover::new(chip(i.value, 20.0), popover([Element::from(picker)], 480.0))
            .appearance(super::PopoverAppearance::Flat)
            .padding(m.space_xs)
            .keep_open(true);
        let invalid = !i.text.trim().is_empty() && Hsva::parse(i.text).is_none();
        let mut entry = super::entry(i.placeholder, i.text)
            .on_input(i.on_input)
            .metrics(m);
        if invalid {
            entry = entry.state(super::EntryState::Error);
        }
        row![Element::from(swatch), Element::from(entry)]
            .spacing(m.space_s)
            .align_y(iced::Alignment::Center)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn surface(with_alpha: bool) -> Surface<'static, Hsva> {
        Surface {
            value: Hsva::new(200.0, 0.5, 0.5, 1.0),
            with_alpha,
            width: Length::Fixed(240.0),
            on_change: Some(Box::new(|c| c)),
            metrics: Metrics::default(),
        }
    }

    fn bounds(alpha: bool) -> Rectangle {
        let tracks = 1.0 + alpha as u8 as f32;
        let m = Metrics::default();
        Rectangle::new(
            Point::new(10.0, 20.0),
            Size::new(240.0, 150.0 + tracks * (TRACK + m.space_m)),
        )
    }

    #[test]
    fn the_square_sets_saturation_and_value_and_keeps_hue_and_alpha() {
        let s = surface(true);
        let b = bounds(true);
        let [sq, ..] = parts(b, true, s.gap());
        let r = sq.unwrap();
        let top_right = s.pick(Part::Square, Point::new(r.x + r.width, r.y), b);
        assert!((top_right.s - 1.0).abs() < 1e-3 && (top_right.v - 1.0).abs() < 1e-3);
        assert_eq!(top_right.h, 200.0);
        let bottom_left = s.pick(Part::Square, Point::new(r.x, r.y + r.height), b);
        assert!(bottom_left.s < 1e-3 && bottom_left.v < 1e-3);
    }

    #[test]
    fn a_slider_changes_only_its_own_channel_and_clamps_past_its_ends() {
        let s = surface(true);
        let b = bounds(true);
        let [_, hue, alpha_r] = parts(b, true, s.gap());
        let (hue, alpha_r) = (hue.unwrap(), alpha_r.unwrap());
        let half = s.pick(Part::Hue, Point::new(hue.x + hue.width / 2.0, hue.y), b);
        assert!((half.h - 180.0).abs() < 0.5 && half.s == 0.5 && half.v == 0.5);
        let past = s.pick(Part::Hue, Point::new(hue.x + hue.width + 500.0, hue.y), b);
        assert!(
            past.h >= 359.0 || past.h < 1.0,
            "the end wraps to red: {}",
            past.h
        );
        let a = s.pick(
            Part::Alpha,
            Point::new(alpha_r.x + alpha_r.width * 0.25, alpha_r.y),
            b,
        );
        assert!((a.a - 0.25).abs() < 1e-3 && a.h == 200.0);
        let gone = s.pick(Part::Alpha, Point::new(alpha_r.x - 99.0, alpha_r.y), b);
        assert_eq!(gone.a, 0.0);
    }

    #[test]
    fn the_pointer_finds_the_control_under_it() {
        let s = surface(true);
        let b = bounds(true);
        let [sq, hue, alpha_r] = parts(b, true, s.gap());
        assert_eq!(s.part_at(sq.unwrap().center(), b), Some(Part::Square));
        assert_eq!(s.part_at(hue.unwrap().center(), b), Some(Part::Hue));
        assert_eq!(s.part_at(alpha_r.unwrap().center(), b), Some(Part::Alpha));
        assert_eq!(s.part_at(Point::new(0.0, 0.0), b), None);
        // Without the alpha slider that area is empty.
        let n = surface(false);
        assert_eq!(
            n.part_at(Point::new(b.x + 5.0, b.y + b.height - 1.0), bounds(false)),
            None
        );
    }

    #[test]
    fn the_controls_stack_without_overlapping() {
        let b = bounds(true);
        let [sq, hue, alpha_r] = parts(b, true, Metrics::default().space_m);
        let (sq, hue, alpha_r) = (sq.unwrap(), hue.unwrap(), alpha_r.unwrap());
        assert!(sq.y + sq.height <= hue.y && hue.y + hue.height <= alpha_r.y);
        assert!(alpha_r.y + alpha_r.height <= b.y + b.height + 0.01);
    }

    #[test]
    fn the_swatches_that_do_not_parse_are_left_out() {
        let p = ColorPicker::<u8>::new(Hsva::BLACK, |_| 0).swatches([
            "#ff0000",
            "nope",
            "rgb(0, 0, 255)",
        ]);
        assert_eq!(p.swatches.len(), 2);
        let _: Element<'_, u8> = p.alpha(true).format(Format::Rgb).into();
    }
}
