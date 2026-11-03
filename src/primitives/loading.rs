//! Headless async loading: which keys are loading, which failed, and which chunks of a long list
//! still have to be fetched.
//!
//! Two small pieces, both pure and controlled (the app starts the fetch and reports the result):
//!
//! * [`Loads`]: a set of keys in flight and the error of those that failed. A
//!   [`Tree`](crate::widget::Tree)'s lazy branches use it (`begin` when a branch opens, `finish`
//!   when its children arrive, `retry` after an error), and so can anything keyed: a row, a
//!   card, a page.
//! * [`Chunks`]: a list of `total` rows fetched `size` at a time. Ask which chunks a window of
//!   rows needs ([`request`](Chunks::request)), fetch those, and report each result; a row of a
//!   chunk that is not ready yet is a placeholder ([`slot`](Chunks::slot)). With
//!   [`virtualize`](super::virtualize) this is a table over thousands of rows that loads only
//!   what is scrolled to.
//!
//! A chunk that **failed is not asked for again** until the app retries it, so a failing server
//! is not hammered by every scroll event.

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::ops::Range;

/// Keys being loaded and the keys whose load failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Loads<K: Eq + Hash> {
    loading: HashSet<K>,
    errors: HashMap<K, String>,
}

impl<K: Eq + Hash> Default for Loads<K> {
    fn default() -> Self {
        Self {
            loading: HashSet::new(),
            errors: HashMap::new(),
        }
    }
}

impl<K: Eq + Hash + Clone> Loads<K> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn is_loading(&self, key: &K) -> bool {
        self.loading.contains(key)
    }
    pub fn error(&self, key: &K) -> Option<&str> {
        self.errors.get(key).map(String::as_str)
    }
    /// Marks `key` as loading and clears its error. Returns whether the app should start the
    /// fetch: `false` when it is already loading.
    pub fn begin(&mut self, key: K) -> bool {
        if self.loading.contains(&key) {
            return false;
        }
        self.errors.remove(&key);
        self.loading.insert(key);
        true
    }
    /// The result of a fetch: success clears the key; failure keeps the reason.
    pub fn finish(&mut self, key: &K, result: Result<(), String>) {
        self.loading.remove(key);
        match result {
            Ok(()) => {
                self.errors.remove(key);
            }
            Err(e) => {
                self.errors.insert(key.clone(), e);
            }
        }
    }
    /// Tries a failed `key` again. Returns whether the app should start the fetch (it had
    /// failed and is not already loading).
    pub fn retry(&mut self, key: &K) -> bool {
        self.errors.contains_key(key) && self.begin(key.clone())
    }
    pub fn clear(&mut self) {
        self.loading.clear();
        self.errors.clear();
    }
}

/// What a row of a chunked list is right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot<'a> {
    /// Its chunk has arrived: show the row.
    Ready,
    /// Its chunk is being fetched, or about to be: a placeholder.
    Loading,
    /// Its chunk failed, with the reason: an error row with Retry.
    Failed(&'a str),
}

/// A list of `total` rows fetched `size` at a time.
///
/// ```
/// use iced_adaptive_design::primitives::loading::{Chunks, Slot};
///
/// let mut c = Chunks::new(1000, 50);
/// assert_eq!(c.request(0..120), vec![0, 1, 2]);  // fetch chunks 0, 1, 2
/// assert_eq!(c.request(0..120), Vec::<usize>::new(), "already on their way");
/// c.finish(0, Ok(()));
/// assert_eq!(c.slot(10), Slot::Ready);
/// assert_eq!(c.slot(60), Slot::Loading);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chunks {
    total: usize,
    size: usize,
    ready: HashSet<usize>,
    loads: Loads<usize>,
}

