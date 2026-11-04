//! `DateInput`: a date entry with a calendar popover.
//!
//! The text is typed as `YYYY-MM-DD` (parsed by [`Date::parse`]); the calendar button opens a
//! month grid to pick a day. Everything is controlled: you keep the text, the month on show and
//! the chosen date, and the widget reports `on_input(text)`, `on_pick(date)` and
//! `on_view(first-of-month)`. Text that does not parse, or is outside `min`/`max`, draws the
//! entry in its error state. Date arithmetic is dependency-free ([`Date`]).
//!
//! ```ignore
//! DateInput::new(&self.text, self.view, Message::Text)
//!     .selected(self.date)
//!     .on_pick(Message::Picked)
//!     .on_view(Message::View)
//! ```
//!
//! The calendar stays open while you page through months or pick a day (click outside or press
//! Escape to close it).

use crate::Element;
use crate::components::{EntryState, entry};
use crate::primitives::calendar::{COLUMNS, CalendarView, Cell, Dir, Level, can_jump};
use crate::primitives::date::{Date, WEEKDAYS};
use crate::primitives::range::within;
use crate::styles::button as sb;
use crate::widgets::popover::{Align, Appearance, Popover, popover};
use crate::{Icon, Metrics, Theme};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, row, space, text};
use iced::{Background, Border, Length, Shadow};

pub struct DateInput<'a, M> {
    text: &'a str,
    view: Date,
    on_input: Option<Box<dyn Fn(String) -> M + 'a>>,
    on_pick: Option<Box<dyn Fn(Date) -> M + 'a>>,
    on_view: Option<OnView<'a, M>>,
    on_nav: Option<OnNav<'a, M>>,
    level: Level,
    min_level: Level,
    max_level: Level,
    today: Date,
    selected: Option<Date>,
    min: Option<Date>,
    max: Option<Date>,
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

