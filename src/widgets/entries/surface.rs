//! EntryRow owns one focus outline around its complete control surface.
use crate::binds::wrap::{Behavior, Content, Wrap};
use crate::{Element, Metrics, Theme};
use iced::advanced::{Shell, layout::Layout, renderer};
use iced::{Event, mouse};
use std::sync::{Arc, Mutex};

/// The row owns its transparent frame. Embedded controls do not define its corners.
/// Keep this style shared with focus and validation rather than copying its tokens.
pub(super) fn row_style(theme: &Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        border: iced::Border::default().rounded(theme.metrics.radius_m),
        ..Default::default()
    }
}

#[derive(Clone, Default)]
pub(super) enum Geometry {
    #[default]
    Row,
    Drawn(Arc<Mutex<Option<iced::border::Radius>>>),
}

impl Geometry {
    /// Observe the effective button style with the widget's actual status, including
    /// popover open state. Both outlines run after the target draws. No status guessing
    /// or custom-style evaluation with a fabricated `Active` state is needed.
    pub(super) fn button(
        style: crate::components::props::ButtonStyle,
    ) -> (Self, crate::components::props::ButtonStyle) {
        let radius = Arc::new(Mutex::new(None));
        let drawn = radius.clone();
        (
            Self::Drawn(radius),
            Arc::new(move |theme, status| {
                let style = style(theme, status);
                if let Ok(mut radius) = drawn.lock() {
                    *radius = Some(style.border.radius);
                }
                style
            }),
        )
    }

    pub(super) fn resolve(&self, theme: &Theme) -> Option<iced::border::Radius> {
        match self {
            Self::Row => Some(row_style(theme).border.radius),
            Self::Drawn(radius) => radius.lock().ok().and_then(|radius| *radius),
        }
    }
}

pub(super) struct Ring {
    ring: bool,
    metrics: Metrics,
    geometry: Geometry,
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
    geometry: Geometry,
) -> Wrap<'a, M, Ring> {
    Wrap::new(
        content,
        Ring {
            ring,
            metrics,
            geometry,
        },
    )
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
            let Some(radius) = self.geometry.resolve(theme) else {
                return;
            };
            use iced::advanced::Renderer as _;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    border: iced::Border {
                        color: theme.semantic.accent.bg,
                        width: self.metrics.focus_ring_width,
                        radius,
                    },
                    ..Default::default()
                },
                iced::Color::TRANSPARENT,
            );
        }
    }
}