impl Chunks {
    /// `total` rows in chunks of `size` (at least 1).
    pub fn new(total: usize, size: usize) -> Self {
        Self {
            total,
            size: size.max(1),
            ready: HashSet::new(),
            loads: Loads::new(),
        }
    }
    pub fn total(&self) -> usize {
        self.total
    }
    pub fn size(&self) -> usize {
        self.size
    }
    /// The number of rows changed (a filter, a refresh): everything is fetched again.
    pub fn reset(&mut self, total: usize) {
        self.total = total;
        self.ready.clear();
        self.loads.clear();
    }
    pub fn chunk_of(&self, row: usize) -> usize {
        row / self.size
    }
    /// The rows of chunk `chunk`.
    pub fn rows_of(&self, chunk: usize) -> Range<usize> {
        let start = chunk * self.size;
        start.min(self.total)..(start + self.size).min(self.total)
    }
    /// The chunks that `rows` (a [`window`](super::virtualize::window)) needs and nobody has
    /// asked for yet: they are marked loading and returned, so the app fetches each once. Failed
    /// chunks are skipped until [`retry`](Self::retry).
    pub fn request(&mut self, rows: Range<usize>) -> Vec<usize> {
        if rows.is_empty() || self.total == 0 {
            return Vec::new();
        }
        let last = self.chunk_of(rows.end.min(self.total).saturating_sub(1));
        (self.chunk_of(rows.start)..=last)
            .filter(|c| {
                !self.ready.contains(c) && self.loads.error(c).is_none() && self.loads.begin(*c)
            })
            .collect()
    }
    /// The result of fetching `chunk`.
    pub fn finish(&mut self, chunk: usize, result: Result<(), String>) {
        let ok = result.is_ok();
        self.loads.finish(&chunk, result);
        if ok {
            self.ready.insert(chunk);
        }
    }
    /// Asks again for a failed chunk. Returns whether to fetch it.
    pub fn retry(&mut self, chunk: usize) -> bool {
        self.loads.retry(&chunk)
    }
    pub fn is_ready(&self, chunk: usize) -> bool {
        self.ready.contains(&chunk)
    }
    /// What row `row` is: ready, loading (a placeholder) or failed.
    pub fn slot(&self, row: usize) -> Slot<'_> {
        let c = self.chunk_of(row);
        if self.ready.contains(&c) {
            Slot::Ready
        } else if let Some(e) = self.loads.error(&c) {
            Slot::Failed(e)
        } else {
            Slot::Loading
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn begin_starts_once_and_finish_reports() {
        let mut l: Loads<&str> = Loads::new();
        assert!(l.begin("a"));
        assert!(!l.begin("a"), "already loading: do not fetch twice");
        assert!(l.is_loading(&"a"));
        l.finish(&"a", Ok(()));
        assert!(!l.is_loading(&"a") && l.error(&"a").is_none());
    }

    #[test]
    fn a_failure_keeps_its_reason_and_retry_asks_again() {
        let mut l: Loads<u8> = Loads::new();
        l.begin(1);
        l.finish(&1, Err("offline".into()));
        assert_eq!(l.error(&1), Some("offline"));
        assert!(!l.is_loading(&1));
        assert!(l.retry(&1));
        assert!(l.error(&1).is_none() && l.is_loading(&1));
        assert!(!l.retry(&1), "loading again: not twice");
        assert!(!l.retry(&2), "nothing to retry");
    }

    #[test]
    fn a_window_asks_only_for_the_chunks_it_touches() {
        let mut c = Chunks::new(1000, 50);
        assert_eq!(c.request(0..120), vec![0, 1, 2]);
        assert_eq!(
            c.request(100..160),
            vec![3],
            "0..=3 are touched; 2 is on its way"
        );
        assert!(c.request(0..0).is_empty());
    }

    #[test]
    fn rows_are_placeholders_until_their_chunk_arrives() {
        let mut c = Chunks::new(100, 10);
        c.request(0..25);
        assert_eq!(c.slot(5), Slot::Loading);
        c.finish(0, Ok(()));
        assert_eq!(c.slot(5), Slot::Ready);
        assert_eq!(c.slot(15), Slot::Loading);
    }

    #[test]
    fn a_failed_chunk_is_not_requested_again_until_retried() {
        let mut c = Chunks::new(100, 10);
        c.request(0..10);
        c.finish(0, Err("boom".into()));
        assert_eq!(c.slot(3), Slot::Failed("boom"));
        assert!(
            c.request(0..10).is_empty(),
            "scrolling does not hammer a failing server"
        );
        assert!(c.retry(0));
        assert_eq!(c.slot(3), Slot::Loading);
        c.finish(0, Ok(()));
        assert_eq!(c.slot(3), Slot::Ready);
    }

    #[test]
    fn the_last_chunk_is_short_and_requests_stop_at_the_total() {
        let mut c = Chunks::new(25, 10);
        assert_eq!(c.rows_of(2), 20..25);
        assert_eq!(c.rows_of(3), 25..25, "past the end");
        assert_eq!(c.request(20..500), vec![2], "no chunks beyond the total");
    }

    #[test]
    fn reset_forgets_everything() {
        let mut c = Chunks::new(100, 10);
        c.request(0..10);
        c.finish(0, Ok(()));
        c.reset(40);
        assert_eq!(c.total(), 40);
        assert_eq!(c.slot(5), Slot::Loading);
        assert_eq!(c.request(0..10), vec![0]);
    }
}
