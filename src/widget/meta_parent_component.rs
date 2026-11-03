//! `MetaParentComponent`: a container that **notifies its meta children** to switch between
//! their modes. It owns no content of its own and no state; the app owns a
//! [`MetaRegistry`](super::MetaRegistry) and the parent reads it.
//!
//! * **Registration.** Every child registers separately and announces the actions it
//!   implements: [`register`](MetaParentComponent::register) takes them from a
//!   [`MetaComponent`] (the keys its variants are selected by), and
//!   [`Registry::register`] takes them from any component that has modes of its own, such as a
//!   [`ReorderableList`](super::ReorderableList) (`Mode::Edit` and `Mode::Preview`).
//! * **Notification.** `registry.notify(&action)` switches exactly the children that announced
//!   `action`. A child that did not announce it (an *unrelated* `MetaComponent`) and a child
//!   that never registered (any non-meta widget) are left as they are.
//! * **Display.** [`meta`](MetaParentComponent::meta) puts a registered `MetaComponent` in with
//!   the action it was last switched to as its selection; [`listener`](MetaParentComponent::listener)
//!   hands that action to a closure that builds any other mode-aware component;
//!   [`child`](MetaParentComponent::child) adds a plain element that is never notified.
//!
//! ```ignore
//! // once, when the app starts: each child announces what it implements
//! let mut modes = Registry::new();
//! MetaParentComponent::register(&mut modes, "bar", &search_bar());        // Bar::Search…
//! modes.register("list", [Mode::Edit, Mode::Preview]);                     // a ReorderableList
//!
//! // in update: tell the children
//! modes.notify(&Mode::Edit);                    // "list" switches; "bar" does not
//!
//! // in view
//! MetaParentComponent::new(&modes)
//!     .meta("bar", search_bar())                // switched, if it announced the action
//!     .on_child_event(Msg::Child)               // what children report: (id, ChildEvent)
//!     .listener("list", |mode, report| {
//!         ReorderableList::of(&self.order, row)
//!             .editing(mode == Some(&Mode::Edit))
//!             .on_event(Msg::List)              // the list's own message
//!             .on_change(report.on_change())    // and the parent's, independently
//!             .into()
//!     })
//!     .child(title_text)                        // never notified
//! ```

use super::{Element, MetaComponent};
use crate::primitives::meta_parent::{ChildEvent, Registry};
use iced::Length;
use iced::widget::{Column, Row};
use std::rc::Rc;

type Report<'a, Id, M> = Rc<dyn Fn(Id, ChildEvent) -> M + 'a>;

/// Turns what one child did into the parent's message. A parent hands one to each
/// [`listener`](MetaParentComponent::listener); the child's own callbacks stay its own.
pub struct Reporter<'a, Id, M> {
    id: Id,
    report: Option<Report<'a, Id, M>>,
}

impl<'a, Id: Clone, M> Reporter<'a, Id, M> {
    /// The message for `event`, or `None` when the parent has no
    /// [`on_child_event`](MetaParentComponent::on_child_event) callback.
    pub fn report(&self, event: ChildEvent) -> Option<M> {
        self.report.as_ref().map(|f| f(self.id.clone(), event))
    }
    /// For [`ReorderableList::on_change`](super::ReorderableList::on_change): the message for a
    /// row that moved. `None` (no callback) is turned into nothing by [`Reporter::on_change`].
    pub fn on_change(self) -> impl Fn(usize, usize) -> Option<M> + 'a
    where
        Id: 'a,
        M: 'a,
    {
        move |from, to| self.report(ChildEvent::Reordered { from, to })
    }
}

/// See the [module](self).
pub struct MetaParentComponent<'a, Id, K, M> {
    registry: &'a Registry<Id, K>,
    on_child_event: Option<Report<'a, Id, M>>,
    children: Vec<Element<'a, M>>,
    spacing: f32,
    horizontal: bool,
    width: Length,
    height: Length,
}

impl<'a, Id: PartialEq, K: PartialEq + Clone + 'a, M: 'a> MetaParentComponent<'a, Id, K, M> {
    pub fn new(registry: &'a Registry<Id, K>) -> Self {
        Self {
            registry,
            on_child_event: None,
            children: Vec::new(),
            spacing: 0.0,
            horizontal: false,
            width: Length::Fill,
            height: Length::Shrink,
        }
    }

    /// Registers `meta` under `id`, announcing the actions its variants answer to.
    pub fn register(registry: &mut Registry<Id, K>, id: Id, meta: &MetaComponent<'a, K, M>) {
        registry.register(id, meta.actions());
    }

    /// A registered meta child, shown with the action it was last switched to as its
    /// selection. An unregistered one, or one never switched, shows as it would alone.
    pub fn meta(mut self, id: Id, meta: MetaComponent<'a, K, M>) -> Self {
        let meta = match self.registry.current(&id) {
            Some(k) => meta.selected(Some(k.clone())),
            None => meta,
        };
        self.children.push(meta.into());
        self
    }

