//! `GtkSwitch`: a 46×26 pill track with a 20px knob. The pill follows the theme's
//! control radius (`radius_m`): fully round at the Adwaita default, square at 0.
//!
//! iced's own `toggler` is always 2:1 and sizes the knob from the height, which
//! looks too wide next to Adwaita. This widget uses Adwaita's proportions.

use crate::animation::{Spec, Tween};
use crate::Theme;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{tree, Tree, Widget};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::{mouse, Background, Border, Color, Element, Event, Length, Rectangle, Shadow, Size, Vector};

pub const WIDTH: f32 = 46.0;
pub const HEIGHT: f32 = 26.0;
const KNOB: f32 = 20.0;

/// An on/off switch. Stateless: the app owns `active`.
pub struct Switch<'a, M> {
    active: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> M + 'a>>,
}

struct State {
    pressed: bool,
    /// 0 = off, 1 = on; animated with the stylesheet's 200ms `$ease-out-quad`.
    progress: Tween,
    shown: f32,
}

/// Builds a [`Switch`]. Without `.on_toggle` it renders disabled.
pub fn switch<'a, M>(active: bool) -> Switch<'a, M> {
    Switch { active, on_toggle: None }
}

impl<'a, M> Switch<'a, M> {
    pub fn on_toggle(mut self, f: impl Fn(bool) -> M + 'a) -> Self {
        self.on_toggle = Some(Box::new(f));
        self
    }
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for Switch<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        let initial = if self.active { 1.0 } else { 0.0 };
        tree::State::new(State { pressed: false, progress: Tween::new(initial), shown: initial })
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(WIDTH), Length::Fixed(HEIGHT))
    }

    fn layout(&mut self, _: &mut Tree, _: &iced::Renderer, _: &layout::Limits) -> layout::Node {
        layout::Node::new(Size::new(WIDTH, HEIGHT))
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
        let state = tree.state.downcast_mut::<State>();
        // Animate the knob and track toward the current value.
        if let Event::Window(iced::window::Event::RedrawRequested(now)) = event {
            let target = if self.active { 1.0 } else { 0.0 };
            state.progress.set_target(target, *now, Spec::STYLE);
            let v = state.progress.tick(*now);
            if (v - state.shown).abs() > f32::EPSILON || state.progress.is_animating() {
                state.shown = v;
                shell.request_redraw();
            }
        }
        let Some(on_toggle) = &self.on_toggle else { return };
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(layout.bounds()) =>
            {
                state.pressed = true;
                shell.capture_event();
                shell.request_redraw();
            }
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) if state.pressed => {
                state.pressed = false;
                if cursor.is_over(layout.bounds()) {
                    shell.publish(on_toggle(!self.active));
                }
                shell.capture_event();
                shell.request_redraw();
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Rectangle,
        _: &iced::Renderer,
    ) -> mouse::Interaction {
        if self.on_toggle.is_some() && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
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
        let b = layout.bounds();
        let s = &theme.semantic;
        let enabled = self.on_toggle.is_some();
        let hovered = enabled && cursor.is_over(b);
        let k = if enabled { 1.0 } else { s.opacity.disabled };

        // The control radius scales the pill: the default (the Adwaita maximum) is fully
        // round, 0 is square; the knob follows so it stays concentric with the track.
        let round = (theme.metrics.radius_m / crate::Metrics::MAX_RADIUS_M).clamp(0.0, 1.0);

        let p = tree.state.downcast_ref::<State>().shown.clamp(0.0, 1.0);
        let off = crate::color::alpha(s.window.fg, s.opacity.trough + if hovered { 0.05 } else { 0.0 });
        let on = if hovered { s.hover_fill(s.accent.bg) } else { s.accent.bg };
        // The track cross-fades between the off and on colors as the knob slides.
        let track = crate::color::mix(off, on, p);
        renderer.fill_quad(
            Quad {
                bounds: b,
                border: Border::default().rounded(HEIGHT / 2.0 * round),
                shadow: Shadow::default(),
                snap: false,
            },
            Background::Color(crate::color::fade(track, k)),
        );

        let margin = (HEIGHT - KNOB) / 2.0;
        let x = b.x + margin + p * (b.width - 2.0 * margin - KNOB);
        let knob = Rectangle { x, y: b.y + margin, width: KNOB, height: KNOB };
        renderer.fill_quad(
            Quad {
                bounds: knob,
                border: Border::default().rounded(KNOB / 2.0 * round),
                shadow: Shadow {
                    color: Color { a: 0.25 * k, ..Color::BLACK },
                    offset: Vector::new(0.0, 1.0),
                    blur_radius: 2.0,
                },
                snap: false,
            },
            Background::Color(crate::color::fade(Color::WHITE, k)),
        );
    }
}

impl<'a, M: 'a> From<Switch<'a, M>> for Element<'a, M, Theme> {
    fn from(s: Switch<'a, M>) -> Self {
        Element::new(s)
    }
}
