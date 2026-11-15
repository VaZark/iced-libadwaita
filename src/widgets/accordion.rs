//! `Accordion`: a group of collapsible items under a [`Selection`] rule (Radix `Accordion`).
//! One surface (a boxed list), one full-surface TreeRow disclosure header per item, bodies that slide.
//!
//! ```ignore
//! Accordion::new(&self.open, Msg::Open)            // open: Vec<Key>
//!     .selection(Selection::SINGLE)                // or Multiple (default)
//!     .item(AccordionItem::new(Key::A, "Shipping", body_a).subtitle("2-4 days"))
//!     .item(AccordionItem::new(Key::B, "Returns", body_b))
//! ```

use super::status_page::CardProps;
use crate::Element;
use crate::components::{ButtonProps, ContainerProps, TextProps};
use crate::primitives::Selection;
use std::rc::Rc;

pub struct AccordionItem<'a, K, M> {
    key: K,
    title: String,
    subtitle: Option<String>,
    body: Element<'a, M>,
    props: AccordionItemProps,
}

impl<'a, K, M> AccordionItem<'a, K, M> {
    pub fn props(mut self, props: AccordionItemProps) -> Self {
        self.props = props;
        self
    }
    pub fn title_props(mut self, props: TextProps) -> Self {
        self.props.title_props = Some(props);
        self
    }
    pub fn subtitle_props(mut self, props: TextProps) -> Self {
        self.props.subtitle_props = Some(props);
        self
    }
    pub fn header_props(mut self, props: ContainerProps) -> Self {
        self.props.header_props = Some(props);
        self
    }
    pub fn body_props(mut self, props: ContainerProps) -> Self {
        self.props.body_props = Some(props);
        self
    }
    pub fn action_props(mut self, props: ButtonProps) -> Self {
        self.props.action_props = Some(props);
        self
    }

    pub fn new(key: K, title: impl Into<String>, body: impl Into<Element<'a, M>>) -> Self {
        Self {
            props: AccordionItemProps::default(),
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
    card_props: CardProps,
    open: Vec<K>,
    on_change: Rc<dyn Fn(Vec<K>) -> M + 'a>,
    selection: Selection,
    items: Vec<AccordionItem<'a, K, M>>,
}

impl<'a, K: Clone + PartialEq + 'a, M: 'a> Accordion<'a, K, M> {
    pub fn card_props(mut self, props: CardProps) -> Self {
        self.card_props = props;
        self
    }
    /// `open`: the keys of the open items (controlled). `on_change` receives the new list.
    pub fn new(open: &[K], on_change: impl Fn(Vec<K>) -> M + 'a) -> Self {
        Self {
            card_props: CardProps::default(),
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
                .title_props(item.props.title_props.unwrap_or_default())
                .subtitle_props(item.props.subtitle_props.unwrap_or_default())
                .header_props(item.props.header_props.unwrap_or_default())
                .children_props(item.props.body_props.unwrap_or_default())
                .header_button_props(item.props.action_props.unwrap_or_default())
                .expanded(is_open)
                .on_toggle(move |_| on_change(rule.toggle(&open, &key)));
            if let Some(s) = item.subtitle {
                row = row.subtitle(s);
            }
            Element::from(row.push(item.body))
        });
        let mut rows = rows;
        let Some(first) = rows.next() else {
            return iced::widget::space().into();
        };
        let card = rows.fold(crate::widgets::Card::new(first).padding(0), |card, row| {
            card.push(row)
        });
        a.card_props.apply(card).into()
    }
}

/// Presentation shared by an accordion item's concrete TreeRow children.
#[derive(Clone, Default)]
pub struct AccordionItemProps {
    pub title_props: Option<TextProps>,
    pub subtitle_props: Option<TextProps>,
    pub header_props: Option<ContainerProps>,
    pub body_props: Option<ContainerProps>,
    pub action_props: Option<ButtonProps>,
}
impl AccordionItemProps {
    pub fn title_props(mut self, props: TextProps) -> Self {
        self.title_props = Some(props);
        self
    }
    pub fn subtitle_props(mut self, props: TextProps) -> Self {
        self.subtitle_props = Some(props);
        self
    }
    pub fn header_props(mut self, props: ContainerProps) -> Self {
        self.header_props = Some(props);
        self
    }
    pub fn body_props(mut self, props: ContainerProps) -> Self {
        self.body_props = Some(props);
        self
    }
    pub fn action_props(mut self, props: ButtonProps) -> Self {
        self.action_props = Some(props);
        self
    }
    pub fn apply<'a, K, M>(&self, widget: AccordionItem<'a, K, M>) -> AccordionItem<'a, K, M> {
        let mut widget = widget;
        if let Some(props) = &self.title_props {
            widget = widget.title_props(props.clone());
        }
        if let Some(props) = &self.subtitle_props {
            widget = widget.subtitle_props(props.clone());
        }
        if let Some(props) = &self.header_props {
            widget = widget.header_props(props.clone());
        }
        if let Some(props) = &self.body_props {
            widget = widget.body_props(props.clone());
        }
        if let Some(props) = &self.action_props {
            widget = widget.action_props(props.clone());
        }
        widget
    }
}
#[derive(Clone, Default)]
pub struct AccordionProps {
    pub card_props: Option<CardProps>,
}
impl AccordionProps {
    pub fn card_props(mut self, props: CardProps) -> Self {
        self.card_props = Some(props);
        self
    }
    pub fn apply<'a, K: Clone + PartialEq + 'a, M: 'a>(
        &self,
        widget: Accordion<'a, K, M>,
    ) -> Accordion<'a, K, M> {
        match &self.card_props {
            Some(props) => widget.card_props(props.clone()),
            None => widget,
        }
    }
}

#[cfg(test)]
mod typed_presentation_tests {
    #[test]
    fn presentation_retains_controlled_selection_and_callback() {
        let accordion = AccordionProps::default()
            .card_props(CardProps::default().container_props(ContainerProps::default().width(123)))
            .apply(
                Accordion::new(&[1u8], |keys| keys)
                    .selection(Selection::SINGLE)
                    .item(
                        AccordionItemProps::default()
                            .title_props(TextProps::default().size(23))
                            .apply(AccordionItem::new(1, "Title", iced::widget::space())),
                    ),
            );
        assert_eq!(accordion.open, vec![1]);
        assert_eq!((accordion.on_change)(vec![2]), vec![2]);
        assert_eq!(accordion.items[0].key, 1);
        assert_eq!(
            accordion.items[0].props.title_props.as_ref().unwrap().size,
            Some(iced::Pixels(23.0))
        );
        let _: Element<'_, Vec<u8>> = accordion.into();
    }

    use super::*;
    use iced::advanced::widget::Tree;
    fn same_tree(a: &Tree, b: &Tree) {
        assert!(a.tag == b.tag);
        assert_eq!(a.children.len(), b.children.len());
        for (a, b) in a.children.iter().zip(&b.children) {
            same_tree(a, b);
        }
    }
    #[test]
    fn reusable_defaults_preserve_geometry_and_tree() {
        let build = || {
            Accordion::new(&[1u8], |_: Vec<u8>| 7u8).item(AccordionItem::new(
                1u8,
                "Title",
                iced::widget::space(),
            ))
        };
        let w = build();
        let baseline: Element<'_, u8> = w.into();
        let w = AccordionProps::default().apply(build());
        let configured: Element<'_, u8> = w.into();
        assert_eq!(baseline.as_widget().size(), configured.as_widget().size());
        same_tree(&Tree::new(&baseline), &Tree::new(&configured));
    }
}
