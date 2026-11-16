//! `Popover`: a trigger that opens a floating surface anchored to it, the base of
//! [`dropdown`](crate::widgets::dropdown), [`Menu`](crate::widgets::Menu) and the other pickers.
//!
//! The closed state is the trigger (drawn as a button, a flat button, a list row...). Opening
//! shows the content on a `.menu` popover surface ([`popover`]) placed by the
//! [anchor primitive](crate::layouts::anchor): on a [`side`](Popover::side) of the trigger,
//! flipping to the other side when it does not fit and shifting to stay inside its bounds.
//! Click outside or Escape closes it.
//!
//! **Location aware.** A popover stays inside the **content area** it was opened from (the pane
//! beside the sidebar, between the bars: see [`content_area`](crate::layouts::content_area)), not the
//! whole window, so it never slides under a sidebar or a bar that is not part of that content. A
//! trigger that is not in a content area (a header bar button, a sidebar item) uses the window.
//! [`bounds`](Popover::bounds) overrides it: [`PopoverBounds::Window`] opts out of the content
//! area. The default is the desktop config's `[popovers] bounds`.

use crate::Element;
use crate::layouts::overlay::rule::{Action, Input, Overlay as Rules};
use crate::styles::{button as sb, container as sc};
use crate::{Metrics, Theme};
use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::keyboard::{self, key::Named};
use iced::widget::{button, column, container};
use iced::{Event, Length, Padding, Point, Rectangle, Shadow, Size, mouse};
use std::cell::Cell;

thread_local! {
    /// Set by an item's press handler while the click is being handled: the popover then stays
    /// open for that click only (a switch in a menu).
    static STAY_OPEN: Cell<bool> = const { Cell::new(false) };
}

/// Called from the handler of a press inside a popover: this click does not close it.
pub(crate) fn stay_open() {
    STAY_OPEN.with(|c| c.set(true));
}

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
    Linked(crate::styles::button::Position),
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

pub(crate) type ScrimShape =
    std::sync::Arc<dyn Fn(&Theme, Rectangle) -> (Rectangle, iced::border::Radius) + Send + Sync>;

/// Presentation-only overrides for a nested [`Popover`]. Values, events, disabled
/// state, focus, dismissal, and keep-open policy remain owned by its parent.
#[derive(Clone, Default)]
pub struct PopoverProps {
    pub side: Option<crate::layouts::anchor::Side>,
    pub offset: Option<f32>,
    pub bounds: Option<PopoverBounds>,
    pub appearance: Option<Appearance>,
    pub align: Option<Align>,
    pub padding: Option<Padding>,
    pub width: Option<Length>,
    pub popover_width: Option<f32>,
    pub fit_content: Option<bool>,
    pub window_radius: Option<f32>,
    /// Exact parent geometry, resolved with the current theme and window bounds at draw time.
    pub scrim_shape: Option<ScrimShape>,
    pub style: Option<crate::components::props::ButtonStyle>,
}
impl PopoverProps {
    pub fn side(mut self, value: impl Into<crate::layouts::anchor::Side>) -> Self {
        self.side = Some(value.into());
        self
    }
    pub fn offset(mut self, value: f32) -> Self {
        self.offset = Some(value);
        self
    }
    pub fn bounds(mut self, value: impl Into<PopoverBounds>) -> Self {
        self.bounds = Some(value.into());
        self
    }
    pub fn appearance(mut self, value: impl Into<Appearance>) -> Self {
        self.appearance = Some(value.into());
        self
    }
    pub fn align(mut self, value: impl Into<Align>) -> Self {
        self.align = Some(value.into());
        self
    }
    pub fn padding(mut self, value: impl Into<Padding>) -> Self {
        self.padding = Some(value.into());
        self
    }
    pub fn width(mut self, value: impl Into<Length>) -> Self {
        self.width = Some(value.into());
        self
    }
    pub fn popover_width(mut self, value: f32) -> Self {
        self.popover_width = Some(value);
        self
    }
    pub fn fit_content(mut self, value: impl Into<bool>) -> Self {
        self.fit_content = Some(value.into());
        self
    }
    /// Legacy explicit uniform parent corners. Does not discover window state.
    pub fn window_radius(mut self, value: f32) -> Self {
        self.window_radius = Some(value);
        self.scrim_shape = None;
        self
    }
    /// Exact background bounds and corners; share the actual parent's geometry resolver.
    pub fn scrim_shape(
        mut self,
        shape: impl Fn(&Theme, Rectangle) -> (Rectangle, iced::border::Radius) + Send + Sync + 'static,
    ) -> Self {
        self.scrim_shape = Some(std::sync::Arc::new(shape));
        self.window_radius = None;
        self
    }
    pub fn style(
        mut self,
        style: impl Fn(&Theme, button::Status) -> button::Style + Send + Sync + 'static,
    ) -> Self {
        self.style = Some(std::sync::Arc::new(style));
        self
    }

    pub fn apply<'a, M: 'a>(&self, mut widget: Popover<'a, M>) -> Popover<'a, M> {
        if let Some(v) = self.appearance {
            widget = widget.appearance(v);
        }
        if let Some(v) = self.side {
            widget = widget.side(v);
        }
        if let Some(v) = self.offset {
            widget = widget.offset(v);
        }
        if let Some(v) = self.bounds {
            widget = widget.bounds(v);
        }
        if let Some(v) = self.align {
            widget = widget.align(v);
        }
        if let Some(v) = self.padding {
            widget = widget.padding(v);
        }
        if let Some(v) = self.width {
            widget = widget.width(v);
        }
        if let Some(v) = self.popover_width {
            widget = widget.popover_width(v);
        }
        if let Some(v) = self.fit_content {
            widget.fit_content = v;
        }
        if let Some(v) = self.window_radius {
            widget = widget.window_radius(v);
        }
        if let Some(shape) = &self.scrim_shape {
            widget.scrim_shape = Some(shape.clone());
            widget.window_radius = None;
        }
        if let Some(style) = self.style.clone() {
            widget.style = Some(style);
        }
        widget
    }
}

/// Presentation overrides for the concrete container around a popover list.
#[derive(Clone, Default)]
pub struct PopoverSurfaceProps {
    pub container: crate::components::ContainerProps,
    pub spacing: Option<f32>,
}

impl PopoverSurfaceProps {
    pub fn container_props(mut self, props: crate::components::ContainerProps) -> Self {
        self.container = props;
        self
    }
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = Some(spacing);
        self
    }
}

pub struct Popover<'a, M> {
    context_trigger: bool,
    menu_navigation: bool,
    style: Option<crate::components::props::ButtonStyle>,
    trigger: Element<'a, M>,
    popover: Element<'a, M>,
    appearance: Appearance,
    align: Align,
    width: Length,
    padding: Padding,
    popover_width: Option<f32>,
    fit_content: bool,
    open_initially: bool,
    presentation: Presentation,
    disabled: bool,
    side: crate::layouts::anchor::Side,
    offset: f32,
    keep_open: bool,
    bounds: PopoverBounds,
    scrim: bool,
    close_on_outside: bool,
    window_radius: Option<f32>,
    scrim_shape: Option<ScrimShape>,
}

fn resolve_scrim_shape(
    shape: Option<&ScrimShape>,
    radius: Option<f32>,
    theme: &Theme,
    bounds: Rectangle,
) -> Option<(Rectangle, iced::border::Radius)> {
    shape
        .map(|shape| shape(theme, bounds))
        .or_else(|| radius.map(|r| (bounds, r.into())))
}

fn context_key(event: &Event) -> bool {
    matches!(
        event,
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(Named::ContextMenu),
            ..
        })
    ) || matches!(event, Event::Keyboard(keyboard::Event::KeyPressed { key: keyboard::Key::Named(Named::F10), modifiers, .. }) if modifiers.shift() && !modifiers.control() && !modifiers.alt() && !modifiers.logo())
}

