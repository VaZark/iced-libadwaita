//! Ordered, time-bounded transitions: close this, then open that, each within a time budget,
//! with a report back when it is done.
//!
//! Components animate themselves, but nothing says when an animation ends, so "the old layout
//! leaves before the new one arrives" could only be guessed with fixed delays. This module is the
//! protocol and the state machine that fixes that. It holds no widgets and reads no clock: every
//! call takes the current [`Instant`], so it is plain data you keep in your application state and
//! test without a window.
//!
//! ```text
//! coordinator -> target   Command::Close { target, within, generation, closing_for }
//! coordinator -> target   Command::Open  { target, within, generation, opening_for }
//! target -> coordinator   Event::Closed  { target, generation }
//! target -> coordinator   Event::Opened  { target, generation }
//! ```
//!
//! The coordinator ([`Transitions`]) owns the clock and decides the [`Outcome`] of each command
//! (a target only says "done"): [`Outcome::Done`] or [`Outcome::TimedOut`].
//!
//! `closing_for` / `opening_for` name the layout the whole change is heading to (set with
//! [`Sequence::towards`]). A layout that holds a page the destination cannot show (a page locked
//! to one layout) can navigate elsewhere when it sees them: **state is always the application's
//! concern**, and where layouts conflict, resolving it is the developer's job.
//!
//! The protocol is for **layouts** (global ideas). Popovers, sheets and panels are components a
//! layout implements, and animate themselves inside it.
//!
//! 1. A target fits its animation into `within`, starts at once and reports when done. A zero
//!    `within` (animations disabled, reduced motion) is resolved by the coordinator in the same
//!    call as [`Outcome::Done`]; the target should just jump to its end state.
//! 2. At the deadline the coordinator hard-closes the target (it is no longer in
//!    [`Transitions::is_shown`], so the view stops rendering it) and records
//!    [`Outcome::TimedOut`]. A stuck target never blocks the next step. A late `Open` is simply
//!    shown: there is nothing to hard-open.
//! 3. One report per command: every command carries a generation, and a late, duplicate or
//!    unknown report is ignored.
//! 4. Any outcome lets the sequence continue. Outcomes are only recorded
//!    ([`Transitions::take_records`]) for logs and tuning.
//! 5. Budgets come from the desktop config, beside the other `[motion]` settings
//!    ([`Budgets::current`]).
//!
//! A [`Sequence`] is an ordered list of steps; a step is one action or a group of actions run
//! together. Starting a sequence replaces the steps that have not started but lets the step in
//! flight finish within its own budget, so nothing is left half-closed.
//!
//! ```
//! use iced_adaptive_design::layouts::transition::{Event, Sequence, Transitions};
//! use std::time::{Duration, Instant};
//!
//! #[derive(Clone, PartialEq, Debug)]
//! enum Layout { Wide, Narrow }
//!
//! let t0 = Instant::now();
//! let ms = Duration::from_millis;
//! let mut t = Transitions::new();
//! t.show(Layout::Wide);
//!
//! // Close(Wide, 30 ms), then Open(Narrow, 30 ms).
//! let cmds = t.start(Sequence::new().close(Layout::Wide, ms(30)).open(Layout::Narrow, ms(30)), t0);
//! assert_eq!(cmds.len(), 1); // only the Close has been sent
//!
//! // The Wide layout reports 5 ms in; the coordinator sends the Open.
//! let cmds = t.report(Event::Closed { target: Layout::Wide, generation: cmds[0].generation() }, t0 + ms(5));
//! assert_eq!(cmds.len(), 1);
//! assert!(t.is_shown(&Layout::Narrow) && !t.is_shown(&Layout::Wide));
//! ```

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// One frame at 60 Hz: how often a coordinator is ticked while a deadline is pending.
pub const FRAME: Duration = Duration::from_millis(16);

/// How a command ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Outcome {
    /// The target reported in time (or the budget was zero).
    Done,
    /// The time ran out and the coordinator closed or opened the target itself.
    TimedOut,
}

/// Whether an action closes or opens its target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Close,
    Open,
}

/// One close or open of one target, within a time.
#[derive(Debug, Clone, PartialEq)]
pub struct Action<T> {
    pub kind: Kind,
    pub target: T,
    pub within: Duration,
}

impl<T> Action<T> {
    pub fn close(target: T, within: Duration) -> Self {
        Self {
            kind: Kind::Close,
            target,
            within,
        }
    }

    pub fn open(target: T, within: Duration) -> Self {
        Self {
            kind: Kind::Open,
            target,
            within,
        }
    }
}

