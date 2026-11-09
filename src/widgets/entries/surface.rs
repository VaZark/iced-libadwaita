//! EntryRow owns one focus outline around its complete control surface.
use crate::binds::wrap::{Behavior, Content, Wrap};
use crate::{Element, Metrics, Theme};
use iced::advanced::{Shell, layout::Layout, renderer};
use iced::{Event, mouse};

pub(super) struct Ring {
    ring: bool,
    metrics: Metrics,
}

#[derive(Default)]
pub(super) struct State {
    focused: bool,
}

/// Wraps `content` and outlines it while any control inside has focus (when `ring`).
pub(super) fn outlined<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    ring: bool,
    metrics: Metrics,
) -> Wrap<'a, M, Ring> {
    Wrap::new(content, Ring { ring, metrics })
}

/// Only these can move focus without an operation; a redraw or pointer move cannot.
fn may_move_focus(event: &Event) -> bool {
    matches!(
        event,
        Event::Mouse(mouse::Event::ButtonPressed(_) | mouse::Event::ButtonReleased(_))
            | Event::Keyboard(_)
            | Event::Touch(_)
            | Event::Window(iced::window::Event::Focused | iced::window::Event::Unfocused)
    )
}

impl<M> Behavior<M> for Ring {
    type State = State;
    fn state(&self) -> State {
        State::default()
    }
    fn after_update(
        &mut self,
        state: &mut State,
        event: &Event,
        shell: &mut Shell<'_, M>,
        content: &mut Content<'_, '_, M>,
    ) {
        if self.ring && may_move_focus(event) {
            let now = content.contains_focus();
            if state.focused != now {
                state.focused = now;
                shell.request_redraw();
            }
        }
    }
    fn after_operate(&mut self, state: &mut State, content: &mut Content<'_, '_, M>) {
        if self.ring {
            state.focused = content.contains_focus();
        }
    }
    fn after_draw(
        &self,
        state: &State,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        layout: Layout<'_>,
    ) {
        if self.ring && state.focused {
            use iced::advanced::Renderer as _;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border: iced::Border {
                        color: theme.semantic.accent.bg,
                        width: self.metrics.focus_ring_width,
                        radius: self.metrics.radius_m.into(),
                    },
                    ..Default::default()
                },
                iced::Color::TRANSPARENT,
            );
        }
    }
}
