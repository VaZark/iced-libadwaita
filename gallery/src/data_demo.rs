//! The Data page: sorting, pagination and infinite pagination.
//!
//! * a sortable, paginated table over 2,000 fake rows, as a table and (in a narrow container) as
//!   cards with the sort control above them; both drive the same `SortState` and `Pagination`;
//! * the `Pagination` widget alone in every state;
//! * two infinite lists over a fake async source (300-700 ms per page, 120 items): one that
//!   loads on scroll and one with a "Load more" button. The first list fails once at item 40
//!   (Retry), and its sort order resets it: sort while the slow second page (2.5 s) is in flight
//!   and the answer arrives last and is discarded (watch the counters).
//!
//! `GALLERY_DATA=table|cards|pagination|infinite|infinite-error|infinite-end|load-more` shows
//! one demo only; the infinite ones are seeded without waiting (`-error` stops at the failed
//! page, `-end` at the end of the data).

use iced_adaptive_design as iad;

use iad::components::typography as ty;
use iad::primitives::infinite::{Infinite, Page, Request};
use iad::primitives::lookup::RequestId;
use iad::primitives::pagination::Pagination as Paging;
use iad::primitives::sorting::{Direction, SortState, cmp_f64, cmp_natural};
use iad::widgets::{
    Column, InfiniteList, LoadTrigger, PageSize, Pagination, PaginationSize, ResponsiveTable,
};
use iad::{Element, Metrics};
use iad::{components as cmp, widgets as w};
use iced::widget::{column, container, row, space};
use iced::{Length, Task};
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Person {
    name: String,
    city: &'static str,
    score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Col {
    Name,
    City,
    Score,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    n: usize,
}

/// Which infinite list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Which {
    Scroll,
    Button,
}

/// The query of an infinite list: the sort order (a change resets the list).
type Query = (Which, bool);

#[derive(Debug, Clone)]
pub enum Msg {
    Sort(Col),
    Page(usize),
    PageSize(usize),
    More(Which),
    Retry(Which),
    Loaded(Which, RequestId, Result<Page<Item>, String>),
    Newest(bool),
}

pub struct State {
    all: Vec<Person>,
    sorted: Vec<Person>,
    sort: SortState<Col>,
    paging: Paging,
    scroll: Infinite<Query, Item>,
    button: Infinite<Query, Item>,
    newest: bool,
}

const CITIES: [&str; 8] = [
    "Paris", "Lyon", "Berlin", "Madrid", "Oslo", "Rome", "Vienna", "Prague",
];
const FIRST: [&str; 10] = [
    "Ada", "Grace", "Linus", "Margaret", "Alan", "Edsger", "Barbara", "Donald", "Ken", "Dennis",
];
const LAST: [&str; 10] = [
    "Lovelace", "Hopper", "Torvalds", "Hamilton", "Turing", "Dijkstra", "Liskov", "Knuth",
    "Thompson", "Ritchie",
];
const PAGE_SIZES: [PageSize; 4] = [PageSize(5), PageSize(10), PageSize(20), PageSize(50)];
const TOTAL_ITEMS: usize = 120;
const LIMIT: usize = 20;

fn people() -> Vec<Person> {
    (0..2000)
        .map(|i| Person {
            name: format!("{} {} {}", FIRST[i % 10], LAST[(i / 10) % 10], i + 1),
            city: CITIES[(i * 7) % 8],
            score: ((i * 37) % 1000) as f64 / 10.0,
        })
        .collect()
}

fn item_number(desc: bool, index: usize) -> usize {
    if desc { TOTAL_ITEMS - index } else { index + 1 }
}

static FAILED_ONCE: AtomicBool = AtomicBool::new(false);

/// The fake database: a page of items after a delay. The scroll list fails once at item 40; its
/// ascending second page is deliberately slow so a reset can overtake it.
async fn source(req: Request<Query>) -> Result<Page<Item>, String> {
    let (which, desc) = req.query;
    let slow = which == Which::Scroll && !desc && req.offset == LIMIT;
    let ms = if slow {
        2500
    } else {
        300 + (req.offset as u64 * 7) % 400
    };
    tokio::time::sleep(Duration::from_millis(ms)).await;
    if which == Which::Scroll && req.offset == 40 && !FAILED_ONCE.swap(true, AtomicOrdering::SeqCst)
    {
        return Err("The server did not answer".to_string());
    }
    let end = (req.offset + req.limit).min(TOTAL_ITEMS);
    let items = (req.offset..end)
        .map(|i| Item {
            n: item_number(desc, i),
        })
        .collect();
    Ok(Page::new(items).more(end < TOTAL_ITEMS))
}

/// A list's first `pages` pages without any waiting, for screenshots: `limit` items a page out
/// of `total`, then (with `fail`) a failed request.
fn seeded(
    which: Which,
    pages: usize,
    limit: usize,
    total: usize,
    fail: bool,
) -> Infinite<Query, Item> {
    let mut l = Infinite::new((which, false)).limit(limit);
    for _ in 0..pages {
        let Some(req) = l.request_next() else { break };
        let end = (req.offset + req.limit).min(total);
        let items = (req.offset..end).map(|i| Item { n: i + 1 }).collect();
        let _ = l.complete(req.id, Ok::<_, String>(Page::new(items).more(end < total)));
    }
    if fail && let Some(req) = l.request_next() {
        let _ = l.complete(req.id, Err::<Page<Item>, _>("The server did not answer"));
    }
    l
}

