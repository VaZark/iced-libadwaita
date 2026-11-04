//! `adaptive`: choose between independent layouts by window width.
//!
//! A layout is whatever the application builds: a phone layout with a bottom tab bar and a
//! desktop layout with a sidebar need not share a single widget. `adaptive` assumes nothing about
//! their contents. It only maps width classes ([`WidthClass`], so the thresholds follow the
//! user's `[breakpoints]`) to layouts, and when the choice changes runs a close-then-open
//! [`transition`](crate::layouts::transition) instead of cutting.
//!
//! ```text
//! width <  768 px  ->  Layout::Narrow
//! otherwise        ->  Layout::Wide
//! ```
//!
//! Layouts hold only visual state. Navigation state (the current page, open overlays) belongs to
//! the application, so changing layout loses nothing. Keys (`K`) are the application's own type,
//! usually a small enum; the layouts are closures, built only if shown.
//!
//! ```ignore
//! // state
//! adaptive: Adaptive::new(
//!     Layouts::new()
//!         .layout(..WidthClass::Md, Layout::Narrow)
//!         .layout(WidthClass::Md.., Layout::Wide),
//! ),
//!
//! // update
//! Message::Resized(w) => for c in self.adaptive.resize(w, Instant::now()) { /* tell the layout */ },
//! Message::Transition(e) => { self.adaptive.report(e, Instant::now()); }
//! Message::Tick(now) => { self.adaptive.tick(now); }
//!
//! // view
//! adaptive(&self.adaptive)
//!     .layout(Layout::Narrow, || narrow(&self.state))
//!     .layout(Layout::Wide, || wide(&self.state))
//!
//! // subscription
//! self.adaptive.subscription(Message::Tick)
//! ```
//!
//! The layouts do not have to animate. A layout that does nothing special is closed when its
//! budget runs out (30 ms by default, `[motion] transition-close`); one that animates reports
//! [`Event::Closed`] / [`Event::Opened`] when it is done, which lets the next step start sooner.

use crate::Element;
use crate::breakpoints::{Breakpoints, WidthClass};
use crate::layouts::transition::{
    Action, Budgets, Command, Event, FRAME, Phase, Record, Sequence, Transitions,
};
use iced::Subscription;
use std::ops::{Bound, RangeBounds};
use std::time::{Duration, Instant};

/// A map from width classes to layouts.
#[derive(Debug, Clone, PartialEq)]
pub struct Layouts<K> {
    entries: Vec<Entry<K>>,
}

#[derive(Debug, Clone, PartialEq)]
struct Entry<K> {
    min: WidthClass,
    max: WidthClass,
    key: K,
}

impl<K> Default for Layouts<K> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<K: Clone + PartialEq> Layouts<K> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Uses `key` for the width classes in `classes`, e.g. `..WidthClass::Md` or
    /// `WidthClass::Md..`. Where ranges overlap, the first one added wins. An empty range is
    /// ignored.
    pub fn layout(mut self, classes: impl RangeBounds<WidthClass>, key: K) -> Self {
        let min = match classes.start_bound() {
            Bound::Included(c) => Some(*c),
            Bound::Excluded(c) => c.next(),
            Bound::Unbounded => Some(WidthClass::Xs),
        };
        let max = match classes.end_bound() {
            Bound::Included(c) => Some(*c),
            Bound::Excluded(c) => c.prev(),
            Bound::Unbounded => Some(WidthClass::Xxl),
        };
        if let (Some(min), Some(max)) = (min, max)
            && min <= max
        {
            self.entries.push(Entry { min, max, key });
        }
        self
    }

    /// The layout for a width class.
    pub fn for_class(&self, class: WidthClass) -> Option<&K> {
        self.entries
            .iter()
            .find(|e| e.min <= class && class <= e.max)
            .map(|e| &e.key)
    }

    /// Whether `key` declares `class` in its range. A layout whose range is every class supports
    /// everything.
    pub fn allows(&self, key: &K, class: WidthClass) -> bool {
        self.entries
            .iter()
            .any(|e| e.key == *key && e.min <= class && class <= e.max)
    }

    /// Every layout whose range includes `class`, in priority order: the first is what the
    /// width picks, the others are what a manual trigger may choose instead.
    pub fn allowed(&self, class: WidthClass) -> Vec<&K> {
        let mut out: Vec<&K> = Vec::new();
        for e in &self.entries {
            if e.min <= class && class <= e.max && !out.contains(&&e.key) {
                out.push(&e.key);
            }
        }
        out
    }

    /// The layout for `width` on `scale`. Without a layout for the class, the closest one.
    pub fn pick(&self, scale: &Breakpoints, width: f32) -> Option<&K> {
        let class = WidthClass::of_with(scale, width);
        self.for_class(class).or_else(|| {
            self.entries
                .iter()
                .min_by_key(|e| {
                    let (a, b) = (e.min as i32, e.max as i32);
                    let c = class as i32;
                    if c < a { a - c } else { c - b }
                })
                .map(|e| &e.key)
        })
    }

    /// Like [`pick`](Self::pick), but `current` is kept while `width` is within `margin` pixels
    /// of its range, so dragging a window edge across a threshold does not flip layouts every
    /// few pixels.
    pub fn pick_sticky(
        &self,
        scale: &Breakpoints,
        width: f32,
        current: Option<&K>,
        margin: f32,
    ) -> Option<&K> {
        if let Some(cur) = current
            && let Some(e) = self.entries.iter().find(|e| e.key == *cur)
        {
            let lower = e.min.min_width_with(scale);
            let upper = e
                .max
                .next()
                .map_or(f32::INFINITY, |n| n.min_width_with(scale));
            if lower - margin <= width && width < upper + margin {
                return Some(&e.key);
            }
        }
        self.pick(scale, width)
    }
}

