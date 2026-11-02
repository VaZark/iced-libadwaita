//! Relative time ("just now", "yesterday", "12 days ago", "in 3 hours"): the wording for a
//! distance from `now`, like `dayjs().fromNow()` or Mantine's date helpers. Pure and
//! dependency-free; the caller passes `now` explicitly, so the result is reproducible and
//! tests need no clock. English only, like [`date`](super::date).
//!
//! * [`Relative::from_seconds`]: the signed distance of an instant (negative = in the past);
//!   below 45 s it is [`Relative::Now`], then minutes, hours and days;
//! * [`Relative::from_dates`]: the distance of two calendar dates in whole days: today,
//!   yesterday, tomorrow, then days, weeks, months and years;
//! * [`Relative`] implements `Display` for the text.
//!
//! ```
//! use iced_libadwaita::primitives::{date::Date, relative::Relative};
//! let today = Date::new(2026, 11, 2).unwrap();
//! assert_eq!(Relative::from_dates(today.add_days(-12), today).to_string(), "1 week ago");
//! assert_eq!(Relative::from_seconds(-90).to_string(), "2 minutes ago");
//! ```

use super::date::Date;
use std::fmt;

/// A unit of a relative distance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl Unit {
    fn name(self) -> &'static str {
        match self {
            Unit::Minute => "minute",
            Unit::Hour => "hour",
            Unit::Day => "day",
            Unit::Week => "week",
            Unit::Month => "month",
            Unit::Year => "year",
        }
    }
}

/// A distance from now, ready to print.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relative {
    /// Within 45 seconds either way.
    Now,
    Today,
    Yesterday,
    Tomorrow,
    /// `n` units in the past.
    Ago(u64, Unit),
    /// `n` units from now.
    In(u64, Unit),
}

impl Relative {
    /// The distance of an instant `delta` seconds from now (negative: in the past).
    pub fn from_seconds(delta: i64) -> Self {
        let n = delta.unsigned_abs();
        if n < 45 {
            return Self::Now;
        }
        let (n, unit) = if n < 45 * 60 {
            ((n + 30) / 60, Unit::Minute)
        } else if n < 22 * 3600 {
            ((n + 1800) / 3600, Unit::Hour)
        } else {
            return Self::from_day_count(delta.div_euclid(86_400).max(-i64::MAX));
        };
        // Rounding up must not print "60 minutes" or "24 hours".
        let (n, unit) = match (n, unit) {
            (60.., Unit::Minute) => (1, Unit::Hour),
            (n, u) => (n.max(1), u),
        };
        if delta < 0 {
            Self::Ago(n, unit)
        } else {
            Self::In(n, unit)
        }
    }

    /// The distance of `date` from `today`, in calendar days.
    pub fn from_dates(date: Date, today: Date) -> Self {
        Self::from_day_count(date.to_days() - today.to_days())
    }

    /// `days` from today (negative: in the past).
    fn from_day_count(days: i64) -> Self {
        let n = days.unsigned_abs();
        let (n, unit) = match n {
            0 => return Self::Today,
            1 if days < 0 => return Self::Yesterday,
            1 => return Self::Tomorrow,
            2..=6 => (n, Unit::Day),
            7..=27 => (n / 7, Unit::Week),
            28..=364 => ((n / 30).max(1), Unit::Month),
            _ => (n / 365, Unit::Year),
        };
        if days < 0 {
            Self::Ago(n, unit)
        } else {
            Self::In(n, unit)
        }
    }
}

impl fmt::Display for Relative {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let plural = |n: u64, u: Unit| format!("{n} {}{}", u.name(), if n == 1 { "" } else { "s" });
        match *self {
            Self::Now => f.write_str("just now"),
            Self::Today => f.write_str("today"),
            Self::Yesterday => f.write_str("yesterday"),
            Self::Tomorrow => f.write_str("tomorrow"),
            Self::Ago(n, u) => write!(f, "{} ago", plural(n, u)),
            Self::In(n, u) => write!(f, "in {}", plural(n, u)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn today() -> Date {
        Date::new(2026, 11, 2).unwrap()
    }
    fn days(n: i64) -> String {
        Relative::from_dates(today().add_days(n), today()).to_string()
    }

    #[test]
    fn dates_use_words_then_units() {
        assert_eq!(days(0), "today");
        assert_eq!(days(-1), "yesterday");
        assert_eq!(days(1), "tomorrow");
        assert_eq!(days(-2), "2 days ago");
        assert_eq!(days(-6), "6 days ago");
        assert_eq!(days(-7), "1 week ago");
        assert_eq!(days(-12), "1 week ago");
        assert_eq!(days(-14), "2 weeks ago");
        assert_eq!(days(-27), "3 weeks ago");
        assert_eq!(days(-28), "1 month ago");
        assert_eq!(days(-90), "3 months ago");
        assert_eq!(days(-364), "12 months ago");
        assert_eq!(days(-365), "1 year ago");
        assert_eq!(days(-800), "2 years ago");
        assert_eq!(days(3), "in 3 days");
        assert_eq!(days(21), "in 3 weeks");
        assert_eq!(days(400), "in 1 year");
    }

    #[test]
    fn instants_round_to_the_nearest_unit() {
        let s = |d: i64| Relative::from_seconds(d).to_string();
        assert_eq!(s(0), "just now");
        assert_eq!(s(-44), "just now");
        assert_eq!(s(44), "just now");
        assert_eq!(s(-45), "1 minute ago");
        assert_eq!(s(-90), "2 minutes ago");
        assert_eq!(s(-60 * 44), "44 minutes ago");
        assert_eq!(s(-60 * 45), "1 hour ago");
        assert_eq!(s(-3600 * 5), "5 hours ago");
        assert_eq!(s(3600 * 2), "in 2 hours");
        assert_eq!(s(60 * 10), "in 10 minutes");
    }

    #[test]
    fn instants_past_a_day_fall_back_to_days() {
        let s = |d: i64| Relative::from_seconds(d).to_string();
        assert_eq!(s(-3600 * 22), "yesterday");
        assert_eq!(s(-86_400 * 3), "3 days ago");
        assert_eq!(s(86_400 * 3), "in 3 days");
        assert_eq!(s(-86_400 * 40), "1 month ago");
    }

    #[test]
    fn never_prints_sixty_minutes() {
        for d in 0..4000 {
            let t = Relative::from_seconds(-d).to_string();
            assert!(!t.starts_with("60 "), "{d}: {t}");
            assert!(!t.starts_with("0 "), "{d}: {t}");
        }
    }
}
