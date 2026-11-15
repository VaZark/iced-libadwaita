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
    props: FormWidgetProps,
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
    spacing: Option<f32>,
    header_spacing: Option<f32>,
    actions_spacing: Option<f32>,
    padding: iced::Padding,
    metrics: Metrics,
}

impl<'a, M: Clone + 'a> Default for Form<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, M: Clone + 'a> Form<'a, M> {
    /// Presentation overrides for native children; actions remain parent-owned.
    pub fn props(mut self, props: FormWidgetProps) -> Self {
        self.props = props;
        self
    }
    pub fn new() -> Self {
        Self {
            props: FormWidgetProps::default(),
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
            spacing: None,
            header_spacing: None,
            actions_spacing: None,
            padding: iced::Padding::ZERO,
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
    /// The gap between the heading, the steps, the content and the action bar (default: the
    /// theme's `space_l`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px.max(0.0));
        self
    }
    /// The gap between the title and the description (default: the theme's `space_xs`).
    pub fn header_spacing(mut self, px: f32) -> Self {
        self.header_spacing = Some(px.max(0.0));
        self
    }
    /// The gap between the action buttons (default: the theme's `space_s`).
    pub fn actions_spacing(mut self, px: f32) -> Self {
        self.actions_spacing = Some(px.max(0.0));
        self
    }
    /// The padding around the whole form (default none).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = p.into();
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

        let mut body = column![]
            .spacing(f.spacing.unwrap_or(m.space_l))
            .width(Length::Fill);
        if f.title.is_some() || f.description.is_some() {
            let mut head = column![].spacing(f.header_spacing.unwrap_or(m.space_xs));
            if let Some(t) = &f.title {
                head = head.push(
                    f.props
                        .title
                        .apply(crate::components::typography::title_3(t.clone())),
                );
            }
            if let Some(d) = &f.description {
                head = head.push(
                    f.props
                        .description
                        .apply(crate::components::typography::body(d.clone()).style(st::dim)),
                );
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
            let mut ind = StepIndicator::new(items)
                .metrics(m)
                .props(f.props.indicator.clone());
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
        body = body.push(
            f.props
                .content
                .apply(container(content).width(Length::Fill)),
        );

        // Action bar.
        let reset = f.props.reset_button.apply(
            raised_button(f.props.reset_label.apply(label(&f.reset_label)))
                .on_press_maybe(idle(&f.on_reset)),
        );
        let back = (many && f.current > 0).then(|| {
            f.props.back_button.apply(
                raised_button(f.props.back_label.apply(label(&f.back_label)))
                    .on_press_maybe(idle(&f.on_back)),
            )
        });
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
        let primary = f.props.primary_button.apply(
            suggested_button(f.props.primary_label.apply(label(&primary_label)))
                .on_press_maybe(idle(&primary_msg)),
        );
        let bar_gap = f.actions_spacing.unwrap_or(m.space_s);
        let mut right = row![].spacing(bar_gap);
        if let Some(b) = back {
            right = right.push(b);
        }
        right = right.push(primary);
        let bar: Element<'a, M> = match f.actions {
            FormActions::Spread => row![reset, space().width(Length::Fill), right].into(),
            FormActions::End => row![space().width(Length::Fill), reset, right]
                .spacing(bar_gap)
                .into(),
            FormActions::Start => row![reset, right].spacing(bar_gap).into(),
        };
        body = body.push(bar);

        let el: Element<'a, M> =
            if f.padding == iced::Padding::ZERO && !f.props.has_surface_overrides() {
                body.into()
            } else {
                f.props
                    .surface
                    .apply(container(body).padding(f.padding))
                    .into()
            };
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
    props: super::stepper::StepperWidgetProps,
    items: Vec<(String, StepStatus)>,
    clickable: Vec<bool>,
    messages: Vec<M>,
    metrics: Metrics,
    _p: std::marker::PhantomData<&'a ()>,
}

impl<'a, M> StepIndicator<'a, M> {
    /// Presentation overrides for native children; actions remain parent-owned.
    pub fn props(mut self, props: super::stepper::StepperWidgetProps) -> Self {
        self.props = props;
        self
    }
    pub fn new(items: Vec<(String, StepStatus)>) -> Self {
        Self {
            props: super::stepper::StepperWidgetProps::default(),
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
            .props(s.props)
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

#[cfg(test)]
mod spacing_params_tests {
    use super::*;

    #[test]
    fn a_form_takes_its_gaps_and_padding() {
        let f = Form::<u8>::new()
            .title("T")
            .description("D")
            .step(Step::new("One", || iced::widget::text("x").into()))
            .spacing(1.0)
            .header_spacing(2.0)
            .actions_spacing(3.0)
            .padding(4.0)
            .layout(FormActions::End);
        assert_eq!(
            (f.spacing, f.header_spacing, f.actions_spacing),
            (Some(1.0), Some(2.0), Some(3.0))
        );
        assert_eq!(f.padding, iced::Padding::from(4.0));
        let _: Element<'_, u8> = f.into();
    }
}

/// Typed presentation for parent-built native children. Defaults preserve the original view.
#[derive(Clone, Default)]
pub struct FormWidgetProps {
    pub indicator: super::stepper::StepperWidgetProps,
    pub title: crate::components::TextProps,
    pub description: crate::components::TextProps,
    pub content: crate::components::ContainerProps,
    pub reset_button: crate::components::ButtonProps,
    pub reset_label: crate::components::TextProps,
    pub back_button: crate::components::ButtonProps,
    pub back_label: crate::components::TextProps,
    pub primary_button: crate::components::ButtonProps,
    pub primary_label: crate::components::TextProps,
    pub surface: crate::components::ContainerProps,
}

impl FormWidgetProps {
    fn has_surface_overrides(&self) -> bool {
        let p = &self.surface;
        p.width.is_some()
            || p.height.is_some()
            || p.max_width.is_some()
            || p.max_height.is_some()
            || p.padding.is_some()
            || p.align_x.is_some()
            || p.align_y.is_some()
            || p.clip.is_some()
            || p.style.is_some()
    }
}

impl FormWidgetProps {
    pub fn indicator(mut self, props: super::stepper::StepperWidgetProps) -> Self {
        self.indicator = props;
        self
    }
    pub fn title(mut self, props: crate::components::TextProps) -> Self {
        self.title = props;
        self
    }
    pub fn description(mut self, props: crate::components::TextProps) -> Self {
        self.description = props;
        self
    }
    pub fn content(mut self, props: crate::components::ContainerProps) -> Self {
        self.content = props;
        self
    }
    pub fn reset_button(mut self, props: crate::components::ButtonProps) -> Self {
        self.reset_button = props;
        self
    }
    pub fn reset_label(mut self, props: crate::components::TextProps) -> Self {
        self.reset_label = props;
        self
    }
    pub fn back_button(mut self, props: crate::components::ButtonProps) -> Self {
        self.back_button = props;
        self
    }
    pub fn back_label(mut self, props: crate::components::TextProps) -> Self {
        self.back_label = props;
        self
    }
    pub fn primary_button(mut self, props: crate::components::ButtonProps) -> Self {
        self.primary_button = props;
        self
    }
    pub fn primary_label(mut self, props: crate::components::TextProps) -> Self {
        self.primary_label = props;
        self
    }
    pub fn surface(mut self, props: crate::components::ContainerProps) -> Self {
        self.surface = props;
        self
    }
    pub fn apply<'a, M>(&self, widget: Form<'a, M>) -> Form<'a, M>
    where
        M: Clone + 'a,
    {
        widget.props(self.clone())
    }
}

#[cfg(test)]
mod native_props_tests {
    use super::*;
    use crate::widgets::synced_text::native_props_tests::{assert_override, text_props};
    #[test]
    fn defaults_preserve_layout_and_overrides_reach_native_children() {
        let make = || {
            Form::<()>::new()
                .title("Title")
                .step(Step::new("One", || iced::widget::space().into()))
        };
        assert_override(
            make().into(),
            make().props(FormWidgetProps::default()).into(),
            make()
                .props(FormWidgetProps::default().title(text_props()))
                .into(),
        );
    }
}

#[cfg(test)]
mod native_behavior_tests {
    use super::*;
    use crate::components::ButtonProps;
    use crate::widgets::synced_text::native_props_tests::click_sized;
    #[test]
    fn presentation_preserves_submit_and_busy_policy() {
        let make = || {
            Form::new().enter_submits(false).on_submit(7).props(
                FormWidgetProps::default()
                    .primary_button(ButtonProps::default().width(137).height(29)),
            )
        };
        assert_eq!(click_sized(make().into()), vec![7]);
        assert!(click_sized(make().submitting(true).into()).is_empty());
    }
}
