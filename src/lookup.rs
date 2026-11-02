//! The async half of [`primitives::lookup`](crate::primitives::lookup): running a request
//! through iced and ticking while a debounce is pending.
//!
//! Kept apart so the state machine stays runtime-agnostic. This module only needs iced's
//! [`Task`] and [`Subscription`]; the future that answers a query is yours (sqlx, reqwest,
//! anything), and it runs on whatever executor your iced application uses.
//!
//! ```ignore
//! use iced_libadwaita::lookup;
//! use iced_libadwaita::primitives::{Lookup, RequestId};
//!
//! struct App { text: String, cities: Lookup<String, City> }
//!
//! enum Msg {
//!     Input(String),
//!     Tick(Instant),
//!     Found(RequestId, Result<Vec<City>, String>),
//!     Retry,
//! }
//!
//! fn update(&mut self, msg: Msg) -> Task<Msg> {
//!     match msg {
//!         // input: the state restarts its debounce; nothing is requested yet
//!         Msg::Input(t) => { self.cities.input(t.clone(), Instant::now()); self.text = t; Task::none() }
//!         // tick (only runs while a debounce is pending): due? run the request
//!         Msg::Tick(now) => self.cities.run(now, db::search, Msg::Found),
//!         // result: only the latest request's answer is applied, older ones are dropped
//!         Msg::Found(id, r) => { self.cities.complete(id, r); Task::none() }
//!         Msg::Retry => match self.cities.retry() {
//!             Some(req) => lookup::run(req, db::search, Msg::Found),
//!             None => Task::none(),
//!         },
//!     }
//! }
//!
//! fn subscription(&self) -> Subscription<Msg> { self.cities.subscription(Msg::Tick) }
//! ```
//!
//! `db::search` is any `Fn(Q) -> impl Future<Output = Result<Vec<T>, E>>` with `E: ToString`.
//!
//! The same file also runs the pages of an infinite list: [`run_page`] and `Infinite::load_next`
//! (see [`primitives::infinite`](crate::primitives::infinite)).

use crate::primitives::lookup::{Lookup, Query, Request, RequestId};
use iced::advanced::graphics::futures::MaybeSend;
use iced::{Subscription, Task};
use std::future::Future;
use std::time::{Duration, Instant};

/// How often the subscription ticks while a debounce is pending.
pub const TICK: Duration = Duration::from_millis(25);

/// Runs `req` with `source` and maps the answer, together with the request's id, to a message.
/// Feed that message to [`Lookup::complete`]. An error is reported as its `to_string()`.
pub fn run<Q, T, E, Fut, M>(
    req: Request<Q>,
    source: impl Fn(Q) -> Fut,
    to_msg: impl Fn(RequestId, Result<Vec<T>, String>) -> M + MaybeSend + 'static,
) -> Task<M>
where
    Fut: Future<Output = Result<Vec<T>, E>> + MaybeSend + 'static,
    E: ToString + 'static,
    T: MaybeSend + 'static,
    M: MaybeSend + 'static,
{
    let id = req.id;
    let fut = source(req.query);
    Task::perform(
        async move { fut.await.map_err(|e| e.to_string()) },
        move |r| to_msg(id, r),
    )
}

impl<Q: Query, T: Clone> Lookup<Q, T> {
    /// [`poll`](Lookup::poll)s at `now` and, if a request is due, [`run`]s it. Call it from the
    /// tick message of [`subscription`](Lookup::subscription).
    pub fn run<E, Fut, M>(
        &mut self,
        now: Instant,
        source: impl Fn(Q) -> Fut,
        to_msg: impl Fn(RequestId, Result<Vec<T>, String>) -> M + MaybeSend + 'static,
    ) -> Task<M>
    where
        Fut: Future<Output = Result<Vec<T>, E>> + MaybeSend + 'static,
        E: ToString + 'static,
        T: MaybeSend + 'static,
        M: MaybeSend + 'static,
    {
        match self.poll(now) {
            Some(req) => run(req, source, to_msg),
            None => Task::none(),
        }
    }

    /// Ticks every [`TICK`] while a debounce is pending, nothing otherwise. `tick` must not
    /// capture anything (a message constructor such as `Msg::Tick` is right).
    pub fn subscription<M: 'static>(
        &self,
        tick: impl Fn(Instant) -> M + MaybeSend + Clone + 'static,
    ) -> Subscription<M> {
        if self.deadline().is_some() {
            iced::time::every(TICK).map(tick)
        } else {
            Subscription::none()
        }
    }
}

// -- infinite pagination