/// A manual trigger asked for a layout the current width does not support.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OutOfRange;

impl std::fmt::Display for OutOfRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("the layout does not support this width")
    }
}

impl std::error::Error for OutOfRange {}

/// Pixels past a threshold before the layout changes (the default [`Hysteresis`]).
pub const HYSTERESIS: f32 = 16.0;

/// How long a new width class must hold before the layout changes, for [`Hysteresis::delay`]:
/// the usual range for debouncing a window resize is 100-250 ms.
pub const DELAY: Duration = Duration::from_millis(200);

/// What keeps a window dragged across a threshold from flipping layouts every few pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Hysteresis {
    /// The width must pass the threshold by this many pixels, and come back by as many to
    /// change back. Reacts at once. The default, `Pixels(16.0)`.
    Pixels(f32),
    /// The new layout must be wanted for this long without interruption (a debounce). Ignores
    /// any amount of crossing, but the layout lags the window by the delay.
    Delay(Duration),
}

impl Hysteresis {
    /// A pixel margin ([`HYSTERESIS`] by default).
    pub const fn pixels(px: f32) -> Self {
        Self::Pixels(px)
    }

    /// A delay ([`DELAY`] is a good one).
    pub const fn delay(d: Duration) -> Self {
        Self::Delay(d)
    }
}

impl Default for Hysteresis {
    fn default() -> Self {
        Self::Pixels(HYSTERESIS)
    }
}

impl From<f32> for Hysteresis {
    fn from(px: f32) -> Self {
        Self::Pixels(px)
    }
}

impl From<Duration> for Hysteresis {
    fn from(d: Duration) -> Self {
        Self::Delay(d)
    }
}

/// A layout the width has asked for, waiting out a [`Hysteresis::Delay`].
#[derive(Debug, Clone)]
struct Pending<K> {
    key: K,
    since: Instant,
    budgets: Budgets,
}

/// The coordinator of an adaptive window: which layout is chosen, and the transitions between
/// them. Keep it in your application state.
#[derive(Debug, Clone)]
pub struct Adaptive<K> {
    layouts: Layouts<K>,
    hysteresis: Hysteresis,
    overlap: bool,
    pending: Option<Pending<K>>,
    pinned: Option<K>,
    last_width: Option<f32>,
    current: Option<K>,
    transitions: Transitions<K>,
}

impl<K: Clone + PartialEq> Adaptive<K> {
    pub fn new(layouts: Layouts<K>) -> Self {
        Self {
            layouts,
            hysteresis: Hysteresis::default(),
            overlap: false,
            pending: None,
            pinned: None,
            last_width: None,
            current: None,
            transitions: Transitions::new(),
        }
    }

    /// What keeps the layout from flipping near a threshold: pixels (`16.0`, the default) or a
    /// delay (`Duration::from_millis(200)`). See [`Hysteresis`].
    pub fn hysteresis(mut self, h: impl Into<Hysteresis>) -> Self {
        self.hysteresis = match h.into() {
            Hysteresis::Pixels(px) => Hysteresis::Pixels(px.max(0.0)),
            d => d,
        };
        self
    }

