//! Async lookup: a state machine for "query -> results that arrive later".
//!
//! Headless, pure and runtime-agnostic: no widgets, no executor, no clock of its own. Every
//! operation that depends on time takes an explicit `now`, and the application owns the state.
//! Useful anywhere a query is answered asynchronously: an autocomplete, a searchable select, a
//! command palette, a remote table's filter.
//!
//! ```text
//!            input(q)                 poll(now >= deadline)            complete(id, ..)
//!  Idle ───────────────▶ Waiting ───────────────────────────▶ Loading{id} ──────────────▶ Loaded
//!   ▲   (>= min_chars)  (debounce)    returns Request{id, q}      │   (latest id only)    / Failed
//!   └── input(short) / clear / cancel ◀───────────────────────────┘
//! ```
//!
//! * **Debounce**: each [`input`](Lookup::input) restarts the wait, so rapid typing makes one
//!   request. [`deadline`](Lookup::deadline) tells a subscription when to
//!   [`poll`](Lookup::poll).
//! * **Stale responses**: a response is applied only if it answers the request that is still
//!   current. A later keystroke, [`cancel`](Lookup::cancel) or [`clear`](Lookup::clear)
//!   invalidates the one in flight; its answer is discarded and never overwrites newer results.
//! * **Cache**: optionally remembers the last N answered queries; a hit completes at
//!   [`poll`](Lookup::poll) without a request.
//!
//! The async half (running the request, ticking) is [`crate::lookup`]. [`Highlight`] is the
//! keyboard cursor over the results, shared by list popups.

use super::validation::Validity;
use std::time::{Duration, Instant};

/// Identifies one request; ids only increase.
pub type RequestId = u64;

/// What a query type must offer: its length (for `min_chars`) and a canonical form.
pub trait Query: Clone + PartialEq {
    /// Characters in the query, compared with `min_chars`.
    fn chars(&self) -> usize;
    /// The canonical form (`String`: trimmed). Queries that normalize equal are the same query.
    fn normalize(self) -> Self {
        self
    }
}

impl Query for String {
    fn chars(&self) -> usize {
        str::chars(self).count()
    }
    fn normalize(self) -> Self {
        let t = self.trim();
        if t.len() == self.len() {
            self
        } else {
            t.to_string()
        }
    }
}

/// Where a [`Lookup`] is.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Status {
    /// Nothing to look up (no query, or shorter than `min_chars`).
    #[default]
    Idle,
    /// Debouncing: the request is issued by [`Lookup::poll`] once this instant has passed.
    Waiting(Instant),
    /// A request is in flight.
    Loading { id: RequestId },
    /// The latest query has its results.
    Loaded,
    /// The latest query failed with this message ([`Lookup::retry`] tries again).
    Failed(String),
}

/// A request for the application to run, from [`Lookup::poll`] or [`Lookup::retry`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request<Q> {
    pub id: RequestId,
    pub query: Q,
}

/// How many responses were applied and how many were thrown away as stale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stats {
    pub applied: u32,
    pub discarded: u32,
}

/// The state of a query and its asynchronous results. Controlled: the application keeps it.
#[derive(Debug, Clone)]
pub struct Lookup<Q, T> {
    debounce: Duration,
    min_chars: usize,
    limit: Option<usize>,
    cache_size: usize,
    normalize: Option<fn(Q) -> Q>,

    query: Option<Q>,
    status: Status,
    results: Vec<T>,
    error: Option<String>,
    /// The last id handed out; a response must carry the id in `Status::Loading`.
    latest: RequestId,
    version: u64,
    stats: Stats,
    cache: Vec<(Q, Vec<T>)>,
}

