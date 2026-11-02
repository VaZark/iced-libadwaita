//! Stepper: how the steps of a sequence look and respond, after Mantine's `Stepper`. Pure and
//! controlled: the application owns `active` (the index of the current step, or `count` for
//! the "completed" screen) and replaces it with the result of these operations.
//!
//! * a step's [`Status`] comes from its position against `active` (before: completed, at:
//!   current, after: upcoming) with two per-step overrides, **invalid** (wins) and **loading**;
//! * [`can_click`]: Mantine's `allowStepClick` (per step) and `allowNextStepsSelect` (whether
//!   steps after the active one respond), and never the active step itself;
//! * [`next`], [`prev`], [`clamp`], [`is_completed_screen`]: moving through the sequence;
//!   `next` from the last step lands on `count`, the completed screen;
//! * [`progress`] and [`connector_fill`]: how full the line between two steps is.
//!
//! Relation to [`form`](super::form): the form primitive decides *whether* a user may advance
//! (validity gating, when errors show) and derives each step's [`Status`] from it (a form step
//! is invalid only once its errors were shown); this primitive decides how a sequence of steps
//! *looks and responds* to a plain `active` index. A wizard can use either: a wizard with no
//! gating (an onboarding, an order tracker) needs only this; a gated one asks the form
//! primitive and hands the resulting statuses to the stepper widget. There is one status type:
//! [`Status`], re-exported as [`form::StepStatus`](super::form::StepStatus).
//!
//! ```ignore
//! let mut s = StepperState::new(3);       // steps 0..3, active 0
//! s.forward();                           // active 1
//! s.status(0, false, false);              // Completed
//! s.can_click(2, None);                   // true (allow_next defaults to true)
//! s.forward(); s.forward();              // active 3: the completed screen
//! ```

/// How a step shows. Used by `Stepper` and, through the form primitive, by `StepIndicator`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Before the current step (and valid, where validity applies).
    Completed,
    /// The step being worked on.
    Current,
    /// Not reached yet.
    Upcoming,
    /// Needs attention (a refused Next, a failed step): drawn with the error color.
    Invalid,
    /// Working in the background (a request in flight): drawn with a loading mark.
    Loading,
}

/// How full the line between two steps is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Fill {
    Empty,
    /// A fraction strictly between 0 and 1.
    Partial(f32),
    Full,
}

impl Fill {
    /// The fill as a fraction, `0.0..=1.0`.
    pub fn fraction(self) -> f32 {
        match self {
            Fill::Empty => 0.0,
            Fill::Partial(f) => f,
            Fill::Full => 1.0,
        }
    }
}

/// `active` limited to `0..=count` (`count` is the completed screen).
pub fn clamp(active: usize, count: usize) -> usize {
    active.min(count)
}

/// The step after `active`; from the last step it is `count` (the completed screen), and it
/// stays there.
pub fn next(active: usize, count: usize) -> usize {
    (active + 1).min(count)
}

/// The step before `active` (from the completed screen: the last step); `0` stays `0`.
pub fn prev(active: usize, count: usize) -> usize {
    clamp(active, count).saturating_sub(1)
}

/// Whether Next would move (not on the completed screen).
pub fn can_next(active: usize, count: usize) -> bool {
    active < count
}

/// Whether Back would move.
pub fn can_prev(active: usize) -> bool {
    active > 0
}

/// Past the last step: every step is completed and the "completed" content shows.
pub fn is_completed_screen(active: usize, count: usize) -> bool {
    count > 0 && active >= count
}

/// The displayed number of `step` (1-based).
pub fn number(step: usize) -> usize {
    step + 1
}

/// The status of `step`. `invalid` wins over `loading`, which wins over the position.
pub fn status(step: usize, active: usize, invalid: bool, loading: bool) -> Status {
    if invalid {
        Status::Invalid
    } else if loading {
        Status::Loading
    } else if step < active {
        Status::Completed
    } else if step == active {
        Status::Current
    } else {
        Status::Upcoming
    }
}

