//! `Collapsible`: the disclosure primitive (Radix `Collapsible`). A header that is always
//! shown and a body that slides open and closed ([`reveal`](super::reveal), 250 ms).
//! Controlled: the application owns `open`, and the header carries the trigger.
//!
//! [`ExpanderRow`](super::ExpanderRow) is a collapsible with a row header;
//! [`Accordion`](super::Accordion) is several of them under a [`Selection`](super::Selection).

use super::Element;
use iced::widget::column;

pub struct Collapsible<'a, M> {
    header: Element<'a, M>,
    body: Element<'a, M>,
    open: bool,
}

impl<'a, M: 'a> Collapsible<'a, M> {
    pub fn new(
        header: impl Into<Element<'a, M>>,
        body: impl Into<Element<'a, M>>,
        open: bool,
    ) -> Self {
        Self {
            header: header.into(),
            body: body.into(),
            open,
        }
    }
}

impl<'a, M: 'a> From<Collapsible<'a, M>> for Element<'a, M> {
    fn from(c: Collapsible<'a, M>) -> Self {
        // The body stays mounted so it animates closed as well as open.
        column![c.header, Element::from(super::reveal(c.body, c.open))].into()
    }
}
