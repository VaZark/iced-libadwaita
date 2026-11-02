//! Calendar navigation by level, after Mantine's `Calendar` / `DatePicker`: the calendar shows a
//! month (a grid of days), a year (a grid of twelve months) or a decade (a grid of twelve years),
//! zooms out by the header and drills down by picking a cell. Pure and dependency-free; the
//! widgets (`DateInput`, `DateRangeInput`) only draw what this decides.
//!
//! * [`Level`] is `Month < Year < Decade`; [`Level::up`] / [`Level::down`] saturate and
//!   [`Level::clamp_to`] keeps a level within a picker's `min_level..=max_level`.
//! * [`CalendarView`] is the controlled state: a level and the **first day of the period on
//!   show** (the first of the month, 1 January, or 1 January of the decade's first year).
//! * [`step`](CalendarView::step) moves by the level's unit (one month, one year, ten years),
//!   [`can_step`](CalendarView::can_step) says whether the next period has any day within
//!   `min..=max`, [`zoom_out`](CalendarView::zoom_out) / [`drill_down`](CalendarView::drill_down)
//!   change the level, [`jump_to`](CalendarView::jump_to) is "Today".
//! * [`cells`](CalendarView::cells) describes the twelve cells of a year or decade grid with
//!   their roles (selected, inside the selection, current, outside the decade, disabled).
//!
//! Decades follow the `2020-2029` convention. The decade grid also shows the last year of the
//! previous and the first year of the next decade, dimmed ([`Cell::outside`]), so the grid is a
//! full 3 x 4 like the year grid.
//!
//! ```
//! use iced_libadwaita::primitives::calendar::{CalendarView, Level};
//! use iced_libadwaita::primitives::date::Date;
//!
//! let nov = CalendarView::month(Date::new(2026, 11, 20).unwrap());
//! assert_eq!(nov.header_label(), "November 2026");
//! let year = nov.zoom_out(Level::Decade);
//! assert_eq!(year.header_label(), "2026");
//! let decade = year.zoom_out(Level::Decade);
//! assert_eq!(decade.header_label(), "2020 \u{2013} 2029");
//! // Picking 2027 in the decade grid drills down to the year 2027.
//! let picked = Date::new(2027, 1, 1).unwrap();
//! assert_eq!(decade.drill_down(picked, Level::Month).unwrap().header_label(), "2027");
//! ```

use super::date::{Date, MONTHS, MONTHS_SHORT, days_in_month};
use super::range::within;

/// What the calendar shows: days of a month, months of a year, or years of a decade.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Level {
    #[default]
    Month,
    Year,
    Decade,
}

impl Level {
    /// The next coarser level (`Decade` stays `Decade`).
    pub fn up(self) -> Self {
        match self {
            Self::Month => Self::Year,
            _ => Self::Decade,
        }
    }
    /// The next finer level (`Month` stays `Month`).
    pub fn down(self) -> Self {
        match self {
            Self::Decade => Self::Year,
            _ => Self::Month,
        }
    }
    /// Within `min..=max`. If the bounds are reversed, `min` wins (a picker never shows a
    /// level finer than its `min_level`).
    pub fn clamp_to(self, min: Level, max: Level) -> Self {
        self.min(max).max(min)
    }
}

/// Which way to step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dir {
    Prev,
    Next,
}

impl Dir {
    fn sign(self) -> i32 {
        match self {
            Dir::Prev => -1,
            Dir::Next => 1,
        }
    }
}

/// The first year of the decade `year` is in: 2026 -> 2020, 2020 -> 2020, 2019 -> 2010.
pub fn decade_of(year: i32) -> i32 {
    year.div_euclid(10) * 10
}

