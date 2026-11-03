//! Sorting: which column is sorted, in which direction, and the stable sort that applies it.
//! After Mantine DataTable's `sortStatus` (`{ columnAccessor, direction }`) and Table sorting
//! recipes. Pure and controlled: the application owns one [`SortState`], feeds header
//! activations to [`activate`](SortState::activate) and sorts its data with
//! [`apply`](SortState::apply); nothing here draws.
//!
//! **The cycle.** Activating a header cycles that column ascending, descending, then unsorted
//! ([`Cycle::ThreeState`], the default; DataTable's `sortStatus` going back to `undefined`).
//! [`Cycle::TwoState`] flips between ascending and descending and never clears. Activating a
//! different column starts it ascending and drops the previous one.
//!
//! **Several columns.** With [`multi`](SortState::multi), activating with `additive` set (a
//! Shift-click, or a "then by" gesture) adds the column as the next sort key or cycles its
//! direction in place; a plain activation replaces the whole list as above. Keys compare in
//! order, later ones only break ties.
//!
//! **Applying.** [`SortState::apply`] is a **stable** sort: equal rows keep their order, also
//! when sorting descending (the comparison is reversed, the list is not). The comparators below
//! ([`cmp_f64`], [`cmp_str`], [`cmp_natural`], [`cmp_option`], ...) cover the usual column types.
//!
//! ```
//! use iced_adaptive_design::primitives::sorting::{Direction, SortState, cmp_natural};
//!
//! #[derive(Clone, Copy, PartialEq, Debug)]
//! enum Col { Name }
//! let mut sort = SortState::<Col>::new();
//! sort.activate(Col::Name, false);
//! assert_eq!(sort.direction(&Col::Name), Some(Direction::Asc));
//! let mut files = ["file10", "file9", "File2"];
//! sort.apply(&mut files, |_, a, b| cmp_natural(a, b));
//! assert_eq!(files, ["File2", "file9", "file10"]);
//! ```

use std::cmp::Ordering;

use super::date::Date;

/// Ascending or descending.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    #[default]
    Asc,
    Desc,
}

impl Direction {
    pub fn toggle(self) -> Self {
        match self {
            Self::Asc => Self::Desc,
            Self::Desc => Self::Asc,
        }
    }
    /// `ord`, reversed for `Desc`.
    pub fn apply(self, ord: Ordering) -> Ordering {
        match self {
            Self::Asc => ord,
            Self::Desc => ord.reverse(),
        }
    }
}

/// What repeated activation of one column does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cycle {
    /// Ascending, descending, unsorted, ascending, ... (the default).
    #[default]
    ThreeState,
    /// Ascending, descending, ascending, ...: once sorted, always sorted.
    TwoState,
}

/// The columns being sorted, most significant first. See the [module](self).
#[derive(Debug, Clone, PartialEq)]
pub struct SortState<K> {
    keys: Vec<(K, Direction)>,
    cycle: Cycle,
    multi: bool,
}

impl<K> Default for SortState<K> {
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            cycle: Cycle::ThreeState,
            multi: false,
        }
    }
}

impl<K: Clone + PartialEq> SortState<K> {
    /// Unsorted, three-state, single column.
    pub fn new() -> Self {
        Self::default()
    }
    /// Sorted by `key` in `dir` from the start.
    pub fn by(key: K, dir: Direction) -> Self {
        Self {
            keys: vec![(key, dir)],
            ..Self::default()
        }
    }
    pub fn cycle(mut self, c: Cycle) -> Self {
        self.cycle = c;
        self
    }
    /// Allow additive activation (several sort keys).
    pub fn multi(mut self, multi: bool) -> Self {
        self.multi = multi;
        self
    }

    /// The sort keys, most significant first.
    pub fn keys(&self) -> &[(K, Direction)] {
        &self.keys
    }
    pub fn is_unsorted(&self) -> bool {
        self.keys.is_empty()
    }
    /// The most significant key.
    pub fn primary(&self) -> Option<&(K, Direction)> {
        self.keys.first()
    }
    /// The direction `key` is sorted in, if it is a sort key.
    pub fn direction(&self, key: &K) -> Option<Direction> {
        self.keys.iter().find(|(k, _)| k == key).map(|(_, d)| *d)
    }
    /// The 1-based position of `key` among the sort keys, only when there are several.
    pub fn priority(&self, key: &K) -> Option<usize> {
        if self.keys.len() < 2 {
            return None;
        }
        self.keys.iter().position(|(k, _)| k == key).map(|i| i + 1)
    }

