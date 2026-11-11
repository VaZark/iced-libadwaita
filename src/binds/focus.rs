//! Keyboard focus for a part of the window, after Mantine's
//! [`use-focus-trap`](https://mantine.dev/hooks/use-focus-trap/),
//! [`use-focus-return`](https://mantine.dev/hooks/use-focus-return/) and
//! [`use-focus-within`](https://mantine.dev/hooks/use-focus-within/). The rules are the
//! [`focus`](crate::primitives::focus) primitive; these wrappers read and set which of the
//! focusable widgets inside them is focused (iced's `Focusable`, in tree order), so they work for
//! every widget that takes focus (entries, text editors) without ids.
//!
//! * [`FocusScope`]: **trap** (Tab and Shift+Tab wrap inside instead of leaving), **initial
//!   focus** (when it appears the autofocus widget, else the first, takes focus) and
//!   **focus-within** (a message when focus enters or leaves it).
//! * [`focus_return`]: around a base and its modal layer; when the layer closes, focus goes back
//!   to what had it when the layer opened.
//!
//! [`Overlay`](crate::layouts::Overlay) (so every dialog and docked sheet) uses both. Use a `FocusScope`
//! yourself for a field that appears on an action (a search bar):
//!
//! ```ignore
//! FocusScope::new(search_entry).trap(false)     // the entry takes focus as it appears
//! FocusScope::new(form).initial_focus("email") // or the child with this id
//! ```

use super::wrap::{Behavior, Content, Wrap};
use crate::Element;
use crate::primitives::focus::{Dir, Return, Within, WithinEvent, index_of, initial, trap_step};
use iced::advanced::Shell;
use iced::advanced::widget::Id;
use iced::advanced::widget::operation::{Focusable, Operation};
use iced::keyboard::{self, Key, key::Named};
use iced::{Event, Rectangle};
use std::rc::Rc;

/// The focusable widgets under a widget, in tree order, and which one is focused.
#[derive(Debug, Default)]
pub(crate) struct Probe {
    pub total: usize,
    pub focused: Option<usize>,
    /// The id of each focusable widget, or `None`. Left empty by [`contains_focus`].
    pub ids: Vec<Option<Id>>,
    skip_ids: bool,
}

impl Probe {
    pub(crate) fn new(collect_ids: bool) -> Self {
        Self {
            skip_ids: !collect_ids,
            ..Self::default()
        }
    }
}

impl Operation for Probe {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }
    fn focusable(&mut self, id: Option<&Id>, _: Rectangle, state: &mut dyn Focusable) {
        if state.is_focused() {
            self.focused = Some(self.total);
        }
        if !self.skip_ids {
            self.ids.push(id.cloned());
        }
        self.total += 1;
    }
}

/// Focuses the `target`-th focusable widget and unfocuses the others.
pub(crate) struct FocusAt {
    target: usize,
    at: usize,
}
impl FocusAt {
    pub(crate) fn new(target: usize) -> Self {
        Self { target, at: 0 }
    }
}

impl Operation for FocusAt {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }
    fn focusable(&mut self, _: Option<&Id>, _: Rectangle, state: &mut dyn Focusable) {
        if self.at == self.target {
            state.focus();
        } else {
            state.unfocus();
        }
        self.at += 1;
    }
}

type OnChange<'a, M> = Rc<dyn Fn(bool) -> M + 'a>;

enum Kind<'a, M> {
    Scope {
        trap: bool,
        auto_focus: bool,
        initial_focus: Option<Id>,
        on_change: Option<OnChange<'a, M>>,
    },
    Return {
        active: bool,
        enabled: bool,
    },
}

/// Tab and Shift+Tab move between the entries of `content` in reading order and wrap from the last
/// to the first; nothing takes focus when it appears. iced's entries do not move focus on Tab by
/// themselves and a window has no focus order of its own, so a form or a table of entries that is
/// not inside a dialog (those have a [`FocusScope`] already) is wrapped in this:
///
/// ```ignore
/// tab_scope(column![header, rows, add_row])
/// ```
pub fn tab_scope<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    FocusScope::new(content).trap(true).auto_focus(false).into()
}

/// A focus scope around `content`. See the [module](self).
pub struct FocusScope<'a, M> {
    content: Element<'a, M>,
    trap: bool,
    auto_focus: bool,
    initial_focus: Option<Id>,
    on_change: Option<OnChange<'a, M>>,
}

