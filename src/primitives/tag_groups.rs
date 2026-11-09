//! Tag groups: sets of tags where a track must have every tag of one group, and several groups
//! are alternatives. Headless, with no widgets.
//!
//! One group is the plain "has all of these tags" filter. With several groups, a track matches
//! when it matches any group (`a` and `b`, or `c`). One group is *active*:
//! toggling a tag changes that group, and the group at the end is the slot for a new one.
//! A group left empty is dropped when the active group moves away from it.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagGroups<K> {
    groups: Vec<Vec<K>>,
    active: usize,
}

impl<K> Default for TagGroups<K> {
    fn default() -> Self {
        Self {
            groups: vec![vec![]],
            active: 0,
        }
    }
}

/// Which way to move the active group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Previous,
    Next,
}

/// What a view of the groups can ask for; [`TagGroups::apply`] does it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupEvent<K> {
    /// Toggle a tag in a group (which becomes the active one).
    Toggle(usize, K),
    /// Make a group the active one.
    Select(usize),
    /// Move the active group.
    Step(Step),
    /// Finish the active group and start a new one.
    AddGroup,
}

impl<K: PartialEq + Clone> TagGroups<K> {
    /// Does what `event` asks.
    pub fn apply(&mut self, event: GroupEvent<K>) {
        match event {
            GroupEvent::Toggle(group, key) => self.toggle_in(group, key),
            GroupEvent::Select(group) => self.select(group),
            GroupEvent::Step(step) => self.step(step),
            GroupEvent::AddGroup => self.add_group(),
        }
    }
    pub fn new() -> Self {
        Self::default()
    }
    /// Groups built from the app's own state, with `active` the one being edited. At least one
    /// group is kept, and `active` is clamped into range. Empty groups are kept as they are.
    pub fn from_parts(mut groups: Vec<Vec<K>>, active: usize) -> Self {
        if groups.is_empty() {
            groups.push(vec![]);
        }
        let active = active.min(groups.len() - 1);
        Self { groups, active }
    }
    /// All groups, in order. There is always at least one.
    pub fn groups(&self) -> &[Vec<K>] {
        &self.groups
    }
    pub fn active(&self) -> usize {
        self.active
    }
    /// The tags of the active group.
    pub fn selected(&self) -> &[K] {
        &self.groups[self.active]
    }
    /// No tag is chosen in any group.
    pub fn is_empty(&self) -> bool {
        self.groups.iter().all(Vec::is_empty)
    }
    /// Adds `key` to the active group, or removes it when it is there. Removing the last tag of a
    /// group drops the group (unless it is the only one), so the groups after it move up; the
    /// group before it, or the one now first, becomes active.
    pub fn toggle(&mut self, key: K) {
        toggle(&mut self.groups[self.active], key);
        if self.groups[self.active].is_empty() && self.groups.len() > 1 {
            self.groups.remove(self.active);
            self.active = self.active.saturating_sub(1);
        }
    }
    /// Toggles `key` in `group` (out of range does nothing), and makes that group active.
    pub fn toggle_in(&mut self, group: usize, key: K) {
        if group < self.groups.len() {
            self.select(group);
            self.toggle(key);
        }
    }
    /// Makes `group` active. An index out of range changes nothing.
    pub fn select(&mut self, group: usize) {
        if group >= self.groups.len() {
            return;
        }
        self.active = group;
        self.tidy();
    }
    /// Moves the active group one step; the step past the last group opens the slot for a new
    /// one, when the last is not empty already.
    pub fn step(&mut self, step: Step) {
        match step {
            Step::Previous if self.active > 0 => self.select(self.active - 1),
            Step::Next if self.active + 1 < self.groups.len() => self.select(self.active + 1),
            Step::Next => self.add_group(),
            Step::Previous => {}
        }
    }
    /// Finishes the active group and starts a new one. A group still empty is reused.
    pub fn add_group(&mut self) {
        self.tidy();
        if self.groups.last().is_some_and(|g| !g.is_empty()) {
            self.groups.push(vec![]);
        }
        self.active = self.groups.len() - 1;
    }
    /// Clears every group.
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    /// Whether a track with `tags` matches: every tag of some group is among them. Nothing chosen
    /// matches everything.
    pub fn matches(&self, tags: &[K]) -> bool {
        self.is_empty()
            || self
                .groups
                .iter()
                .filter(|g| !g.is_empty())
                .any(|g| g.iter().all(|t| tags.contains(t)))
    }
    /// Drops empty groups other than the active one (and keeps one group at least).
    fn tidy(&mut self) {
        let keep = self.active;
        let mut index = 0;
        self.groups.retain(|group| {
            let kept = index == keep || !group.is_empty();
            if !kept && index < keep {
                self.active -= 1;
            }
            index += 1;
            kept
        });
        if self.groups.is_empty() {
            self.groups.push(vec![]);
            self.active = 0;
        }
    }
}

