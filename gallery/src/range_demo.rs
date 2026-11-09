//! Multi-value inputs: the two-thumb `RangeSlider` (and its row) and the `DateRangeInput`, each
//! alone and wrapped in a `Field` with shared validation.

use iced_adaptive_design as iad;

use iad::primitives::calendar::{CalendarView, Level};
use iad::primitives::date::Date;
use iad::primitives::form::{FormSpec, FormState, Outcome};
use iad::primitives::numeric::{Decimal, Numeric};
use iad::primitives::range::{DateRange, Range, Track};
use iad::primitives::validation::Validity;
use iad::widgets::Field;
use iad::{Element, Metrics};
use iad::{components as cmp, widgets as w};
use iced::Length;
use iced::widget::{column, container, row};

#[derive(Debug, Clone)]
pub enum Msg {
    Basic((f32, f32)),
    Step((f32, f32)),
    Gap((f32, f32)),
    Push((f32, f32)),
    Marks((f32, f32)),
    Row((f32, f32)),
    Price((f32, f32)),
    Temp((f32, f32)),
    TempLo(String),
    TempHi(String),
    FieldRange((f32, f32)),
    Text(String),
    Pick(DateRange),
    Nav(CalendarView),
    Hover(Option<Date>),
    FieldText(String),
    FieldPick(DateRange),
    FieldNav(CalendarView),
    FieldHover(Option<Date>),
    BoundedText(String),
    BoundedPick(DateRange),
    BoundedNav(CalendarView),
    BoundedHover(Option<Date>),
    Submit,
    Reset,
    Noop,
}

/// A date range entry with its calendar state.
#[derive(Clone)]
struct Dates {
    text: String,
    range: DateRange,
    nav: CalendarView,
    hover: Option<Date>,
}

fn d(y: i32, m: u8, day: u8) -> Date {
    Date::new(y, m, day).unwrap()
}

impl Dates {
    fn new(start: Option<Date>, end: Option<Date>) -> Self {
        Self {
            text: DateRange::new(start, end).text(),
            range: DateRange::new(start, end),
            nav: CalendarView::month(d(2026, 11, 1)),
            hover: None,
        }
    }
    fn text(&mut self, t: String) {
        self.range = DateRange::parse(&t);
        if let Some(p) = self.range.start {
            self.nav = CalendarView::new(p, self.nav.level);
        }
        self.text = t;
    }
    fn pick(&mut self, r: DateRange) {
        *self = Self {
            text: r.text(),
            range: r,
            hover: self.hover,
            nav: self.nav,
        };
    }
}

pub struct State {
    basic: (f32, f32),
    step: (f32, f32),
    gap: (f32, f32),
    push: (f32, f32),
    marks: (f32, f32),
    row: (f32, f32),
    price: (f32, f32),
    temp: (f32, f32),
    temp_text: (String, String),
    field_range: (f32, f32),
    live: Dates,
    field_dates: Dates,
    bounded: Dates,
    /// What the user touched or submitted, for the two fields below.
    form: FormState<&'static str>,
}

/// The form's field keys: the slider field and the date range field.
const KEYS: [&str; 2] = ["range", "dates"];

fn spec() -> FormSpec<&'static str> {
    FormSpec::single(KEYS)
}

impl Default for State {
    fn default() -> Self {
        // GALLERY_OPEN=daterange: the live calendar opens with only a start chosen and the
        // pointer on a later day; daterange-full: with a range chosen.
        let open = std::env::var("GALLERY_OPEN").unwrap_or_default();
        let mut live = Dates::new(Some(d(2026, 11, 3)), Some(d(2026, 11, 9)));
        if open == "daterange" {
            live = Dates::new(Some(d(2026, 11, 10)), None);
            live.hover = Some(d(2026, 11, 14));
        }
        // daterange-year / daterange-decade: open at that level (the range is kept).
        if let Some(level) = match open.as_str() {
            "daterange-year" => Some(Level::Year),
            "daterange-decade" => Some(Level::Decade),
            _ => None,
        } {
            live.nav = CalendarView::new(d(2026, 11, 1), level);
        }
        // daterange-bounded[-year|-decade]: the min/max demo, across two year boundaries.
        let mut bounded = Dates::new(Some(d(2026, 2, 3)), Some(d(2026, 9, 9)));
        if let Some(level) = match open.as_str() {
            "daterange-bounded" => Some(Level::Month),
            "daterange-bounded-year" => Some(Level::Year),
            "daterange-bounded-decade" => Some(Level::Decade),
            _ => None,
        } {
            bounded.nav = CalendarView::new(d(2026, 2, 1), level);
        }
        Self {
            basic: (20.0, 70.0),
            step: (25.0, 75.0),
            gap: (30.0, 50.0),
            push: (40.0, 70.0),
            marks: (10.0, 40.0),
            row: (200.0, 800.0),
            price: (12.0, 48.0),
            temp: (18.0, 26.5),
            temp_text: ("18.0".into(), "26.5".into()),
            field_range: (45.0, 55.0),
            live,
            field_dates: Dates::new(Some(d(2026, 11, 12)), Some(d(2026, 11, 8))),
            bounded,
            form: FormState::new(),
        }
    }
}

