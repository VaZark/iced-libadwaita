//! Multi-value inputs: the two-thumb `RangeSlider` (and its row) and the `DateRangeInput`, each
//! alone and wrapped in a `Field` with shared validation.

use iced_libadwaita as adw;

use adw::primitives::range::{DateRange, Edge, Range, Track};
use adw::widget::{self as w, CalendarView, Date, Field, Level, typography as ty};
use adw::{Element, Metrics};
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
    FieldRange((f32, f32)),
    Text(Edge, String),
    Pick(DateRange),
    Nav(CalendarView),
    Hover(Option<Date>),
    FieldText(Edge, String),
    FieldPick(DateRange),
    FieldNav(CalendarView),
    FieldHover(Option<Date>),
    BoundedText(Edge, String),
    BoundedPick(DateRange),
    BoundedNav(CalendarView),
    BoundedHover(Option<Date>),
    Submit,
    Reset,
    Noop,
}

/// A date entry pair with its calendar state.
#[derive(Clone)]
struct Dates {
    start: String,
    end: String,
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
            start: start.map(|d| d.format()).unwrap_or_default(),
            end: end.map(|d| d.format()).unwrap_or_default(),
            range: DateRange::new(start, end),
            nav: CalendarView::month(d(2026, 11, 1)),
            hover: None,
        }
    }
    fn text(&mut self, edge: Edge, t: String) {
        let parsed = Date::parse(&t);
        self.range = self.range.with(edge, parsed);
        if let Some(p) = parsed {
            self.nav = CalendarView::new(p, self.nav.level);
        }
        match edge {
            Edge::Start => self.start = t,
            Edge::End => self.end = t,
        }
    }
    fn pick(&mut self, r: DateRange) {
        *self = Self {
            start: r.start.map(|d| d.format()).unwrap_or_default(),
            end: r.end.map(|d| d.format()).unwrap_or_default(),
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
    field_range: (f32, f32),
    live: Dates,
    field_dates: Dates,
    bounded: Dates,
    submitted: bool,
    touched: Vec<&'static str>,
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
            field_range: (45.0, 55.0),
            live,
            field_dates: Dates::new(Some(d(2026, 11, 12)), Some(d(2026, 11, 8))),
            bounded,
            submitted: false,
            touched: Vec::new(),
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
            Msg::FieldRange(v) => {
                self.field_range = v;
                self.touch("range");
            }
            Msg::Text(e, t) => self.live.text(e, t),
            Msg::Pick(r) => self.live.pick(r),
            Msg::Nav(v) => self.live.nav = v,
            Msg::Hover(h) => self.live.hover = h,
            Msg::FieldText(e, t) => {
                self.field_dates.text(e, t);
                self.touch("dates");
            }
            Msg::FieldPick(r) => {
                self.field_dates.pick(r);
                self.touch("dates");
            }
            Msg::FieldNav(v) => self.field_dates.nav = v,
            Msg::FieldHover(h) => self.field_dates.hover = h,
            Msg::BoundedText(e, t) => self.bounded.text(e, t),
            Msg::BoundedPick(r) => self.bounded.pick(r),
            Msg::BoundedNav(v) => self.bounded.nav = v,
            Msg::BoundedHover(h) => self.bounded.hover = h,
            Msg::Submit => self.submitted = true,
            Msg::Reset => *self = Self::default(),
            Msg::Noop => {}
        }
    }

    fn touch(&mut self, k: &'static str) {
        if !self.touched.contains(&k) {
            self.touched.push(k);
        }
    }

    fn group<'a>(title: &'static str, m: Metrics) -> w::PreferencesGroup<'a, Msg> {
        w::PreferencesGroup::<Msg>::new().title(title).metrics(m)
    }
}

