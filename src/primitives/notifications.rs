//! Notifications: the queue behind stacked toasts, after Mantine's notifications store
//! (`notifications.show / update / hide / clean`, `limit`, `autoClose`). Pure and controlled:
//! the application owns one [`Queue`], every operation that depends on time takes `now`
//! explicitly, and nothing here draws.
//!
//! * [`push`](Queue::push) adds a notification and returns its [`Id`]; above the
//!   [`limit`](Queue::limit) the oldest are dropped first;
//! * [`replace`](Queue::replace) swaps the content of a live one in place (a toast whose action
//!   ran becomes a confirmation) and restarts its timer;
//! * [`dismiss`](Queue::dismiss) / [`clear`](Queue::clear) remove by hand;
//! * with a [`timeout`](Queue::timeout), [`expire`](Queue::expire)`(now)` removes the ones whose
//!   time has come, and [`next_expiry`](Queue::next_expiry) says when to look again (so the
//!   app ticks only while something can expire).
//!
//! ```ignore
//! // update:
//! Msg::Show(text) => { self.toasts.push(text, Instant::now()); }
//! Msg::Dismiss(id) => { self.toasts.dismiss(id); }
//! Msg::Tick(now) => { self.toasts.expire(now); }
//! // view: toast_stack_from(content, &self.toasts, |id, t| Toast::new(*t).on_dismiss(Msg::Dismiss(id)))
//! // subscription: self.toasts.next_expiry().map(|_| iced::time::every(Duration::from_millis(250)).map(Msg::Tick))
//! ```

use std::time::{Duration, Instant};

/// The identity of a notification in its [`Queue`]; never reused.
pub type Id = u32;

#[derive(Debug, Clone)]
struct Entry<T> {
    id: Id,
    value: T,
    expires: Option<Instant>,
}

