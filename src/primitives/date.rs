//! `Date`: a calendar date in the proleptic Gregorian calendar, with the month arithmetic a date
//! picker needs (parsing, validation, weekday, month navigation, the month grid). Pure and
//! dependency-free (no `chrono`). English month names, Monday-first weeks, ISO format.

/// A calendar date in the proleptic Gregorian calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

pub const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
pub const WEEKDAYS: [&str; 7] = ["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"];

pub fn is_leap_year(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

pub fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        _ => 28,
    }
}

impl Date {
    /// `None` unless the date exists.
    pub fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        ((1..=12).contains(&month) && day >= 1 && day <= days_in_month(year, month))
            .then_some(Self { year, month, day })
    }

    /// Parses `YYYY-MM-DD` (also `/` or `.` as separators, and one-digit month and day).
    pub fn parse(s: &str) -> Option<Self> {
        let mut it = s.trim().split(['-', '/', '.']);
        let (y, m, d) = (it.next()?, it.next()?, it.next()?);
        if it.next().is_some()
            || y.len() != 4
            || !(1..=2).contains(&m.len())
            || !(1..=2).contains(&d.len())
        {
            return None;
        }
        Self::new(y.parse().ok()?, m.parse().ok()?, d.parse().ok()?)
    }

    /// `YYYY-MM-DD`.
    pub fn format(&self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// 0 = Monday … 6 = Sunday.
    pub fn weekday(&self) -> u8 {
        // Sakamoto's method gives 0 = Sunday.
        const T: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
        let y = if self.month < 3 {
            self.year - 1
        } else {
            self.year
        };
        let w = (y + y / 4 - y / 100 + y / 400 + T[self.month as usize - 1] + self.day as i32)
            .rem_euclid(7);
        ((w + 6) % 7) as u8
    }

    /// The first day of this date's month.
    pub fn month_start(&self) -> Self {
        Self { day: 1, ..*self }
    }

    /// The first of the next month.
    pub fn next_month(&self) -> Self {
        if self.month == 12 {
            Self {
                year: self.year + 1,
                month: 1,
                day: 1,
            }
        } else {
            Self {
                month: self.month + 1,
                day: 1,
                ..*self
            }
        }
    }

    /// The first of the previous month.
    pub fn prev_month(&self) -> Self {
        if self.month == 1 {
            Self {
                year: self.year - 1,
                month: 12,
                day: 1,
            }
        } else {
            Self {
                month: self.month - 1,
                day: 1,
                ..*self
            }
        }
    }

    /// The month as weeks (Monday first); `None` pads the days of the neighbouring months.
    pub fn month_grid(&self) -> Vec<[Option<u8>; 7]> {
        let first = self.month_start().weekday() as usize;
        let n = days_in_month(self.year, self.month) as usize;
        let mut weeks = Vec::new();
        let mut cell = 0;
        while cell < first + n {
            let mut w = [None; 7];
            for (i, slot) in w.iter_mut().enumerate() {
                let idx = cell + i;
                if idx >= first && idx < first + n {
                    *slot = Some((idx - first + 1) as u8);
                }
            }
            weeks.push(w);
            cell += 7;
        }
        weeks
    }
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.format())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u8, dd: u8) -> Date {
        Date::new(y, m, dd).unwrap()
    }

    #[test]
    fn parses_and_formats() {
        assert_eq!(Date::parse("2026-11-02"), Some(d(2026, 11, 2)));
        assert_eq!(Date::parse(" 2026/1/5 "), Some(d(2026, 1, 5)));
        assert_eq!(d(2026, 1, 5).format(), "2026-01-05");
    }

    #[test]
    fn rejects_what_is_not_a_date() {
        for bad in [
            "",
            "2026",
            "2026-13-01",
            "2026-02-30",
            "2025-02-29",
            "26-01-01",
            "2026-1-1-1",
            "a-b-c",
            "2026-00-10",
        ] {
            assert_eq!(Date::parse(bad), None, "{bad}");
        }
        assert_eq!(Date::parse("2024-02-29"), Some(d(2024, 2, 29)));
    }

    #[test]
    fn month_lengths_and_leap_years() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(1900, 2), 28);
        assert_eq!(days_in_month(2000, 2), 29);
        assert_eq!(days_in_month(2026, 4), 30);
    }

    #[test]
    fn weekdays_are_monday_first() {
        assert_eq!(d(2026, 11, 2).weekday(), 0, "a Monday");
        assert_eq!(d(2000, 1, 1).weekday(), 5, "a Saturday");
        assert_eq!(d(2026, 11, 8).weekday(), 6);
    }

    #[test]
    fn months_navigate_across_years() {
        assert_eq!(d(2026, 12, 15).next_month(), d(2027, 1, 1));
        assert_eq!(d(2026, 1, 15).prev_month(), d(2025, 12, 1));
        assert_eq!(d(2026, 3, 31).prev_month(), d(2026, 2, 1));
    }

    #[test]
    fn the_grid_places_days_under_their_weekday() {
        let g = d(2026, 11, 1).month_grid(); // Sunday the 1st
        assert_eq!(g[0], [None, None, None, None, None, None, Some(1)]);
        assert_eq!(g.len(), 6);
        assert_eq!(g[5][0], Some(30));
        let feb = d(2027, 2, 1).month_grid(); // Monday, 28 days: exactly four weeks
        assert_eq!(feb.len(), 4);
        assert_eq!(feb[3][6], Some(28));
    }
}