    pub fn clear(&mut self) {
        self.keys.clear();
    }
    /// Sorts by exactly `key` in `dir` (a sort control that picks a direction directly).
    pub fn set(&mut self, key: K, dir: Direction) {
        self.keys = vec![(key, dir)];
    }
    /// Flips the direction of the primary key, if any.
    pub fn toggle_direction(&mut self) {
        if let Some((_, d)) = self.keys.first_mut() {
            *d = d.toggle();
        }
    }

    /// `d` after one more activation: ascending -> descending -> (unsorted).
    fn step(&self, d: Direction) -> Option<Direction> {
        match (d, self.cycle) {
            (Direction::Asc, _) => Some(Direction::Desc),
            (Direction::Desc, Cycle::TwoState) => Some(Direction::Asc),
            (Direction::Desc, Cycle::ThreeState) => None,
        }
    }

    /// A header was activated. `additive` (Shift held) adds to or edits the key list when
    /// [`multi`](Self::multi) is on; otherwise it is the plain single-column cycle.
    pub fn activate(&mut self, key: K, additive: bool) {
        let at = self.keys.iter().position(|(k, _)| *k == key);
        if additive && self.multi {
            match at {
                Some(i) => match self.step(self.keys[i].1) {
                    Some(d) => self.keys[i].1 = d,
                    None => {
                        self.keys.remove(i);
                    }
                },
                None => self.keys.push((key, Direction::Asc)),
            }
            return;
        }
        let next = match at {
            // Only a lone key cycles; with several keys a plain click starts over from it.
            Some(i) if self.keys.len() == 1 => self.step(self.keys[i].1),
            _ => Some(Direction::Asc),
        };
        self.keys.clear();
        if let Some(d) = next {
            self.keys.push((key, d));
        }
    }

    /// Sorts `items` by the keys, stably. `cmp(key, a, b)` compares two items on one column,
    /// ascending; descending columns are reversed here. With no keys the order is untouched.
    pub fn apply<T>(&self, items: &mut [T], cmp: impl Fn(&K, &T, &T) -> Ordering) {
        if self.keys.is_empty() {
            return;
        }
        items.sort_by(|a, b| {
            self.keys
                .iter()
                .map(|(k, d)| d.apply(cmp(k, a, b)))
                .find(|o| o.is_ne())
                .unwrap_or(Ordering::Equal)
        });
    }

    /// Like [`apply`](Self::apply), but the comparator gets the direction and is **not**
    /// reversed: for columns where "nulls last" must hold in both directions.
    pub fn apply_directed<T>(
        &self,
        items: &mut [T],
        cmp: impl Fn(&K, Direction, &T, &T) -> Ordering,
    ) {
        if self.keys.is_empty() {
            return;
        }
        items.sort_by(|a, b| {
            self.keys
                .iter()
                .map(|(k, d)| cmp(k, *d, a, b))
                .find(|o| o.is_ne())
                .unwrap_or(Ordering::Equal)
        });
    }
}

// -- comparators

/// Numbers, with NaN last (a total order).
pub fn cmp_f64(a: f64, b: f64) -> Ordering {
    match (a.is_nan(), b.is_nan()) {
        (true, true) => Ordering::Equal,
        (true, false) => Ordering::Greater,
        (false, true) => Ordering::Less,
        _ => a.total_cmp(&b),
    }
}

/// Strings, case-insensitive; equal ignoring case falls back to the exact bytes so the order
/// is total ("a" before "A").
pub fn cmp_str(a: &str, b: &str) -> Ordering {
    let fold = |s: &str| s.chars().flat_map(char::to_lowercase).collect::<Vec<_>>();
    fold(a).cmp(&fold(b)).then_with(|| b.cmp(a))
}

/// Calendar dates.
pub fn cmp_date(a: &Date, b: &Date) -> Ordering {
    a.cmp(b)
}

/// "Natural" ordering: runs of digits compare as numbers, everything else case-insensitively,
/// so `file2 < file10` and `a9 < a10`. Equal numbers with different zero padding
/// (`007`, `7`) order the shorter first; a full tie falls back to [`cmp_str`].
pub fn cmp_natural(a: &str, b: &str) -> Ordering {
    let (mut x, mut y) = (a.chars().peekable(), b.chars().peekable());
    loop {
        match (x.peek().copied(), y.peek().copied()) {
            (None, None) => return cmp_str(a, b),
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(c), Some(d)) if c.is_ascii_digit() && d.is_ascii_digit() => {
                let take = |it: &mut std::iter::Peekable<std::str::Chars<'_>>| {
                    let mut s = String::new();
                    while let Some(&ch) = it.peek() {
                        if !ch.is_ascii_digit() {
                            break;
                        }
                        s.push(ch);
                        it.next();
                    }
                    s
                };
                let (n, m) = (take(&mut x), take(&mut y));
                let (nt, mt) = (n.trim_start_matches('0'), m.trim_start_matches('0'));
                let o = nt
                    .len()
                    .cmp(&mt.len())
                    .then_with(|| nt.cmp(mt))
                    .then_with(|| n.len().cmp(&m.len()));
                if o.is_ne() {
                    return o;
                }
            }
            (Some(c), Some(d)) => {
                let (lc, ld) = (
                    c.to_lowercase().next().unwrap_or(c),
                    d.to_lowercase().next().unwrap_or(d),
                );
                if lc != ld {
                    return lc.cmp(&ld);
                }
                x.next();
                y.next();
            }
        }
    }
}

