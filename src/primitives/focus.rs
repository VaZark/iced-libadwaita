//! Keyboard focus rules, after Mantine's hooks
//! [`use-focus-trap`](https://mantine.dev/hooks/use-focus-trap/),
//! [`use-focus-return`](https://mantine.dev/hooks/use-focus-return/) and
//! [`use-focus-within`](https://mantine.dev/hooks/use-focus-within/): pure, with no widgets.
//!
//! A scope has `total` focusable widgets, numbered in tree order, and at most one of them is
//! focused (`Some(index)`); the widgets ([`binds::FocusScope`](crate::binds::FocusScope),
//! [`binds::focus_return`](crate::binds::focus_return)) read and set that index.
//!
//! * **Trap** ([`trap_step`], [`initial`]): Tab and Shift+Tab move focus through the scope and
//!   wrap at its ends instead of leaving it; when the scope appears, [`initial`] says what takes
//!   focus (the widget marked as the autofocus one, else the first).
//! * **Return** ([`Return`]): remembers what was focused when a scope opened and gives it back
//!   when the scope closes.
//! * **Within** ([`Within`]): whether focus is anywhere inside a scope, with the moment it
//!   enters (`Focus`) and leaves (`Blur`).

/// Which way Tab moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    /// Tab.
    Forward,
    /// Shift+Tab.
    Backward,
}

impl Dir {
    /// Tab is `Forward`, Shift+Tab `Backward`.
    pub fn from_shift(shift: bool) -> Self {
        if shift { Self::Backward } else { Self::Forward }
    }
}

/// Where Tab sends focus in a scope of `total` focusable widgets with `focused` focused: the
/// next (or previous) one, wrapping at the ends so focus never leaves. With nothing focused,
/// Tab goes to the first and Shift+Tab to the last. `None` when the scope has nothing to focus.
pub fn trap_step(focused: Option<usize>, total: usize, dir: Dir) -> Option<usize> {
    if total == 0 {
        return None;
    }
    Some(match (focused, dir) {
        (None, Dir::Forward) => 0,
        (None, Dir::Backward) => total - 1,
        (Some(i), Dir::Forward) => (i + 1) % total,
        (Some(i), Dir::Backward) => (i.min(total - 1) + total - 1) % total,
    })
}

/// Where the widget named `target` is among a scope's focusable widgets: `ids` holds the id of each
/// (`None` for one without), in tree order. This is how an `initial_focus` id becomes the index
/// [`initial`] takes.
pub fn index_of<K: PartialEq>(ids: &[Option<K>], target: &K) -> Option<usize> {
    ids.iter().position(|id| id.as_ref() == Some(target))
}

/// What takes focus when a scope appears: the `autofocus` widget (see [`index_of`]) (clamped into the scope) or
/// else the first. `None` when the scope has nothing to focus.
pub fn initial(total: usize, autofocus: Option<usize>) -> Option<usize> {
    if total == 0 {
        None
    } else {
        Some(autofocus.map_or(0, |i| i.min(total - 1)))
    }
}

/// Remembers the focus from before a scope opened (`use-focus-return`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Return {
    open: bool,
    saved: Option<usize>,
    /// Whether closing gives focus back (Mantine's `shouldReturnFocus`, default true).
    pub enabled: bool,
}

impl Default for Return {
    fn default() -> Self {
        Self {
            open: false,
            saved: None,
            enabled: true,
        }
    }
}

impl Return {
    /// Feeds the scope's openness with what is `focused` now (the index among the widgets under
    /// the scope's owner). Returns the index to focus when the scope has just **closed**: the
    /// one that was focused when it opened, once. Opening saves `focused` and returns `None`.
    pub fn update(&mut self, open: bool, focused: Option<usize>) -> Option<usize> {
        match (self.open, open) {
            (false, true) => {
                self.open = true;
                self.saved = focused;
                None
            }
            (true, false) => {
                self.open = false;
                self.saved.take().filter(|_| self.enabled)
            }
            _ => None,
        }
    }
    pub fn is_open(&self) -> bool {
        self.open
    }
}

