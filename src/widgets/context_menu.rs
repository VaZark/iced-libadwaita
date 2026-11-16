//! Context menus wrapping arbitrary caller-owned content.
use super::menu::{Menu, pages_content, popover_content, popover_max_height, popover_width};
use super::popover::{Appearance, Popover, PopoverProps};
use crate::{Element, Metrics};

/// Native floating-surface presentation. Input and open state belong to the wrapper.
#[derive(Clone, Default)]
pub struct ContextMenuProps {
    pub popover: PopoverProps,
}

/// A native [`Menu`] opened over caller-owned content by right click, Menu, or
/// Shift+F10. Keyboard invocation belongs to the focused target or its focused
/// descendant. The wrapper does not replace the target's normal left-click behavior.
///
/// Escape and outside presses dismiss the surface. Arrow keys/Home/End select
/// enabled rows; Enter/Space activate; Right/Left enter and leave submenu pages.
/// Open state and command messages remain owned by the menu, not presentation props.
pub struct ContextMenu<'a, M> {
    target: Element<'a, M>,
    menu: Menu<M>,
    props: ContextMenuProps,
}

/// Attach a context menu to arbitrary content without turning it into a button.
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
    /// Override safe surface presentation; trigger padding and appearance remain
    /// transparent so the caller's content keeps its original geometry.
    pub fn props(mut self, props: ContextMenuProps) -> Self {
        self.props = props;
        self
    }
}

impl<'a, M: Clone + 'a> From<ContextMenu<'a, M>> for Element<'a, M> {
    fn from(value: ContextMenu<'a, M>) -> Self {
        let width = popover_width();
        let content = if value.menu.has_submenu() {
            pages_content(&value.menu, width, 0)
        } else {
            popover_content(&value.menu, popover_max_height())
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

#[cfg(test)]
mod tests {
    use super::super::menu::MenuItem;
    use super::*;
    use iced::advanced::{Shell, layout, renderer::Headless, widget::Tree};
    use iced::{Event, Point, Rectangle, Size, mouse};

    #[test]
    fn surface_props_preserve_target_geometry_and_submenus_fit_a_small_viewport() {
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let child = Menu::new("Child").section([MenuItem::action("Command", ())]);
        let menu = Menu::new("Context").section([MenuItem::submenu("Child", child)]);
        let mut element: Element<'_, ()> =
            context_menu(iced::widget::space().width(100).height(30), menu)
                .props(ContextMenuProps {
                    popover: PopoverProps::default()
                        .padding(40)
                        .appearance(Appearance::Button),
                })
                .into();
        let mut tree = Tree::new(&element);
        let viewport = Rectangle::with_size(Size::new(80.0, 60.0));
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(500.0, 300.0)),
        );
        assert_eq!(node.size(), Size::new(100.0, 30.0));
        element.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)),
            layout::Layout::new(&node),
            mouse::Cursor::Available(Point::new(20.0, 20.0)),
            &renderer,
            &mut iced::advanced::clipboard::Null,
            &mut Shell::new(&mut vec![]),
            &viewport,
        );
        let mut overlay = element
            .as_widget_mut()
            .overlay(
                &mut tree,
                layout::Layout::new(&node),
                &renderer,
                &viewport,
                iced::Vector::ZERO,
            )
            .unwrap();
        let bounds = overlay
            .as_overlay_mut()
            .layout(&renderer, viewport.size())
            .bounds();
        assert!(bounds.x >= 0.0 && bounds.y >= 0.0, "{bounds:?}");
        assert!(
            bounds.x + bounds.width <= viewport.width
                && bounds.y + bounds.height <= viewport.height,
            "{bounds:?}"
        );
    }
}
