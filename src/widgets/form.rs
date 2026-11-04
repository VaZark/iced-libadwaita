//! `Form`: the form "meta tag" (HTML `<form>`). It renders the current step's content with
//! the standard chrome (title, step indicator, action bar) and emits the app's messages; the
//! pure rules (what may be submitted, when errors show) are
//! [`primitives::form`](crate::primitives::form).
//!
//! One step is a single-page form: only Reset and Submit. Several steps are a wizard: a
//! [`StepIndicator`] (completed / current / upcoming / invalid), Back, Reset, and Next, which
//! becomes Submit on the last step.
//!
//! The loop, all controlled (the widget stores nothing):
//!
//! ```ignore
//! // state:  values + `form: FormState<Key>`; spec: `FormSpec::steps([vec![Key::Name], ..])`
//! // update:
//! Msg::Edit(v)  => { self.name = v; self.form.touch(Key::Name); }
//! Msg::Next     => { self.form.next(&spec, &|k| self.validity(k)); }
//! Msg::Back     => { self.form.back(); }
//! Msg::Step(i)  => { self.form.goto(&spec, i, &|k| self.validity(k)); }
//! Msg::Submit   => if self.form.submit(&spec, &|k| self.validity(k)) == FormOutcome::Submitted { /* send */ }
//! Msg::Reset    => { self.form.reset(); self.values = Default::default(); }
//! // view:
//! Form::new()
//!     .title("Sign up")
//!     .state(&self.form, &spec, |k| self.validity(k))
//!     .step(Step::new("Account", || account_fields(&self.form)))   // built only while current
//!     .on_next(Msg::Next).on_back(Msg::Back).on_step(Msg::Step)
//!     .on_submit(Msg::Submit).on_reset(Msg::Reset)
//! ```
//!
//! A `Field` takes its reveal state from the form with
//! `Field::interaction(form.interaction(&spec, &key))`.
//!
//! Only the current step's content is built: the same choice as a
//! [`MetaComponent`](crate::metas::MetaComponent) with `Trigger::selected(step)` (it uses the same
//! `primitives::meta::pick`), but called directly, because `MetaComponent` is built on iced's
//! `responsive`, which cannot shrink to its content's height and so cannot sit in a scrolled
//! column. **Enter** triggers the primary action (Next, or Submit on the last step) through
//! the shortcut scope; a text input that handles Enter itself takes precedence. Turn it off
//! with [`enter_submits`](Form::enter_submits).

use crate::Element;
use crate::components::{raised_button, suggested_button};
use crate::primitives::form::{FormSpec, FormState, StepStatus};
use crate::primitives::meta::{self, Trigger};
use crate::primitives::validation::Validity;
use crate::shortcut::Shortcuts;
use crate::styles::text as st;
use crate::widgets::{Stepper, StepperStep};
use crate::{Metrics, Theme};
use iced::Length;
use iced::widget::{column, container, row, space};

/// One step of a [`Form`]: a label and a closure that builds its content, called only while
/// the step is current.
pub struct Step<'a, M> {
    label: String,
    build: Box<dyn Fn() -> Element<'a, M> + 'a>,
}

impl<'a, M> Step<'a, M> {
    pub fn new(label: impl Into<String>, build: impl Fn() -> Element<'a, M> + 'a) -> Self {
        Self {
            label: label.into(),
            build: Box::new(build),
        }
    }
}

/// Where the action bar sits its buttons.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FormActions {
    /// Reset at the start, Back / Next / Submit at the end (the dialog convention).
    #[default]
    Spread,
    /// Every button at the end.
    End,
    /// Every button at the start.
    Start,
}

/// See the [module](self).
pub struct Form<'a, M> {
    title: Option<String>,
    description: Option<String>,
    steps: Vec<Step<'a, M>>,
    current: usize,
    statuses: Vec<StepStatus>,
    reachable: Vec<bool>,
    on_submit: Option<M>,
    on_reset: Option<M>,
    on_next: Option<M>,
    on_back: Option<M>,
    on_step: Option<Box<dyn Fn(usize) -> M + 'a>>,
    submit_label: String,
    reset_label: String,
    next_label: String,
    back_label: String,
    submitting: bool,
    enter: bool,
    actions: FormActions,
    metrics: Metrics,
}