#[derive(Default)]
struct MenuRows(Vec<(Rectangle, bool, bool)>, iced::Vector);
impl iced::advanced::widget::Operation for MenuRows {
    fn scrollable(
        &mut self,
        _: Option<&iced::advanced::widget::Id>,
        _: Rectangle,
        _: Rectangle,
        translation: iced::Vector,
        _: &mut dyn iced::advanced::widget::operation::Scrollable,
    ) {
        self.1 = translation;
    }
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn iced::advanced::widget::Operation)) {
        operate(self);
    }
    fn container(&mut self, id: Option<&iced::advanced::widget::Id>, bounds: Rectangle) {
        let is = |name| id == Some(&iced::advanced::widget::Id::new(name));
        let submenu = is("iad-menu-submenu");
        let back = is("iad-menu-back");
        if is("iad-menu-row") || submenu || back {
            self.0.push((
                Rectangle::new(bounds.position() - self.1, bounds.size()),
                submenu,
                back,
            ));
        }
    }
}

struct RevealMenuRow(Rectangle);
impl iced::advanced::widget::Operation for RevealMenuRow {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn iced::advanced::widget::Operation)) {
        operate(self);
    }
    fn scrollable(
        &mut self,
        _: Option<&iced::advanced::widget::Id>,
        bounds: Rectangle,
        content: Rectangle,
        _: iced::Vector,
        state: &mut dyn iced::advanced::widget::operation::Scrollable,
    ) {
        let dy = if self.0.y < bounds.y {
            self.0.y - bounds.y
        } else {
            (self.0.y + self.0.height - bounds.y - bounds.height).max(0.0)
        };
        state.scroll_by(
            iced::advanced::widget::operation::scrollable::AbsoluteOffset { x: 0.0, y: dy },
            bounds,
            content,
        );
    }
}

#[derive(Default)]
struct TargetFocus {
    focused: bool,
    count: usize,
}
impl iced::advanced::widget::Operation for TargetFocus {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn iced::advanced::widget::Operation)) {
        operate(self);
    }
    fn focusable(
        &mut self,
        _: Option<&iced::advanced::widget::Id>,
        _: Rectangle,
        state: &mut dyn iced::advanced::widget::operation::Focusable,
    ) {
        self.focused |= state.is_focused();
        self.count += 1;
    }
}
impl iced::advanced::widget::operation::Focusable for State {
    fn is_focused(&self) -> bool {
        self.focused
    }
    fn focus(&mut self) {
        self.focused = true;
    }
    fn unfocus(&mut self) {
        self.focused = false;
    }
}

struct State {
    focused: bool,
    selected: Option<usize>,
    keyboard_pointer: Option<Point>,
    pointer: Option<Point>,
    open: bool,
    /// The content was on screen since it was last reset: closing it puts it back as new, so a
    /// menu that was left on a submenu opens on its first page again.
    shown: bool,
    /// The side and alignment the popover was last placed with: kept while they still fit (CSS
    /// anchor positioning's "last successful option"), and forgotten when it closes.
    placed: Option<(crate::layouts::anchor::Side, crate::layouts::anchor::Align)>,
    /// 0 → 1 while the popover opens; restarted each time it opens.
    appear: crate::layouts::animation::Progress,
}

impl State {
    fn new(open: bool) -> Self {
        let start = if open && crate::layouts::animation::Trigger::Popover.animated() {
            0.0
        } else {
            1.0
        };
        Self {
            focused: false,
            selected: None,
            keyboard_pointer: None,
            pointer: None,
            open,
            shown: open,
            placed: None,
            appear: crate::layouts::animation::Progress::new(start),
        }
    }
    fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.selected = None;
            self.keyboard_pointer = None;
            let start = if crate::layouts::animation::Trigger::Popover.animated() {
                0.0
            } else {
                1.0
            };
            self.appear = crate::layouts::animation::Progress::new(start);
        }
    }
}

impl<'a, M: 'a> Popover<'a, M> {
    /// A popover opened by `trigger` and showing `content`.
    pub fn new(trigger: impl Into<Element<'a, M>>, content: impl Into<Element<'a, M>>) -> Self {
        let m = Metrics::desktop();
        Self {
            context_trigger: false,
            menu_navigation: false,
            style: None,
            trigger: trigger.into(),
            popover: content.into(),
            appearance: Appearance::Button,
            align: Align::Start,
            width: Length::Shrink,
            padding: m.button_padding,
            popover_width: None,
            fit_content: false,
            open_initially: false,
            presentation: Presentation::Popover,
            disabled: false,
            side: crate::layouts::anchor::Side::Bottom,
            offset: GAP,
            keep_open: false,
            // The desktop config's default; `bounds(..)` overrides it for this popover.
            bounds: popover_bounds(),
            scrim: true,
            close_on_outside: true,
            window_radius: None,
            scrim_shape: None,
        }
    }
    pub(crate) fn context_trigger(mut self) -> Self {
        self.context_trigger = true;
        self.menu_navigation = true;
        self
    }

    pub(crate) fn menu_navigation(mut self) -> Self {
        self.menu_navigation = true;
        self
    }

    pub fn props(self, props: PopoverProps) -> Self {
        props.apply(self)
    }
    pub fn style(
        mut self,
        style: impl Fn(&Theme, button::Status) -> button::Style + Send + Sync + 'static,
    ) -> Self {
        self.style = Some(std::sync::Arc::new(style));
        self
    }
    /// The side of the trigger the popover opens on (default below). It flips to the opposite
    /// side when it does not fit; see [`anchor`](crate::layouts::anchor).
    pub fn side(mut self, side: crate::layouts::anchor::Side) -> Self {
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
    /// [`dropdown_disabled`](crate::widgets::dropdown_disabled)); this only blocks the interaction.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// [`Presentation::Popover`] (default) or [`Presentation::Page`].
    /// The child of the popover that takes focus when it opens, by its widget id: the popover
    /// content becomes a [`FocusScope`](crate::binds::FocusScope) (it does not trap Tab) starting
    /// there. Without it focus stays where it was.
    pub fn initial_focus(mut self, id: impl Into<iced::widget::Id>) -> Self {
        let content = std::mem::replace(&mut self.popover, iced::widget::space().into());
        self.popover = crate::binds::FocusScope::new(content)
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
    /// Legacy explicit uniform parent corners (`window::State::radius`).
    /// Without this or [`Self::scrim_shape`], no scrim is painted: arbitrary
    /// windows have no known silhouette. Modal input and dismissal are unchanged.
    pub fn window_radius(mut self, r: f32) -> Self {
        self.window_radius = Some(r);
        self.scrim_shape = None;
        self
    }
    /// Exact geometry of the parent being dimmed, resolved at draw time with the
    /// current theme and window bounds. Return its actual bounds and four corners;
    /// reuse the parent's resolver rather than guessing floating-window metrics.
    pub fn scrim_shape(
        mut self,
        shape: impl Fn(&Theme, Rectangle) -> (Rectangle, iced::border::Radius) + Send + Sync + 'static,
    ) -> Self {
        self.scrim_shape = Some(std::sync::Arc::new(shape));
        self.window_radius = None;
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
    /// Sizes the popover to its content (and the surface's padding), with no minimum width: for a
    /// small panel such as a vertical slider. Content that fills its width then has nothing to
    /// fill, so give it a width of its own.
    pub fn fit_content(mut self) -> Self {
        self.fit_content = true;
        self
    }
}

/// The popover surface wrapping `items`: scrolls past `max_height`. The items are `space_xxs`
/// apart and the surface pads them by `popover_padding`; [`popover_spaced`] sets both.
pub fn popover<'a, M: 'a>(
    items: impl IntoIterator<Item = Element<'a, M>>,
    max_height: f32,
) -> Element<'a, M> {
    let m = Metrics::desktop();
    popover_spaced(items, max_height, m.space_xxs, m.popover_padding)
}

/// [`popover`] with its own gap between the items and padding around them.
pub fn popover_spaced<'a, M: 'a>(
    items: impl IntoIterator<Item = Element<'a, M>>,
    max_height: f32,
    spacing: f32,
    padding: impl Into<Padding>,
) -> Element<'a, M> {
    popover_spaced_with_props(
        items,
        max_height,
        spacing,
        padding,
        PopoverSurfaceProps::default(),
    )
}

