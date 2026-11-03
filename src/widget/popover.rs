//! `Popover`: a trigger that opens a floating surface anchored to it, the base of
//! [`dropdown`](super::dropdown), [`Menu`](super::Menu) and the other pickers.
//!
//! The closed state is the trigger (drawn as a button, a flat button, a list row...). Opening
//! shows the content on a `.menu` popover surface ([`popover`]) placed by the
//! [anchor primitive](crate::primitives::anchor): on a [`side`](Popover::side) of the trigger,
//! flipping to the other side when it does not fit and shifting to stay inside its bounds.
//! Click outside or Escape closes it.
//!
//! **Location aware.** A popover stays inside the **content area** it was opened from (the pane
//! beside the sidebar, between the bars: see [`content_area`](super::content_area)), not the
//! whole window, so it never slides under a sidebar or a bar that is not part of that content. A
//! trigger that is not in a content area (a header bar button, a sidebar item) uses the window.
//! [`bounds`](Popover::bounds) overrides it: [`PopoverBounds::Window`] opts out of the content
//! area. The default is the desktop config's `[popovers] bounds`.

use super::Element;
use crate::primitives::overlay::{Action, Input, Overlay as Rules};
use crate::style::{button as sb, container as sc};
use crate::{Metrics, Theme};
use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::keyboard::{self, key::Named};
use iced::widget::{button, column, container};
use iced::{Event, Length, Padding, Point, Rectangle, Shadow, Size, mouse};

/// What a popover is kept inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PopoverBounds {
    /// The content area the trigger is in (the window when it is in none).
    #[default]
    Content,
    /// The whole window, whatever the trigger is in.
    Window,
}

impl PopoverBounds {
    pub const ALL: [PopoverBounds; 2] = [PopoverBounds::Content, PopoverBounds::Window];

    /// The `[popovers] bounds` config value.
    pub fn key(self) -> &'static str {
        match self {
            Self::Content => "content",
            Self::Window => "window",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|b| b.key() == key)
    }
}

impl std::fmt::Display for PopoverBounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Content => "Content area",
            Self::Window => "Window",
        })
    }
}

/// The bounds in effect by default: the desktop config's `[popovers] bounds`.
pub fn popover_bounds() -> PopoverBounds {
    crate::config::current().popover_bounds
}

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
    /// A regular button joined to its neighbours (`.linked`), for the arrow of a split button.
    Linked(crate::style::button::Position),
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
    /// A modal dialog centered in the window, over a faded parent
    /// ([`scrim`](Popover::scrim), on by default); a press outside or Escape closes it.
    Dialog,
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
    Center,
    End,
}

pub struct Popover<'a, M> {
    trigger: Element<'a, M>,
    popover: Element<'a, M>,
    appearance: Appearance,
    align: Align,
    width: Length,
    padding: Padding,
    popover_width: Option<f32>,
    open_initially: bool,
    presentation: Presentation,
    disabled: bool,
    side: crate::primitives::anchor::Side,
    offset: f32,
    keep_open: bool,
    bounds: PopoverBounds,
    scrim: bool,
    close_on_outside: bool,
    window_radius: f32,
}

struct State {
    open: bool,
    /// 0 → 1 while the popover opens; restarted each time it opens.
    appear: crate::animation::Progress,
}

impl State {
    fn new(open: bool) -> Self {
        let start = if open && crate::animation::Trigger::Popover.animated() {
            0.0
        } else {
            1.0
        };
        Self {
            open,
            appear: crate::animation::Progress::new(start),
        }
    }
    fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            let start = if crate::animation::Trigger::Popover.animated() {
                0.0
            } else {
                1.0
            };
            self.appear = crate::animation::Progress::new(start);
        }
    }
}

