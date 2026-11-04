//! The Autocomplete page: suggestions from a fake async "database" (a few hundred cities behind
//! `tokio::time::sleep`), the lookup primitive's status as a live readout, the states side by
//! side, and one inside a `Field` that only accepts a picked city.
//!
//! * "fail" errors on every other attempt, so Retry can be tried.
//! * "san" is a deliberately slow query (2.5 s): type "san", wait for the spinner, then add a
//!   letter ("sant"). The slow answer arrives last and is discarded (watch the counters).
//!
//! `GALLERY_AUTOCOMPLETE=idle|loading|results|empty|error|disabled` shows the States first with
//! that one open, without waiting. `GALLERY_AUTOCOMPLETE=type:a>b>c` instead types `a`, then
//! `b` 1 s later, then `c` 1 s after that into the live entry, through the real async path
//! (`type:san>sant` shows a stale response being discarded; pair it with `GALLERY_ANIM=keys`,
//! which captures after 6 s).

use iced_adaptive_design as iad;

use iad::components::typography as ty;
use iad::lookup as glue;
use iad::primitives::lookup::{Lookup, RequestId, Status, validate_pick};
use iad::primitives::validation::Validity;
use iad::primitives::validation::required;
use iad::widgets::{Autocomplete, Field};
use iad::{Element, Metrics};
use iad::{components as cmp, widgets as w};
use iced::widget::{column, container, row, space, text};
use iced::{Length, Subscription, Task};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq)]
pub struct Place {
    name: &'static str,
    country: &'static str,
}

