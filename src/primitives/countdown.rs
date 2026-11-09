//! A one-shot countdown, such as a toast's auto-dismiss: pure, with no widgets.
//!
//! It starts at the first [`tick`](Countdown::tick), fires exactly once when its time is up, and
//! a duration of zero never fires. Replace it with a new [`Countdown`] to restart.
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Countdown {
    deadline: Option<Instant>,
    fired: bool,
}

/// What a [`Countdown`] says at a moment in time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tick {
    /// Nothing to wait for: no duration, or it has already fired.
    Idle,
    /// Not yet; check again at this time.
    Wait(Instant),
    /// The time is up. Reported once, then [`Idle`](Tick::Idle).
    Fire,
}

impl Countdown {
    pub fn fired(&self) -> bool {
        self.fired
    }
    /// Advances to `now` for a countdown of `millis`.
    pub fn tick(&mut self, now: Instant, millis: u32) -> Tick {
        if self.fired || millis == 0 {
            return Tick::Idle;
        }
        let deadline = *self
            .deadline
            .get_or_insert_with(|| now + Duration::from_millis(millis as u64));
        if now >= deadline {
            self.fired = true;
            Tick::Fire
        } else {
            Tick::Wait(deadline)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_fires_exactly_once_at_its_deadline() {
        let start = Instant::now();
        let at = |ms| start + Duration::from_millis(ms);
        let mut c = Countdown::default();
        assert_eq!(c.tick(at(0), 75), Tick::Wait(at(75)));
        assert_eq!(c.tick(at(74), 75), Tick::Wait(at(75)));
        assert_eq!(c.tick(at(75), 75), Tick::Fire);
        assert!(c.fired());
        assert_eq!(c.tick(at(500), 75), Tick::Idle);
    }

    #[test]
    fn zero_never_fires() {
        let mut c = Countdown::default();
        assert_eq!(c.tick(Instant::now(), 0), Tick::Idle);
        assert!(!c.fired());
    }
}