    /// **Overlap** the layouts of a change instead of closing the old one before opening the new:
    /// the incoming layout is shown at once, one layer *below* the outgoing one, which closes on
    /// top of it. The old layout is never replaced by a blank frame, and what the old layout
    /// uncovers when it goes is the new layout. [`adaptive`] draws them in that order. Default
    /// off: close, then open (a layout that cannot show the page the other one holds needs
    /// that order, see [`Sequence::towards`]).
    pub fn overlap(mut self, overlap: bool) -> Self {
        self.overlap = overlap;
        self
    }

    /// [`overlap`](Self::overlap) on an existing coordinator (a settings switch).
    pub fn set_overlap(&mut self, overlap: bool) {
        self.overlap = overlap;
    }

    /// Whether changes overlap (see [`overlap`](Self::overlap)).
    pub fn overlaps(&self) -> bool {
        self.overlap
    }

    /// Forces a layout regardless of the width, with no transition: for a gallery or a test.
    /// The next [`resize`](Self::resize) goes back to following the width.
    pub fn force(&mut self, key: K) {
        self.transitions = Transitions::new();
        self.pending = None;
        self.pinned = None;
        self.transitions.show(key.clone());
        self.current = Some(key);
    }

    /// The layout the width calls for. While a transition runs, it may not be shown yet.
    pub fn current(&self) -> Option<&K> {
        self.current.as_ref()
    }

    /// The layouts the view renders now.
    pub fn shown(&self) -> &[K] {
        self.transitions.shown()
    }

    pub fn phase(&self) -> Phase {
        self.transitions.phase()
    }

    /// Feeds the window width on the configured scale and budgets. See
    /// [`resize_with`](Self::resize_with).
    pub fn resize(&mut self, width: f32, now: Instant) -> Vec<Command<K>> {
        self.resize_with(&Breakpoints::current(), Budgets::current(), width, now)
    }

    /// Feeds the window width. The first call shows the layout at once. After that, a change of
    /// layout starts `Close(old)` then `Open(new)`; send the returned commands to the layouts.
    pub fn resize_with(
        &mut self,
        scale: &Breakpoints,
        budgets: Budgets,
        width: f32,
        now: Instant,
    ) -> Vec<Command<K>> {
        self.last_width = Some(width);
        if let Some(pin) = &self.pinned {
            // A pin holds only while the layout still supports the width.
            if self.layouts.allows(pin, WidthClass::of_with(scale, width)) {
                return Vec::new();
            }
            self.pinned = None;
        }
        let next = match self.hysteresis {
            Hysteresis::Pixels(margin) => {
                self.layouts
                    .pick_sticky(scale, width, self.current.as_ref(), margin)
            }
            // The layout in use stays while its range holds the width, as with pixels.
            Hysteresis::Delay(_) => {
                self.layouts
                    .pick_sticky(scale, width, self.current.as_ref(), 0.0)
            }
        }
        .cloned();
        let Some(next) = next else {
            return Vec::new();
        };
        if let (Hysteresis::Delay(_), Some(cur)) = (self.hysteresis, &self.current) {
            if *cur == next {
                self.pending = None;
                return Vec::new();
            }
            // Wanted again after a different want: the clock restarts.
            if self.pending.as_ref().is_none_or(|p| p.key != next) {
                self.pending = Some(Pending {
                    key: next,
                    since: now,
                    budgets,
                });
            }
            return self.commit_if_due(now);
        }
        self.switch_to(next, budgets, now)
    }

    /// Changes to `next` now: the first layout is shown at once, later ones by transition.
    fn switch_to(&mut self, next: K, budgets: Budgets, now: Instant) -> Vec<Command<K>> {
        self.pending = None;
        match self.current.replace(next.clone()) {
            None => {
                self.transitions.show(next);
                Vec::new()
            }
            Some(old) if old == next => Vec::new(),
            Some(_) if self.overlap => {
                // The new layout opens (and is drawn) below; what is shown closes above it.
                let mut actions = vec![Action::open(next.clone(), budgets.open)];
                for shown in self.transitions.shown() {
                    if *shown != next {
                        actions.push(Action::close(shown.clone(), budgets.close));
                    }
                }
                let seq = Sequence::new().towards(next).together(actions);
                self.transitions.start(seq, now)
            }
            Some(_) => {
                // Close what is shown (a layout still closing from an earlier change finishes
                // first). A layout that never opened has nothing to close.
                let mut seq = Sequence::new().towards(next.clone());
                for shown in self.transitions.shown() {
                    if *shown != next {
                        seq = seq.close(shown.clone(), budgets.close);
                    }
                }
                self.transitions.start(seq.open(next, budgets.open), now)
            }
        }
    }

