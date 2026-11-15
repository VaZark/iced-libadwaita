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
//! ColorInput::new(self.color, &self.text, Msg::Text, Msg::Color)   // a color box that opens the picker and an entry for the code
//! ```
//!
//! A drag keeps working outside the control. Keyboard nudging of the thumbs is not included:
//! iced has no focus model to hang it on.

use crate::Element;
use crate::Theme;
use crate::components::typography as ty;
use crate::primitives::color_picker::{Format, Hsva};
use crate::widgets::{Popover, popover, wrap_box};
use crate::{Metrics, styles::button as sb};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell};
use iced::gradient::Linear;
use iced::widget::{button, column, container, row, space};
use iced::{
    Background, Border, Color, Event, Length, Point, Radians, Rectangle, Shadow, Size, mouse,
};

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
fn parts(bounds: Rectangle, with_alpha: bool, gap: f32, track: f32) -> [Option<Rectangle>; 3] {
    let square_h = (bounds.height
        - track * (1.0 + with_alpha as u8 as f32)
        - gap * (1.0 + with_alpha as u8 as f32))
        .max(0.0);
    let square = Rectangle::new(bounds.position(), Size::new(bounds.width, square_h));
    let hue = Rectangle::new(
        Point::new(bounds.x, square.y + square_h + gap),
        Size::new(bounds.width, track),
    );
    let alpha = with_alpha.then(|| {
        Rectangle::new(
            Point::new(bounds.x, hue.y + track + gap),
            Size::new(bounds.width, track),
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
        let rects = parts(
            bounds,
            self.with_alpha,
            self.gap(),
            self.metrics.progress_height,
        );
        let frac =
            |x: f32, from: f32, len: f32| ((x - from) / len.max(f32::EPSILON)).clamp(0.0, 1.0);
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
        let rects = parts(
            bounds,
            self.with_alpha,
            self.gap(),
            self.metrics.progress_height,
        );
        // The sliders' grab areas are a little taller than their tracks.
        let grab = |r: Rectangle| r.expand(self.metrics.space_s);
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
            _ => limits.max().width.min(self.metrics.sidebar_width),
        };
        // The saturation/value square, then the sliders.
        let tracks = 1.0 + self.with_alpha as u8 as f32;
        let square = width;
        let height = square + tracks * (self.metrics.progress_height + self.gap());
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
        let [square, hue, alpha_r] = parts(
            layout.bounds(),
            self.with_alpha,
            self.gap(),
            self.metrics.progress_height,
        );
        let radius = theme.metrics.radius_s;
        let ring = Border {
            color: theme.semantic.border,
            width: theme.metrics.border_width,
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
                theme,
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
                theme,
                Point::new(r.x + self.value.h / 360.0 * r.width, r.y + r.height / 2.0),
                to_color(full),
            );
        }
        // The alpha slider: a checkerboard under the color fading in.
        if let Some(r) = alpha_r {
            let cell = self.metrics.progress_height.max(f32::EPSILON) / 2.0;
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
                theme,
                Point::new(r.x + self.value.a * r.width, r.y + r.height / 2.0),
                to_color(self.value),
            );
        }
    }
}