/// [`popover_spaced`] with overrides applied to its existing surface container.
pub fn popover_spaced_with_props<'a, M: 'a>(
    items: impl IntoIterator<Item = Element<'a, M>>,
    max_height: f32,
    spacing: f32,
    padding: impl Into<Padding>,
    props: PopoverSurfaceProps,
) -> Element<'a, M> {
    crate::components::framed(
        props.container.apply(
            container(crate::layouts::scrolled_window(
                column(items)
                    .spacing(props.spacing.unwrap_or(spacing))
                    .width(Length::Fill),
            ))
            .padding(padding)
            .max_height(max_height)
            .style(sc::popover),
        ),
        "popover",
    )
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

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        if self.context_trigger || self.menu_navigation {
            let mut target = TargetFocus::default();
            self.trigger.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                &mut target,
            );
            if target.count == 0 {
                operation.focusable(None, layout.bounds(), tree.state.downcast_mut::<State>());
            }
        }
        self.trigger.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }

    fn size(&self) -> Size<Length> {
        if self.context_trigger {
            return self.trigger.as_widget().size();
        }
        Size::new(self.width, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = self.size();
        layout::padded(limits, size.width, size.height, self.padding, |limits| {
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
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        if self.context_trigger {
            let mut focus = TargetFocus::default();
            self.trigger.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                &mut focus,
            );
            let state = tree.state.downcast_mut::<State>();
            let inside = cursor.is_over(layout.bounds()) && cursor.is_over(*viewport);
            if matches!(
                event,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            ) {
                state.focused = inside;
            }
            let pointer = matches!(
                event,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right))
            ) && inside;
            let keyboard =
                ((state.focused && focus.count == 0) || focus.focused) && context_key(event);
            // Descendant context targets own their gestures before an enclosing target.
            self.trigger.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout.children().next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
            if shell.is_event_captured() {
                return;
            }
            if self.disabled {
                state.open = false;
            }
            if !self.disabled && (pointer || keyboard) {
                state.focused = true;
                state.pointer = if pointer { cursor.position() } else { None };
                if !state.open {
                    state.toggle();
                }
                shell.capture_event();
                shell.invalidate_layout();
                shell.request_redraw();
                return;
            }
            return;
        }
        let state = tree.state.downcast_mut::<State>();
        if self.disabled {
            state.open = false;
            return;
        }
        if self.menu_navigation {
            if matches!(
                event,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            ) {
                state.focused = cursor.is_over(layout.bounds()) && cursor.is_over(*viewport);
            }
            if state.focused
                && matches!(
                    event,
                    Event::Keyboard(keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(Named::Enter | Named::Space | Named::ArrowDown),
                        ..
                    })
                )
            {
                if !state.open {
                    state.toggle();
                }
                shell.capture_event();
                shell.invalidate_layout();
                shell.request_redraw();
                return;
            }
        }
        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event
            && cursor.is_over(layout.bounds())
            && cursor.is_over(*viewport)
        {
            state.toggle();
            shell.capture_event();
            shell.invalidate_layout();
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if self.context_trigger {
            return self.trigger.as_widget().mouse_interaction(
                &tree.children[0],
                layout.children().next().unwrap(),
                cursor,
                viewport,
                renderer,
            );
        }
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
        if self.context_trigger {
            self.trigger.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                viewport,
            );
            return;
        }
        let open = tree.state.downcast_ref::<State>().open;
        let status = if open {
            button::Status::Pressed
        } else if cursor.is_over(layout.bounds()) {
            button::Status::Hovered
        } else {
            button::Status::Active
        };
        let st = self.style.as_ref().map_or_else(
            || match self.appearance {
                Appearance::Button => sb::default(theme, status),
                Appearance::Flat => sb::flat(theme, status),
                Appearance::Row => sb::list_row(theme, status),
                Appearance::Linked(pos) => sb::linked(pos)(theme, status),
            },
            |style| style(theme, status),
        );
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
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: iced::Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        let Tree {
            state, children, ..
        } = tree;
        let state = state.downcast_mut::<State>();
        if !state.open {
            // Closed: forget where the content was left (the page of a submenu, a scroll), so
            // it opens as new, on its preferred side.
            state.placed = None;
            if state.shown {
                state.shown = false;
                children[1] = Tree::new(self.popover.as_widget());
            }
            return self.trigger.as_widget_mut().overlay(
                &mut children[0],
                layout.children().next().unwrap(),
                renderer,
                viewport,
                translation,
            );
        }
        state.shown = true;
        let b = layout.bounds();
        let anchor = if self.context_trigger {
            let p = state.pointer.unwrap_or(Point::new(b.x, b.y + b.height));
            Rectangle {
                x: p.x + translation.x,
                y: p.y + translation.y,
                width: 0.0,
                height: 0.0,
            }
        } else {
            Rectangle {
                x: b.x + translation.x,
                y: b.y + translation.y,
                ..b
            }
        };
        Some(overlay::Element::new(Box::new(PopoverOverlay {
            content: &mut self.popover,
            tree: &mut children[1],
            open: &mut state.open,
            selected: &mut state.selected,
            keyboard_pointer: &mut state.keyboard_pointer,
            menu_navigation: self.menu_navigation,
            appear: &mut state.appear,
            placed: &mut state.placed,
            anchor,
            align: self.align,
            min_width: self.popover_width,
            fit_content: self.fit_content,
            presentation: self.presentation,
            side: self.side,
            offset: self.offset,
            keep_open: self.keep_open,
            window: Size::ZERO,
            scrim: self.scrim,
            close_on_outside: self.close_on_outside,
            window_radius: self.window_radius,
            scrim_shape: self.scrim_shape.clone(),
            // A full-window page ignores the content area.
            area: (self.bounds == PopoverBounds::Content
                && self.presentation == Presentation::Popover)
                .then(crate::layouts::content_area::current)
                .flatten(),
        })))
    }
}