/// Where missing values go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Nulls {
    First,
    #[default]
    Last,
}

/// Compares optional values with `cmp`; `None` is placed by `nulls` (ascending). Under
/// [`SortState::apply`] a descending column reverses this too, nulls included; use
/// [`SortState::apply_directed`] with [`cmp_option_directed`] to pin them.
pub fn cmp_option<T>(
    a: &Option<T>,
    b: &Option<T>,
    nulls: Nulls,
    cmp: impl Fn(&T, &T) -> Ordering,
) -> Ordering {
    match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => match nulls {
            Nulls::First => Ordering::Less,
            Nulls::Last => Ordering::Greater,
        },
        (Some(_), None) => match nulls {
            Nulls::First => Ordering::Greater,
            Nulls::Last => Ordering::Less,
        },
        (Some(x), Some(y)) => cmp(x, y),
    }
}

/// [`cmp_option`] for [`SortState::apply_directed`]: values follow `dir`, `None` stays where
/// `nulls` says in either direction.
pub fn cmp_option_directed<T>(
    a: &Option<T>,
    b: &Option<T>,
    dir: Direction,
    nulls: Nulls,
    cmp: impl Fn(&T, &T) -> Ordering,
) -> Ordering {
    match (a, b) {
        (Some(x), Some(y)) => dir.apply(cmp(x, y)),
        _ => cmp_option(a, b, nulls, cmp),
    }
}

#[cfg(test)]
mod tests {
    use super::Direction::{Asc, Desc};
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum C {
        A,
        B,
        D,
    }

    #[test]
    fn three_state_cycle() {
        let mut s = SortState::new();
        s.activate(C::A, false);
        assert_eq!(s.keys(), [(C::A, Asc)]);
        s.activate(C::A, false);
        assert_eq!(s.keys(), [(C::A, Desc)]);
        s.activate(C::A, false);
        assert!(s.is_unsorted());
        s.activate(C::A, false);
        assert_eq!(s.direction(&C::A), Some(Asc));
    }

    #[test]
    fn two_state_never_clears() {
        let mut s = SortState::new().cycle(Cycle::TwoState);
        for want in [Asc, Desc, Asc, Desc] {
            s.activate(C::A, false);
            assert_eq!(s.direction(&C::A), Some(want));
        }
    }

    #[test]
    fn another_column_replaces_and_starts_ascending() {
        let mut s = SortState::by(C::A, Desc);
        s.activate(C::B, false);
        assert_eq!(s.keys(), [(C::B, Asc)]);
        assert_eq!(s.direction(&C::A), None);
    }

    #[test]
    fn multi_adds_cycles_and_removes_in_place() {
        let mut s = SortState::new().multi(true);
        s.activate(C::A, false);
        s.activate(C::B, true);
        s.activate(C::D, true);
        assert_eq!(s.keys(), [(C::A, Asc), (C::B, Asc), (C::D, Asc)]);
        assert_eq!(s.priority(&C::B), Some(2));
        s.activate(C::B, true);
        assert_eq!(s.keys(), [(C::A, Asc), (C::B, Desc), (C::D, Asc)]);
        s.activate(C::B, true);
        assert_eq!(s.keys(), [(C::A, Asc), (C::D, Asc)]);
        // a plain click starts over from that column
        s.activate(C::D, false);
        assert_eq!(s.keys(), [(C::D, Asc)]);
        assert_eq!(s.priority(&C::D), None, "no priority badge for one key");
    }

    #[test]
    fn additive_is_ignored_without_multi() {
        let mut s = SortState::new();
        s.activate(C::A, false);
        s.activate(C::B, true);
        assert_eq!(s.keys(), [(C::B, Asc)]);
    }

    #[test]
    fn setting_and_toggling_directly() {
        let mut s = SortState::new();
        s.set(C::B, Desc);
        s.toggle_direction();
        assert_eq!(s.keys(), [(C::B, Asc)]);
        s.clear();
        s.toggle_direction();
        assert!(s.is_unsorted());
    }

