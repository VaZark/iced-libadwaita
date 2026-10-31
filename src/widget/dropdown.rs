//! `GtkDropDown` / the popover behind `AdwComboRow`.
//!
//! The closed state is a button (label + chevron). Opening shows a `.menu`
//! popover below it: rounded surface, 6px padding, items with a hover layer
//! and a trailing check on the selected one. Click outside or Escape closes.

use super::Element;
use crate::style::{button as sb, container as sc};
use crate::{Icon, Metrics, Theme};
use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{tree, Tree, Widget};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::alignment::Vertical;
use iced::keyboard::{self, key::Named};
use iced::widget::{button, column, container, row, space, text};
use iced::{mouse, Event, Length, Padding, Point, Rectangle, Shadow, Size};

/// How the closed dropdown is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Appearance {
    /// Regular translucent button.
    #[default]
    Button,
    /// Transparent until hovered (toolbars, `.flat`).
    Flat,
    /// Fills a list row (used by `ComboRow`).
    Row,
}

/// How the opened content is presented.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Presentation {
    /// A popover anchored to the trigger (desktop default).
    #[default]
    Popover,
    /// A full-window page that replaces the view, with a back affordance in its
    /// top-left corner ([`PAGE_HEADER_HEIGHT`] × [`PAGE_BACK_WIDTH`]). The content
    /// should provide its own header drawn in that area.
    Page,
}

/// Height of the full-page header region (matches the header bar height).
pub const PAGE_HEADER_HEIGHT: f32 = 47.0;
/// Width of the back-button hit area in a full-page presentation.
pub const PAGE_BACK_WIDTH: f32 = 64.0;

/// Where the popover sits relative to the trigger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Start,
    End,
}

pub struct Dropdown<'a, M> {
    trigger: Element<'a, M>,
    popover: Element<'a, M>,
    appearance: Appearance,
    align: Align,
    width: Length,
    padding: Padding,
    popover_width: Option<f32>,
    open_initially: bool,
    presentation: Presentation,
}

struct State {
    open: bool,
    /// 0 → 1 while the popover opens; restarted each time it opens.
    appear: crate::animation::Tween,
    shown: f32,
}

impl State {
    fn new(open: bool) -> Self {
        let start = if open && crate::animation::Trigger::Popover.animated() { 0.0 } else { 1.0 };
        Self { open, appear: crate::animation::Tween::new(start), shown: start }
    }
    fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            let start = if crate::animation::Trigger::Popover.animated() { 0.0 } else { 1.0 };
            self.appear = crate::animation::Tween::new(start);
            self.shown = start;
        }
    }
}

impl<'a, M: 'a> Dropdown<'a, M> {
    /// A dropdown with fully custom trigger content and popover content.
    pub fn custom(trigger: impl Into<Element<'a, M>>, popover: impl Into<Element<'a, M>>) -> Self {
        let m = Metrics::desktop();
        Self {
            trigger: trigger.into(),
            popover: popover.into(),
            appearance: Appearance::Button,
            align: Align::Start,
            width: Length::Shrink,
            padding: m.button_padding,
            popover_width: None,
            open_initially: false,
            presentation: Presentation::Popover,
        }
    }
    /// [`Presentation::Popover`] (default) or [`Presentation::Page`].
    pub fn presentation(mut self, p: Presentation) -> Self {
        self.presentation = p;
        self
    }
    /// Padding around the trigger content (0 for icon-only triggers).
    pub fn padding(mut self, p: impl Into<Padding>) -> Self {
        self.padding = p.into();
        self
    }
    /// Start with the popover open (screenshots, demos). Only read when the
    /// widget is first created.
    pub fn open_initially(mut self, open: bool) -> Self {
        self.open_initially = open;
        self
    }
    pub fn appearance(mut self, a: Appearance) -> Self {
        self.appearance = a;
        if a == Appearance::Row {
            self.padding = Padding::ZERO;
            self.width = Length::Fill;
        }
        self
    }
    pub fn align(mut self, a: Align) -> Self {
        self.align = a;
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    /// Popover width; defaults to the trigger width.
    pub fn popover_width(mut self, w: f32) -> Self {
        self.popover_width = Some(w);
        self
    }
}

/// A menu item for a dropdown popover: label with a trailing check when selected. The
/// selected item is the active one, in the accent color.
pub fn menu_item<'a, M: Clone + 'a>(
    label: impl Into<String>,
    selected: bool,
    on_press: M,
) -> Element<'a, M> {
    menu_item_inner(label.into(), selected, Some(on_press))
}