/// Coordinator to target.
#[derive(Debug, Clone, PartialEq)]
pub enum Command<T> {
    /// Close within `within`, then report [`Event::Closed`] with this `generation`.
    Close {
        target: T,
        within: Duration,
        generation: u64,
        /// The layout the change is heading to, if the sequence names one.
        closing_for: Option<T>,
    },
    /// Open within `within`, then report [`Event::Opened`] with this `generation`.
    Open {
        target: T,
        within: Duration,
        generation: u64,
        /// The layout the change is heading to, if the sequence names one.
        opening_for: Option<T>,
    },
}

impl<T> Command<T> {
    pub fn target(&self) -> &T {
        match self {
            Self::Close { target, .. } | Self::Open { target, .. } => target,
        }
    }

    pub fn within(&self) -> Duration {
        match self {
            Self::Close { within, .. } | Self::Open { within, .. } => *within,
        }
    }

    /// The layout the change is heading to (`closing_for` / `opening_for`).
    pub fn towards(&self) -> Option<&T> {
        match self {
            Self::Close { closing_for, .. } => closing_for.as_ref(),
            Self::Open { opening_for, .. } => opening_for.as_ref(),
        }
    }

    pub fn generation(&self) -> u64 {
        match self {
            Self::Close { generation, .. } | Self::Open { generation, .. } => *generation,
        }
    }

    /// The event that answers this command.
    pub fn done(&self) -> Event<T>
    where
        T: Clone,
    {
        match self {
            Self::Close {
                target, generation, ..
            } => Event::Closed {
                target: target.clone(),
                generation: *generation,
            },
            Self::Open {
                target, generation, ..
            } => Event::Opened {
                target: target.clone(),
                generation: *generation,
            },
        }
    }
}

/// Target to coordinator: the command with this `generation` is done. Route it through your
/// `update` like any other message.
#[derive(Debug, Clone, PartialEq)]
pub enum Event<T> {
    Closed { target: T, generation: u64 },
    Opened { target: T, generation: u64 },
}

/// The log entry of a finished command.
#[derive(Debug, Clone, PartialEq)]
pub struct Record<T> {
    pub kind: Kind,
    pub target: T,
    pub within: Duration,
    pub outcome: Outcome,
}

/// An ordered list of steps; a step is a group of actions run at the same time.
#[derive(Debug, Clone, PartialEq)]
pub struct Sequence<T> {
    steps: Vec<Vec<Action<T>>>,
    towards: Option<T>,
}

impl<T> Default for Sequence<T> {
    fn default() -> Self {
        Self {
            steps: Vec::new(),
            towards: None,
        }
    }
}

/// A new empty [`Sequence`].
pub fn sequence<T>() -> Sequence<T> {
    Sequence::new()
}

impl<T> Sequence<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Names the layout the change is heading to; every command carries it
    /// (`closing_for` / `opening_for`).
    pub fn towards(mut self, layout: T) -> Self {
        self.towards = Some(layout);
        self
    }

    /// Adds a step that closes `target`.
    pub fn close(self, target: T, within: Duration) -> Self {
        self.together([Action::close(target, within)])
    }

    /// Adds a step that opens `target`.
    pub fn open(self, target: T, within: Duration) -> Self {
        self.together([Action::open(target, within)])
    }

    /// Adds a step whose actions run at the same time; it ends when all have.
    pub fn together(mut self, actions: impl IntoIterator<Item = Action<T>>) -> Self {
        let step: Vec<_> = actions.into_iter().collect();
        if !step.is_empty() {
            self.steps.push(step);
        }
        self
    }

    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// The longest the sequence can take: the sum of its step budgets, a group counting its
    /// longest action.
    pub fn budget(&self) -> Duration {
        self.steps
            .iter()
            .map(|s| s.iter().map(|a| a.within).max().unwrap_or_default())
            .sum()
    }
}

/// The time budgets of a layout change, from the desktop config.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Budgets {
    pub close: Duration,
    pub open: Duration,
}

impl Budgets {
    pub const ZERO: Budgets = Budgets {
        close: Duration::ZERO,
        open: Duration::ZERO,
    };

    /// `[motion] transition-close` / `transition-open` (milliseconds), or [`Budgets::ZERO`] when
    /// layout changes do not animate (animations disabled, reduced motion, `layout=none`).
    pub fn current() -> Self {
        if !crate::layouts::animation::Trigger::Layout.animated() {
            return Self::ZERO;
        }
        let c = crate::config::current();
        Self {
            close: Duration::from_secs_f32(c.transition_close_ms / 1000.0),
            open: Duration::from_secs_f32(c.transition_open_ms / 1000.0),
        }
    }
}

