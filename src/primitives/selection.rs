//! `Selection`: the rule for "which of these keys are on", shared by everything that picks
//! from a collection (accordion, multi-select, toggle groups, tabs). Pure, controlled: the
//! application owns the list of keys and applies the result.

/// How many keys may be on at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Selection {
    /// At most one. `required`: the one that is on cannot be switched off (a tab bar), else
    /// activating it again clears it (a collapsible accordion).
    Single { required: bool },
    /// Any number.
    #[default]
    Multiple,
}

impl Selection {
    /// One key on at a time, and it can be turned off.
    pub const SINGLE: Self = Self::Single { required: false };

    /// The keys that are on after `key` is activated.
    pub fn toggle<K: Clone + PartialEq>(self, on: &[K], key: &K) -> Vec<K> {
        let is_on = on.contains(key);
        match self {
            Self::Multiple if is_on => on.iter().filter(|k| *k != key).cloned().collect(),
            Self::Multiple => on.iter().cloned().chain([key.clone()]).collect(),
            Self::Single { required: true } => vec![key.clone()],
            Self::Single { required: false } if is_on => Vec::new(),
            Self::Single { required: false } => vec![key.clone()],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiple_toggles_independently() {
        assert_eq!(Selection::Multiple.toggle(&[1], &2), [1, 2]);
        assert_eq!(Selection::Multiple.toggle(&[1, 2], &1), [2]);
    }

    #[test]
    fn single_replaces_and_may_clear() {
        assert_eq!(Selection::SINGLE.toggle(&[1], &2), [2]);
        assert_eq!(Selection::SINGLE.toggle(&[2], &2), Vec::<i32>::new());
    }

    #[test]
    fn a_required_single_never_clears() {
        let s = Selection::Single { required: true };
        assert_eq!(s.toggle(&[2], &2), [2]);
        assert_eq!(s.toggle(&[2], &3), [3]);
    }
}