    fn commit_if_due(&mut self, now: Instant) -> Vec<Command<K>> {
        let Hysteresis::Delay(d) = self.hysteresis else {
            return Vec::new();
        };
        match self.pending.take() {
            Some(p) if now.saturating_duration_since(p.since) >= d => {
                self.switch_to(p.key, p.budgets, now)
            }
            other => {
                self.pending = other;
                Vec::new()
            }
        }
    }

    /// The layout a manual trigger has pinned, if any.
    pub fn pinned(&self) -> Option<&K> {
        self.pinned.as_ref()
    }

    /// A **manual trigger**: `Some(key)` moves to `key` by transition and keeps it while the
    /// width stays in its range; `None` hands control back to the width (it re-evaluates the
    /// last one fed to [`resize`](Self::resize)). Breakpoint triggers and manual triggers are the
    /// same transition; only what starts it differs.
    ///
    /// The layouts' ranges are respected: a layout can be pinned only where its range includes
    /// the current width class (one that supports every class, anywhere), else
    /// [`OutOfRange`]. A resize that leaves the pinned layout's range drops the pin
    /// ([`pinned`](Self::pinned) turns `None`).
    pub fn pin(&mut self, key: Option<K>, now: Instant) -> Result<Vec<Command<K>>, OutOfRange> {
        self.pin_with(&Breakpoints::current(), Budgets::current(), key, now)
    }

    /// [`pin`](Self::pin) on an explicit scale and budgets.
    pub fn pin_with(
        &mut self,
        scale: &Breakpoints,
        budgets: Budgets,
        key: Option<K>,
        now: Instant,
    ) -> Result<Vec<Command<K>>, OutOfRange> {
        let Some(k) = key else {
            self.pending = None;
            self.pinned = None;
            return Ok(match self.last_width {
                Some(w) => self.resize_with(scale, budgets, w, now),
                None => Vec::new(),
            });
        };
        let class = self
            .last_width
            .map(|w| WidthClass::of_with(scale, w))
            .ok_or(OutOfRange)?;
        if !self.layouts.allows(&k, class) {
            return Err(OutOfRange);
        }
        self.pending = None;
        self.pinned = Some(k.clone());
        Ok(self.switch_to(k, budgets, now))
    }

    /// The layouts a manual trigger may choose at `width` (see [`Layouts::allowed`]).
    pub fn allowed_at(&self, scale: &Breakpoints, width: f32) -> Vec<&K> {
        self.layouts.allowed(WidthClass::of_with(scale, width))
    }

    /// A layout reports it is done closing or opening.
    pub fn report(&mut self, event: Event<K>, now: Instant) -> Vec<Command<K>> {
        self.transitions.report(event, now)
    }

    /// Enforces deadlines; call it from the tick of [`subscription`](Self::subscription).
    pub fn tick(&mut self, now: Instant) -> Vec<Command<K>> {
        let mut out = self.commit_if_due(now);
        out.extend(self.transitions.tick(now));
        out
    }

    /// Drains the log of finished commands, for tuning the budgets.
    pub fn take_records(&mut self) -> Vec<Record<K>> {
        self.transitions.take_records()
    }

    /// Ticks while a transition is running, so a deadline is enforced even if no layout reports.
    /// Idle otherwise. `tick` must not capture anything (a message constructor like
    /// `Message::Tick` is right).
    pub fn subscription<M: 'static>(
        &self,
        tick: impl Fn(Instant) -> M + iced::advanced::graphics::futures::MaybeSend + Clone + 'static,
    ) -> Subscription<M> {
        if self.transitions.deadline().is_some() || self.pending.is_some() {
            iced::time::every(FRAME).map(tick)
        } else {
            Subscription::none()
        }
    }
}

/// The view of an [`Adaptive`]: the shown layouts, each built from the closure registered for
/// it. Usually that is one; during an [overlapping](Adaptive::overlap) change it is two, the
/// outgoing layout above the incoming one.
pub struct AdaptiveView<'a, K, M> {
    shown: Vec<K>,
    /// The layout the width calls for: drawn below any other shown layout.
    current: Option<K>,
    built: Vec<(K, Element<'a, M>)>,
}