/// An ordered set of live notifications, oldest first.
#[derive(Debug, Clone)]
pub struct Queue<T> {
    entries: Vec<Entry<T>>,
    next_id: Id,
    limit: usize,
    timeout: Option<Duration>,
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Queue<T> {
    /// An empty queue: at most 4 live, no timeout (the app dismisses).
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 0,
            limit: 4,
            timeout: None,
        }
    }

    /// At most `n` live at once (default 4, minimum 1); beyond it the oldest are dropped.
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = n.max(1);
        self
    }

    /// Dismiss each notification this long after it was pushed (or replaced). `None`, the
    /// default, keeps them until dismissed.
    pub fn timeout(mut self, timeout: Option<Duration>) -> Self {
        self.timeout = timeout;
        self
    }

    /// Changes the timeout of a queue in use; it applies to notifications pushed afterwards.
    pub fn set_timeout(&mut self, timeout: Option<Duration>) {
        self.timeout = timeout;
    }

    /// Adds a notification at `now`; returns its id.
    pub fn push(&mut self, value: T, now: Instant) -> Id {
        self.next_id = self.next_id.wrapping_add(1);
        let id = self.next_id;
        self.entries.push(Entry {
            id,
            value,
            expires: self.timeout.map(|t| now + t),
        });
        let extra = self.entries.len().saturating_sub(self.limit);
        self.entries.drain(..extra);
        id
    }

    /// Swaps the content of notification `id` in place and restarts its timer. `false` when
    /// it is gone (dismissed or expired), in which case nothing is added.
    pub fn replace(&mut self, id: Id, value: T, now: Instant) -> bool {
        let timeout = self.timeout;
        match self.entries.iter_mut().find(|e| e.id == id) {
            Some(e) => {
                e.value = value;
                e.expires = timeout.map(|t| now + t);
                true
            }
            None => false,
        }
    }

    /// Removes notification `id`, returning its content.
    pub fn dismiss(&mut self, id: Id) -> Option<T> {
        let i = self.entries.iter().position(|e| e.id == id)?;
        Some(self.entries.remove(i).value)
    }

    /// Removes everything.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Removes the notifications whose time is up at `now`; returns how many.
    pub fn expire(&mut self, now: Instant) -> usize {
        let before = self.entries.len();
        self.entries.retain(|e| e.expires.is_none_or(|t| t > now));
        before - self.entries.len()
    }

    /// The earliest moment something will expire, `None` when nothing can (no timeout, or
    /// empty): the app needs a timer only while this is `Some`.
    pub fn next_expiry(&self) -> Option<Instant> {
        self.entries.iter().filter_map(|e| e.expires).min()
    }

    pub fn get(&self, id: Id) -> Option<&T> {
        self.entries.iter().find(|e| e.id == id).map(|e| &e.value)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// The live notifications, oldest first.
    pub fn iter(&self) -> impl Iterator<Item = (Id, &T)> {
        self.entries.iter().map(|e| (e.id, &e.value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids<T>(q: &Queue<T>) -> Vec<Id> {
        q.iter().map(|(i, _)| i).collect()
    }

    #[test]
    fn push_numbers_and_caps_at_the_limit() {
        let now = Instant::now();
        let mut q = Queue::new().limit(3);
        let a = q.push("a", now);
        q.push("b", now);
        q.push("c", now);
        assert_eq!(q.len(), 3);
        let d = q.push("d", now);
        assert_eq!(q.len(), 3);
        assert!(q.get(a).is_none(), "the oldest went first");
        assert_eq!(q.iter().last().map(|(i, _)| i), Some(d));
        assert_eq!(
            q.iter().map(|(_, v)| *v).collect::<Vec<_>>(),
            ["b", "c", "d"]
        );
    }

    #[test]
    fn ids_are_not_reused() {
        let now = Instant::now();
        let mut q = Queue::new();
        let a = q.push(1, now);
        q.dismiss(a);
        let b = q.push(2, now);
        assert_ne!(a, b);
        assert_eq!(q.dismiss(a), None, "already gone");
    }

    #[test]
    fn replace_keeps_place_and_id() {
        let now = Instant::now();
        let mut q = Queue::new();
        let a = q.push("undo", now);
        let b = q.push("other", now);
        assert!(q.replace(a, "restored", now));
        assert_eq!(ids(&q), [a, b]);
        assert_eq!(q.get(a), Some(&"restored"));
        q.dismiss(a);
        assert!(!q.replace(a, "late", now), "a dismissed one is not revived");
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn without_a_timeout_nothing_expires() {
        let now = Instant::now();
        let mut q = Queue::new();
        q.push(1, now);
        assert_eq!(q.next_expiry(), None);
        assert_eq!(q.expire(now + Duration::from_secs(3600)), 0);
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn expiry_is_per_notification_and_exclusive_of_the_deadline() {
        let t0 = Instant::now();
        let mut q = Queue::new().timeout(Some(Duration::from_secs(4)));
        let a = q.push("a", t0);
        let b = q.push("b", t0 + Duration::from_secs(2));
        assert_eq!(q.next_expiry(), Some(t0 + Duration::from_secs(4)));
        assert_eq!(q.expire(t0 + Duration::from_secs(3)), 0);
        assert_eq!(q.expire(t0 + Duration::from_secs(4)), 1);
        assert!(q.get(a).is_none() && q.get(b).is_some());
        assert_eq!(q.next_expiry(), Some(t0 + Duration::from_secs(6)));
        assert_eq!(q.expire(t0 + Duration::from_secs(60)), 1);
        assert!(q.is_empty());
        assert_eq!(q.next_expiry(), None);
    }

    #[test]
    fn replace_restarts_the_timer() {
        let t0 = Instant::now();
        let mut q = Queue::new().timeout(Some(Duration::from_secs(4)));
        let a = q.push("undo", t0);
        q.replace(a, "restored", t0 + Duration::from_secs(3));
        assert_eq!(q.expire(t0 + Duration::from_secs(5)), 0);
        assert_eq!(q.expire(t0 + Duration::from_secs(7)), 1);
    }

    #[test]
    fn a_zero_limit_still_keeps_one() {
        let now = Instant::now();
        let mut q = Queue::new().limit(0);
        q.push(1, now);
        q.push(2, now);
        assert_eq!(q.len(), 1);
    }
}
