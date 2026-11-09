//! A value being edited, with the value it started from: **reset** goes back to it (a Cancel or
//! Reset button), **commit** makes the edited value the new start (Apply or Done). Headless, with
//! no widgets.
//!
//! It dereferences to the edited value, so the editing code does not change:
//!
//! ```
//! use iced_adaptive_design::primitives::{draft::Draft, tag_groups::TagGroups};
//!
//! let mut filter = Draft::new(TagGroups::new());
//! filter.toggle("rock");                 // edits the value, through Deref
//! assert!(filter.is_dirty());
//! filter.reset();                        // back to the start
//! assert!(!filter.is_dirty() && filter.is_empty());
//! ```
use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft<T> {
    initial: T,
    value: T,
}

impl<T: Clone> Draft<T> {
    /// Starts editing `initial`; the value is a copy of it.
    pub fn new(initial: T) -> Self {
        Self {
            value: initial.clone(),
            initial,
        }
    }
    /// The edited value.
    pub fn value(&self) -> &T {
        &self.value
    }
    pub fn value_mut(&mut self) -> &mut T {
        &mut self.value
    }
    /// The value it started from: what [`reset`](Self::reset) returns to.
    pub fn initial(&self) -> &T {
        &self.initial
    }
    /// Whether the edited value differs from the one it started from (to enable Reset or Apply).
    pub fn is_dirty(&self) -> bool
    where
        T: PartialEq,
    {
        self.value != self.initial
    }
    /// Goes back to the value it started from, dropping every edit.
    pub fn reset(&mut self) {
        self.value = self.initial.clone();
    }
    /// Makes the edited value the one it starts from, so there is nothing to reset.
    pub fn commit(&mut self) {
        self.initial = self.value.clone();
    }
    /// Starts again from `initial` (a new thing to edit): the value becomes it too.
    pub fn restart(&mut self, initial: T) {
        self.value = initial.clone();
        self.initial = initial;
    }
    /// The edited value, leaving nothing to reset: for when the edit is applied.
    pub fn into_value(self) -> T {
        self.value
    }
}

impl<T> Deref for Draft<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}
impl<T> DerefMut for Draft<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reset_goes_back_to_the_initial_value_and_is_clean_again() {
        let mut d = Draft::new(vec![1, 2]);
        assert!(!d.is_dirty());
        d.push(3);
        d.remove(0);
        assert_eq!(d.value(), &[2, 3]);
        assert!(d.is_dirty());
        d.reset();
        assert_eq!(d.value(), &[1, 2]);
        assert!(!d.is_dirty());
    }

    #[test]
    fn editing_back_to_the_initial_value_is_not_dirty() {
        let mut d = Draft::new(1);
        *d = 2;
        assert!(d.is_dirty());
        *d = 1;
        assert!(!d.is_dirty());
    }

    #[test]
    fn commit_moves_the_start_so_reset_keeps_the_edit() {
        let mut d = Draft::new(String::from("a"));
        d.push('b');
        d.commit();
        assert!(!d.is_dirty());
        d.push('c');
        d.reset();
        assert_eq!((d.value().as_str(), d.initial().as_str()), ("ab", "ab"));
    }

    #[test]
    fn restart_replaces_both_values() {
        let mut d = Draft::new(1);
        *d = 5;
        d.restart(9);
        assert_eq!((*d.value(), *d.initial(), d.is_dirty()), (9, 9, false));
        assert_eq!(d.into_value(), 9);
    }
}