/// A menu item that cannot be chosen: dimmed text, no hover or press effect, no message.
/// (Distinct from selected, which is accent-colored.)
pub fn menu_item_disabled<'a, M: Clone + 'a>(label: impl Into<String>, selected: bool) -> Element<'a, M> {
    menu_item_inner(label.into(), selected, None)
}

fn menu_item_inner<'a, M: Clone + 'a>(label: String, selected: bool, on_press: Option<M>) -> Element<'a, M> {
    let m = Metrics::desktop();
    let mut r = row![text(label).wrapping(text::Wrapping::None).width(Length::Fill)]
        .spacing(m.space_m)
        .align_y(Vertical::Center);
    r = r.push(if selected {
        // The check follows the text color (accent when active, dimmed when disabled).
        let disabled = on_press.is_none();
        Element::from(Icon::Check.view(m.icon_size).style(move |t: &Theme, _| iced::widget::svg::Style {
            color: Some(if disabled {
                crate::color::fade(t.semantic.accent.color, t.semantic.opacity.disabled)
            } else {
                t.semantic.accent.color
            }),
        }))
    } else {
        space().width(m.icon_size).into()
    });
    button(container(r).padding([0.0, m.space_m]).center_y(Length::Fixed(32.0)))
        .padding(0)
        .width(Length::Fill)
        .style(sb::menu_item(selected))
        .on_press_maybe(on_press)
        .into()
}

/// The popover surface wrapping `items`: scrolls past `max_height`.
pub fn popover<'a, M: 'a>(
    items: impl IntoIterator<Item = Element<'a, M>>,
    max_height: f32,
) -> Element<'a, M> {
    let m = Metrics::desktop();
    container(super::scrolled_window(column(items).spacing(2).width(Length::Fill)))
        .padding(m.popover_padding)
        .max_height(max_height)
        .style(sc::popover)
        .into()
}

/// `GtkDropDown` over a slice of options.
pub fn dropdown<'a, T, M>(
    options: &'a [T],
    selected: Option<T>,
    on_select: impl Fn(T) -> M + 'a,
) -> Dropdown<'a, M>
where
    T: ToString + PartialEq + Clone + 'a,
    M: Clone + 'a,
{
    let m = Metrics::desktop();
    let current = selected.as_ref().map(|s| s.to_string()).unwrap_or_default();
    let trigger = container(
        row![
            text(current).wrapping(text::Wrapping::None),
            Icon::GoDown.view(m.icon_size)
        ]
        .spacing(m.space_s)
        .align_y(Vertical::Center),
    )
    .center_y(Length::Fixed(m.button_min_height - m.button_padding.y()));
    let items = options.iter().map(|o| {
        menu_item(o.to_string(), selected.as_ref() == Some(o), on_select(o.clone()))
    });
    Dropdown::custom(trigger, popover(items, 280.0))
}

impl<'a, M: 'a> Widget<M, Theme, iced::Renderer> for Dropdown<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.open_initially))
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.trigger), Tree::new(&self.popover)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.trigger.as_widget(), self.popover.as_widget()]);
    }

    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Shrink)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        layout::padded(limits, self.width, Length::Shrink, self.padding, |limits| {
            self.trigger.as_widget_mut().layout(&mut tree.children[0], renderer, limits)
        })
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &iced::Renderer,
        _: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        _: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
            && cursor.is_over(layout.bounds())
        {
            state.toggle();
            shell.capture_event();
            shell.invalidate_layout();
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        _: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _: &Rectangle,
        _: &iced::Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::None
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let open = tree.state.downcast_ref::<State>().open;
        let status = if open {
            button::Status::Pressed
        } else if cursor.is_over(layout.bounds()) {
            button::Status::Hovered
        } else {
            button::Status::Active
        };
        let st = match self.appearance {
            Appearance::Button => sb::default(theme, status),
            Appearance::Flat => sb::flat(theme, status),
            Appearance::Row => sb::list_row(theme, status),
        };
        if let Some(bg) = st.background {
            renderer.fill_quad(
                Quad {
                    bounds: layout.bounds(),
                    border: st.border,
                    shadow: Shadow::default(),
                    snap: false,
                },
                bg,
            );
        }
        let inner = renderer::Style { text_color: st.text_color };
        self.trigger.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &inner,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
        let _ = style;
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        _: &iced::Renderer,
        _: &Rectangle,
        translation: iced::Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        let Tree { state, children, .. } = tree;
        let state = state.downcast_mut::<State>();
        if !state.open {
            return None;
        }
        let b = layout.bounds();
        let anchor = Rectangle {
            x: b.x + translation.x,
            y: b.y + translation.y,
            ..b
        };
        Some(overlay::Element::new(Box::new(Popover {
            content: &mut self.popover,
            tree: &mut children[1],
            open: &mut state.open,
            appear: &mut state.appear,
            shown: &mut state.shown,
            anchor,
            align: self.align,
            min_width: self.popover_width,
            presentation: self.presentation,
        })))
    }
}

