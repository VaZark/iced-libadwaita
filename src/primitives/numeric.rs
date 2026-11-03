//! Numbers typed or stepped by handles (Mantine's `NumberInput`): fixed-point decimals with a
//! chosen number of digits, never floats, so `0.1 + 0.2` is exactly `0.3` and a value formats
//! the same everywhere.
//!
//! A [`Decimal`] is whole `units` of `10^-digits` (`Decimal::new(1250, 2)` is `12.50`). A
//! [`Numeric`] is the rule an input follows: `digits` (0 is an integer), `min`, `max` and the
//! `step` a handle moves by. Everything is pure and controlled: the application keeps the text
//! (so typing is not interrupted), and a handle gives the new text from [`Numeric::up_text`] /
//! [`Numeric::down_text`]. [`Scrub`] turns a drag along an axis into a number of steps, for
//! changing a value continuously by dragging.

use crate::primitives::validation::Validity;
use std::cmp::Ordering;

/// A fixed-point number: `units / 10^digits`.
#[derive(Debug, Clone, Copy)]
pub struct Decimal {
    pub units: i64,
    pub digits: u8,
}

/// The most fraction digits a [`Decimal`] carries (`i64` units stay far from overflow).
pub const MAX_DIGITS: u8 = 9;

fn pow10(n: u8) -> i128 {
    10i128.pow(u32::from(n.min(MAX_DIGITS)))
}

impl Decimal {
    pub const fn new(units: i64, digits: u8) -> Self {
        Self { units, digits }
    }
    /// A whole number.
    pub const fn int(n: i64) -> Self {
        Self::new(n, 0)
    }

    /// The same value with `digits` fraction digits, rounding half away from zero when it loses
    /// some. Saturates at the `i64` range.
    pub fn rescale(self, digits: u8) -> Self {
        let digits = digits.min(MAX_DIGITS);
        let u = i128::from(self.units);
        let v = match digits.cmp(&self.digits) {
            Ordering::Equal => u,
            Ordering::Greater => u * pow10(digits - self.digits),
            Ordering::Less => {
                let d = pow10(self.digits - digits);
                let half = d / 2;
                if u >= 0 {
                    (u + half) / d
                } else {
                    (u - half) / d
                }
            }
        };
        Self::new(
            v.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64,
            digits,
        )
    }

    /// Reads `-12`, `+3.5`, `.5`, `7.`, `1.250`: an optional sign, digits and one `.`. The
    /// result has as many digits as were typed (up to [`MAX_DIGITS`]; more are rounded off).
    pub fn parse(text: &str) -> Option<Self> {
        let t = text.trim();
        let (neg, t) = match t.strip_prefix('-') {
            Some(r) => (true, r),
            None => (false, t.strip_prefix('+').unwrap_or(t)),
        };
        let (int, frac) = t.split_once('.').unwrap_or((t, ""));
        if (int.is_empty() && frac.is_empty())
            || !int.bytes().all(|b| b.is_ascii_digit())
            || !frac.bytes().all(|b| b.is_ascii_digit())
        {
            return None;
        }
        let digits = frac.len().min(usize::from(MAX_DIGITS)) as u8;
        let whole: i128 = if int.is_empty() { 0 } else { int.parse().ok()? };
        let kept: i128 = if digits == 0 {
            0
        } else {
            frac[..usize::from(digits)].parse().ok()?
        };
        let mut units = whole.checked_mul(pow10(digits))?.checked_add(kept)?;
        // Digits past the ninth round the last one.
        if frac.len() > usize::from(digits) && frac.as_bytes()[usize::from(digits)] >= b'5' {
            units += 1;
        }
        let units = i64::try_from(if neg { -units } else { units }).ok()?;
        Some(Self::new(units, digits))
    }

    /// `12.50`: exactly `digits` fraction digits.
    pub fn format(&self) -> String {
        let sign = if self.units < 0 { "-" } else { "" };
        let abs = i128::from(self.units).abs();
        let d = pow10(self.digits);
        if self.digits == 0 {
            return format!("{sign}{abs}");
        }
        format!(
            "{sign}{}.{:0width$}",
            abs / d,
            abs % d,
            width = usize::from(self.digits)
        )
    }

