//! Infinite pagination: a list that grows page by page as the user scrolls (or presses "Load
//! more"), for offset- or cursor-based sources. Headless, pure and runtime-agnostic like
//! [`lookup`](super::lookup): no widgets, no executor, no clock; the application owns one
//! [`Infinite`] and runs the [`Request`]s it hands out.
//!
//! ```text
//!               request_next()                   complete(id, Ok(page))
//!   Idle ──────────────────────▶ Loading{id} ─────────────────────────────▶ Idle (has more)
//!    ▲  ▲   (only here)             │  │                                  └─▶ Exhausted (no more)
//!    │  └── retry() ◀── Failed ◀────┘  └── complete(id, Err(..))
//!    └─── reset(query): clears the items, invalidates the request in flight
//! ```
//!
//! * **One request at a time**: [`request_next`](Infinite::request_next) hands out a request
//!   only from `Idle`, so a scroll handler that fires on every frame cannot flood the source.
//!   Neither does it ask again after a failure (that would loop on a dead source): the user
//!   presses Retry ([`retry`](Infinite::retry)).
//! * **Stale responses**: [`complete`](Infinite::complete) applies a response only if it answers
//!   the request still in flight. [`reset`](Infinite::reset) (a new query, a new sort order) clears the
//!   list and forgets that request, so its late answer is discarded instead of being appended to
//!   the wrong list. Request ids only increase, across resets.
//! * **Both kinds of source**: a request carries the `offset` (items loaded so far), the opaque
//!   `cursor` from the previous page (None for the first) and the `limit`; use whichever the
//!   backend understands.
//! * **When to load**: [`should_load_more`] is the pure scroll rule (the viewport is within
//!   `threshold_px` of the end of the content); [`on_scroll`](Infinite::on_scroll) combines it
//!   with `request_next`.
//!
//! The async half (running a request as an `iced::Task`) is [`crate::lookup::run_page`].

use super::lookup::{RequestId, Stats};

/// What to fetch next; run it and report back with [`Infinite::complete`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request<Q> {
    pub id: RequestId,
    /// The query the list was [`reset`](Infinite::reset) with.
    pub query: Q,
    /// How many items are loaded so far (the offset of the next page).
    pub offset: usize,
    /// The cursor the previous page returned, if it returned one.
    pub cursor: Option<String>,
    /// The page size asked for.
    pub limit: usize,
}

/// One page of an answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page<T> {
    pub items: Vec<T>,
    /// Where the next page starts, for cursor-based sources.
    pub next_cursor: Option<String>,
    /// Whether anything follows. When `None` it is inferred: a next cursor, or a full page.
    pub has_more: Option<bool>,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            items,
            next_cursor: None,
            has_more: None,
        }
    }
    pub fn cursor(mut self, next: impl Into<String>) -> Self {
        self.next_cursor = Some(next.into());
        self
    }
    pub fn more(mut self, has_more: bool) -> Self {
        self.has_more = Some(has_more);
        self
    }
}

/// Where an [`Infinite`] list is.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Status {
    /// Nothing in flight; more may be loaded.
    #[default]
    Idle,
    /// A request is in flight.
    Loading { id: RequestId },
    /// The last request failed with this message ([`Infinite::retry`] tries again).
    Failed(String),
    /// Everything is loaded.
    Exhausted,
}

/// What [`Infinite::complete`] did with a response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Completion {
    /// The page was appended.
    Appended,
    /// The request failed; the list keeps what it had.
    Failed,
    /// The response was stale and thrown away.
    Discarded,
}

/// A scrollable's position, in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollMetrics {
    /// Distance scrolled from the top.
    pub offset: f32,
    pub viewport_h: f32,
    pub content_h: f32,
}

impl ScrollMetrics {
    /// Pixels of content below the viewport.
    pub fn remaining(&self) -> f32 {
        (self.content_h - self.offset - self.viewport_h).max(0.0)
    }
}

/// Whether the viewport is within `threshold_px` of the end of the content. Content that does
/// not fill the viewport counts (there is nothing to scroll, so nothing else would load more).
pub fn should_load_more(m: ScrollMetrics, threshold_px: f32) -> bool {
    m.remaining() <= threshold_px.max(0.0)
}