struct Popover<'a, 'b, M> {
    content: &'b mut Element<'a, M>,
    tree: &'b mut Tree,
    open: &'b mut bool,
    appear: &'b mut crate::animation::Tween,
    shown: &'b mut f32,
    anchor: Rectangle,
    align: Align,
    min_width: Option<f32>,
    presentation: Presentation,
}

const MARGIN: f32 = 6.0;
const GAP: f32 = 4.0;

impl<'a, 'b, M> overlay::Overlay<M, Theme, iced::Renderer> for Popover<'a, 'b, M> {
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        if self.presentation == Presentation::Page {
            // Fill the whole window.
            let limits = layout::Limits::new(bounds, bounds);
            let node = self.content.as_widget_mut().layout(self.tree, renderer, &limits);
            return layout::Node::with_children(bounds, vec![node]);
        }
        let below = self.anchor.y + self.anchor.height + GAP;
        let space_below = bounds.height - below - MARGIN;
        let space_above = self.anchor.y - GAP - MARGIN;
        let max_h = space_below.max(space_above).max(80.0);
        // Items fill their container, so the popover needs a defined width:
        // the trigger's width unless the dropdown asks for another.
        let w = self.min_width.unwrap_or(self.anchor.width).clamp(120.0, bounds.width - 2.0 * MARGIN);
        let limits = layout::Limits::new(Size::new(w, 0.0), Size::new(w, max_h));
        let node = self.content.as_widget_mut().layout(self.tree, renderer, &limits);
        let size = node.size();

        let x = match self.align {
            Align::Start => self.anchor.x,
            Align::End => self.anchor.x + self.anchor.width - size.width,
        }
        .min(bounds.width - size.width - MARGIN)
        .max(MARGIN);
        let y = if size.height <= space_below {
            below
        } else {
            (self.anchor.y - GAP - size.height).max(MARGIN)
        };
        layout::Node::with_children(size, vec![node]).move_to(Point::new(x, y))
    }

    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        // Opening: a popover rises 8px into place (a page slides in from the end edge),
        // 200ms ease-out-quad.
        let p = (*self.shown).clamp(0.0, 1.0);
        let d = if self.presentation == Presentation::Page {
            iced::Vector::new(layout.bounds().width * (1.0 - p), 0.0)
        } else {
            iced::Vector::new(0.0, -8.0 * (1.0 - p))
        };
        renderer.with_translation(d, |renderer| {
            self.content.as_widget().draw(
                self.tree,
                renderer,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                &Rectangle::with_size(Size::INFINITE),
            );
        });
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
    ) {
        if let Event::Window(iced::window::Event::RedrawRequested(now)) = event {
            self.appear.set_target(
                1.0,
                *now,
                crate::animation::Trigger::Popover.spec(crate::animation::Spec::STYLE),
            );
            *self.shown = self.appear.tick(*now);
            if self.appear.is_animating() {
                shell.request_redraw();
            }
        }
        let bounds = layout.bounds();
        let close = |open: &mut bool, shell: &mut Shell<'_, M>| {
            *open = false;
            shell.capture_event();
            shell.invalidate_layout();
            shell.request_redraw();
        };
        let page = self.presentation == Presentation::Page;
        match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::Escape),
                ..
            }) => return close(self.open, shell),
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if !page && !cursor.is_over(bounds) =>
            {
                return close(self.open, shell);
            }
            _ => {}
        }
        if page {
            // The back area in the page header closes it.
            let back = Rectangle {
                x: bounds.x,
                y: bounds.y,
                width: PAGE_BACK_WIDTH,
                height: PAGE_HEADER_HEIGHT,
            };
            if cursor.is_over(back) {
                match event {
                    Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                        shell.capture_event();
                        return;
                    }
                    Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                        return close(self.open, shell);
                    }
                    _ => {}
                }
            }
        }
        self.content.as_widget_mut().update(
            self.tree,
            event,
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            &Rectangle::with_size(Size::INFINITE),
        );
        // A completed click on an item (it captured the release) closes the menu.
        if matches!(event, Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)))
            && cursor.is_over(bounds)
            && shell.is_event_captured()
        {
            close(self.open, shell);
        }
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            self.tree,
            layout.children().next().unwrap(),
            cursor,
            &Rectangle::with_size(Size::INFINITE),
            renderer,
        )
    }
}

impl<'a, M: 'a> From<Dropdown<'a, M>> for Element<'a, M> {
    fn from(d: Dropdown<'a, M>) -> Self {
        iced::Element::new(d)
    }
}