impl<Q: Query, T: Clone> Default for Lookup<Q, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Q: Query, T: Clone> Lookup<Q, T> {
    /// 200 ms debounce, one character minimum, no limit, no cache.
    pub fn new() -> Self {
        Self {
            debounce: Duration::from_millis(200),
            min_chars: 1,
            limit: None,
            cache_size: 0,
            normalize: None,
            query: None,
            status: Status::Idle,
            results: Vec::new(),
            error: None,
            latest: 0,
            version: 0,
            stats: Stats::default(),
            cache: Vec::new(),
        }
    }
    /// How long typing must pause before a request is issued (default 200 ms).
    pub fn debounce(mut self, d: Duration) -> Self {
        self.debounce = d;
        self
    }
    /// Queries shorter than this are not looked up (default 1; 0 looks up the empty query).
    pub fn min_chars(mut self, n: usize) -> Self {
        self.min_chars = n;
        self
    }
    /// Keep at most this many results.
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }
    /// Remember the answers of the last `n` queries (0: off, the default).
    pub fn cache(mut self, n: usize) -> Self {
        self.cache_size = n;
        self
    }
    /// Replaces the query type's own canonical form (`String`: trimmed).
    pub fn normalize_with(mut self, f: fn(Q) -> Q) -> Self {
        self.normalize = Some(f);
        self
    }

    // ------------------------------------------------------------------ reading

    pub fn status(&self) -> &Status {
        &self.status
    }
    /// The normalized query last accepted.
    pub fn query(&self) -> Option<&Q> {
        self.query.as_ref()
    }
    /// The results of the last answered query (kept while the next one is pending).
    pub fn results(&self) -> &[T] {
        &self.results
    }
    /// The message of the last failure, while [`Status::Failed`].
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
    /// The id of the request in flight, if any.
    pub fn loading_id(&self) -> Option<RequestId> {
        match self.status {
            Status::Loading { id } => Some(id),
            _ => None,
        }
    }
    /// The last id handed out (0: none yet).
    pub fn latest_id(&self) -> RequestId {
        self.latest
    }
    pub fn is_loading(&self) -> bool {
        matches!(self.status, Status::Loading { .. })
    }
    /// Debouncing or loading: results are on their way.
    pub fn is_pending(&self) -> bool {
        matches!(self.status, Status::Waiting(_) | Status::Loading { .. })
    }
    /// When [`poll`](Self::poll) is next due (`Waiting` only): the tick subscription's cue.
    pub fn deadline(&self) -> Option<Instant> {
        match self.status {
            Status::Waiting(t) => Some(t),
            _ => None,
        }
    }
    /// Changes whenever the results are replaced or cleared (reset a highlight on it).
    pub fn version(&self) -> u64 {
        self.version
    }
    /// Responses applied and discarded so far.
    pub fn stats(&self) -> Stats {
        self.stats
    }

    // --------------------------------------------------------------- operations

    fn set_results(&mut self, mut v: Vec<T>) {
        if let Some(n) = self.limit {
            v.truncate(n);
        }
        self.results = v;
        self.version += 1;
    }

    fn reset_idle(&mut self) {
        self.status = Status::Idle;
        self.error = None;
        self.query = None;
        if !self.results.is_empty() {
            self.results.clear();
            self.version += 1;
        }
    }

    /// The query changed. Normalizes it; shorter than `min_chars` goes [`Idle`](Status::Idle),
    /// clears the results and drops what is in flight (its answer will be discarded);
    /// otherwise (re)starts the debounce at `now`. The same query again changes nothing, so a
    /// trailing space is not a new query.
    pub fn input(&mut self, query: Q, now: Instant) {
        let query = match self.normalize {
            Some(f) => f(query.normalize()),
            None => query.normalize(),
        };
        if query.chars() < self.min_chars {
            self.reset_idle();
            return;
        }
        let unchanged = self.query.as_ref() == Some(&query)
            && matches!(
                self.status,
                Status::Waiting(_) | Status::Loading { .. } | Status::Loaded
            );
        if unchanged {
            return;
        }
        self.query = Some(query);
        self.error = None;
        self.status = Status::Waiting(now + self.debounce);
    }

    /// Issues the request once the debounce has elapsed at `now`: the status becomes
    /// [`Loading`](Status::Loading) and the application runs the returned request. A cached
    /// answer completes at once and returns `None`.
    pub fn poll(&mut self, now: Instant) -> Option<Request<Q>> {
        let Status::Waiting(deadline) = self.status else {
            return None;
        };
        if now < deadline {
            return None;
        }
        let query = self.query.clone()?;
        if let Some(hit) = self.cache.iter().position(|(q, _)| *q == query) {
            let entry = self.cache.remove(hit);
            self.set_results(entry.1.clone());
            self.cache.push(entry);
            self.status = Status::Loaded;
            self.stats.applied += 1;
            return None;
        }
        Some(self.issue(query))
    }

    fn issue(&mut self, query: Q) -> Request<Q> {
        self.latest += 1;
        self.status = Status::Loading { id: self.latest };
        Request {
            id: self.latest,
            query,
        }
    }

    /// The answer to request `id`. Applied only if that request is still the one in flight:
    /// a response for an older query (the user typed on, or it was cancelled) is discarded and
    /// never overwrites newer results. Returns whether it was applied.
    pub fn complete(&mut self, id: RequestId, result: Result<Vec<T>, String>) -> bool {
        if self.loading_id() != Some(id) {
            self.stats.discarded += 1;
            return false;
        }
        self.stats.applied += 1;
        match result {
            Ok(v) => {
                self.set_results(v);
                self.error = None;
                self.status = Status::Loaded;
                if self.cache_size > 0
                    && let Some(q) = self.query.clone()
                {
                    self.cache.retain(|(c, _)| *c != q);
                    self.cache.push((q, self.results.clone()));
                    if self.cache.len() > self.cache_size {
                        self.cache.remove(0);
                    }
                }
            }
            Err(msg) => {
                // The previous results stay; the failure is what to show.
                self.error = Some(msg.clone());
                self.status = Status::Failed(msg);
            }
        }
        true
    }

    /// Stops what is pending (debouncing or loading) and goes back to
    /// [`Idle`](Status::Idle); the results stay as they were. An in-flight answer is discarded.
    pub fn cancel(&mut self) {
        if self.is_pending() {
            self.status = Status::Idle;
        }
    }

    /// Tries the failed (or the loaded) query again, now, skipping the debounce and the cache.
    pub fn retry(&mut self) -> Option<Request<Q>> {
        if !matches!(self.status, Status::Failed(_) | Status::Loaded) {
            return None;
        }
        let query = self.query.clone()?;
        self.error = None;
        Some(self.issue(query))
    }

    /// Forgets the query and the results, and drops what is in flight. The cache stays.
    pub fn clear(&mut self) {
        self.reset_idle();
    }
}

