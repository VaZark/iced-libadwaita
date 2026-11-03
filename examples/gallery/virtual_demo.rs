//! The virtualization and async pages of the Data group: the same two primitives the tree uses,
//! on a table.
//!
//! * **Virtualization**: a table of 100,000 rows. The app keeps a `Viewport` (from the table's
//!   `on_scroll`), asks it for the window, and passes only those rows; the table pads the scroll
//!   area with exact spacers (`primitives::virtualize`).
//! * **Async chunks**: 5,000 rows fetched 50 at a time as they are scrolled to. A row whose chunk
//!   has not arrived is a placeholder, a failed chunk is an error row with Retry, and a failing
//!   chunk is not asked for again until retried (`primitives::loading::Chunks`). Every seventh
//!   chunk fails once.

use iced_libadwaita as adw;

use adw::primitives::loading::{Chunks, Slot};
use adw::primitives::virtualize::Viewport;
use adw::widget::{self as w, Column, ResponsiveTable, typography as ty};
use adw::{Element, Metrics};
use iced::widget::{column, container};
use iced::{Length, Task};
use std::collections::{HashMap, HashSet};
use std::time::Duration;

const TOTAL: usize = 100_000;
const ASYNC_TOTAL: usize = 5_000;
const CHUNK: usize = 50;
const ROW: f32 = 36.0;
const OVERSCAN: usize = 4;
const VIEW_HEIGHT: f32 = 440.0;

#[derive(Debug, Clone)]
pub struct Person {
    n: usize,
    name: String,
    city: &'static str,
    score: u32,
}

fn person(n: usize) -> Person {
    const FIRST: [&str; 8] = [
        "Ada", "Grace", "Linus", "Margaret", "Alan", "Barbara", "Dennis", "Tim",
    ];
    const LAST: [&str; 6] = [
        "Lovelace", "Hopper", "Torvalds", "Hamilton", "Turing", "Liskov",
    ];
    const CITY: [&str; 5] = ["London", "Berlin", "Tokyo", "Lagos", "Lima"];
    Person {
        n,
        name: format!("{} {}", FIRST[n % FIRST.len()], LAST[(n / 8) % LAST.len()]),
        city: CITY[n % CITY.len()],
        score: ((n * 7919) % 1000) as u32,
    }
}

/// A row of the async table: the person, or why there is none yet.
#[derive(Debug, Clone)]
pub enum Slotted {
    Ready(Person),
    Loading,
    Failed { chunk: usize, reason: String },
}

#[derive(Debug, Clone)]
pub enum Msg {
    Scroll(Viewport),
    AsyncScroll(Viewport),
    Fetched(usize, Result<Vec<Person>, String>),
    Retry(usize),
}

pub struct State {
    vp: Viewport,
    /// The rows in the virtual table's window, and the first one's index.
    vrows: Vec<Person>,
    vfirst: usize,
    avp: Viewport,
    chunks: Chunks,
    cache: HashMap<usize, Vec<Person>>,
    failed_once: HashSet<usize>,
    arows: Vec<Slotted>,
    afirst: usize,
}

async fn fetch(chunk: usize, fail: bool) -> Result<Vec<Person>, String> {
    tokio::time::sleep(Duration::from_millis(450)).await;
    if fail {
        return Err("The server did not answer".into());
    }
    let start = chunk * CHUNK;
    Ok((start..(start + CHUNK).min(ASYNC_TOTAL))
        .map(person)
        .collect())
}

fn pitch() -> f32 {
    ResponsiveTable::<Person, ()>::pitch(ROW)
}

impl State {
    /// The state and the first fetches.
    pub fn start() -> (Self, Task<Msg>) {
        let mut s = Self {
            vp: Viewport::new(VIEW_HEIGHT),
            vrows: Vec::new(),
            vfirst: 0,
            avp: Viewport::new(VIEW_HEIGHT),
            chunks: Chunks::new(ASYNC_TOTAL, CHUNK),
            cache: HashMap::new(),
            failed_once: HashSet::new(),
            arows: Vec::new(),
            afirst: 0,
        };
        s.window_virtual();
        let task = s.request();
        (s, task)
    }

    fn window_virtual(&mut self) {
        let range = self.vp.window(TOTAL, pitch(), OVERSCAN);
        self.vfirst = range.start;
        self.vrows = range.map(person).collect();
    }

    /// Asks for the chunks the async window needs and rebuilds its rows.
    fn request(&mut self) -> Task<Msg> {
        let range = self.avp.window(ASYNC_TOTAL, pitch(), OVERSCAN);
        let tasks: Vec<Task<Msg>> = self
            .chunks
            .request(range.clone())
            .into_iter()
            .map(|c| self.fetch_task(c))
            .collect();
        self.rebuild(range);
        Task::batch(tasks)
    }

