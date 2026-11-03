//! Range: the rule for a value with two ends (`lo <= hi`), shared by every multi-value input
//! (the two-thumb slider, the date range). Pure and controlled: the application owns the
//! [`Range`] and applies what these functions return.
//!
//! * [`Range`] is the value; [`Track`] is the scale it lives on (bounds, step, minimum gap and
//!   what happens when the thumbs meet). Both are generic over [`Scalar`], implemented for `f32`,
//!   `f64`, `i32` and [`Date`], so one rule drives a slider of numbers and one of days.
//! * [`Track`] maps a value to a fraction of the track and back, snaps to the step, picks the
//!   thumb nearest to a pointer, moves a thumb while keeping the ordering invariant, and turns
//!   a key (arrows, page, home, end) into a move.
//! * [`DateRange`] is the calendar selection rule: the first click sets the start, the second
//!   the end (swapped if earlier), a third starts over; a hover shows a preview.

use super::date::Date;
use super::validation::Validity;

/// A value that sits on a number line: converts to and from `f64` (a [`Date`] counts days).
pub trait Scalar: Copy + PartialOrd {
    fn to_f64(self) -> f64;
    fn from_f64(v: f64) -> Self;
}

impl Scalar for f32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
    fn from_f64(v: f64) -> Self {
        v as f32
    }
}
impl Scalar for f64 {
    fn to_f64(self) -> f64 {
        self
    }
    fn from_f64(v: f64) -> Self {
        v
    }
}
impl Scalar for i32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
    fn from_f64(v: f64) -> Self {
        v.round() as i32
    }
}

fn days_from_civil(d: Date) -> i64 {
    d.to_days()
}

fn civil_from_days(z: i64) -> Date {
    Date::from_days(z)
}

impl Scalar for Date {
    fn to_f64(self) -> f64 {
        days_from_civil(self) as f64
    }
    fn from_f64(v: f64) -> Self {
        civil_from_days(v.round() as i64)
    }
}

/// Which end of a [`Range`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Thumb {
    Lo,
    Hi,
}

impl Thumb {
    pub fn other(self) -> Self {
        match self {
            Self::Lo => Self::Hi,
            Self::Hi => Self::Lo,
        }
    }
}

/// A pair of values with `lo <= hi`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Range<T> {
    pub lo: T,
    pub hi: T,
}

impl<T: Scalar> Range<T> {
    /// The two values in order, whichever way round they are given.
    pub fn new(a: T, b: T) -> Self {
        if a <= b {
            Self { lo: a, hi: b }
        } else {
            Self { lo: b, hi: a }
        }
    }
    pub fn get(&self, thumb: Thumb) -> T {
        match thumb {
            Thumb::Lo => self.lo,
            Thumb::Hi => self.hi,
        }
    }
    /// `hi - lo`, in the scalar's unit (days for a [`Date`]).
    pub fn span(&self) -> f64 {
        self.hi.to_f64() - self.lo.to_f64()
    }
    pub fn contains(&self, v: T) -> bool {
        self.lo <= v && v <= self.hi
    }
}

impl<T> From<(T, T)> for Range<T>
where
    T: Scalar,
{
    fn from((a, b): (T, T)) -> Self {
        Self::new(a, b)
    }
}

/// What happens when a thumb is moved into the other one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Collide {
    /// The moving thumb stops against the other (the default).
    #[default]
    Stop,
    /// The moving thumb carries the other along, keeping the gap.
    Push,
}

/// A key a thumb responds to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    /// One step down (left / down arrow).
    Dec,
    /// One step up (right / up arrow).
    Inc,
    /// A page down (`PageDown`): ten steps.
    PageDec,
    /// A page up (`PageUp`): ten steps.
    PageInc,
    /// To the start of the track (`Home`).
    Home,
    /// To the end of the track (`End`).
    End,
}

/// The scale a [`Range`] lives on.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Track<T> {
    pub min: T,
    pub max: T,
    /// Grid spacing from `min`, in the scalar's unit; `0` is continuous.
    pub step: f64,
    /// The smallest allowed `hi - lo`.
    pub min_gap: f64,
    pub collide: Collide,
}