    #[test]
    fn sorting_is_stable_in_both_directions() {
        // (group, original position)
        let rows = vec![(2, 0), (1, 1), (2, 2), (1, 3), (2, 4), (1, 5)];
        let by = |dir| {
            let mut v = rows.clone();
            SortState::by(C::A, dir).apply(&mut v, |_, a, b| a.0.cmp(&b.0));
            v
        };
        assert_eq!(by(Asc), [(1, 1), (1, 3), (1, 5), (2, 0), (2, 2), (2, 4)]);
        // descending: ties keep their original order too
        assert_eq!(by(Desc), [(2, 0), (2, 2), (2, 4), (1, 1), (1, 3), (1, 5)]);
    }

    #[test]
    fn unsorted_leaves_the_order_alone() {
        let mut v = [3, 1, 2];
        SortState::<C>::new().apply(&mut v, |_, a, b| a.cmp(b));
        assert_eq!(v, [3, 1, 2]);
    }

    #[test]
    fn multi_sort_breaks_ties_with_later_keys() {
        // (team, score)
        let mut v = vec![("b", 1), ("a", 2), ("b", 3), ("a", 1)];
        let mut s = SortState::new().multi(true);
        s.activate(C::A, false);
        s.activate(C::B, true);
        s.activate(C::B, true); // score descending
        s.apply(&mut v, |k, a, b| match k {
            C::A => a.0.cmp(b.0),
            _ => a.1.cmp(&b.1),
        });
        assert_eq!(v, [("a", 2), ("a", 1), ("b", 3), ("b", 1)]);
    }

    #[test]
    fn numbers_put_nan_last() {
        let mut v = [2.5, f64::NAN, -1.0, 10.0];
        v.sort_by(|a, b| cmp_f64(*a, *b));
        assert_eq!(&v[..3], [-1.0, 2.5, 10.0]);
        assert!(v[3].is_nan());
    }

    #[test]
    fn strings_ignore_case_but_stay_total() {
        let mut v = ["banana", "Apple", "cherry", "apple", "Banana"];
        v.sort_by(|a, b| cmp_str(a, b));
        assert_eq!(v, ["apple", "Apple", "banana", "Banana", "cherry"]);
    }

    #[test]
    fn natural_order_reads_digit_runs_as_numbers() {
        let mut v = ["file10", "file9", "File2", "file1", "a", "a1", "a01", "b"];
        v.sort_by(|a, b| cmp_natural(a, b));
        assert_eq!(
            v,
            ["a", "a1", "a01", "b", "file1", "File2", "file9", "file10"]
        );
        assert_eq!(cmp_natural("x99", "x100"), Ordering::Less);
        assert_eq!(cmp_natural("1.5", "1.10"), Ordering::Less);
        assert_eq!(cmp_natural("abc", "abc"), Ordering::Equal);
        assert_eq!(cmp_natural("a", "ab"), Ordering::Less);
        // 20 digits do not overflow
        assert_eq!(
            cmp_natural("n99999999999999999999", "n100000000000000000000"),
            Ordering::Less
        );
    }

    #[test]
    fn dates_compare_chronologically() {
        let a = Date::new(2025, 12, 31).unwrap();
        let b = Date::new(2026, 1, 1).unwrap();
        assert_eq!(cmp_date(&a, &b), Ordering::Less);
    }

    #[test]
    fn options_place_nulls() {
        let c = |a: &Option<i32>, b: &Option<i32>, n| cmp_option(a, b, n, i32::cmp);
        assert_eq!(c(&None, &Some(1), Nulls::Last), Ordering::Greater);
        assert_eq!(c(&None, &Some(1), Nulls::First), Ordering::Less);
        assert_eq!(c(&None, &None, Nulls::Last), Ordering::Equal);
        assert_eq!(c(&Some(1), &Some(2), Nulls::Last), Ordering::Less);
    }

    #[test]
    fn plain_apply_flips_nulls_but_directed_pins_them() {
        let data = vec![Some(2), None, Some(1), Some(3)];
        let mut plain = data.clone();
        SortState::by(C::A, Desc).apply(&mut plain, |_, a, b| {
            cmp_option(a, b, Nulls::Last, i32::cmp)
        });
        assert_eq!(plain, [None, Some(3), Some(2), Some(1)]);
        let mut pinned = data;
        SortState::by(C::A, Desc).apply_directed(&mut pinned, |_, d, a, b| {
            cmp_option_directed(a, b, d, Nulls::Last, i32::cmp)
        });
        assert_eq!(pinned, [Some(3), Some(2), Some(1), None]);
    }
}