/// Whether a click on `step` should be answered.
///
/// * the active step never is (there is nowhere to go);
/// * `over` is the step's own setting: `Some(false)` (Mantine `allowStepClick={false}`) always
///   refuses, `Some(true)` always allows (also past the active step with `allow_next` off);
/// * otherwise steps before `active` always respond, and steps after it only with
///   `allow_next` (Mantine `allowNextStepsSelect`).
pub fn can_click(
    step: usize,
    active: usize,
    count: usize,
    allow_next: bool,
    over: Option<bool>,
) -> bool {
    if step >= count || step == active {
        return false;
    }
    match over {
        Some(allowed) => allowed,
        None => step < active || allow_next,
    }
}

/// The share of the sequence behind `active`: `0.0` at the first step, `1.0` on the completed
/// screen.
pub fn progress(active: usize, count: usize) -> f32 {
    if count == 0 {
        return 0.0;
    }
    clamp(active, count) as f32 / count as f32
}

/// The line after `step` (joining it to `step + 1`): full when `step` is behind `active`, empty
/// when ahead; the line leaving the active step takes `current` (the active step's own
/// progress, `0.0..=1.0`, `0.0` when unknown). The last step has no line.
pub fn connector_fill(step: usize, active: usize, count: usize, current: f32) -> Fill {
    if step + 1 >= count {
        return Fill::Empty;
    }
    if step < active {
        Fill::Full
    } else if step == active {
        let f = current.clamp(0.0, 1.0);
        if f <= 0.0 {
            Fill::Empty
        } else if f >= 1.0 {
            Fill::Full
        } else {
            Fill::Partial(f)
        }
    } else {
        Fill::Empty
    }
}

/// The controlled state, for apps that want methods instead of free functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepperState {
    active: usize,
    count: usize,
    /// Whether steps after the active one respond to a click.
    pub allow_next: bool,
}

