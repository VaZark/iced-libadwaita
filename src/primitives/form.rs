//! Form: when a form may move on or be submitted, and when its fields show their errors.
//! Pure and controlled, like an HTML `<form>` with a native multi-step wizard: a single-page
//! form is the one-step case.
//!
//! The application owns the field values and one [`FormState`]; this module only transforms
//! the state. Every operation takes the [`FormSpec`] (which keys belong to which step) and a
//! lookup from a key to its current [`Validity`], and returns an [`Outcome`] telling the app
//! what happened (move on, submit, or what blocked it, with the key to focus).
//!
//! * a field shows its validity once it was **touched**, or its **step was attempted** (Next
//!   was pressed on it), or the form was **submitted** ([`FormState::revealed`]); this is the
//!   `:user-invalid` rule of [`validation`](super::validation): only invalid (and warnings)
//!   draw anything, a valid field adds nothing unless the widget opts in;
//! * only errors block ([`Validity::blocks_submit`]); a warning never does;
//! * going **back** is always allowed; going **forward** (Next, or jumping to a later step)
//!   needs every step before the target to be valid.
//!
//! ```ignore
//! // update:
//! Msg::Next => { if let Blocked { key, .. } = self.form.next(&spec, &|k| self.validity(k)) { /* focus key */ } }
//! Msg::Submit => if self.form.submit(&spec, &|k| self.validity(k)) == Outcome::Submitted { /* send */ }
//! Msg::Reset => { self.form.reset(); self.values = Values::default(); }
//! Msg::Name(v) => { self.name = v; self.form.touch(Key::Name); }
//! ```

use super::validation::{Interaction, Validity};

/// The ordered steps of a form, each a list of field keys. One step is a single-page form.
#[derive(Debug, Clone, PartialEq)]
pub struct FormSpec<K> {
    steps: Vec<Vec<K>>,
}

impl<K> Default for FormSpec<K> {
    fn default() -> Self {
        Self { steps: Vec::new() }
    }
}

impl<K: Clone + PartialEq> FormSpec<K> {
    /// A single-page form: one step holding `keys`.
    pub fn single(keys: impl IntoIterator<Item = K>) -> Self {
        Self::default().step(keys)
    }
    /// A wizard from its steps.
    pub fn steps(steps: impl IntoIterator<Item = Vec<K>>) -> Self {
        Self {
            steps: steps.into_iter().collect(),
        }
    }
    /// Appends a step.
    pub fn step(mut self, keys: impl IntoIterator<Item = K>) -> Self {
        self.steps.push(keys.into_iter().collect());
        self
    }
    pub fn len(&self) -> usize {
        self.steps.len()
    }
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }
    /// The keys of `step` (empty when out of range).
    pub fn keys(&self, step: usize) -> &[K] {
        self.steps.get(step).map_or(&[], Vec::as_slice)
    }
    /// The step `key` belongs to.
    pub fn step_of(&self, key: &K) -> Option<usize> {
        self.steps.iter().position(|s| s.contains(key))
    }
}

/// Looks a key's current validity up. The app computes it from its values.
pub type ValidityOf<'a, K> = &'a dyn Fn(&K) -> Validity;

/// What an operation did.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome<K> {
    /// The current step is now `to` (Next, Back or a jump).
    Moved { to: usize },
    /// Every step is valid: the app may send the form. (Only from [`FormState::submit`].)
    Submitted,
    /// Not allowed yet. The form is now on `step` (the first step with an error), its fields
    /// are revealed, and `key` is the first invalid field, for the app to focus.
    Blocked { step: usize, key: K },
    /// Nothing to do (Back on the first step, Next on the last, a step out of range).
    Ignored,
}

/// How a step shows in a step indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    /// Before the current step and valid.
    Completed,
    /// The step being filled.
    Current,
    /// Not reached yet.
    Upcoming,
    /// Has an error the user has been shown (a Next or Submit was refused on it, or a field
    /// of it was touched): drawn as needing attention.
    Invalid,
}

/// The form's interaction state. Controlled: the app stores it and replaces it with the
/// result of these operations.
#[derive(Debug, Clone, PartialEq)]
pub struct FormState<K> {
    touched: Vec<K>,
    submitted: bool,
    step: usize,
    attempted: Vec<usize>,
}

impl<K> Default for FormState<K> {
    fn default() -> Self {
        Self {
            touched: Vec::new(),
            submitted: false,
            step: 0,
            attempted: Vec::new(),
        }
    }
}

impl<K: Clone + PartialEq> FormState<K> {
    pub fn new() -> Self {
        Self::default()
    }
    /// Starts on `step` (a restored draft, a screenshot).
    pub fn at_step(mut self, step: usize) -> Self {
        self.step = step;
        self
    }
    /// Starts with the current step attempted, so its errors already show.
    pub fn attempted(mut self) -> Self {
        self.attempted.push(self.step);
        self
    }