impl State {
    pub fn update(&mut self, m: Msg) {
        match m {
            Msg::Basic(v) => self.basic = v,
            Msg::Step(v) => self.step = v,
            Msg::Gap(v) => self.gap = v,
            Msg::Push(v) => self.push = v,
            Msg::Marks(v) => self.marks = v,
            Msg::Row(v) => self.row = v,
            Msg::Price(v) => self.price = v,
            Msg::Temp(v) => {
                self.temp = v;
                self.temp_text = (temp_text(v.0), temp_text(v.1));
            }
            Msg::TempLo(t) => {
                if let Some(v) = temp_value(&t) {
                    self.temp.0 = v.min(self.temp.1);
                }
                self.temp_text.0 = t;
            }
            Msg::TempHi(t) => {
                if let Some(v) = temp_value(&t) {
                    self.temp.1 = v.max(self.temp.0);
                }
                self.temp_text.1 = t;
            }
            Msg::FieldRange(v) => {
                self.field_range = v;
                self.touch("range");
            }
            Msg::Text(t) => self.live.text(t),
            Msg::Pick(r) => self.live.pick(r),
            Msg::Nav(v) => self.live.nav = v,
            Msg::Hover(h) => self.live.hover = h,
            Msg::FieldText(t) => {
                self.field_dates.text(t);
                self.touch("dates");
            }
            Msg::FieldPick(r) => {
                self.field_dates.pick(r);
                self.touch("dates");
            }
            Msg::FieldNav(v) => self.field_dates.nav = v,
            Msg::FieldHover(h) => self.field_dates.hover = h,
            Msg::BoundedText(t) => self.bounded.text(t),
            Msg::BoundedPick(r) => self.bounded.pick(r),
            Msg::BoundedNav(v) => self.bounded.nav = v,
            Msg::BoundedHover(h) => self.bounded.hover = h,
            Msg::Submit => {
                let mut form = std::mem::take(&mut self.form);
                let outcome = form.submit(&spec(), &|k| self.validity(k));
                self.form = form;
                match outcome {
                    // Every error now shows; a real app would also send the values here.
                    Outcome::Submitted => {}
                    // A real app would focus `key` (the toolkit entries take no widget id yet).
                    Outcome::Blocked { .. } | Outcome::Moved { .. } | Outcome::Ignored => {}
                }
            }
            Msg::Reset => {
                // The form forgets what was touched; the app resets its own values.
                self.form.reset();
                let form = std::mem::take(&mut self.form);
                *self = Self {
                    form,
                    ..Self::default()
                };
            }
            Msg::Noop => {}
        }
    }

    fn touch(&mut self, k: &'static str) {
        self.form.touch(k);
    }

    /// The validity of the field `key`, from the app's values.
    fn validity(&self, key: &&'static str) -> Validity {
        match *key {
            "range" => Track::new(0.0f32, 100.0)
                .min_gap(30.0)
                .validity(Range::from(self.field_range)),
            "dates" => Self::dates_input(&self.field_dates, Metrics::desktop()).validity(),
            _ => Validity::Ok,
        }
    }

    /// The date range of the "In a field" demo: strict, so the end must be after the start.
    fn dates_input(f: &Dates, m: Metrics) -> w::DateRangeInput<'_, Msg> {
        Self::live_input(f, m, false, Msg::FieldText)
            .strict(true)
            .on_pick(Msg::FieldPick)
            .on_nav(Msg::FieldNav)
            .on_hover(Msg::FieldHover)
    }

    fn group<'a>(title: &'static str, m: Metrics) -> w::PreferencesGroup<'a, Msg> {
        w::PreferencesGroup::<Msg>::new().title(title).metrics(m)
    }
}

/// The numeric rule of the temperature range: one decimal, 0 to 40, half-degree steps.
fn temp_rule() -> Numeric {
    Numeric::decimal(1).min(0).max(40).step(Decimal::new(5, 1))
}

