//! Headless logic of a spoiler: show part of something and reveal the rest on request. After
//! Mantine's [Spoiler](https://mantine.dev/core/spoiler/).
//!
//! Two kinds of "part":
//!
//! * **Part of the height.** Content is clipped to a `max_height` and a control shows the rest
//!   ([`overflows`], [`height`]). The control only exists when there is something to reveal.
//! * **Part of the data.** A list of `total` items of which only the first `shown` are built.
//!   [`SpoilerState`] keeps `shown`; a press reveals the next `step` items (or all) and "Hide"
//!   returns to the first `initial`. Only the visible items are ever built or laid out, so a
//!   long list costs what the visible part costs: this is the lazily rendered spoiler.
//!
//! Pure and controlled: the app owns the state and applies [`Event`]s.
//!
//! ```
//! use iced_libadwaita::primitives::spoiler::SpoilerState;
//!
//! let mut s = SpoilerState::new(100, 5).step(20);
//! assert_eq!(s.visible(), 0..5);
//! s.more();
//! assert_eq!(s.visible(), 0..25);
//! assert_eq!(s.hidden(), 75);
//! s.less();
//! assert_eq!(s.shown(), 5);
//! ```

use std::ops::Range;

/// Whether `content` of this height needs a control at all: it is taller than `max_height`.
pub fn overflows(content: f32, max_height: f32) -> bool {
    content > max_height + 0.5
}

/// The height shown while `t` (`0.0` collapsed, `1.0` expanded) of the way open. Content that fits
/// is never clamped.
pub fn height(content: f32, max_height: f32, t: f32) -> f32 {
    if !overflows(content, max_height) {
        return content;
    }
    max_height + (content - max_height) * t.clamp(0.0, 1.0)
}

/// What the control reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// Show more: the next chunk, or all.
    More,
    /// Hide: back to the first `initial` items.
    Less,
}

/// How many of `total` items are shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpoilerState {
    total: usize,
    initial: usize,
    /// How many more a press reveals; `None`: all of them.
    step: Option<usize>,
    shown: usize,
}

impl SpoilerState {
    /// `total` items, the first `initial` shown.
    pub fn new(total: usize, initial: usize) -> Self {
        Self {
            total,
            initial: initial.min(total),
            step: None,
            shown: initial.min(total),
        }
    }
    /// A press reveals `n` more instead of all of them.
    pub fn step(mut self, n: usize) -> Self {
        self.step = Some(n.max(1));
        self
    }
    pub fn total(&self) -> usize {
        self.total
    }
    pub fn shown(&self) -> usize {
        self.shown
    }
    /// The indices to build.
    pub fn visible(&self) -> Range<usize> {
        0..self.shown
    }
    /// How many are not shown.
    pub fn hidden(&self) -> usize {
        self.total - self.shown
    }
    /// How many the next press would reveal.
    pub fn next_chunk(&self) -> usize {
        self.step.map_or(self.hidden(), |s| s.min(self.hidden()))
    }
    /// Whether there is anything to reveal.
    pub fn can_expand(&self) -> bool {
        self.shown < self.total
    }
    /// Whether "Hide" would change anything.
    pub fn can_collapse(&self) -> bool {
        self.shown > self.initial
    }
    /// Whether a control is needed at all: there is more than the first part.
    pub fn needs_control(&self) -> bool {
        self.total > self.initial
    }
    /// Reveals the next chunk. Returns whether anything changed.
    pub fn more(&mut self) -> bool {
        let to = self.shown + self.next_chunk();
        std::mem::replace(&mut self.shown, to) != to
    }
    /// Back to the first part. Returns whether anything changed.
    pub fn less(&mut self) -> bool {
        std::mem::replace(&mut self.shown, self.initial) != self.initial
    }
    /// The data changed length; the shown count follows within it.
    pub fn set_total(&mut self, total: usize) {
        self.total = total;
        self.initial = self.initial.min(total);
        self.shown = self.shown.min(total).max(self.initial);
    }
    pub fn apply(&mut self, e: Event) -> bool {
        match e {
            Event::More => self.more(),
            Event::Less => self.less(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_content_taller_than_the_maximum_needs_a_control() {
        assert!(overflows(300.0, 100.0));
        assert!(!overflows(100.0, 100.0));
        assert!(
            !overflows(100.3, 100.0),
            "a fraction of a pixel is not worth a control"
        );
    }

    #[test]
    fn the_height_runs_from_the_maximum_to_the_whole_content() {
        assert_eq!(height(300.0, 100.0, 0.0), 100.0);
        assert_eq!(height(300.0, 100.0, 1.0), 300.0);
        assert_eq!(height(300.0, 100.0, 0.5), 200.0);
        assert_eq!(height(300.0, 100.0, 9.0), 300.0, "clamped");
        assert_eq!(
            height(80.0, 100.0, 0.0),
            80.0,
            "content that fits is not clamped"
        );
    }

    #[test]
    fn a_press_reveals_all_or_a_chunk() {
        let mut all = SpoilerState::new(50, 5);
        assert!(all.more());
        assert_eq!(all.shown(), 50);
        assert!(!all.more(), "nothing left");
        let mut by = SpoilerState::new(50, 5).step(20);
        by.more();
        assert_eq!(by.visible(), 0..25);
        by.more();
        by.more();
        assert_eq!(by.shown(), 50, "the last chunk is short");
        assert_eq!(by.next_chunk(), 0);
    }

    #[test]
    fn hide_goes_back_to_the_first_part() {
        let mut s = SpoilerState::new(50, 5).step(10);
        s.more();
        s.more();
        assert!(s.can_collapse());
        assert!(s.less());
        assert_eq!(s.shown(), 5);
        assert!(!s.can_collapse() && !s.less());
    }

    #[test]
    fn a_short_list_needs_no_control() {
        let s = SpoilerState::new(3, 5);
        assert_eq!(s.shown(), 3);
        assert!(!s.needs_control() && !s.can_expand());
        assert_eq!(s.hidden(), 0);
    }

    #[test]
    fn the_counts_follow_the_data() {
        let mut s = SpoilerState::new(100, 10).step(30);
        s.more();
        s.set_total(25);
        assert_eq!(s.shown(), 25);
        assert_eq!(s.hidden(), 0);
        s.set_total(5);
        assert_eq!((s.shown(), s.visible()), (5, 0..5));
        s.set_total(200);
        assert!(s.can_expand());
    }

    #[test]
    fn events_drive_the_state() {
        let mut s = SpoilerState::new(20, 5);
        assert!(s.apply(Event::More));
        assert_eq!(s.shown(), 20);
        assert!(s.apply(Event::Less));
        assert_eq!(s.shown(), 5);
    }
}