    /// A registered child that is not a `MetaComponent` but has modes of its own: `build`
    /// gets the action it was last switched to (`None` before any).
    ///
    /// It also gets a [`Reporter`] to wire the child's own events to the parent's
    /// [`on_child_event`](Self::on_child_event) callback.
    pub fn listener(
        mut self,
        id: Id,
        build: impl FnOnce(Option<&K>, Reporter<'a, Id, M>) -> Element<'a, M>,
    ) -> Self
    where
        Id: Clone,
    {
        let reporter = Reporter {
            id: id.clone(),
            report: self.on_child_event.clone(),
        };
        self.children
            .push(build(self.registry.current(&id), reporter));
        self
    }

    /// The parent's own callback for what its children report (a list's order changing): it
    /// gets the child's id and the [`ChildEvent`]. Set it *before* the listeners that report.
    /// Independent of the callbacks the children carry themselves; the app can run
    /// [`Registry::emit`] on it and react, e.g. by notifying other children.
    pub fn on_child_event(mut self, f: impl Fn(Id, ChildEvent) -> M + 'a) -> Self {
        self.on_child_event = Some(Rc::new(f));
        self
    }

    /// A plain child. It does not register and is never notified.
    pub fn child(mut self, child: impl Into<Element<'a, M>>) -> Self {
        self.children.push(child.into());
        self
    }

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }
    /// Lay the children out in a row instead of a column.
    pub fn horizontal(mut self, horizontal: bool) -> Self {
        self.horizontal = horizontal;
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }
}

impl<'a, Id, K, M: 'a> From<MetaParentComponent<'a, Id, K, M>> for Element<'a, M> {
    fn from(p: MetaParentComponent<'a, Id, K, M>) -> Self {
        if p.horizontal {
            Row::with_children(p.children)
                .spacing(p.spacing)
                .width(p.width)
                .height(p.height)
                .into()
        } else {
            Column::with_children(p.children)
                .spacing(p.spacing)
                .width(p.width)
                .height(p.height)
                .into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::When;

    type Reg = Registry<&'static str, &'static str>;
    type P<'a> = MetaParentComponent<'a, &'static str, &'static str, ()>;

    fn meta(keys: &[&'static str]) -> MetaComponent<'static, &'static str, ()> {
        let mut m = MetaComponent::new();
        for k in keys {
            m = m.when_selected(*k, |_| iced::widget::space().into());
        }
        m.variant("rest", When::always(), |_| iced::widget::space().into())
    }

    #[test]
    fn a_meta_component_announces_the_keys_its_variants_answer_to() {
        assert_eq!(
            meta(&["search", "filter"]).actions(),
            vec!["search", "filter"]
        );
    }

    #[test]
    fn registering_a_meta_component_announces_its_actions() {
        let mut r = Reg::new();
        P::register(&mut r, "bar", &meta(&["search"]));
        P::register(&mut r, "path", &meta(&["edit"]));
        assert_eq!(r.notify(&"search"), vec![&"bar"]);
        assert_eq!(r.current(&"path"), None, "the unrelated meta stays put");
    }

    #[test]
    fn a_listener_reports_to_the_parents_callback_and_without_one_to_nobody() {
        let mut r = Reg::new();
        r.register("list", ["edit"]);
        let mut got = Vec::new();
        let _: Element<'_, (&'static str, ChildEvent)> =
            MetaParentComponent::<&str, &str, _>::new(&r)
                .on_child_event(|id, e| (id, e))
                .listener("list", |_, rep| {
                    got.push(rep.on_change()(0, 2));
                    iced::widget::space().into()
                })
                .into();
        assert_eq!(
            got,
            vec![Some(("list", ChildEvent::Reordered { from: 0, to: 2 }))]
        );
        let mut none = Vec::new();
        let _: Element<'_, ()> = P::new(&r)
            .listener("list", |_, rep| {
                none.push(rep.on_change()(0, 2));
                iced::widget::space().into()
            })
            .into();
        assert_eq!(none, vec![None]);
    }

    #[test]
    fn listeners_get_what_they_were_switched_to_and_plain_children_nothing() {
        let mut r = Reg::new();
        r.register("list", ["edit", "preview"]);
        r.notify(&"edit");
        let mut seen = Vec::new();
        let _: Element<'_, ()> = P::new(&r)
            .listener("list", |a, _| {
                seen.push(a.copied());
                iced::widget::space().into()
            })
            .listener("stranger", |a, _| {
                seen.push(a.copied());
                iced::widget::space().into()
            })
            .child(iced::widget::space())
            .into();
        assert_eq!(seen, vec![Some("edit"), None]);
    }
}