/// The items loaded so far and how to get more. See the [module](self).
#[derive(Debug, Clone)]
pub struct Infinite<Q, T> {
    query: Q,
    limit: usize,
    items: Vec<T>,
    cursor: Option<String>,
    status: Status,
    latest: RequestId,
    stats: Stats,
}

impl<Q: Clone, T> Infinite<Q, T> {
    /// A list for `query`, 20 items per request.
    pub fn new(query: Q) -> Self {
        Self {
            query,
            limit: 20,
            items: Vec::new(),
            cursor: None,
            status: Status::Idle,
            latest: 0,
            stats: Stats::default(),
        }
    }
    /// Items per request (default 20; at least 1).
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = n.max(1);
        self
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }
    pub fn len(&self) -> usize {
        self.items.len()
    }
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
    pub fn query(&self) -> &Q {
        &self.query
    }
    pub fn status(&self) -> &Status {
        &self.status
    }
    pub fn stats(&self) -> Stats {
        self.stats
    }
    /// The id of the most recent request handed out (0 before the first).
    pub fn latest_id(&self) -> RequestId {
        self.latest
    }
    pub fn is_loading(&self) -> bool {
        matches!(self.status, Status::Loading { .. })
    }
    pub fn is_failed(&self) -> bool {
        matches!(self.status, Status::Failed(_))
    }
    /// The error of the last request, while it is failed.
    pub fn error(&self) -> Option<&str> {
        match &self.status {
            Status::Failed(e) => Some(e),
            _ => None,
        }
    }
    /// Whether more may exist (not exhausted); true while loading or failed.
    pub fn has_more(&self) -> bool {
        self.status != Status::Exhausted
    }
    /// Loaded everything and it was nothing: "no results".
    pub fn is_empty_result(&self) -> bool {
        self.items.is_empty() && self.status == Status::Exhausted
    }

    fn issue(&mut self) -> Request<Q> {
        self.latest += 1;
        self.status = Status::Loading { id: self.latest };
        Request {
            id: self.latest,
            query: self.query.clone(),
            offset: self.items.len(),
            cursor: self.cursor.clone(),
            limit: self.limit,
        }
    }

    /// The next request, only when idle: not while one is in flight, after a failure (use
    /// [`retry`](Self::retry)) or when exhausted.
    pub fn request_next(&mut self) -> Option<Request<Q>> {
        (self.status == Status::Idle).then(|| self.issue())
    }

    /// After a failure, the same request again.
    pub fn retry(&mut self) -> Option<Request<Q>> {
        self.is_failed().then(|| self.issue())
    }

    /// Whether [`on_scroll`](Self::on_scroll) would ask for a page now (pure).
    pub fn wants_more(&self, m: ScrollMetrics, threshold_px: f32) -> bool {
        self.status == Status::Idle && should_load_more(m, threshold_px)
    }

    /// A scroll position arrived: the next request if the viewport is near the end and the
    /// list can load.
    pub fn on_scroll(&mut self, m: ScrollMetrics, threshold_px: f32) -> Option<Request<Q>> {
        if self.wants_more(m, threshold_px) {
            self.request_next()
        } else {
            None
        }
    }

    /// Applies the answer to request `id`. Only the one in flight counts; anything else (an
    /// answer from before a [`reset`](Self::reset), a duplicate) is discarded and counted.
    pub fn complete<E: ToString>(
        &mut self,
        id: RequestId,
        result: Result<Page<T>, E>,
    ) -> Completion {
        if self.status != (Status::Loading { id }) {
            self.stats.discarded += 1;
            return Completion::Discarded;
        }
        self.stats.applied += 1;
        match result {
            Ok(page) => {
                let full = page.items.len() >= self.limit;
                let inferred = !page.items.is_empty() && (page.next_cursor.is_some() || full);
                let more = page.has_more.unwrap_or(inferred);
                self.items.extend(page.items);
                self.cursor = page.next_cursor;
                self.status = if more {
                    Status::Idle
                } else {
                    Status::Exhausted
                };
                Completion::Appended
            }
            Err(e) => {
                self.status = Status::Failed(e.to_string());
                Completion::Failed
            }
        }
    }

    /// A new query (or sort order): clears the items and the cursor, and forgets the request in
    /// flight so that its answer is discarded. The next [`request_next`](Self::request_next)
    /// loads the first page.
    pub fn reset(&mut self, query: Q) {
        self.query = query;
        self.items.clear();
        self.cursor = None;
        self.status = Status::Idle;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(range: std::ops::Range<u32>) -> Page<u32> {
        Page::new(range.collect())
    }

    fn list() -> Infinite<&'static str, u32> {
        Infinite::new("q").limit(10)
    }

    #[test]
    fn the_first_request_starts_at_zero() {
        let mut l = list();
        let r = l.request_next().unwrap();
        assert_eq!(
            r,
            Request {
                id: 1,
                query: "q",
                offset: 0,
                cursor: None,
                limit: 10
            }
        );
        assert_eq!(l.status(), &Status::Loading { id: 1 });
    }

    #[test]
    fn requests_are_not_duplicated_while_one_is_in_flight() {
        let mut l = list();
        assert!(l.request_next().is_some());
        assert!(l.request_next().is_none());
        assert!(l.on_scroll(near_end(), 100.0).is_none());
        assert_eq!(l.latest_id(), 1);
    }

    fn near_end() -> ScrollMetrics {
        ScrollMetrics {
            offset: 900.0,
            viewport_h: 300.0,
            content_h: 1250.0,
        }
    }

    #[test]
    fn pages_append_in_order_with_the_right_offset() {
        let mut l = list();
        let r = l.request_next().unwrap();
        assert_eq!(
            l.complete::<String>(r.id, Ok(page(0..10))),
            Completion::Appended
        );
        let r = l.request_next().unwrap();
        assert_eq!((r.offset, r.id), (10, 2));
        l.complete::<String>(r.id, Ok(page(10..20)));
        assert_eq!(l.items(), (0..20).collect::<Vec<_>>());
        assert_eq!(l.status(), &Status::Idle);
        assert!(l.has_more());
    }

    #[test]
    fn cursors_are_passed_on() {
        let mut l = list();
        let r = l.request_next().unwrap();
        l.complete::<String>(r.id, Ok(page(0..10).cursor("abc")));
        let r = l.request_next().unwrap();
        assert_eq!(r.cursor.as_deref(), Some("abc"));
    }

    #[test]
    fn a_short_page_exhausts_the_list() {
        let mut l = list();
        let r = l.request_next().unwrap();
        l.complete::<String>(r.id, Ok(page(0..10)));
        let r = l.request_next().unwrap();
        l.complete::<String>(r.id, Ok(page(10..14)));
        assert_eq!(l.status(), &Status::Exhausted);
        assert!(!l.has_more());
        assert!(l.request_next().is_none());
        assert!(l.on_scroll(near_end(), 100.0).is_none());
        assert_eq!(l.len(), 14);
    }

    #[test]
    fn has_more_overrides_the_guess() {
        let mut l = list();
        let r = l.request_next().unwrap();
        // a short page, but the source says there is more
        l.complete::<String>(r.id, Ok(page(0..3).more(true)));
        assert_eq!(l.status(), &Status::Idle);
        let r = l.request_next().unwrap();
        // a full page, but the source says it is the last
        l.complete::<String>(r.id, Ok(page(3..13).more(false)));
        assert_eq!(l.status(), &Status::Exhausted);
    }

    #[test]
    fn an_empty_first_page_is_an_empty_result() {
        let mut l = list();
        let r = l.request_next().unwrap();
        l.complete::<String>(r.id, Ok(Page::new(Vec::new())));
        assert!(l.is_empty_result());
        assert!(l.request_next().is_none());
    }

    #[test]
    fn an_empty_page_ends_the_list_even_with_a_cursor() {
        let mut l = list();
        let r = l.request_next().unwrap();
        l.complete::<String>(r.id, Ok(Page::new(Vec::new()).cursor("x")));
        assert_eq!(l.status(), &Status::Exhausted);
    }

    #[test]
    fn failing_keeps_the_items_and_waits_for_retry() {
        let mut l = list();
        let r = l.request_next().unwrap();
        l.complete::<String>(r.id, Ok(page(0..10)));
        let r = l.request_next().unwrap();
        assert_eq!(
            l.complete(r.id, Err::<Page<u32>, _>("timeout")),
            Completion::Failed
        );
        assert_eq!(l.error(), Some("timeout"));
        assert_eq!(l.len(), 10);
        assert!(l.request_next().is_none(), "no automatic retry loop");
        assert!(l.on_scroll(near_end(), 100.0).is_none());
        let again = l.retry().unwrap();
        assert_eq!((again.offset, again.id), (10, 3));
        l.complete::<String>(again.id, Ok(page(10..20)));
        assert_eq!(l.len(), 20);
        assert!(l.retry().is_none(), "nothing to retry");
    }

    #[test]
    fn a_stale_response_after_reset_is_discarded() {
        let mut l = list();
        let old = l.request_next().unwrap();
        l.reset("other");
        // not loading any more, so the old request cannot be completed either
        assert_eq!(
            l.complete::<String>(old.id, Ok(page(0..10))),
            Completion::Discarded
        );
        assert!(l.is_empty());
        let new = l.request_next().unwrap();
        assert!(new.id > old.id);
        assert_eq!(new.query, "other");
        // the old answer arrives again while the new one is in flight
        assert_eq!(
            l.complete::<String>(old.id, Ok(page(0..10))),
            Completion::Discarded
        );
        assert!(l.is_loading());
        assert_eq!(
            l.complete::<String>(new.id, Ok(page(100..110))),
            Completion::Appended
        );
        assert_eq!(l.items()[0], 100);
        assert_eq!(
            l.stats(),
            Stats {
                applied: 1,
                discarded: 2
            }
        );
    }

    #[test]
    fn a_duplicate_answer_is_discarded() {
        let mut l = list();
        let r = l.request_next().unwrap();
        l.complete::<String>(r.id, Ok(page(0..10)));
        assert_eq!(
            l.complete::<String>(r.id, Ok(page(0..10))),
            Completion::Discarded
        );
        assert_eq!(l.len(), 10);
    }

    #[test]
    fn reset_revives_an_exhausted_or_failed_list() {
        let mut l = list();
        let r = l.request_next().unwrap();
        l.complete(r.id, Err::<Page<u32>, _>("x"));
        l.reset("y");
        assert_eq!(l.status(), &Status::Idle);
        assert!(l.request_next().is_some());
        let mut e = list();
        let r = e.request_next().unwrap();
        e.complete::<String>(r.id, Ok(page(0..2)));
        assert!(!e.has_more());
        e.reset("z");
        assert!(e.has_more());
        assert_eq!(e.request_next().unwrap().offset, 0);
    }

    #[test]
    fn threshold_math() {
        let m = |offset| ScrollMetrics {
            offset,
            viewport_h: 400.0,
            content_h: 1000.0,
        };
        assert_eq!(m(0.0).remaining(), 600.0);
        assert!(!should_load_more(m(0.0), 200.0));
        assert!(!should_load_more(m(399.0), 200.0), "201 px left");
        assert!(should_load_more(m(400.0), 200.0), "exactly the threshold");
        assert!(should_load_more(m(600.0), 200.0));
        assert!(should_load_more(m(700.0), 0.0), "overscroll");
        assert!(should_load_more(m(600.0), 0.0), "at the very end");
        assert!(!should_load_more(m(599.0), 0.0));
        // content shorter than the viewport: nothing to scroll, so load
        let short = ScrollMetrics {
            offset: 0.0,
            viewport_h: 400.0,
            content_h: 120.0,
        };
        assert!(should_load_more(short, 0.0));
        // a negative threshold behaves as 0
        assert!(!should_load_more(m(599.0), -50.0));
    }

    #[test]
    fn on_scroll_requests_once_near_the_end() {
        let mut l = list();
        let far = ScrollMetrics {
            offset: 0.0,
            viewport_h: 300.0,
            content_h: 1250.0,
        };
        assert!(l.on_scroll(far, 100.0).is_none());
        assert!(l.on_scroll(near_end(), 100.0).is_some());
        assert!(l.on_scroll(near_end(), 100.0).is_none());
    }
}
