//! Headless primitives: behavior with no widgets and no look, after Radix UI.
//!
//! Each is pure (plain data in, plain data out), controlled (the application owns the state)
//! and unit-tested. The widgets in [`widget`](crate::widget) apply them to the screen, so a
//! more complex component can combine the same primitives without copying their logic. See
//! `docs/lib/primitives.md`.
//!
//! * [`anchor`]: where a floating surface sits against an anchor (flip, shift);
//! * [`calendar`]: calendar navigation by level (month, year, decade);
//! * [`date`]: calendar dates and month math;
//! * [`form`]: submit / reset / next / back gating and when a form's fields reveal errors (steps);
//! * [`lookup`]: a query answered asynchronously: debounce, stale-response discarding, cache;
//! * [`notifications`]: the queue behind stacked toasts: ids, a limit, in-place update, auto-dismiss from an explicit `now`;
//! * [`range`]: a value with two ends (`lo <= hi`): the two-thumb slider's thumbs, a date range;
//! * [`sections`]: text padding around an input's left / right sections;
//! * [`relative`]: relative time wording ("yesterday", "12 days ago", "in 3 hours") from an explicit `now`;
//! * [`selection`]: which keys are on (single, required single, multiple);
//! * [`stepper`]: step statuses from an `active` index, click rules, next / back, connector fill;
//! * [`validation`]: when a field shows valid / invalid, and chaining validators;
//! * [`meta`]: which variant of a meta component shows, and its [`Trigger`] (breakpoint or select);
//! * [`responsive`]: which presentation a width gets ([`When`](responsive::When)).

pub mod anchor;
pub mod calendar;
pub mod date;
pub mod form;
pub mod lookup;
pub mod meta;
pub mod notifications;
pub mod range;
pub mod relative;
pub mod responsive;
pub mod sections;
pub mod selection;
pub mod stepper;
pub mod validation;

pub use form::{FormSpec, FormState, Outcome as FormOutcome, StepStatus};
pub use lookup::{Highlight, Lookup, Query, Request, RequestId, Status as LookupStatus};
pub use meta::Trigger;
pub use relative::Relative;
pub use responsive::When;
pub use selection::Selection;
pub use stepper::{Fill as StepperFill, Status as StepperStatus, StepperState};
