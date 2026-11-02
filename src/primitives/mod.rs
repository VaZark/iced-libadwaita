//! Headless primitives: behavior with no widgets and no look, after Radix UI.
//!
//! Each is pure (plain data in, plain data out), controlled (the application owns the state)
//! and unit-tested. The widgets in [`widget`](crate::widget) apply them to the screen, so a
//! more complex component can combine the same primitives without copying their logic. See
//! `docs/lib/primitives.md`.
//!
//! * [`anchor`]: where a floating surface sits against an anchor (flip, shift);
//! * [`date`]: calendar dates and month math;
//! * [`form`]: submit / reset / next / back gating and when a form's fields reveal errors (steps);
//! * [`range`]: a value with two ends (`lo <= hi`): the two-thumb slider's thumbs, a date range;
//! * [`sections`]: text padding around an input's left / right sections;
//! * [`selection`]: which keys are on (single, required single, multiple);
//! * [`validation`]: when a field shows valid / invalid, and chaining validators;
//! * [`meta`]: which variant of a meta component shows, and its [`Trigger`] (breakpoint or select);
//! * [`responsive`]: which presentation a width gets ([`When`](responsive::When)).

pub mod anchor;
pub mod date;
pub mod form;
pub mod meta;
pub mod range;
pub mod responsive;
pub mod sections;
pub mod selection;
pub mod validation;

pub use form::{FormSpec, FormState, Outcome as FormOutcome, StepStatus};
pub use meta::Trigger;
pub use responsive::When;
pub use selection::Selection;
