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
//! * [`infinite`]: a list that grows page by page (offset or cursor): one request in flight, stale responses discarded, the scroll rule;
//! * [`lookup`]: a query answered asynchronously: debounce, stale-response discarding, cache;
//! * [`notifications`]: the queue behind stacked toasts: ids, a limit, in-place update, auto-dismiss from an explicit `now`;
//! * [`pagination`]: page, page size, "Showing 21-40 of 187", Mantine's page-number window with dots;
//! * [`range`]: a value with two ends (`lo <= hi`): the two-thumb slider's thumbs, a date range;
//! * [`sections`]: text padding around an input's left / right sections;
//! * [`reorder`]: a list with a preview and an edit view, the draft order, and what dragging a row means;
//! * [`relative`]: relative time wording ("yesterday", "12 days ago", "in 3 hours") from an explicit `now`;
//! * [`animation`]: the animation engine (easing, timed and spring animations, `Tween`, `Progress`);
//! * [`transition`]: ordered close / open transitions between layouts (`Sequence`, `Transitions`);
//! * [`selection`]: which keys are on (single, required single, multiple);
//! * [`sorting`]: sort keys with the asc / desc / unsorted cycle, multi-column, a stable sort and comparators;
//! * [`tabs`]: which tab a click or an arrow key activates, skipping disabled ones (Radix Tabs); shared by `Tabs` and the segmented control;
//! * [`virtualize`]: which rows of a long fixed-height list to build for a scroll offset, and the spacers around them (shared by `Tree` and `ResponsiveTable`);
//! * [`loading`]: keys being loaded (with their errors and retry), and a list fetched in chunks with placeholders for what has not arrived (shared by `Tree` and the tables);
//! * [`tree`]: nodes, open branches, selection, checking, search / filter and keyboard navigation (Mantine's `useTree`, GTK's tree expander keys);
//! * [`stepper`]: step statuses from an `active` index, click rules, next / back, connector fill;
//! * [`validation`]: when a field shows valid / invalid, and chaining validators;
//! * [`meta`]: which variant of a meta component shows, and its [`Trigger`] (breakpoint or select);
//! * [`meta_parent`]: the children of a meta parent: what each announced when it registered, and who a notification reaches;
//! * [`resize`]: dragging a handle into a size (grips, min / max, grid and breakpoint snapping);
//! * [`responsive`]: which presentation a width gets ([`When`](responsive::When)).

pub mod anchor;
pub mod animation;
pub mod calendar;
pub mod date;
pub mod form;
pub mod infinite;
pub mod loading;
pub mod lookup;
pub mod meta;
pub mod meta_parent;
pub mod notifications;
pub mod pagination;
pub mod range;
pub mod relative;
pub mod reorder;
pub mod resize;
pub mod responsive;
pub mod sections;
pub mod selection;
pub mod sorting;
pub mod stepper;
pub mod tabs;
pub mod transition;
pub mod tree;
pub mod validation;
pub mod virtualize;

pub use form::{FormSpec, FormState, Outcome as FormOutcome, StepStatus};
pub use infinite::{Infinite, Page as InfinitePage, ScrollMetrics};
pub use lookup::{Highlight, Lookup, Query, Request, RequestId, Status as LookupStatus};
pub use meta::Trigger;
pub use pagination::{PageItem, Pagination};
pub use relative::Relative;
pub use reorder::{Event as ReorderEvent, Mode as ReorderMode, ReorderState};
pub use responsive::When;
pub use selection::Selection;
pub use sorting::{Cycle as SortCycle, Direction as SortDirection, SortState};
pub use stepper::{Fill as StepperFill, Status as StepperStatus, StepperState};
