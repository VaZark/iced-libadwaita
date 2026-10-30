//! Widgets that animate themselves, following libadwaita's animation model.
//!
//! They keep their own progress in widget state and advance it on iced's
//! `RedrawRequested` events, requesting another redraw while moving, so the app needs
//! no subscription or message plumbing. Parameters come from [`animation::presets`]
//! (read from libadwaita's source); animations are skipped when the desktop disables
//! them ([`animation::enabled`]).
//!
//! * [`reveal`]: a `GtkRevealer`-style slide for `ExpanderRow` / `Banner`.
//! * [`drawer_layer`]: the collapsed-sidebar drawer; animates open *and* closed.
//! * [`entrance`]: slide-in (+ scrim fade) for dialogs, bottom sheets and toasts. These
//!   are mounted by the app only while shown, so they animate *in*; there is no exit
//!   animation because the widget is gone as soon as the app unmounts it.

use crate::animation::{presets, Spec, Trigger, Tween};
use crate::Theme;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{tree, Tree, Widget};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::{mouse, window, Background, Border, Color, Element, Event, Length, Rectangle, Size, Vector};
use std::time::Instant;

/// Animated progress in `[0, 1]` (overshoot allowed for underdamped springs).
struct Progress {
    tween: Tween,
    shown: f32,
}

impl Progress {
    fn new(initial: f32) -> Self {
        Self { tween: Tween::new(initial), shown: initial }
    }

    /// Moves toward `target` and advances to `now`. Returns whether it changed or is
    /// still moving (the caller then requests a redraw/relayout).
    fn drive(&mut self, target: f32, now: Instant, spec: Spec) -> bool {
        self.tween.set_target(target, now, spec);
        let v = self.tween.tick(now);
        let changed = (v - self.shown).abs() > f32::EPSILON;
        self.shown = v;
        changed || self.tween.is_animating()
    }
}

fn redraw_time(event: &Event) -> Option<Instant> {
    match event {
        Event::Window(window::Event::RedrawRequested(now)) => Some(*now),
        _ => None,
    }
}

/// The scrim color: `--shade-color` at double alpha, scaled by `k`.
fn scrim(theme: &Theme, k: f32) -> Color {
    let c = theme.semantic.shade;
    Color { a: (c.a * 2.0).min(1.0) * k.clamp(0.0, 1.0), ..c }
}

// ---------------------------------------------------------------------------
// Reveal
// ---------------------------------------------------------------------------

/// Slides `content` open/closed (`GtkRevealer` SLIDE_DOWN): 250ms, `ADW_EASE`.
pub struct Reveal<'a, M> {
    content: Element<'a, M, Theme>,
    revealed: bool,
    spec: Spec,
}

pub fn reveal<'a, M>(content: impl Into<Element<'a, M, Theme>>, revealed: bool) -> Reveal<'a, M> {
    Reveal {
        content: content.into(),
        revealed,
        spec: Spec::Timed(presets::REVEAL.0, presets::REVEAL.1),
    }
}

impl<'a, M> Reveal<'a, M> {
    pub fn spec(mut self, spec: Spec) -> Self {
        self.spec = spec;
        self
    }
}

struct RevealState {
    progress: Progress,
    full_height: f32,
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for Reveal<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<RevealState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(RevealState {
            progress: Progress::new(if self.revealed { 1.0 } else { 0.0 }),
            full_height: 0.0,
        })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        let width = limits.max().width;
        let child = self.content.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(width, f32::INFINITY)),
        );
        let state = tree.state.downcast_mut::<RevealState>();
        state.full_height = child.size().height;
        let h = (child.size().height * state.progress.shown.clamp(0.0, 1.0)).max(0.0);
        layout::Node::with_children(Size::new(child.size().width, h), vec![child])
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
        let state = tree.state.downcast_mut::<RevealState>();
        if let Some(now) = redraw_time(event)
            && state.progress.drive(if self.revealed { 1.0 } else { 0.0 }, now, Trigger::Reveal.spec(self.spec))
        {
            shell.invalidate_layout();
            shell.request_redraw();
        }
        if state.progress.shown > 0.001 {
            let bounds = layout.bounds();
            let cursor = if cursor.is_over(bounds) { cursor } else { mouse::Cursor::Unavailable };
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout.children().next().unwrap(),
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
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
        if !cursor.is_over(layout.bounds()) {
            return mouse::Interaction::None;
        }
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
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
        let bounds = layout.bounds();
        if bounds.height <= 0.5 {
            return;
        }
        // Clip to the revealed part; the child stays put and is uncovered downward.
        renderer.with_layer(bounds, |r| {
            self.content.as_widget().draw(
                &tree.children[0],
                r,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                viewport,
            );
        });
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        if tree.state.downcast_ref::<RevealState>().progress.shown > 0.001 {
            self.content.as_widget_mut().operate(&mut tree.children[0], layout.children().next().unwrap(), renderer, operation);
        }
    }
}