/// The first and last day of the period of `level` that contains `date`.
pub fn period(level: Level, date: Date) -> (Date, Date) {
    match level {
        Level::Month => (
            date.month_start(),
            Date {
                day: days_in_month(date.year, date.month),
                ..date
            },
        ),
        Level::Year => (jan1(date.year), dec31(date.year)),
        Level::Decade => {
            let y = decade_of(date.year);
            (jan1(y), dec31(y + 9))
        }
    }
}

fn jan1(year: i32) -> Date {
    Date {
        year,
        month: 1,
        day: 1,
    }
}
fn dec31(year: i32) -> Date {
    Date {
        year,
        month: 12,
        day: 31,
    }
}

/// A calendar's position: a level and the first day of the period on show. Build it with
/// [`new`](Self::new) (which normalizes the date), so equal views compare equal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CalendarView {
    /// The first day of the period on show.
    pub date: Date,
    pub level: Level,
}

/// One cell of a year (months) or decade (years) grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    /// The first day of the cell's period (the 1st of a month, 1 January of a year).
    pub date: Date,
    /// "Jan", or "2027".
    pub label: String,
    /// The period contains the selected date (or one end of the selected range).
    pub selected: bool,
    /// Between the ends of the selected range, without containing an end.
    pub inside: bool,
    /// The period contains today.
    pub current: bool,
    /// A year of the previous or next decade, shown to fill the grid.
    pub outside: bool,
    /// The whole period is outside `min..=max`.
    pub disabled: bool,
}

/// Columns of the month and year grids.
pub const COLUMNS: usize = 3;

impl CalendarView {
    /// The view of the `level` period containing `date`.
    pub fn new(date: Date, level: Level) -> Self {
        Self {
            date: period(level, date).0,
            level,
        }
    }
    /// The month containing `date`.
    pub fn month(date: Date) -> Self {
        Self::new(date, Level::Month)
    }
    /// "Today": the month of `date` at the `Month` level.
    pub fn jump_to(date: Date) -> Self {
        Self::month(date)
    }

    /// The same period kept within `min..=max` levels: a coarser level shows the period
    /// containing this one, a finer one its first month or year.
    pub fn clamped(self, min: Level, max: Level) -> Self {
        Self::new(self.date, self.level.clamp_to(min, max))
    }

    /// The previous or next period of this level: a month, a year, ten years.
    pub fn step(self, dir: Dir) -> Self {
        let n = dir.sign();
        let date = match self.level {
            Level::Month => self.date.add_months(n),
            Level::Year => self.date.add_years(n),
            Level::Decade => self.date.add_years(10 * n),
        };
        Self::new(date, self.level)
    }

    /// Whether the previous / next period has any day within `min..=max` (the arrow enables).
    pub fn can_step(self, dir: Dir, min: Option<Date>, max: Option<Date>) -> bool {
        self.step(dir).overlaps(min, max)
    }

    /// Whether any day of the period on show is within `min..=max`.
    pub fn overlaps(self, min: Option<Date>, max: Option<Date>) -> bool {
        let (a, b) = self.bounds();
        min.is_none_or(|m| b >= m) && max.is_none_or(|m| a <= m)
    }

    /// The first and last day of the period on show.
    pub fn bounds(self) -> (Date, Date) {
        period(self.level, self.date)
    }

    /// The header text: "November 2026", "2026", "2020 \u{2013} 2029".
    pub fn header_label(self) -> String {
        match self.level {
            Level::Month => format!(
                "{} {}",
                MONTHS[self.date.month as usize - 1],
                self.date.year
            ),
            Level::Year => self.date.year.to_string(),
            Level::Decade => {
                format!("{} \u{2013} {}", self.date.year, self.date.year + 9)
            }
        }
    }

    /// Whether the header can zoom out (the level is below `max`).
    pub fn can_zoom_out(self, max: Level) -> bool {
        self.level < max
    }

    /// The header was pressed: the same date one level up, or unchanged at `max`.
    pub fn zoom_out(self, max: Level) -> Self {
        if self.can_zoom_out(max) {
            Self::new(self.date, self.level.up())
        } else {
            self
        }
    }