impl std::fmt::Display for Place {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

const PLACES: &[(&str, &str)] = &[
    ("Paris", "France"),
    ("Lyon", "France"),
    ("Marseille", "France"),
    ("Toulouse", "France"),
    ("Nice", "France"),
    ("Nantes", "France"),
    ("Bordeaux", "France"),
    ("Lille", "France"),
    ("Strasbourg", "France"),
    ("Rennes", "France"),
    ("London", "United Kingdom"),
    ("Manchester", "United Kingdom"),
    ("Birmingham", "United Kingdom"),
    ("Leeds", "United Kingdom"),
    ("Glasgow", "United Kingdom"),
    ("Liverpool", "United Kingdom"),
    ("Bristol", "United Kingdom"),
    ("Edinburgh", "United Kingdom"),
    ("Sheffield", "United Kingdom"),
    ("Cardiff", "United Kingdom"),
    ("Slough", "United Kingdom"),
    ("Failsworth", "United Kingdom"),
    ("Berlin", "Germany"),
    ("Hamburg", "Germany"),
    ("Munich", "Germany"),
    ("Cologne", "Germany"),
    ("Frankfurt", "Germany"),
    ("Stuttgart", "Germany"),
    ("Dresden", "Germany"),
    ("Leipzig", "Germany"),
    ("Hanover", "Germany"),
    ("Nuremberg", "Germany"),
    ("Madrid", "Spain"),
    ("Barcelona", "Spain"),
    ("Valencia", "Spain"),
    ("Seville", "Spain"),
    ("Zaragoza", "Spain"),
    ("Malaga", "Spain"),
    ("Bilbao", "Spain"),
    ("Granada", "Spain"),
    ("Rome", "Italy"),
    ("Milan", "Italy"),
    ("Naples", "Italy"),
    ("Turin", "Italy"),
    ("Florence", "Italy"),
    ("Venice", "Italy"),
    ("Bologna", "Italy"),
    ("Genoa", "Italy"),
    ("Palermo", "Italy"),
    ("Verona", "Italy"),
    ("Lisbon", "Portugal"),
    ("Porto", "Portugal"),
    ("Braga", "Portugal"),
    ("Faro", "Portugal"),
    ("Amsterdam", "Netherlands"),
    ("Rotterdam", "Netherlands"),
    ("Utrecht", "Netherlands"),
    ("The Hague", "Netherlands"),
    ("Eindhoven", "Netherlands"),
    ("Brussels", "Belgium"),
    ("Antwerp", "Belgium"),
    ("Ghent", "Belgium"),
    ("Bruges", "Belgium"),
    ("Zurich", "Switzerland"),
    ("Geneva", "Switzerland"),
    ("Basel", "Switzerland"),
    ("Bern", "Switzerland"),
    ("Lausanne", "Switzerland"),
    ("Vienna", "Austria"),
    ("Salzburg", "Austria"),
    ("Graz", "Austria"),
    ("Innsbruck", "Austria"),
    ("Prague", "Czechia"),
    ("Brno", "Czechia"),
    ("Warsaw", "Poland"),
    ("Krakow", "Poland"),
    ("Gdansk", "Poland"),
    ("Wroclaw", "Poland"),
    ("Poznan", "Poland"),
    ("Budapest", "Hungary"),
    ("Debrecen", "Hungary"),
    ("Bucharest", "Romania"),
    ("Cluj-Napoca", "Romania"),
    ("Sofia", "Bulgaria"),
    ("Athens", "Greece"),
    ("Thessaloniki", "Greece"),
    ("Copenhagen", "Denmark"),
    ("Aarhus", "Denmark"),
    ("Stockholm", "Sweden"),
    ("Gothenburg", "Sweden"),
    ("Malmo", "Sweden"),
    ("Oslo", "Norway"),
    ("Bergen", "Norway"),
    ("Helsinki", "Finland"),
    ("Tampere", "Finland"),
    ("Dublin", "Ireland"),
    ("Cork", "Ireland"),
    ("Reykjavik", "Iceland"),
    ("Tallinn", "Estonia"),
    ("Riga", "Latvia"),
    ("Vilnius", "Lithuania"),
    ("Istanbul", "Turkey"),
    ("Ankara", "Turkey"),
    ("Izmir", "Turkey"),
    ("Kyiv", "Ukraine"),
    ("Lviv", "Ukraine"),
    ("Moscow", "Russia"),
    ("Saint Petersburg", "Russia"),
    ("New York", "United States"),
    ("Los Angeles", "United States"),
    ("Chicago", "United States"),
    ("Houston", "United States"),
    ("Phoenix", "United States"),
    ("Philadelphia", "United States"),
    ("San Antonio", "United States"),
    ("San Diego", "United States"),
    ("Dallas", "United States"),
    ("San Jose", "United States"),
    ("Austin", "United States"),
    ("Seattle", "United States"),
    ("Denver", "United States"),
    ("Boston", "United States"),
    ("Portland", "United States"),
    ("Miami", "United States"),
    ("Atlanta", "United States"),
    ("San Francisco", "United States"),
    ("Santa Fe", "United States"),
    ("Santa Barbara", "United States"),
    ("Toronto", "Canada"),
    ("Montreal", "Canada"),
    ("Vancouver", "Canada"),
    ("Calgary", "Canada"),
    ("Ottawa", "Canada"),
    ("Edmonton", "Canada"),
    ("Quebec City", "Canada"),
    ("Winnipeg", "Canada"),
    ("Mexico City", "Mexico"),
    ("Guadalajara", "Mexico"),
    ("Monterrey", "Mexico"),
    ("Puebla", "Mexico"),
    ("Havana", "Cuba"),
    ("Bogota", "Colombia"),
    ("Medellin", "Colombia"),
    ("Lima", "Peru"),
    ("Cusco", "Peru"),
    ("Santiago", "Chile"),
    ("Valparaiso", "Chile"),
    ("Buenos Aires", "Argentina"),
    ("Cordoba", "Argentina"),
    ("Rosario", "Argentina"),
    ("Montevideo", "Uruguay"),
    ("Sao Paulo", "Brazil"),
    ("Rio de Janeiro", "Brazil"),
    ("Brasilia", "Brazil"),
    ("Salvador", "Brazil"),
    ("Recife", "Brazil"),
    ("Quito", "Ecuador"),
    ("Caracas", "Venezuela"),
    ("La Paz", "Bolivia"),
    ("Santo Domingo", "Dominican Republic"),
    ("Cairo", "Egypt"),
    ("Alexandria", "Egypt"),
    ("Casablanca", "Morocco"),
    ("Marrakesh", "Morocco"),
    ("Rabat", "Morocco"),
    ("Tunis", "Tunisia"),
    ("Algiers", "Algeria"),
    ("Lagos", "Nigeria"),
    ("Abuja", "Nigeria"),
    ("Accra", "Ghana"),
    ("Dakar", "Senegal"),
    ("Nairobi", "Kenya"),
    ("Mombasa", "Kenya"),
    ("Addis Ababa", "Ethiopia"),
    ("Dar es Salaam", "Tanzania"),
    ("Kampala", "Uganda"),
    ("Kigali", "Rwanda"),
    ("Johannesburg", "South Africa"),
    ("Cape Town", "South Africa"),
    ("Durban", "South Africa"),
    ("Pretoria", "South Africa"),
    ("Luanda", "Angola"),
    ("Maputo", "Mozambique"),
    ("Tokyo", "Japan"),
    ("Osaka", "Japan"),
    ("Kyoto", "Japan"),
    ("Yokohama", "Japan"),
    ("Nagoya", "Japan"),
    ("Sapporo", "Japan"),
    ("Fukuoka", "Japan"),
    ("Hiroshima", "Japan"),
    ("Seoul", "South Korea"),
    ("Busan", "South Korea"),
    ("Incheon", "South Korea"),
    ("Beijing", "China"),
    ("Shanghai", "China"),
    ("Shenzhen", "China"),
    ("Guangzhou", "China"),
    ("Chengdu", "China"),
    ("Hangzhou", "China"),
    ("Xi'an", "China"),
    ("Hong Kong", "China"),
    ("Taipei", "Taiwan"),
    ("Singapore", "Singapore"),
    ("Kuala Lumpur", "Malaysia"),
    ("Bangkok", "Thailand"),
    ("Chiang Mai", "Thailand"),
    ("Hanoi", "Vietnam"),
    ("Ho Chi Minh City", "Vietnam"),
    ("Manila", "Philippines"),
    ("Cebu", "Philippines"),
    ("Jakarta", "Indonesia"),
    ("Surabaya", "Indonesia"),
    ("Bandung", "Indonesia"),
    ("Mumbai", "India"),
    ("Delhi", "India"),
    ("Bangalore", "India"),
    ("Hyderabad", "India"),
    ("Chennai", "India"),
    ("Kolkata", "India"),
    ("Pune", "India"),
    ("Ahmedabad", "India"),
    ("Jaipur", "India"),
    ("Kochi", "India"),
    ("Karachi", "Pakistan"),
    ("Lahore", "Pakistan"),
    ("Islamabad", "Pakistan"),
    ("Dhaka", "Bangladesh"),
    ("Colombo", "Sri Lanka"),
    ("Kathmandu", "Nepal"),
    ("Tehran", "Iran"),
    ("Isfahan", "Iran"),
    ("Baghdad", "Iraq"),
    ("Riyadh", "Saudi Arabia"),
    ("Jeddah", "Saudi Arabia"),
    ("Dubai", "United Arab Emirates"),
    ("Abu Dhabi", "United Arab Emirates"),
    ("Doha", "Qatar"),
    ("Tel Aviv", "Israel"),
    ("Jerusalem", "Israel"),
    ("Amman", "Jordan"),
    ("Beirut", "Lebanon"),
    ("Tashkent", "Uzbekistan"),
    ("Almaty", "Kazakhstan"),
    ("Sydney", "Australia"),
    ("Melbourne", "Australia"),
    ("Brisbane", "Australia"),
    ("Perth", "Australia"),
    ("Adelaide", "Australia"),
    ("Canberra", "Australia"),
    ("Auckland", "New Zealand"),
    ("Wellington", "New Zealand"),
    ("Christchurch", "New Zealand"),
    ("Suva", "Fiji"),
    ("Santander", "Spain"),
    ("Santiago de Compostela", "Spain"),
    ("Santa Cruz", "Bolivia"),
    ("Sanaa", "Yemen"),
    ("Salt Lake City", "United States"),
];

static FLAKY: AtomicUsize = AtomicUsize::new(0);

/// The fake database: substring match, prefix matches first, 300-800 ms of latency.
async fn search(query: String) -> Result<Vec<Place>, String> {
    let q = query.to_lowercase();
    let latency = match q.as_str() {
        "san" => 2500,
        _ => 300 + (q.bytes().map(u64::from).sum::<u64>() * 37) % 500,
    };
    tokio::time::sleep(Duration::from_millis(latency)).await;
    if q == "fail" && FLAKY.fetch_add(1, Ordering::Relaxed).is_multiple_of(2) {
        return Err(String::from("The database did not answer (timeout)."));
    }
    let mut hits: Vec<(bool, Place)> = PLACES
        .iter()
        .filter(|(n, _)| n.to_lowercase().contains(&q))
        .map(|(n, c)| {
            (
                n.to_lowercase().starts_with(&q),
                Place {
                    name: n,
                    country: c,
                },
            )
        })
        .collect();
    hits.sort_by_key(|(prefix, p)| (!*prefix, p.name));
    Ok(hits.into_iter().map(|(_, p)| p).take(30).collect())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Which {
    Live,
    Field,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Input(Which, String),
    Tick(Which, Instant),
    Found(Which, RequestId, Result<Vec<Place>, String>),
    Retry(Which),
    Pick(Which, Place),
    Submit,
    Reset,
    Noop,
}

struct Slot {
    text: String,
    lookup: Lookup<String, Place>,
    picked: Option<String>,
}

impl Slot {
    fn new(cache: usize) -> Self {
        Self {
            text: String::new(),
            lookup: Lookup::new().limit(30).cache(cache),
            picked: None,
        }
    }
}

/// A lookup in a given state, built through the primitive's own API.
fn seeded(kind: &str, query: &str) -> Lookup<String, Place> {
    let t0 = Instant::now();
    let mut l = Lookup::new();
    l.input(query.to_string(), t0);
    let Some(req) = l.poll(t0 + Duration::from_secs(1)) else {
        return l;
    };
    let place = |i: usize| Place {
        name: PLACES[i].0,
        country: PLACES[i].1,
    };
    match kind {
        "results" => {
            l.complete(req.id, Ok((0..4).map(place).collect()));
        }
        "empty" => {
            l.complete(req.id, Ok(Vec::new()));
        }
        "error" => {
            l.complete(req.id, Err(String::from("The database did not answer.")));
        }
        _ => {}
    }
    l
}

fn padded_card<'a>(c: Element<'a, Msg>, sp: f32) -> Element<'a, Msg> {
    w::card(container(c).padding(sp).width(Length::Fill))
}

