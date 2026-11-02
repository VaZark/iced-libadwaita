//! Pagination: which page, how many items per page, and which page numbers a pager shows.
//! After Mantine's `usePagination` hook and `Pagination` component (and Bootstrap's pagination).
//! Pure and controlled: the application owns one [`Pagination`]; nothing here draws.
//!
//! **Numbering.** Pages are **1-based** everywhere (`page()`, [`PageItem::Page`], `goto`), as
//! Mantine does and as users read them. Item positions are **0-based** (`offset()`, `slice`),
//! as Rust indexes. [`range`](Pagination::range) reports 1-based, inclusive item numbers for
//! "Showing 21-40 of 187".
//!
//! * [`set_page_size`](Pagination::set_page_size) keeps the first visible item in view: from page
//!   3 of size 20 (item 41 first), a size of 50 lands on page 1 and a size of 10 on page 5;
//! * [`set_total`](Pagination::set_total) clamps the page when the data shrinks;
//! * [`first`](Pagination::first), [`prev`](Pagination::prev), [`next`](Pagination::next),
//!   [`last`](Pagination::last), [`goto`](Pagination::goto) clamp into `1..=page_count` and say
//!   whether the page changed; [`next_page`](Pagination::next_page) and friends are their pure
//!   twins (the target page, or `None` at the end), which a button's `on_press` uses;
//! * [`items`](Pagination::items) is Mantine's page-number window (`siblings`, `boundaries`,
//!   dots), a list of [`PageItem`].
//!
//! With an unknown total ([`Pagination::unknown`], a server that cannot count) there is no
//! `page_count`, `last` does nothing and [`has_next`](Pagination::has_next) stays true until the
//! app calls [`set_total`](Pagination::set_total).
//!
//! ```
//! use iced_libadwaita::primitives::pagination::{PageItem::*, Pagination};
//! let mut p = Pagination::new(187, 20);
//! assert_eq!(p.page_count(), Some(10));
//! p.goto(2);
//! assert_eq!(p.label(), "Showing 21\u{2013}40 of 187");
//! assert_eq!(p.items(1, 1), [Page(1), Page(2), Page(3), Ellipsis, Page(10)]);
//! ```

use std::fmt;

/// One entry of the page-number window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageItem {
    /// A page button, 1-based.
    Page(usize),
    /// The "..." between two runs of pages. Not a button (Mantine's `'dots'`).
    Ellipsis,
}

/// The inclusive item numbers on a page (1-based), and the total if known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Range {
    pub first: usize,
    pub last: usize,
    pub total: Option<usize>,
}

impl fmt::Display for Range {
    /// `21-40 of 187` with an en dash (`21-40` alone with an unknown total).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.first == self.last {
            write!(f, "{}", self.first)?;
        } else {
            write!(f, "{}\u{2013}{}", self.first, self.last)?;
        }
        if let Some(t) = self.total {
            write!(f, " of {t}")?;
        }
        Ok(())
    }
}

/// The paging state: page (1-based), page size and the number of items, if known.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pagination {
    page: usize,
    page_size: usize,
    total: Option<usize>,
}

impl Default for Pagination {
    /// No items, 10 per page.
    fn default() -> Self {
        Self::new(0, 10)
    }
}

impl Pagination {
    /// `total` items at `page_size` per page (a size of 0 counts as 1), on page 1.
    pub fn new(total: usize, page_size: usize) -> Self {
        Self {
            page: 1,
            page_size: page_size.max(1),
            total: Some(total),
        }
    }

    /// Paging over a source that cannot count its items.
    pub fn unknown(page_size: usize) -> Self {
        Self {
            page: 1,
            page_size: page_size.max(1),
            total: None,
        }
    }

    /// Starts on `page` (clamped).
    pub fn at(mut self, page: usize) -> Self {
        self.goto(page);
        self
    }

    pub fn page(&self) -> usize {
        self.page
    }
    pub fn page_size(&self) -> usize {
        self.page_size
    }
    pub fn total(&self) -> Option<usize> {
        self.total
    }

    /// The number of pages: `ceil(total / page_size)`, 0 for no items, `None` if unknown.
    pub fn page_count(&self) -> Option<usize> {
        self.total.map(|t| t.div_ceil(self.page_size))
    }

    /// The highest page that can be shown: the page count, at least 1.
    fn max_page(&self) -> usize {
        self.page_count().map_or(usize::MAX, |c| c.max(1))
    }

    /// Index (0-based) of the first item on the page.
    pub fn offset(&self) -> usize {
        (self.page - 1) * self.page_size
    }