impl<'a, M: 'a> From<Reveal<'a, M>> for Element<'a, M, Theme> {
    fn from(r: Reveal<'a, M>) -> Self {
        Element::new(r)
    }
}

// ---------------------------------------------------------------------------
// Drawer layer
// ---------------------------------------------------------------------------

/// The collapsed-sidebar drawer: a pane that slides in from the start edge over a
/// fading scrim. Always mounted so it animates open *and* closed with libadwaita's
/// sidebar spring (damping ratio 1, mass 0.5, stiffness 500).
pub struct DrawerLayer<'a, M> {
    pane: Element<'a, M, Theme>,
    open: bool,
    width: f32,
    window_radius: f32,
    on_close: Option<M>,
    spec: Spec,
}

pub fn drawer_layer<'a, M>(
    pane: impl Into<Element<'a, M, Theme>>,
    open: bool,
    width: f32,
) -> DrawerLayer<'a, M> {
    DrawerLayer {
        pane: pane.into(),
        open,
        width,
        window_radius: 0.0,
        on_close: None,
        spec: Spec::Spring(presets::sidebar()),
    }
}

impl<'a, M> DrawerLayer<'a, M> {
    pub fn window_radius(mut self, r: f32) -> Self {
        self.window_radius = r;
        self
    }
    pub fn on_close(mut self, msg: M) -> Self {
        self.on_close = Some(msg);
        self
    }
}

struct DrawerState {
    progress: Progress,
}

impl<'a, M: Clone> Widget<M, Theme, iced::Renderer> for DrawerLayer<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<DrawerState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(DrawerState { progress: Progress::new(if self.open { 1.0 } else { 0.0 }) })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.pane)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.pane.as_widget()]);
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        let max = limits.max();
        let pane = self.pane.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(Size::new(self.width, max.height), Size::new(self.width, max.height)),
        );
        layout::Node::with_children(max, vec![pane])
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
        let state = tree.state.downcast_mut::<DrawerState>();
        if let Some(now) = redraw_time(event)
            && state.progress.drive(if self.open { 1.0 } else { 0.0 }, now, Trigger::Drawer.spec(self.spec))
        {
            shell.request_redraw();
        }
        let p = state.progress.shown;
        if p <= 0.001 && !self.open {
            return; // closed: invisible and transparent to events
        }
        let settled = (p - 1.0).abs() < 0.001;
        let pane_layout = layout.children().next().unwrap();
        let is_mouse = matches!(event, Event::Mouse(_) | Event::Touch(_));

        if settled {
            self.pane.as_widget_mut().update(&mut tree.children[0], event, pane_layout, cursor, renderer, clipboard, shell, viewport);
        }
        if is_mouse && !shell.is_event_captured() {
            // Anything not caught by the pane is the scrim: block it, and close on press.
            if matches!(event, Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) | Event::Touch(iced::touch::Event::FingerPressed { .. }))
                && !cursor.is_over(pane_layout.bounds())
                && let Some(m) = &self.on_close
            {
                shell.publish(m.clone());
            }
            if cursor.is_over(layout.bounds()) {
                shell.capture_event();
            }
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
        let p = tree.state.downcast_ref::<DrawerState>().progress.shown;
        if p <= 0.001 && !self.open {
            return mouse::Interaction::None;
        }
        self.pane.as_widget().mouse_interaction(&tree.children[0], layout.children().next().unwrap(), cursor, viewport, renderer)
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
        let p = tree.state.downcast_ref::<DrawerState>().progress.shown;
        if p <= 0.001 {
            return;
        }
        let bounds = layout.bounds();
        renderer.fill_quad(
            Quad {
                bounds,
                border: Border::default().rounded(self.window_radius),
                shadow: Default::default(),
                snap: false,
            },
            Background::Color(scrim(theme, p.min(1.0))),
        );
        let dx = -(1.0 - p) * self.width;
        renderer.with_translation(Vector::new(dx, 0.0), |r| {
            self.pane.as_widget().draw(&tree.children[0], r, theme, style, layout.children().next().unwrap(), cursor, viewport);
        });
    }
}