pub struct State {
    live: Slot,
    field: Slot,
    touched: bool,
    submitted: bool,
    strip: Vec<(&'static str, &'static str, Lookup<String, Place>)>,
    open: Option<String>,
}

impl Default for State {
    fn default() -> Self {
        let strip = vec![
            ("idle", "", Lookup::new()),
            ("loading", "par", seeded("loading", "par")),
            ("results", "par", seeded("results", "par")),
            ("empty", "zzz", seeded("empty", "zzz")),
            ("error", "fail", seeded("error", "fail")),
            ("disabled", "Paris", Lookup::new()),
        ];
        Self {
            live: Slot::new(8),
            field: Slot::new(0),
            touched: false,
            submitted: false,
            strip,
            open: std::env::var("GALLERY_AUTOCOMPLETE")
                .ok()
                .filter(|v| !v.starts_with("type:")),
        }
    }
}

/// `GALLERY_AUTOCOMPLETE=type:a>b>c`: scripted typing into the live entry (see the module docs).
pub fn script() -> Task<Msg> {
    let Ok(v) = std::env::var("GALLERY_AUTOCOMPLETE") else {
        return Task::none();
    };
    let Some(steps) = v.strip_prefix("type:") else {
        return Task::none();
    };
    Task::batch(steps.split('>').enumerate().map(|(i, t)| {
        let t = t.to_string();
        Task::perform(
            tokio::time::sleep(Duration::from_millis(1000 * i as u64 + 1500)),
            move |_| Msg::Input(Which::Live, t.clone()),
        )
    }))
}

