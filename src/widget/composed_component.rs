//! `ComposedComponent`: the opposite of [`MetaComponent`](super::MetaComponent). A meta
//! component has one piece of content and chooses between presentations. A composed component
//! has several components and **one shared state**, and renders as a single widget.
//!
//! Every part is built from the shared state and answers with the next state. Whichever part the
//! user touches, the application gets one value through one `on_change`, and every part is
//! built from that value on the next view, so they cannot disagree. A histogram over a range
//! slider, a color box next to a hex entry, a calendar beside a date entry: each pair edits one
//! value.
//!
//! ```ignore
//! ComposedComponent::new(self.range, Msg::Range)
//!     .part(|r| {
//!         BarChart::histogram(&self.histogram)
//!             .selected(|i| self.histogram.selected(i, *r))   // the bars inside the range
//!             .on_pick(|i| self.histogram.bin_range(i))        // a pressed bar is a range
//!             .into()
//!     })
//!     .part(|r| range_slider(0.0..=100.0, *r, |moved| moved).into())
//!     .normalize(|(lo, hi)| (lo.min(hi), lo.max(hi)))
//! ```
//!
//! * **State.** The application owns it ([controlled](crate::widget)); `new` takes the current
//!   value and `on_change` the message that brings back the next.
//! * **Parts.** [`part`](ComposedComponent::part) gets the current state and returns an element
//!   whose messages are the *next state*. Add as many as needed, in display order.
//! * **Rule.** [`normalize`](ComposedComponent::normalize) runs on every next state before it is
//!   reported, so the invariant (ordering, clamping) lives in one place, not in each part.
//! * **Layout.** A column by default; [`axis`](ComposedComponent::axis) for a row.

use super::Element;
use iced::widget::{column, row};
use iced::{Alignment, Length};
use std::rc::Rc;

/// Which way the parts are laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ComposeAxis {
    /// One above the next.
    #[default]
    Vertical,
    /// Side by side.
    Horizontal,
}

/// See the [module](self).
pub struct ComposedComponent<'a, S, M> {
    state: S,
    on_change: Rc<dyn Fn(S) -> M + 'a>,
    normalize: Option<Rc<dyn Fn(S) -> S + 'a>>,
    parts: Vec<Element<'a, S>>,
    axis: ComposeAxis,
    spacing: f32,
    width: Length,
}

impl<'a, S: 'a, M: 'a> ComposedComponent<'a, S, M> {
    /// `state` is the shared value; `on_change` turns the next one into the application's message.
    pub fn new(state: S, on_change: impl Fn(S) -> M + 'a) -> Self {
        Self {
            state,
            on_change: Rc::new(on_change),
            normalize: None,
            parts: Vec::new(),
            axis: ComposeAxis::Vertical,
            spacing: 8.0,
            width: Length::Fill,
        }
    }
    /// Adds a component. `build` receives the shared state; the element it returns sends the
    /// state that should replace it.
    pub fn part(mut self, build: impl FnOnce(&S) -> Element<'a, S>) -> Self {
        let e = build(&self.state);
        self.parts.push(e);
        self
    }
    /// The rule every next state passes before it is reported (clamp, order, round).
    pub fn normalize(mut self, rule: impl Fn(S) -> S + 'a) -> Self {
        self.normalize = Some(Rc::new(rule));
        self
    }
    pub fn axis(mut self, axis: ComposeAxis) -> Self {
        self.axis = axis;
        self
    }
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
}

fn report<S, M>(on_change: &Rc<dyn Fn(S) -> M + '_>, rule: Option<&dyn Fn(S) -> S>, s: S) -> M {
    on_change(match rule {
        Some(rule) => rule(s),
        None => s,
    })
}

impl<'a, S: 'a, M: 'a> From<ComposedComponent<'a, S, M>> for Element<'a, M> {
    fn from(c: ComposedComponent<'a, S, M>) -> Self {
        let (axis, spacing, width) = (c.axis, c.spacing, c.width);
        let (on_change, rule) = (c.on_change, c.normalize);
        let parts = c.parts.into_iter().map(|p| {
            let (on_change, rule) = (on_change.clone(), rule.clone());
            p.map(move |s| report(&on_change, rule.as_deref(), s))
        });
        match axis {
            ComposeAxis::Vertical => column(parts).spacing(spacing).width(width).into(),
            ComposeAxis::Horizontal => row(parts)
                .spacing(spacing)
                .width(width)
                .align_y(Alignment::Center)
                .into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parts_are_built_from_the_shared_state() {
        let seen = std::cell::RefCell::new(Vec::new());
        let _c: ComposedComponent<'_, (f32, f32), ()> = ComposedComponent::new((1.0, 2.0), |_| ())
            .part(|s| {
                seen.borrow_mut().push(*s);
                iced::widget::space().into()
            })
            .part(|s| {
                seen.borrow_mut().push(*s);
                iced::widget::space().into()
            });
        assert_eq!(*seen.borrow(), vec![(1.0, 2.0), (1.0, 2.0)]);
    }

    #[test]
    fn the_rule_applies_before_the_application_hears() {
        let c: ComposedComponent<'_, (f32, f32), (f32, f32)> =
            ComposedComponent::new((0.0, 1.0), |s| s)
                .normalize(|(a, b): (f32, f32)| (a.min(b), a.max(b)));
        assert_eq!(
            report(&c.on_change, c.normalize.as_deref(), (5.0, 2.0)),
            (2.0, 5.0)
        );
    }
}
