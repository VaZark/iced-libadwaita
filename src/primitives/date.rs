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
pub const MONTHS_SHORT: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
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

    /// Days since 1970-01-01 (Hinnant's `days_from_civil`).
    pub fn to_days(&self) -> i64 {
        let y = self.year as i64 - (self.month <= 2) as i64;
        let era = y.div_euclid(400);
        let yoe = y - era * 400;
        let m = self.month as i64;
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + self.day as i64 - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146097 + doe - 719468
    }

    /// The date `z` days after 1970-01-01.
    pub fn from_days(z: i64) -> Self {
        let z = z + 719468;
        let era = z.div_euclid(146097);
        let doe = z - era * 146097;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let y = yoe + era * 400 + (m <= 2) as i64;
        Self {
            year: y as i32,
            month: m as u8,
            day: d as u8,
        }
    }

    /// Today according to the system clock (UTC); falls back to 1970-01-01 if the clock is
    /// before the epoch.
    pub fn today() -> Self {
        let secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs() as i64);
        Self::from_days(secs.div_euclid(86_400))
    }

    /// `n` days later (earlier when negative).
    pub fn add_days(&self, n: i64) -> Self {
        Self::from_days(self.to_days() + n)
    }

    /// `n` months later (earlier when negative); the day is clamped to the new month's length
    /// (31 January + 1 month = 28 or 29 February).
    pub fn add_months(&self, n: i32) -> Self {
        let total = self.year * 12 + (self.month as i32 - 1) + n;
        let (year, month) = (total.div_euclid(12), total.rem_euclid(12) as u8 + 1);
        Self {
            year,
            month,
            day: self.day.min(days_in_month(year, month)),
        }
    }

    /// `n` years later (earlier when negative); 29 February clamps to the 28th in a common year.
    pub fn add_years(&self, n: i32) -> Self {
        self.add_months(n * 12)
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

/// Validates typed `text` as a date within `min..=max`: empty is `Ok` (use `required` for that),
/// an unparsable date or one out of range is an error with a message a field can show.
pub fn validate(
    text: &str,
    min: Option<Date>,
    max: Option<Date>,
) -> crate::primitives::validation::Validity {
    use crate::primitives::validation::Validity;
    if text.trim().is_empty() {
        return Validity::Ok;
    }
    match Date::parse(text) {
        None => Validity::error("Not a date. Use YYYY-MM-DD."),
        Some(d) if min.is_some_and(|m| d < m) => {
            Validity::error(format!("Must be {} or later.", min.unwrap()))
        }
        Some(d) if max.is_some_and(|m| d > m) => {
            Validity::error(format!("Must be {} or earlier.", max.unwrap()))
        }
        Some(_) => Validity::Ok,
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
    fn day_counts_round_trip() {
        assert_eq!(d(1970, 1, 1).to_days(), 0);
        for z in [-800_000i64, -1, 0, 1, 11_000, 20_000, 2_932_896] {
            assert_eq!(Date::from_days(z).to_days(), z);
        }
        assert_eq!(d(2026, 11, 2).add_days(30), d(2026, 12, 2));
        assert_eq!(d(2026, 1, 1).add_days(-1), d(2025, 12, 31));
    }

    #[test]
    fn adding_months_and_years_clamps_the_day() {
        assert_eq!(d(2026, 1, 31).add_months(1), d(2026, 2, 28));
        assert_eq!(d(2024, 1, 31).add_months(1), d(2024, 2, 29));
        assert_eq!(d(2026, 11, 15).add_months(2), d(2027, 1, 15));
        assert_eq!(d(2026, 1, 15).add_months(-1), d(2025, 12, 15));
        assert_eq!(d(2026, 3, 15).add_months(-26), d(2024, 1, 15));
        assert_eq!(d(2024, 2, 29).add_years(1), d(2025, 2, 28));
        assert_eq!(d(2024, 2, 29).add_years(4), d(2028, 2, 29));
        assert_eq!(d(2024, 2, 29).add_years(-4), d(2020, 2, 29));
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

    #[test]
    fn validate_reports_unparsable_and_out_of_range() {
        use crate::primitives::validation::Validity;
        let (a, b) = (Date::new(2026, 1, 1), Date::new(2026, 12, 31));
        assert_eq!(validate("", a, b), Validity::Ok);
        assert_eq!(validate("2026-06-01", a, b), Validity::Ok);
        assert!(validate("2026-13-01", a, b).blocks_submit());
        assert!(validate("2025-12-31", a, b).blocks_submit());
        assert!(validate("2027-01-01", a, b).blocks_submit());
    }
}