/// A titled group whose body brings its own surface (a table has its own card and padding).
fn bare<'a>(
    m: Metrics,
    t: &'static str,
    d: &'static str,
    body: Element<'a, Msg>,
) -> Element<'a, Msg> {
    column![
        ty::heading(t),
        ty::body(d).style(iad::styles::text::dim),
        body
    ]
    .spacing(m.space_s)
    .into()
}

fn group<'a>(
    m: Metrics,
    t: &'static str,
    d: &'static str,
    body: Element<'a, Msg>,
) -> Element<'a, Msg> {
    w::PreferencesGroup::<Msg>::new()
        .title(t)
        .description(d)
        .metrics(m)
        .push(w::card(
            container(body).padding(m.space_m).width(Length::Fill),
        ))
        .into()
}

impl Default for State {
    fn default() -> Self {
        let mode = std::env::var("GALLERY_DATA").unwrap_or_default();
        let all = people();
        let (scroll, button) = match mode.as_str() {
            // A short first page, then the failed request: the Retry footer is in view.
            "infinite-error" => (
                seeded(Which::Scroll, 1, 5, TOTAL_ITEMS, true),
                seeded(Which::Button, 1, LIMIT, TOTAL_ITEMS, false),
            ),
            // All the data loaded: the end-of-list footer is in view.
            "infinite-end" => (
                seeded(Which::Scroll, 1, 6, 6, false),
                seeded(Which::Button, 1, 6, 6, false),
            ),
            _ => (
                seeded(Which::Scroll, 1, LIMIT, TOTAL_ITEMS, false),
                seeded(Which::Button, 1, LIMIT, TOTAL_ITEMS, false),
            ),
        };
        let mut s = Self {
            sorted: all.clone(),
            all,
            sort: SortState::by(Col::Score, Direction::Desc),
            paging: Paging::new(2000, 5),
            scroll,
            button,
            newest: false,
        };
        s.resort();
        s
    }
}

impl State {
    fn resort(&mut self) {
        self.sorted = self.all.clone();
        self.sort.apply(&mut self.sorted, |k, a, b| match k {
            Col::Name => cmp_natural(&a.name, &b.name),
            Col::City => a.city.cmp(b.city),
            Col::Score => cmp_f64(a.score, b.score),
        });
    }