impl<'a, M: Clone + 'a> From<DrawerLayer<'a, M>> for Element<'a, M, Theme> {
    fn from(d: DrawerLayer<'a, M>) -> Self {
        Element::new(d)
    }
}

// ---------------------------------------------------------------------------
// Entrance
// ---------------------------------------------------------------------------

/// Slides `content` in from `offset` (pixels) to its place on mount, optionally with a
/// fading scrim over the whole layer. Dialogs and sheets use the sheet spring
/// (damping ratio 0.8, mass 1, stiffness 400); toasts use 300ms `ADW_EASE`.
pub struct Entrance<'a, M> {
    content: Element<'a, M, Theme>,
    offset: Vector,
    scrim_radius: Option<f32>,
    spec: Spec,
    trigger: Trigger,
}

/// Stacks an optional overlay layer over `content`. The stack is **always** present, so
/// showing or hiding the layer never changes the content's place in the widget tree: its
/// state (scroll offsets, focus, animations) survives, and it does not move. Without a
/// layer the second slot is an empty placeholder.
pub fn layered<'a, M: 'a>(
    content: impl Into<Element<'a, M, Theme>>,
    layer: Option<Element<'a, M, Theme>>,
) -> Element<'a, M, Theme> {
    iced::widget::stack![
        content.into(),
        layer.unwrap_or_else(|| iced::widget::space().into())
    ]
    .into()
}

pub fn entrance<'a, M>(content: impl Into<Element<'a, M, Theme>>, offset: Vector) -> Entrance<'a, M> {
    Entrance {
        content: content.into(),
        offset,
        scrim_radius: None,
        spec: Spec::Spring(presets::sheet(false)),
        trigger: Trigger::Dialog,
    }
}

impl<'a, M> Entrance<'a, M> {
    /// Draws a scrim (rounded to `radius`) behind the content that fades in with it.
    pub fn scrim(mut self, radius: f32) -> Self {
        self.scrim_radius = Some(radius);
        self
    }
    pub fn spec(mut self, spec: Spec) -> Self {
        self.spec = spec;
        self
    }
    /// Which trigger this entrance is: [`Trigger::Dialog`] (default), `Sheet` or `Toast`.
    /// Its `[motion]` setting (or reduced motion) can turn the animation off.
    pub fn trigger(mut self, t: Trigger) -> Self {
        self.trigger = t;
        self
    }
}

struct EntranceState {
    progress: Progress,
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for Entrance<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<EntranceState>()
    }
    fn state(&self) -> tree::State {
        // Starts hidden and animates to 1 on the first redraw (mounting is the trigger).
        tree::State::new(EntranceState { progress: Progress::new(if self.trigger.animated() { 0.0 } else { 1.0 }) })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        let child = self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits);
        layout::Node::with_children(child.size(), vec![child])
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
        let state = tree.state.downcast_mut::<EntranceState>();
        if let Some(now) = redraw_time(event)
            && state.progress.drive(1.0, now, self.trigger.spec(self.spec))
        {
            shell.request_redraw();
        }
        let settled = (state.progress.shown - 1.0).abs() < 0.002;
        if settled {
            self.content.as_widget_mut().update(&mut tree.children[0], event, layout.children().next().unwrap(), cursor, renderer, clipboard, shell, viewport);
        } else if matches!(event, Event::Mouse(_) | Event::Touch(_)) && cursor.is_over(layout.bounds()) {
            shell.capture_event(); // no click-through while sliding in
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
        self.content.as_widget().mouse_interaction(&tree.children[0], layout.children().next().unwrap(), cursor, viewport, renderer)
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
        let p = tree.state.downcast_ref::<EntranceState>().progress.shown;
        if let Some(radius) = self.scrim_radius {
            renderer.fill_quad(
                Quad { bounds: layout.bounds(), border: Border::default().rounded(radius), shadow: Default::default(), snap: false },
                Background::Color(scrim(theme, p.min(1.0))),
            );
        }
        let d = Vector::new(self.offset.x * (1.0 - p), self.offset.y * (1.0 - p));
        renderer.with_translation(d, |r| {
            self.content.as_widget().draw(&tree.children[0], r, theme, style, layout.children().next().unwrap(), cursor, viewport);
        });
    }
}