impl<M> Surface<'_, M> {
    /// A round thumb with a white ring, filled with the color under it.
    fn thumb(&self, renderer: &mut iced::Renderer, theme: &Theme, at: Point, fill: Color) {
        use iced::advanced::Renderer as _;
        let thumb = self.metrics.icon_size;
        let bounds = Rectangle::new(
            Point::new(at.x - thumb / 2.0, at.y - thumb / 2.0),
            Size::new(thumb, thumb),
        );
        renderer.fill_quad(
            Quad {
                bounds,
                border: Border {
                    color: Color::WHITE,
                    width: theme.metrics.focus_ring_width,
                    radius: (thumb / 2.0).into(),
                },
                shadow: theme.shadows.popover().shadow,
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
    label_props: crate::components::TextProps,
    button_props: crate::components::ButtonProps,
    value: Hsva,
    on_change: Option<std::rc::Rc<dyn Fn(Hsva) -> M + 'a>>,
    alpha: bool,
    swatches: Vec<Hsva>,
    format: Format,
    readout: bool,
    width: Option<Length>,
    spacing: Option<f32>,
    swatch_spacing: Option<f32>,
    metrics: Metrics,
}

impl<'a, M: Clone + 'a> ColorPicker<'a, M> {
    pub fn label_props(mut self, props: crate::components::TextProps) -> Self {
        self.label_props = props;
        self
    }

    pub fn button_props(mut self, props: crate::components::ButtonProps) -> Self {
        self.button_props = props;
        self
    }

    pub fn new(value: Hsva, on_change: impl Fn(Hsva) -> M + 'a) -> Self {
        Self {
            label_props: Default::default(),
            button_props: Default::default(),

            value,
            on_change: Some(std::rc::Rc::new(on_change)),
            alpha: false,
            swatches: Vec::new(),
            format: Format::Hex,
            readout: true,
            width: None,
            spacing: None,
            swatch_spacing: None,
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
        self.width = Some(w.into());
        self
    }
    /// The gap between the square and sliders, the swatches and the readout (default `space_m`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
        self
    }
    /// The gap between swatches (default `space_xs`).
    pub fn swatch_spacing(mut self, px: f32) -> Self {
        self.swatch_spacing = Some(px);
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
                width: t.metrics.border_width,
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
        let width = p.width.unwrap_or_else(|| {
            Length::Fixed(
                crate::theme_config::current()
                    .number("color-picker", "width")
                    .unwrap_or(m.sidebar_width),
            )
        });
        let surface = Surface {
            value: p.value,
            with_alpha: p.alpha,
            width,
            on_change: change
                .clone()
                .map(|f| Box::new(move |c| f(c)) as Box<dyn Fn(Hsva) -> M + 'a>),
            metrics: m,
        };
        let mut col = column![Element::from(surface)].spacing(p.spacing.unwrap_or(m.space_m));
        if !p.swatches.is_empty() {
            let chips: Vec<Element<'a, M>> = p
                .swatches
                .iter()
                .map(|&c| {
                    let on_press = change.as_ref().map(|f| f(c));
                    p.button_props
                        .apply(
                            button(chip(c, m.button_min_height))
                                .padding(0)
                                .style(sb::flat)
                                .on_press_maybe(on_press),
                        )
                        .into()
                })
                .collect();
            col = col.push(wrap_box(chips, p.swatch_spacing.unwrap_or(m.space_xs)));
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
                row![
                    chip(p.value, m.icon_size),
                    p.label_props.apply(ty::monospace(p.value.format(format)))
                ]
                .spacing(m.space_s)
                .align_y(iced::Alignment::Center),
            );
        }
        container(col).width(width).into()
    }
}

/// A color input: a box showing the color (like `GtkColorDialogButton`) that opens a
/// [`ColorPicker`] with an entry for the color code, in a popover or a dialog. See the
/// [module](self).
pub struct ColorInput<'a, M> {
    dropdown_props: super::popover::PopoverProps,
    entry_props: crate::components::EntryProps,
    value: Hsva,
    text: &'a str,
    on_input: Box<dyn Fn(String) -> M + 'a>,
    on_change: std::rc::Rc<dyn Fn(Hsva) -> M + 'a>,
    alpha: bool,
    swatches: Vec<Hsva>,
    trigger: Option<Element<'a, M>>,
    dialog: bool,
    scrim: bool,
    close_on_outside: bool,
    window_radius: Option<f32>,
    disabled: bool,
    open_initially: bool,
    spacing: Option<f32>,
    metrics: Metrics,
}