fn temp_text(v: f32) -> String {
    temp_rule().format(Decimal::new((v * 10.0).round() as i64, 1))
}

/// The slider value of a typed temperature, `None` for text that is not one or out of range.
fn temp_value(text: &str) -> Option<f32> {
    let n = temp_rule();
    n.validate(text).is_ok().then_some(())?;
    n.parse(text).map(|d| d.units as f32 / 10.0)
}

fn pair((lo, hi): (f32, f32)) -> String {
    format!("{lo:.0} \u{2013} {hi:.0}")
}

impl State {
    /// The range slider demos.
    pub fn slider_view(&self, m: Metrics) -> Element<'_, Msg> {
        let sp = m.space_m;
        let rs = |r: std::ops::RangeInclusive<f32>, v, f: fn((f32, f32)) -> Msg| {
            w::range_slider(r, v, f)
        };
        let sliders = column![
            Field::new(
                format!("Vertical: {}", pair(self.basic)),
                rs(0.0..=100.0, self.basic, Msg::Basic)
                    .orientation(iad::primitives::scrub::Axis::Vertical)
                    .height(180.0)
                    .marks_every(20.0)
            )
            .plain_label(true)
            .metrics(m),
            Field::new(
                format!("Basic: {}", pair(self.basic)),
                rs(0.0..=100.0, self.basic, Msg::Basic)
            )
            .plain_label(true)
            .metrics(m),
            Field::new(
                format!("Step 5: {}", pair(self.step)),
                rs(0.0..=100.0, self.step, Msg::Step).step(5.0)
            )
            .plain_label(true)
            .metrics(m),
            Field::new(
                format!("Minimum gap 20, thumbs stop: {}", pair(self.gap)),
                rs(0.0..=100.0, self.gap, Msg::Gap).step(5.0).min_gap(20.0)
            )
            .plain_label(true)
            .metrics(m),
            Field::new(
                format!("Minimum gap 20, thumbs push: {}", pair(self.push)),
                rs(0.0..=100.0, self.push, Msg::Push)
                    .step(5.0)
                    .min_gap(20.0)
                    .push_thumbs(true)
            )
            .plain_label(true)
            .metrics(m),
            Field::new(
                format!("Marks every 10: {}", pair(self.marks)),
                rs(0.0..=50.0, self.marks, Msg::Marks)
                    .step(5.0)
                    .marks_every(10.0)
            )
            .plain_label(true)
            .metrics(m),
            Field::new(
                "Disabled",
                rs(0.0..=100.0, (25.0, 75.0), |_| Msg::Noop).disabled(true)
            )
            .plain_label(true)
            .metrics(m),
            Field::new(
                "Disabled, with marks",
                rs(0.0..=50.0, (10.0, 40.0), |_| Msg::Noop)
                    .step(5.0)
                    .marks_every(10.0)
                    .disabled(true)
            )
            .plain_label(true)
            .metrics(m),
        ]
        .spacing(sp);

        let rows = Self::group("Range slider row", m)
            .description("Wraps below the title when it does not fit, like the slider row.")
            .push(
                w::RangeSliderRow::new("Budget", self.row, 0.0..=1000.0, Msg::Row)
                    .subtitle("Per month")
                    .step(50.0)
                    .metrics(m),
            )
            .push(
                w::RangeSliderRow::new("Price", self.price, 0.0..=60.0, Msg::Price)
                    .step(1.0)
                    .format(|v| format!("${v:.0}"))
                    .min_gap(5.0)
                    .metrics(m),
            )
            .push(
                w::RangeSliderRow::new("Volume", (20.0, 60.0), 0.0..=100.0, |_| Msg::Noop)
                    .subtitle("Disabled")
                    .disabled(true)
                    .metrics(m),
            );

        let numbers = Self::group("With number inputs", m)
            .description("The handles step by half a degree; the fixed-point input keeps one decimal and the slider follows what is typed.")
            .push(w::card(
                column![
                    w::range_slider(0.0..=40.0, self.temp, Msg::Temp).step(0.5),
                    row![
                        w::NumberInput::new(&self.temp_text.0, Msg::TempLo)
                            .numeric(temp_rule())
                            .metrics(m),
                        w::NumberInput::new(&self.temp_text.1, Msg::TempHi)
                            .numeric(temp_rule())
                            .metrics(m),
                    ]
                    .spacing(m.space_m),
                ]
                .spacing(m.space_m),
            ));