impl<'a, M: 'a> From<Entrance<'a, M>> for Element<'a, M, Theme> {
    fn from(e: Entrance<'a, M>) -> Self {
        Element::new(e)
    }
}

// ---------------------------------------------------------------------------
// Page transition (content cross-fade)
// ---------------------------------------------------------------------------

/// Fades `content` in whenever `key` changes: the `AdwViewStack` cross-fade (200ms,
/// ease-out-quad) for switching pages. The page below is not kept, so the new page fades
/// up out of the window background rather than blending with the old one.
pub struct PageTransition<'a, M> {
    content: Element<'a, M, Theme>,
    key: u64,
}

pub fn page_transition<'a, M>(content: impl Into<Element<'a, M, Theme>>, key: impl Into<u64>) -> PageTransition<'a, M> {
    PageTransition { content: content.into(), key: key.into() }
}

struct PageState {
    key: u64,
    progress: Progress,
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for PageTransition<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<PageState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(PageState { key: self.key, progress: Progress::new(1.0) })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<PageState>();
        if state.key != self.key {
            // A different page: restart hidden, so the very next frame is already faded.
            state.key = self.key;
            state.progress = Progress::new(if Trigger::PageChange.animated() { 0.0 } else { 1.0 });
        }
        tree.diff_children(&[self.content.as_widget()]);
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        let child = self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits);
        layout::Node::with_children(child.size(), vec![child])
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
        if let Some(now) = redraw_time(event)
            && tree.state.downcast_mut::<PageState>().progress.drive(1.0, now, Trigger::PageChange.spec(Spec::STYLE))
        {
            shell.request_redraw();
        }
        self.content.as_widget_mut().update(&mut tree.children[0], event, layout.children().next().unwrap(), cursor, renderer, clipboard, shell, viewport);
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(&tree.children[0], layout.children().next().unwrap(), cursor, viewport, renderer)
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content.as_widget_mut().operate(&mut tree.children[0], layout.children().next().unwrap(), renderer, operation);
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
        let p = tree.state.downcast_ref::<PageState>().progress.shown;
        if p >= 0.999 {
            return self.content.as_widget().draw(&tree.children[0], renderer, theme, style, layout.children().next().unwrap(), cursor, viewport);
        }
        let bounds = layout.bounds();
        renderer.with_layer(bounds, |r| {
            self.content.as_widget().draw(&tree.children[0], r, theme, style, layout.children().next().unwrap(), cursor, viewport);
        });
        // Fading the page in = fading the window background out over it. A later layer, so
        // it covers the page's text as well as its quads.
        let bg = theme.semantic.window.bg;
        renderer.with_layer(bounds, |r| {
            r.fill_quad(
                Quad { bounds, border: Border::default(), shadow: Default::default(), snap: false },
                Background::Color(Color { a: 1.0 - p.clamp(0.0, 1.0), ..bg }),
            );
        });
    }
}

impl<'a, M: 'a> From<PageTransition<'a, M>> for Element<'a, M, Theme> {
    fn from(t: PageTransition<'a, M>) -> Self {
        Element::new(t)
    }
}

// ---------------------------------------------------------------------------
// Push stack (AdwNavigationView push / pop)
// ---------------------------------------------------------------------------

/// Two pages of a navigation stack: `root` and the page `pushed` over it. Pushing slides
/// `top` in from the end edge while `root` moves a third of the way out under a fading
/// shade; popping reverses it, on libadwaita's navigation spring (damping ratio 1,
/// mass 1, stiffness 1000). Both pages stay mounted so each keeps its state.
pub struct PushStack<'a, M> {
    root: Element<'a, M, Theme>,
    top: Element<'a, M, Theme>,
    pushed: bool,
    window_radius: f32,
}