impl<'a, M> DateInput<'a, M> {
    /// `text` is the entry's content, `view` any date in the month the calendar shows.
    pub fn new(text: &'a str, view: Date, on_input: impl Fn(String) -> M + 'a) -> Self {
        Self {
            text,
            view: view.month_start(),
            on_input: Some(Box::new(on_input)),
            on_pick: None,
            on_view: None,
            on_nav: None,
            level: Level::Month,
            min_level: Level::Month,
            max_level: Level::Decade,
            today: Date::today(),
            selected: None,
            min: None,
            max: None,
            disabled: false,
            width: Length::Fixed(220.0),
            open_initially: false,
            dialog: false,
            scrim: true,
            close_on_outside: true,
            window_radius: Metrics::desktop().radius_window,
            revealed: true,
            metrics: Metrics::desktop(),
        }
    }
    /// The chosen date, highlighted in the calendar.
    pub fn selected(mut self, d: Option<Date>) -> Self {
        self.selected = d;
        self
    }
    /// A day was clicked in the calendar.
    pub fn on_pick(mut self, f: impl Fn(Date) -> M + 'a) -> Self {
        self.on_pick = Some(Box::new(f));
        self
    }
    /// The previous / next month button was pressed; gets the first day of the new month.
    /// This is the month-level compatibility path: it is called only while the calendar shows
    /// months, and the title and level are not navigable. Use [`on_nav`](Self::on_nav) for
    /// the levels (it takes precedence).
    pub fn on_view(mut self, f: impl Fn(Date) -> M + 'a) -> Self {
        self.on_view = Some(Box::new(f));
        self
    }
    /// The calendar moved: an arrow, the title (zoom out), a month or year cell (drill down) or
    /// "Today". Gets the whole new [`CalendarView`] (period and level), so one message updates
    /// both atomically; keep it and pass it back with [`calendar`](Self::calendar).
    pub fn on_nav(mut self, f: impl Fn(CalendarView) -> M + 'a) -> Self {
        self.on_nav = Some(Box::new(f));
        self
    }
    /// The level on show (default `Month`); the `view` date of [`new`](Self::new) picks the
    /// period. Clamped to [`min_level`](Self::min_level)..=[`max_level`](Self::max_level).
    pub fn level(mut self, l: Level) -> Self {
        self.level = l;
        self
    }
    /// The calendar position as one value: `.level(v.level)` with `v.date` as the view.
    pub fn calendar(mut self, v: CalendarView) -> Self {
        self.view = v.date;
        self.level = v.level;
        self
    }
    /// The finest level (default `Month`). Picking a cell at this level selects it
    /// (`on_pick` gets the first day of that month or year) instead of drilling down, so
    /// `min_level(Level::Year)` is a month picker and `min_level(Level::Decade)` a year picker.
    pub fn min_level(mut self, l: Level) -> Self {
        self.min_level = l;
        self
    }
    /// The coarsest level the title can zoom out to (default `Decade`); at it the title is
    /// dimmed and inert.
    pub fn max_level(mut self, l: Level) -> Self {
        self.max_level = l;
        self
    }
    /// The date "Today" jumps to and the current day, month and year are marked for (default:
    /// the system clock, UTC).
    pub fn today(mut self, d: Date) -> Self {
        self.today = d;
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
    /// Whether validation may show yet (`:user-invalid`). Default `true`: a bad date shows its
    /// error ring at once. Inside a [`Field`](crate::widgets::Field) pass the field's
    /// [`Interaction::revealed`](crate::primitives::validation::Interaction::revealed) so a
    /// pristine field stays quiet and the Field draws the one ring.
    pub fn revealed(mut self, revealed: bool) -> Self {
        self.revealed = revealed;
        self
    }
    /// A disabled input is drawn dimmed and cannot be edited or opened.
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

    /// The validity of the typed text ([`crate::primitives::date::validate`]).
    pub fn validity(&self) -> crate::primitives::validation::Validity {
        crate::primitives::date::validate(self.text, self.min, self.max)
    }

    /// The entry state of the typed text: an error only once [`revealed`](Self::revealed).
    pub fn text_state(&self) -> EntryState {
        // Only errors are drawn on the entry itself; a valid date keeps the normal ring (a Field
        // may add a success ring around it).
        match crate::primitives::validation::present(&self.validity(), self.revealed) {
            crate::primitives::validation::Presentation::Invalid => EntryState::Error,
            _ => EntryState::Normal,
        }
    }
}

/// Side of a day cell in the calendar.
pub(crate) const CELL: f32 = 34.0;
/// Width of the calendar popover: seven cells, their gaps and the popover's padding.
pub(crate) const POPOVER_WIDTH: f32 = CELL * 7.0 + 2.0 * 6.0 + 12.0 + 12.0;
/// Gap between cells.
const GAP: f32 = 2.0;
/// Height of the weekday-name row.
const WEEKDAY_H: f32 = 22.0;

/// How a date input's calendar opens: a popover or a dialog.
#[derive(Clone, Copy)]
pub(crate) struct Opening {
    pub dialog: bool,
    pub scrim: bool,
    pub close_on_outside: bool,
    pub window_radius: f32,
    pub disabled: bool,
    pub open_initially: bool,
}

/// The calendar button for an entry's left section: opens `calendar`.
pub(crate) fn opener<'a, M: 'a>(
    calendar: Element<'a, M>,
    o: Opening,
    m: Metrics,
) -> Element<'a, M> {
    let mut p = Popover::new(Icon::Calendar.view(m.icon_size), calendar)
        .appearance(Appearance::Flat)
        .padding(m.space_xs)
        .align(Align::Start)
        .popover_width(POPOVER_WIDTH)
        .keep_open(true)
        .scrim(o.scrim)
        .close_on_outside(o.close_on_outside)
        .window_radius(o.window_radius)
        .disabled(o.disabled)
        .open_initially(o.open_initially);
    if o.dialog {
        p = p.presentation(crate::widgets::PopoverPresentation::Dialog);
    }
    p.into()
}

pub(crate) type OnNav<'a, M> = Box<dyn Fn(CalendarView) -> M + 'a>;
pub(crate) type OnView<'a, M> = Box<dyn Fn(Date) -> M + 'a>;

/// The message for moving the calendar to `view`: `on_nav`, or for the month-only compatibility
/// path `on_view` (which only knows the `Month` level); `None` when neither can express it.
pub(crate) fn nav_message<'a, M>(
    on_nav: &'a Option<OnNav<'a, M>>,
    on_view: &'a Option<OnView<'a, M>>,
) -> impl Fn(CalendarView) -> Option<M> + 'a {
    move |v| match (on_nav, on_view) {
        (Some(f), _) => Some(f(v)),
        (None, Some(f)) if v.level == Level::Month => Some(f(v.date)),
        _ => None,
    }
}

/// What [`calendar`] draws and how it reports.
pub(crate) struct Cal<'a, M> {
    pub view: CalendarView,
    /// The finest and the coarsest level the picker allows.
    pub levels: (Level, Level),
    pub today: Date,
    /// The selected date as `(d, d)`, or the ends of the selected range.
    pub span: Option<(Date, Date)>,
    pub min: Option<Date>,
    pub max: Option<Date>,
    /// No gap between the days of a week, so a highlighted range reads as one band.
    pub band: bool,
    pub metrics: Metrics,
    /// Whether the app handles `CalendarView` changes (`on_nav`); without it the title is plain
    /// text and only the month-level `on_view` path exists.
    pub navigable: bool,
    pub nav: &'a dyn Fn(CalendarView) -> Option<M>,
    /// A period cell was picked at the finest level (a month picker, a year picker).
    pub select: &'a dyn Fn(Date) -> Option<M>,
}

