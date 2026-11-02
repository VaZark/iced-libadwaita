//! `DateInput`: the live calendar with an app-owned `CalendarView`, a month picker, a year
//! picker, and min/max across year boundaries.
//!
//! GALLERY_OPEN opens one popover straight into a level, for screenshots: `date`, `date-year`,
//! `date-decade`, `month-picker`, `year-picker`, `date-bounded`, `date-bounded-year`,
//! `date-bounded-decade`.

use iced_libadwaita as adw;

use adw::widget::{self as w, CalendarView, Date, Field, FieldLayout, Level, typography as ty};
use adw::{Element, Metrics};
use iced::widget::column;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Which {
    Live,
    Month,
    Year,
    Bounded,
}

#[derive(Debug, Clone)]
pub enum Msg {
    Text(Which, String),
    Pick(Which, Date),
    Nav(Which, CalendarView),
    Noop,
}

/// One input's state: the text, the date it holds and the calendar position, all owned here.
#[derive(Clone)]
struct Picker {
    text: String,
    pick: Option<Date>,
    nav: CalendarView,
    open: bool,
}

impl Picker {
    fn new(pick: Option<Date>, nav: CalendarView, open: bool) -> Self {
        Self {
            text: pick.map(|d| d.format()).unwrap_or_default(),
            pick,
            nav,
            open,
        }
    }
    fn text(&mut self, t: String) {
        if let Some(d) = Date::parse(&t) {
            self.pick = Some(d);
            self.nav = CalendarView::new(d, self.nav.level);
        }
        self.text = t;
    }
    fn pick(&mut self, d: Date) {
        self.pick = Some(d);
        self.text = d.format();
    }
}

pub struct State {
    live: Picker,
    month: Picker,
    year: Picker,
    bounded: Picker,
}

fn d(y: i32, m: u8, day: u8) -> Date {
    Date::new(y, m, day).unwrap()
}

/// The bounds of the cross-year demo.
const MIN: (i32, u8, u8) = (2025, 3, 10);
const MAX: (i32, u8, u8) = (2027, 6, 20);

fn level_of(open: &str, prefix: &str) -> Option<Level> {
    let rest = open.strip_prefix(prefix)?;
    match rest {
        "" => Some(Level::Month),
        "-year" => Some(Level::Year),
        "-decade" => Some(Level::Decade),
        _ => None,
    }
}

impl Default for State {
    fn default() -> Self {
        let open = std::env::var("GALLERY_OPEN").unwrap_or_default();
        let nov = d(2026, 11, 2);
        let live_level = level_of(&open, "date");
        let bounded_level = level_of(&open, "date-bounded");
        // `date` also prefixes `date-bounded`: the bounded demo is the longer name.
        let live_level = live_level.filter(|_| !open.starts_with("date-bounded"));
        let live = Picker::new(
            Some(nov),
            CalendarView::new(nov, live_level.unwrap_or_default()),
            live_level.is_some(),
        );
        let month = Picker::new(
            Some(d(2026, 3, 1)),
            CalendarView::new(nov, Level::Year),
            open == "month-picker",
        );
        let year = Picker::new(
            Some(d(2024, 1, 1)),
            CalendarView::new(nov, Level::Decade),
            open == "year-picker",
        );
        let bounded = Picker::new(
            Some(d(2026, 6, 15)),
            // Opened at the upper bound, where the later months, days and years are disabled.
            CalendarView::new(d(2027, 6, 15), bounded_level.unwrap_or_default()),
            bounded_level.is_some(),
        );
        Self {
            live,
            month,
            year,
            bounded,
        }
    }
}

impl State {
    fn of(&mut self, w: Which) -> &mut Picker {
        match w {
            Which::Live => &mut self.live,
            Which::Month => &mut self.month,
            Which::Year => &mut self.year,
            Which::Bounded => &mut self.bounded,
        }
    }

    pub fn update(&mut self, m: Msg) {
        match m {
            Msg::Text(w, t) => self.of(w).text(t),
            Msg::Pick(w, date) => self.of(w).pick(date),
            Msg::Nav(w, v) => self.of(w).nav = v,
            Msg::Noop => {}
        }
    }

