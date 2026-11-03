//! `DateRangeInput`: one entry for a date range (`2026-11-02 – 2026-11-09`) with the calendar in
//! its left section, in a popover or a dialog, that selects the range; the sibling of [`DateInput`](super::DateInput) the way
//! [`RangeSlider`](super::RangeSlider) is of [`scale`](super::scale).
//!
//! The rule is [`DateRange::pick`](crate::primitives::range::DateRange::pick): the first click
//! sets the start, the second the end (swapped when earlier), a third starts over. Everything
//! is controlled like `DateInput`: you keep the text, the month on show and the
//! [`DateRange`] (set the text from [`DateRange::text`] when a range is picked), and the widget
//! reports `on_input(text)`, `on_pick(range)`,
//! `on_view(first-of-month)` and, to preview the range under the pointer, `on_hover(day)`.
//!
//! ```ignore
//! DateRangeInput::new(&self.text, self.view, Message::Text)
//!     .selected(self.range)
//!     .hover(self.hover)
//!     .on_pick(Message::Picked)
//!     .on_view(Message::View)
//!     .on_hover(Message::Hover)
//! ```
//!
//! An unparsable or out-of-range date, or an end before the start, draws the entry in its error
//! state, once [`revealed`](DateRangeInput::revealed) (pass the `Field`'s interaction inside a
//! [`Field`](super::Field)); [`validity`](DateRangeInput::validity) is the message for it.

use super::date_input::{CELL, Cal, OnNav, OnView, Opening, calendar, nav_message, opener};
use super::popover::popover;
use super::{Element, EntryState, entry};
use crate::Metrics;
use crate::Theme;
use crate::primitives::calendar::{CalendarView, Level};
use crate::primitives::date::{self, Date};
use crate::primitives::range::{DateRange, DayRole, Edge, within};
use crate::primitives::validation::{self, Presentation, Validity};
use iced::widget::button::Status;
use iced::widget::{button, container, mouse_area, text};
use iced::{Background, Border, Length, Shadow};

pub struct DateRangeInput<'a, M> {
    text: &'a str,
    view: Date,
    on_input: Option<Box<dyn Fn(String) -> M + 'a>>,
    on_pick: Option<Box<dyn Fn(DateRange) -> M + 'a>>,
    on_view: Option<OnView<'a, M>>,
    on_nav: Option<OnNav<'a, M>>,
    level: Level,
    max_level: Level,
    today: Date,
    on_hover: Option<Box<dyn Fn(Option<Date>) -> M + 'a>>,
    selected: DateRange,
    hover: Option<Date>,
    min: Option<Date>,
    max: Option<Date>,
    strict: bool,
    disabled: bool,
    width: Length,
    open_initially: bool,
    dialog: bool,
    scrim: bool,
    close_on_outside: bool,
    window_radius: f32,
    revealed: bool,
    metrics: Metrics,
}