    /// A cell was picked (`picked`: its [`Cell::date`]): the view one level down on that
    /// period, or `None` at `min` (the finest level the picker allows), where the pick
    /// selects `picked` instead of drilling.
    pub fn drill_down(self, picked: Date, min: Level) -> Option<Self> {
        (self.level > min).then(|| Self::new(picked, self.level.down()))
    }

    /// The twelve cells of the year or decade grid, row by row ([`COLUMNS`] per row); empty at
    /// `Month`, whose cells are days (see [`Date::month_grid`]). `span` is the selected date as
    /// `(d, d)` or the selected range's ends; `today` marks the current period.
    pub fn cells(
        self,
        today: Option<Date>,
        span: Option<(Date, Date)>,
        min: Option<Date>,
        max: Option<Date>,
    ) -> Vec<Cell> {
        let cell_level = match self.level {
            Level::Month => return Vec::new(),
            l => l.down(),
        };
        let first = self.date.year;
        (0..12)
            .map(|i| {
                let (date, label, outside) = match self.level {
                    Level::Year => (
                        Date {
                            month: i as u8 + 1,
                            ..self.date
                        },
                        MONTHS_SHORT[i].to_string(),
                        false,
                    ),
                    _ => {
                        let y = first - 1 + i as i32;
                        (jan1(y), y.to_string(), !(first..first + 10).contains(&y))
                    }
                };
                let (a, b) = period(cell_level, date);
                let has = |d: Date| d >= a && d <= b;
                let selected = span.is_some_and(|(s, e)| has(s) || has(e));
                Cell {
                    date: a,
                    label,
                    selected,
                    inside: !selected && span.is_some_and(|(s, e)| s < a && b < e),
                    current: today.is_some_and(has),
                    outside,
                    disabled: !(min.is_none_or(|m| b >= m) && max.is_none_or(|m| a <= m)),
                }
            })
            .collect()
    }
}

