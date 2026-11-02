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