    fn input<'a>(p: &'a Picker, which: Which, m: Metrics) -> w::DateInput<'a, Msg> {
        w::DateInput::new(&p.text, p.nav.date, move |t| Msg::Text(which, t))
            .calendar(p.nav)
            .selected(p.pick)
            .on_pick(move |date| Msg::Pick(which, date))
            .on_nav(move |v| Msg::Nav(which, v))
            .open_initially(p.open)
            .metrics(m)
    }

    pub fn view(&self, m: Metrics) -> Element<'_, Msg> {
        let sp = m.space_m;
        let group = |t: &'static str| w::PreferencesGroup::<Msg>::new().title(t).metrics(m);
        let live = Element::from(Self::input(&self.live, Which::Live, m));
        let min = d(MIN.0, MIN.1, MIN.2);
        let max = d(MAX.0, MAX.1, MAX.2);
        let bounded = Element::from(
            Self::input(&self.bounded, Which::Bounded, m)
                .min(min)
                .max(max),
        );
        let month = Element::from(
            Self::input(&self.month, Which::Month, m)
                .min_level(Level::Year)
                .max_level(Level::Decade),
        );
        let year = Element::from(
            Self::input(&self.year, Which::Year, m)
                .min_level(Level::Decade)
                .max_level(Level::Decade),
        );
        let ranged =
            w::DateInput::new("2026-10-15", d(2026, 11, 1), |_| Msg::Noop).min(d(2026, 11, 1));
        let invalid = w::DateInput::new("2026-02-30", d(2026, 11, 1), |_| Msg::Noop);
        let off = w::DateInput::new("2026-11-02", d(2026, 11, 1), |_| Msg::Noop).disabled(true);

        let controls = column![
            Field::new("Date", live)
                .description(format!(
                    "Calendar: {:?} level, {}",
                    self.live.nav.level,
                    self.live.nav.header_label()
                ))
                .layout(FieldLayout::Row)
                .plain_label(true)
                .metrics(m),
            Field::new("Between 2025-03-10 and 2027-06-20", bounded)
                .layout(FieldLayout::Row)
                .plain_label(true)
                .metrics(m),
            Field::new("Month picker (picking a month selects it)", month)
                .layout(FieldLayout::Row)
                .plain_label(true)
                .metrics(m),
            Field::new("Year picker (picking a year selects it)", year)
                .layout(FieldLayout::Row)
                .plain_label(true)
                .metrics(m),
            Field::new("Before the minimum", ranged)
                .layout(FieldLayout::Row)
                .plain_label(true)
                .metrics(m),
            Field::new("Not a date", invalid)
                .layout(FieldLayout::Row)
                .plain_label(true)
                .metrics(m),
            Field::new("Disabled", off)
                .layout(FieldLayout::Row)
                .plain_label(true)
                .metrics(m),
        ]
        .spacing(sp);
        // Relative time wording, from an explicit `now` (today here; the instants are offsets).
        // A name and its read-only value: a row field whose control is dimmed text.
        let value_row = |label: String, value: String| -> Element<'_, Msg> {
            Field::new(label, ty::body(value).style(adw::style::text::dim))
                .layout(FieldLayout::Row)
                .plain_label(true)
                .metrics(m)
                .into()
        };
        let today = Date::today();
        let day_rows = [0i64, -1, 1, -3, -12, -45, -400, 9].into_iter().map(|n| {
            let r = adw::primitives::Relative::from_dates(today.add_days(n), today);
            value_row(format!("{n:+} days"), r.to_string())
        });
        let second_rows = [-20i64, -300, -3 * 3600, 2 * 3600].into_iter().map(|n| {
            let r = adw::primitives::Relative::from_seconds(n);
            value_row(format!("{n:+} seconds"), r.to_string())
        });
        let relative = column(day_rows.chain(second_rows)).spacing(sp);
        column![
            group("Date input")
                .description("Type YYYY-MM-DD or pick. The title zooms out from days to months to years; picking a month or year drills back down; Today returns to the current month.")
                .push(w::card(controls)),
            group("Relative time")
                .description("Wording for a distance from now (primitives::Relative): words for today and its neighbours, then days, weeks, months and years; minutes and hours for instants.")
                .push(w::card(relative))
        ]
        .spacing(m.group_spacing)
        .into()
    }
}
