//! `Collapsible`: the disclosure primitive (Radix `Collapsible`). A header that is always
//! shown and a body that slides open and closed ([`reveal`](crate::layouts::reveal), 250 ms).
//! Controlled: the application owns `open`, and the header carries the trigger.
//!
//! [`ExpanderRow`](crate::widgets::ExpanderRow) is a collapsible with a row header;
//! [`Accordion`](crate::widgets::Accordion) is several of them under a [`Selection`](crate::primitives::Selection).

use crate::Element;
use crate::components::ContainerProps;
use iced::widget::column;
use iced::widget::container;

pub struct Collapsible<'a, M> {
    header: Element<'a, M>,
    body: Element<'a, M>,
    open: bool,
    props: CollapsibleProps,
}

impl<'a, M: 'a> Collapsible<'a, M> {
    pub fn header_props(mut self, props: ContainerProps) -> Self {
        self.props.header_props = Some(props);
        self
    }
    pub fn body_props(mut self, props: ContainerProps) -> Self {
        self.props.body_props = Some(props);
        self
    }
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.props.container_props = Some(props);
        self
    }

    pub fn new(
        header: impl Into<Element<'a, M>>,
        body: impl Into<Element<'a, M>>,
        open: bool,
    ) -> Self {
        Self {
            props: CollapsibleProps::default(),
            header: header.into(),
            body: body.into(),
            open,
        }
    }
}

impl<'a, M: 'a> From<Collapsible<'a, M>> for Element<'a, M> {
    fn from(c: Collapsible<'a, M>) -> Self {
        // The body stays mounted so it animates closed as well as open.
        let header = match c.props.header_props {
            Some(p) => p.apply(container(c.header)).into(),
            None => c.header,
        };
        let body = match c.props.body_props {
            Some(p) => p.apply(container(c.body)).into(),
            None => c.body,
        };
        let content = column![header, Element::from(crate::layouts::reveal(body, c.open))];
        match c.props.container_props {
            Some(p) => p.apply(container(content)).into(),
            None => content.into(),
        }
    }
}

/// Optional presentation wrappers; defaults leave the disclosure tree unchanged.
#[derive(Clone, Default)]
pub struct CollapsibleProps {
    pub header_props: Option<ContainerProps>,
    pub body_props: Option<ContainerProps>,
    pub container_props: Option<ContainerProps>,
}
impl CollapsibleProps {
    pub fn header_props(mut self, props: ContainerProps) -> Self {
        self.header_props = Some(props);
        self
    }
    pub fn body_props(mut self, props: ContainerProps) -> Self {
        self.body_props = Some(props);
        self
    }
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = Some(props);
        self
    }
    pub fn apply<'a, M: 'a>(&self, mut widget: Collapsible<'a, M>) -> Collapsible<'a, M> {
        if let Some(props) = &self.header_props {
            widget = widget.header_props(props.clone());
        }
        if let Some(props) = &self.body_props {
            widget = widget.body_props(props.clone());
        }
        if let Some(props) = &self.container_props {
            widget = widget.container_props(props.clone());
        }
        widget
    }
}

#[cfg(test)]
mod typed_presentation_tests {
    #[test]
    fn presentation_does_not_change_controlled_state_or_unmount_closed_body() {
        let props = CollapsibleProps::default()
            .container_props(ContainerProps::default().width(123).height(45));
        let c = props.apply(Collapsible::<u8>::new(
            iced::widget::space(),
            iced::widget::space(),
            false,
        ));
        assert!(!c.open);
        let element: Element<'_, u8> = c.into();
        assert_eq!(
            element.as_widget().size(),
            iced::Size::new(iced::Length::Fixed(123.0), iced::Length::Fixed(45.0))
        );
        let tree = Tree::new(&element);
        assert_eq!(tree.children.len(), 2, "header and reveal stay mounted");
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
        let build = || Collapsible::<u8>::new(iced::widget::space(), iced::widget::space(), false);
        let w = build();
        let baseline: Element<'_, u8> = w.into();
        let w = CollapsibleProps::default().apply(build());
        let configured: Element<'_, u8> = w.into();
        assert_eq!(baseline.as_widget().size(), configured.as_widget().size());
        same_tree(&Tree::new(&baseline), &Tree::new(&configured));
    }
}