impl State {
    fn slot(&mut self, which: Which) -> &mut Slot {
        match which {
            Which::Live => &mut self.live,
            Which::Field => &mut self.field,
        }
    }

    pub fn subscription(&self) -> Subscription<Msg> {
        Subscription::batch([
            self.live.lookup.subscription(|n| Msg::Tick(Which::Live, n)),
            self.field
                .lookup
                .subscription(|n| Msg::Tick(Which::Field, n)),
        ])
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        match msg {
            Msg::Input(w, t) => {
                if w == Which::Field {
                    self.touched = true;
                }
                let s = self.slot(w);
                if s.picked.as_deref() != Some(&t) {
                    s.picked = None;
                }
                s.lookup.input(t.clone(), Instant::now());
                s.text = t;
                Task::none()
            }
            Msg::Tick(w, now) => self
                .slot(w)
                .lookup
                .run(now, search, move |id, r| Msg::Found(w, id, r)),
            Msg::Found(w, id, r) => {
                self.slot(w).lookup.complete(id, r);
                Task::none()
            }
            Msg::Retry(w) => match self.slot(w).lookup.retry() {
                Some(req) => glue::run(req, search, move |id, r| Msg::Found(w, id, r)),
                None => Task::none(),
            },
            Msg::Pick(w, p) => {
                if w == Which::Field {
                    self.touched = true;
                }
                let s = self.slot(w);
                s.text = p.name.to_string();
                s.picked = Some(p.name.to_string());
                s.lookup.clear();
                Task::none()
            }
            Msg::Submit => {
                self.submitted = true;
                Task::none()
            }
            Msg::Reset => {
                self.field = Slot::new(0);
                self.touched = false;
                self.submitted = false;
                Task::none()
            }
            Msg::Noop => Task::none(),
        }
    }

