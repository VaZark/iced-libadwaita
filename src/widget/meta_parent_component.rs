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
//!     .listener("list", |mode| {
//!         ReorderableList::new(items, row).editing(mode == Some(&Mode::Edit)).into()
//!     })
//!     .child(title_text)                        // never notified
//! ```

use super::{Element, MetaComponent};
use crate::primitives::meta_parent::Registry;
use iced::Length;
use iced::widget::{Column, Row};

/// See the [module](self).
pub struct MetaParentComponent<'a, Id, K, M> {
    registry: &'a Registry<Id, K>,
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
    pub fn listener(mut self, id: Id, build: impl FnOnce(Option<&K>) -> Element<'a, M>) -> Self {
        self.children.push(build(self.registry.current(&id)));
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
    fn listeners_get_what_they_were_switched_to_and_plain_children_nothing() {
        let mut r = Reg::new();
        r.register("list", ["edit", "preview"]);
        r.notify(&"edit");
        let mut seen = Vec::new();
        let _: Element<'_, ()> = P::new(&r)
            .listener("list", |a| {
                seen.push(a.copied());
                iced::widget::space().into()
            })
            .listener("stranger", |a| {
                seen.push(a.copied());
                iced::widget::space().into()
            })
            .child(iced::widget::space())
            .into();
        assert_eq!(seen, vec![Some("edit"), None]);
    }
}