    fn cmp_value(&self, other: &Self) -> Ordering {
        let digits = self.digits.max(other.digits);
        let a = i128::from(self.rescale(digits).units);
        let b = i128::from(other.rescale(digits).units);
        a.cmp(&b)
    }
}

impl PartialEq for Decimal {
    fn eq(&self, other: &Self) -> bool {
        self.cmp_value(other) == Ordering::Equal
    }
}
impl Eq for Decimal {}
impl PartialOrd for Decimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Decimal {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_value(other)
    }
}
impl std::fmt::Display for Decimal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.format())
    }
}
impl From<i64> for Decimal {
    fn from(n: i64) -> Self {
        Self::int(n)
    }
}
impl From<i32> for Decimal {
    fn from(n: i32) -> Self {
        Self::int(i64::from(n))
    }
}

/// What a number input follows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Numeric {
    /// Fraction digits kept and shown; `0` is an integer.
    pub digits: u8,
    pub min: Option<Decimal>,
    pub max: Option<Decimal>,
    /// What one handle press adds or takes (kept at `digits`).
    pub step: Decimal,
}

impl Default for Numeric {
    /// An integer with no limits, stepping by 1.
    fn default() -> Self {
        Self {
            digits: 0,
            min: None,
            max: None,
            step: Decimal::int(1),
        }
    }
}

impl Numeric {
    pub fn integer() -> Self {
        Self::default()
    }
    /// `digits` fraction digits; the step becomes one unit of the last digit unless set after.
    pub fn decimal(digits: u8) -> Self {
        let digits = digits.min(MAX_DIGITS);
        Self {
            digits,
            step: Decimal::new(1, digits),
            ..Self::default()
        }
    }
    pub fn digits(mut self, digits: u8) -> Self {
        self.digits = digits.min(MAX_DIGITS);
        self
    }
    pub fn min(mut self, d: impl Into<Decimal>) -> Self {
        self.min = Some(d.into());
        self
    }
    pub fn max(mut self, d: impl Into<Decimal>) -> Self {
        self.max = Some(d.into());
        self
    }
    pub fn step(mut self, d: impl Into<Decimal>) -> Self {
        self.step = d.into();
        self
    }

    /// `d` kept to the digits and inside `min..=max`.
    pub fn clamp(&self, d: Decimal) -> Decimal {
        let mut d = d.rescale(self.digits);
        if let Some(lo) = self.min.filter(|lo| d < *lo) {
            d = lo.rescale(self.digits);
        }
        if let Some(hi) = self.max.filter(|hi| d > *hi) {
            d = hi.rescale(self.digits);
        }
        d
    }

    /// The number a typed text spells, rounded to the digits; `None` for text that is not one.
    pub fn parse(&self, text: &str) -> Option<Decimal> {
        Decimal::parse(text).map(|d| d.rescale(self.digits))
    }

    /// `d` as text with exactly the digits.
    pub fn format(&self, d: Decimal) -> String {
        d.rescale(self.digits).format()
    }

    /// Whether `d` may go up / down by a step (it is not at the limit).
    pub fn can_up(&self, d: Decimal) -> bool {
        self.max.is_none_or(|hi| d < hi)
    }
    pub fn can_down(&self, d: Decimal) -> bool {
        self.min.is_none_or(|lo| d > lo)
    }

