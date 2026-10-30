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

use crate::animation::{presets, Spec, Tween};
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
            && state.progress.drive(if self.revealed { 1.0 } else { 0.0 }, now, self.spec)
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
            && state.progress.drive(if self.open { 1.0 } else { 0.0 }, now, self.spec)
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
}

pub fn entrance<'a, M>(content: impl Into<Element<'a, M, Theme>>, offset: Vector) -> Entrance<'a, M> {
    Entrance {
        content: content.into(),
        offset,
        scrim_radius: None,
        spec: Spec::Spring(presets::sheet(crate::config::current().reduced_motion)),
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
        tree::State::new(EntranceState { progress: Progress::new(if crate::animation::enabled() { 0.0 } else { 1.0 }) })
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
            && state.progress.drive(1.0, now, self.spec)
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