const EPS: f64 = 1e-9;

impl<T: Scalar> Track<T> {
    /// A continuous track from `min` to `max` (swapped if given backwards).
    pub fn new(min: T, max: T) -> Self {
        let (min, max) = if min <= max { (min, max) } else { (max, min) };
        Self {
            min,
            max,
            step: 0.0,
            min_gap: 0.0,
            collide: Collide::Stop,
        }
    }
    pub fn step(mut self, step: f64) -> Self {
        self.step = step.max(0.0);
        self
    }
    pub fn min_gap(mut self, gap: f64) -> Self {
        self.min_gap = gap.max(0.0);
        self
    }
    pub fn collide(mut self, c: Collide) -> Self {
        self.collide = c;
        self
    }

    fn lo(&self) -> f64 {
        self.min.to_f64()
    }
    fn hi(&self) -> f64 {
        self.max.to_f64()
    }
    fn length(&self) -> f64 {
        self.hi() - self.lo()
    }
    /// The gap that fits on the track at all.
    fn gap(&self) -> f64 {
        self.min_gap.min(self.length())
    }

    fn grid(&self, v: f64, round: fn(f64) -> f64) -> f64 {
        if self.step <= 0.0 {
            return v;
        }
        // Floor looks a hair up and ceil a hair down, so a value that is on the grid up to
        // float error stays where it is.
        let nudge = if round(0.5) == 0.0 { EPS } else { -EPS };
        self.lo() + round((v - self.lo()) / self.step + nudge) * self.step
    }

    /// The nearest grid point, or the end of the track when that is nearer (it is always
    /// reachable, even off the grid).
    fn round_grid(&self, v: f64) -> f64 {
        let g = self.lo() + ((v - self.lo()) / self.step).round() * self.step;
        if self.step > 0.0 && (v - self.hi()).abs() < (v - g).abs() {
            self.hi()
        } else if self.step > 0.0 {
            g
        } else {
            v
        }
    }

    /// `v` clamped to the track and snapped to the nearest step.
    fn snap_f(&self, v: f64) -> f64 {
        self.round_grid(v.clamp(self.lo(), self.hi()))
            .min(self.hi())
    }

    /// `v` inside `a..=b`, on the grid when a grid point fits.
    fn snap_within(&self, v: f64, a: f64, b: f64) -> f64 {
        let (a, b) = (a.max(self.lo()), b.min(self.hi()).max(a.max(self.lo())));
        let s = self.round_grid(v.clamp(a, b));
        if s > b + EPS {
            let f = self.grid(b, f64::floor);
            if f >= a - EPS { f } else { b }
        } else if s < a - EPS {
            let c = self.grid(a, f64::ceil);
            if c <= b + EPS { c } else { a }
        } else {
            s.clamp(a, b)
        }
    }

    /// `v` clamped to the track and snapped to the step.
    pub fn snap(&self, v: T) -> T {
        T::from_f64(self.snap_f(v.to_f64()))
    }

    /// `range` with both ends clamped to the track (ordering kept).
    pub fn clamp(&self, range: Range<T>) -> Range<T> {
        let c = |v: T| T::from_f64(v.to_f64().clamp(self.lo(), self.hi()));
        Range::new(c(range.lo), c(range.hi))
    }

    /// `range` clamped and snapped, with at least the minimum gap (the high end gives way).
    pub fn normalize(&self, range: Range<T>) -> Range<T> {
        let r = self.clamp(range);
        let lo = self.snap_within(r.lo.to_f64(), self.lo(), self.hi() - self.gap());
        let hi = self.snap_within(r.hi.to_f64(), lo + self.gap(), self.hi());
        Range {
            lo: T::from_f64(lo),
            hi: T::from_f64(hi),
        }
    }

    /// Where `v` sits on the track, `0.0..=1.0`.
    pub fn fraction(&self, v: T) -> f32 {
        if self.length() <= 0.0 {
            return 0.0;
        }
        ((v.to_f64() - self.lo()) / self.length()).clamp(0.0, 1.0) as f32
    }

