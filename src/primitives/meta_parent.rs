//! Headless logic of a meta parent: a registry of the children that can switch mode, and the
//! notification that reaches them.
//!
//! Each child **registers separately** and announces the actions it implements (for a
//! [`MetaComponent`](crate::widget::MetaComponent), the keys its variants are selected by). A
//! [`notify`](Registry::notify) switches exactly the children that announced the action:
//!
//! * a child that did not announce it is *unrelated* and is left as it is;
//! * a child that never registered (any non-meta widget) is never notified.
//!
//! The other direction: a child **reports** what it did ([`ChildEvent`]); [`Registry::emit`]
//! keeps the report of a registered child and drops the rest, and the app reacts on its own
//! (for instance by notifying other children).
//!
//! Controlled like every primitive: the app owns the registry and calls `notify` from its
//! `update`; the widget only reads [`Registry::current`].

/// What a child reports back to its parent: the other direction from a notification, so the
/// parent can react to what a child did without the child knowing who is listening.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildEvent {
    /// A reorderable child's row moved: it was at `from` and now belongs at `to`.
    Reordered { from: usize, to: usize },
}

/// A [`ChildEvent`] from a child that is registered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Emitted<'a, Id> {
    pub child: &'a Id,
    pub event: ChildEvent,
}

/// One registered child.
#[derive(Debug, Clone, PartialEq)]
struct Entry<Id, K> {
    id: Id,
    actions: Vec<K>,
    current: Option<K>,
}

/// The children of a meta parent: who they are, what each announced, and the action each was
/// last switched to.
///
/// ```
/// use iced_libadwaita::primitives::meta_parent::Registry;
///
/// let mut r = Registry::new();
/// r.register("list", ["edit", "preview"]);
/// r.register("bar", ["search"]);
/// assert_eq!(r.notify(&"edit"), vec![&"list"]);   // "bar" is unrelated
/// assert_eq!(r.current(&"list"), Some(&"edit"));
/// assert_eq!(r.current(&"bar"), None);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Registry<Id, K> {
    entries: Vec<Entry<Id, K>>,
}

