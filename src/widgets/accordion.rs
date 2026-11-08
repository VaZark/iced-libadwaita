//! `Accordion`: a group of collapsible items under a [`Selection`] rule (Radix `Accordion`).
//! One surface (a boxed list), one full-surface TreeRow disclosure header per item, bodies that slide.
//!
//! ```ignore
//! Accordion::new(&self.open, Msg::Open)            // open: Vec<Key>
//!     .selection(Selection::SINGLE)                // or Multiple (default)
//!     .item(AccordionItem::new(Key::A, "Shipping", body_a).subtitle("2-4 days"))
//!     .item(AccordionItem::new(Key::B, "Returns", body_b))
//! ```

use crate::Element;
use crate::primitives::Selection;
use std::rc::Rc;

pub struct AccordionItem<'a, K, M> {
    key: K,
    title: String,
    subtitle: Option<String>,
    body: Element<'a, M>,
}

impl<'a, K, M> AccordionItem<'a, K, M> {
    pub fn new(key: K, title: impl Into<String>, body: impl Into<Element<'a, M>>) -> Self {
        Self {
            key,
            title: title.into(),
            subtitle: None,
            body: body.into(),
        }
    }
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.subtitle = Some(s.into());
        self
    }
}

pub struct Accordion<'a, K, M> {
    open: Vec<K>,
    on_change: Rc<dyn Fn(Vec<K>) -> M + 'a>,
    selection: Selection,
    items: Vec<AccordionItem<'a, K, M>>,
}

impl<'a, K: Clone + PartialEq + 'a, M: 'a> Accordion<'a, K, M> {
    /// `open`: the keys of the open items (controlled). `on_change` receives the new list.
    pub fn new(open: &[K], on_change: impl Fn(Vec<K>) -> M + 'a) -> Self {
        Self {
            open: open.to_vec(),
            on_change: Rc::new(on_change),
            selection: Selection::Multiple,
            items: Vec::new(),
        }
    }
    pub fn selection(mut self, s: Selection) -> Self {
        self.selection = s;
        self
    }
    pub fn item(mut self, item: AccordionItem<'a, K, M>) -> Self {
        self.items.push(item);
        self
    }
}

impl<'a, K: Clone + PartialEq + 'a, M: Clone + 'a> From<Accordion<'a, K, M>> for Element<'a, M> {
    fn from(a: Accordion<'a, K, M>) -> Self {
        let rows = a.items.into_iter().map(|item| {
            let is_open = a.open.contains(&item.key);
            let (open, on_change, rule, key) =
                (a.open.clone(), a.on_change.clone(), a.selection, item.key);
            let mut row = crate::widgets::TreeRow::new(item.title)
                .expanded(is_open)
                .on_toggle(move |_| on_change(rule.toggle(&open, &key)));
            if let Some(s) = item.subtitle {
                row = row.subtitle(s);
            }
            Element::from(row.push(item.body))
        });
        crate::widgets::boxed_list(rows)
    }
}