/// The keyboard cursor over a list of results: Up / Down with wrap-around, and the window of
/// at most `max` rows to show so the highlighted row stays visible. Reset it when the results
/// change ([`Lookup::version`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Highlight {
    index: Option<usize>,
    start: usize,
}

impl Highlight {
    pub fn index(&self) -> Option<usize> {
        self.index
    }
    pub fn set(&mut self, index: Option<usize>) {
        self.index = index;
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    /// Down: the first row from nothing, the next one, wrapping after the last.
    pub fn next(&mut self, len: usize) {
        self.index = match (len, self.index) {
            (0, _) => None,
            (_, None) => Some(0),
            (n, Some(i)) => Some((i + 1) % n),
        };
    }
    /// Up: the last row from nothing, the previous one, wrapping before the first.
    pub fn prev(&mut self, len: usize) {
        self.index = match (len, self.index) {
            (0, _) => None,
            (n, None | Some(0)) => Some(n - 1),
            (_, Some(i)) => Some(i - 1),
        };
    }
    /// The rows to show: at most `max` of `len`, scrolled just enough to include the
    /// highlighted one.
    pub fn window(&mut self, len: usize, max: usize) -> std::ops::Range<usize> {
        let max = max.max(1);
        if let Some(i) = self.index.filter(|i| *i < len) {
            if i < self.start {
                self.start = i;
            } else if i >= self.start + max {
                self.start = i + 1 - max;
            }
        }
        self.start = self.start.min(len.saturating_sub(max));
        self.start..(self.start + max).min(len)
    }
}

/// Validity of an autocomplete's text. With `free_text` anything goes. Otherwise a non-empty
/// text must be the label of the suggestion the user picked (`picked`), else `msg` is an error.
/// Empty text is valid here; combine with [`required`](super::validation::required) when a
/// value is mandatory.
pub fn validate_pick(text: &str, picked: Option<&str>, free_text: bool, msg: &str) -> Validity {
    if free_text || text.trim().is_empty() || picked == Some(text) {
        Validity::Ok
    } else {
        Validity::error(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type L = Lookup<String, &'static str>;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }
    fn s(x: &str) -> String {
        x.to_string()
    }

    #[test]
    fn debounce_coalesces_rapid_typing_into_one_request() {
        let t0 = Instant::now();
        let mut l = L::new();
        for (i, q) in ["a", "ab", "abc"].into_iter().enumerate() {
            l.input(s(q), t0 + ms(50 * i as u64));
            // Polling between keystrokes never fires early.
            assert_eq!(l.poll(t0 + ms(50 * i as u64 + 10)), None);
        }
        // 200 ms after the LAST keystroke (at 100 ms).
        assert_eq!(l.deadline(), Some(t0 + ms(300)));
        assert_eq!(l.poll(t0 + ms(299)), None);
        let r = l.poll(t0 + ms(300)).unwrap();
        assert_eq!((r.id, r.query.as_str()), (1, "abc"));
        assert!(l.is_loading());
        assert_eq!(l.deadline(), None);
        assert_eq!(l.poll(t0 + ms(400)), None, "one request only");
    }

    #[test]
    fn a_response_for_an_older_query_is_ignored() {
        let t0 = Instant::now();
        let mut l = L::new();
        l.input(s("a"), t0);
        let first = l.poll(t0 + ms(200)).unwrap();
        l.input(s("ab"), t0 + ms(210));
        let second = l.poll(t0 + ms(410)).unwrap();
        assert!(second.id > first.id);
        // The answer to "ab" comes first, then the slow answer to "a" must not replace it.
        assert!(l.complete(second.id, Ok(vec!["abc"])));
        assert!(!l.complete(first.id, Ok(vec!["apple", "avocado"])));
        assert_eq!(l.results(), ["abc"]);
        assert_eq!(l.status(), &Status::Loaded);
        assert_eq!(
            l.stats(),
            Stats {
                applied: 1,
                discarded: 1
            }
        );
    }

    #[test]
    fn out_of_order_completion_in_either_order() {
        let t0 = Instant::now();
        let mut l = L::new();
        l.input(s("a"), t0);
        let first = l.poll(t0 + ms(200)).unwrap();
        l.input(s("ab"), t0 + ms(210));
        // The old answer lands while the new query is still debouncing: discarded.
        assert!(!l.complete(first.id, Ok(vec!["old"])));
        assert!(l.results().is_empty());
        let second = l.poll(t0 + ms(410)).unwrap();
        assert!(l.complete(second.id, Ok(vec!["new"])));
        // And a duplicate or late answer for the finished request is discarded too.
        assert!(!l.complete(second.id, Ok(vec!["dup"])));
        assert_eq!(l.results(), ["new"]);
    }

    #[test]
    fn error_then_retry() {
        let t0 = Instant::now();
        let mut l = L::new();
        l.input(s("fail"), t0);
        let r = l.poll(t0 + ms(200)).unwrap();
        assert!(l.complete(r.id, Err(s("timeout"))));
        assert_eq!(l.status(), &Status::Failed(s("timeout")));
        assert_eq!(l.error(), Some("timeout"));
        let again = l.retry().unwrap();
        assert_eq!(again.query, "fail");
        assert!(again.id > r.id);
        assert!(l.is_loading());
        assert_eq!(l.error(), None);
        assert!(l.complete(again.id, Ok(vec!["ok"])));
        assert_eq!(l.results(), ["ok"]);
        assert_eq!(l.retry().map(|r| r.query), Some(s("fail")), "refresh");
    }

    #[test]
    fn retry_only_applies_to_a_settled_query() {
        let mut l = L::new();
        assert_eq!(l.retry(), None);
        l.input(s("a"), Instant::now());
        assert_eq!(l.retry(), None, "still waiting");
    }

    #[test]
    fn min_chars_gates_the_lookup() {
        let t0 = Instant::now();
        let mut l = L::new().min_chars(3);
        l.input(s("ab"), t0);
        assert_eq!(l.status(), &Status::Idle);
        assert_eq!(l.deadline(), None);
        assert_eq!(l.poll(t0 + ms(500)), None);
        l.input(s("abc"), t0);
        assert!(matches!(l.status(), Status::Waiting(_)));
        // Spaces do not count.
        l.input(s("  ab "), t0);
        assert_eq!(l.status(), &Status::Idle);
    }

    #[test]
    fn shortening_below_min_chars_clears_results_and_drops_the_request() {
        let t0 = Instant::now();
        let mut l = L::new().min_chars(2);
        l.input(s("ab"), t0);
        let r = l.poll(t0 + ms(200)).unwrap();
        l.complete(r.id, Ok(vec!["x", "y"]));
        let v = l.version();
        assert_eq!(l.results().len(), 2);
        l.input(s("a"), t0 + ms(300));
        assert!(l.results().is_empty());
        assert_ne!(l.version(), v);
        assert_eq!(l.status(), &Status::Idle);
        assert_eq!(l.query(), None);
        // An answer still in flight at that moment is discarded.
        l.input(s("ab"), t0 + ms(400));
        let r2 = l.poll(t0 + ms(600)).unwrap();
        l.input(s(""), t0 + ms(610));
        assert!(!l.complete(r2.id, Ok(vec!["late"])));
        assert!(l.results().is_empty());
    }

    #[test]
    fn empty_query_is_looked_up_with_min_chars_zero() {
        let t0 = Instant::now();
        let mut l = L::new().min_chars(0);
        l.input(s(""), t0);
        assert!(matches!(l.status(), Status::Waiting(_)));
        assert_eq!(l.poll(t0 + ms(200)).unwrap().query, "");
    }

    #[test]
    fn trimming_makes_the_same_query_a_no_op() {
        let t0 = Instant::now();
        let mut l = L::new();
        l.input(s("rust"), t0);
        let deadline = l.deadline();
        l.input(s("rust "), t0 + ms(100));
        assert_eq!(l.deadline(), deadline, "the debounce did not restart");
        let r = l.poll(t0 + ms(200)).unwrap();
        l.input(s(" rust"), t0 + ms(210));
        assert!(l.is_loading(), "an in-flight query is not restarted");
        l.complete(r.id, Ok(vec!["rust"]));
        l.input(s("rust  "), t0 + ms(300));
        assert_eq!(l.status(), &Status::Loaded);
    }

    #[test]
    fn custom_normalization() {
        let t0 = Instant::now();
        let mut l = L::new().normalize_with(|q| q.to_lowercase());
        l.input(s("  RuSt "), t0);
        assert_eq!(l.query().map(String::as_str), Some("rust"));
    }

    #[test]
    fn limit_truncates() {
        let t0 = Instant::now();
        let mut l = L::new().limit(2);
        l.input(s("a"), t0);
        let r = l.poll(t0 + ms(200)).unwrap();
        l.complete(r.id, Ok(vec!["1", "2", "3"]));
        assert_eq!(l.results(), ["1", "2"]);
    }

    #[test]
    fn a_cache_hit_completes_without_a_request() {
        let t0 = Instant::now();
        let mut l = L::new().cache(2);
        l.input(s("a"), t0);
        let r = l.poll(t0 + ms(200)).unwrap();
        l.complete(r.id, Ok(vec!["apple"]));
        l.input(s("b"), t0 + ms(300));
        let r = l.poll(t0 + ms(500)).unwrap();
        l.complete(r.id, Ok(vec!["banana"]));
        // Back to "a": served from the cache at the poll, no new request, id unchanged.
        l.input(s("a"), t0 + ms(600));
        let latest = l.latest_id();
        assert_eq!(l.poll(t0 + ms(800)), None);
        assert_eq!(l.status(), &Status::Loaded);
        assert_eq!(l.results(), ["apple"]);
        assert_eq!(l.latest_id(), latest);
    }

    #[test]
    fn the_cache_keeps_only_the_last_n_queries() {
        let t0 = Instant::now();
        let mut l = L::new().cache(2);
        for (i, q) in ["a", "b", "c"].into_iter().enumerate() {
            let at = t0 + ms(1000 * i as u64);
            l.input(s(q), at);
            let r = l.poll(at + ms(200)).unwrap();
            l.complete(r.id, Ok(vec!["r"]));
        }
        // "a" was evicted: asking again needs a request; "c" is still cached.
        l.input(s("a"), t0 + ms(5000));
        assert!(l.poll(t0 + ms(5200)).is_some());
        l.input(s("c"), t0 + ms(6000));
        assert!(l.poll(t0 + ms(6200)).is_none());
    }

    #[test]
    fn cancel_stops_pending_work_and_discards_the_answer() {
        let t0 = Instant::now();
        let mut l = L::new();
        l.input(s("a"), t0);
        l.cancel();
        assert_eq!(l.status(), &Status::Idle);
        assert_eq!(l.poll(t0 + ms(500)), None);
        l.input(s("ab"), t0 + ms(600));
        let r = l.poll(t0 + ms(800)).unwrap();
        l.cancel();
        assert!(!l.complete(r.id, Ok(vec!["late"])));
        assert!(l.results().is_empty());
        assert_eq!(l.stats().discarded, 1);
    }

    #[test]
    fn clear_forgets_everything_but_the_cache() {
        let t0 = Instant::now();
        let mut l = L::new().cache(1);
        l.input(s("a"), t0);
        let r = l.poll(t0 + ms(200)).unwrap();
        l.complete(r.id, Ok(vec!["apple"]));
        l.clear();
        assert_eq!((l.status(), l.query(), l.results().len()), (&Status::Idle, None, 0));
        l.input(s("a"), t0 + ms(300));
        assert_eq!(l.poll(t0 + ms(500)), None);
        assert_eq!(l.results(), ["apple"]);
    }

    #[test]
    fn typing_again_after_a_failure_starts_over() {
        let t0 = Instant::now();
        let mut l = L::new();
        l.input(s("a"), t0);
        let r = l.poll(t0 + ms(200)).unwrap();
        l.complete(r.id, Err(s("boom")));
        l.input(s("a"), t0 + ms(300));
        assert!(matches!(l.status(), Status::Waiting(_)));
        assert_eq!(l.error(), None);
    }

    #[test]
    fn highlight_wraps_and_windows() {
        let mut h = Highlight::default();
        h.next(3);
        assert_eq!(h.index(), Some(0));
        h.prev(3);
        assert_eq!(h.index(), Some(2));
        h.next(3);
        assert_eq!(h.index(), Some(0));
        h.next(0);
        assert_eq!(h.index(), None);

        let mut h = Highlight::default();
        assert_eq!(h.window(10, 4), 0..4);
        h.set(Some(6));
        assert_eq!(h.window(10, 4), 3..7);
        h.set(Some(1));
        assert_eq!(h.window(10, 4), 1..5);
        h.set(Some(9));
        assert_eq!(h.window(10, 4), 6..10);
        assert_eq!(h.window(2, 4), 0..2);
    }

    #[test]
    fn picking_from_the_list_validates_free_text_does_not() {
        let msg = "Choose one";
        assert!(validate_pick("Par", None, true, msg).is_ok());
        assert!(validate_pick("", None, false, msg).is_ok());
        assert!(validate_pick("Paris", Some("Paris"), false, msg).is_ok());
        assert!(validate_pick("Par", Some("Paris"), false, msg).blocks_submit());
        assert!(validate_pick("Par", None, false, msg).blocks_submit());
    }
}