/// The look of a month or year cell.
fn period_style(
    cell: &Cell,
) -> impl Fn(&Theme, iced::widget::button::Status) -> iced::widget::button::Style + use<> {
    use iced::widget::button::Status;
    let (selected, inside, current, outside) =
        (cell.selected, cell.inside, cell.current, cell.outside);
    move |t: &Theme, status| {
        let s = &t.semantic;
        let r = t.metrics.radius_menu;
        let hover = matches!(status, Status::Hovered | Status::Pressed);
        let (bg, mut fg) = if selected {
            (
                Some(if hover {
                    s.hover_fill(s.accent.bg)
                } else {
                    s.accent.bg
                }),
                s.accent.fg,
            )
        } else if inside {
            (
                Some(crate::color::alpha(
                    s.accent.color,
                    if hover { 0.26 } else { 0.16 },
                )),
                s.accent.color,
            )
        } else {
            (
                hover.then(|| crate::color::alpha(s.popover.fg, 0.10)),
                if current {
                    s.accent.color
                } else {
                    s.popover.fg
                },
            )
        };
        if outside && !selected {
            fg = crate::color::fade(fg, 0.5);
        }
        if status == Status::Disabled {
            fg = crate::color::fade(fg, s.opacity.disabled);
        }
        iced::widget::button::Style {
            background: bg.map(Background::Color),
            text_color: fg,
            border: Border {
                radius: r.into(),
                // Today is outlined, like Mantine's highlighted current period.
                width: if current && !selected { 1.0 } else { 0.0 },
                color: crate::color::alpha(s.accent.color, 0.6),
            },
            shadow: Shadow::default(),
            snap: false,
        }
    }
}

/// The calendar shared by [`DateInput`] and [`DateRangeInput`](crate::widgets::DateRangeInput), at the
/// view's level: a header (previous / next by the level's unit, the title that zooms out), the
/// body (the month's days through `day(date)`, or a 3 x 4 grid of months or years) and a
/// "Today" action. The body has a fixed size, so the popover does not jump between levels, and
/// every row is exactly as wide as the grid and centered.
pub(crate) fn calendar<'a, M: Clone + 'a>(
    c: Cal<'_, M>,
    day: impl Fn(Date) -> Element<'a, M>,
) -> Element<'a, M> {
    let m = c.metrics;
    let view = c.view;
    let (min_level, max_level) = c.levels;
    let gap = if c.band { 0.0 } else { GAP };
    let cell_h = if c.band { CELL - 4.0 } else { CELL };
    let grid_w = CELL * 7.0 + 6.0 * gap;
    let body_h = WEEKDAY_H + 6.0 * cell_h + 6.0 * GAP;

    let arrow = |icon: Icon, dir: Dir| {
        let to = view.step(dir);
        let msg = (c.nav)(to).filter(|_| view.can_step(dir, c.min, c.max));
        // An icon takes no part in the button's disabled look: dim it here.
        let on = msg.is_some();
        let glyph = icon.view(m.icon_size).style(move |t: &Theme, _| {
            let (fg, bg) = (t.semantic.popover.fg, t.semantic.popover.bg);
            // Mixed toward the popover, not faded: a tinted icon ignores alpha.
            iced::widget::svg::Style {
                color: Some(if on {
                    fg
                } else {
                    crate::color::mix(bg, fg, t.semantic.opacity.disabled)
                }),
            }
        });
        let mut b = button(glyph).padding(m.space_s).style(sb::flat);
        if let Some(msg) = msg {
            b = b.on_press(msg);
        }
        b
    };
    let label = view.header_label();
    let zoom = (c.nav)(view.zoom_out(max_level)).filter(|_| view.can_zoom_out(max_level));
    let title: Element<'a, M> = if c.navigable || zoom.is_some() {
        let mut b = button(text(label).width(Length::Fill).align_x(Horizontal::Center))
            .width(Length::Fill)
            .padding([m.space_s, m.space_m])
            .style(sb::flat);
        if let Some(msg) = zoom {
            b = b.on_press(msg);
        }
        b.into()
    } else {
        container(text(label))
            .width(Length::Fill)
            .align_x(Horizontal::Center)
            .into()
    };
    let header = container(
        row![
            arrow(Icon::GoPrevious, Dir::Prev),
            title,
            arrow(Icon::GoNext, Dir::Next)
        ]
        .align_y(Vertical::Center),
    )
    .width(Length::Fixed(grid_w));

    let body: Element<'a, M> = if view.level == Level::Month {
        // One pitch for the weekday labels and the day cells, so every label sits exactly over
        // its column: cells are CELL wide and `gap` apart in both rows.
        let weekdays = row(WEEKDAYS.iter().map(|w| {
            container(text(*w).size(m.space_m).style(crate::styles::text::dim))
                .center_x(Length::Fixed(CELL))
                .center_y(Length::Fixed(WEEKDAY_H))
                .into()
        }))
        .spacing(gap);
        let mut grid = column![weekdays].spacing(GAP);
        for week in view.date.month_grid() {
            grid = grid.push(
                row(week.iter().map(|slot| match slot {
                    None => space().width(CELL).height(cell_h).into(),
                    Some(d) => day(Date {
                        day: *d,
                        ..view.date
                    }),
                }))
                .spacing(gap),
            );
        }
        grid.into()
    } else {
        let cells = view.cells(Some(c.today), c.span, c.min, c.max);
        let w = (grid_w - GAP * (COLUMNS as f32 - 1.0)) / COLUMNS as f32;
        let h = (body_h - GAP * 3.0) / 4.0;
        let mut grid = column![].spacing(GAP);
        for chunk in cells.chunks(COLUMNS) {
            grid = grid.push(
                row(chunk.iter().map(|cell| {
                    let mut b = button(
                        container(text(cell.label.clone()))
                            .center_x(Length::Fixed(w))
                            .center_y(Length::Fixed(h)),
                    )
                    .padding(0)
                    .style(period_style(cell));
                    if !cell.disabled {
                        let msg = match view.drill_down(cell.date, min_level) {
                            Some(to) => (c.nav)(to),
                            None => (c.select)(cell.date),
                        };
                        if let Some(msg) = msg {
                            b = b.on_press(msg);
                        }
                    }
                    b.into()
                }))
                .spacing(GAP),
            );
        }
        grid.into()
    };
    let body = container(body)
        .width(Length::Fixed(grid_w))
        .height(Length::Fixed(body_h));

    // "Today" (as in Firefox's picker): back to the current month (the period of today at the
    // finest level, for a month or year picker); dimmed when already there
    // or when today is outside min..max. Absent when the app cannot take a view change.
    let home = CalendarView::jump_to(c.today).clamped(min_level, max_level);
    let mut col = column![header, body]
        .spacing(m.space_s)
        .align_x(Horizontal::Center)
        .width(Length::Fill);
    if let Some(msg) = (c.nav)(home) {
        let mut b = button(text("Today").size(m.space_m + 1.0))
            .padding([m.space_xs, m.space_m])
            .style(sb::flat);
        if can_jump(c.today, c.min, c.max) && view != home {
            b = b.on_press(msg);
        }
        col = col.push(
            container(b)
                .width(Length::Fixed(grid_w))
                .center_x(Length::Fill),
        );
    }
    col.into()
}