impl StepperState {
    /// `count` steps, on the first.
    pub fn new(count: usize) -> Self {
        Self {
            active: 0,
            count,
            allow_next: true,
        }
    }
    /// Starts on `active` (clamped).
    pub fn at(mut self, active: usize) -> Self {
        self.active = clamp(active, self.count);
        self
    }
    pub fn allow_next_steps_select(mut self, allow: bool) -> Self {
        self.allow_next = allow;
        self
    }
    pub fn active(&self) -> usize {
        self.active
    }
    pub fn count(&self) -> usize {
        self.count
    }
    /// Moves forward; `true` when it moved.
    pub fn forward(&mut self) -> bool {
        let to = next(self.active, self.count);
        std::mem::replace(&mut self.active, to) != to
    }
    /// Moves back; `true` when it moved.
    pub fn back(&mut self) -> bool {
        let to = prev(self.active, self.count);
        std::mem::replace(&mut self.active, to) != to
    }
    /// Jumps to `step` (clamped); `true` when it moved.
    pub fn goto(&mut self, step: usize) -> bool {
        let to = clamp(step, self.count);
        std::mem::replace(&mut self.active, to) != to
    }
    pub fn is_first(&self) -> bool {
        self.active == 0
    }
    /// On the last step (not yet past it).
    pub fn is_last(&self) -> bool {
        self.count > 0 && self.active + 1 == self.count
    }
    pub fn is_completed(&self) -> bool {
        is_completed_screen(self.active, self.count)
    }
    pub fn status(&self, step: usize, invalid: bool, loading: bool) -> Status {
        status(step, self.active, invalid, loading)
    }
    /// Whether a click on `step` should be answered, given the step's own setting.
    pub fn can_click(&self, step: usize, over: Option<bool>) -> bool {
        can_click(step, self.active, self.count, self.allow_next, over)
    }
    pub fn progress(&self) -> f32 {
        progress(self.active, self.count)
    }
    pub fn connector_fill(&self, step: usize, current: f32) -> Fill {
        connector_fill(step, self.active, self.count, current)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_follows_position_with_overrides() {
        let s = |i, inv, load| status(i, 1, inv, load);
        assert_eq!(s(0, false, false), Status::Completed);
        assert_eq!(s(1, false, false), Status::Current);
        assert_eq!(s(2, false, false), Status::Upcoming);
        // Loading and invalid apply anywhere; invalid wins.
        assert_eq!(s(2, false, true), Status::Loading);
        assert_eq!(s(0, true, true), Status::Invalid);
        assert_eq!(s(1, true, false), Status::Invalid);
    }

    #[test]
    fn moving_stops_at_the_completed_screen_and_the_start() {
        assert_eq!(next(0, 3), 1);
        assert_eq!(next(2, 3), 3);
        assert_eq!(next(3, 3), 3);
        assert_eq!(prev(3, 3), 2);
        assert_eq!(prev(0, 3), 0);
        assert_eq!(prev(9, 3), 2);
        assert!(can_next(2, 3) && !can_next(3, 3));
        assert!(!can_prev(0) && can_prev(1));
        assert_eq!(clamp(9, 3), 3);
        assert!(is_completed_screen(3, 3) && !is_completed_screen(2, 3));
        assert!(!is_completed_screen(0, 0));
        assert_eq!(number(0), 1);
    }

    #[test]
    fn click_rules() {
        // Active step never; out of range never.
        assert!(!can_click(1, 1, 4, true, None));
        assert!(!can_click(4, 1, 4, true, Some(true)));
        // Back always; next with allow_next only.
        assert!(can_click(0, 1, 4, false, None));
        assert!(can_click(2, 1, 4, true, None));
        assert!(!can_click(2, 1, 4, false, None));
        // The step's own setting wins both ways.
        assert!(!can_click(0, 1, 4, true, Some(false)));
        assert!(can_click(3, 1, 4, false, Some(true)));
        // On the completed screen every step is behind.
        assert!(can_click(2, 4, 4, false, None));
    }

    #[test]
    fn progress_and_connectors() {
        assert_eq!(progress(0, 4), 0.0);
        assert_eq!(progress(2, 4), 0.5);
        assert_eq!(progress(9, 4), 1.0);
        assert_eq!(progress(0, 0), 0.0);
        let f = |i, a| connector_fill(i, a, 4, 0.0);
        assert_eq!(f(0, 0), Fill::Empty);
        assert_eq!(f(0, 1), Fill::Full);
        assert_eq!(f(1, 1), Fill::Empty);
        assert_eq!(f(2, 4), Fill::Full);
        // No line after the last step.
        assert_eq!(f(3, 4), Fill::Empty);
        // The line leaving the active step shows its progress.
        assert_eq!(connector_fill(1, 1, 4, 0.4), Fill::Partial(0.4));
        assert_eq!(connector_fill(1, 1, 4, 1.5), Fill::Full);
        assert_eq!(connector_fill(1, 1, 4, -1.0), Fill::Empty);
        assert_eq!(connector_fill(2, 1, 4, 0.9), Fill::Empty);
        assert_eq!(Fill::Partial(0.25).fraction(), 0.25);
    }

    #[test]
    fn state_methods_report_movement() {
        let mut s = StepperState::new(2);
        assert!(s.is_first() && !s.back());
        assert!(s.forward() && s.is_last());
        assert!(s.forward() && s.is_completed());
        assert!(!s.forward());
        assert!(s.back() && s.active() == 1);
        assert!(s.goto(0) && !s.goto(0));
        assert!(s.goto(9) && s.active() == 2);
        assert_eq!(StepperState::new(3).at(7).active(), 3);
        let s = StepperState::new(3).at(1).allow_next_steps_select(false);
        assert!(!s.can_click(2, None) && s.can_click(0, None) && s.can_click(2, Some(true)));
        assert_eq!(s.status(0, false, false), Status::Completed);
        assert_eq!(s.connector_fill(0, 0.0), Fill::Full);
        assert_eq!(s.progress(), 1.0 / 3.0);
    }
}
