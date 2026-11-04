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

use crate::Element;
use crate::Theme;
use crate::primitives::focus::{Dir, Return, Within, WithinEvent, index_of, initial, trap_step};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::operation::{Focusable, Operation};
use iced::advanced::widget::{Id, Tree, Widget, tree};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::keyboard::{self, Key, key::Named};
use iced::{Event, Length, Rectangle, Size, Vector, mouse};
use std::rc::Rc;

/// The focusable widgets under a widget, in tree order, and which one is focused.
#[derive(Debug, Default)]
struct Probe {
    total: usize,
    focused: Option<usize>,
    /// The id of each focusable widget, or `None`.
    ids: Vec<Option<Id>>,
}

impl Operation for Probe {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
        operate(self);
    }
    fn focusable(&mut self, id: Option<&Id>, _: Rectangle, state: &mut dyn Focusable) {
        if state.is_focused() {
            self.focused = Some(self.total);
        }
        self.ids.push(id.cloned());
        self.total += 1;
    }
}

/// Focuses the `target`-th focusable widget and unfocuses the others.
struct FocusAt {
    target: usize,
    at: usize,
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
        iced::Element::new(Wrapper {
            content: s.content,
            kind: Kind::Scope {
                trap: s.trap,
                auto_focus: s.auto_focus,
                initial_focus: s.initial_focus,
                on_change: s.on_change,
            },
        })
    }
}

/// `content` (a base and its modal layer) that gives focus back when `active` turns off: to what
/// had it when `active` turned on. Keep it in the tree whether or not the layer is up (pass
/// `false` when it is not), so its memory survives.
pub fn focus_return<'a, M: 'a>(content: impl Into<Element<'a, M>>, active: bool) -> Element<'a, M> {
    iced::Element::new(Wrapper {
        content: content.into(),
        kind: Kind::Return {
            active,
            enabled: true,
        },
    })
}

struct Wrapper<'a, M> {
    content: Element<'a, M>,
    kind: Kind<'a, M>,
}

#[derive(Default)]
struct State {
    mounted: bool,
    within: Within,
    ret: Return,
}

impl<M> Wrapper<'_, M> {
    fn probe(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &iced::Renderer) -> Probe {
        let mut p = Probe::default();
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut p);
        p
    }
    fn focus(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        index: usize,
    ) {
        let mut op = FocusAt {
            target: index,
            at: 0,
        };
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, &mut op);
    }
}

impl<'a, M: 'a> Widget<M, Theme, iced::Renderer> for Wrapper<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
        match &self.kind {
            Kind::Scope {
                trap,
                auto_focus,
                initial_focus,
                ..
            } => {
                let (trap, auto_focus, initial_focus) = (*trap, *auto_focus, initial_focus.clone());
                if !tree.state.downcast_ref::<State>().mounted {
                    tree.state.downcast_mut::<State>().mounted = true;
                    if auto_focus {
                        let p = self.probe(tree, layout, renderer);
                        let wanted = initial_focus.as_ref().and_then(|id| index_of(&p.ids, id));
                        if let Some(i) = initial(p.total, wanted) {
                            self.focus(tree, layout, renderer, i);
                            shell.request_redraw();
                        }
                    }
                }
                if trap
                    && let Event::Keyboard(keyboard::Event::KeyPressed {
                        key: Key::Named(Named::Tab),
                        modifiers,
                        ..
                    }) = event
                {
                    let p = self.probe(tree, layout, renderer);
                    if let Some(i) =
                        trap_step(p.focused, p.total, Dir::from_shift(modifiers.shift()))
                    {
                        self.focus(tree, layout, renderer, i);
                    }
                    shell.capture_event();
                    shell.request_redraw();
                    return;
                }
            }
            Kind::Return { active, enabled } => {
                let (active, enabled) = (*active, *enabled);
                let state = tree.state.downcast_mut::<State>();
                state.ret.enabled = enabled;
                if active != state.ret.is_open() {
                    let p = self.probe(tree, layout, renderer);
                    let back = tree
                        .state
                        .downcast_mut::<State>()
                        .ret
                        .update(active, p.focused);
                    if let Some(i) = back {
                        self.focus(tree, layout, renderer, i);
                        shell.request_redraw();
                    }
                }
            }
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        if let Kind::Scope {
            on_change: Some(f), ..
        } = &self.kind
        {
            let f = f.clone();
            let inside = self.probe(tree, layout, renderer).focused.is_some();
            if let Some(ev) = tree.state.downcast_mut::<State>().within.update(inside) {
                shell.publish(f(ev == WithinEvent::Focus));
            }
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
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout,
            cursor,
            viewport,
            renderer,
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
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(&mut tree.children[0], layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout,
            renderer,
            viewport,
            translation,
        )
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