    pub fn has_prev(&self) -> bool {
        self.page > 1
    }
    /// Whether a later page exists (always true while the total is unknown).
    pub fn has_next(&self) -> bool {
        self.page < self.max_page()
    }

    // -- targets (pure)

    /// The page [`prev`](Self::prev) would go to, if any.
    pub fn prev_page(&self) -> Option<usize> {
        self.has_prev().then(|| self.page - 1)
    }
    pub fn next_page(&self) -> Option<usize> {
        self.has_next().then(|| self.page + 1)
    }
    pub fn first_page(&self) -> Option<usize> {
        self.has_prev().then_some(1)
    }
    /// `None` on the last page, or when the total is unknown.
    pub fn last_page(&self) -> Option<usize> {
        let last = self.page_count()?.max(1);
        (self.page != last).then_some(last)
    }
    /// `page` clamped into range, `None` if that is the current page.
    pub fn goto_page(&self, page: usize) -> Option<usize> {
        let p = page.clamp(1, self.max_page());
        (p != self.page).then_some(p)
    }

    // -- transitions

    /// Moves to `page`, clamped; true if the page changed.
    pub fn goto(&mut self, page: usize) -> bool {
        match self.goto_page(page) {
            Some(p) => {
                self.page = p;
                true
            }
            None => false,
        }
    }
    pub fn first(&mut self) -> bool {
        self.goto(1)
    }
    pub fn last(&mut self) -> bool {
        match self.page_count() {
            Some(c) => self.goto(c),
            None => false,
        }
    }
    pub fn prev(&mut self) -> bool {
        self.prev_page().is_some_and(|p| self.goto(p))
    }
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> bool {
        self.next_page().is_some_and(|p| self.goto(p))
    }

    /// A new page size (0 counts as 1), keeping the first visible item in view.
    pub fn set_page_size(&mut self, size: usize) {
        let size = size.max(1);
        let first_item = self.offset();
        self.page_size = size;
        self.page = first_item / size + 1;
        self.page = self.page.min(self.max_page());
    }

    /// The data changed size: the total, clamping the page if it shrank.
    pub fn set_total(&mut self, total: usize) {
        self.total = Some(total);
        self.page = self.page.min(self.max_page());
    }

    /// The first item numbers on this page, or `None` when there are no items.
    pub fn range(&self) -> Option<Range> {
        let first = self.offset() + 1;
        let end = match self.total {
            Some(0) => return None,
            Some(t) => (first - 1 + self.page_size).min(t),
            None => first - 1 + self.page_size,
        };
        (first <= end).then_some(Range {
            first,
            last: end,
            total: self.total,
        })
    }

    /// "Showing 21-40 of 187" (en dash), "Showing 21-40" with an unknown total, "No results".
    pub fn label(&self) -> String {
        match self.range() {
            Some(r) => format!("Showing {r}"),
            None => String::from("No results"),
        }
    }

    /// The part of `all` (every item, in order) on this page; empty past the end.
    pub fn slice<'a, T>(&self, all: &'a [T]) -> &'a [T] {
        let start = self.offset().min(all.len());
        let end = start.saturating_add(self.page_size).min(all.len());
        &all[start..end]
    }

    /// Mantine's page-number window for this page: see [`window`].
    pub fn items(&self, siblings: usize, boundaries: usize) -> Vec<PageItem> {
        match self.page_count() {
            Some(total) => window(total, self.page, siblings, boundaries),
            None => Vec::new(),
        }
    }
}

fn run(from: i64, to: i64, out: &mut Vec<PageItem>) {
    out.extend((from..=to).map(|n| PageItem::Page(n as usize)));
}