    // ---- reading ----

    /// The current step index.
    pub fn step(&self) -> usize {
        self.step
    }
    pub fn is_first(&self) -> bool {
        self.step == 0
    }
    pub fn is_last(&self, spec: &FormSpec<K>) -> bool {
        self.step + 1 >= spec.len()
    }
    /// A submit was attempted (or succeeded).
    pub fn is_submitted(&self) -> bool {
        self.submitted
    }
    pub fn is_touched(&self, key: &K) -> bool {
        self.touched.contains(key)
    }
    /// Whether Next / Submit was refused on `step`.
    pub fn is_attempted(&self, step: usize) -> bool {
        self.attempted.contains(&step)
    }
    /// The user has edited anything since the last reset.
    pub fn dirty(&self) -> bool {
        !self.touched.is_empty()
    }
    /// Steps reached including the current one, as a fraction of all steps (`1.0` on the last).
    pub fn progress(&self, spec: &FormSpec<K>) -> f32 {
        if spec.is_empty() {
            return 0.0;
        }
        ((self.step + 1).min(spec.len())) as f32 / spec.len() as f32
    }

    /// Whether `key` may show its validity: touched, or its step attempted, or submitted.
    pub fn revealed(&self, spec: &FormSpec<K>, key: &K) -> bool {
        self.interaction(spec, key).revealed()
    }

    /// The [`Interaction`] of `key`, ready for `Field::interaction` or `validation::present`:
    /// `touched` is the field's own flag; `submitted` is set by a submit or by an attempt on
    /// the field's step (what the "show it all" rule means for the field).
    pub fn interaction(&self, spec: &FormSpec<K>, key: &K) -> Interaction {
        Interaction {
            touched: self.is_touched(key),
            submitted: self.submitted || spec.step_of(key).is_some_and(|s| self.is_attempted(s)),
        }
    }

    /// Whether no field of `step` has an error.
    pub fn step_valid(&self, spec: &FormSpec<K>, step: usize, validity_of: ValidityOf<K>) -> bool {
        first_invalid(spec, step, validity_of).is_none()
    }

    /// The indicator state of `step`. A step is [`Invalid`](StepStatus::Invalid) once the user
    /// was shown its errors (attempted, submitted, or a field of it was touched) and one remains.
    pub fn status(
        &self,
        spec: &FormSpec<K>,
        step: usize,
        validity_of: ValidityOf<K>,
    ) -> StepStatus {
        let valid = self.step_valid(spec, step, validity_of);
        let shown = self.submitted
            || self.is_attempted(step)
            || spec.keys(step).iter().any(|k| self.is_touched(k));
        if !valid && shown && (step != self.step || self.is_attempted(step) || self.submitted) {
            StepStatus::Invalid
        } else if step == self.step {
            StepStatus::Current
        } else if step < self.step && valid {
            StepStatus::Completed
        } else {
            StepStatus::Upcoming
        }
    }

    /// Whether a jump to `step` is allowed: always backwards, forwards only when every step
    /// before it is valid.
    pub fn can_goto(&self, spec: &FormSpec<K>, step: usize, validity_of: ValidityOf<K>) -> bool {
        step < spec.len()
            && (step <= self.step || (0..step).all(|s| self.step_valid(spec, s, validity_of)))
    }

    // ---- changing ----

    /// The user edited or left `key`.
    pub fn touch(&mut self, key: K) {
        if !self.touched.contains(&key) {
            self.touched.push(key);
        }
    }

    /// Next: advances when the current step is valid; otherwise marks it attempted (its
    /// invalid fields reveal) and returns [`Outcome::Blocked`]. [`Outcome::Ignored`] on the
    /// last step (use [`submit`](Self::submit)).
    pub fn next(&mut self, spec: &FormSpec<K>, validity_of: ValidityOf<K>) -> Outcome<K> {
        if self.is_last(spec) {
            return Outcome::Ignored;
        }
        match first_invalid(spec, self.step, validity_of) {
            None => {
                self.step += 1;
                Outcome::Moved { to: self.step }
            }
            Some(key) => self.block(self.step, key),
        }
    }

    /// Back: always allowed, never validates.
    pub fn back(&mut self) -> Outcome<K> {
        if self.step == 0 {
            return Outcome::Ignored;
        }
        self.step -= 1;
        Outcome::Moved { to: self.step }
    }