    fn moved(&self, d: Option<Decimal>, sign: i128) -> Decimal {
        // Nothing typed: a handle starts from the nearest limit to zero.
        let from = d
            .map(|d| d.rescale(self.digits))
            .unwrap_or_else(|| self.clamp(Decimal::new(0, self.digits)));
        let step = self.step.rescale(self.digits);
        let units = i128::from(from.units) + sign * i128::from(step.units.max(1));
        self.clamp(Decimal::new(
            units.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64,
            self.digits,
        ))
    }
    /// One step up from `d` (from the limit nearest zero when `d` is `None`), clamped.
    pub fn up(&self, d: Option<Decimal>) -> Decimal {
        self.moved(d, 1)
    }
    pub fn down(&self, d: Option<Decimal>) -> Decimal {
        self.moved(d, -1)
    }
    /// The text after pressing the up / down handle with `text` in the entry.
    pub fn up_text(&self, text: &str) -> String {
        self.format(self.up(self.parse(text)))
    }
    pub fn down_text(&self, text: &str) -> String {
        self.format(self.down(self.parse(text)))
    }

    /// `start` moved by `steps` steps (negative goes down), clamped: the value a drag has reached.
    /// With no `start` (an empty or invalid entry) it counts from the limit nearest zero.
    pub fn offset(&self, start: Option<Decimal>, steps: i64) -> Decimal {
        self.offset_by(start, steps, Precision::Normal)
    }

    /// Like [`offset`](Self::offset), with each step as large as `precision` says.
    pub fn offset_by(&self, start: Option<Decimal>, steps: i64, precision: Precision) -> Decimal {
        let from = start
            .map(|d| d.rescale(self.digits))
            .unwrap_or_else(|| self.clamp(Decimal::new(0, self.digits)));
        let step = i128::from(self.step.rescale(self.digits).units.max(1));
        let unit = match precision {
            Precision::Normal => step,
            Precision::Coarse => step * 10,
            // One unit of the last digit, never more than a step.
            Precision::Fine => 1.min(step),
        };
        let units = i128::from(from.units) + i128::from(steps) * unit;
        self.clamp(Decimal::new(
            units.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64,
            self.digits,
        ))
    }

    /// Whether the typed text is acceptable: empty is `Ok` (use `required` for that).
    pub fn validate(&self, text: &str) -> Validity {
        if text.trim().is_empty() {
            return Validity::Ok;
        }
        let Some(d) = self.parse(text) else {
            return Validity::error("Not a number.");
        };
        match (self.min, self.max) {
            (Some(lo), _) if d < lo => {
                Validity::error(format!("Must be {} or more.", self.format(lo)))
            }
            (_, Some(hi)) if d > hi => {
                Validity::error(format!("Must be {} or less.", self.format(hi)))
            }
            _ => Validity::Ok,
        }
    }
}

/// How large a step of a drag is: a modifier key trades speed for precision, as in a design tool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Precision {
    /// One `step`.
    #[default]
    Normal,
    /// Ten steps (Shift).
    Coarse,
    /// One unit of the last digit (Alt).
    Fine,
}

impl Precision {
    /// Shift is coarse and Alt fine; both, or neither, is normal.
    pub fn from_modifiers(shift: bool, alt: bool) -> Self {
        match (shift, alt) {
            (true, false) => Self::Coarse,
            (false, true) => Self::Fine,
            _ => Self::Normal,
        }
    }
}

/// The direction a drag changes a value in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Axis {
    /// Dragging right increases, left decreases.
    Horizontal,
    /// Dragging up increases, down decreases.
    #[default]
    Vertical,
}

/// A drag along an [`Axis`] as steps: `px_per_step` of travel is one step, and nothing counts as
/// a drag until it has travelled `threshold` px along the axis (so a click stays a click).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scrub {
    pub axis: Axis,
    pub px_per_step: f32,
    pub threshold: f32,
}

impl Scrub {
    pub fn new(axis: Axis) -> Self {
        Self {
            axis,
            px_per_step: 4.0,
            threshold: 4.0,
        }
    }
    pub fn px_per_step(mut self, px: f32) -> Self {
        self.px_per_step = px.max(0.5);
        self
    }