impl<Id, K> Default for Registry<Id, K> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<Id: PartialEq, K: PartialEq> Registry<Id, K> {
    pub fn new() -> Self {
        Self::default()
    }
    /// Registers a child and what it announces. Registering an id again replaces its actions,
    /// keeping its current action while it is still announced.
    pub fn register(&mut self, id: Id, actions: impl IntoIterator<Item = K>) {
        let mut actions: Vec<K> = actions.into_iter().collect();
        let mut dedup: Vec<K> = Vec::with_capacity(actions.len());
        for a in actions.drain(..) {
            if !dedup.contains(&a) {
                dedup.push(a);
            }
        }
        match self.entries.iter_mut().find(|e| e.id == id) {
            Some(e) => {
                if e.current.as_ref().is_some_and(|c| !dedup.contains(c)) {
                    e.current = None;
                }
                e.actions = dedup;
            }
            None => self.entries.push(Entry {
                id,
                actions: dedup,
                current: None,
            }),
        }
    }
    /// Removes a child; it is never notified again.
    pub fn unregister(&mut self, id: &Id) {
        self.entries.retain(|e| e.id != *id);
    }
    pub fn is_registered(&self, id: &Id) -> bool {
        self.entries.iter().any(|e| e.id == *id)
    }
    /// What `id` announced (empty when it is not registered).
    pub fn actions(&self, id: &Id) -> &[K] {
        self.entries
            .iter()
            .find(|e| e.id == *id)
            .map_or(&[], |e| &e.actions)
    }
    /// The children that announced `action`.
    pub fn listeners<'a>(&'a self, action: &'a K) -> impl Iterator<Item = &'a Id> {
        self.entries
            .iter()
            .filter(move |e| e.actions.contains(action))
            .map(|e| &e.id)
    }
    /// Switches every child that announced `action` to it and returns them. The others are not
    /// touched.
    pub fn notify(&mut self, action: &K) -> Vec<&Id>
    where
        K: Clone,
    {
        let mut hit = Vec::new();
        for e in &mut self.entries {
            if e.actions.contains(action) {
                e.current = Some(action.clone());
                hit.push(&e.id);
            }
        }
        hit
    }
    /// Switches only `id`, if it announced `action`. Returns whether it did.
    pub fn notify_one(&mut self, id: &Id, action: &K) -> bool
    where
        K: Clone,
    {
        match self
            .entries
            .iter_mut()
            .find(|e| e.id == *id && e.actions.contains(action))
        {
            Some(e) => {
                e.current = Some(action.clone());
                true
            }
            None => false,
        }
    }
    /// A child's report, kept only when the child registered: an event from an unregistered
    /// child (a plain widget) is ignored, the mirror of it never being notified. The app calls
    /// this from `update` and runs its independent reaction on `Some`; the reaction may
    /// [`notify`](Self::notify) other children.
    pub fn emit<'a>(&'a self, id: &'a Id, event: ChildEvent) -> Option<Emitted<'a, Id>> {
        let child = &self.entries.iter().find(|e| e.id == *id)?.id;
        Some(Emitted { child, event })
    }
    /// Takes every child back to no action (each shows its fallback).
    pub fn reset(&mut self) {
        for e in &mut self.entries {
            e.current = None;
        }
    }
    /// The action `id` was last switched to.
    pub fn current(&self, id: &Id) -> Option<&K> {
        self.entries
            .iter()
            .find(|e| e.id == *id)
            .and_then(|e| e.current.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reg() -> Registry<&'static str, &'static str> {
        let mut r = Registry::new();
        r.register("list", ["edit", "preview"]);
        r.register("table", ["edit", "cards"]);
        r.register("bar", ["search"]);
        r
    }

    #[test]
    fn a_notification_reaches_only_those_that_announced_it() {
        let mut r = reg();
        assert_eq!(r.notify(&"edit"), vec![&"list", &"table"]);
        assert_eq!(r.current(&"list"), Some(&"edit"));
        assert_eq!(r.current(&"table"), Some(&"edit"));
        assert_eq!(r.current(&"bar"), None, "unrelated");
    }

    #[test]
    fn an_action_nobody_announced_notifies_nobody() {
        let mut r = reg();
        assert!(r.notify(&"zoom").is_empty());
        assert!(
            ["list", "table", "bar"]
                .iter()
                .all(|i| r.current(i).is_none())
        );
    }

    #[test]
    fn unregistered_children_are_never_notified() {
        let mut r = reg();
        r.notify(&"edit");
        assert_eq!(r.current(&"plain"), None);
        assert!(!r.notify_one(&"plain", &"edit"));
    }

    #[test]
    fn notifying_one_leaves_the_others() {
        let mut r = reg();
        assert!(r.notify_one(&"table", &"cards"));
        assert!(!r.notify_one(&"table", &"search"));
        assert_eq!(r.current(&"table"), Some(&"cards"));
        assert_eq!(r.current(&"list"), None);
    }

    #[test]
    fn a_later_action_replaces_an_earlier_one_and_others_keep_theirs() {
        let mut r = reg();
        r.notify(&"search");
        r.notify(&"preview");
        assert_eq!(r.current(&"list"), Some(&"preview"));
        assert_eq!(r.current(&"bar"), Some(&"search"));
    }

    #[test]
    fn reregistering_updates_the_announcement() {
        let mut r = reg();
        r.notify(&"edit");
        r.register("list", ["preview"]);
        assert_eq!(r.current(&"list"), None, "edit is no longer announced");
        assert_eq!(r.actions(&"list"), ["preview"]);
        assert_eq!(r.notify(&"edit"), vec![&"table"]);
        r.register("table", ["edit"]);
        assert_eq!(r.current(&"table"), Some(&"edit"), "still announced: kept");
    }

    #[test]
    fn only_registered_children_can_report() {
        let r = reg();
        let ev = ChildEvent::Reordered { from: 0, to: 2 };
        let e = r.emit(&"list", ev).expect("registered");
        assert_eq!((e.child, e.event), (&"list", ev));
        assert!(r.emit(&"plain", ev).is_none());
    }

    #[test]
    fn unregistering_and_reset() {
        let mut r = reg();
        r.notify(&"edit");
        r.reset();
        assert_eq!(r.current(&"list"), None);
        r.unregister(&"list");
        assert!(!r.is_registered(&"list"));
        assert_eq!(r.listeners(&"edit").collect::<Vec<_>>(), vec![&"table"]);
    }
}