    /// The value at `fraction` of the track, snapped.
    pub fn value_at(&self, fraction: f32) -> T {
        let f = (fraction as f64).clamp(0.0, 1.0);
        T::from_f64(self.snap_f(self.lo() + f * self.length()))
    }

    /// The thumb a pointer at `v` should grab: the nearer one; on a tie, the one on `v`'s side,
    /// and for two thumbs on the same spot the one that has room to move towards `v`.
    pub fn nearest(&self, range: Range<T>, v: T) -> Thumb {
        let (lo, hi, v) = (range.lo.to_f64(), range.hi.to_f64(), v.to_f64());
        let (dlo, dhi) = ((v - lo).abs(), (v - hi).abs());
        if dlo < dhi - EPS {
            Thumb::Lo
        } else if dhi < dlo - EPS {
            Thumb::Hi
        } else if v < lo {
            Thumb::Lo
        } else if v > hi {
            Thumb::Hi
        } else if (lo - hi).abs() <= EPS {
            // Stacked: the one that can still move away from the nearer end.
            if lo >= (self.lo() + self.hi()) / 2.0 {
                Thumb::Lo
            } else {
                Thumb::Hi
            }
        } else {
            Thumb::Lo
        }
    }

    /// Moves `thumb` to `to` (clamped, snapped), keeping `lo <= hi` and the minimum gap; per
    /// [`Collide`] the other thumb stops it or is pushed.
    pub fn move_thumb(&self, range: Range<T>, thumb: Thumb, to: T) -> Range<T> {
        let range = self.clamp(range);
        let (lo, hi) = (range.lo.to_f64(), range.hi.to_f64());
        let (to, gap) = (to.to_f64(), self.gap());
        let (min, max) = (self.lo(), self.hi());
        let (nlo, nhi) = match (thumb, self.collide) {
            (Thumb::Lo, Collide::Stop) => (self.snap_within(to, min, hi - gap), hi),
            (Thumb::Hi, Collide::Stop) => (lo, self.snap_within(to, lo + gap, max)),
            (Thumb::Lo, Collide::Push) => {
                let l = self.snap_within(to, min, max - gap);
                (
                    l,
                    if hi < l + gap {
                        self.push_to(l + gap)
                    } else {
                        hi
                    },
                )
            }
            (Thumb::Hi, Collide::Push) => {
                let h = self.snap_within(to, min + gap, max);
                (
                    if lo > h - gap {
                        self.pull_to(h - gap)
                    } else {
                        lo
                    },
                    h,
                )
            }
        };
        Range {
            lo: T::from_f64(nlo),
            hi: T::from_f64(nhi),
        }
    }

    fn push_to(&self, v: f64) -> f64 {
        self.grid(v, f64::ceil).clamp(self.lo(), self.hi())
    }
    fn pull_to(&self, v: f64) -> f64 {
        self.grid(v, f64::floor).clamp(self.lo(), self.hi())
    }

    /// The range after `key` on `thumb`.
    pub fn key(&self, range: Range<T>, thumb: Thumb, key: Key) -> Range<T> {
        let step = if self.step > 0.0 {
            self.step
        } else {
            self.length() / 100.0
        };
        let now = range.get(thumb).to_f64();
        let to = match key {
            Key::Dec => now - step,
            Key::Inc => now + step,
            Key::PageDec => now - 10.0 * step,
            Key::PageInc => now + 10.0 * step,
            Key::Home => self.lo(),
            Key::End => self.hi(),
        };
        self.move_thumb(range, thumb, T::from_f64(to))
    }

    /// Whether the range is at least the minimum gap wide.
    pub fn validity(&self, range: Range<T>) -> Validity {
        if range.span() + EPS < self.min_gap {
            Validity::error("The range is too narrow.")
        } else {
            Validity::Ok
        }
    }
}

/// The validity of two optional ends typed or picked separately: an end before the start is an
/// error ("End must be after start"). `min_gap` of `0` allows both ends equal.
pub fn pair_validity<T: Scalar>(lo: Option<T>, hi: Option<T>, min_gap: f64) -> Validity {
    match (lo, hi) {
        (Some(a), Some(b)) if b.to_f64() - a.to_f64() + EPS < min_gap => {
            Validity::error(if min_gap > 0.0 {
                "End must be after start."
            } else {
                "End must not be before start."
            })
        }
        _ => Validity::Ok,
    }
}