struct PopoverOverlay<'a, 'b, M> {
    content: &'b mut Element<'a, M>,
    tree: &'b mut Tree,
    open: &'b mut bool,
    selected: &'b mut Option<usize>,
    keyboard_pointer: &'b mut Option<Point>,
    menu_navigation: bool,
    appear: &'b mut crate::layouts::animation::Progress,
    placed: &'b mut Option<(crate::layouts::anchor::Side, crate::layouts::anchor::Align)>,
    anchor: Rectangle,
    align: Align,
    min_width: Option<f32>,
    fit_content: bool,
    presentation: Presentation,
    side: crate::layouts::anchor::Side,
    offset: f32,
    keep_open: bool,
    /// The window, as of the last layout.
    window: Size,
    scrim: bool,
    close_on_outside: bool,
    window_radius: Option<f32>,
    scrim_shape: Option<ScrimShape>,
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
            let max = crate::layouts::overlay::rule::max_size(bounds, DIALOG_MARGIN);
            let w = self.min_width.unwrap_or(DIALOG_WIDTH).min(max.width);
            let limits = layout::Limits::new(Size::new(w, 0.0), Size::new(w, max.height));
            let node = self
                .content
                .as_widget_mut()
                .layout(self.tree, renderer, &limits);
            let at = crate::layouts::overlay::rule::centered(
                Rectangle::with_size(bounds),
                node.size(),
                DIALOG_MARGIN,
            );
            return layout::Node::with_children(node.size(), vec![node]).move_to(at.position());
        }
        use crate::layouts::anchor::{self, Side};
        let placement = anchor::Placement {
            side: self.side,
            align: match self.align {
                Align::Start => anchor::Align::Start,
                Align::Center => anchor::Align::Center,
                Align::End => anchor::Align::End,
            },
            offset: self.offset,
            margin: MARGIN,
            last: *self.placed,
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
            (region.height - 2.0 * MARGIN).max(0.0)
        } else {
            below
                .max(above)
                .max(0.0)
                .min((region.height - 2.0 * MARGIN).max(0.0))
        };
        // Items fill their container, so the popover needs a defined width:
        // the trigger's width unless the popover asks for another.
        let w = self
            .min_width
            .unwrap_or(self.anchor.width)
            .max(0.0)
            .min((region.width - 2.0 * MARGIN).max(0.0));
        let w = if sideways {
            w.min(left.max(right).max(0.0))
        } else {
            w
        };
        let limits = if self.fit_content {
            let max_w = (region.width - 2.0 * MARGIN).max(0.0);
            layout::Limits::new(Size::ZERO, Size::new(max_w, max_h))
        } else {
            layout::Limits::new(Size::new(w, 0.0), Size::new(w, max_h))
        };
        let node = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &limits);
        let size = node.size();
        let placed = place_within_choice(self.anchor, size, region, placement);
        *self.placed = Some(placed.1);
        layout::Node::with_children(size, vec![node]).move_to(placed.0)
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
        if rules.scrim_amount(p) > 0.0
            && let Some((bounds, radius)) = resolve_scrim_shape(
                self.scrim_shape.as_ref(),
                self.window_radius,
                theme,
                Rectangle::with_size(self.window),
            )
        {
            renderer.fill_quad(
                Quad {
                    bounds,
                    border: iced::Border::default().rounded(radius),
                    shadow: Shadow::default(),
                    snap: false,
                },
                iced::Background::Color(crate::layouts::motion::scrim(
                    theme,
                    rules.scrim_amount(p),
                )),
            );
        }
        let cursor = self
            .keyboard_pointer
            .map(mouse::Cursor::Available)
            .unwrap_or(cursor);
        renderer.with_translation(d, |renderer| {
            self.content.as_widget().draw(
                self.tree,
                renderer,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                &layout.bounds(),
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
        crate::layouts::motion::advance(
            self.appear,
            event,
            1.0,
            crate::layouts::animation::Trigger::Popover
                .spec(crate::layouts::animation::Spec::STYLE),
            shell,
            false,
        );
        let bounds = layout.bounds();
        if matches!(event, Event::Mouse(mouse::Event::CursorMoved { .. })) {
            *self.keyboard_pointer = None;
            *self.selected = None;
        }
        if self.menu_navigation {
            if let Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(key),
                ..
            }) = event
            {
                let mut rows = MenuRows::default();
                self.content.as_widget_mut().operate(
                    self.tree,
                    layout.children().next().unwrap(),
                    renderer,
                    &mut rows,
                );
                if !rows.0.is_empty() {
                    let n = rows.0.len();
                    let next = match key {
                        Named::ArrowDown => Some(self.selected.map_or(0, |i| (i + 1) % n)),
                        Named::ArrowUp => Some(self.selected.map_or(n - 1, |i| (i + n - 1) % n)),
                        Named::Home => Some(0),
                        Named::End => Some(n - 1),
                        _ => None,
                    };
                    if let Some(i) = next {
                        *self.selected = Some(i);
                        self.content.as_widget_mut().operate(
                            self.tree,
                            layout.children().next().unwrap(),
                            renderer,
                            &mut RevealMenuRow(rows.0[i].0),
                        );
                        rows = MenuRows::default();
                        self.content.as_widget_mut().operate(
                            self.tree,
                            layout.children().next().unwrap(),
                            renderer,
                            &mut rows,
                        );
                        let at = rows.0[i].0.center();
                        *self.keyboard_pointer = Some(at);
                        self.content.as_widget_mut().update(
                            self.tree,
                            &Event::Mouse(mouse::Event::CursorMoved { position: at }),
                            layout.children().next().unwrap(),
                            mouse::Cursor::Available(at),
                            renderer,
                            clipboard,
                            shell,
                            &bounds,
                        );
                        shell.capture_event();
                        shell.request_redraw();
                        return;
                    }
                    if matches!(
                        key,
                        Named::Enter | Named::Space | Named::ArrowRight | Named::ArrowLeft
                    ) {
                        let index = match key {
                            Named::ArrowLeft => rows.0.iter().position(|row| row.2),
                            Named::ArrowRight => self.selected.filter(|i| *i < n && rows.0[*i].1),
                            _ => Some(self.selected.unwrap_or(0)),
                        };
                        if let Some(i) = index.filter(|i| *i < n) {
                            let at = rows.0[i].0.center();
                            for event in [
                                mouse::Event::ButtonPressed(mouse::Button::Left),
                                mouse::Event::ButtonReleased(mouse::Button::Left),
                            ] {
                                // Keep each synthetic pointer event's capture status separate:
                                // a submenu captures the press, but deliberately not the release.
                                let mut messages = Vec::new();
                                let mut input_shell = Shell::new(&mut messages);
                                self.update(
                                    &Event::Mouse(event),
                                    layout,
                                    mouse::Cursor::Available(at),
                                    renderer,
                                    clipboard,
                                    &mut input_shell,
                                );
                                shell.merge(input_shell, |message| message);
                            }
                            *self.selected = None;
                            shell.capture_event();
                            return;
                        }
                    }
                }
            }
        }
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
            Event::Mouse(mouse::Event::ButtonPressed(_)) => Some(Input::Press {
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
            &bounds,
        );
        // An item that asked to stay (see `stay_open`) keeps the menu up for this click.
        let stay = STAY_OPEN.with(|c| c.replace(false));
        // A completed click on an item (it captured the release) closes the menu.
        if !stay
            && matches!(
                event,
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
            )
            && cursor.is_over(bounds)
            && shell.is_event_captured()
            && (!self.menu_navigation || !shell.is_empty())
            && !self.keep_open
        {
            close(self.open, shell);
        }
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content.as_widget_mut().operate(
            self.tree,
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
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
    placement: crate::layouts::anchor::Placement,
) -> Point {
    place_within_choice(anchor, size, region, placement).0
}

/// [`place_within`], and the side and alignment it used, to remember for the next placement.
fn place_within_choice(
    anchor: Rectangle,
    size: Size,
    region: Rectangle,
    placement: crate::layouts::anchor::Placement,
) -> (
    Point,
    (crate::layouts::anchor::Side, crate::layouts::anchor::Align),
) {
    let rel = Rectangle {
        x: anchor.x - region.x,
        y: anchor.y - region.y,
        ..anchor
    };
    let placed = crate::layouts::anchor::place(rel, size, region.size(), placement);
    let at = placed.position;
    // An anchor may be outside the current viewport after a resize or scroll.
    // The anchor primitive flips against it; clamp both axes to the final region.
    (
        Point::new(
            (at.x + region.x).clamp(region.x, region.x + (region.width - size.width).max(0.0)),
            (at.y + region.y).clamp(region.y, region.y + (region.height - size.height).max(0.0)),
        ),
        placed.choice(),
    )
}

#[cfg(test)]
mod context_event_tests {
    use super::*;
    use crate::widgets::menu::{Menu, MenuItem, pages_content, popover_content};
    use iced::advanced::renderer::Headless;

    fn renderer() -> iced::Renderer {
        iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap()
    }
    fn key(key: Named, modifiers: keyboard::Modifiers) -> Event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(key),
            modified_key: keyboard::Key::Named(key),
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        })
    }
    fn send(
        p: &mut Popover<'_, u8>,
        tree: &mut Tree,
        node: &layout::Node,
        renderer: &iced::Renderer,
        event: Event,
        at: Point,
    ) -> (bool, Vec<u8>) {
        let mut messages = Vec::new();
        let mut shell = Shell::new(&mut messages);
        p.update(
            tree,
            &event,
            Layout::new(node),
            mouse::Cursor::Available(at),
            renderer,
            &mut iced::advanced::clipboard::Null,
            &mut shell,
            &Rectangle::with_size(Size::new(500.0, 300.0)),
        );
        let captured = shell.is_event_captured();
        (captured, messages)
    }
    fn overlay_event(
        p: &mut Popover<'_, u8>,
        tree: &mut Tree,
        node: &layout::Node,
        renderer: &iced::Renderer,
        event: Event,
    ) -> Vec<u8> {
        let viewport = Rectangle::with_size(Size::new(500.0, 300.0));
        let mut overlay = p
            .overlay(
                tree,
                Layout::new(node),
                renderer,
                &viewport,
                iced::Vector::ZERO,
            )
            .unwrap();
        let node = overlay.as_overlay_mut().layout(renderer, viewport.size());
        let mut messages = Vec::new();
        overlay.as_overlay_mut().update(
            &event,
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            renderer,
            &mut iced::advanced::clipboard::Null,
            &mut Shell::new(&mut messages),
        );
        messages
    }
    fn settle(
        p: &mut Popover<'_, u8>,
        tree: &mut Tree,
        node: &layout::Node,
        renderer: &iced::Renderer,
    ) {
        let now = std::time::Instant::now();
        for time in [now, now + std::time::Duration::from_secs(1)] {
            overlay_event(
                p,
                tree,
                node,
                renderer,
                Event::Window(iced::window::Event::RedrawRequested(time)),
            );
        }
    }

    #[test]
    fn context_target_passes_left_click_and_requires_keyboard_focus() {
        let renderer = renderer();
        let mut p = Popover::new(button("Target").on_press(9), iced::widget::space())
            .padding(0)
            .context_trigger();
        let mut tree = Tree::new(&p as &dyn Widget<u8, Theme, iced::Renderer>);
        let node = p.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(500.0, 300.0)),
        );
        let at = node.bounds().center();
        assert!(
            !send(
                &mut p,
                &mut tree,
                &node,
                &renderer,
                key(Named::ContextMenu, keyboard::Modifiers::empty()),
                at
            )
            .0
        );
        send(
            &mut p,
            &mut tree,
            &node,
            &renderer,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            at,
        );
        assert_eq!(
            send(
                &mut p,
                &mut tree,
                &node,
                &renderer,
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                at
            )
            .1,
            vec![9]
        );
        assert!(!tree.state.downcast_ref::<State>().open);
        assert!(
            send(
                &mut p,
                &mut tree,
                &node,
                &renderer,
                key(Named::F10, keyboard::Modifiers::SHIFT),
                at
            )
            .0
        );
        assert!(tree.state.downcast_ref::<State>().open);
        assert!(tree.state.downcast_ref::<State>().pointer.is_none());
        overlay_event(
            &mut p,
            &mut tree,
            &node,
            &renderer,
            key(Named::Escape, keyboard::Modifiers::empty()),
        );
        assert!(!tree.state.downcast_ref::<State>().open);
    }
    #[test]
    fn focused_text_input_owns_keyboard_invocation_and_nested_target_owns_right_click() {
        let renderer = renderer();
        let input = iced::widget::text_input("", "text").on_input(|_| 0);
        let mut p = Popover::new(input, iced::widget::space())
            .padding(0)
            .context_trigger();
        let mut tree = Tree::new(&p as &dyn Widget<u8, Theme, iced::Renderer>);
        let node = p.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(500.0, 300.0)),
        );
        struct Focus;
        impl iced::advanced::widget::Operation for Focus {
            fn traverse(&mut self, f: &mut dyn FnMut(&mut dyn iced::advanced::widget::Operation)) {
                f(self);
            }
            fn focusable(
                &mut self,
                _: Option<&iced::advanced::widget::Id>,
                _: Rectangle,
                state: &mut dyn iced::advanced::widget::operation::Focusable,
            ) {
                state.focus();
            }
        }
        p.operate(&mut tree, Layout::new(&node), &renderer, &mut Focus);
        assert!(
            !tree.state.downcast_ref::<State>().focused,
            "focus belongs to the input, not the wrapper"
        );
        assert!(
            send(
                &mut p,
                &mut tree,
                &node,
                &renderer,
                key(Named::ContextMenu, keyboard::Modifiers::empty()),
                Point::ORIGIN
            )
            .0
        );
        assert!(tree.state.downcast_ref::<State>().open);

        let inner = Popover::new(
            iced::widget::space().width(100).height(40),
            iced::widget::space(),
        )
        .padding(0)
        .context_trigger();
        let mut outer = Popover::new(inner, iced::widget::space())
            .padding(0)
            .context_trigger();
        let mut tree = Tree::new(&outer as &dyn Widget<u8, Theme, iced::Renderer>);
        let node = outer.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(500.0, 300.0)),
        );
        assert!(
            send(
                &mut outer,
                &mut tree,
                &node,
                &renderer,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)),
                Point::new(20.0, 20.0)
            )
            .0
        );
        assert!(!tree.state.downcast_ref::<State>().open);
        assert!(tree.children[0].state.downcast_ref::<State>().open);
    }

    #[test]
    fn keyboard_end_reveals_and_activates_a_scrolled_row() {
        let renderer = renderer();
        let menu =
            Menu::new("Long").section((0..30).map(|i| MenuItem::action(format!("Command {i}"), i)));
        let mut p = Popover::new(
            iced::widget::space().width(100).height(30),
            popover_content(&menu, 100.0),
        )
        .padding(0)
        .popover_width(260.0)
        .context_trigger()
        .open_initially(true);
        let mut tree = Tree::new(&p as &dyn Widget<u8, Theme, iced::Renderer>);
        let node = p.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(500.0, 300.0)),
        );
        overlay_event(
            &mut p,
            &mut tree,
            &node,
            &renderer,
            key(Named::End, keyboard::Modifiers::empty()),
        );
        let pointer = tree.state.downcast_ref::<State>().keyboard_pointer.unwrap();
        assert!(
            pointer.y < 150.0,
            "selected row should scroll into view: {pointer:?}"
        );
        assert_eq!(
            overlay_event(
                &mut p,
                &mut tree,
                &node,
                &renderer,
                key(Named::Enter, keyboard::Modifiers::empty())
            ),
            vec![29]
        );
        assert!(!tree.state.downcast_ref::<State>().open);
    }

    #[test]
    fn text_trigger_builds_a_real_menu_overlay() {
        let renderer = renderer();
        let menu = Menu::new("File").section([MenuItem::action("Open", 2_u8)]);
        let mut element: Element<'_, u8> =
            crate::widgets::menu::menu_button(crate::Icon::OpenMenu, menu)
                .text_trigger("File")
                .props(crate::widgets::menu::MenuProps {
                    label: crate::components::TextProps::default().width(80),
                    ..Default::default()
                })
                .into();
        let mut tree = Tree::new(&element);
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(500.0, 300.0)),
        );
        assert_eq!(
            node.size().width,
            80.0,
            "typed trigger label props reach the native text"
        );
        let viewport = Rectangle::with_size(Size::new(500.0, 300.0));
        element.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Layout::new(&node),
            mouse::Cursor::Available(node.bounds().center()),
            &renderer,
            &mut iced::advanced::clipboard::Null,
            &mut Shell::new(&mut vec![]),
            &viewport,
        );
        let mut overlay = element
            .as_widget_mut()
            .overlay(
                &mut tree,
                Layout::new(&node),
                &renderer,
                &viewport,
                iced::Vector::ZERO,
            )
            .expect("text trigger opens a native menu, not a dialog");
        let node = overlay.as_overlay_mut().layout(&renderer, viewport.size());
        let mut rows = MenuRows::default();
        overlay
            .as_overlay_mut()
            .operate(Layout::new(&node), &renderer, &mut rows);
        assert_eq!(rows.0.len(), 1);
    }

    #[test]
    fn keyboard_check_item_publishes_without_dismissing() {
        let renderer = renderer();
        let menu = Menu::new("Context").section([MenuItem::check("Toggle", false, 4)]);
        let mut p = Popover::new(
            iced::widget::space().width(100).height(30),
            popover_content(&menu, 360.0),
        )
        .padding(0)
        .popover_width(260.0)
        .context_trigger()
        .open_initially(true);
        let mut tree = Tree::new(&p as &dyn Widget<u8, Theme, iced::Renderer>);
        let node = p.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(500.0, 300.0)),
        );
        overlay_event(
            &mut p,
            &mut tree,
            &node,
            &renderer,
            key(Named::Home, keyboard::Modifiers::empty()),
        );
        assert_eq!(
            overlay_event(
                &mut p,
                &mut tree,
                &node,
                &renderer,
                key(Named::Space, keyboard::Modifiers::empty())
            ),
            vec![4]
        );
        assert!(tree.state.downcast_ref::<State>().open);
    }

    #[test]
    fn right_click_anchors_to_pointer_and_layout_fits_viewport() {
        let renderer = renderer();
        let menu = Menu::new("Context").section([MenuItem::action("Run", 1)]);
        let mut p = Popover::new(
            iced::widget::space().width(500).height(300),
            popover_content(&menu, 360.0),
        )
        .padding(0)
        .popover_width(260.0)
        .context_trigger();
        let mut tree = Tree::new(&p as &dyn Widget<u8, Theme, iced::Renderer>);
        let node = p.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(500.0, 300.0)),
        );
        let at = Point::new(490.0, 290.0);
        assert!(
            send(
                &mut p,
                &mut tree,
                &node,
                &renderer,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)),
                at
            )
            .0
        );
        assert_eq!(tree.state.downcast_ref::<State>().pointer, Some(at));
        let viewport = Rectangle::with_size(Size::new(500.0, 300.0));
        {
            let mut overlay = p
                .overlay(
                    &mut tree,
                    Layout::new(&node),
                    &renderer,
                    &viewport,
                    iced::Vector::ZERO,
                )
                .unwrap();
            for size in [Size::new(500.0, 300.0), Size::new(80.0, 60.0)] {
                let bounds = overlay.as_overlay_mut().layout(&renderer, size).bounds();
                assert!(bounds.x >= 0.0 && bounds.y >= 0.0, "{bounds:?}");
                assert!(
                    bounds.x + bounds.width <= size.width
                        && bounds.y + bounds.height <= size.height,
                    "{bounds:?}"
                );
            }
        }
        overlay_event(
            &mut p,
            &mut tree,
            &node,
            &renderer,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)),
        );
        assert!(!tree.state.downcast_ref::<State>().open);
    }
    #[test]
    fn keyboard_skips_disabled_rows_activates_commands_and_traverses_submenus() {
        let renderer = renderer();
        let submenu = Menu::new("Child").section([MenuItem::action("Child command", 7)]);
        let menu = Menu::new("Context").section([
            MenuItem::action("Disabled", 1).disabled(),
            MenuItem::submenu("Child", submenu),
        ]);
        let mut p = Popover::new(
            iced::widget::space().width(100).height(30),
            pages_content(&menu, 260.0, 0),
        )
        .padding(0)
        .popover_width(260.0)
        .context_trigger()
        .open_initially(true);
        let mut tree = Tree::new(&p as &dyn Widget<u8, Theme, iced::Renderer>);
        let node = p.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(500.0, 300.0)),
        );
        let none = keyboard::Modifiers::empty();
        overlay_event(
            &mut p,
            &mut tree,
            &node,
            &renderer,
            key(Named::ArrowDown, none),
        );
        assert_eq!(tree.state.downcast_ref::<State>().selected, Some(0));
        assert!(
            overlay_event(&mut p, &mut tree, &node, &renderer, key(Named::Enter, none)).is_empty()
        );
        assert!(tree.state.downcast_ref::<State>().open);
        settle(&mut p, &mut tree, &node, &renderer);
        overlay_event(
            &mut p,
            &mut tree,
            &node,
            &renderer,
            key(Named::ArrowLeft, none),
        );
        assert!(tree.state.downcast_ref::<State>().open);
        settle(&mut p, &mut tree, &node, &renderer);
        overlay_event(&mut p, &mut tree, &node, &renderer, key(Named::Home, none));
        overlay_event(
            &mut p,
            &mut tree,
            &node,
            &renderer,
            key(Named::ArrowRight, none),
        );
        settle(&mut p, &mut tree, &node, &renderer);
        overlay_event(&mut p, &mut tree, &node, &renderer, key(Named::End, none));
        assert_eq!(
            overlay_event(&mut p, &mut tree, &node, &renderer, key(Named::Enter, none)),
            vec![7]
        );
        assert!(!tree.state.downcast_ref::<State>().open);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layouts::anchor::{Align, Placement, Side};

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
            crate::layouts::anchor::place(anchor, size, window.size(), below()).position
        );
    }

    #[test]
    fn a_popover_stays_where_it_last_fit_until_that_overflows() {
        let region = Rectangle::new(Point::new(100.0, 100.0), Size::new(300.0, 200.0));
        let size = Size::new(80.0, 60.0);
        let anchor = |y| Rectangle::new(Point::new(150.0, y), Size::new(40.0, 20.0));
        // Near the bottom of the region: opens above.
        let (_, first) = place_within_choice(anchor(270.0), size, region, below());
        assert_eq!(first.0, Side::Top);
        // The anchor moves up so below fits again; remembering the choice keeps it above.
        let remembered = Placement {
            last: Some(first),
            ..below()
        };
        let (at, kept) = place_within_choice(anchor(200.0), size, region, remembered);
        assert_eq!(kept.0, Side::Top);
        assert!(at.y + size.height <= 200.0, "above the anchor: {at:?}");
        // Forgotten (the popover closed): back to the preferred side.
        let (_, fresh) = place_within_choice(anchor(200.0), size, region, below());
        assert_eq!(fresh.0, Side::Bottom);
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

#[cfg(test)]
mod spacing_params {
    use super::*;

    #[test]
    fn the_surface_gap_and_padding_are_parameters() {
        let _: Element<'_, ()> = popover_spaced(
            [
                iced::widget::text("a").into(),
                iced::widget::text("b").into(),
            ],
            200.0,
            9.0,
            [3.0, 5.0],
        );
    }
}