impl<'a, M: 'a> Popover<'a, M> {
    /// A popover opened by `trigger` and showing `content`.
    pub fn new(trigger: impl Into<Element<'a, M>>, content: impl Into<Element<'a, M>>) -> Self {
        let m = Metrics::desktop();
        Self {
            trigger: trigger.into(),
            popover: content.into(),
            appearance: Appearance::Button,
            align: Align::Start,
            width: Length::Shrink,
            padding: m.button_padding,
            popover_width: None,
            open_initially: false,
            presentation: Presentation::Popover,
            disabled: false,
            side: crate::primitives::anchor::Side::Bottom,
            offset: GAP,
            keep_open: false,
            // The desktop config's default; `bounds(..)` overrides it for this popover.
            bounds: popover_bounds(),
            scrim: true,
            close_on_outside: true,
            window_radius: m.radius_window,
        }
    }
    /// The side of the trigger the popover opens on (default below). It flips to the opposite
    /// side when it does not fit; see [`anchor`](crate::primitives::anchor).
    pub fn side(mut self, side: crate::primitives::anchor::Side) -> Self {
        self.side = side;
        self
    }
    /// Gap between the trigger and the popover (default 4).
    pub fn offset(mut self, px: f32) -> Self {
        self.offset = px;
        self
    }
    /// What the popover is kept inside: the content area it was opened from
    /// ([`PopoverBounds::Content`]), or the whole window ([`PopoverBounds::Window`]). It starts
    /// at the desktop config's `[popovers] bounds`.
    pub fn bounds(mut self, bounds: PopoverBounds) -> Self {
        self.bounds = bounds;
        self
    }
    /// Keeps the popover open when something inside it is activated (multi-selection,
    /// calendar navigation). Click outside or Escape still close it.
    pub fn keep_open(mut self, keep: bool) -> Self {
        self.keep_open = keep;
        self
    }
    /// A disabled dropdown cannot be opened. Build its trigger dimmed (see
    /// [`dropdown_disabled`]); this only blocks the interaction.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// [`Presentation::Popover`] (default) or [`Presentation::Page`].
    /// The child of the popover that takes focus when it opens, by its widget id: the popover
    /// content becomes a [`FocusScope`](super::FocusScope) (it does not trap Tab) starting
    /// there. Without it focus stays where it was.
    pub fn initial_focus(mut self, id: impl Into<iced::widget::Id>) -> Self {
        let content = std::mem::replace(&mut self.popover, iced::widget::space().into());
        self.popover = super::FocusScope::new(content)
            .trap(false)
            .initial_focus(id)
            .into();
        self
    }
    /// Fades the window behind a [`Dialog`](Presentation::Dialog) (default on).
    pub fn scrim(mut self, on: bool) -> Self {
        self.scrim = on;
        self
    }
    /// Whether a press outside a [`Dialog`](Presentation::Dialog) closes it (default true).
    pub fn close_on_outside(mut self, on: bool) -> Self {
        self.close_on_outside = on;
        self
    }
    /// Rounds that fade to the window's corners (`window::State::radius`).
    pub fn window_radius(mut self, r: f32) -> Self {
        self.window_radius = r;
        self
    }
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

/// The popover surface wrapping `items`: scrolls past `max_height`.
pub fn popover<'a, M: 'a>(
    items: impl IntoIterator<Item = Element<'a, M>>,
    max_height: f32,
) -> Element<'a, M> {
    let m = Metrics::desktop();
    container(super::scrolled_window(
        column(items).spacing(2).width(Length::Fill),
    ))
    .padding(m.popover_padding)
    .max_height(max_height)
    .style(sc::popover)
    .into()
}

impl<'a, M: 'a> Widget<M, Theme, iced::Renderer> for Popover<'a, M> {
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

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::padded(limits, self.width, Length::Shrink, self.padding, |limits| {
            self.trigger
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, limits)
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
        if self.disabled {
            state.open = false;
            return;
        }
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
        if !self.disabled && cursor.is_over(layout.bounds()) {
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
            Appearance::Linked(pos) => sb::linked(pos)(theme, status),
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
        let inner = renderer::Style {
            text_color: st.text_color,
        };
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
        let Tree {
            state, children, ..
        } = tree;
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
        Some(overlay::Element::new(Box::new(PopoverOverlay {
            content: &mut self.popover,
            tree: &mut children[1],
            open: &mut state.open,
            appear: &mut state.appear,
            anchor,
            align: self.align,
            min_width: self.popover_width,
            presentation: self.presentation,
            side: self.side,
            offset: self.offset,
            keep_open: self.keep_open,
            window: Size::ZERO,
            scrim: self.scrim,
            close_on_outside: self.close_on_outside,
            window_radius: self.window_radius,
            // A full-window page ignores the content area.
            area: (self.bounds == PopoverBounds::Content
                && self.presentation == Presentation::Popover)
                .then(super::content_area::current)
                .flatten(),
        })))
    }
}

struct PopoverOverlay<'a, 'b, M> {
    content: &'b mut Element<'a, M>,
    tree: &'b mut Tree,
    open: &'b mut bool,
    appear: &'b mut crate::animation::Progress,
    anchor: Rectangle,
    align: Align,
    min_width: Option<f32>,
    presentation: Presentation,
    side: crate::primitives::anchor::Side,
    offset: f32,
    keep_open: bool,
    /// The window, as of the last layout.
    window: Size,
    scrim: bool,
    close_on_outside: bool,
    window_radius: f32,
    /// The content area it is kept inside, in window coordinates; `None`: the window.
    area: Option<Rectangle>,
}