use crate::primitives::infinite::{Infinite, Page};

/// Runs one page request of an [`Infinite`] list: `source` receives the whole [`Request`]
/// (query, `offset`, `cursor`, `limit`) and answers a [`Page`]. The answer is mapped, together
/// with the request id, to a message for [`Infinite::complete`]; an error is reported as its
/// `to_string()`. A stale answer is harmless: `complete` discards it.
///
/// ```ignore
/// use iced_libadwaita::{lookup, primitives::infinite::{Infinite, Page}};
///
/// enum Msg { Scrolled(ScrollMetrics), More, Loaded(RequestId, Result<Page<Row>, String>), Retry, Sort(Key) }
///
/// fn update(&mut self, msg: Msg) -> Task<Msg> {
///     match msg {
///         Msg::Scrolled(m) => self.rows.load_on_scroll(m, 200.0, db::page, Msg::Loaded),
///         Msg::More => self.rows.load_next(db::page, Msg::Loaded),
///         Msg::Loaded(id, r) => { self.rows.complete(id, r); Task::none() }
///         Msg::Retry => self.rows.load_retry(db::page, Msg::Loaded),
///         // a new sort order: clear, then ask for the first page of the new list
///         Msg::Sort(k) => { self.rows.reset(k); self.rows.load_next(db::page, Msg::Loaded) }
///     }
/// }
/// ```
pub fn run_page<Q, T, E, Fut, M>(
    req: crate::primitives::infinite::Request<Q>,
    source: impl Fn(crate::primitives::infinite::Request<Q>) -> Fut,
    to_msg: impl Fn(RequestId, Result<Page<T>, String>) -> M + MaybeSend + 'static,
) -> Task<M>
where
    Fut: Future<Output = Result<Page<T>, E>> + MaybeSend + 'static,
    E: ToString + 'static,
    T: MaybeSend + 'static,
    M: MaybeSend + 'static,
{
    let id = req.id;
    let fut = source(req);
    Task::perform(
        async move { fut.await.map_err(|e| e.to_string()) },
        move |r| to_msg(id, r),
    )
}

impl<Q: Clone, T> Infinite<Q, T> {
    /// [`request_next`](Infinite::request_next), run with [`run_page`]. Nothing happens (a
    /// `Task::none()`) while loading, failed or exhausted.
    pub fn load_next<E, Fut, M>(
        &mut self,
        source: impl Fn(crate::primitives::infinite::Request<Q>) -> Fut,
        to_msg: impl Fn(RequestId, Result<Page<T>, String>) -> M + MaybeSend + 'static,
    ) -> Task<M>
    where
        Fut: Future<Output = Result<Page<T>, E>> + MaybeSend + 'static,
        E: ToString + 'static,
        T: MaybeSend + 'static,
        M: MaybeSend + 'static,
    {
        match self.request_next() {
            Some(req) => run_page(req, source, to_msg),
            None => Task::none(),
        }
    }

    /// [`retry`](Infinite::retry), run with [`run_page`].
    pub fn load_retry<E, Fut, M>(
        &mut self,
        source: impl Fn(crate::primitives::infinite::Request<Q>) -> Fut,
        to_msg: impl Fn(RequestId, Result<Page<T>, String>) -> M + MaybeSend + 'static,
    ) -> Task<M>
    where
        Fut: Future<Output = Result<Page<T>, E>> + MaybeSend + 'static,
        E: ToString + 'static,
        T: MaybeSend + 'static,
        M: MaybeSend + 'static,
    {
        match self.retry() {
            Some(req) => run_page(req, source, to_msg),
            None => Task::none(),
        }
    }

    /// [`on_scroll`](Infinite::on_scroll), run with [`run_page`]: load a page when the scroll
    /// position is within `threshold_px` of the end.
    pub fn load_on_scroll<E, Fut, M>(
        &mut self,
        metrics: crate::primitives::infinite::ScrollMetrics,
        threshold_px: f32,
        source: impl Fn(crate::primitives::infinite::Request<Q>) -> Fut,
        to_msg: impl Fn(RequestId, Result<Page<T>, String>) -> M + MaybeSend + 'static,
    ) -> Task<M>
    where
        Fut: Future<Output = Result<Page<T>, E>> + MaybeSend + 'static,
        E: ToString + 'static,
        T: MaybeSend + 'static,
        M: MaybeSend + 'static,
    {
        match self.on_scroll(metrics, threshold_px) {
            Some(req) => run_page(req, source, to_msg),
            None => Task::none(),
        }
    }
}
