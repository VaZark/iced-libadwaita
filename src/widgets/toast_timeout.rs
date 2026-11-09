//! Widget-owned toast expiry: no application subscription is needed.
use crate::Element;
use crate::binds::wrap::{Behavior, Content, Wrap};
use iced::Event;
use iced::advanced::Shell;
use crate::primitives::countdown::{Countdown, Tick};
use std::time::Instant;

pub(super) struct Timeout<M> {
    pub key: (String, u32),
    pub on_dismiss: Option<M>,
}
pub(super) struct State {
    key: (String, u32),
    countdown: Countdown,
}
impl State {
    fn new(key: (String, u32)) -> Self {
        Self {
            key,
            countdown: Countdown::default(),
        }
    }
}

/// Wraps `content` so it disappears, and publishes `on_dismiss`, once `key.1` ms have passed.
/// A different `key` restarts the countdown.
pub(super) fn timeout<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M>>,
    key: (String, u32),
    on_dismiss: Option<M>,
) -> Element<'a, M> {
    Wrap::new(content, Timeout { key, on_dismiss }).into()
}

impl<M: Clone> Behavior<M> for Timeout<M> {
    type State = State;
    fn state(&self) -> State {
        State::new(self.key.clone())
    }
    fn diff(&self, state: &mut State) {
        if state.key != self.key {
            *state = State::new(self.key.clone());
        }
    }
    fn hidden(&self, state: &State) -> bool {
        state.countdown.fired()
    }
    fn before_update(
        &mut self,
        state: &mut State,
        event: &Event,
        shell: &mut Shell<'_, M>,
        _: &mut Content<'_, '_, M>,
    ) -> bool {
        let now = match event {
            Event::Window(iced::window::Event::RedrawRequested(now)) => *now,
            _ => Instant::now(),
        };
        match state.countdown.tick(now, self.key.1) {
            Tick::Idle => false,
            Tick::Wait(deadline) => {
                shell.request_redraw_at(deadline);
                false
            }
            Tick::Fire => {
                if let Some(message) = &self.on_dismiss {
                    shell.publish(message.clone());
                }
                shell.invalidate_layout();
                shell.request_redraw();
                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::{
        clipboard,
        layout::{self, Layout},
        renderer::Headless,
        widget::{Tree, Widget},
    };
    use iced::{Rectangle, Size, mouse};
    use std::time::Duration;

    #[test]
    fn expiry_is_exact_once_and_can_be_disabled_or_restarted() {
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let limits = layout::Limits::new(Size::ZERO, Size::new(400.0, 200.0));
        let viewport = Rectangle::new(iced::Point::ORIGIN, Size::new(400.0, 200.0));
        for timeout in [0u32, 75, 3000] {
            for dismiss in [None, Some(1u8)] {
                let mut widget = Wrap::new(
                    iced::widget::space().width(100).height(30),
                    Timeout {
                        key: ("Toast".into(), timeout),
                        on_dismiss: dismiss,
                    },
                );
                let mut tree = Tree::new(&widget as &dyn Widget<_, _, _>);
                let node = widget.layout(&mut tree, &renderer, &limits);
                let start = Instant::now();
                let mut messages = vec![];
                for elapsed in [
                    0,
                    timeout.saturating_sub(1),
                    timeout,
                    timeout.saturating_add(100),
                ] {
                    widget.update(
                        &mut tree,
                        &Event::Window(iced::window::Event::RedrawRequested(
                            start + Duration::from_millis(elapsed as u64),
                        )),
                        Layout::new(&node),
                        mouse::Cursor::Unavailable,
                        &renderer,
                        &mut clipboard::Null,
                        &mut Shell::new(&mut messages),
                        &viewport,
                    );
                    assert_eq!(
                        tree.state.downcast_ref::<State>().countdown.fired(),
                        timeout > 0 && elapsed >= timeout
                    );
                }
                assert_eq!(
                    messages,
                    if timeout > 0 {
                        dismiss.into_iter().collect()
                    } else {
                        vec![]
                    }
                );
                if timeout > 0 {
                    assert_eq!(
                        widget.layout(&mut tree, &renderer, &limits).size(),
                        Size::ZERO
                    );
                    widget.behavior.key.0 = "Replacement".into();
                    widget.diff(&mut tree);
                    assert!(!tree.state.downcast_ref::<State>().countdown.fired());
                    assert!(tree.state.downcast_ref::<State>().countdown == Countdown::default());
                    assert_eq!(
                        widget.layout(&mut tree, &renderer, &limits).size(),
                        Size::new(100.0, 30.0)
                    );
                }
            }
        }
    }
}
