//! Headless logic of a splitter: panes along one axis with a draggable handle between each pair.
//! After Mantine's [Splitter](https://mantine.dev/core/splitter/).
//!
//! Pure and controlled: the app owns one [`SplitterState`] and applies the widget's [`Event`]s.
//!
//! * **Sizes.** A pane is [`Size::Px`] (fixed: it keeps its size when the container changes and
//!   changes only when its own handle is dragged) or [`Size::Flex`] (a weight: flexible panes
//!   share what the fixed ones leave). If the fixed panes together exceed the container they
//!   shrink in proportion.
//! * **Limits.** Each pane may have a `min` and a `max`, in pixels or as a fraction of the
//!   container ([`Limit`]).
//! * **Dragging.** A handle moves space between the panes beside it. With
//!   [`Redistribute::Adjacent`] only those two take part; with [`Redistribute::Nearest`], when
//!   the neighbour has reached its limit the drag takes from the next pane beyond it, and so on.
//! * **Collapsing.** A `collapsible` pane dragged past half its minimum collapses to nothing
//!   (and back out again past half its minimum); [`collapse`](SplitterState::collapse),
//!   [`expand`](SplitterState::expand) and [`toggle`](SplitterState::toggle) do it from code.
//!   A collapsed pane remembers its size.

#![allow(clippy::needless_range_loop)] // parallel arrays of panes, sizes and flags

/// A pane's size.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    /// Fixed, in pixels.
    Px(f32),
    /// Flexible: a weight. Only the proportions between flexible panes matter.
    Flex(f32),
}

/// A pane's minimum or maximum.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Limit {
    Px(f32),
    /// A fraction of the container (0.2 is 20%).
    Fraction(f32),
}

impl Limit {
    fn px(self, total: f32) -> f32 {
        match self {
            Self::Px(p) => p,
            Self::Fraction(f) => f * total,
        }
    }
}

/// One pane's declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct Pane {
    pub size: Size,
    pub min: Option<Limit>,
    pub max: Option<Limit>,
    pub collapsible: bool,
}

impl Pane {
    /// A flexible pane of weight 1.
    pub fn flex() -> Self {
        Self {
            size: Size::Flex(1.0),
            min: None,
            max: None,
            collapsible: false,
        }
    }
    /// A fixed pane of `px` pixels.
    pub fn fixed(px: f32) -> Self {
        Self {
            size: Size::Px(px),
            ..Self::flex()
        }
    }
    /// A flexible pane of `weight`.
    pub fn weight(weight: f32) -> Self {
        Self {
            size: Size::Flex(weight),
            ..Self::flex()
        }
    }
    pub fn min(mut self, l: Limit) -> Self {
        self.min = Some(l);
        self
    }
    pub fn max(mut self, l: Limit) -> Self {
        self.max = Some(l);
        self
    }
    pub fn collapsible(mut self, on: bool) -> Self {
        self.collapsible = on;
        self
    }
}

/// Where a drag takes space from once the neighbour is at its limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Redistribute {
    /// Only the two panes beside the handle.
    #[default]
    Adjacent,
    /// The next pane beyond, and so on, nearest first.
    Nearest,
}

/// What the widget reports.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    /// A handle moved by `delta` pixels (positive: toward the end), in a container whose panes
    /// share `total` pixels (without the handles).
    Resize {
        handle: usize,
        delta: f32,
        total: f32,
    },
    Collapse(usize),
    Expand(usize),
    Toggle(usize),
}

/// The panes, their sizes and which are collapsed.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitterState {
    panes: Vec<Pane>,
    sizes: Vec<Size>,
    collapsed: Vec<bool>,
    redistribute: Redistribute,
}