    fn field_validity(&self) -> Validity {
        let s = &self.field;
        match required(&s.text, "Choose a city.") {
            Validity::Ok => validate_pick(
                &s.text,
                s.picked.as_deref(),
                false,
                "Pick a city from the list.",
            ),
            e => e,
        }
    }

    fn auto<'a>(
        &'a self,
        which: Which,
        slot: &'a Slot,
        m: Metrics,
    ) -> Autocomplete<'a, Place, Msg> {
        Autocomplete::new(&slot.text, &slot.lookup, move |t| Msg::Input(which, t))
            .on_select(move |p| Msg::Pick(which, p))
            .on_retry(Msg::Retry(which))
            .item(|p: &Place| {
                row![
                    text(p.name).wrapping(text::Wrapping::None),
                    space().width(Length::Fill),
                    ty::caption(p.country).style(iad::styles::text::dim),
                ]
                .spacing(12.0)
                .into()
            })
            .search_icon(true)
            .metrics(m)
    }

    fn readout(&self, m: Metrics) -> Element<'_, Msg> {
        let l = &self.live.lookup;
        let status = match l.status() {
            Status::Idle => String::from("Idle"),
            Status::Waiting(t) => format!(
                "Waiting (debounce, {} ms left)",
                t.saturating_duration_since(Instant::now()).as_millis()
            ),
            Status::Loading { id } => format!("Loading request #{id}"),
            Status::Loaded => format!("Loaded, {} results", l.results().len()),
            Status::Failed(e) => format!("Failed: {e}"),
        };
        let stats = l.stats();
        let line = |k: &'static str, v: String| {
            row![
                container(ty::caption(k).style(iad::styles::text::dim)).width(Length::Fixed(150.0)),
                ty::caption(v)
            ]
        };
        container(
            column![
                line("Status", status),
                line(
                    "Query",
                    format!("{:?}", l.query().map(String::as_str).unwrap_or(""))
                ),
                line("Latest request id", l.latest_id().to_string()),
                line("Responses applied", stats.applied.to_string()),
                line("Stale, discarded", stats.discarded.to_string()),
            ]
            .spacing(m.space_xs),
        )
        .padding(m.space_m)
        .width(Length::Fill)
        .into()
    }

    pub fn view(&self, m: Metrics) -> Element<'_, Msg> {
        let sp = m.space_m;
        let g = |t: &'static str| w::PreferencesGroup::<Msg>::new().title(t).metrics(m);

        let live = column![
            self.auto(Which::Live, &self.live, m)
                .placeholder("Search cities")
                .max_rows(6),
            ty::caption(
                "Type a city. \"fail\" errors every other time (Retry). \"san\" is slow \
                 (2.5 s): type it, wait for the spinner, then add a letter; the late answer is \
                 discarded."
            )
            .style(iad::styles::text::dim),
        ]
        .spacing(m.space_s);

        let strip = {
            let cell = |i: usize| -> Element<'_, Msg> {
                let (name, text_, lookup) = &self.strip[i];
                let open = self.open.as_deref() == Some(*name);
                let a = Autocomplete::new(text_, lookup, |_| Msg::Noop)
                    .on_retry(Msg::Noop)
                    .placeholder("Search cities")
                    .disabled(*name == "disabled")
                    .open_initially(open)
                    .metrics(m);
                column![
                    ty::caption(*name).style(iad::styles::text::dim),
                    Element::from(a)
                ]
                .spacing(m.space_xs)
                .width(Length::Fill)
                .into()
            };
            let r = |a: usize, b: usize| row![cell(a), cell(b)].spacing(sp);
            // The row with the open state goes first so its popover has room below.
            let mut rows: Vec<(usize, usize)> = vec![(0, 1), (2, 3), (4, 5)];
            if let Some(o) = &self.open
                && let Some(i) = self.strip.iter().position(|(n, ..)| n == o)
            {
                let at = rows.iter().position(|(a, b)| *a == i || *b == i).unwrap();
                let first = rows.remove(at);
                rows.insert(0, first);
            }
            padded_card(
                column(rows.into_iter().map(|(a, b)| r(a, b).into()))
                    .spacing(sp * 1.5)
                    .into(),
                sp,
            )
        };

        let interaction = iad::primitives::validation::Interaction {
            touched: self.touched,
            submitted: self.submitted,
        };
        let validity = self.field_validity();
        let field = column![
            Field::new(
                "City",
                self.auto(Which::Field, &self.field, m)
                    .placeholder("Pick a city")
                    .free_text(false)
                    .selected(self.field.picked.as_deref())
                    .revealed(interaction.revealed()),
            )
            .required(true)
            .description("Typing is not enough: choose one of the suggestions.")
            .validity(validity)
            .interaction(interaction)
            .metrics(m),
            row![
                cmp::suggested_button(ty::label("Submit")).on_press(Msg::Submit),
                cmp::flat_button(ty::label("Reset")).on_press(Msg::Reset),
            ]
            .spacing(sp),
        ]
        .spacing(sp);

        let live_g = g("Live")
            .description("Debounced 200 ms, results cached for the last 8 queries.")
            .push(padded_card(live.into(), sp));
        let readout_g = g("Lookup status")
            .description("The headless primitive behind it, as it changes.")
            .push(w::card(self.readout(m)));
        let strip_g = g("States")
            .description("Click an entry to open it. The last one is disabled.")
            .push(strip);
        let field_g = g("In a field")
            .description("Free text is off: the text must be a picked city.")
            .push(padded_card(field.into(), sp));
        // GALLERY_AUTOCOMPLETE puts the states first.
        if self.open.is_some() {
            column![strip_g, live_g, readout_g, field_g]
        } else {
            column![live_g, readout_g, strip_g, field_g]
        }
        .spacing(m.group_spacing)
        .into()
    }
}