    fn list(&mut self, which: Which) -> &mut Infinite<Query, Item> {
        match which {
            Which::Scroll => &mut self.scroll,
            Which::Button => &mut self.button,
        }
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Sort(col) => {
                self.sort.activate(col, false);
                self.resort();
                self.paging.goto(1);
                Task::none()
            }
            Msg::Page(p) => {
                self.paging.goto(p);
                Task::none()
            }
            Msg::PageSize(n) => {
                self.paging.set_page_size(n);
                Task::none()
            }
            Msg::More(which) => self
                .list(which)
                .load_next(source, move |id, r| Msg::Loaded(which, id, r)),
            Msg::Retry(which) => self
                .list(which)
                .load_retry(source, move |id, r| Msg::Loaded(which, id, r)),
            Msg::Loaded(which, id, r) => {
                let _ = self.list(which).complete(id, r);
                Task::none()
            }
            Msg::Newest(desc) => {
                self.newest = desc;
                // A new sort order: clear, then ask for its first page. The old answer, if
                // still in flight, is discarded when it arrives.
                self.scroll.reset((Which::Scroll, desc));
                self.scroll
                    .load_next(source, |id, r| Msg::Loaded(Which::Scroll, id, r))
            }
        }
    }

    pub fn view(&self, m: Metrics) -> Element<'_, Msg> {
        let sp = m.space_m;
        let only = std::env::var("GALLERY_DATA").unwrap_or_default();
        let show = |names: &[&str]| only.is_empty() || names.contains(&only.as_str());
        let mut col = column![].spacing(m.group_spacing);

        // -- table and cards
        if show(&["table", "cards"]) {
            let start = self.paging.offset();
            let end = (start + self.paging.page_size()).min(self.sorted.len());
            let shown = &self.sorted[start..end];
            let table = |height: f32| -> Element<'_, Msg> {
                ResponsiveTable::<Person, Msg>::new(shown)
                    .column(
                        Column::new("Name", |p: &Person| ty::body(p.name.clone()).into())
                            .min_width(160.0)
                            .sortable(Col::Name),
                    )
                    .column(
                        Column::new("City", |p: &Person| ty::body(p.city).into())
                            .min_width(90.0)
                            .sortable(Col::City),
                    )
                    .column(
                        Column::new("Score", |p: &Person| {
                            ty::body(format!("{:.1}", p.score)).into()
                        })
                        .min_width(70.0)
                        .sortable(Col::Score),
                    )
                    .sort(&self.sort)
                    .on_sort(Msg::Sort)
                    .height(Length::Fixed(height))
                    .into()
            };
            if show(&["table"]) {
                col = col.push(bare(
                    m,
                    "Sortable table",
                    "Click a header: ascending, descending, unsorted. Paginated below.",
                    table(330.0),
                ));
            }
            if show(&["cards"]) {
                let narrow = container(table(430.0)).width(Length::Fixed(300.0));
                col = col.push(bare(
                    m,
                    "The same table in a narrow container",
                    "Cards have no header row, so a sort control sits above them (the same message).",
                    narrow.into(),
                ));
            }
            let footer = column![
                row![
                    w::pagination_label::<Msg>(&self.paging),
                    container(space().width(Length::Fill)),
                    w::page_size_dropdown(&PAGE_SIZES, self.paging.page_size(), Msg::PageSize),
                ]
                .align_y(iced::Alignment::Center),
                Pagination::new(&self.paging, Msg::Page).siblings(1),
            ]
            .spacing(sp);
            col = col.push(group(
                m,
                "Footer",
                "Caption, pager and page size.",
                footer.into(),
            ));
        }

        // -- the pager alone
        if show(&["pagination"]) {
            let p = |total: usize, page: usize| Paging::new(total * 10, 10).at(page);
            let line = |label: &'static str, e: Element<'static, Msg>| -> Element<'static, Msg> {
                column![ty::caption(label).style(iad::styles::text::dim), e]
                    .spacing(m.space_s)
                    .into()
            };
            let states = column![
                line("First page", Pagination::new(&p(12, 1), Msg::Page).into()),
                line(
                    "Middle, with ellipses",
                    Pagination::new(&p(40, 20), Msg::Page).into()
                ),
                line("Last page", Pagination::new(&p(12, 12), Msg::Page).into()),
                line("Few pages", Pagination::new(&p(3, 2), Msg::Page).into()),
                line(
                    "With first / last",
                    Pagination::new(&p(40, 20), Msg::Page)
                        .with_edges(true)
                        .into()
                ),
                line(
                    "Siblings 2, boundaries 2",
                    Pagination::new(&p(40, 20), Msg::Page)
                        .siblings(2)
                        .boundaries(2)
                        .into()
                ),
                line(
                    "Small",
                    Pagination::new(&p(12, 5), Msg::Page)
                        .size(PaginationSize::Small)
                        .into()
                ),
                line(
                    "Large",
                    Pagination::new(&p(12, 5), Msg::Page)
                        .size(PaginationSize::Large)
                        .into()
                ),
                line(
                    "Disabled",
                    Pagination::new(&p(12, 5), Msg::Page).disabled(true).into()
                ),
            ]
            .spacing(m.space_m);
            col = col.push(group(
                m,
                "Pagination",
                "Every state of the pager.",
                states.into(),
            ));
        }

        // -- infinite lists
        if show(&["infinite", "infinite-error", "infinite-end", "load-more"]) {
            let rows = |l: &Infinite<Query, Item>| -> Element<'static, Msg> {
                column(l.items().iter().map(|i| {
                    container(ty::body(format!("Item {}", i.n)))
                        .padding([m.space_s, m.space_m])
                        .width(Length::Fill)
                        .into()
                }))
                .into()
            };
            let readout = |l: &Infinite<Query, Item>| -> Element<'static, Msg> {
                let s = l.stats();
                ty::caption(format!(
                    "{} loaded - applied {} - discarded {}",
                    l.len(),
                    s.applied,
                    s.discarded
                ))
                .style(iad::styles::text::dim)
                .into()
            };
            if show(&["infinite", "infinite-error", "infinite-end"]) {
                let list = InfiniteList::new(rows(&self.scroll), &self.scroll)
                    .on_load_more(Msg::More(Which::Scroll))
                    .on_retry(Msg::Retry(Which::Scroll))
                    .height(Length::Fixed(260.0));
                let order = row![
                    cmp::toggle_group(
                        vec![
                            cmp::ToggleItem::new(false, "Oldest first"),
                            cmp::ToggleItem::new(true, "Newest first"),
                        ],
                        Some(self.newest),
                        Msg::Newest,
                    ),
                    readout(&self.scroll),
                ]
                .spacing(sp)
                .align_y(iced::Alignment::Center);
                col = col.push(group(
                    m,
                    "Infinite scroll",
                    "Scroll to the end to load the next page. Fails once at item 40 (Retry).",
                    column![order, Element::from(list)].spacing(sp).into(),
                ));
            }
            if show(&["infinite", "load-more"]) {
                let list = InfiniteList::new(rows(&self.button), &self.button)
                    .trigger(LoadTrigger::LoadMore)
                    .on_load_more(Msg::More(Which::Button))
                    .on_retry(Msg::Retry(Which::Button))
                    .height(Length::Fixed(260.0));
                col = col.push(group(
                    m,
                    "Load more",
                    "The same list with a button instead of scrolling.",
                    column![readout(&self.button), Element::from(list)]
                        .spacing(sp)
                        .into(),
                ));
            }
        }
        col.into()
    }
}