impl SplitterState {
    pub fn new(panes: Vec<Pane>) -> Self {
        let sizes = panes.iter().map(|p| p.size).collect();
        let collapsed = vec![false; panes.len()];
        Self {
            panes,
            sizes,
            collapsed,
            redistribute: Redistribute::default(),
        }
    }
    pub fn redistribute(mut self, r: Redistribute) -> Self {
        self.redistribute = r;
        self
    }
    pub fn set_redistribute(&mut self, r: Redistribute) {
        self.redistribute = r;
    }
    pub fn len(&self) -> usize {
        self.panes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.panes.is_empty()
    }
    /// The sizes in the unit each pane was declared with (Mantine's `sizes`): fixed panes in
    /// pixels, flexible ones as weights.
    pub fn sizes(&self) -> &[Size] {
        &self.sizes
    }
    /// Sets the sizes (controlled use). Ignored when the count differs.
    pub fn set_sizes(&mut self, sizes: Vec<Size>) {
        if sizes.len() == self.panes.len() {
            self.sizes = sizes;
        }
    }
    pub fn is_collapsed(&self, i: usize) -> bool {
        self.collapsed.get(i).copied().unwrap_or(false)
    }

    // -- limits

    fn lo(&self, i: usize, total: f32) -> f32 {
        self.panes[i].min.map_or(0.0, |l| l.px(total))
    }
    fn hi(&self, i: usize, total: f32) -> f32 {
        self.panes[i].max.map_or(f32::INFINITY, |l| l.px(total))
    }

    // -- layout

    /// The size in pixels of every pane for `total` pixels shared by the panes.
    pub fn resolve(&self, total: f32) -> Vec<f32> {
        let n = self.panes.len();
        let total = total.max(0.0);
        let mut px = vec![0.0_f32; n];
        // Fixed panes first; they shrink in proportion when they do not fit.
        let fixed: f32 = (0..n)
            .filter(|&i| !self.collapsed[i])
            .filter_map(|i| match self.sizes[i] {
                Size::Px(p) => Some(p.max(0.0)),
                _ => None,
            })
            .sum();
        let scale = if fixed > total && fixed > 0.0 {
            total / fixed
        } else {
            1.0
        };
        for i in 0..n {
            if let (false, Size::Px(p)) = (self.collapsed[i], self.sizes[i]) {
                px[i] = p.max(0.0) * scale;
            }
        }
        let room = (total - fixed * scale).max(0.0);
        let flex: Vec<usize> = (0..n)
            .filter(|&i| !self.collapsed[i] && matches!(self.sizes[i], Size::Flex(_)))
            .collect();
        let weight = |i: usize| match self.sizes[i] {
            Size::Flex(w) => w.max(0.0),
            _ => 0.0,
        };
        let sum: f32 = flex.iter().map(|&i| weight(i)).sum();
        for &i in &flex {
            px[i] = if sum > 0.0 {
                room * weight(i) / sum
            } else {
                room / flex.len() as f32
            };
        }
        // Limits: clamp, and hand what that frees (or takes) to the flexible panes that are
        // not at a limit. A few passes settle it.
        for _ in 0..4 {
            let mut slack = 0.0;
            let mut free = Vec::new();
            for &i in &flex {
                let c = px[i].clamp(self.lo(i, total).min(self.hi(i, total)), self.hi(i, total));
                slack += px[i] - c;
                if (c - px[i]).abs() < 1e-3 {
                    free.push(i);
                }
                px[i] = c;
            }
            if slack.abs() < 1e-3 || free.is_empty() {
                break;
            }
            let share = slack / free.len() as f32;
            for i in free {
                px[i] += share;
            }
        }
        px
    }

    /// Writes pixel sizes back in each pane's own unit.
    fn store(&mut self, px: &[f32], total: f32) {
        let fixed: f32 = (0..px.len())
            .filter(|&i| !self.collapsed[i] && matches!(self.sizes[i], Size::Px(_)))
            .map(|i| px[i])
            .sum();
        let room = (total - fixed).max(1.0);
        for i in 0..px.len() {
            if self.collapsed[i] {
                continue;
            }
            self.sizes[i] = match self.sizes[i] {
                Size::Px(_) => Size::Px(px[i]),
                Size::Flex(_) => Size::Flex(px[i] / room),
            };
        }
    }

    // -- dragging