        let spec = spec();
        let field = Field::new(
            "Allowed volume",
            w::range_slider(0.0..=100.0, self.field_range, Msg::FieldRange),
        )
        .description(format!("At least 30 wide: {}", pair(self.field_range)))
        .validity(self.validity(&"range"))
        .interaction(self.form.interaction(&spec, &"range"))
        .inline(true)
        .metrics(m);

        column![
            Self::group("Range slider", m)
                .description("Two thumbs on one rail. Drag a thumb or click the rail; arrows, Page and Home/End when focused.")
                .push(w::card(sliders)),
            rows,
            numbers,
            Self::group("In a field", m)
                .description("The field draws the one ring and the message; the slider has no state of its own.")
                .push(w::card(field)),
        ]
        .spacing(m.group_spacing)
        .into()
    }

    fn live_input<'a>(
        t: &'a Dates,
        m: Metrics,
        open: bool,
        wrap: impl Fn(String) -> Msg + 'a,
    ) -> w::DateRangeInput<'a, Msg> {
        w::DateRangeInput::new(&t.text, t.nav.date, wrap)
            .calendar(t.nav)
            .selected(t.range)
            .hover(t.hover)
            .open_initially(open)
            .metrics(m)
    }

    /// The date range demos.
    pub fn date_view(&self, m: Metrics) -> Element<'_, Msg> {
        let sp = m.space_m;
        let open_env = std::env::var("GALLERY_OPEN").unwrap_or_default();
        let open_bounded = open_env.starts_with("daterange-bounded");
        let open = open_env.starts_with("daterange") && !open_bounded;
        let live = Self::live_input(&self.live, m, open, Msg::Text)
            .on_pick(Msg::Pick)
            .on_nav(Msg::Nav)
            .on_hover(Msg::Hover);

        let fixed = |text: &'static str| {
            w::DateRangeInput::new(text, d(2026, 11, 1), |_| Msg::Noop)
                .selected(DateRange::parse(text))
                .metrics(m)
        };
        let min = d(2026, 11, 1);
        let bounded = Self::live_input(&self.bounded, m, open_bounded, Msg::BoundedText)
            .min(d(2025, 3, 10))
            .max(d(2027, 6, 20))
            .on_pick(Msg::BoundedPick)
            .on_nav(Msg::BoundedNav)
            .on_hover(Msg::BoundedHover);
        let controls = column![
            Field::new("Date range", live).plain_label(true).metrics(m),
            Field::new(
                "Between 2025-03-10 and 2027-06-20 (months and years outside are disabled)",
                bounded
            )
            .plain_label(true)
            .metrics(m),
            Field::new("Empty", fixed("")).plain_label(true).metrics(m),
            Field::new("Start chosen", fixed("2026-11-10"))
                .plain_label(true)
                .metrics(m),
            Field::new("Selected", fixed("2026-11-03 \u{2013} 2026-11-09"))
                .plain_label(true)
                .metrics(m),
            Field::new(
                "Minimum 2026-11-01 (the first is before it)",
                fixed("2026-10-28 \u{2013} 2026-11-09").min(min)
            )
            .plain_label(true)
            .metrics(m),
            Field::new("End before start", fixed("2026-11-09 \u{2013} 2026-11-02"))
                .plain_label(true)
                .metrics(m),
            Field::new(
                "Disabled",
                fixed("2026-11-03 \u{2013} 2026-11-09").disabled(true)
            )
            .plain_label(true)
            .metrics(m),
        ]
        .spacing(sp);

        // In a Field: validity from the primitive, revealed once touched or submitted.
        let f = &self.field_dates;
        let spec = spec();
        let input = Self::dates_input(f, m).revealed(self.form.revealed(&spec, &"dates"));
        let field = Field::new("Stay", input)
            .description("From check-in to check-out.")
            .required(true)
            .validity(self.validity(&"dates"))
            .interaction(self.form.interaction(&spec, &"dates"))
            .metrics(m);
        let buttons = row![
            container(cmp::flat_button("Reset").on_press(Msg::Reset)),
            cmp::suggested_button("Submit").on_press(Msg::Submit),
        ]
        .spacing(sp);

        column![
            Self::group("Date range input", m)
                .description("Type YYYY-MM-DD or pick: the first day sets the start, the second the end, a third starts over.")
                .push(w::card(controls)),
            Self::group("In a field", m).push(w::card(
                column![field, buttons].spacing(sp).width(Length::Fill)
            )),
        ]
        .spacing(m.group_spacing)
        .into()
    }
}