impl<'a, M: Clone + 'a> ColorInput<'a, M> {
    pub fn dropdown_props(mut self, props: super::popover::PopoverProps) -> Self {
        self.dropdown_props = props;
        self
    }

    pub fn entry_props(mut self, props: crate::components::EntryProps) -> Self {
        self.entry_props = props;
        self
    }

    /// `text` is what the entry in the picker shows (the app keeps it, so typing is not
    /// interrupted); `value` is the color. `on_input` gets the typed text, `on_change` a color
    /// picked with the controls: parse the text with [`Hsva::parse`] and format a picked color
    /// with [`Hsva::format`].
    pub fn new(
        value: Hsva,
        text: &'a str,
        on_input: impl Fn(String) -> M + 'a,
        on_change: impl Fn(Hsva) -> M + 'a,
    ) -> Self {
        Self {
            dropdown_props: Default::default(),
            entry_props: Default::default(),

            value,
            text,
            on_input: Box::new(on_input),
            on_change: std::rc::Rc::new(on_change),
            alpha: false,
            swatches: Vec::new(),
            trigger: None,
            dialog: false,
            scrim: true,
            close_on_outside: true,
            window_radius: None,
            disabled: false,
            open_initially: false,
            spacing: None,
            metrics: Metrics::desktop(),
        }
    }
    /// The gap between the picker's parts (default `space_m`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
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
    /// What opens the picker, instead of the color box.
    pub fn trigger(mut self, t: impl Into<Element<'a, M>>) -> Self {
        self.trigger = Some(t.into());
        self
    }
    /// Opens the picker as a dialog centered in the window (default: a popover at the box).
    pub fn dialog(mut self, on: bool) -> Self {
        self.dialog = on;
        self
    }
    /// Fades the window behind the dialog (default on); only a [`dialog`](Self::dialog) has it.
    pub fn scrim(mut self, on: bool) -> Self {
        self.scrim = on;
        self
    }
    /// Whether a press outside the dialog closes it (default true).
    pub fn close_on_outside(mut self, on: bool) -> Self {
        self.close_on_outside = on;
        self
    }
    /// Rounds the dialog's fade to the window's corners (`window::State::radius`).
    pub fn window_radius(mut self, r: f32) -> Self {
        self.window_radius = Some(r);
        self
    }
    pub fn disabled(mut self, on: bool) -> Self {
        self.disabled = on;
        self
    }
    pub fn open_initially(mut self, open: bool) -> Self {
        self.open_initially = open;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

/// The id of the color code entry in the picker, its initial focus.
const ENTRY_ID: &str = "iced-adaptive-design.color-input.code";

impl<'a, M: Clone + 'a> From<ColorInput<'a, M>> for Element<'a, M> {
    fn from(i: ColorInput<'a, M>) -> Self {
        let m = i.metrics;
        let change = i.on_change.clone();
        let mut picker = ColorPicker::new(i.value, move |c| change(c))
            .alpha(i.alpha)
            .readout(false)
            .width(Length::Fill)
            .metrics(m);
        picker.swatches = i.swatches;
        picker.spacing = i.spacing;
        let invalid = !i.text.trim().is_empty() && Hsva::parse(i.text).is_none();
        // The color code is what the picker is opened for: its entry takes focus.
        let mut entry = crate::components::entry("", i.text)
            .id(ENTRY_ID)
            .on_input(i.on_input)
            .metrics(m);
        if invalid {
            entry = entry.state(crate::components::EntryState::Error);
        }
        let content = popover(
            [
                Element::from(picker),
                Element::from(i.entry_props.apply(entry)),
            ],
            m.clamp_maximum,
        );
        let trigger = i.trigger.unwrap_or_else(|| color_box(i.value, m));
        let mut opener = Popover::new(trigger, content)
            .appearance(crate::widgets::PopoverAppearance::Flat)
            .padding(0)
            .popover_width(
                crate::theme_config::current()
                    .number("color-input", "popover-width")
                    .unwrap_or(m.sidebar_width),
            )
            .keep_open(true)
            .disabled(i.disabled)
            .open_initially(i.open_initially)
            .initial_focus(ENTRY_ID)
            .scrim(i.scrim)
            .close_on_outside(i.close_on_outside)
            .window_radius(i.window_radius.unwrap_or(m.radius_window));
        if i.dialog {
            opener = opener.presentation(crate::widgets::PopoverPresentation::Dialog);
        }
        i.dropdown_props.apply(opener).into()
    }
}

/// The color in a box with a hairline: the closed state of a [`ColorInput`].
fn color_box<'a, M: 'a>(c: Hsva, m: Metrics) -> Element<'a, M> {
    container(space())
        .width(m.icon_size + m.entry_padding.left + m.entry_padding.right)
        .height(m.entry_min_height)
        .style(move |t: &Theme| iced::widget::container::Style {
            background: Some(Background::Color(to_color(c))),
            border: Border {
                color: t.semantic.border,
                width: t.metrics.border_width,
                radius: t.metrics.radius_s.into(),
            },
            ..Default::default()
        })
        .into()
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
            Size::new(240.0, 150.0 + tracks * (m.progress_height + m.space_m)),
        )
    }

    #[test]
    fn the_square_sets_saturation_and_value_and_keeps_hue_and_alpha() {
        let s = surface(true);
        let b = bounds(true);
        let [sq, ..] = parts(b, true, s.gap(), s.metrics.progress_height);
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
        let [_, hue, alpha_r] = parts(b, true, s.gap(), s.metrics.progress_height);
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
        let [sq, hue, alpha_r] = parts(b, true, s.gap(), s.metrics.progress_height);
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
        let [sq, hue, alpha_r] = parts(
            b,
            true,
            Metrics::default().space_m,
            Metrics::default().progress_height,
        );
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

#[cfg(test)]
mod spacing_params {
    use super::*;

    #[test]
    fn the_picker_gaps_are_parameters() {
        let p = ColorPicker::<()>::new(Hsva::new(10.0, 0.5, 0.5, 1.0), |_| ())
            .spacing(3.0)
            .swatch_spacing(1.0);
        assert_eq!((p.spacing, p.swatch_spacing), (Some(3.0), Some(1.0)));
        let _: Element<'_, ()> = p.into();
        let i =
            ColorInput::<()>::new(Hsva::new(10.0, 0.5, 0.5, 1.0), "", |_| (), |_| ()).spacing(4.0);
        assert_eq!(i.spacing, Some(4.0));
        let _: Element<'_, ()> = i.into();
    }
}

#[cfg(test)]
mod metric_regressions {
    use super::*;

    #[test]
    fn track_geometry_and_hit_testing_share_custom_metrics() {
        let m = Metrics {
            progress_height: 19.0,
            space_m: 7.0,
            space_s: 0.0,
            ..Metrics::default()
        };
        let surface = Surface::<()> {
            value: Hsva::BLACK,
            with_alpha: true,
            width: Length::Fill,
            on_change: None,
            metrics: m,
        };
        let bounds = Rectangle::new(Point::ORIGIN, Size::new(200.0, 252.0));
        let [square, hue, alpha] = parts(bounds, true, surface.gap(), m.progress_height);
        assert_eq!(square.unwrap().height, 200.0);
        assert_eq!(hue.unwrap().height, 19.0);
        assert_eq!(alpha.unwrap().height, 19.0);
        assert_eq!(
            surface.part_at(alpha.unwrap().center(), bounds),
            Some(Part::Alpha)
        );
    }

    #[test]
    fn geometry_defaults_remain_inherited_until_build_time() {
        let m = Metrics {
            radius_window: 7.0,
            sidebar_width: 177.0,
            ..Metrics::default()
        };
        let input = ColorInput::<()>::new(Hsva::BLACK, "", |_| (), |_| ()).metrics(m);
        assert_eq!(input.window_radius, None);
        assert_eq!(input.metrics.radius_window, 7.0);
        let explicit = input.window_radius(0.0).metrics(Metrics::default());
        assert_eq!(explicit.window_radius, Some(0.0));
        let picker = ColorPicker::<()>::new(Hsva::BLACK, |_| ()).metrics(m);
        assert_eq!(picker.width, None);
        let explicit = picker.width(123.0).metrics(Metrics::default());
        assert_eq!(explicit.width, Some(Length::Fixed(123.0)));
    }
}

/// Presentation-only overrides supported by [`ColorPicker`]. Values and events stay parent-owned.
#[derive(Clone, Default)]
pub struct ColorPickerProps {
    pub width: Option<iced::Length>,
    pub spacing: Option<f32>,
    pub swatch_spacing: Option<f32>,
    pub metrics: Option<crate::Metrics>,
    pub label_props: Option<crate::components::TextProps>,
    pub button_props: Option<crate::components::ButtonProps>,
}
impl ColorPickerProps {
    pub fn width(mut self, value: impl Into<iced::Length>) -> Self {
        self.width = Some(value.into());
        self
    }
    pub fn spacing(mut self, value: impl Into<f32>) -> Self {
        self.spacing = Some(value.into());
        self
    }
    pub fn swatch_spacing(mut self, value: impl Into<f32>) -> Self {
        self.swatch_spacing = Some(value.into());
        self
    }
    pub fn metrics(mut self, value: impl Into<crate::Metrics>) -> Self {
        self.metrics = Some(value.into());
        self
    }
    pub fn label_props(mut self, value: impl Into<crate::components::TextProps>) -> Self {
        self.label_props = Some(value.into());
        self
    }
    pub fn button_props(mut self, value: impl Into<crate::components::ButtonProps>) -> Self {
        self.button_props = Some(value.into());
        self
    }
    pub fn apply<'a, M: Clone + 'a>(&self, widget: ColorPicker<'a, M>) -> ColorPicker<'a, M> {
        let mut widget = widget;
        if let Some(value) = &self.width {
            widget = widget.width(value.clone());
        }
        if let Some(value) = &self.spacing {
            widget = widget.spacing(value.clone());
        }
        if let Some(value) = &self.swatch_spacing {
            widget = widget.swatch_spacing(value.clone());
        }
        if let Some(value) = &self.metrics {
            widget = widget.metrics(value.clone());
        }
        if let Some(props) = &self.label_props {
            widget.label_props.merge(props);
        }
        if let Some(props) = &self.button_props {
            widget.button_props.merge(props);
        }
        widget
    }
}

/// Presentation-only overrides supported by [`ColorInput`]. Values and events stay parent-owned.
#[derive(Clone, Default)]
pub struct ColorInputProps {
    pub dropdown_props: Option<super::popover::PopoverProps>,
    pub spacing: Option<f32>,
    pub metrics: Option<crate::Metrics>,
    pub entry_props: Option<crate::components::EntryProps>,
}
impl ColorInputProps {
    pub fn dropdown_props(mut self, props: super::popover::PopoverProps) -> Self {
        self.dropdown_props = Some(props);
        self
    }

    pub fn spacing(mut self, value: impl Into<f32>) -> Self {
        self.spacing = Some(value.into());
        self
    }
    pub fn metrics(mut self, value: impl Into<crate::Metrics>) -> Self {
        self.metrics = Some(value.into());
        self
    }
    pub fn entry_props(mut self, value: impl Into<crate::components::EntryProps>) -> Self {
        self.entry_props = Some(value.into());
        self
    }
    pub fn apply<'a, M: Clone + 'a>(&self, widget: ColorInput<'a, M>) -> ColorInput<'a, M> {
        let mut widget = widget;
        if let Some(props) = &self.dropdown_props {
            widget.dropdown_props.merge(props);
        }
        if let Some(value) = &self.spacing {
            widget = widget.spacing(value.clone());
        }
        if let Some(value) = &self.metrics {
            widget = widget.metrics(value.clone());
        }
        if let Some(props) = &self.entry_props {
            widget.entry_props.merge(props);
        }
        widget
    }
}
