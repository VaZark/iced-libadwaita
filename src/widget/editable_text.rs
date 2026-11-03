//! `EditableText`: text that is read until you ask to change it. A long text is clamped to a few
//! lines ([`Spoiler`](super::Spoiler)) and **pushes what is below it down** when expanded; an
//! edit button swaps it for an [`Entry`](super::Entry), and Enter or the check button finishes.
//!
//! One piece of content, two variants chosen by the application's `editing` flag with the same
//! rules a [`MetaComponent`](super::MetaComponent) uses, built with
//! [`flow`](super::MetaComponent::flow) so the result takes the height of its content instead of
//! filling its container. While the value is empty a dimmed placeholder stands in, and is itself
//! the edit button.
//!
//! ```ignore
//! EditableText::new(&self.bio, Msg::Bio)
//!     .placeholder("Add a description")
//!     .editing(self.editing)
//!     .expanded(self.expanded)
//!     .on_edit(Msg::Edit)
//!     .on_done(Msg::Done)
//!     .on_expand(Msg::Expand)
//! ```

use super::{Element, MetaComponent, typography as ty};
use crate::style::text as st;
use crate::{Icon, Metrics};
use iced::widget::{container, row};
use iced::{Alignment, Length};
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Variant {
    Entry,
    Text,
}

/// See the [module](self).
pub struct EditableText<'a, M> {
    value: &'a str,
    on_input: Rc<dyn Fn(String) -> M + 'a>,
    placeholder: String,
    editing: bool,
    expanded: bool,
    max_height: f32,
    on_edit: Option<M>,
    on_done: Option<M>,
    on_expand: Option<Rc<dyn Fn(bool) -> M + 'a>>,
    metrics: Metrics,
}

impl<'a, M: Clone + 'a> EditableText<'a, M> {
    /// `value` is the text; `on_input` gets what is typed while editing.
    pub fn new(value: &'a str, on_input: impl Fn(String) -> M + 'a) -> Self {
        Self {
            value,
            on_input: Rc::new(on_input),
            placeholder: String::new(),
            editing: false,
            expanded: false,
            max_height: 80.0,
            on_edit: None,
            on_done: None,
            on_expand: None,
            metrics: Metrics::desktop(),
        }
    }
    /// What an empty value says (and the entry's placeholder).
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }
    /// Whether the entry shows (controlled).
    pub fn editing(mut self, editing: bool) -> Self {
        self.editing = editing;
        self
    }
    /// Whether a long text shows whole (controlled).
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
    /// The height the text is clamped to (default 80).
    pub fn max_height(mut self, px: f32) -> Self {
        self.max_height = px;
        self
    }
    /// The edit button was pressed: start editing.
    pub fn on_edit(mut self, m: M) -> Self {
        self.on_edit = Some(m);
        self
    }
    /// Enter or the check button: finish editing.
    pub fn on_done(mut self, m: M) -> Self {
        self.on_done = Some(m);
        self
    }
    /// The Show more / Hide control: the state the text would take.
    pub fn on_expand(mut self, f: impl Fn(bool) -> M + 'a) -> Self {
        self.on_expand = Some(Rc::new(f));
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: Clone + 'a> From<EditableText<'a, M>> for Element<'a, M> {
    fn from(t: EditableText<'a, M>) -> Self {
        let m = t.metrics;
        let (value, placeholder) = (t.value, t.placeholder);
        let (on_edit, on_done) = (t.on_edit, t.on_done);
        let (on_input, on_expand) = (t.on_input, t.on_expand);
        let (expanded, max_height) = (t.expanded, t.max_height);
        let (p1, p2) = (placeholder.clone(), placeholder);
        let (done1, edit1, edit2) = (on_done.clone(), on_edit.clone(), on_edit);

        MetaComponent::new()
            .selected(t.editing.then_some(Variant::Entry))
            .when_selected(Variant::Entry, move |_| {
                let mut e = super::entry(p1.clone(), value)
                    .on_input({
                        let f = on_input.clone();
                        move |s| f(s)
                    })
                    .metrics(m)
                    .width(Length::Fill);
                if let Some(d) = done1.clone() {
                    e = e.on_submit(d);
                }
                let mut done = super::icon_button(Icon::Check);
                if let Some(d) = done1.clone() {
                    done = done.on_press(d);
                }
                row![e, done]
                    .spacing(m.space_s)
                    .align_y(Alignment::Center)
                    .into()
            })
            .variant(Variant::Text, super::When::always(), move |_| {
                let edit = || {
                    let mut b = super::icon_button(Icon::Edit);
                    if let Some(e) = edit1.clone() {
                        b = b.on_press(e);
                    }
                    b
                };
                if value.is_empty() {
                    let mut b = super::flat_button(
                        row![ty::body(p2.clone()).style(st::dim), Icon::Edit.view(14.0)]
                            .spacing(m.space_s)
                            .align_y(Alignment::Center),
                    );
                    if let Some(e) = edit2.clone() {
                        b = b.on_press(e);
                    }
                    return b.into();
                }
                let mut spoiler = super::Spoiler::new(ty::body(value.to_string()))
                    .max_height(max_height)
                    .expanded(expanded)
                    .metrics(m);
                if let Some(f) = on_expand.clone() {
                    spoiler = spoiler.on_toggle(move |x| f(x));
                }
                row![container(spoiler).width(Length::Fill), edit()]
                    .spacing(m.space_s)
                    .align_y(Alignment::Start)
                    .into()
            })
            .flow(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builder_keeps_what_it_is_given() {
        let t = EditableText::new("hi", |s| s)
            .placeholder("Add")
            .editing(true)
            .max_height(40.0);
        assert!(t.editing && t.max_height == 40.0 && t.placeholder == "Add");
    }
}