impl<'a, M: Clone + 'a> Default for Form<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, M: Clone + 'a> Form<'a, M> {
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            steps: Vec::new(),
            current: 0,
            statuses: Vec::new(),
            reachable: Vec::new(),
            on_submit: None,
            on_reset: None,
            on_next: None,
            on_back: None,
            on_step: None,
            submit_label: "Submit".into(),
            reset_label: "Reset".into(),
            next_label: "Next".into(),
            back_label: "Back".into(),
            submitting: false,
            enter: true,
            actions: FormActions::Spread,
            metrics: Metrics::desktop(),
        }
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    /// Adds a step. Call once for a single-page form.
    pub fn step(mut self, s: Step<'a, M>) -> Self {
        self.steps.push(s);
        self
    }
    /// The controlled state: the current step, each step's indicator state and which steps a
    /// click may reach are computed now from `state`, `spec` and `validity_of`. Pass the same
    /// spec the app uses in `update`.
    pub fn state<K: Clone + PartialEq>(
        mut self,
        state: &FormState<K>,
        spec: &FormSpec<K>,
        validity_of: impl Fn(&K) -> Validity,
    ) -> Self {
        let v: &dyn Fn(&K) -> Validity = &validity_of;
        self.current = state.step();
        self.statuses = (0..spec.len()).map(|i| state.status(spec, i, v)).collect();
        self.reachable = (0..spec.len())
            .map(|i| state.can_goto(spec, i, v))
            .collect();
        self
    }
    pub fn on_submit(mut self, m: M) -> Self {
        self.on_submit = Some(m);
        self
    }
    pub fn on_reset(mut self, m: M) -> Self {
        self.on_reset = Some(m);
        self
    }
    pub fn on_next(mut self, m: M) -> Self {
        self.on_next = Some(m);
        self
    }
    pub fn on_back(mut self, m: M) -> Self {
        self.on_back = Some(m);
        self
    }
    /// A click on a step of the indicator (only steps the goto rules allow are clickable).
    pub fn on_step(mut self, f: impl Fn(usize) -> M + 'a) -> Self {
        self.on_step = Some(Box::new(f));
        self
    }
    pub fn submit_label(mut self, l: impl Into<String>) -> Self {
        self.submit_label = l.into();
        self
    }
    pub fn reset_label(mut self, l: impl Into<String>) -> Self {
        self.reset_label = l.into();
        self
    }
    pub fn next_label(mut self, l: impl Into<String>) -> Self {
        self.next_label = l.into();
        self
    }
    pub fn back_label(mut self, l: impl Into<String>) -> Self {
        self.back_label = l.into();
        self
    }
    /// A submit is in flight: every button is disabled and Submit reads "Submitting…".
    pub fn submitting(mut self, s: bool) -> Self {
        self.submitting = s;
        self
    }
    /// Enter triggers the primary action (on by default).
    pub fn enter_submits(mut self, on: bool) -> Self {
        self.enter = on;
        self
    }
    /// Where the action bar puts its buttons.
    pub fn layout(mut self, l: FormActions) -> Self {
        self.actions = l;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }

    /// Whether the current step is the last (also true for a single-page form).
    fn is_last(&self) -> bool {
        self.current + 1 >= self.steps.len()
    }
}

