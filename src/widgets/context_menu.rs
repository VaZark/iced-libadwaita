//! Context menus wrapping arbitrary caller-owned content.
use super::menu::{Menu, pages_content, popover_content};
use super::popover::{Appearance, Popover, PopoverProps};
use crate::{Element, Metrics};

/// Native floating-surface presentation. Input and open state belong to the wrapper.
#[derive(Clone, Default)]
pub struct ContextMenuProps {
    pub popover: PopoverProps,
}

pub struct ContextMenu<'a, M> {
    target: Element<'a, M>,
    menu: Menu<M>,
    props: ContextMenuProps,
}

pub fn context_menu<'a, M: Clone + 'a>(
    target: impl Into<Element<'a, M>>,
    menu: Menu<M>,
) -> ContextMenu<'a, M> {
    ContextMenu::new(target, menu)
}

impl<'a, M: Clone + 'a> ContextMenu<'a, M> {
    pub fn new(target: impl Into<Element<'a, M>>, menu: Menu<M>) -> Self {
        Self {
            target: target.into(),
            menu,
            props: ContextMenuProps::default(),
        }
    }
    pub fn props(mut self, props: ContextMenuProps) -> Self {
        self.props = props;
        self
    }
}

impl<'a, M: Clone + 'a> From<ContextMenu<'a, M>> for Element<'a, M> {
    fn from(value: ContextMenu<'a, M>) -> Self {
        let width = 260.0;
        let content = if value.menu.has_submenu() {
            pages_content(&value.menu, width, 0)
        } else {
            popover_content(&value.menu, 360.0)
        };
        // Trigger geometry must remain the caller's: popover padding and appearance
        // are surface options, not permission to turn the target into a button.
        let popover = Popover::new(value.target, content)
            .popover_width(width)
            .offset(Metrics::desktop().space_xxs)
            .props(value.props.popover)
            .appearance(Appearance::Flat)
            .padding(0)
            .context_trigger();
        popover.into()
    }
}