    /// Moves handle `handle` (between panes `handle` and `handle + 1`) by `delta` pixels, with
    /// `total` pixels shared by the panes. Returns whether anything changed.
    pub fn drag(&mut self, handle: usize, delta: f32, total: f32) -> bool {
        if handle + 1 >= self.panes.len() || delta == 0.0 {
            return false;
        }
        let before = self.resolve(total);
        let mut px = before.clone();
        let mut collapsed = self.collapsed.clone();
        if delta > 0.0 {
            transfer(
                self,
                &mut px,
                &mut collapsed,
                total,
                handle,
                handle + 1,
                delta,
                true,
            );
        } else {
            transfer(
                self,
                &mut px,
                &mut collapsed,
                total,
                handle + 1,
                handle,
                -delta,
                false,
            );
        }
        if px == before && collapsed == self.collapsed {
            return false;
        }
        // A pane that collapses keeps the size it had; one that comes back out gets at least
        // its minimum.
        for i in 0..px.len() {
            if collapsed[i] && !self.collapsed[i] {
                self.collapsed[i] = true;
            } else if !collapsed[i] && self.collapsed[i] {
                self.collapsed[i] = false;
            }
        }
        self.store(&px, total);
        true
    }

    // -- collapsing

    /// Collapses pane `i` if it is collapsible.
    pub fn collapse(&mut self, i: usize) -> bool {
        if self.panes.get(i).is_some_and(|p| p.collapsible) && !self.collapsed[i] {
            self.collapsed[i] = true;
            return true;
        }
        false
    }
    /// Brings a collapsed pane back at the size it had.
    pub fn expand(&mut self, i: usize) -> bool {
        if self.collapsed.get(i).copied().unwrap_or(false) {
            self.collapsed[i] = false;
            return true;
        }
        false
    }
    pub fn toggle(&mut self, i: usize) -> bool {
        if self.is_collapsed(i) {
            self.expand(i)
        } else {
            self.collapse(i)
        }
    }

    /// Applies what the widget reported. Returns whether the state changed.
    pub fn apply(&mut self, e: Event) -> bool {
        match e {
            Event::Resize {
                handle,
                delta,
                total,
            } => self.drag(handle, delta, total),
            Event::Collapse(i) => self.collapse(i),
            Event::Expand(i) => self.expand(i),
            Event::Toggle(i) => self.toggle(i),
        }
    }
}