    /// Jumps to `step` (a click on the step indicator). Backwards is always allowed. Forwards
    /// needs every step before `step` valid; otherwise the form goes to (or stays on) the
    /// first invalid step, attempted, and the outcome is [`Outcome::Blocked`].
    pub fn goto(
        &mut self,
        spec: &FormSpec<K>,
        step: usize,
        validity_of: ValidityOf<K>,
    ) -> Outcome<K> {
        if step >= spec.len() || step == self.step {
            return Outcome::Ignored;
        }
        if step < self.step {
            self.step = step;
            return Outcome::Moved { to: step };
        }
        // Forwards: the first invalid step from here up to (not including) the target blocks.
        for s in self.step..step {
            if let Some(key) = first_invalid(spec, s, validity_of) {
                return self.block(s, key);
            }
        }
        self.step = step;
        Outcome::Moved { to: step }
    }

    /// Submit: when every step is valid, marks the form submitted and returns
    /// [`Outcome::Submitted`]. Otherwise jumps to the first invalid step, marks the form
    /// submitted (every error now shows) and returns [`Outcome::Blocked`].
    pub fn submit(&mut self, spec: &FormSpec<K>, validity_of: ValidityOf<K>) -> Outcome<K> {
        self.submitted = true;
        for s in 0..spec.len() {
            if let Some(key) = first_invalid(spec, s, validity_of) {
                return self.block(s, key);
            }
        }
        Outcome::Submitted
    }

    /// Reset: forgets touched, submitted and attempted, and returns to the first step. The
    /// app resets its own values.
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    fn block(&mut self, step: usize, key: K) -> Outcome<K> {
        self.step = step;
        if !self.attempted.contains(&step) {
            self.attempted.push(step);
        }
        Outcome::Blocked { step, key }
    }
}