/// Mantine's `usePagination` range: the page numbers (and dots) for `total` pages with `page`
/// active, `siblings` pages on each side of the active one and `boundaries` pages at each end.
///
/// If `siblings * 2 + 3 + boundaries * 2 >= total`, every page is listed. Otherwise the window
/// always has the same number of entries, so the pager does not change width as the page moves:
/// dots appear on a side only when they would hide at least two pages.
pub fn window(total: usize, page: usize, siblings: usize, boundaries: usize) -> Vec<PageItem> {
    let (total, page) = (total as i64, page as i64);
    let (s, b) = (siblings as i64, boundaries as i64);
    let mut out = Vec::new();
    if s * 2 + 3 + b * 2 >= total {
        run(1, total, &mut out);
        return out;
    }
    let left = (page - s).max(b);
    let right = (page + s).min(total - b);
    let dots_left = left > b + 2;
    let dots_right = right < total - (b + 1);

    if !dots_left && dots_right {
        run(1, s * 2 + b + 2, &mut out);
        out.push(PageItem::Ellipsis);
        run(total - (b - 1), total, &mut out);
    } else if dots_left && !dots_right {
        run(1, b, &mut out);
        out.push(PageItem::Ellipsis);
        run(total - (b + 1 + 2 * s), total, &mut out);
    } else {
        run(1, b, &mut out);
        out.push(PageItem::Ellipsis);
        run(left, right, &mut out);
        out.push(PageItem::Ellipsis);
        run(total - b + 1, total, &mut out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::PageItem::{Ellipsis as D, Page as P};
    use super::*;

    fn pages(v: &[usize]) -> Vec<PageItem> {
        v.iter().map(|&n| P(n)).collect()
    }

    #[test]
    fn counts_and_ranges() {
        let mut p = Pagination::new(187, 20);
        assert_eq!(p.page_count(), Some(10));
        assert_eq!(p.range().unwrap().to_string(), "1\u{2013}20 of 187");
        p.goto(10);
        assert_eq!(p.label(), "Showing 181\u{2013}187 of 187");
        assert_eq!(p.offset(), 180);
    }

    #[test]
    fn no_items_and_one_page() {
        let p = Pagination::new(0, 20);
        assert_eq!(p.page_count(), Some(0));
        assert_eq!(p.range(), None);
        assert_eq!(p.label(), "No results");
        assert!(!p.has_prev() && !p.has_next());
        assert!(p.items(1, 1).is_empty());
        let one = Pagination::new(5, 20);
        assert_eq!(one.page_count(), Some(1));
        assert!(!one.has_next());
        assert_eq!(one.items(1, 1), pages(&[1]));
        assert_eq!(one.label(), "Showing 1\u{2013}5 of 5");
        assert_eq!(Pagination::new(1, 20).label(), "Showing 1 of 1");
    }

    #[test]
    fn moves_clamp_and_report() {
        let mut p = Pagination::new(50, 10);
        assert!(!p.prev() && !p.first());
        assert!(p.next());
        assert_eq!(p.page(), 2);
        assert!(p.last());
        assert_eq!(p.page(), 5);
        assert!(!p.next() && !p.last());
        assert!(!p.goto(99) && p.page() == 5);
        assert!(p.goto(0));
        assert_eq!(p.page(), 1);
        assert_eq!(p.next_page(), Some(2));
        assert_eq!(p.prev_page(), None);
        assert_eq!(p.last_page(), Some(5));
        assert_eq!(p.goto_page(1), None);
        assert_eq!(p.goto_page(100), Some(5));
    }

    #[test]
    fn page_size_change_keeps_the_first_visible_item() {
        let mut p = Pagination::new(187, 20).at(3); // items 41-60
        p.set_page_size(10);
        assert_eq!((p.page(), p.range().unwrap().first), (5, 41));
        p.set_page_size(50);
        assert_eq!(p.page(), 1); // 41 is on page 1 of 50
        p.set_page_size(25);
        assert_eq!(p.page(), 1);
        let mut q = Pagination::new(187, 20).at(10); // item 181
        q.set_page_size(100);
        assert_eq!(q.page(), 2);
        q.set_page_size(0);
        assert_eq!(q.page_size(), 1);
        assert_eq!(q.page(), 101);
    }

    #[test]
    fn total_shrinking_clamps_the_page() {
        let mut p = Pagination::new(100, 10).at(10);
        p.set_total(35);
        assert_eq!(p.page(), 4);
        p.set_total(0);
        assert_eq!(p.page(), 1);
        p.set_total(500);
        assert_eq!(p.page(), 1, "growing never moves the page");
    }

    #[test]
    fn unknown_total_is_open_ended() {
        let mut p = Pagination::unknown(10);
        assert_eq!(p.page_count(), None);
        assert!(p.has_next() && !p.has_prev());
        assert!(!p.last());
        assert!(p.next() && p.next());
        assert_eq!(p.page(), 3);
        assert_eq!(p.label(), "Showing 21\u{2013}30");
        assert!(p.items(1, 1).is_empty());
        p.set_total(25);
        assert_eq!(p.page(), 3);
        assert!(!p.has_next());
    }

    #[test]
    fn slices_the_page_and_tolerates_the_end() {
        let all: Vec<u32> = (0..45).collect();
        let mut p = Pagination::new(all.len(), 20);
        assert_eq!(p.slice(&all), &all[0..20]);
        p.goto(3);
        assert_eq!(p.slice(&all), &all[40..45]);
        let past = Pagination::new(100, 20).at(5);
        assert!(past.slice(&all).is_empty());
        assert!(Pagination::new(0, 5).slice::<u32>(&[]).is_empty());
    }

    // -- Mantine usePagination parity (siblings = 1, boundaries = 1 unless noted)

    #[test]
    fn window_lists_everything_when_it_fits() {
        // 1*2 + 3 + 1*2 = 7 >= total
        assert_eq!(window(7, 4, 1, 1), pages(&[1, 2, 3, 4, 5, 6, 7]));
        assert_eq!(window(1, 1, 1, 1), pages(&[1]));
        assert_eq!(window(0, 1, 1, 1), vec![]);
        assert_eq!(window(5, 3, 2, 0), pages(&[1, 2, 3, 4, 5]));
    }

    #[test]
    fn window_dots_on_the_right_near_the_start() {
        let w = |page| window(20, page, 1, 1);
        assert_eq!(w(1), [P(1), P(2), P(3), P(4), P(5), D, P(20)]);
        assert_eq!(w(4), [P(1), P(2), P(3), P(4), P(5), D, P(20)]);
    }

    #[test]
    fn window_dots_on_both_sides_in_the_middle() {
        assert_eq!(
            window(20, 10, 1, 1),
            [P(1), D, P(9), P(10), P(11), D, P(20)]
        );
        // the first page where the left dots appear (left sibling 5 > 1 + 2)
        assert_eq!(window(20, 6, 1, 1), [P(1), D, P(5), P(6), P(7), D, P(20)]);
    }

    #[test]
    fn window_dots_on_the_left_near_the_end() {
        let w = |page| window(20, page, 1, 1);
        assert_eq!(w(20), [P(1), D, P(16), P(17), P(18), P(19), P(20)]);
        assert_eq!(w(17), [P(1), D, P(16), P(17), P(18), P(19), P(20)]);
    }

    #[test]
    fn window_has_a_constant_width_while_it_has_dots() {
        for (s, b) in [(1, 1), (2, 1), (1, 2), (0, 1), (2, 2), (1, 0)] {
            let total = 40;
            let want = s * 2 + 3 + b * 2;
            for page in 1..=total {
                let w = window(total, page, s, b);
                assert_eq!(w.len(), want, "s={s} b={b} page={page}: {w:?}");
                assert!(!w.windows(2).any(|p| p[0] == D && p[1] == D));
                // ascending
                let nums: Vec<usize> = w
                    .iter()
                    .filter_map(|i| if let P(n) = i { Some(*n) } else { None })
                    .collect();
                assert!(nums.windows(2).all(|p| p[0] < p[1]));
                assert!(nums.contains(&page), "active page is shown");
            }
        }
    }

    #[test]
    fn window_with_two_siblings_and_two_boundaries() {
        assert_eq!(
            window(30, 15, 2, 2),
            [
                P(1),
                P(2),
                D,
                P(13),
                P(14),
                P(15),
                P(16),
                P(17),
                D,
                P(29),
                P(30)
            ]
        );
        assert_eq!(
            window(30, 1, 2, 2),
            [
                P(1),
                P(2),
                P(3),
                P(4),
                P(5),
                P(6),
                P(7),
                P(8),
                D,
                P(29),
                P(30)
            ]
        );
    }

    #[test]
    fn window_without_boundaries() {
        // b = 0: no fixed first/last page, only dots at the ends.
        assert_eq!(window(20, 10, 1, 0), [D, P(9), P(10), P(11), D]);
        assert_eq!(window(20, 1, 1, 0), [P(1), P(2), P(3), P(4), D]);
    }

    #[test]
    fn window_with_huge_page_counts() {
        let w = window(1_000_000, 500_000, 1, 1);
        assert_eq!(
            w,
            [P(1), D, P(499_999), P(500_000), P(500_001), D, P(1_000_000)]
        );
        let w = window(usize::MAX / 4, 1, 1, 1);
        assert_eq!(w.len(), 7);
    }

    #[test]
    fn items_follow_the_current_page() {
        let mut p = Pagination::new(400, 20);
        assert_eq!(p.items(1, 1), [P(1), P(2), P(3), P(4), P(5), D, P(20)]);
        p.goto(10);
        assert_eq!(p.items(1, 1), [P(1), D, P(9), P(10), P(11), D, P(20)]);
    }
}