    fn fetch_task(&mut self, chunk: usize) -> Task<Msg> {
        // Every seventh chunk fails the first time.
        let fail = chunk % 7 == 3 && self.failed_once.insert(chunk);
        Task::perform(fetch(chunk, fail), move |r| Msg::Fetched(chunk, r))
    }

    fn rebuild(&mut self, range: std::ops::Range<usize>) {
        self.afirst = range.start;
        self.arows = range
            .map(|row| match self.chunks.slot(row) {
                Slot::Ready => {
                    let c = self.chunks.chunk_of(row);
                    self.cache
                        .get(&c)
                        .and_then(|rows| rows.get(row - c * CHUNK))
                        .cloned()
                        .map_or(Slotted::Loading, Slotted::Ready)
                }
                Slot::Loading => Slotted::Loading,
                Slot::Failed(e) => Slotted::Failed {
                    chunk: self.chunks.chunk_of(row),
                    reason: e.to_string(),
                },
            })
            .collect();
    }

    pub fn update(&mut self, m: Msg) -> Task<Msg> {
        match m {
            Msg::Scroll(v) => {
                self.vp = v;
                self.window_virtual();
                Task::none()
            }
            Msg::AsyncScroll(v) => {
                self.avp = v;
                self.request()
            }
            Msg::Fetched(c, Ok(rows)) => {
                self.cache.insert(c, rows);
                self.chunks.finish(c, Ok(()));
                self.request()
            }
            Msg::Fetched(c, Err(e)) => {
                self.chunks.finish(c, Err(e));
                self.request()
            }
            Msg::Retry(c) => {
                let task = if self.chunks.retry(c) {
                    self.fetch_task(c)
                } else {
                    Task::none()
                };
                let range = self.avp.window(ASYNC_TOTAL, pitch(), OVERSCAN);
                self.rebuild(range);
                task
            }
        }
    }

    /// 100,000 rows, the ones in view built.
    pub fn virtual_view(&self, m: Metrics) -> Element<'_, Msg> {
        let table = ResponsiveTable::new(&self.vrows)
            .column(
                Column::new("#", |p: &Person| ty::body(p.n.to_string()).into())
                    .width(Length::Fixed(80.0)),
            )
            .column(
                Column::new("Name", |p: &Person| ty::heading(p.name.clone()).into())
                    .min_width(160.0),
            )
            .column(Column::new("City", |p: &Person| ty::body(p.city).into()).min_width(100.0))
            .column(
                Column::new("Score", |p: &Person| ty::body(p.score.to_string()).into())
                    .min_width(80.0),
            )
            .virtualized(TOTAL, self.vfirst, ROW)
            .on_scroll(Msg::Scroll)
            .metrics(m);
        column![
            ty::caption(format!(
                "{TOTAL} rows, {} built (rows {}–{})",
                self.vrows.len(),
                self.vfirst,
                self.vfirst + self.vrows.len()
            )),
            container(table).height(VIEW_HEIGHT)
        ]
        .spacing(m.space_m)
        .into()
    }

    /// 5,000 rows fetched 50 at a time, with placeholders and Retry.
    pub fn async_view(&self, m: Metrics) -> Element<'_, Msg> {
        let cell = move |text: fn(&Person) -> String, first: bool| {
            move |r: &Slotted| -> Element<'_, Msg> {
                match r {
                    Slotted::Ready(p) => ty::body(text(p)).into(),
                    Slotted::Loading => ty::body("·····").style(adw::style::text::dim).into(),
                    Slotted::Failed { chunk, reason, .. } if first => iced::widget::row![
                        ty::body(format!("Failed: {reason}")).style(adw::style::text::error),
                        w::flat_button(ty::label("Retry")).on_press(Msg::Retry(*chunk)),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center)
                    .into(),
                    Slotted::Failed { .. } => iced::widget::space().into(),
                }
            }
        };
        let ready = self.cache.len();
        let table = ResponsiveTable::new(&self.arows)
            .column(Column::new("#", cell(|p| p.n.to_string(), false)).width(Length::Fixed(80.0)))
            .column(Column::new("Name", cell(|p| p.name.clone(), true)).min_width(160.0))
            .column(Column::new("City", cell(|p| p.city.to_string(), false)).min_width(100.0))
            .column(Column::new("Score", cell(|p| p.score.to_string(), false)).min_width(80.0))
            .virtualized(ASYNC_TOTAL, self.afirst, ROW)
            .on_scroll(Msg::AsyncScroll)
            .metrics(m);
        column![
            ty::caption(format!(
                "{ASYNC_TOTAL} rows in chunks of {CHUNK}; {ready} chunk{} loaded. Scroll fast to see placeholders; every seventh chunk fails once.",
                if ready == 1 { "" } else { "s" }
            )),
            container(table).height(VIEW_HEIGHT)
        ]
        .spacing(m.space_m)
        .into()
    }
}