// ---------------------------------------------------------------------------------- DateRange

/// Which end of a [`DateRange`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    Start,
    End,
}

/// How a day relates to the range on show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayRole {
    /// Not in the range.
    Outside,
    /// The first day of a longer range.
    Start,
    /// The last day of a longer range.
    End,
    /// Between the first and the last day.
    Inside,
    /// A range of one day.
    Single,
}

/// A calendar selection, `start` and `end` set one after the other.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DateRange {
    pub start: Option<Date>,
    pub end: Option<Date>,
}

impl From<(Option<Date>, Option<Date>)> for DateRange {
    fn from((start, end): (Option<Date>, Option<Date>)) -> Self {
        Self { start, end }
    }
}
impl From<DateRange> for (Option<Date>, Option<Date>) {
    fn from(r: DateRange) -> Self {
        (r.start, r.end)
    }
}

/// Whether `d` is within the optional bounds.
pub fn within(d: Date, min: Option<Date>, max: Option<Date>) -> bool {
    min.is_none_or(|m| d >= m) && max.is_none_or(|m| d <= m)
}

impl DateRange {
    pub fn new(start: Option<Date>, end: Option<Date>) -> Self {
        Self { start, end }
    }
    /// The range as one text: `YYYY-MM-DD – YYYY-MM-DD` (a missing end leaves that side empty).
    pub fn text(&self) -> String {
        let f = |d: Option<Date>| d.map(|d| d.format()).unwrap_or_default();
        match (self.start, self.end) {
            (None, None) => String::new(),
            (s, e) => format!("{} \u{2013} {}", f(s), f(e)).trim_end().to_string(),
        }
    }
    /// The two sides of a typed range: split at an en or em dash, ` - ` or ` to `; without a
    /// separator all of it is the start.
    pub fn split_text(text: &str) -> (&str, &str) {
        for sep in ["\u{2013}", "\u{2014}", " - ", " to "] {
            if let Some((a, b)) = text.split_once(sep) {
                return (a.trim(), b.trim());
            }
        }
        (text.trim(), "")
    }
    /// The range a typed text spells; a side that does not parse is `None`.
    pub fn parse(text: &str) -> Self {
        let (a, b) = Self::split_text(text);
        Self::new(Date::parse(a), Date::parse(b))
    }
    /// Both ends, in order.
    pub fn complete(&self) -> Option<Range<Date>> {
        Some(Range::new(self.start?, self.end?))
    }
    pub fn is_empty(&self) -> bool {
        self.start.is_none() && self.end.is_none()
    }

    /// The selection after `d` is clicked: nothing yet or a finished range starts a new range
    /// at `d`; with only a start, `d` becomes the end (the two swap if `d` is earlier; the same
    /// day is a range of one day). A day outside `min..=max` changes nothing.
    pub fn pick(self, d: Date, min: Option<Date>, max: Option<Date>) -> Self {
        if !within(d, min, max) {
            return self;
        }
        match (self.start, self.end) {
            (Some(s), None) if d < s => Self {
                start: Some(d),
                end: Some(s),
            },
            (Some(s), None) => Self {
                start: Some(s),
                end: Some(d),
            },
            _ => Self {
                start: Some(d),
                end: None,
            },
        }
    }

    /// Sets one end, as typing it would (the other keeps its value).
    pub fn with(self, edge: Edge, d: Option<Date>) -> Self {
        match edge {
            Edge::Start => Self { start: d, ..self },
            Edge::End => Self { end: d, ..self },
        }
    }

    /// The range to highlight: the selection, or while only the start is set, the preview from
    /// the start to the day under the pointer (`hover`, ignored when out of `min..=max`).
    pub fn shown(
        &self,
        hover: Option<Date>,
        min: Option<Date>,
        max: Option<Date>,
    ) -> Option<Range<Date>> {
        match (self.start, self.end, hover) {
            (Some(s), Some(e), _) => Some(Range::new(s, e)),
            (Some(s), None, Some(h)) if within(h, min, max) => Some(Range::new(s, h)),
            (Some(s), None, _) => Some(Range::new(s, s)),
            _ => None,
        }
    }