/// Grows pane `grow` by up to `want` pixels, taking from the panes on the other side of the
/// handle: `shrink` first, then (with `Nearest`) the ones beyond it. `forward` is the direction
/// of the panes beyond (to the end when true).
#[allow(clippy::too_many_arguments)]
fn transfer(
    s: &SplitterState,
    px: &mut [f32],
    collapsed: &mut [bool],
    total: f32,
    grow: usize,
    shrink: usize,
    want: f32,
    forward: bool,
) {
    let n = px.len();
    // The panes that give space, nearest first.
    let mut givers = vec![shrink];
    if s.redistribute == Redistribute::Nearest {
        let mut j = shrink;
        loop {
            j = if forward {
                if j + 1 >= n {
                    break;
                }
                j + 1
            } else {
                if j == 0 {
                    break;
                }
                j - 1
            };
            givers.push(j);
        }
    }
    // The growing pane: how much more it can take, and a collapsed one only re-opens past half
    // its minimum.
    let mut room = (s.hi(grow, total) - px[grow]).max(0.0);
    if collapsed[grow] {
        let min = s.lo(grow, total);
        room = room.min(f32::INFINITY);
        if min > 0.0 && want < min / 2.0 {
            return;
        }
    }
    let mut want = want.min(room);
    let mut taken_total = 0.0;
    for j in givers {
        if want <= 1e-3 {
            break;
        }
        if collapsed[j] {
            continue;
        }
        let lo = s.lo(j, total);
        let avail = (px[j] - lo).max(0.0);
        let take = if want <= avail {
            want
        } else if s.panes[j].collapsible && lo > 0.0 && px[j] - want <= lo / 2.0 {
            // Dragged past half the minimum: the pane collapses, giving all it has.
            collapsed[j] = true;
            px[j]
        } else {
            avail
        };
        px[j] -= take;
        taken_total += take;
        want -= take;
    }
    if taken_total > 0.0 {
        px[grow] += taken_total;
        if collapsed[grow] {
            collapsed[grow] = false;
            px[grow] = px[grow].max(s.lo(grow, total));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two() -> SplitterState {
        SplitterState::new(vec![Pane::weight(30.0), Pane::weight(70.0)])
    }

    fn close(a: &[f32], b: &[f32]) -> bool {
        a.len() == b.len() && a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.01)
    }

    #[test]
    fn flexible_panes_share_the_room_by_weight() {
        assert!(close(&two().resolve(1000.0), &[300.0, 700.0]));
        let eq = SplitterState::new(vec![Pane::flex(), Pane::flex(), Pane::flex()]);
        assert!(close(&eq.resolve(900.0), &[300.0, 300.0, 300.0]));
    }

    #[test]
    fn a_fixed_pane_keeps_its_size_when_the_container_changes() {
        let s = SplitterState::new(vec![Pane::fixed(240.0), Pane::flex()]);
        assert!(close(&s.resolve(1000.0), &[240.0, 760.0]));
        assert!(close(&s.resolve(600.0), &[240.0, 360.0]));
    }

    #[test]
    fn fixed_panes_shrink_in_proportion_when_they_do_not_fit() {
        let s = SplitterState::new(vec![Pane::fixed(300.0), Pane::fixed(100.0), Pane::flex()]);
        assert!(close(&s.resolve(200.0), &[150.0, 50.0, 0.0]));
    }

    #[test]
    fn limits_clamp_and_the_rest_goes_to_the_other_flexible_panes() {
        let s = SplitterState::new(vec![
            Pane::weight(10.0).min(Limit::Px(200.0)),
            Pane::weight(90.0),
        ]);
        // 10% of 1000 is 100, below the 200 minimum: the other pane gives up the difference.
        assert!(close(&s.resolve(1000.0), &[200.0, 800.0]));
        let f = SplitterState::new(vec![
            Pane::weight(50.0).max(Limit::Fraction(0.3)),
            Pane::weight(50.0),
        ]);
        assert!(close(&f.resolve(1000.0), &[300.0, 700.0]));
    }

    #[test]
    fn dragging_a_handle_moves_space_between_its_neighbours() {
        let mut s = two();
        assert!(s.drag(0, 100.0, 1000.0));
        assert!(close(&s.resolve(1000.0), &[400.0, 600.0]));
        assert!(s.drag(0, -250.0, 1000.0));
        assert!(close(&s.resolve(1000.0), &[150.0, 850.0]));
        assert!(!s.drag(0, 0.0, 1000.0));
        assert!(!s.drag(1, 10.0, 1000.0), "no such handle");
    }

    #[test]
    fn a_drag_stops_at_the_limits() {
        let mut s = SplitterState::new(vec![
            Pane::weight(50.0).max(Limit::Px(600.0)),
            Pane::weight(50.0).min(Limit::Px(300.0)),
        ]);
        s.drag(0, 1000.0, 1000.0);
        let px = s.resolve(1000.0);
        assert!(
            close(&px, &[600.0, 400.0]),
            "the left maximum stops it: {px:?}"
        );
        let mut t = SplitterState::new(vec![
            Pane::weight(50.0),
            Pane::weight(50.0).min(Limit::Px(300.0)),
        ]);
        t.drag(0, 1000.0, 1000.0);
        assert!(
            close(&t.resolve(1000.0), &[700.0, 300.0]),
            "the right minimum stops it"
        );
    }

    #[test]
    fn fixed_panes_are_written_back_in_pixels_and_flexible_ones_as_weights() {
        let mut s = SplitterState::new(vec![Pane::fixed(240.0), Pane::flex()]);
        s.drag(0, 60.0, 1000.0);
        assert_eq!(s.sizes()[0], Size::Px(300.0));
        let Size::Flex(w) = s.sizes()[1] else {
            panic!("flexible stays flexible")
        };
        assert!(w > 0.0);
        assert!(close(&s.resolve(1000.0), &[300.0, 700.0]));
        // The fixed pane still ignores the container.
        assert!(close(&s.resolve(800.0), &[300.0, 500.0]));
    }

    #[test]
    fn adjacent_stops_at_the_neighbour_and_nearest_takes_from_the_next() {
        let panes = vec![
            Pane::weight(1.0),
            Pane::weight(1.0).min(Limit::Px(100.0)),
            Pane::weight(1.0),
        ];
        let mut adj = SplitterState::new(panes.clone());
        adj.drag(0, 500.0, 900.0);
        let a = adj.resolve(900.0);
        assert!(
            close(&a, &[500.0, 100.0, 300.0]),
            "adjacent: only the neighbour gives: {a:?}"
        );
        let mut near = SplitterState::new(panes).redistribute(Redistribute::Nearest);
        near.drag(0, 500.0, 900.0);
        let n = near.resolve(900.0);
        assert!(
            close(&n, &[800.0, 100.0, 0.0]),
            "nearest: the next gives too: {n:?}"
        );
    }

    #[test]
    fn a_collapsible_pane_collapses_when_dragged_past_half_its_minimum() {
        let mut s = SplitterState::new(vec![
            Pane::weight(30.0).min(Limit::Px(200.0)).collapsible(true),
            Pane::weight(70.0),
        ]);
        // 300 -> 150 is below the minimum but above half of it: it stops at the minimum.
        s.drag(0, -150.0, 1000.0);
        assert!(close(&s.resolve(1000.0), &[200.0, 800.0]));
        assert!(!s.is_collapsed(0));
        // Dragged on past half the minimum: it collapses.
        s.drag(0, -150.0, 1000.0);
        assert!(s.is_collapsed(0));
        assert!(close(&s.resolve(1000.0), &[0.0, 1000.0]));
    }

    #[test]
    fn a_collapsed_pane_comes_back_at_the_size_it_had() {
        let mut s = SplitterState::new(vec![
            Pane::weight(30.0).min(Limit::Px(200.0)).collapsible(true),
            Pane::weight(70.0),
        ]);
        assert!(s.collapse(0));
        assert!(close(&s.resolve(1000.0), &[0.0, 1000.0]));
        assert!(s.expand(0));
        assert!(close(&s.resolve(1000.0), &[300.0, 700.0]));
        assert!(s.toggle(0) && s.is_collapsed(0));
        assert!(s.toggle(0) && !s.is_collapsed(0));
        assert!(
            !SplitterState::new(vec![Pane::flex()]).collapse(0),
            "not collapsible"
        );
    }

    #[test]
    fn a_collapsed_pane_opens_again_when_dragged_out_past_half_its_minimum() {
        let mut s = SplitterState::new(vec![
            Pane::weight(30.0).min(Limit::Px(200.0)).collapsible(true),
            Pane::weight(70.0),
        ]);
        s.collapse(0);
        s.drag(0, 50.0, 1000.0);
        assert!(
            s.is_collapsed(0),
            "less than half the minimum: still collapsed"
        );
        s.drag(0, 150.0, 1000.0);
        assert!(!s.is_collapsed(0));
        assert!(s.resolve(1000.0)[0] >= 200.0 - 0.01);
    }

    #[test]
    fn events_drive_the_state() {
        let mut s = two();
        assert!(s.apply(Event::Resize {
            handle: 0,
            delta: 100.0,
            total: 1000.0
        }));
        assert!(close(&s.resolve(1000.0), &[400.0, 600.0]));
        let mut c = SplitterState::new(vec![Pane::flex().collapsible(true), Pane::flex()]);
        assert!(c.apply(Event::Collapse(0)) && c.is_collapsed(0));
        assert!(c.apply(Event::Expand(0)) && !c.is_collapsed(0));
        assert!(c.apply(Event::Toggle(0)) && c.is_collapsed(0));
    }

    #[test]
    fn set_sizes_ignores_a_wrong_count() {
        let mut s = two();
        s.set_sizes(vec![Size::Flex(1.0)]);
        assert_eq!(s.sizes().len(), 2);
        s.set_sizes(vec![Size::Flex(50.0), Size::Flex(50.0)]);
        assert!(close(&s.resolve(1000.0), &[500.0, 500.0]));
    }
}