fn item<'a>(label: String, control: impl Into<Element<'a, Msg>>) -> Element<'a, Msg> {
    column![
        ty::caption(label).style(adw::style::text::dim),
        control.into()
    ]
    .spacing(4)
    .into()
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
            item(
                format!("Basic: {}", pair(self.basic)),
                rs(0.0..=100.0, self.basic, Msg::Basic)
            ),
            item(
                format!("Step 5: {}", pair(self.step)),
                rs(0.0..=100.0, self.step, Msg::Step).step(5.0)
            ),
            item(
                format!("Minimum gap 20, thumbs stop: {}", pair(self.gap)),
                rs(0.0..=100.0, self.gap, Msg::Gap).step(5.0).min_gap(20.0)
            ),
            item(
                format!("Minimum gap 20, thumbs push: {}", pair(self.push)),
                rs(0.0..=100.0, self.push, Msg::Push)
                    .step(5.0)
                    .min_gap(20.0)
                    .push_thumbs(true)
            ),
            item(
                format!("Marks every 10: {}", pair(self.marks)),
                rs(0.0..=50.0, self.marks, Msg::Marks)
                    .step(5.0)
                    .marks_every(10.0)
            ),
            item(
                "Disabled".into(),
                rs(0.0..=100.0, (25.0, 75.0), |_| Msg::Noop).disabled(true)
            ),
            item(
                "Disabled, with marks".into(),
                rs(0.0..=50.0, (10.0, 40.0), |_| Msg::Noop)
                    .step(5.0)
                    .marks_every(10.0)
                    .disabled(true)
            ),
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

        let track = Track::new(0.0f32, 100.0).min_gap(30.0);
        let span = Range::from(self.field_range);
        let validity = track.validity(span);
        let touched = self.t("range");
        let field = Field::new(
            "Allowed volume",
            w::range_slider(0.0..=100.0, self.field_range, Msg::FieldRange),
        )
        .description(format!("At least 30 wide: {}", pair(self.field_range)))
        .validity(validity)
        .touched(touched)
        .submitted(self.submitted)
        .inline(true)
        .metrics(m);

        column![
            Self::group("Range slider", m)
                .description("Two thumbs on one rail. Drag a thumb or click the rail; arrows, Page and Home/End when focused.")
                .push(w::card(sliders)),
            rows,
            Self::group("In a field", m)
                .description("The field draws the one ring and the message; the slider has no state of its own.")
                .push(w::card(field)),
        ]
        .spacing(m.group_spacing)
        .into()
    }

    fn t(&self, k: &'static str) -> bool {
        self.touched.contains(&k)
    }

    fn live_input<'a>(
        t: &'a Dates,
        m: Metrics,
        open: bool,
        wrap: impl Fn(Edge, String) -> Msg + 'a,
    ) -> w::DateRangeInput<'a, Msg> {
        w::DateRangeInput::new(&t.start, &t.end, t.nav.date, wrap)
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

        let fixed = |a: &'static str, b: &'static str| {
            w::DateRangeInput::new(a, b, d(2026, 11, 1), |_, _| Msg::Noop)
                .selected(DateRange::new(Date::parse(a), Date::parse(b)))
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
            item("Date range".into(), live),
            item(
                "Between 2025-03-10 and 2027-06-20 (months and years outside are disabled)".into(),
                bounded
            ),
            item("Empty".into(), fixed("", "")),
            item("Start chosen".into(), fixed("2026-11-10", "")),
            item("Selected".into(), fixed("2026-11-03", "2026-11-09")),
            item(
                "Minimum 2026-11-01 (the first is before it)".into(),
                fixed("2026-10-28", "2026-11-09").min(min)
            ),
            item("End before start".into(), fixed("2026-11-09", "2026-11-02")),
            item(
                "Disabled".into(),
                fixed("2026-11-03", "2026-11-09").disabled(true)
            ),
        ]
        .spacing(sp);

        // In a Field: validity from the primitive, revealed once touched or submitted.
        let f = &self.field_dates;
        let revealed = self.t("dates") || self.submitted;
        let input = Self::live_input(f, m, false, Msg::FieldText)
            .revealed(revealed)
            .strict(true)
            .on_pick(Msg::FieldPick)
            .on_nav(Msg::FieldNav)
            .on_hover(Msg::FieldHover);
        let validity = input.validity();
        let field = Field::new("Stay", input)
            .description("From check-in to check-out.")
            .required(true)
            .validity(validity)
            .touched(self.t("dates"))
            .submitted(self.submitted)
            .metrics(m);
        let buttons = row![
            container(w::flat_button("Reset").on_press(Msg::Reset)),
            w::suggested_button("Submit").on_press(Msg::Submit),
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