pub fn push_stack<'a, M>(
    root: impl Into<Element<'a, M, Theme>>,
    top: impl Into<Element<'a, M, Theme>>,
    pushed: bool,
) -> PushStack<'a, M> {
    PushStack { root: root.into(), top: top.into(), pushed, window_radius: 0.0 }
}

impl<'a, M> PushStack<'a, M> {
    /// Rounds the opaque backing of the sliding page to the window's corners.
    pub fn window_radius(mut self, r: f32) -> Self {
        self.window_radius = r;
        self
    }
}

struct PushState {
    progress: Progress,
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for PushStack<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<PushState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(PushState { progress: Progress::new(if self.pushed { 1.0 } else { 0.0 }) })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.root), Tree::new(&self.top)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.root.as_widget(), self.top.as_widget()]);
    }
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }
    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        let max = limits.max();
        let l = layout::Limits::new(max, max);
        let root = self.root.as_widget_mut().layout(&mut tree.children[0], renderer, &l);
        let top = self.top.as_widget_mut().layout(&mut tree.children[1], renderer, &l);
        layout::Node::with_children(max, vec![root, top])
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
        let state = tree.state.downcast_mut::<PushState>();
        if let Some(now) = redraw_time(event)
            && state.progress.drive(if self.pushed { 1.0 } else { 0.0 }, now, Trigger::PagePush.spec(Spec::Spring(presets::navigation())))
        {
            shell.request_redraw();
        }
        let p = state.progress.shown;
        let mut kids = layout.children();
        let (root_l, top_l) = (kids.next().unwrap(), kids.next().unwrap());
        if p <= 0.001 {
            self.root.as_widget_mut().update(&mut tree.children[0], event, root_l, cursor, renderer, clipboard, shell, viewport);
        } else if p >= 0.999 {
            self.top.as_widget_mut().update(&mut tree.children[1], event, top_l, cursor, renderer, clipboard, shell, viewport);
        } else if matches!(event, Event::Mouse(_) | Event::Touch(_)) {
            shell.capture_event(); // no clicks while a page is sliding
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
        let p = tree.state.downcast_ref::<PushState>().progress.shown;
        let mut kids = layout.children();
        let (root_l, top_l) = (kids.next().unwrap(), kids.next().unwrap());
        if p <= 0.001 {
            self.root.as_widget().mouse_interaction(&tree.children[0], root_l, cursor, viewport, renderer)
        } else if p >= 0.999 {
            self.top.as_widget().mouse_interaction(&tree.children[1], top_l, cursor, viewport, renderer)
        } else {
            mouse::Interaction::None
        }
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        let p = tree.state.downcast_ref::<PushState>().progress.shown;
        let mut kids = layout.children();
        let (root_l, top_l) = (kids.next().unwrap(), kids.next().unwrap());
        let (a, b) = tree.children.split_at_mut(1);
        if p < 0.999 {
            self.root.as_widget_mut().operate(&mut a[0], root_l, renderer, operation);
        }
        if p > 0.001 {
            self.top.as_widget_mut().operate(&mut b[0], top_l, renderer, operation);
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
        let p = tree.state.downcast_ref::<PushState>().progress.shown;
        let bounds = layout.bounds();
        let mut kids = layout.children();
        let (root_l, top_l) = (kids.next().unwrap(), kids.next().unwrap());
        let draw_root = |r: &mut iced::Renderer| {
            self.root.as_widget().draw(&tree.children[0], r, theme, style, root_l, cursor, viewport);
        };
        let draw_top = |r: &mut iced::Renderer| {
            self.top.as_widget().draw(&tree.children[1], r, theme, style, top_l, cursor, viewport);
        };
        if p <= 0.001 {
            return draw_root(renderer);
        }
        if p >= 0.999 {
            return draw_top(renderer);
        }
        // Each part in its own layer: within one layer all quads paint before all text, so the
        // old page's text would show through the new page's backing.
        renderer.with_layer(bounds, |r| {
            r.with_translation(Vector::new(-bounds.width / 3.0 * p, 0.0), |r| draw_root(r));
        });
        // The old page dims as the new one covers it.
        renderer.with_layer(bounds, |r| {
            r.fill_quad(
                Quad { bounds, border: Border::default().rounded(self.window_radius), shadow: Default::default(), snap: false },
                Background::Color(scrim(theme, p * 0.5)),
            );
        });
        let dx = bounds.width * (1.0 - p);
        renderer.with_layer(bounds, |r| {
            r.with_translation(Vector::new(dx, 0.0), |r| {
                // The sliding page is opaque and casts a shadow on the page it covers.
                r.fill_quad(
                    Quad {
                        bounds,
                        border: Border::default().rounded(self.window_radius),
                        shadow: iced::Shadow {
                            color: Color { a: 0.25, ..Color::BLACK },
                            offset: Vector::new(-2.0, 0.0),
                            blur_radius: 12.0,
                        },
                        snap: false,
                    },
                    Background::Color(theme.semantic.window.bg),
                );
                draw_top(r);
            });
        });
    }
}