    /// Travel from `from` to `to` along the axis: positive in the direction that increases.
    pub fn travel(&self, from: (f32, f32), to: (f32, f32)) -> f32 {
        match self.axis {
            Axis::Horizontal => to.0 - from.0,
            Axis::Vertical => from.1 - to.1,
        }
    }
    /// Whether the pointer has moved far enough along the axis for this to be a drag.
    pub fn started(&self, from: (f32, f32), to: (f32, f32)) -> bool {
        self.travel(from, to).abs() >= self.threshold
    }
    /// Where the drag's anchor goes after `steps` steps were taken from it: moved along the axis
    /// by the travel they used, keeping the part of the pointer's travel that is not yet a step.
    pub fn advance(&self, anchor: (f32, f32), steps: i64) -> (f32, f32) {
        let travel = steps as f32 * self.px_per_step;
        match self.axis {
            Axis::Horizontal => (anchor.0 + travel, anchor.1),
            Axis::Vertical => (anchor.0, anchor.1 - travel),
        }
    }
    /// The steps a drag from `from` to `to` has made: whole steps, towards zero.
    pub fn steps(&self, from: (f32, f32), to: (f32, f32)) -> i64 {
        (self.travel(from, to) / self.px_per_step).trunc() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimals_format_with_exactly_their_digits() {
        assert_eq!(Decimal::new(1250, 2).format(), "12.50");
        assert_eq!(Decimal::new(-5, 2).format(), "-0.05");
        assert_eq!(Decimal::new(7, 0).format(), "7");
        assert_eq!(Decimal::new(0, 3).format(), "0.000");
    }

    #[test]
    fn text_is_read_as_typed() {
        assert_eq!(Decimal::parse("12.5"), Some(Decimal::new(125, 1)));
        assert_eq!(Decimal::parse(" -0.05 "), Some(Decimal::new(-5, 2)));
        assert_eq!(Decimal::parse(".5"), Some(Decimal::new(5, 1)));
        assert_eq!(Decimal::parse("7."), Some(Decimal::new(7, 0)));
        assert_eq!(Decimal::parse("+3"), Some(Decimal::new(3, 0)));
        for bad in ["", "-", ".", "1.2.3", "1e3", "abc", "1,5", "--1"] {
            assert_eq!(Decimal::parse(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn values_compare_across_digits() {
        assert_eq!(Decimal::new(10, 1), Decimal::new(100, 2));
        assert!(Decimal::new(25, 1) > Decimal::new(249, 2));
        assert_eq!(Decimal::new(1, 0).format(), "1");
    }

    #[test]
    fn rescaling_rounds_half_away_from_zero() {
        assert_eq!(Decimal::new(125, 2).rescale(1), Decimal::new(13, 1));
        assert_eq!(Decimal::new(-125, 2).rescale(1), Decimal::new(-13, 1));
        assert_eq!(Decimal::new(124, 2).rescale(1), Decimal::new(12, 1));
        assert_eq!(Decimal::new(5, 1).rescale(3).units, 500);
    }

    #[test]
    fn a_step_is_exact_not_floating() {
        let n = Numeric::decimal(1).step(Decimal::new(1, 1));
        let mut t = "0.0".to_string();
        for _ in 0..3 {
            t = n.up_text(&t);
        }
        assert_eq!(t, "0.3");
    }

    #[test]
    fn steps_stop_at_the_limits() {
        let n = Numeric::integer().min(0).max(10).step(4);
        assert_eq!(n.up_text("8"), "10");
        assert_eq!(n.down_text("3"), "0");
        assert!(!n.can_up(Decimal::int(10)) && n.can_up(Decimal::int(9)));
        assert!(!n.can_down(Decimal::int(0)) && n.can_down(Decimal::int(1)));
    }

    #[test]
    fn an_empty_entry_starts_from_the_limit_nearest_zero() {
        assert_eq!(Numeric::integer().up_text(""), "1");
        assert_eq!(Numeric::integer().min(5).up_text(""), "6");
        assert_eq!(Numeric::integer().max(-3).down_text("nope"), "-4");
    }

    #[test]
    fn typed_digits_are_rounded_to_the_spec() {
        let n = Numeric::decimal(2);
        assert_eq!(n.parse("1.005"), Some(Decimal::new(101, 2)));
        assert_eq!(n.parse("3"), Some(Decimal::new(300, 2)));
        assert_eq!(Numeric::integer().parse("2.5"), Some(Decimal::int(3)));
    }

    #[test]
    fn validation_names_the_problem() {
        let n = Numeric::decimal(2).min(0).max(100);
        assert!(n.validate("").is_ok() && n.validate("12.5").is_ok());
        assert_eq!(n.validate("x"), Validity::error("Not a number."));
        assert_eq!(n.validate("-1"), Validity::error("Must be 0.00 or more."));
        assert_eq!(
            n.validate("101"),
            Validity::error("Must be 100.00 or less.")
        );
    }

    #[test]
    fn a_drag_is_steps_in_the_direction_of_its_axis() {
        let h = Scrub::new(Axis::Horizontal).px_per_step(5.0);
        assert_eq!(
            h.steps((10.0, 0.0), (40.0, 90.0)),
            6,
            "right: up, whatever the y"
        );
        assert_eq!(h.steps((40.0, 0.0), (10.0, 0.0)), -6);
        let v = Scrub::new(Axis::Vertical).px_per_step(5.0);
        assert_eq!(v.steps((0.0, 50.0), (0.0, 20.0)), 6, "up increases");
        assert_eq!(v.steps((0.0, 20.0), (99.0, 50.0)), -6, "down decreases");
        assert_eq!(
            h.steps((0.0, 0.0), (4.9, 0.0)),
            0,
            "less than a step is none"
        );
    }

    #[test]
    fn a_click_is_not_a_drag_until_it_has_travelled_along_the_axis() {
        let h = Scrub::new(Axis::Horizontal);
        assert!(
            !h.started((10.0, 10.0), (12.0, 80.0)),
            "moving across the axis is not a drag"
        );
        assert!(h.started((10.0, 10.0), (14.0, 10.0)));
        assert!(h.started((10.0, 10.0), (6.0, 10.0)), "either way");
    }

    #[test]
    fn a_scrubbed_value_steps_exactly_and_stops_at_the_limits() {
        let n = Numeric::decimal(1).min(0).max(1).step(Decimal::new(1, 1));
        assert_eq!(n.offset(Some(Decimal::new(2, 1)), 3).format(), "0.5");
        assert_eq!(n.offset(Some(Decimal::new(2, 1)), -9).format(), "0.0");
        assert_eq!(n.offset(Some(Decimal::new(2, 1)), 99).format(), "1.0");
        assert_eq!(
            n.offset(None, 4).format(),
            "0.4",
            "an empty entry counts from zero"
        );
    }

    #[test]
    fn shift_is_coarse_and_alt_is_fine() {
        let n = Numeric::decimal(2).step(Decimal::new(5, 2)).min(0).max(100);
        let from = Some(Decimal::new(100, 2));
        assert_eq!(n.offset_by(from, 2, Precision::Normal).format(), "1.10");
        assert_eq!(n.offset_by(from, 2, Precision::Coarse).format(), "2.00");
        assert_eq!(n.offset_by(from, 2, Precision::Fine).format(), "1.02");
        assert_eq!(Precision::from_modifiers(true, false), Precision::Coarse);
        assert_eq!(Precision::from_modifiers(false, true), Precision::Fine);
        assert_eq!(Precision::from_modifiers(true, true), Precision::Normal);
    }

    #[test]
    fn the_anchor_keeps_the_travel_that_is_not_yet_a_step() {
        let v = Scrub::new(Axis::Vertical).px_per_step(5.0);
        // 12 px up from y 100 is 2 steps (10 px); the anchor moves to y 90, leaving 2 px.
        let steps = v.steps((0.0, 100.0), (0.0, 88.0));
        assert_eq!(steps, 2);
        assert_eq!(v.advance((0.0, 100.0), steps), (0.0, 90.0));
        let h = Scrub::new(Axis::Horizontal).px_per_step(5.0);
        assert_eq!(h.advance((10.0, 7.0), -3), (-5.0, 7.0));
    }
}