#[cfg(test)]
mod scrim_geometry_tests {
    use super::*;
    use iced::advanced::{clipboard, renderer::Headless};
    use std::sync::{Arc, Mutex};

    fn renderer() -> iced::Renderer {
        iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("software renderer")
    }

    fn viewport() -> Rectangle {
        Rectangle::with_size(Size::new(640.0, 640.0))
    }

    fn pixels(renderer: &mut iced::Renderer) -> Vec<u8> {
        renderer.screenshot(Size::new(640, 640), 1.0, iced::Color::WHITE)
    }

    fn dialog() -> Popover<'static, u8> {
        Popover::new(
            iced::widget::space(),
            button(iced::widget::space())
                .width(40)
                .height(40)
                .on_press(7),
        )
        .presentation(Presentation::Dialog)
        .open_initially(true)
        .keep_open(true)
    }

    fn draw_overlay(
        element: &mut Element<'_, u8>,
        tree: &mut Tree,
        node: &layout::Node,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        settle: bool,
    ) -> Vec<u8> {
        let viewport = viewport();
        let mut overlay = element
            .as_widget_mut()
            .overlay(
                tree,
                Layout::new(node),
                renderer,
                &viewport,
                iced::Vector::ZERO,
            )
            .expect("open dialog overlay");
        let node = overlay.as_overlay_mut().layout(renderer, viewport.size());
        if settle {
            let now = std::time::Instant::now();
            for time in [now, now + std::time::Duration::from_secs(1)] {
                overlay.as_overlay_mut().update(
                    &Event::Window(iced::window::Event::RedrawRequested(time)),
                    Layout::new(&node),
                    mouse::Cursor::Unavailable,
                    renderer,
                    &mut clipboard::Null,
                    &mut Shell::new(&mut Vec::new()),
                );
            }
        }
        renderer.reset(viewport);
        overlay.as_overlay().draw(
            renderer,
            theme,
            &renderer::Style {
                text_color: iced::Color::BLACK,
            },
            Layout::new(&node),
            mouse::Cursor::Unavailable,
        );
        pixels(renderer)
    }

    #[test]
    fn rendered_scrim_tracks_actual_square_pill_asymmetric_bounds_current_theme_and_state() {
        let mut renderer = renderer();
        let shape_state = Arc::new(Mutex::new(false));
        let state = shape_state.clone();
        // Keep the same widget, tree and resolver across parent-state/theme changes.
        let p = Popover::<u8>::new(iced::widget::space(), iced::widget::space())
            .presentation(Presentation::Dialog)
            .open_initially(true)
            .scrim_shape(move |theme, bounds| {
                let bounds = Rectangle {
                    x: 11.0,
                    y: 17.0,
                    width: bounds.width - 22.0,
                    height: 80.0,
                };
                let radius = if *state.lock().unwrap() {
                    iced::border::Radius {
                        top_left: theme.metrics.radius_m,
                        bottom_right: 7.0,
                        ..Default::default()
                    }
                } else {
                    theme.metrics.radius_m.into()
                };
                (bounds, radius)
            });
        let mut element: Element<'_, u8> = p.into();
        let mut tree = Tree::new(&element);
        let node = element.as_widget_mut().layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, viewport().size()),
        );
        for asymmetric in [false, true, false] {
            *shape_state.lock().unwrap() = asymmetric;
            for radius in [0.0, 40.0] {
                let mut theme = Theme::light();
                theme.metrics.radius_m = radius;
                for progress in [0.25, 0.5, 1.0] {
                    tree.state
                        .downcast_mut::<State>()
                        .appear
                        .set_immediate(progress);
                    let actual =
                        draw_overlay(&mut element, &mut tree, &node, &mut renderer, &theme, false);
                    renderer.reset(viewport());
                    let radius = if asymmetric {
                        iced::border::Radius {
                            top_left: radius,
                            bottom_right: 7.0,
                            ..Default::default()
                        }
                    } else {
                        radius.into()
                    };
                    renderer.fill_quad(
                        Quad {
                            bounds: Rectangle {
                                x: 11.0,
                                y: 17.0,
                                width: 618.0,
                                height: 80.0,
                            },
                            border: iced::Border::default().rounded(radius),
                            shadow: Shadow::default(),
                            snap: false,
                        },
                        crate::layouts::motion::scrim(&theme, progress),
                    );
                    assert_eq!(
                        actual,
                        pixels(&mut renderer),
                        "asymmetric={asymmetric}, progress={progress}"
                    );
                }
            }
        }
    }

    #[test]
    fn props_are_noop_by_default_and_last_explicit_geometry_builder_wins() {
        let theme = Theme::light();
        let bounds = viewport();
        let original = dialog().scrim_shape(|_, b| {
            (
                b,
                iced::border::Radius {
                    top_left: 1.0,
                    top_right: 2.0,
                    bottom_right: 3.0,
                    bottom_left: 4.0,
                },
            )
        });
        let before = original.scrim_shape.clone().unwrap();
        let result = PopoverProps::default().apply(original);
        assert!(Arc::ptr_eq(&before, result.scrim_shape.as_ref().unwrap()));
        let legacy = result.window_radius(0.0);
        assert!(legacy.scrim_shape.is_none());
        assert_eq!(
            resolve_scrim_shape(None, legacy.window_radius, &theme, bounds),
            Some((bounds, 0.0.into()))
        );
        let props = PopoverProps::default()
            .window_radius(19.0)
            .scrim_shape(|t, b| (b, t.metrics.radius_m.into()));
        let result = props.apply(legacy);
        assert_eq!(result.window_radius, None);
        assert_eq!(
            resolve_scrim_shape(result.scrim_shape.as_ref(), None, &theme, bounds),
            Some((bounds, theme.metrics.radius_m.into()))
        );
        let props = props.window_radius(3.0);
        assert!(props.scrim_shape.is_none());
        let result = props.apply(result);
        assert!(result.scrim_shape.is_none());
        assert_eq!(result.window_radius, Some(3.0));
        assert_eq!(resolve_scrim_shape(None, None, &theme, bounds), None);
    }

    fn input(kind: usize, props: PopoverProps, scrim: bool) -> Element<'static, u8> {
        use crate::primitives::{color_picker::Hsva, date::Date};
        let date = Date::new(2026, 11, 8).unwrap();
        let metrics = Metrics {
            radius_window: 73.0,
            ..Metrics::default()
        };
        match kind {
            0 => crate::widgets::DateInput::new("2026-11-08", date, |_| 1)
                .metrics(metrics)
                .dialog(true)
                .open_initially(true)
                .scrim(scrim)
                .dropdown_props(props)
                .into(),
            1 => crate::widgets::DateRangeInput::new("", date, |_| 1)
                .metrics(metrics)
                .dialog(true)
                .open_initially(true)
                .scrim(scrim)
                .dropdown_props(props)
                .into(),
            _ => crate::widgets::ColorInput::new(Hsva::BLACK, "", |_| 1, |_| 2)
                .metrics(metrics)
                .dialog(true)
                .open_initially(true)
                .scrim(scrim)
                .dropdown_props(props)
                .into(),
        }
    }

    #[test]
    fn date_range_color_never_guess_metric_corners_and_forward_current_shape_props() {
        let mut renderer = renderer();
        for kind in 0..3 {
            let mut render = |props, scrim| {
                let mut element = input(kind, props, scrim);
                let mut tree = Tree::new(&element);
                let node = element.as_widget_mut().layout(
                    &mut tree,
                    &renderer,
                    &layout::Limits::new(Size::ZERO, viewport().size()),
                );
                draw_overlay(
                    &mut element,
                    &mut tree,
                    &node,
                    &mut renderer,
                    &Theme::light(),
                    true,
                )
            };
            let baseline = render(PopoverProps::default(), false);
            assert_eq!(
                baseline,
                render(PopoverProps::default(), true),
                "unknown parent kind={kind}"
            );
            assert_ne!(
                baseline,
                render(PopoverProps::default().window_radius(0.0), true),
                "explicit square kind={kind}"
            );
            let calls = Arc::new(Mutex::new(Vec::new()));
            let recorded = calls.clone();
            let mut element = input(
                kind,
                PopoverProps::default().scrim_shape(move |t, bounds| {
                    recorded.lock().unwrap().push((t.metrics.radius_m, bounds));
                    (bounds, t.metrics.radius_m.into())
                }),
                true,
            );
            let mut tree = Tree::new(&element);
            let node = element.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, viewport().size()),
            );
            for radius in [0.0, 31.0, 0.0] {
                let mut theme = Theme::light();
                theme.metrics.radius_m = radius;
                draw_overlay(&mut element, &mut tree, &node, &mut renderer, &theme, true);
                assert_eq!(calls.lock().unwrap().last(), Some(&(radius, viewport())));
            }
        }
    }

    #[test]
    fn input_shape_builders_override_legacy_radius_without_metric_fallback() {
        use crate::primitives::{color_picker::Hsva, date::Date};
        let mut renderer = renderer();
        for kind in 0..3 {
            let calls = Arc::new(Mutex::new(Vec::new()));
            let recorded = calls.clone();
            let shape = move |theme: &Theme, bounds: Rectangle| {
                recorded
                    .lock()
                    .unwrap()
                    .push((theme.metrics.radius_m, bounds));
                (bounds, theme.metrics.radius_m.into())
            };
            let date = Date::new(2026, 11, 8).unwrap();
            let metrics = Metrics {
                radius_window: 91.0,
                ..Metrics::default()
            };
            let mut element: Element<'_, u8> = match kind {
                0 => crate::widgets::DateInput::new("", date, |_| 1)
                    .dialog(true)
                    .open_initially(true)
                    .window_radius(88.0)
                    .scrim_shape(shape)
                    .metrics(metrics)
                    .into(),
                1 => crate::widgets::DateRangeInput::new("", date, |_| 1)
                    .dialog(true)
                    .open_initially(true)
                    .window_radius(88.0)
                    .scrim_shape(shape)
                    .metrics(metrics)
                    .into(),
                _ => crate::widgets::ColorInput::new(Hsva::BLACK, "", |_| 1, |_| 2)
                    .dialog(true)
                    .open_initially(true)
                    .window_radius(88.0)
                    .scrim_shape(shape)
                    .metrics(metrics)
                    .into(),
            };
            let mut tree = Tree::new(&element);
            let node = element.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, viewport().size()),
            );
            for radius in [0.0, 23.0] {
                let mut theme = Theme::light();
                theme.metrics.radius_m = radius;
                draw_overlay(&mut element, &mut tree, &node, &mut renderer, &theme, true);
                assert_eq!(calls.lock().unwrap().last(), Some(&(radius, viewport())));
            }
        }
    }

    #[test]
    fn modal_capture_content_activation_outside_and_escape_do_not_depend_on_geometry() {
        let renderer = renderer();
        for geometry in 0..3 {
            for close_on_outside in [false, true] {
                let p = dialog().close_on_outside(close_on_outside);
                let mut p = match geometry {
                    0 => p,
                    1 => p.window_radius(0.0),
                    _ => p.scrim_shape(|_, b| {
                        (
                            b,
                            iced::border::Radius {
                                top_left: 0.0,
                                top_right: 12.0,
                                bottom_right: 24.0,
                                bottom_left: 36.0,
                            },
                        )
                    }),
                };
                let mut tree = Tree::new(&p as &dyn Widget<u8, Theme, iced::Renderer>);
                let node = p.layout(
                    &mut tree,
                    &renderer,
                    &layout::Limits::new(Size::ZERO, viewport().size()),
                );
                tree.state.downcast_mut::<State>().appear.set_immediate(1.0);
                {
                    let mut overlay = p
                        .overlay(
                            &mut tree,
                            Layout::new(&node),
                            &renderer,
                            &viewport(),
                            iced::Vector::ZERO,
                        )
                        .unwrap();
                    let node = overlay
                        .as_overlay_mut()
                        .layout(&renderer, viewport().size());
                    let center = Layout::new(&node)
                        .children()
                        .next()
                        .unwrap()
                        .bounds()
                        .center();
                    let mut messages = vec![];
                    for event in [
                        mouse::Event::ButtonPressed(mouse::Button::Left),
                        mouse::Event::ButtonReleased(mouse::Button::Left),
                    ] {
                        overlay.as_overlay_mut().update(
                            &Event::Mouse(event),
                            Layout::new(&node),
                            mouse::Cursor::Available(center),
                            &renderer,
                            &mut clipboard::Null,
                            &mut Shell::new(&mut messages),
                        );
                    }
                    assert_eq!(messages, vec![7]);
                    let mut shell = Shell::new(&mut messages);
                    overlay.as_overlay_mut().update(
                        &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                        Layout::new(&node),
                        mouse::Cursor::Available(Point::new(1.0, 1.0)),
                        &renderer,
                        &mut clipboard::Null,
                        &mut shell,
                    );
                    assert!(
                        shell.is_event_captured(),
                        "modal background remains blocked"
                    );
                }
                assert_eq!(tree.state.downcast_ref::<State>().open, !close_on_outside);
                if !close_on_outside {
                    let mut overlay = p
                        .overlay(
                            &mut tree,
                            Layout::new(&node),
                            &renderer,
                            &viewport(),
                            iced::Vector::ZERO,
                        )
                        .unwrap();
                    let node = overlay
                        .as_overlay_mut()
                        .layout(&renderer, viewport().size());
                    let mut messages = vec![];
                    let mut shell = Shell::new(&mut messages);
                    let event = Event::Keyboard(keyboard::Event::KeyPressed {
                        key: keyboard::Key::Named(Named::Escape),
                        modified_key: keyboard::Key::Named(Named::Escape),
                        physical_key: keyboard::key::Physical::Unidentified(
                            keyboard::key::NativeCode::Unidentified,
                        ),
                        location: keyboard::Location::Standard,
                        modifiers: keyboard::Modifiers::default(),
                        text: None,
                        repeat: false,
                    });
                    overlay.as_overlay_mut().update(
                        &event,
                        Layout::new(&node),
                        mouse::Cursor::Unavailable,
                        &renderer,
                        &mut clipboard::Null,
                        &mut shell,
                    );
                }
                assert!(!tree.state.downcast_ref::<State>().open);
            }
        }
    }
}