/// Renders the layouts `state` shows. Register every layout with
/// [`layout`](AdaptiveView::layout); only the shown ones are built. Between a close and an open
/// nothing is shown (a blank frame at most); with [`Adaptive::overlap`] there is no such gap.
pub fn adaptive<'a, K: Clone + PartialEq, M>(state: &Adaptive<K>) -> AdaptiveView<'a, K, M> {
    AdaptiveView {
        shown: state.shown().to_vec(),
        current: state.current().cloned(),
        built: Vec::new(),
    }
}

impl<'a, K: PartialEq, M> AdaptiveView<'a, K, M> {
    pub fn layout(mut self, key: K, build: impl FnOnce() -> Element<'a, M>) -> Self {
        if self.shown.contains(&key) && self.built.iter().all(|(k, _)| *k != key) {
            let element = build();
            self.built.push((key, element));
        }
        self
    }
}

/// The order to draw layouts in, bottom first, as indexes into `keys`: the layout the width
/// wants (`current`) at the bottom (z-1), any other shown layout (one that is closing) above it,
/// and among those the one shown last on top.
fn z_order<K: PartialEq>(keys: &[&K], current: Option<&K>, shown: &[K]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..keys.len()).collect();
    order.sort_by_key(|&i| {
        (
            current != Some(keys[i]),
            shown
                .iter()
                .position(|s| s == keys[i])
                .unwrap_or(usize::MAX),
        )
    });
    order
}