impl<'a, M: 'a> From<PushStack<'a, M>> for Element<'a, M, Theme> {
    fn from(s: PushStack<'a, M>) -> Self {
        Element::new(s)
    }
}

// ---------------------------------------------------------------------------
// Selection highlight
// ---------------------------------------------------------------------------

/// A selection background that fades in and out (200ms, ease-out-quad, like GTK's style
/// transitions) behind `content` instead of switching instantly. Used by sidebar rows.
pub struct Highlight<'a, M> {
    content: Element<'a, M, Theme>,
    selected: bool,
    radius: f32,
    alpha: f32,
}

pub fn highlight<'a, M>(
    content: impl Into<Element<'a, M, Theme>>,
    selected: bool,
    radius: f32,
    alpha: f32,
) -> Highlight<'a, M> {
    Highlight { content: content.into(), selected, radius, alpha }
}

struct HighlightState {
    progress: Progress,
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for Highlight<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<HighlightState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(HighlightState { progress: Progress::new(if self.selected { 1.0 } else { 0.0 }) })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&[self.content.as_widget()]);
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        let child = self.content.as_widget_mut().layout(&mut tree.children[0], renderer, limits);
        layout::Node::with_children(child.size(), vec![child])
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
        if let Some(now) = redraw_time(event)
            && tree.state.downcast_mut::<HighlightState>().progress.drive(
                if self.selected { 1.0 } else { 0.0 },
                now,
                Trigger::Selection.spec(Spec::STYLE),
            )
        {
            shell.request_redraw();
        }
        self.content.as_widget_mut().update(&mut tree.children[0], event, layout.children().next().unwrap(), cursor, renderer, clipboard, shell, viewport);
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(&tree.children[0], layout.children().next().unwrap(), cursor, viewport, renderer)
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
        let p = tree.state.downcast_ref::<HighlightState>().progress.shown;
        if p > 0.001 {
            renderer.fill_quad(
                Quad { bounds: layout.bounds(), border: Border::default().rounded(self.radius), shadow: Default::default(), snap: false },
                Background::Color(crate::color::alpha(theme.semantic.sidebar.fg, self.alpha * p)),
            );
        }
        self.content.as_widget().draw(&tree.children[0], renderer, theme, style, layout.children().next().unwrap(), cursor, viewport);
    }
}

impl<'a, M: 'a> From<Highlight<'a, M>> for Element<'a, M, Theme> {
    fn from(h: Highlight<'a, M>) -> Self {
        Element::new(h)
    }
}

#[cfg(test)]
mod layered_tests {
    use super::*;
    use iced::advanced::widget::Tree;

    #[test]
    fn content_keeps_its_tree_position_with_or_without_a_layer() {
        let bare: iced::Element<'_, (), Theme> = layered(iced::widget::text("content"), None);
        let over: iced::Element<'_, (), Theme> =
            layered(iced::widget::text("content"), Some(iced::widget::text("layer").into()));
        let (a, b) = (Tree::new(&bare), Tree::new(&over));
        // Same shape: the content is child 0 either way, so its state survives showing
        // or hiding the layer and it never moves.
        assert_eq!(a.children.len(), 2);
        assert_eq!(b.children.len(), 2);
        assert_eq!(a.children[0].tag, b.children[0].tag);
    }
}