impl<'a, M> DateRangeInput<'a, M> {
    /// `text` is the entry's content (`2026-11-02 – 2026-11-09`, see [`DateRange::parse`]);
    /// `view` is any date in the month the calendar shows.
    pub fn new(text: &'a str, view: Date, on_input: impl Fn(String) -> M + 'a) -> Self {
        Self {
            text,
            view: view.month_start(),
            on_input: Some(Box::new(on_input)),
            on_pick: None,
            on_view: None,
            on_nav: None,
            level: Level::Month,
            max_level: Level::Decade,
            today: Date::today(),
            on_hover: None,
            selected: DateRange::default(),
            hover: None,
            min: None,
            max: None,
            strict: false,
            disabled: false,
            width: Length::Fixed(300.0),
            open_initially: false,
            dialog: false,
            scrim: true,
            close_on_outside: true,
            window_radius: Metrics::desktop().radius_window,
            revealed: true,
            metrics: Metrics::desktop(),
        }
    }
    /// The chosen range `(start, end)`, highlighted in the calendar.
    pub fn selected(mut self, r: impl Into<DateRange>) -> Self {
        self.selected = r.into();
        self
    }
    /// The day under the pointer, for the preview while only the start is chosen.
    pub fn hover(mut self, d: Option<Date>) -> Self {
        self.hover = d;
        self
    }
    /// A day was clicked: gets the new selection ([`DateRange::pick`]).
    pub fn on_pick(mut self, f: impl Fn(DateRange) -> M + 'a) -> Self {
        self.on_pick = Some(Box::new(f));
        self
    }
    /// The previous / next month button was pressed; gets the first day of the new month.
    /// The month-level compatibility path: called only while months show, and the title is
    /// not navigable. Use [`on_nav`](Self::on_nav) for the levels (it takes precedence).
    pub fn on_view(mut self, f: impl Fn(Date) -> M + 'a) -> Self {
        self.on_view = Some(Box::new(f));
        self
    }
    /// The calendar moved (arrow, title, month or year cell, "Today"): gets the whole new
    /// [`CalendarView`]. Drilling down never changes the selection: days are picked only at
    /// the `Month` level, so the range rule ([`DateRange::pick`]) is untouched by zooming.
    pub fn on_nav(mut self, f: impl Fn(CalendarView) -> M + 'a) -> Self {
        self.on_nav = Some(Box::new(f));
        self
    }
    /// The level on show (default `Month`); see [`DateInput::level`](super::DateInput::level).
    pub fn level(mut self, l: Level) -> Self {
        self.level = l;
        self
    }
    /// The calendar position as one value.
    pub fn calendar(mut self, v: CalendarView) -> Self {
        self.view = v.date;
        self.level = v.level;
        self
    }
    /// The coarsest level the title can zoom out to (default `Decade`). A range is always
    /// picked in days, so there is no `min_level`.
    pub fn max_level(mut self, l: Level) -> Self {
        self.max_level = l;
        self
    }
    /// The date "Today" jumps to and that is marked (default: the system clock, UTC).
    pub fn today(mut self, d: Date) -> Self {
        self.today = d;
        self
    }
    /// The pointer entered a day (`Some`) or left it (`None`); keep it and pass it to
    /// [`hover`](Self::hover). Only sent while a start is chosen and the end is not.
    pub fn on_hover(mut self, f: impl Fn(Option<Date>) -> M + 'a) -> Self {
        self.on_hover = Some(Box::new(f));
        self
    }
    pub fn min(mut self, d: Date) -> Self {
        self.min = Some(d);
        self
    }
    pub fn max(mut self, d: Date) -> Self {
        self.max = Some(d);
        self
    }
    /// The end must be a later day than the start (the same day is an error); default off, a
    /// range of one day is fine.
    pub fn strict(mut self, strict: bool) -> Self {
        self.strict = strict;
        self
    }
    /// Whether validation may show yet; see [`DateInput::revealed`](super::DateInput::revealed).
    pub fn revealed(mut self, revealed: bool) -> Self {
        self.revealed = revealed;
        self
    }
    /// Drawn dimmed; cannot be edited or opened.
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    pub fn open_initially(mut self, open: bool) -> Self {
        self.open_initially = open;
        self
    }
    /// Opens the calendar as a dialog centered in the window instead of a popover.
    pub fn dialog(mut self, on: bool) -> Self {
        self.dialog = on;
        self
    }
    /// Fades the window behind the dialog (default on); only a [`dialog`](Self::dialog) has it.
    pub fn scrim(mut self, on: bool) -> Self {
        self.scrim = on;
        self
    }
    /// Whether a press outside the dialog closes it (default true).
    pub fn close_on_outside(mut self, on: bool) -> Self {
        self.close_on_outside = on;
        self
    }
    /// Rounds the dialog's fade to the window's corners (`window::State::radius`).
    pub fn window_radius(mut self, r: f32) -> Self {
        self.window_radius = r;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }

    /// The range the text spells, a side that does not parse being `None`.
    pub fn typed(&self) -> DateRange {
        DateRange::parse(self.text)
    }

    /// Which entry a problem belongs to, with its message: an unparsable or out-of-range text
    /// on its own entry, an end before the start on the end.
    fn problem(&self) -> Option<(Edge, Validity)> {
        let (a, b) = DateRange::split_text(self.text);
        for (edge, t) in [(Edge::Start, a), (Edge::End, b)] {
            let v = date::validate(t, self.min, self.max);
            if v.blocks_submit() {
                return Some((edge, v));
            }
        }
        let v = self.typed().validity(self.min, self.max, self.strict);
        v.blocks_submit().then_some((Edge::End, v))
    }

    /// The validity of the typed texts: the first problem, or `Ok`.
    pub fn validity(&self) -> Validity {
        self.problem().map_or(Validity::Ok, |(_, v)| v)
    }

    /// The entry's state: an error only once [`revealed`](Self::revealed).
    pub fn text_state(&self) -> EntryState {
        match self.problem() {
            Some((_, v)) if validation::present(&v, self.revealed) == Presentation::Invalid => {
                EntryState::Error
            }
            _ => EntryState::Normal,
        }
    }
}

/// A day cell's look: the accent band for the range, rounded at its ends.
fn day_style(
    role: DayRole,
    enabled: bool,
) -> impl Fn(&Theme, Status) -> iced::widget::button::Style {
    move |t: &Theme, status| {
        let s = &t.semantic;
        let r = t.metrics.radius_menu;
        let radius = match role {
            DayRole::Start => iced::border::Radius {
                top_left: r,
                bottom_left: r,
                top_right: 0.0,
                bottom_right: 0.0,
            },
            DayRole::End => iced::border::Radius {
                top_left: 0.0,
                bottom_left: 0.0,
                top_right: r,
                bottom_right: r,
            },
            DayRole::Inside => 0.0.into(),
            DayRole::Single | DayRole::Outside => r.into(),
        };
        let hover = matches!(status, Status::Hovered | Status::Pressed);
        let (bg, fg) = match role {
            DayRole::Start | DayRole::End | DayRole::Single => (
                Some(if hover {
                    s.hover_fill(s.accent.bg)
                } else {
                    s.accent.bg
                }),
                s.accent.fg,
            ),
            DayRole::Inside => (
                Some(crate::color::alpha(
                    s.accent.color,
                    if hover { 0.26 } else { 0.16 },
                )),
                s.accent.color,
            ),
            DayRole::Outside => (
                hover.then(|| crate::color::alpha(s.popover.fg, 0.10)),
                s.popover.fg,
            ),
        };
        let fg = if enabled {
            fg
        } else {
            crate::color::fade(fg, s.opacity.disabled)
        };
        iced::widget::button::Style {
            background: bg.map(Background::Color),
            text_color: fg,
            border: Border {
                radius,
                ..Border::default()
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

impl<'a, M: Clone + 'a> From<DateRangeInput<'a, M>> for Element<'a, M> {
    fn from(mut d: DateRangeInput<'a, M>) -> Self {
        let m = d.metrics;
        let mut field = entry::<M>("YYYY-MM-DD \u{2013} YYYY-MM-DD", d.text)
            .state(d.text_state())
            .metrics(m);
        if !d.disabled
            && let Some(f) = d.on_input.take()
        {
            field = field.on_input(f);
        }

        let (selected, hover, min, max) = (d.selected, d.hover, d.min, d.max);
        let previewing = selected.start.is_some() && selected.end.is_none();
        let view = CalendarView::new(d.view, d.level).clamped(Level::Month, d.max_level);
        let nav = nav_message(&d.on_nav, &d.on_view);
        let span = match (selected.start, selected.end) {
            (Some(a), Some(b)) => Some((a.min(b), a.max(b))),
            (Some(a), None) | (None, Some(a)) => Some((a, a)),
            _ => None,
        };
        let no_select = |_: Date| None;
        let cal = Cal {
            view,
            levels: (Level::Month, d.max_level),
            today: d.today,
            span,
            min,
            max,
            band: true,
            metrics: m,
            navigable: d.on_nav.is_some(),
            nav: &nav,
            select: &no_select,
        };
        let grid = calendar(cal, |date| {
            let ok = within(date, min, max);
            let role = selected.role(date, hover, min, max);
            let mut b = button(
                container(text(date.day.to_string()))
                    .center_x(Length::Fixed(CELL))
                    .center_y(Length::Fixed(CELL - 4.0)),
            )
            .padding(0)
            .style(day_style(role, ok));
            if ok
                && !d.disabled
                && let Some(f) = &d.on_pick
            {
                b = b.on_press(f(selected.pick(date, min, max)));
            }
            let cell: Element<'a, M> = b.into();
            match &d.on_hover {
                Some(h) if previewing && ok => mouse_area(cell)
                    .on_enter(h(Some(date)))
                    .on_exit(h(None))
                    .into(),
                _ => cell,
            }
        });

        let opener = opener(
            popover([grid], 400.0),
            Opening {
                dialog: d.dialog,
                scrim: d.scrim,
                close_on_outside: d.close_on_outside,
                window_radius: d.window_radius,
                disabled: d.disabled,
                open_initially: d.open_initially,
            },
            m,
        );
        field.left_section(opener).width(d.width).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u8, dd: u8) -> Date {
        Date::new(y, m, dd).unwrap()
    }
    fn mk(text: &str) -> DateRangeInput<'_, ()> {
        DateRangeInput::new(text, d(2026, 11, 1), |_| ())
    }

    #[test]
    fn the_texts_spell_a_range() {
        assert_eq!(
            mk("2026-11-02 \u{2013} 2026-11-09").typed(),
            DateRange::new(Some(d(2026, 11, 2)), Some(d(2026, 11, 9)))
        );
        assert_eq!(mk("2026-11-02").typed().end, None);
    }

    #[test]
    fn a_problem_in_either_date_or_their_order_is_an_error() {
        use EntryState::*;
        assert_eq!(mk("").text_state(), Normal);
        assert_eq!(mk("2026-11-02 \u{2013} 2026-11-09").text_state(), Normal);
        assert_eq!(mk("2026-13-02 \u{2013} 2026-11-09").text_state(), Error);
        assert_eq!(mk("2026-11-02 \u{2013} nope").text_state(), Error);
        assert_eq!(mk("2026-11-09 \u{2013} 2026-11-02").text_state(), Error);
        assert_eq!(
            mk("2026-11-09 \u{2013} 2026-11-02")
                .revealed(false)
                .text_state(),
            Normal,
            "quiet until revealed"
        );
    }

    #[test]
    fn the_end_must_follow_the_start() {
        assert_eq!(
            mk("2026-11-09 \u{2013} 2026-11-02").validity(),
            Validity::error("End must not be before start.")
        );
        assert!(mk("2026-11-02 \u{2013} 2026-11-02").validity().is_ok());
        assert!(
            mk("2026-11-02 \u{2013} 2026-11-02")
                .strict(true)
                .validity()
                .blocks_submit()
        );
        assert!(
            mk("2026-11-02").validity().is_ok(),
            "an open end is not an error"
        );
    }

    #[test]
    fn bounds_apply_to_both_ends() {
        let lo = d(2026, 11, 1);
        assert!(
            mk("2026-10-30 \u{2013} 2026-11-09")
                .min(lo)
                .validity()
                .blocks_submit()
        );
        assert!(
            mk("2026-11-02 \u{2013} 2026-12-09")
                .max(d(2026, 11, 30))
                .validity()
                .blocks_submit()
        );
    }
}