/// Focus entering or leaving a scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WithinEvent {
    Focus,
    Blur,
}

/// Whether focus is anywhere inside a scope (`use-focus-within`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Within {
    inside: bool,
}

impl Within {
    /// Feeds whether a widget in the scope is focused now; says when that changed.
    pub fn update(&mut self, inside: bool) -> Option<WithinEvent> {
        if inside == self.inside {
            return None;
        }
        self.inside = inside;
        Some(if inside {
            WithinEvent::Focus
        } else {
            WithinEvent::Blur
        })
    }
    pub fn focused(&self) -> bool {
        self.inside
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tab_moves_forward_and_wraps_at_the_end() {
        let step = |f, d| trap_step(f, 3, d);
        assert_eq!(step(Some(0), Dir::Forward), Some(1));
        assert_eq!(step(Some(2), Dir::Forward), Some(0), "wraps to the first");
        assert_eq!(step(Some(2), Dir::Backward), Some(1));
        assert_eq!(step(Some(0), Dir::Backward), Some(2), "wraps to the last");
    }

    #[test]
    fn with_nothing_focused_tab_enters_at_either_end() {
        assert_eq!(trap_step(None, 4, Dir::Forward), Some(0));
        assert_eq!(trap_step(None, 4, Dir::Backward), Some(3));
    }

    #[test]
    fn an_empty_scope_traps_nothing() {
        assert_eq!(trap_step(None, 0, Dir::Forward), None);
        assert_eq!(trap_step(Some(0), 0, Dir::Backward), None);
        assert_eq!(initial(0, Some(2)), None);
    }

    #[test]
    fn a_single_widget_keeps_focus() {
        assert_eq!(trap_step(Some(0), 1, Dir::Forward), Some(0));
        assert_eq!(trap_step(Some(0), 1, Dir::Backward), Some(0));
    }

    #[test]
    fn an_id_is_found_among_the_focusable_widgets() {
        let ids = [None, Some("name"), None, Some("email")];
        assert_eq!(index_of(&ids, &"email"), Some(3));
        assert_eq!(index_of(&ids, &"name"), Some(1));
        assert_eq!(index_of(&ids, &"phone"), None);
        // A missing id falls back to the first widget.
        assert_eq!(initial(ids.len(), index_of(&ids, &"phone")), Some(0));
    }

    #[test]
    fn the_autofocus_widget_or_the_first_starts_focused() {
        assert_eq!(initial(3, None), Some(0));
        assert_eq!(initial(3, Some(2)), Some(2));
        assert_eq!(initial(3, Some(9)), Some(2), "clamped into the scope");
    }

    #[test]
    fn closing_gives_back_what_was_focused_when_it_opened_once() {
        let mut r = Return::default();
        assert_eq!(r.update(false, Some(4)), None, "closed and staying closed");
        assert_eq!(
            r.update(true, Some(4)),
            None,
            "opening saves, returns nothing"
        );
        assert_eq!(
            r.update(true, Some(9)),
            None,
            "later focus is the scope's own"
        );
        assert_eq!(r.update(false, None), Some(4));
        assert_eq!(r.update(false, None), None, "only once");
    }

    #[test]
    fn nothing_focused_at_opening_returns_nothing() {
        let mut r = Return::default();
        r.update(true, None);
        assert_eq!(r.update(false, Some(1)), None);
    }

    #[test]
    fn returning_can_be_turned_off() {
        let mut r = Return {
            enabled: false,
            ..Return::default()
        };
        r.update(true, Some(2));
        assert_eq!(r.update(false, None), None);
    }

    #[test]
    fn within_reports_the_moment_focus_enters_and_leaves() {
        let mut w = Within::default();
        assert_eq!(w.update(false), None);
        assert_eq!(w.update(true), Some(WithinEvent::Focus));
        assert_eq!(w.update(true), None, "moving inside is not an event");
        assert!(w.focused());
        assert_eq!(w.update(false), Some(WithinEvent::Blur));
    }
}