impl<'a, K: PartialEq, M: 'a> From<AdaptiveView<'a, K, M>> for Element<'a, M> {
    fn from(v: AdaptiveView<'a, K, M>) -> Self {
        let mut layers: Vec<Option<(K, Element<'a, M>)>> = v.built.into_iter().map(Some).collect();
        let keys: Vec<&K> = layers.iter().flatten().map(|(k, _)| k).collect();
        let order = z_order(&keys, v.current.as_ref(), &v.shown);
        let mut layers: Vec<Element<'a, M>> = order
            .into_iter()
            .filter_map(|i| layers[i].take().map(|(_, e)| e))
            .collect();
        match layers.len() {
            0 => iced::widget::space().into(),
            1 => layers.pop().expect("one layer"),
            _ => iced::widget::stack(layers).into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layouts::transition::Outcome;
    use std::time::Duration;

    #[derive(Debug, Clone, PartialEq)]
    enum L {
        Narrow,
        Wide,
    }

    fn layouts() -> Layouts<L> {
        Layouts::new()
            .layout(..WidthClass::Md, L::Narrow)
            .layout(WidthClass::Md.., L::Wide)
    }

    const B: Breakpoints = Breakpoints {
        sm: 576.0,
        md: 768.0,
        lg: 992.0,
        xl: 1200.0,
        xxl: 1400.0,
    };
    const BUDGETS: Budgets = Budgets {
        close: Duration::from_millis(30),
        open: Duration::from_millis(30),
    };

    #[test]
    fn ranges_map_classes_to_layouts() {
        let l = layouts();
        assert_eq!(l.for_class(WidthClass::Xs), Some(&L::Narrow));
        assert_eq!(l.for_class(WidthClass::Sm), Some(&L::Narrow));
        assert_eq!(l.for_class(WidthClass::Md), Some(&L::Wide));
        assert_eq!(l.for_class(WidthClass::Xxl), Some(&L::Wide));
        assert_eq!(l.pick(&B, 767.0), Some(&L::Narrow));
        assert_eq!(l.pick(&B, 768.0), Some(&L::Wide));
    }

    #[test]
    fn a_tablet_layout_is_one_more_entry() {
        let l = Layouts::new()
            .layout(..WidthClass::Md, 'p')
            .layout(WidthClass::Md..=WidthClass::Lg, 't')
            .layout(WidthClass::Xl.., 'd');
        assert_eq!(l.pick(&B, 500.0), Some(&'p'));
        assert_eq!(l.pick(&B, 900.0), Some(&'t'));
        assert_eq!(l.pick(&B, 1000.0), Some(&'t'));
        assert_eq!(l.pick(&B, 1200.0), Some(&'d'));
    }

    #[test]
    fn a_gap_uses_the_closest_layout() {
        let l = Layouts::new()
            .layout(..WidthClass::Sm, 'p')
            .layout(WidthClass::Xl.., 'd');
        assert_eq!(l.pick(&B, 800.0), Some(&'p'), "Md is closer to Xs");
        assert_eq!(l.pick(&B, 1000.0), Some(&'d'));
    }

    #[test]
    fn the_scale_is_the_users() {
        let b = Breakpoints { md: 640.0, ..B };
        assert_eq!(layouts().pick(&b, 700.0), Some(&L::Wide));
        assert_eq!(layouts().pick(&B, 700.0), Some(&L::Narrow));
    }

    #[test]
    fn dragging_across_a_threshold_does_not_flicker() {
        let l = layouts();
        let mut cur = l.pick(&B, 700.0).cloned();
        assert_eq!(cur, Some(L::Narrow));
        let mut flips = 0;
        // Wobble across 768 within the 16 px margin.
        for w in [760.0, 770.0, 775.0, 766.0, 780.0, 764.0] {
            let next = l.pick_sticky(&B, w, cur.as_ref(), 16.0).cloned();
            flips += (next != cur) as u32;
            cur = next;
        }
        assert_eq!(flips, 0);
        // Past the margin it does change, and changing back needs the same distance.
        cur = l.pick_sticky(&B, 785.0, cur.as_ref(), 16.0).cloned();
        assert_eq!(cur, Some(L::Wide));
        cur = l.pick_sticky(&B, 755.0, cur.as_ref(), 16.0).cloned();
        assert_eq!(cur, Some(L::Wide));
        cur = l.pick_sticky(&B, 751.0, cur.as_ref(), 16.0).cloned();
        assert_eq!(cur, Some(L::Narrow));
    }

    #[test]
    fn a_delay_waits_for_the_new_layout_to_hold() {
        let t0 = Instant::now();
        let ms = Duration::from_millis;
        let mut a = Adaptive::new(layouts()).hysteresis(ms(200));
        a.resize_with(&B, BUDGETS, 1000.0, t0);
        // Crossing and coming back inside the delay changes nothing, however often.
        assert!(a.resize_with(&B, BUDGETS, 700.0, t0 + ms(10)).is_empty());
        assert!(a.resize_with(&B, BUDGETS, 800.0, t0 + ms(100)).is_empty());
        assert!(a.resize_with(&B, BUDGETS, 700.0, t0 + ms(150)).is_empty());
        assert_eq!(a.current(), Some(&L::Wide));
        // Held for the delay (counted from the last time it was wanted again): it switches,
        // even if the window has stopped moving and only the tick arrives.
        assert!(a.tick(t0 + ms(349)).is_empty());
        let close = a.tick(t0 + ms(350));
        assert!(matches!(
            close.as_slice(),
            [Command::Close {
                target: L::Wide,
                ..
            }]
        ));
        assert_eq!(a.current(), Some(&L::Narrow));
    }

    /// Mobile up to Sm, Tablet from Sm to Lg, Desktop from Lg; Everywhere supports all.
    fn overlapping() -> Layouts<&'static str> {
        Layouts::new()
            .layout(..=WidthClass::Sm, "mobile")
            .layout(WidthClass::Sm..=WidthClass::Md, "tablet")
            .layout(WidthClass::Md.., "desktop")
    }

    #[test]
    fn a_manual_trigger_respects_the_ranges() {
        let t0 = Instant::now();
        let mut a = Adaptive::new(overlapping());
        a.resize_with(&B, Budgets::ZERO, 800.0, t0); // Md: tablet by priority
        assert_eq!(a.shown(), ["tablet"]);
        assert_eq!(a.allowed_at(&B, 800.0), [&"tablet", &"desktop"]);

        // Desktop's range includes Md: accepted, and the width no longer decides while it holds.
        assert!(a.pin_with(&B, Budgets::ZERO, Some("desktop"), t0).is_ok());
        assert_eq!(a.shown(), ["desktop"]);
        a.resize_with(&B, Budgets::ZERO, 1000.0, t0);
        assert_eq!(a.shown(), ["desktop"]);

        // Mobile does not support Md: refused, nothing changes.
        assert_eq!(
            a.pin_with(&B, Budgets::ZERO, Some("mobile"), t0),
            Err(OutOfRange)
        );
        assert_eq!(a.pinned(), Some(&"desktop"));

        // Leaving the pinned layout's range drops the pin and the width decides again.
        a.resize_with(&B, Budgets::ZERO, 500.0, t0);
        assert_eq!(a.pinned(), None);
        assert_eq!(a.shown(), ["mobile"]);
    }

    #[test]
    fn releasing_a_pin_returns_to_the_width() {
        let t0 = Instant::now();
        let mut a = Adaptive::new(overlapping());
        a.resize_with(&B, Budgets::ZERO, 800.0, t0);
        a.pin_with(&B, Budgets::ZERO, Some("desktop"), t0).unwrap();
        // Released, the width decides again. Desktop still supports it, so nothing jumps...
        a.pin_with(&B, Budgets::ZERO, None, t0).unwrap();
        assert_eq!(a.pinned(), None);
        assert_eq!(a.shown(), ["desktop"]);
        // ...until the width leaves its range; then the first layout that supports it wins.
        a.resize_with(&B, Budgets::ZERO, 600.0, t0);
        assert_eq!(a.shown(), ["mobile"]);
    }

    #[test]
    fn a_layout_that_supports_everything_can_be_pinned_anywhere() {
        let t0 = Instant::now();
        let mut a = Adaptive::new(
            Layouts::new()
                .layout(..WidthClass::Md, "compact")
                .layout(.., "any"),
        );
        a.resize_with(&B, Budgets::ZERO, 300.0, t0);
        assert!(a.pin_with(&B, Budgets::ZERO, Some("any"), t0).is_ok());
        a.resize_with(&B, Budgets::ZERO, 1500.0, t0);
        assert_eq!(a.shown(), ["any"], "the pin holds across every class");
    }

    #[test]
    fn hysteresis_takes_pixels_or_a_duration() {
        assert_eq!(Hysteresis::from(8.0), Hysteresis::Pixels(8.0));
        assert_eq!(
            Hysteresis::from(Duration::from_millis(300)),
            Hysteresis::Delay(Duration::from_millis(300))
        );
        assert_eq!(Hysteresis::default(), Hysteresis::Pixels(16.0));
    }

    #[test]
    fn the_first_width_shows_a_layout_without_a_transition() {
        let mut a = Adaptive::new(layouts());
        assert!(
            a.resize_with(&B, BUDGETS, 1000.0, Instant::now())
                .is_empty()
        );
        assert_eq!(a.shown(), [L::Wide]);
        assert_eq!(a.phase(), Phase::Idle);
    }

    #[test]
    fn crossing_a_breakpoint_closes_the_old_layout_then_opens_the_new() {
        let t0 = Instant::now();
        let mut a = Adaptive::new(layouts());
        a.resize_with(&B, BUDGETS, 1000.0, t0);
        let close = a.resize_with(&B, BUDGETS, 600.0, t0);
        assert!(matches!(
            close.as_slice(),
            [Command::Close {
                target: L::Wide,
                ..
            }]
        ));
        assert_eq!(a.current(), Some(&L::Narrow));
        assert_eq!(a.shown(), [L::Wide], "still on screen while it closes");

        let open = a.report(close[0].done(), t0 + Duration::from_millis(5));
        assert!(matches!(
            open.as_slice(),
            [Command::Open {
                target: L::Narrow,
                ..
            }]
        ));
        assert_eq!(a.shown(), [L::Narrow]);
        a.report(open[0].done(), t0 + Duration::from_millis(10));
        assert_eq!(a.phase(), Phase::Idle);
        assert_eq!(a.take_records().len(), 2);
    }

    #[test]
    fn a_layout_that_never_answers_cannot_block() {
        let t0 = Instant::now();
        let mut a = Adaptive::new(layouts());
        a.resize_with(&B, BUDGETS, 1000.0, t0);
        a.resize_with(&B, BUDGETS, 600.0, t0);
        let ms = Duration::from_millis;
        a.tick(t0 + ms(30));
        assert_eq!(a.shown(), [L::Narrow]);
        a.tick(t0 + ms(60));
        assert_eq!(a.phase(), Phase::Idle);
        let outcomes: Vec<_> = a.take_records().iter().map(|r| r.outcome).collect();
        assert_eq!(outcomes, [Outcome::TimedOut, Outcome::TimedOut]);
    }

    #[test]
    fn reduced_motion_swaps_in_one_update() {
        let t0 = Instant::now();
        let mut a = Adaptive::new(layouts());
        a.resize_with(&B, Budgets::ZERO, 1000.0, t0);
        a.resize_with(&B, Budgets::ZERO, 600.0, t0);
        assert_eq!(a.shown(), [L::Narrow]);
        assert_eq!(a.phase(), Phase::Idle);
    }

    #[test]
    fn flipping_back_mid_close_ends_on_the_layout_the_width_wants() {
        let t0 = Instant::now();
        let ms = Duration::from_millis;
        let mut a = Adaptive::new(layouts());
        a.resize_with(&B, BUDGETS, 1000.0, t0);
        let close = a.resize_with(&B, BUDGETS, 600.0, t0);
        // Back to wide before Wide finished closing: it finishes, then reopens. Narrow, which
        // never opened, is not asked to close.
        assert!(a.resize_with(&B, BUDGETS, 1000.0, t0 + ms(5)).is_empty());
        let open = a.report(close[0].done(), t0 + ms(10));
        assert!(matches!(
            open.as_slice(),
            [Command::Open {
                target: L::Wide,
                ..
            }]
        ));
        a.report(open[0].done(), t0 + ms(12));
        assert_eq!(a.shown(), [L::Wide]);
        assert_eq!(a.current(), Some(&L::Wide));
    }

    #[test]
    fn the_view_builds_only_the_shown_layout() {
        let mut a = Adaptive::new(layouts());
        a.resize_with(&B, BUDGETS, 1000.0, Instant::now());
        let mut built = Vec::new();
        let _: Element<'_, ()> = adaptive(&a)
            .layout(L::Narrow, || {
                built.push("narrow");
                iced::widget::space().into()
            })
            .layout(L::Wide, || {
                built.push("wide");
                iced::widget::space().into()
            })
            .into();
        assert_eq!(built, ["wide"]);
    }

    #[test]
    fn overlap_shows_the_new_layout_at_once_below_the_old_one() {
        let t0 = Instant::now();
        let mut a = Adaptive::new(layouts()).overlap(true);
        a.resize_with(&B, BUDGETS, 1000.0, t0);
        let cmds = a.resize_with(&B, BUDGETS, 600.0, t0);
        // Both are shown from the start: no gap, no blank frame between a close and an open.
        assert_eq!(a.shown(), [L::Wide, L::Narrow]);
        assert_eq!(a.current(), Some(&L::Narrow));
        // The old one is told to close and the new one to open, together.
        assert!(cmds.iter().any(|c| matches!(
            c,
            Command::Close {
                target: L::Wide,
                ..
            }
        )));
        assert!(cmds.iter().any(|c| matches!(
            c,
            Command::Open {
                target: L::Narrow,
                ..
            }
        )));
        // When the old one is gone only the new one remains.
        for c in &cmds {
            a.report(c.done(), t0 + Duration::from_millis(5));
        }
        assert_eq!(a.shown(), [L::Narrow]);
        assert_eq!(a.phase(), Phase::Idle);
    }

    #[test]
    fn overlap_off_keeps_close_then_open() {
        let t0 = Instant::now();
        let mut a = Adaptive::new(layouts());
        a.resize_with(&B, BUDGETS, 1000.0, t0);
        a.resize_with(&B, BUDGETS, 600.0, t0);
        assert_eq!(
            a.shown(),
            [L::Wide],
            "the new layout is not shown until the old one closed"
        );
    }

    #[test]
    fn the_incoming_layout_is_drawn_below_the_closing_one() {
        // Shown: old first, new last; `current` is the new one.
        let shown = [L::Wide, L::Narrow];
        let keys = [&L::Wide, &L::Narrow];
        assert_eq!(z_order(&keys, Some(&L::Narrow), &shown), [1, 0]);
        // The order the layouts were registered in does not matter.
        let keys = [&L::Narrow, &L::Wide];
        assert_eq!(z_order(&keys, Some(&L::Narrow), &shown), [0, 1]);
        // One shown: one layer.
        assert_eq!(z_order(&[&L::Narrow], Some(&L::Narrow), &[L::Narrow]), [0]);
    }

    #[test]
    fn the_view_builds_both_layouts_while_they_overlap() {
        let t0 = Instant::now();
        let mut a = Adaptive::new(layouts()).overlap(true);
        a.resize_with(&B, BUDGETS, 1000.0, t0);
        a.resize_with(&B, BUDGETS, 600.0, t0);
        let mut built = Vec::new();
        let _: Element<'_, ()> = adaptive(&a)
            .layout(L::Narrow, || {
                built.push("narrow");
                iced::widget::space().into()
            })
            .layout(L::Wide, || {
                built.push("wide");
                iced::widget::space().into()
            })
            .into();
        built.sort();
        assert_eq!(built, ["narrow", "wide"]);
    }
}