#[derive(Debug, Clone)]
struct InFlight<T> {
    action: Action<T>,
    generation: u64,
    deadline: Instant,
}

/// Where a coordinator is in a sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Closing,
    Opening,
}

/// The coordinator: runs [`Sequence`]s, owns the clock and tracks which targets are shown.
#[derive(Debug, Clone)]
pub struct Transitions<T> {
    queued: VecDeque<Vec<Action<T>>>,
    towards: Option<T>,
    in_flight: Vec<InFlight<T>>,
    shown: Vec<T>,
    next_generation: u64,
    records: Vec<Record<T>>,
}

impl<T> Default for Transitions<T> {
    fn default() -> Self {
        Self {
            queued: VecDeque::new(),
            towards: None,
            in_flight: Vec::new(),
            shown: Vec::new(),
            next_generation: 1,
            records: Vec::new(),
        }
    }
}

impl<T: Clone + PartialEq> Transitions<T> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Marks `target` as shown without a transition (the initial state).
    pub fn show(&mut self, target: T) {
        if !self.is_shown(&target) {
            self.shown.push(target);
        }
    }

    /// Whether the view should render `target`: opened (or opening), and not yet closed.
    pub fn is_shown(&self, target: &T) -> bool {
        self.shown.contains(target)
    }

    /// The shown targets, oldest first.
    pub fn shown(&self) -> &[T] {
        &self.shown
    }

    pub fn phase(&self) -> Phase {
        if self.in_flight.is_empty() {
            Phase::Idle
        } else if self.in_flight.iter().any(|f| f.action.kind == Kind::Close) {
            Phase::Closing
        } else {
            Phase::Opening
        }
    }

    pub fn is_idle(&self) -> bool {
        self.phase() == Phase::Idle
    }

    /// When the coordinator next needs a [`tick`](Self::tick): the earliest deadline in flight.
    pub fn deadline(&self) -> Option<Instant> {
        self.in_flight.iter().map(|f| f.deadline).min()
    }

    /// Starts `sequence`. Steps still waiting from an earlier sequence are replaced; the step in
    /// flight finishes within its own budget first. Returns the commands to send now.
    pub fn start(&mut self, sequence: Sequence<T>, now: Instant) -> Vec<Command<T>> {
        let mut out = Vec::new();
        self.expire(now);
        self.queued = sequence.steps.into();
        self.towards = sequence.towards;
        self.advance(now, &mut out);
        out
    }

    /// A target reports it is done. Returns the commands of the next step, if this ended one.
    /// A report that is stale, duplicate or unknown (or that arrives after its deadline, which
    /// has already been recorded as a timeout) is ignored.
    pub fn report(&mut self, event: Event<T>, now: Instant) -> Vec<Command<T>> {
        let mut out = Vec::new();
        self.expire(now);
        let (kind, target, generation) = match event {
            Event::Closed { target, generation } => (Kind::Close, target, generation),
            Event::Opened { target, generation } => (Kind::Open, target, generation),
        };
        if let Some(i) = self.in_flight.iter().position(|f| {
            f.generation == generation && f.action.kind == kind && f.action.target == target
        }) {
            let f = self.in_flight.remove(i);
            self.finish(f.action, Outcome::Done);
        }
        self.advance(now, &mut out);
        out
    }

    /// Enforces deadlines: call it when [`deadline`](Self::deadline) passes (see
    /// `Adaptive::subscription`). Returns the commands of the next step, if one ended.
    pub fn tick(&mut self, now: Instant) -> Vec<Command<T>> {
        let mut out = Vec::new();
        self.expire(now);
        self.advance(now, &mut out);
        out
    }

    /// Drains the log of finished commands.
    pub fn take_records(&mut self) -> Vec<Record<T>> {
        std::mem::take(&mut self.records)
    }

    fn finish(&mut self, action: Action<T>, outcome: Outcome) {
        // Closing and opening are final even when the target never answered: the view stops
        // rendering a closed target and renders an opened one.
        match action.kind {
            Kind::Close => self.shown.retain(|t| *t != action.target),
            Kind::Open => self.show(action.target.clone()),
        }
        self.records.push(Record {
            kind: action.kind,
            target: action.target,
            within: action.within,
            outcome,
        });
    }

    fn expire(&mut self, now: Instant) {
        let mut i = 0;
        while i < self.in_flight.len() {
            if self.in_flight[i].deadline <= now {
                let f = self.in_flight.remove(i);
                self.finish(f.action, Outcome::TimedOut);
            } else {
                i += 1;
            }
        }
    }

    /// Starts queued steps while nothing is in flight (a step of zero budgets ends at once).
    fn advance(&mut self, now: Instant, out: &mut Vec<Command<T>>) {
        while self.in_flight.is_empty() {
            let Some(step) = self.queued.pop_front() else {
                return;
            };
            for action in step {
                let generation = self.next_generation;
                self.next_generation += 1;
                let (target, within) = (action.target.clone(), action.within);
                let towards = self.towards.clone();
                out.push(match action.kind {
                    Kind::Close => Command::Close {
                        target,
                        within,
                        generation,
                        closing_for: towards,
                    },
                    Kind::Open => Command::Open {
                        target,
                        within,
                        generation,
                        opening_for: towards,
                    },
                });
                if action.kind == Kind::Open {
                    self.show(action.target.clone());
                }
                if within.is_zero() {
                    self.finish(action, Outcome::Done);
                } else {
                    self.in_flight.push(InFlight {
                        deadline: now + within,
                        action,
                        generation,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum L {
        Wide,
        Narrow,
    }

    const fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    fn swap() -> Sequence<L> {
        Sequence::new()
            .towards(L::Narrow)
            .close(L::Wide, ms(100))
            .open(L::Narrow, ms(100))
    }

    fn started() -> (Transitions<L>, Instant, Vec<Command<L>>) {
        let t0 = Instant::now();
        let mut t = Transitions::new();
        t.show(L::Wide);
        let cmds = t.start(swap(), t0);
        (t, t0, cmds)
    }

    #[test]
    fn the_budget_is_known_up_front() {
        assert_eq!(swap().budget(), ms(200));
        let grouped = Sequence::new()
            .together([
                Action::close(L::Wide, ms(30)),
                Action::open(L::Narrow, ms(80)),
            ])
            .open(L::Narrow, ms(20));
        assert_eq!(grouped.budget(), ms(100));
    }

    #[test]
    fn close_then_open_in_order() {
        let (mut t, t0, cmds) = started();
        assert_eq!(t.phase(), Phase::Closing);
        assert!(matches!(
            cmds.as_slice(),
            [Command::Close {
                target: L::Wide,
                ..
            }]
        ));
        assert!(t.is_shown(&L::Wide) && !t.is_shown(&L::Narrow));

        // The close is reported in time and the open follows at once.
        let next = t.report(cmds[0].done(), t0 + ms(10));
        assert!(matches!(
            next.as_slice(),
            [Command::Open {
                target: L::Narrow,
                ..
            }]
        ));
        assert_eq!(t.phase(), Phase::Opening);
        assert!(!t.is_shown(&L::Wide) && t.is_shown(&L::Narrow));

        let at = t0 + ms(10) + ms(95);
        assert!(t.report(next[0].done(), at).is_empty());
        assert!(t.is_idle());
        let outcomes: Vec<_> = t.take_records().iter().map(|r| r.outcome).collect();
        assert_eq!(outcomes, [Outcome::Done, Outcome::Done]);
        assert!(t.take_records().is_empty(), "the log is drained");
    }

    #[test]
    fn a_stuck_target_is_closed_by_the_coordinator() {
        let (mut t, t0, cmds) = started();
        assert_eq!(t.deadline(), Some(t0 + ms(100)));
        assert!(t.tick(t0 + ms(99)).is_empty(), "not yet");
        assert!(t.is_shown(&L::Wide));

        let next = t.tick(t0 + ms(100));
        assert!(!t.is_shown(&L::Wide), "hard-closed: no longer rendered");
        assert!(matches!(
            next.as_slice(),
            [Command::Open {
                target: L::Narrow,
                ..
            }]
        ));
        assert_eq!(t.take_records()[0].outcome, Outcome::TimedOut);

        // The report that finally arrives belongs to a command that is over: ignored.
        assert!(t.report(cmds[0].done(), t0 + ms(101)).is_empty());
        assert_eq!(t.phase(), Phase::Opening);
        assert!(t.take_records().is_empty());
    }

    #[test]
    fn a_report_after_the_deadline_is_a_timeout_not_a_success() {
        let (mut t, t0, cmds) = started();
        let next = t.report(cmds[0].done(), t0 + ms(150));
        assert_eq!(t.take_records()[0].outcome, Outcome::TimedOut);
        assert_eq!(next.len(), 1, "the sequence still continues");
    }

    #[test]
    fn a_late_open_is_hard_opened() {
        let (mut t, t0, cmds) = started();
        let open = t.report(cmds[0].done(), t0);
        t.tick(t0 + ms(100));
        assert!(t.is_idle());
        assert!(t.is_shown(&L::Narrow));
        assert_eq!(t.take_records()[1].outcome, Outcome::TimedOut);
        assert!(t.report(open[0].done(), t0 + ms(120)).is_empty());
    }

    #[test]
    fn stale_duplicate_and_unknown_reports_are_ignored() {
        let (mut t, t0, cmds) = started();
        let wrong_generation = Event::Closed {
            target: L::Wide,
            generation: cmds[0].generation() + 40,
        };
        assert!(t.report(wrong_generation, t0 + ms(1)).is_empty());
        let wrong_target = Event::Closed {
            target: L::Narrow,
            generation: cmds[0].generation(),
        };
        assert!(t.report(wrong_target, t0 + ms(1)).is_empty());
        assert_eq!(t.phase(), Phase::Closing, "nothing moved");

        let open = t.report(cmds[0].done(), t0 + ms(2));
        assert_eq!(open.len(), 1);
        // The same report again is a duplicate: one report per command.
        assert!(t.report(cmds[0].done(), t0 + ms(3)).is_empty());
        assert_eq!(t.phase(), Phase::Opening);
    }

    #[test]
    fn zero_budget_runs_the_whole_sequence_in_one_call() {
        let t0 = Instant::now();
        let mut t = Transitions::new();
        t.show(L::Wide);
        let cmds = t.start(
            Sequence::new()
                .close(L::Wide, Duration::ZERO)
                .open(L::Narrow, Duration::ZERO),
            t0,
        );
        assert_eq!(cmds.len(), 2, "both commands go out, already settled");
        assert!(t.is_idle());
        assert!(t.is_shown(&L::Narrow) && !t.is_shown(&L::Wide));
        assert_eq!(t.deadline(), None);
        assert!(t.take_records().iter().all(|r| r.outcome == Outcome::Done));
    }

    #[test]
    fn a_new_sequence_replaces_waiting_steps_but_lets_the_step_in_flight_finish() {
        let (mut t, t0, cmds) = started();
        // While Wide is closing, the window flips back: Narrow must not open, Wide must return.
        let none = t.start(
            Sequence::new()
                .close(L::Narrow, ms(100))
                .open(L::Wide, ms(100)),
            t0 + ms(20),
        );
        assert!(none.is_empty(), "the in-flight Close(Wide) finishes first");
        assert_eq!(t.phase(), Phase::Closing);
        assert_eq!(
            t.deadline(),
            Some(t0 + ms(100)),
            "its own budget, not extended"
        );

        // Narrow was never shown, so the replacement sequence closes it (a no-op for the
        // target) before reopening Wide.
        let next = t.report(cmds[0].done(), t0 + ms(30));
        assert!(matches!(
            next.as_slice(),
            [Command::Close {
                target: L::Narrow,
                ..
            }]
        ));
        let next = t.report(next[0].done(), t0 + ms(40));
        assert!(matches!(
            next.as_slice(),
            [Command::Open {
                target: L::Wide,
                ..
            }]
        ));
        assert!(!t.is_shown(&L::Narrow) && t.is_shown(&L::Wide));
    }

    #[test]
    fn a_group_waits_for_all_of_its_actions() {
        let t0 = Instant::now();
        let mut t = Transitions::new();
        t.show(L::Wide);
        let cmds = t.start(
            Sequence::new()
                .together([
                    Action::close(L::Wide, ms(50)),
                    Action::open(L::Narrow, ms(80)),
                ])
                .open(L::Wide, ms(10)),
            t0,
        );
        assert_eq!(cmds.len(), 2);
        assert!(
            t.report(cmds[0].done(), t0 + ms(10)).is_empty(),
            "one is not enough"
        );
        assert_eq!(t.phase(), Phase::Opening);
        // The other never answers; its own deadline ends the step.
        assert_eq!(t.deadline(), Some(t0 + ms(80)));
        let next = t.tick(t0 + ms(80));
        assert!(matches!(
            next.as_slice(),
            [Command::Open {
                target: L::Wide,
                ..
            }]
        ));
    }

    #[test]
    fn commands_name_the_layout_the_change_is_heading_to() {
        let (mut t, t0, cmds) = started();
        assert!(matches!(
            cmds.as_slice(),
            [Command::Close {
                closing_for: Some(L::Narrow),
                ..
            }]
        ));
        let open = t.report(cmds[0].done(), t0);
        assert!(matches!(
            open.as_slice(),
            [Command::Open {
                opening_for: Some(L::Narrow),
                ..
            }]
        ));
        assert_eq!(open[0].towards(), Some(&L::Narrow));
    }

    #[test]
    fn generations_never_repeat() {
        let (mut t, t0, cmds) = started();
        let open = t.report(cmds[0].done(), t0);
        assert_ne!(cmds[0].generation(), open[0].generation());
    }
}
