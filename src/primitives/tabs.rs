//! Headless logic of a row of exclusive tabs: which can be activated, and where the arrow keys
//! go. After Radix UI's Tabs; shared by [`Tabs`](crate::widget::Tabs) and the segmented control
//! behind [`toggle_group`](crate::widget::toggle_group), which are the same behavior with a
//! different look.
//!
//! Controlled like every primitive: the app owns which tab is active. A [`TabList`] only knows
//! how many tabs there are and which are disabled, and answers:
//!
//! * [`activate`](TabList::activate): the tab a click on `i` activates (none when it is disabled);
//! * [`step`](TabList::step): the tab an arrow key moves to, skipping disabled ones, with or
//!   without wrapping at the ends;
//! * [`first`](TabList::first) / [`last`](TabList::last): Home and End.

/// The direction of an arrow key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Next,
    Previous,
}

/// The tabs of one list: how many, and which are disabled.
///
/// ```
/// use iced_adaptive_design::primitives::tabs::{Dir, TabList};
///
/// let tabs = TabList::with_disabled(3, vec![false, true, false]);
/// assert_eq!(tabs.activate(1), None);                    // disabled
/// assert_eq!(tabs.step(Some(0), Dir::Next, false), Some(2)); // skips the disabled one
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TabList {
    disabled: Vec<bool>,
}

impl TabList {
    /// `len` tabs, all enabled.
    pub fn new(len: usize) -> Self {
        Self {
            disabled: vec![false; len],
        }
    }
    /// `len` tabs with the given flags (one per tab). A list of the wrong length is ignored
    /// rather than guessed at, and every tab stays enabled.
    pub fn with_disabled(len: usize, flags: Vec<bool>) -> Self {
        if flags.len() == len {
            Self { disabled: flags }
        } else {
            Self::new(len)
        }
    }
    pub fn len(&self) -> usize {
        self.disabled.len()
    }
    pub fn is_empty(&self) -> bool {
        self.disabled.is_empty()
    }
    /// Whether tab `i` exists and can be activated.
    pub fn is_enabled(&self, i: usize) -> bool {
        self.disabled.get(i).is_some_and(|d| !d)
    }
    /// The tab a click on `i` activates: `i` itself when it is enabled.
    pub fn activate(&self, i: usize) -> Option<usize> {
        self.is_enabled(i).then_some(i)
    }
    /// The first enabled tab (Home).
    pub fn first(&self) -> Option<usize> {
        (0..self.len()).find(|i| self.is_enabled(*i))
    }
    /// The last enabled tab (End).
    pub fn last(&self) -> Option<usize> {
        (0..self.len()).rev().find(|i| self.is_enabled(*i))
    }
    /// The enabled tab an arrow key moves to from `from` (none yet: `Next` starts at the first,
    /// `Previous` at the last). At an end it stops, or with `wrap` comes round to the other end.
    /// `None` when there is nowhere to go.
    pub fn step(&self, from: Option<usize>, dir: Dir, wrap: bool) -> Option<usize> {
        let Some(from) = from else {
            return match dir {
                Dir::Next => self.first(),
                Dir::Previous => self.last(),
            };
        };
        let n = self.len();
        (1..n)
            .map(|d| match dir {
                Dir::Next => from + d,
                Dir::Previous => (from + n).wrapping_sub(d),
            })
            .map(|i| (i, wrapped(dir, i, n)))
            .take_while(|(_, w)| wrap || !*w)
            .map(|(i, _)| i % n)
            .find(|i| self.is_enabled(*i))
    }
}

/// Whether stepping to the unreduced index `i` went past an end.
fn wrapped(dir: Dir, i: usize, n: usize) -> bool {
    match dir {
        Dir::Next => i >= n,
        // `from + n - d` is below `n` exactly when `d > from`: it went below the first tab.
        Dir::Previous => i < n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list() -> TabList {
        TabList::with_disabled(5, vec![false, true, false, true, false])
    }

    #[test]
    fn only_enabled_tabs_activate() {
        let t = list();
        assert_eq!(t.activate(0), Some(0));
        assert_eq!(t.activate(1), None);
        assert_eq!(t.activate(9), None, "out of range");
    }

    #[test]
    fn a_list_of_the_wrong_length_is_ignored() {
        let t = TabList::with_disabled(3, vec![true]);
        assert_eq!(t.activate(0), Some(0));
    }

    #[test]
    fn home_and_end_are_the_first_and_last_enabled() {
        let t = TabList::with_disabled(4, vec![true, false, false, true]);
        assert_eq!((t.first(), t.last()), (Some(1), Some(2)));
        assert_eq!(TabList::new(0).first(), None);
        assert_eq!(TabList::with_disabled(1, vec![true]).last(), None);
    }

    #[test]
    fn arrows_skip_disabled_tabs() {
        let t = list();
        assert_eq!(t.step(Some(0), Dir::Next, false), Some(2));
        assert_eq!(t.step(Some(2), Dir::Next, false), Some(4));
        assert_eq!(t.step(Some(4), Dir::Previous, false), Some(2));
        assert_eq!(t.step(Some(2), Dir::Previous, false), Some(0));
    }

    #[test]
    fn arrows_stop_at_the_ends_unless_they_wrap() {
        let t = list();
        assert_eq!(t.step(Some(4), Dir::Next, false), None);
        assert_eq!(t.step(Some(0), Dir::Previous, false), None);
        assert_eq!(t.step(Some(4), Dir::Next, true), Some(0));
        assert_eq!(t.step(Some(0), Dir::Previous, true), Some(4));
    }

    #[test]
    fn arrows_from_nothing_start_at_an_end() {
        let t = list();
        assert_eq!(t.step(None, Dir::Next, false), Some(0));
        assert_eq!(t.step(None, Dir::Previous, false), Some(4));
    }

    #[test]
    fn a_single_tab_has_nowhere_to_go() {
        let t = TabList::new(1);
        assert_eq!(t.step(Some(0), Dir::Next, true), None);
        assert_eq!(t.step(Some(0), Dir::Previous, true), None);
    }
}