/// Room kept around a dialog, and its width unless the content asks for another.
const DIALOG_MARGIN: f32 = 12.0;
const DIALOG_WIDTH: f32 = 360.0;
const MARGIN: f32 = 6.0;
const GAP: f32 = 4.0;

impl<M> PopoverOverlay<'_, '_, M> {
    /// The layer rules of this presentation: a dialog is modal and may fade the window; a popover
    /// and a page are neither, a page only closes by its back area or Escape.
    fn rules(&self) -> Rules {
        let dialog = self.presentation == Presentation::Dialog;
        Rules::default()
            .scrim(dialog && self.scrim)
            .modal(dialog)
            .close_on_outside(match self.presentation {
                Presentation::Page => false,
                Presentation::Dialog => self.close_on_outside,
                Presentation::Popover => true,
            })
    }
}

impl<'a, 'b, M> overlay::Overlay<M, Theme, iced::Renderer> for PopoverOverlay<'a, 'b, M> {
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        if self.presentation == Presentation::Page {
            // Fill the whole window.
            let limits = layout::Limits::new(bounds, bounds);
            let node = self
                .content
                .as_widget_mut()
                .layout(self.tree, renderer, &limits);
            return layout::Node::with_children(bounds, vec![node]);
        }
        if self.presentation == Presentation::Dialog {
            self.window = bounds;
            let max = crate::primitives::overlay::max_size(bounds, DIALOG_MARGIN);
            let w = self.min_width.unwrap_or(DIALOG_WIDTH).min(max.width);
            let limits = layout::Limits::new(Size::new(w, 0.0), Size::new(w, max.height));
            let node = self
                .content
                .as_widget_mut()
                .layout(self.tree, renderer, &limits);
            let at = crate::primitives::overlay::centered(
                Rectangle::with_size(bounds),
                node.size(),
                DIALOG_MARGIN,
            );
            return layout::Node::with_children(node.size(), vec![node]).move_to(at.position());
        }
        use crate::primitives::anchor::{self, Side};
        let placement = anchor::Placement {
            side: self.side,
            align: match self.align {
                Align::Start => anchor::Align::Start,
                Align::Center => anchor::Align::Center,
                Align::End => anchor::Align::End,
            },
            offset: self.offset,
            margin: MARGIN,
            ..anchor::Placement::default()
        };
        // Everything is measured inside the region the popover is kept in: the content area it
        // was opened from, or the window.
        let region = self.area.unwrap_or(Rectangle::with_size(bounds));
        // Size the popover to the larger side: the preferred one, or the one it may flip to.
        let (above, below) = (
            self.anchor.y - region.y - self.offset - MARGIN,
            region.y + region.height - (self.anchor.y + self.anchor.height) - self.offset - MARGIN,
        );
        let (left, right) = (
            self.anchor.x - region.x - self.offset - MARGIN,
            region.x + region.width - (self.anchor.x + self.anchor.width) - self.offset - MARGIN,
        );
        let sideways = matches!(self.side, Side::Left | Side::Right);
        let max_h = if sideways {
            region.height - 2.0 * MARGIN
        } else {
            below.max(above).max(80.0)
        };
        // Items fill their container, so the popover needs a defined width:
        // the trigger's width unless the popover asks for another.
        let w = self
            .min_width
            .unwrap_or(self.anchor.width)
            .clamp(120.0, (region.width - 2.0 * MARGIN).max(120.0));
        let w = if sideways {
            w.min(left.max(right).max(120.0))
        } else {
            w
        };
        let limits = layout::Limits::new(Size::new(w, 0.0), Size::new(w, max_h));
        let node = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);
        let size = node.size();
        let at = place_within(self.anchor, size, region, placement);
        layout::Node::with_children(size, vec![node]).move_to(at)
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
        let p = self.appear.shown.clamp(0.0, 1.0);
        let d = if self.presentation == Presentation::Page {
            iced::Vector::new(layout.bounds().width * (1.0 - p), 0.0)
        } else if self.presentation == Presentation::Dialog {
            iced::Vector::new(0.0, 24.0 * (1.0 - p))
        } else {
            iced::Vector::new(0.0, -8.0 * (1.0 - p))
        };
        let rules = self.rules();
        if rules.scrim_amount(p) > 0.0 {
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle::with_size(self.window),
                    border: iced::Border::default().rounded(self.window_radius),
                    shadow: Shadow::default(),
                    snap: false,
                },
                iced::Background::Color(super::motion::scrim(theme, rules.scrim_amount(p))),
            );
        }
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
        super::motion::advance(
            self.appear,
            event,
            1.0,
            crate::animation::Trigger::Popover.spec(crate::animation::Spec::STYLE),
            shell,
            false,
        );
        let bounds = layout.bounds();
        let close = |open: &mut bool, shell: &mut Shell<'_, M>| {
            *open = false;
            shell.capture_event();
            shell.invalidate_layout();
            shell.request_redraw();
        };
        let page = self.presentation == Presentation::Page;
        let rules = self.rules();
        let input = match event {
            Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(Named::Escape),
                ..
            }) => Some(Input::Escape),
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => Some(Input::Press {
                inside: cursor.is_over(bounds),
            }),
            _ => None,
        };
        match input.map(|i| rules.route(i)) {
            Some(Action::Close) => return close(self.open, shell),
            Some(Action::Swallow) => {
                shell.capture_event();
                return;
            }
            _ => {}
        }
        // A modal dialog keeps the pointer from reaching the window below.
        if rules.modal && matches!(event, Event::Mouse(_)) && !cursor.is_over(bounds) {
            shell.capture_event();
            return;
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
        if matches!(
            event,
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
        ) && cursor.is_over(bounds)
            && shell.is_event_captured()
            && !self.keep_open
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

impl<'a, M: 'a> From<Popover<'a, M>> for Element<'a, M> {
    fn from(d: Popover<'a, M>) -> Self {
        iced::Element::new(d)
    }
}

/// Where a popover of `size` goes against `anchor` inside `region` (all in window coordinates):
/// placement is worked out relative to the region, so it flips and shifts to stay inside it.
fn place_within(
    anchor: Rectangle,
    size: Size,
    region: Rectangle,
    placement: crate::primitives::anchor::Placement,
) -> Point {
    let rel = Rectangle {
        x: anchor.x - region.x,
        y: anchor.y - region.y,
        ..anchor
    };
    let at = crate::primitives::anchor::place(rel, size, region.size(), placement).position;
    Point::new(at.x + region.x, at.y + region.y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::anchor::{Align, Placement, Side};

    fn below() -> Placement {
        Placement {
            side: Side::Bottom,
            align: Align::Start,
            offset: 4.0,
            margin: 6.0,
            ..Placement::default()
        }
    }

    #[test]
    fn a_popover_stays_inside_the_content_area_not_the_window() {
        // A 800x600 window with a 200-wide sidebar: the content area is x 200..800.
        let content = Rectangle::new(Point::new(200.0, 0.0), Size::new(600.0, 600.0));
        let window = Rectangle::with_size(Size::new(800.0, 600.0));
        // A trigger at the content's left edge, popover wider than the room to its left.
        let anchor = Rectangle::new(Point::new(210.0, 100.0), Size::new(60.0, 32.0));
        let size = Size::new(300.0, 200.0);
        let in_content = place_within(anchor, size, content, below());
        assert!(
            in_content.x >= 200.0 + 6.0 - 0.01,
            "never under the sidebar: {in_content:?}"
        );
        // A trigger hanging off the right edge: the popover shifts back inside the content.
        let right_edge = Rectangle::new(Point::new(780.0, 100.0), Size::new(60.0, 32.0));
        let shifted = place_within(right_edge, size, content, below());
        assert!(
            shifted.x + size.width <= 800.0 - 6.0 + 0.01,
            "inside: {shifted:?}"
        );
        // The window as the region is the plain placement.
        let plain = place_within(anchor, size, window, below());
        assert_eq!(
            plain,
            crate::primitives::anchor::place(anchor, size, window.size(), below()).position
        );
    }

    #[test]
    fn the_region_is_where_it_flips() {
        // A short content area (y 0..300) over a tall window: a popover that fits below in the
        // window but not in the area flips above.
        let area = Rectangle::new(Point::new(0.0, 0.0), Size::new(400.0, 300.0));
        let window = Rectangle::with_size(Size::new(400.0, 800.0));
        let anchor = Rectangle::new(Point::new(20.0, 200.0), Size::new(80.0, 32.0));
        let size = Size::new(200.0, 150.0);
        let in_area = place_within(anchor, size, area, below());
        let in_window = place_within(anchor, size, window, below());
        assert!(
            in_area.y + size.height <= 300.0,
            "flipped above, inside the area: {in_area:?}"
        );
        assert!(
            in_window.y > anchor.y,
            "below the trigger in the window: {in_window:?}"
        );
    }

    #[test]
    fn bounds_have_config_keys() {
        assert_eq!(
            PopoverBounds::from_key("window"),
            Some(PopoverBounds::Window)
        );
        assert_eq!(
            PopoverBounds::from_key("content"),
            Some(PopoverBounds::Content)
        );
        assert_eq!(PopoverBounds::from_key("page"), None);
        assert_eq!(PopoverBounds::default(), PopoverBounds::Content);
    }
}