impl<'a, M: Clone + 'a> From<DateInput<'a, M>> for Element<'a, M> {
    fn from(mut d: DateInput<'a, M>) -> Self {
        let m = d.metrics;
        let state = d.text_state();
        let mut field = entry::<M>("YYYY-MM-DD", d.text).state(state).metrics(m);
        if !d.disabled
            && let Some(f) = d.on_input.take()
        {
            field = field.on_input(f);
        }
        let selected = d.selected;
        let (min, max) = (d.min, d.max);
        let view = CalendarView::new(d.view, d.level).clamped(d.min_level, d.max_level);
        let nav = nav_message(&d.on_nav, &d.on_view);
        let select = |date: Date| d.on_pick.as_ref().map(|f| f(date));
        let grid = calendar(
            Cal {
                view,
                levels: (d.min_level, d.max_level),
                today: d.today,
                span: selected.map(|s| (s, s)),
                min,
                max,
                band: false,
                metrics: m,
                navigable: d.on_nav.is_some(),
                nav: &nav,
                select: &select,
            },
            |date| {
                let mut b = button(
                    container(text(date.day.to_string()))
                        .center_x(Length::Fixed(CELL))
                        .center_y(Length::Fixed(CELL)),
                )
                .padding(0)
                .style(sb::menu_item(selected == Some(date)));
                if within(date, min, max)
                    && let Some(f) = &d.on_pick
                {
                    b = b.on_press(f(date));
                }
                b.into()
            },
        );
        let calendar = popover([grid], 400.0);
        let opener = opener(
            calendar,
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

    #[test]
    fn text_state_flags_bad_and_out_of_range_dates() {
        let mk = |t| DateInput::<()>::new(t, d(2026, 11, 1), |_| ()).min(d(2026, 1, 1));
        assert_eq!(mk("").text_state(), EntryState::Normal);
        assert_eq!(mk("2026-11-02").text_state(), EntryState::Normal);
        assert_eq!(mk("2026-13-02").text_state(), EntryState::Error);
        assert_eq!(mk("2025-12-31").text_state(), EntryState::Error);
        // Inside a pristine Field the error stays hidden until revealed.
        assert_eq!(
            mk("2026-13-02").revealed(false).text_state(),
            EntryState::Normal
        );
    }
}