    /// The role of `d` in the range on show ([`shown`](Self::shown)).
    pub fn role(
        &self,
        d: Date,
        hover: Option<Date>,
        min: Option<Date>,
        max: Option<Date>,
    ) -> DayRole {
        match self.shown(hover, min, max) {
            Some(r) if r.lo == r.hi && d == r.lo => DayRole::Single,
            Some(r) if d == r.lo => DayRole::Start,
            Some(r) if d == r.hi => DayRole::End,
            Some(r) if r.contains(d) => DayRole::Inside,
            _ => DayRole::Outside,
        }
    }

    /// Both ends within bounds and in order. `strict`: the end must be after the start, not
    /// the same day.
    pub fn validity(&self, min: Option<Date>, max: Option<Date>, strict: bool) -> Validity {
        for d in [self.start, self.end].into_iter().flatten() {
            if min.is_some_and(|m| d < m) {
                return Validity::error(format!("Must be {} or later.", min.unwrap()));
            }
            if max.is_some_and(|m| d > m) {
                return Validity::error(format!("Must be {} or earlier.", max.unwrap()));
            }
        }
        pair_validity(self.start, self.end, if strict { 1.0 } else { 0.0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(y: i32, m: u8, dd: u8) -> Date {
        Date::new(y, m, dd).unwrap()
    }
    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }
    fn t() -> Track<f32> {
        Track::new(0.0, 100.0).step(5.0)
    }
    fn r(a: f32, b: f32) -> Range<f32> {
        Range { lo: a, hi: b }
    }

    #[test]
    fn a_range_orders_its_ends() {
        assert_eq!(Range::new(8, 3), Range { lo: 3, hi: 8 });
        assert_eq!(Range::from((2.0, 1.0)), r(1.0, 2.0));
        assert!(near(r(2.0, 5.0).span() as f32, 3.0));
        assert!(r(2.0, 5.0).contains(5.0) && !r(2.0, 5.0).contains(5.5));
    }

    #[test]
    fn dates_round_trip_through_days() {
        for date in [d(1970, 1, 1), d(2024, 2, 29), d(2026, 11, 2), d(1900, 3, 1)] {
            assert_eq!(Date::from_f64(date.to_f64()), date);
        }
        assert_eq!(d(1970, 1, 2).to_f64() - d(1970, 1, 1).to_f64(), 1.0);
        assert_eq!(d(2026, 3, 1).to_f64() - d(2026, 2, 1).to_f64(), 28.0);
    }

    #[test]
    fn clamps_to_bounds_and_keeps_order() {
        assert_eq!(t().clamp(r(-10.0, 120.0)), r(0.0, 100.0));
        assert_eq!(t().clamp(r(30.0, 20.0)), r(20.0, 30.0));
    }

    #[test]
    fn snaps_to_the_step_and_to_the_end() {
        assert!(near(t().snap(12.0), 10.0));
        assert!(near(t().snap(13.0), 15.0));
        assert!(near(t().snap(500.0), 100.0));
        let odd = Track::new(0.0f32, 10.0).step(4.0);
        assert!(near(odd.snap(9.9), 10.0), "the end is always reachable");
        assert!(near(Track::new(0.0f32, 1.0).snap(0.37), 0.37), "continuous");
    }

    #[test]
    fn normalize_clamps_snaps_and_widens() {
        let tr = t().min_gap(10.0);
        assert_eq!(tr.normalize(r(-5.0, 3.0)), r(0.0, 10.0));
        assert_eq!(tr.normalize(r(97.0, 99.0)), r(90.0, 100.0));
    }

    #[test]
    fn maps_values_and_fractions() {
        assert!(near(t().fraction(25.0), 0.25));
        assert!(near(t().fraction(-3.0), 0.0));
        assert!(near(t().value_at(0.5), 50.0));
        assert!(near(t().value_at(0.53), 55.0), "snapped");
        assert!(near(t().value_at(2.0), 100.0));
        assert!(
            near(Track::new(5.0f32, 5.0).fraction(5.0), 0.0),
            "empty track"
        );
    }

    #[test]
    fn the_nearest_thumb_is_picked() {
        let rg = r(20.0, 60.0);
        assert_eq!(t().nearest(rg, 10.0), Thumb::Lo);
        assert_eq!(t().nearest(rg, 35.0), Thumb::Lo);
        assert_eq!(t().nearest(rg, 45.0), Thumb::Hi);
        assert_eq!(t().nearest(rg, 90.0), Thumb::Hi);
        assert_eq!(t().nearest(rg, 40.0), Thumb::Lo, "the midpoint goes to Lo");
    }

    #[test]
    fn stacked_thumbs_pick_the_one_that_can_move() {
        assert_eq!(t().nearest(r(30.0, 30.0), 10.0), Thumb::Lo);
        assert_eq!(t().nearest(r(30.0, 30.0), 80.0), Thumb::Hi);
        assert_eq!(
            t().nearest(r(0.0, 0.0), 0.0),
            Thumb::Hi,
            "at the start: Hi can go up"
        );
        assert_eq!(t().nearest(r(100.0, 100.0), 100.0), Thumb::Lo);
    }

    #[test]
    fn a_thumb_stops_against_the_other() {
        let tr = t();
        assert_eq!(tr.move_thumb(r(20.0, 60.0), Thumb::Lo, 80.0), r(60.0, 60.0));
        assert_eq!(tr.move_thumb(r(20.0, 60.0), Thumb::Hi, 5.0), r(20.0, 20.0));
        assert_eq!(tr.move_thumb(r(20.0, 60.0), Thumb::Lo, -9.0), r(0.0, 60.0));
        assert_eq!(tr.move_thumb(r(20.0, 60.0), Thumb::Hi, 52.0), r(20.0, 50.0));
    }

    #[test]
    fn the_minimum_gap_holds() {
        let tr = t().min_gap(10.0);
        assert_eq!(tr.move_thumb(r(20.0, 60.0), Thumb::Lo, 80.0), r(50.0, 60.0));
        assert_eq!(tr.move_thumb(r(20.0, 60.0), Thumb::Hi, 0.0), r(20.0, 30.0));
        // A gap that is not on the grid rounds towards the thumb that moves away.
        let tr = t().min_gap(7.0);
        assert_eq!(tr.move_thumb(r(20.0, 60.0), Thumb::Lo, 80.0), r(50.0, 60.0));
    }

    #[test]
    fn push_carries_the_other_thumb() {
        let tr = t().min_gap(10.0).collide(Collide::Push);
        assert_eq!(tr.move_thumb(r(20.0, 60.0), Thumb::Lo, 80.0), r(80.0, 90.0));
        assert_eq!(tr.move_thumb(r(20.0, 60.0), Thumb::Hi, 10.0), r(0.0, 10.0));
        assert_eq!(
            tr.move_thumb(r(20.0, 60.0), Thumb::Lo, 100.0),
            r(90.0, 100.0)
        );
        assert_eq!(tr.move_thumb(r(20.0, 60.0), Thumb::Lo, 30.0), r(30.0, 60.0));
    }

    #[test]
    fn keys_step_page_and_jump() {
        let tr = t();
        let rg = r(20.0, 60.0);
        assert_eq!(tr.key(rg, Thumb::Lo, Key::Inc), r(25.0, 60.0));
        assert_eq!(tr.key(rg, Thumb::Hi, Key::Dec), r(20.0, 55.0));
        assert_eq!(tr.key(rg, Thumb::Hi, Key::PageInc), r(20.0, 100.0));
        assert_eq!(tr.key(rg, Thumb::Lo, Key::PageDec), r(0.0, 60.0));
        assert_eq!(tr.key(rg, Thumb::Lo, Key::Home), r(0.0, 60.0));
        assert_eq!(tr.key(rg, Thumb::Hi, Key::End), r(20.0, 100.0));
        assert_eq!(
            tr.key(rg, Thumb::Lo, Key::End),
            r(60.0, 60.0),
            "stops at Hi"
        );
        assert_eq!(
            tr.key(r(0.0, 5.0), Thumb::Lo, Key::Dec),
            r(0.0, 5.0),
            "at the edge"
        );
    }

    #[test]
    fn a_continuous_track_keys_by_a_percent() {
        let tr = Track::new(0.0f32, 1.0);
        let moved = tr.key(r(0.2, 0.8), Thumb::Lo, Key::Inc);
        assert!(near(moved.lo, 0.21), "{moved:?}");
    }

    #[test]
    fn a_range_of_dates_moves_in_days() {
        let tr = Track::new(d(2026, 11, 1), d(2026, 11, 30))
            .step(1.0)
            .min_gap(2.0);
        let rg = Range::new(d(2026, 11, 10), d(2026, 11, 20));
        assert_eq!(
            tr.move_thumb(rg, Thumb::Lo, d(2026, 11, 25)).lo,
            d(2026, 11, 18)
        );
        assert_eq!(tr.key(rg, Thumb::Hi, Key::Inc).hi, d(2026, 11, 21));
        assert!(near(tr.fraction(d(2026, 11, 15)), 14.0 / 29.0));
        assert_eq!(tr.value_at(1.0), d(2026, 11, 30));
    }

    #[test]
    fn validity_reports_a_range_too_narrow() {
        let tr = t().min_gap(10.0);
        assert!(tr.validity(r(10.0, 30.0)).is_ok());
        assert!(tr.validity(r(10.0, 15.0)).blocks_submit());
        assert!(t().validity(r(10.0, 10.0)).is_ok());
    }

    #[test]
    fn pair_validity_wants_the_end_after_the_start() {
        let (a, b) = (Some(d(2026, 11, 5)), Some(d(2026, 11, 2)));
        assert_eq!(
            pair_validity(a, b, 1.0),
            Validity::error("End must be after start.")
        );
        assert!(pair_validity(a, a, 0.0).is_ok());
        assert!(pair_validity(a, a, 1.0).blocks_submit());
        assert!(pair_validity(a, None, 1.0).is_ok());
        assert!(pair_validity(b, a, 1.0).is_ok());
    }

    #[test]
    fn picking_sets_start_then_end_then_restarts() {
        let none = None;
        let r0 = DateRange::default();
        let r1 = r0.pick(d(2026, 11, 10), none, none);
        assert_eq!(r1, DateRange::new(Some(d(2026, 11, 10)), None));
        let r2 = r1.pick(d(2026, 11, 14), none, none);
        assert_eq!(
            r2.complete(),
            Some(Range::new(d(2026, 11, 10), d(2026, 11, 14)))
        );
        let r3 = r2.pick(d(2026, 11, 20), none, none);
        assert_eq!(r3, DateRange::new(Some(d(2026, 11, 20)), None));
    }

    #[test]
    fn an_earlier_second_click_swaps() {
        let r = DateRange::new(Some(d(2026, 11, 10)), None).pick(d(2026, 11, 3), None, None);
        assert_eq!(
            r,
            DateRange::new(Some(d(2026, 11, 3)), Some(d(2026, 11, 10)))
        );
        let same = DateRange::new(Some(d(2026, 11, 10)), None).pick(d(2026, 11, 10), None, None);
        assert_eq!(same.start, same.end, "one day");
    }

    #[test]
    fn days_outside_the_bounds_are_ignored() {
        let (lo, hi) = (Some(d(2026, 11, 5)), Some(d(2026, 11, 25)));
        let start = DateRange::new(Some(d(2026, 11, 10)), None);
        assert_eq!(start.pick(d(2026, 11, 1), lo, hi), start);
        assert_eq!(start.pick(d(2026, 11, 30), lo, hi), start);
        assert_eq!(
            DateRange::default().pick(d(2026, 11, 4), lo, hi),
            DateRange::default()
        );
    }

    #[test]
    fn a_hover_previews_the_range() {
        let r = DateRange::new(Some(d(2026, 11, 10)), None);
        let h = Some(d(2026, 11, 14));
        assert_eq!(
            r.shown(h, None, None),
            Some(Range::new(d(2026, 11, 10), d(2026, 11, 14)))
        );
        assert_eq!(
            r.shown(Some(d(2026, 11, 3)), None, None),
            Some(Range::new(d(2026, 11, 3), d(2026, 11, 10))),
            "backwards"
        );
        assert_eq!(
            r.shown(None, None, None),
            Some(Range::new(d(2026, 11, 10), d(2026, 11, 10)))
        );
        assert_eq!(
            r.shown(h, None, Some(d(2026, 11, 12))),
            Some(Range::new(d(2026, 11, 10), d(2026, 11, 10))),
            "a hover out of bounds is ignored"
        );
        let done = DateRange::new(Some(d(2026, 11, 10)), Some(d(2026, 11, 12)));
        assert_eq!(
            done.shown(h, None, None),
            done.complete(),
            "a finished range ignores the hover"
        );
        assert_eq!(DateRange::default().shown(h, None, None), None);
    }

    #[test]
    fn days_know_their_role() {
        let r = DateRange::new(Some(d(2026, 11, 10)), Some(d(2026, 11, 12)));
        let role = |day| r.role(d(2026, 11, day), None, None, None);
        assert_eq!(role(9), DayRole::Outside);
        assert_eq!(role(10), DayRole::Start);
        assert_eq!(role(11), DayRole::Inside);
        assert_eq!(role(12), DayRole::End);
        assert_eq!(role(13), DayRole::Outside);
        let one = DateRange::new(Some(d(2026, 11, 10)), None);
        assert_eq!(one.role(d(2026, 11, 10), None, None, None), DayRole::Single);
    }

    #[test]
    fn date_range_validity() {
        let (lo, hi) = (Some(d(2026, 1, 1)), Some(d(2026, 12, 31)));
        let ok = DateRange::new(Some(d(2026, 3, 1)), Some(d(2026, 3, 9)));
        assert!(ok.validity(lo, hi, true).is_ok());
        let backwards = DateRange::new(Some(d(2026, 3, 9)), Some(d(2026, 3, 1)));
        assert_eq!(
            backwards.validity(lo, hi, true),
            Validity::error("End must be after start.")
        );
        let same = DateRange::new(Some(d(2026, 3, 1)), Some(d(2026, 3, 1)));
        assert!(same.validity(lo, hi, false).is_ok());
        assert!(same.validity(lo, hi, true).blocks_submit());
        let early = DateRange::new(Some(d(2025, 3, 1)), None);
        assert!(early.validity(lo, hi, true).blocks_submit());
        assert!(DateRange::default().validity(lo, hi, true).is_ok());
    }

    #[test]
    fn edges_set_one_end() {
        let r = DateRange::new(Some(d(2026, 3, 1)), Some(d(2026, 3, 9)));
        assert_eq!(r.with(Edge::End, None).end, None);
        assert_eq!(
            r.with(Edge::Start, Some(d(2026, 2, 1))).start,
            Some(d(2026, 2, 1))
        );
        assert_eq!(<(Option<Date>, Option<Date>)>::from(r), (r.start, r.end));
    }

    #[test]
    fn a_range_is_typed_as_two_dates_around_a_dash() {
        let (a, b) = (
            Date::new(2026, 11, 2).unwrap(),
            Date::new(2026, 11, 9).unwrap(),
        );
        let r = DateRange::new(Some(a), Some(b));
        assert_eq!(r.text(), "2026-11-02 \u{2013} 2026-11-09");
        assert_eq!(DateRange::parse(&r.text()), r);
        assert_eq!(DateRange::parse("2026-11-02 to 2026-11-09"), r);
        assert_eq!(DateRange::parse("2026-11-02 - 2026-11-09"), r);
        assert_eq!(
            DateRange::parse("2026-11-02"),
            DateRange::new(Some(a), None)
        );
        let open = DateRange::new(Some(a), None);
        assert_eq!(open.text(), "2026-11-02 \u{2013}");
        assert_eq!(DateRange::parse(&open.text()), open);
        assert_eq!(DateRange::default().text(), "");
    }
}