/// Whether "Today" is available: `today` is within `min..=max`.
pub fn can_jump(today: Date, min: Option<Date>, max: Option<Date>) -> bool {
    within(today, min, max)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u8, dd: u8) -> Date {
        Date::new(y, m, dd).unwrap()
    }
    fn v(y: i32, m: u8, l: Level) -> CalendarView {
        CalendarView::new(d(y, m, 17), l)
    }

    #[test]
    fn levels_are_ordered_and_saturate() {
        assert!(Level::Month < Level::Year && Level::Year < Level::Decade);
        assert_eq!(Level::Month.up(), Level::Year);
        assert_eq!(Level::Year.up(), Level::Decade);
        assert_eq!(Level::Decade.up(), Level::Decade);
        assert_eq!(Level::Decade.down(), Level::Year);
        assert_eq!(Level::Month.down(), Level::Month);
        assert_eq!(
            Level::Month.clamp_to(Level::Year, Level::Decade),
            Level::Year
        );
        assert_eq!(
            Level::Decade.clamp_to(Level::Month, Level::Year),
            Level::Year
        );
        assert_eq!(
            Level::Year.clamp_to(Level::Month, Level::Decade),
            Level::Year
        );
        // Reversed bounds: min wins.
        assert_eq!(
            Level::Month.clamp_to(Level::Year, Level::Month),
            Level::Year
        );
    }

    #[test]
    fn views_normalize_to_the_period_start() {
        assert_eq!(v(2026, 11, Level::Month).date, d(2026, 11, 1));
        assert_eq!(v(2026, 11, Level::Year).date, d(2026, 1, 1));
        assert_eq!(v(2026, 11, Level::Decade).date, d(2020, 1, 1));
        assert_eq!(v(2020, 1, Level::Decade).date, d(2020, 1, 1));
        assert_eq!(v(2029, 12, Level::Decade).date, d(2020, 1, 1));
        assert_eq!(v(2030, 1, Level::Decade).date, d(2030, 1, 1));
        assert_eq!(decade_of(2019), 2010);
        assert_eq!(decade_of(-1), -10);
        assert_eq!(decade_of(0), 0);
    }

    #[test]
    fn steps_by_the_unit_of_the_level() {
        assert_eq!(
            v(2026, 12, Level::Month).step(Dir::Next).date,
            d(2027, 1, 1)
        );
        assert_eq!(
            v(2026, 1, Level::Month).step(Dir::Prev).date,
            d(2025, 12, 1)
        );
        assert_eq!(v(2026, 3, Level::Year).step(Dir::Next).date, d(2027, 1, 1));
        assert_eq!(v(2024, 3, Level::Year).step(Dir::Prev).date, d(2023, 1, 1));
        assert_eq!(
            v(2026, 3, Level::Decade).step(Dir::Next).date,
            d(2030, 1, 1)
        );
        assert_eq!(
            v(2020, 3, Level::Decade).step(Dir::Prev).date,
            d(2010, 1, 1)
        );
        assert_eq!(v(2026, 3, Level::Year).step(Dir::Next).level, Level::Year);
    }

    #[test]
    fn labels() {
        assert_eq!(v(2026, 11, Level::Month).header_label(), "November 2026");
        assert_eq!(v(2026, 11, Level::Year).header_label(), "2026");
        assert_eq!(
            v(2026, 11, Level::Decade).header_label(),
            "2020 \u{2013} 2029"
        );
        assert_eq!(
            v(2030, 1, Level::Decade).header_label(),
            "2030 \u{2013} 2039"
        );
    }

    #[test]
    fn bounds_cover_the_period() {
        assert_eq!(
            v(2024, 2, Level::Month).bounds(),
            (d(2024, 2, 1), d(2024, 2, 29))
        );
        assert_eq!(
            v(2026, 2, Level::Month).bounds(),
            (d(2026, 2, 1), d(2026, 2, 28))
        );
        assert_eq!(
            v(2026, 5, Level::Year).bounds(),
            (d(2026, 1, 1), d(2026, 12, 31))
        );
        assert_eq!(
            v(2026, 5, Level::Decade).bounds(),
            (d(2020, 1, 1), d(2029, 12, 31))
        );
    }

    #[test]
    fn zooming_out_stops_at_the_max_level() {
        let m = v(2026, 11, Level::Month);
        assert!(m.can_zoom_out(Level::Decade));
        let y = m.zoom_out(Level::Decade);
        assert_eq!((y.level, y.date), (Level::Year, d(2026, 1, 1)));
        let dec = y.zoom_out(Level::Decade);
        assert_eq!((dec.level, dec.date), (Level::Decade, d(2020, 1, 1)));
        assert!(!dec.can_zoom_out(Level::Decade));
        assert_eq!(dec.zoom_out(Level::Decade), dec);
        // A year-capped picker never reaches the decade.
        assert_eq!(y.zoom_out(Level::Year), y);
        assert_eq!(m.zoom_out(Level::Month), m);
    }

    #[test]
    fn drilling_down_follows_the_picked_cell_and_stops_at_the_min_level() {
        let dec = v(2026, 5, Level::Decade);
        let year = dec.drill_down(d(2027, 1, 1), Level::Month).unwrap();
        assert_eq!((year.level, year.date), (Level::Year, d(2027, 1, 1)));
        let month = year.drill_down(d(2027, 3, 1), Level::Month).unwrap();
        assert_eq!((month.level, month.date), (Level::Month, d(2027, 3, 1)));
        assert_eq!(month.drill_down(d(2027, 3, 9), Level::Month), None);
        // A month picker (min Year) selects at the year grid; a year picker (min Decade) at
        // the decade grid.
        assert_eq!(year.drill_down(d(2027, 3, 1), Level::Year), None);
        assert_eq!(dec.drill_down(d(2027, 1, 1), Level::Decade), None);
        // Picking an outside year of the neighbouring decade drills into that year.
        let prev = dec.drill_down(d(2019, 1, 1), Level::Month).unwrap();
        assert_eq!(prev.date, d(2019, 1, 1));
    }

    #[test]
    fn clamping_a_view_into_the_allowed_levels() {
        let m = v(2026, 11, Level::Month);
        let mp = m.clamped(Level::Year, Level::Decade);
        assert_eq!((mp.level, mp.date), (Level::Year, d(2026, 1, 1)));
        let dec = v(2026, 11, Level::Decade).clamped(Level::Month, Level::Year);
        assert_eq!((dec.level, dec.date), (Level::Year, d(2020, 1, 1)));
        assert_eq!(m.clamped(Level::Month, Level::Decade), m);
    }

    #[test]
    fn jumping_to_today_lands_on_its_month() {
        let t = CalendarView::jump_to(d(2026, 11, 2));
        assert_eq!((t.level, t.date), (Level::Month, d(2026, 11, 1)));
        assert!(can_jump(d(2026, 11, 2), Some(d(2026, 1, 1)), None));
        assert!(!can_jump(d(2026, 11, 2), Some(d(2026, 11, 3)), None));
        assert!(!can_jump(d(2026, 11, 2), None, Some(d(2026, 11, 1))));
    }

    #[test]
    fn arrows_disable_when_the_whole_next_period_is_out_of_range() {
        let (min, max) = (Some(d(2026, 3, 10)), Some(d(2027, 6, 20)));
        let m = v(2026, 3, Level::Month);
        assert!(
            !m.can_step(Dir::Prev, min, max),
            "February is wholly before min"
        );
        assert!(m.can_step(Dir::Next, min, max));
        let last = v(2027, 6, Level::Month);
        assert!(last.can_step(Dir::Prev, min, max));
        assert!(!last.can_step(Dir::Next, min, max));
        // A partly-in-range period still enables the arrow (min is the 10th of March).
        assert!(v(2026, 4, Level::Month).can_step(Dir::Prev, min, max));
        // Years.
        let y = v(2026, 1, Level::Year);
        assert!(!y.can_step(Dir::Prev, min, max), "2025 is out");
        assert!(y.can_step(Dir::Next, min, max), "2027 is partly in");
        assert!(!v(2027, 1, Level::Year).can_step(Dir::Next, min, max));
        // Decades: 2010s are out, 2030s are out.
        let dec = v(2026, 1, Level::Decade);
        assert!(!dec.can_step(Dir::Prev, min, max));
        assert!(!dec.can_step(Dir::Next, min, max));
        assert!(dec.can_step(Dir::Next, min, None));
        // No bounds: always.
        assert!(dec.can_step(Dir::Prev, None, None));
    }

    #[test]
    fn month_cells_flag_selected_current_and_disabled() {
        let view = v(2026, 5, Level::Year);
        let sel = d(2026, 11, 2);
        let cells = view.cells(
            Some(d(2026, 7, 9)),
            Some((sel, sel)),
            Some(d(2026, 3, 15)),
            Some(d(2026, 10, 31)),
        );
        assert_eq!(cells.len(), 12);
        assert_eq!(cells[0].label, "Jan");
        assert_eq!(cells[0].date, d(2026, 1, 1));
        assert_eq!(cells[11].label, "Dec");
        assert!(
            cells[0].disabled && cells[1].disabled,
            "Jan and Feb are before min"
        );
        assert!(!cells[2].disabled, "March contains min");
        assert!(!cells[9].disabled, "October contains max");
        assert!(
            cells[10].disabled && cells[11].disabled,
            "Nov and Dec are after max"
        );
        assert!(cells[6].current && cells.iter().filter(|c| c.current).count() == 1);
        assert!(cells[10].selected && cells.iter().filter(|c| c.selected).count() == 1);
        assert!(cells.iter().all(|c| !c.outside && !c.inside));
    }

    #[test]
    fn year_cells_include_the_neighbouring_decades_dimmed() {
        let cells = v(2026, 1, Level::Decade).cells(Some(d(2026, 11, 2)), None, None, None);
        let labels: Vec<_> = cells.iter().map(|c| c.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "2019", "2020", "2021", "2022", "2023", "2024", "2025", "2026", "2027", "2028",
                "2029", "2030"
            ]
        );
        assert!(cells[0].outside && cells[11].outside);
        assert!(cells[1..11].iter().all(|c| !c.outside));
        assert_eq!(cells.iter().position(|c| c.current), Some(7));
        assert_eq!(cells[1].date, d(2020, 1, 1));
        // The grid is exactly three columns by four rows.
        assert_eq!(cells.len(), COLUMNS * 4);
    }

    #[test]
    fn year_cells_are_disabled_by_min_and_max() {
        let cells =
            v(2026, 1, Level::Decade).cells(None, None, Some(d(2022, 6, 1)), Some(d(2027, 3, 1)));
        let enabled: Vec<_> = cells
            .iter()
            .filter(|c| !c.disabled)
            .map(|c| c.label.as_str())
            .collect();
        assert_eq!(enabled, ["2022", "2023", "2024", "2025", "2026", "2027"]);
    }

    #[test]
    fn a_range_marks_its_ends_selected_and_what_lies_between_inside() {
        let span = Some((d(2024, 11, 5), d(2026, 2, 3)));
        let cells = v(2024, 1, Level::Decade).cells(None, span, None, None);
        let selected: Vec<_> = cells
            .iter()
            .filter(|c| c.selected)
            .map(|c| &*c.label)
            .collect();
        let inside: Vec<_> = cells
            .iter()
            .filter(|c| c.inside)
            .map(|c| &*c.label)
            .collect();
        assert_eq!(selected, ["2024", "2026"]);
        assert_eq!(inside, ["2025"]);
        // Months of a range inside one year.
        let months = v(2025, 1, Level::Year).cells(None, span, None, None);
        assert!(
            months.iter().all(|c| c.inside),
            "all of 2025 is between the ends"
        );
        let months = v(2024, 1, Level::Year).cells(None, span, None, None);
        let sel: Vec<_> = months
            .iter()
            .filter(|c| c.selected)
            .map(|c| &*c.label)
            .collect();
        assert_eq!(sel, ["Nov"]);
        assert!(months[11].inside, "December 2024 lies between");
        assert!(
            !months[9].inside && !months[9].selected,
            "October is before the start"
        );
    }

    #[test]
    fn the_month_level_has_day_cells_not_period_cells() {
        assert!(
            v(2026, 11, Level::Month)
                .cells(None, None, None, None)
                .is_empty()
        );
    }

    #[test]
    fn a_selected_leap_day_marks_its_month_and_year() {
        let leap = d(2024, 2, 29);
        let months = v(2024, 1, Level::Year).cells(None, Some((leap, leap)), None, None);
        assert!(months[1].selected);
        let years = v(2024, 1, Level::Decade).cells(None, Some((leap, leap)), None, None);
        assert_eq!(years.iter().filter(|c| c.selected).count(), 1);
        // Jumping the selection a year keeps the day within the month.
        assert_eq!(leap.add_years(1), d(2025, 2, 28));
    }

    #[test]
    fn distant_years_and_negative_decades_behave() {
        let w = CalendarView::new(d(1, 1, 1), Level::Decade);
        assert_eq!(w.date, d(0, 1, 1));
        let neg = CalendarView::new(
            Date {
                year: -5,
                month: 1,
                day: 1,
            },
            Level::Decade,
        );
        assert_eq!(neg.date.year, -10);
        assert_eq!(neg.step(Dir::Next).date.year, 0);
    }
}