impl<'a, M: 'a> FocusScope<'a, M> {
    /// A scope that traps Tab and focuses its first widget when it appears.
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            content: content.into(),
            trap: true,
            auto_focus: true,
            initial_focus: None,
            on_change: None,
        }
    }
    /// Whether Tab and Shift+Tab wrap inside the scope instead of leaving it (default true).
    pub fn trap(mut self, on: bool) -> Self {
        self.trap = on;
        self
    }
    /// Whether the scope takes focus when it appears (default true).
    pub fn auto_focus(mut self, on: bool) -> Self {
        self.auto_focus = on;
        self
    }
    /// The child that takes the initial focus, by its widget id, instead of the first focusable
    /// one (Mantine's `data-autofocus`). Give the child the same id (`entry(..).id(id)`,
    /// `text_input(..).id(id)`); an id that is not found leaves the first.
    pub fn initial_focus(mut self, id: impl Into<Id>) -> Self {
        self.initial_focus = Some(id.into());
        self
    }
    /// A message when focus enters (`true`) or leaves (`false`) the scope.
    pub fn on_focus_change(mut self, f: impl Fn(bool) -> M + 'a) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}

impl<'a, M: 'a> From<FocusScope<'a, M>> for Element<'a, M> {
    fn from(s: FocusScope<'a, M>) -> Self {
        Wrap::new(
            s.content,
            Kind::Scope {
                trap: s.trap,
                auto_focus: s.auto_focus,
                initial_focus: s.initial_focus,
                on_change: s.on_change,
            },
        )
        .into()
    }
}

/// `content` (a base and its modal layer) that gives focus back when `active` turns off: to what
/// had it when `active` turned on. Keep it in the tree whether or not the layer is up (pass
/// `false` when it is not), so its memory survives.
pub fn focus_return<'a, M: 'a>(content: impl Into<Element<'a, M>>, active: bool) -> Element<'a, M> {
    Wrap::new(
        content,
        Kind::Return {
            active,
            enabled: true,
        },
    )
    .into()
}

#[derive(Default)]
struct State {
    mounted: bool,
    within: Within,
    ret: Return,
}

impl<'a, M: 'a> Behavior<M> for Kind<'a, M> {
    type State = State;
    fn state(&self) -> State {
        State::default()
    }
    fn before_update(
        &mut self,
        state: &mut State,
        event: &Event,
        shell: &mut Shell<'_, M>,
        content: &mut Content<'_, '_, M>,
    ) -> bool {
        match self {
            Kind::Scope {
                trap,
                auto_focus,
                initial_focus,
                ..
            } => {
                if !state.mounted {
                    state.mounted = true;
                    if *auto_focus {
                        let p = content.probe();
                        let wanted = initial_focus.as_ref().and_then(|id| index_of(&p.ids, id));
                        if let Some(i) = initial(p.total, wanted) {
                            content.focus(i);
                            shell.request_redraw();
                        }
                    }
                }
                if *trap
                    && let Event::Keyboard(keyboard::Event::KeyPressed {
                        key: Key::Named(Named::Tab),
                        modifiers,
                        ..
                    }) = event
                {
                    let p = content.probe();
                    if let Some(i) =
                        trap_step(p.focused, p.total, Dir::from_shift(modifiers.shift()))
                    {
                        content.focus(i);
                    }
                    shell.capture_event();
                    shell.request_redraw();
                    return true;
                }
            }
            Kind::Return { active, enabled } => {
                state.ret.enabled = *enabled;
                if *active != state.ret.is_open() {
                    let p = content.probe();
                    if let Some(i) = state.ret.update(*active, p.focused) {
                        content.focus(i);
                        shell.request_redraw();
                    }
                }
            }
        }
        false
    }
    fn after_update(
        &mut self,
        state: &mut State,
        _event: &Event,
        shell: &mut Shell<'_, M>,
        content: &mut Content<'_, '_, M>,
    ) {
        if let Kind::Scope {
            on_change: Some(f), ..
        } = self
            && let Some(ev) = state.within.update(content.contains_focus())
        {
            shell.publish(f(ev == WithinEvent::Focus));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_wrappers_build_around_anything() {
        let _: Element<'_, ()> = FocusScope::new(iced::widget::space())
            .trap(false)
            .auto_focus(false)
            .initial_focus(Id::new("field"))
            .on_focus_change(|_| ())
            .into();
        let _: Element<'_, ()> = focus_return(iced::widget::space(), true);
    }
}