impl<'a, M: Clone + 'a> From<Form<'a, M>> for Element<'a, M> {
    fn from(f: Form<'a, M>) -> Self {
        let m = f.metrics;
        let many = f.steps.len() > 1;
        let last = f.is_last();
        let busy = f.submitting;
        let idle = |msg: &Option<M>| if busy { None } else { msg.clone() };

        let mut body = column![].spacing(m.space_l).width(Length::Fill);
        if f.title.is_some() || f.description.is_some() {
            let mut head = column![].spacing(m.space_xs);
            if let Some(t) = &f.title {
                head = head.push(crate::components::typography::title_3(t.clone()));
            }
            if let Some(d) = &f.description {
                head = head.push(crate::components::typography::body(d.clone()).style(st::dim));
            }
            body = body.push(head);
        }
        if many {
            let items = f
                .steps
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    (
                        s.label.clone(),
                        f.statuses.get(i).copied().unwrap_or(StepStatus::Upcoming),
                    )
                })
                .collect();
            let mut ind = StepIndicator::new(items).metrics(m);
            if let (Some(on), false) = (&f.on_step, busy) {
                let clickable = (0..f.steps.len())
                    .map(|i| i != f.current && f.reachable.get(i).copied().unwrap_or(false))
                    .collect();
                ind = ind.clickable(clickable);
                let msgs: Vec<M> = (0..f.steps.len()).map(&**on).collect();
                body = body.push(ind.on_step_messages(msgs));
            } else {
                body = body.push(ind);
            }
        }

        // The current step only: `Trigger::selected(i)` over the shared meta primitive.
        let triggers: Vec<Trigger<usize>> = (0..f.steps.len()).map(Trigger::selected).collect();
        let content: Element<'a, M> = meta::pick(
            triggers.iter(),
            &[f.current],
            0.0,
            &crate::breakpoints::Breakpoints::default(),
        )
        .map_or_else(|| space().into(), |i| (f.steps[i].build)());
        body = body.push(container(content).width(Length::Fill));

        // Action bar.
        let reset = raised_button(label(&f.reset_label)).on_press_maybe(idle(&f.on_reset));
        let back = (many && f.current > 0)
            .then(|| raised_button(label(&f.back_label)).on_press_maybe(idle(&f.on_back)));
        let (primary_label, primary_msg) = if last {
            (
                if busy {
                    "Submitting…".to_owned()
                } else {
                    f.submit_label.clone()
                },
                f.on_submit.clone(),
            )
        } else {
            (f.next_label.clone(), f.on_next.clone())
        };
        let primary = suggested_button(label(&primary_label)).on_press_maybe(idle(&primary_msg));
        let mut right = row![].spacing(m.space_s);
        if let Some(b) = back {
            right = right.push(b);
        }
        right = right.push(primary);
        let bar: Element<'a, M> = match f.actions {
            FormActions::Spread => row![reset, space().width(Length::Fill), right].into(),
            FormActions::End => row![space().width(Length::Fill), reset, right]
                .spacing(m.space_s)
                .into(),
            FormActions::Start => row![reset, right].spacing(m.space_s).into(),
        };
        body = body.push(bar);

        let el: Element<'a, M> = body.into();
        match (f.enter, idle(&primary_msg)) {
            (true, Some(msg)) => Shortcuts::new().bind("Enter", msg).scope(el),
            _ => el,
        }
    }
}

fn label<'a>(s: &str) -> iced::widget::Text<'a, Theme> {
    crate::components::typography::label(s.to_owned())
}

/// The numbered, labelled steps of a wizard, joined by lines: completed (check), current,
/// upcoming, and invalid (an error mark after a refused Next / Submit). Part of [`Form`], and
/// usable alone. A thin wrapper over [`Stepper`] (indicator only, given statuses and reachable
/// steps); use `Stepper` directly for content, orientation, icons and the rest.
pub struct StepIndicator<'a, M> {
    items: Vec<(String, StepStatus)>,
    clickable: Vec<bool>,
    messages: Vec<M>,
    metrics: Metrics,
    _p: std::marker::PhantomData<&'a ()>,
}

impl<'a, M> StepIndicator<'a, M> {
    pub fn new(items: Vec<(String, StepStatus)>) -> Self {
        Self {
            items,
            clickable: Vec::new(),
            messages: Vec::new(),
            metrics: Metrics::desktop(),
            _p: std::marker::PhantomData,
        }
    }
    /// Which steps respond to a click (those with a message from `on_step_messages`).
    pub fn clickable(mut self, c: Vec<bool>) -> Self {
        self.clickable = c;
        self
    }
    /// The message for each step, by index.
    pub fn on_step_messages(mut self, m: Vec<M>) -> Self {
        self.messages = m;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: Clone + 'a> From<StepIndicator<'a, M>> for Element<'a, M> {
    fn from(s: StepIndicator<'a, M>) -> Self {
        let statuses = s.items.iter().map(|(_, st)| *st).collect();
        let clickable = (0..s.items.len())
            .map(|i| s.clickable.get(i).copied().unwrap_or(false) && i < s.messages.len())
            .collect();
        let messages = s.messages;
        let mut st = Stepper::new()
            .statuses(statuses)
            .clickable(clickable)
            // As before the extraction: a long row clips instead of collapsing.
            .collapse(false)
            .metrics(s.metrics);
        // Out of range of `active` is irrelevant: statuses and clickable are explicit.
        st = st.on_step_click(move |i| messages[i].clone());
        for (label, _) in s.items {
            st = st.step(StepperStep::new(label));
        }
        st.into()
    }
}