/// The first key of `step` whose validity blocks.
pub fn first_invalid<K: Clone + PartialEq>(
    spec: &FormSpec<K>,
    step: usize,
    validity_of: ValidityOf<K>,
) -> Option<K> {
    spec.keys(step)
        .iter()
        .find(|k| validity_of(k).blocks_submit())
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::super::validation::{self, Presentation};
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;

    type S = FormState<&'static str>;

    /// Values by key; empty is an error ("a") or a warning ("w").
    struct Vals(RefCell<HashMap<&'static str, &'static str>>);
    impl Vals {
        fn new(v: &[(&'static str, &'static str)]) -> Self {
            Self(RefCell::new(v.iter().copied().collect()))
        }
        fn set(&self, k: &'static str, v: &'static str) {
            self.0.borrow_mut().insert(k, v);
        }
        fn validity(&self, k: &&'static str) -> Validity {
            match self.0.borrow().get(k).copied().unwrap_or("") {
                "" => Validity::error("required"),
                "warn" => Validity::warning("hmm"),
                _ => Validity::Ok,
            }
        }
    }

    fn wizard() -> FormSpec<&'static str> {
        FormSpec::steps([vec!["a", "b"], vec!["c"], vec!["d"]])
    }

    #[test]
    fn single_page_submit_is_gated_by_validity() {
        let spec = FormSpec::single(["name", "pw"]);
        let v = Vals::new(&[("name", "ada")]);
        let mut s = S::new();
        assert!(s.is_first() && s.is_last(&spec));
        assert_eq!(
            s.submit(&spec, &|k| v.validity(k)),
            Outcome::Blocked { step: 0, key: "pw" }
        );
        assert!(s.is_submitted() && s.is_attempted(0));
        v.set("pw", "secret");
        assert_eq!(s.submit(&spec, &|k| v.validity(k)), Outcome::Submitted);
        // Next on a one-step form does nothing.
        assert_eq!(s.next(&spec, &|k| v.validity(k)), Outcome::Ignored);
    }

    #[test]
    fn warnings_do_not_block() {
        let spec = FormSpec::single(["a"]);
        let v = Vals::new(&[("a", "warn")]);
        assert_eq!(
            S::new().submit(&spec, &|k| v.validity(k)),
            Outcome::Submitted
        );
    }

    #[test]
    fn next_is_blocked_until_the_step_is_fixed() {
        let spec = wizard();
        let v = Vals::new(&[("a", "x")]);
        let mut s = S::new();
        assert!(!s.revealed(&spec, &"b"));
        assert_eq!(
            s.next(&spec, &|k| v.validity(k)),
            Outcome::Blocked { step: 0, key: "b" }
        );
        assert_eq!(s.step(), 0);
        // The attempted step reveals its invalid fields (and nothing of the other steps).
        assert!(s.revealed(&spec, &"b") && s.revealed(&spec, &"a"));
        assert!(!s.revealed(&spec, &"c"));
        v.set("b", "y");
        assert_eq!(s.next(&spec, &|k| v.validity(k)), Outcome::Moved { to: 1 });
        assert_eq!(s.progress(&spec), 2.0 / 3.0);
    }

    #[test]
    fn back_is_always_allowed() {
        let spec = wizard();
        let v = Vals::new(&[("a", "x"), ("b", "x")]);
        let mut s = S::new();
        assert_eq!(s.back(), Outcome::Ignored);
        s.next(&spec, &|k| v.validity(k));
        // Step 1 is empty (invalid), yet Back works and does not mark anything.
        assert_eq!(s.back(), Outcome::Moved { to: 0 });
        assert!(!s.is_attempted(1));
    }

    #[test]
    fn goto_back_free_forward_needs_valid_earlier_steps() {
        let spec = wizard();
        let v = Vals::new(&[("a", "x"), ("b", "x")]);
        let mut s = S::new();
        // 0 -> 2 skips the invalid step 1: lands on step 1, attempted.
        assert_eq!(
            s.goto(&spec, 2, &|k| v.validity(k)),
            Outcome::Blocked { step: 1, key: "c" }
        );
        assert_eq!((s.step(), s.is_attempted(1)), (1, true));
        assert!(!s.can_goto(&spec, 2, &|k| v.validity(k)));
        v.set("c", "x");
        assert!(s.can_goto(&spec, 2, &|k| v.validity(k)));
        assert_eq!(
            s.goto(&spec, 2, &|k| v.validity(k)),
            Outcome::Moved { to: 2 }
        );
        // Back to 0 even if it had become invalid.
        v.set("a", "");
        assert_eq!(
            s.goto(&spec, 0, &|k| v.validity(k)),
            Outcome::Moved { to: 0 }
        );
        assert_eq!(s.goto(&spec, 0, &|k| v.validity(k)), Outcome::Ignored);
        assert_eq!(s.goto(&spec, 9, &|k| v.validity(k)), Outcome::Ignored);
        // Forward from the invalid current step stays and attempts it.
        assert_eq!(
            s.goto(&spec, 1, &|k| v.validity(k)),
            Outcome::Blocked { step: 0, key: "a" }
        );
    }

    #[test]
    fn submit_jumps_to_the_first_invalid_step() {
        let spec = wizard();
        let v = Vals::new(&[("a", "x"), ("b", "x"), ("d", "x")]);
        let mut s = S::new().at_step(2);
        assert_eq!(
            s.submit(&spec, &|k| v.validity(k)),
            Outcome::Blocked { step: 1, key: "c" }
        );
        assert_eq!(s.step(), 1);
        assert!(s.is_submitted() && s.revealed(&spec, &"d"));
        v.set("c", "x");
        assert_eq!(s.submit(&spec, &|k| v.validity(k)), Outcome::Submitted);
    }

    #[test]
    fn reset_clears_everything() {
        let spec = wizard();
        let v = Vals::new(&[]);
        let mut s = S::new();
        s.touch("a");
        s.next(&spec, &|k| v.validity(k));
        s.submit(&spec, &|k| v.validity(k));
        assert!(s.dirty());
        s.reset();
        assert_eq!(s, S::new());
        assert!(!s.dirty() && !s.is_submitted() && !s.revealed(&spec, &"a"));
    }

    #[test]
    fn errors_reveal_only_when_touched_or_attempted() {
        // Same rule as validation::present: pristine until revealed, then only invalid and
        // warnings draw something.
        let spec = wizard();
        let v = Vals::new(&[]);
        let mut s = S::new();
        let show =
            |s: &S, k: &'static str| validation::present(&v.validity(&k), s.revealed(&spec, &k));
        assert_eq!(show(&s, "a"), Presentation::Pristine);
        s.touch("a");
        assert_eq!(show(&s, "a"), Presentation::Invalid);
        assert_eq!(show(&s, "b"), Presentation::Pristine);
        s.next(&spec, &|k| v.validity(k));
        assert_eq!(show(&s, "b"), Presentation::Invalid);
        assert_eq!(show(&s, "c"), Presentation::Pristine);
        let i = s.interaction(&spec, &"b");
        assert!(!i.touched && i.submitted);
    }

    #[test]
    fn statuses_for_the_indicator() {
        let spec = wizard();
        let v = Vals::new(&[("a", "x"), ("b", "x")]);
        let mut s = S::new();
        let all = |s: &S| -> Vec<StepStatus> {
            (0..3)
                .map(|i| s.status(&spec, i, &|k| v.validity(k)))
                .collect()
        };
        use StepStatus::*;
        assert_eq!(all(&s), [Current, Upcoming, Upcoming]);
        s.next(&spec, &|k| v.validity(k));
        assert_eq!(all(&s), [Completed, Current, Upcoming]);
        // Refused on step 1: it is now flagged.
        s.next(&spec, &|k| v.validity(k));
        assert_eq!(all(&s), [Completed, Invalid, Upcoming]);
        v.set("c", "x");
        assert_eq!(all(&s), [Completed, Current, Upcoming]);
    }
}