#[cfg(test)]
mod typed_props_tests {
    use super::*;
    use iced::advanced::Widget;

    fn widget() -> Popover<'static, ()> {
        Popover::new(iced::widget::text("trigger"), iced::widget::text("body"))
    }

    #[test]
    fn native_props_defaults_do_not_change_parent_policy() {
        let original = widget()
            .disabled(true)
            .keep_open(true)
            .scrim(false)
            .close_on_outside(false);
        let before = original.size();
        let original = PopoverProps::default().apply(original);
        assert_eq!(original.size(), before);
        assert!(original.disabled && original.keep_open);
        assert!(!original.scrim && !original.close_on_outside);
        assert!(original.style.is_none());
        assert!(!original.fit_content);
    }

    #[test]
    fn native_props_forward_geometry_and_owned_style() {
        let props = PopoverProps::default()
            .width(151)
            .popover_width(221.0)
            .offset(11.0)
            .padding(7)
            .fit_content(true)
            .window_radius(19.0)
            .appearance(Appearance::Flat)
            .align(Align::End)
            .bounds(PopoverBounds::Window)
            .side(crate::layouts::anchor::Side::Top)
            .style(|_, _| button::Style {
                text_color: iced::Color::BLACK,
                ..Default::default()
            });
        let result = props.apply(widget().disabled(true).keep_open(true));
        assert_eq!(result.width, Length::Fixed(151.0));
        assert_eq!(result.popover_width, Some(221.0));
        assert_eq!(result.offset, 11.0);
        assert_eq!(result.padding, Padding::from(7));
        assert!(result.fit_content && result.disabled && result.keep_open);
        assert_eq!(result.window_radius, Some(19.0));
        assert_eq!(
            (result.style.unwrap())(&Theme::default(), button::Status::Active).text_color,
            iced::Color::BLACK
        );
    }
}

crate::components::props::impl_merge!(PopoverProps; fields: [side, offset, bounds, appearance, align, padding, width, popover_width, fit_content, window_radius, style]; optional: []; nested: []);

crate::components::props::impl_merge!(PopoverSurfaceProps; fields: [spacing]; optional: []; nested: [container]);