fn toggle<K: PartialEq>(group: &mut Vec<K>, key: K) {
    match group.iter().position(|k| *k == key) {
        Some(at) => {
            group.remove(at);
        }
        None => group.push(key),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_parts_keeps_a_group_and_clamps_the_active_one() {
        let g = TagGroups::from_parts(vec![vec!["a"], vec![]], 7);
        assert_eq!((g.groups().len(), g.active()), (2, 1));
        let g = TagGroups::<&str>::from_parts(vec![], 3);
        assert_eq!((g.groups().len(), g.active()), (1, 0));
    }

    #[test]
    fn one_group_is_all_of() {
        let mut g = TagGroups::new();
        g.toggle("a");
        g.toggle("b");
        assert_eq!(g.groups(), [vec!["a", "b"]]);
        assert!(g.matches(&["a", "b", "c"]));
        assert!(!g.matches(&["a"]));
        g.toggle("a");
        assert_eq!(g.selected(), ["b"]);
    }

    #[test]
    fn groups_are_alternatives_and_the_last_slot_is_for_a_new_one() {
        let mut g = TagGroups::new();
        g.toggle("a");
        g.toggle("b");
        g.add_group();
        g.add_group(); // the empty slot is reused
        assert_eq!((g.groups().len(), g.active()), (2, 1));
        g.toggle("c");
        assert!(g.matches(&["a", "b"]));
        assert!(g.matches(&["c"]));
        assert!(!g.matches(&["a"]));
        assert!(!g.matches(&[]));
        assert!(TagGroups::<&str>::new().matches(&[]));
    }

    #[test]
    fn an_empty_group_is_dropped_when_the_active_group_leaves_it() {
        let mut g = TagGroups::new();
        g.toggle("a");
        g.add_group();
        g.step(Step::Previous);
        assert_eq!((g.groups(), g.active()), (&[vec!["a"]][..], 0));
        g.add_group();
        g.toggle("b");
        g.select(0);
        g.toggle("a"); // group 0 is empty now
        g.select(1);
        assert_eq!((g.groups(), g.active()), (&[vec!["b"]][..], 0));
    }

    #[test]
    fn stepping_walks_the_groups_and_opens_the_new_slot_at_the_end() {
        let mut g = TagGroups::new();
        g.toggle("a");
        g.add_group();
        g.toggle("b");
        g.step(Step::Previous);
        assert_eq!(g.active(), 0);
        g.step(Step::Next);
        assert_eq!(g.active(), 1);
        g.step(Step::Next);
        assert_eq!((g.groups().len(), g.active()), (3, 2));
        g.step(Step::Next); // already on the empty slot
        assert_eq!((g.groups().len(), g.active()), (3, 2));
    }

    #[test]
    fn emptying_a_group_by_removal_drops_it_so_later_groups_move_up() {
        let mut g = TagGroups::new();
        g.toggle("a");
        g.add_group();
        g.toggle("b");
        g.add_group();
        g.toggle("c");
        g.toggle_in(0, "a");
        assert_eq!((g.groups(), g.active()), (&[vec!["b"], vec!["c"]][..], 0));
        g.toggle_in(1, "c");
        assert_eq!((g.groups(), g.active()), (&[vec!["b"]][..], 0));
        g.toggle("b"); // the only group stays, empty
        assert_eq!((g.groups(), g.active()), (&[vec![]][..], 0));
    }

    #[test]
    fn events_do_what_their_methods_do() {
        let mut g = TagGroups::new();
        g.apply(GroupEvent::Toggle(0, "a"));
        g.apply(GroupEvent::AddGroup);
        g.apply(GroupEvent::Toggle(1, "b"));
        g.apply(GroupEvent::Step(Step::Previous));
        g.apply(GroupEvent::Select(1));
        assert_eq!((g.groups(), g.active()), (&[vec!["a"], vec!["b"]][..], 1));
    }

    #[test]
    fn toggling_in_a_group_activates_it_and_clear_keeps_the_mode() {
        let mut g = TagGroups::new();
        g.toggle("a");
        g.add_group();
        g.toggle("b");
        g.toggle_in(0, "c");
        assert_eq!(g.active(), 0);
        assert_eq!(g.groups()[0], ["a", "c"]);
        g.clear();
        assert!(g.is_empty());
    }
}
