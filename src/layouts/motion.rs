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

use crate::Theme;
use crate::layouts::animation::{Progress, Spec, Trigger, presets};
use crate::layouts::boundary::{self, SurfaceShape};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Tree, Widget, tree};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::{
    Background, Border, Color, Element, Event, Length, Rectangle, Size, Vector, mouse, window,
};
use std::time::Instant;

/// The frame loop every animated widget runs, written once: on a redraw, moves `progress`
/// toward `target` with `spec`, and asks the shell for the next frame (and a new layout, when
/// the progress changes the widget's size: `relayout`) while it is still moving. Returns whether
/// it moved.
pub(crate) fn advance<M>(
    progress: &mut Progress,
    event: &Event,
    target: f32,
    spec: Spec,
    shell: &mut Shell<'_, M>,
    relayout: bool,
) -> bool {
    let Some(now) = redraw_time(event) else {
        return false;
    };
    let moving = progress.drive(target, now, spec);
    if moving {
        if relayout {
            shell.invalidate_layout();
        }
        shell.request_redraw();
    }
    moving
}

/// Draws `f` translated by `d` inside `bounds`, handing it the viewport it must clip to (see
/// [`translated_viewport`]): the one place that draws a child sliding in or out.
pub(crate) fn draw_translated<R: iced::advanced::Renderer>(
    renderer: &mut R,
    bounds: Rectangle,
    viewport: &Rectangle,
    d: Vector,
    f: impl FnOnce(&mut R, &Rectangle),
) {
    let view = translated_viewport(bounds, viewport, d);
    with_translated_surfaces(d, || {
        renderer.with_translation(d, |r| f(r, &view));
    });
}

/// Ancestor shapes are stationary in renderer space, so child-local coordinates
/// move in the opposite direction. Reinstall the entire stack, not just its tip.
fn with_translated_surfaces<T>(delta: Vector, f: impl FnOnce() -> T) -> T {
    fn enter<T>(shapes: &[SurfaceShape], delta: Vector, f: impl FnOnce() -> T) -> T {
        if let Some((shape, rest)) = shapes.split_first() {
            let translated = SurfaceShape {
                bounds: Rectangle {
                    x: shape.bounds.x - delta.x,
                    y: shape.bounds.y - delta.y,
                    ..shape.bounds
                },
                ..*shape
            };
            boundary::with_surface(translated, || enter(rest, delta, f))
        } else {
            f()
        }
    }
    let ancestors = boundary::current_boundary();
    boundary::with_portal(|| enter(&ancestors.shapes, delta, f))
}

pub(crate) fn redraw_time(event: &Event) -> Option<Instant> {
    match event {
        Event::Window(window::Event::RedrawRequested(now)) => Some(*now),
        _ => None,
    }
}

/// The scrim color: `--shade-color` at double alpha, scaled by `k`.
pub(crate) fn scrim(theme: &Theme, k: f32) -> Color {
    let c = theme.semantic.shade;
    Color {
        a: (c.a * 2.0).min(1.0) * k.clamp(0.0, 1.0),
        ..c
    }
}

/// The `viewport` to give a child that is drawn translated by `d` inside `bounds`.
///
/// Widgets that clip (scrollables, clipping containers) open a layer of their own from the
/// viewport they are handed, and a layer is not clipped by the one it is inside. A translated
/// child handed the parent's viewport would clip in the wrong place and its content would be
/// painted outside `bounds` while it slides: the part of `bounds` that is visible, moved back by
/// the translation (the child's own coordinates), is what it must clip to.
pub(crate) fn translated_viewport(bounds: Rectangle, viewport: &Rectangle, d: Vector) -> Rectangle {
    let visible = bounds
        .intersection(viewport)
        .unwrap_or(Rectangle::new(bounds.position(), Size::ZERO));
    Rectangle {
        x: visible.x - d.x,
        y: visible.y - d.y,
        ..visible
    }
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
    /// How long it takes to open or close (default 250 ms).
    pub fn duration(mut self, d: std::time::Duration) -> Self {
        self.spec = self.spec.with_duration(d);
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

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
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
        advance(
            &mut state.progress,
            event,
            if self.revealed { 1.0 } else { 0.0 },
            Trigger::Reveal.spec(self.spec),
            shell,
            true,
        );
        if state.progress.shown > 0.001 {
            let bounds = layout.bounds();
            let cursor = if cursor.is_over(bounds) {
                cursor
            } else {
                mouse::Cursor::Unavailable
            };
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

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, Theme, iced::Renderer>> {
        if tree.state.downcast_ref::<RevealState>().progress.shown <= 0.001 {
            return None;
        }
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next()?,
            renderer,
            viewport,
            translation,
        )
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
        // Nested render layers do not inherit this layer's clip. Give descendants the
        // same visible viewport so their own layers cannot escape the revealed area.
        let Some(visible) = bounds.intersection(viewport) else {
            return;
        };
        renderer.with_layer(visible, |r| {
            self.content.as_widget().draw(
                &tree.children[0],
                r,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                &visible,
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
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                layout.children().next().unwrap(),
                renderer,
                operation,
            );
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
/// fading scrim when its background geometry is supplied. Always mounted so it
/// animates open *and* closed with libadwaita's
/// sidebar spring (damping ratio 1, mass 0.5, stiffness 500).
pub struct DrawerLayer<'a, M> {
    pane: Element<'a, M, Theme>,
    open: bool,
    width: f32,
    scrim_shape: Option<EffectShape<'a>>,
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
        scrim_shape: None,
        on_close: None,
        spec: Spec::Spring(presets::sidebar()),
    }
}

impl<'a, M> DrawerLayer<'a, M> {
    /// How long the drawer takes to open or close; the default spring is stretched to it.
    pub fn duration(mut self, d: std::time::Duration) -> Self {
        self.spec = self.spec.with_duration(d);
        self
    }
}

impl<'a, M> DrawerLayer<'a, M> {
    /// Explicit uniform background geometry, not automatic discovery of the pane or
    /// window silhouette. Without explicit geometry the nearest scoped parent
    /// surface supplies the scrim; outside a surface no scrim is painted.
    pub fn window_radius(self, r: f32) -> Self {
        self.scrim_shape(move |_, bounds| (bounds, r.into()))
    }

    /// Exact shape of the background being dimmed (not the sliding pane), resolved
    /// from its current geometry/style. Unknown backgrounds are left unmasked.
    pub fn scrim_shape(
        mut self,
        shape: impl Fn(&Theme, Rectangle) -> (Rectangle, iced::border::Radius) + 'a,
    ) -> Self {
        self.scrim_shape = Some(std::rc::Rc::new(shape));
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
        tree::State::new(DrawerState {
            progress: Progress::new(if self.open { 1.0 } else { 0.0 }),
        })
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

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let max = limits.max();
        let pane = self.pane.as_widget_mut().layout(
            &mut tree.children[0],
            renderer,
            &layout::Limits::new(
                Size::new(self.width, max.height),
                Size::new(self.width, max.height),
            ),
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
        advance(
            &mut state.progress,
            event,
            if self.open { 1.0 } else { 0.0 },
            Trigger::Drawer.spec(self.spec),
            shell,
            false,
        );
        let p = state.progress.shown;
        if p <= 0.001 && !self.open {
            return; // closed: invisible and transparent to events
        }
        let settled = (p - 1.0).abs() < 0.001;
        let pane_layout = layout.children().next().unwrap();
        let is_mouse = matches!(event, Event::Mouse(_) | Event::Touch(_));

        if settled {
            self.pane.as_widget_mut().update(
                &mut tree.children[0],
                event,
                pane_layout,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }
        if is_mouse && !shell.is_event_captured() {
            // Anything not caught by the pane is the scrim: block it, and close on press.
            if matches!(
                event,
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                    | Event::Touch(iced::touch::Event::FingerPressed { .. })
            ) && !cursor.is_over(pane_layout.bounds())
                && let Some(m) = &self.on_close
            {
                shell.publish(m.clone());
            }
            if cursor.is_over(layout.bounds()) {
                shell.capture_event();
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, Theme, iced::Renderer>> {
        if tree.state.downcast_ref::<DrawerState>().progress.shown <= 0.001 && !self.open {
            return None;
        }
        let p = tree.state.downcast_ref::<DrawerState>().progress.shown;
        let d = Vector::new(-(1.0 - p) * self.width, 0.0);
        self.pane.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next()?,
            renderer,
            &translated_viewport(layout.bounds(), viewport, d),
            translation + d,
        )
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
        self.pane.as_widget().mouse_interaction(
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
        let p = tree.state.downcast_ref::<DrawerState>().progress.shown;
        if p <= 0.001 {
            return;
        }
        let bounds = layout.bounds();
        if let Some(SurfaceShape { bounds, radius }) =
            background_shape(self.scrim_shape.as_ref(), theme, bounds)
        {
            renderer.fill_quad(
                Quad {
                    bounds,
                    border: Border::default().rounded(radius),
                    shadow: Default::default(),
                    snap: false,
                },
                Background::Color(scrim(theme, p)),
            );
        }
        let dx = -(1.0 - p) * self.width;
        draw_translated(
            renderer,
            bounds,
            viewport,
            Vector::new(dx, 0.0),
            |r, view| {
                self.pane.as_widget().draw(
                    &tree.children[0],
                    r,
                    theme,
                    style,
                    layout.children().next().unwrap(),
                    cursor,
                    view,
                );
            },
        );
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
    /// Where a [`Slide`](Effect::Slide) starts (and ends), relative to the content's place.
    offset: Vector,
    surface_scrim: bool,
    scrim_shape: Option<EffectShape<'a>>,
    fade_shape: Option<EffectShape<'a>>,
    spec: Spec,
    /// The spec of the way out, when it is not the way in's.
    exit_spec: Option<Spec>,
    trigger: Trigger,
    /// Whether the content is wanted: it animates in while `true` and out while `false`.
    open: bool,
    enter: Effect,
    exit: Effect,
    /// The value its effect has when the way in begins (0 to 1).
    starting: f32,
    /// The value its effect has when the way out is done, before it is discarded (0 to 1).
    ending: f32,
    on_settled: Option<Box<dyn Fn() -> M + 'a>>,
}

/// What a surface does as it arrives or leaves: one thing, chosen per way. The way in and the
/// way out each have their own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Effect {
    /// Moves in from (out to) its edge. Nothing else about it changes.
    #[default]
    Slide,
    /// Its opacity runs between hidden and shown, by laying the window colour over it (iced has no
    /// layer opacity). It does not move, and takes the surface's own bounds and corners itself.
    Fade,
    /// Shown and hidden at once.
    None,
}

impl Effect {
    /// Whether it moves.
    pub fn slides(self) -> bool {
        self == Effect::Slide
    }
    /// Whether its opacity changes.
    pub fn fades(self) -> bool {
        self == Effect::Fade
    }
    /// Whether it does nothing: shown and hidden at once.
    pub fn is_none(self) -> bool {
        self == Effect::None
    }
}


// Keep the public closure convention while resolving background geometry through
// the same normalized shape as actual boundaries.
type EffectShape<'a> =
    std::rc::Rc<dyn Fn(&Theme, Rectangle) -> (Rectangle, iced::border::Radius) + 'a>;

fn background_shape(
    explicit: Option<&EffectShape<'_>>,
    theme: &Theme,
    bounds: Rectangle,
) -> Option<SurfaceShape> {
    explicit
        .map(|resolve| {
            let (bounds, radius) = resolve(theme, bounds);
            SurfaceShape::new(bounds, radius)
        })
        .or_else(boundary::current_surface)
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

pub fn entrance<'a, M>(
    content: impl Into<Element<'a, M, Theme>>,
    offset: Vector,
) -> Entrance<'a, M> {
    Entrance {
        content: content.into(),
        offset,
        surface_scrim: false,
        scrim_shape: None,
        fade_shape: None,
        spec: Spec::Spring(presets::sheet(false)),
        exit_spec: None,
        trigger: Trigger::Dialog,
        open: true,
        enter: Effect::Slide,
        exit: Effect::Slide,
        starting: 0.0,
        ending: 0.0,
        on_settled: None,
    }
}

/// How opaque the window colour is over a layer that is fading in, at `progress` (0 to 1): it
/// starts covering the layer and clears away as the layer settles.
pub fn fade_cover(progress: f32) -> f32 {
    (1.0 - progress).clamp(0.0, 1.0)
}

impl<'a, M> Entrance<'a, M> {
    /// Dim the nearest actual parent surface during entrance. Outside a scoped
    /// surface this paints nothing. Explicit scrim geometry takes precedence.
    /// This never supplies a foreground fade mask: an arbitrary child's shape
    /// cannot be inferred from its parent. Plain entrances do not dim by default.
    pub fn surface_scrim(mut self) -> Self {
        self.surface_scrim = true;
        self
    }

    /// Dim the background with a uniform `radius` over the entrance's bounds.
    /// This never defines the foreground fade shape. Use [`Self::scrim_shape`]
    /// for independently resolved background bounds and corner radii, or
    /// [`Self::surface_scrim`] to inherit the actual parent surface.
    pub fn scrim(self, radius: f32) -> Self {
        self.scrim_shape(move |_, bounds| (bounds, radius.into()))
    }

    /// Geometry of the background being dimmed, resolved at draw time. It is independent
    /// of the foreground fade target. Return its exact bounds and four corner radii;
    /// reuse the background's geometry resolver rather than duplicating design values.
    pub fn scrim_shape(
        mut self,
        shape: impl Fn(&Theme, Rectangle) -> (Rectangle, iced::border::Radius) + 'a,
    ) -> Self {
        self.scrim_shape = Some(std::rc::Rc::new(shape));
        self
    }

    /// Exact geometry of the foreground surface being faded, in child coordinates.
    /// Resolve from the target's current style/layout, including asymmetric corners.
    /// This is a window-color cover, not true opacity: it is suitable for a filled
    /// rounded-rectangle surface, not arbitrary transparent content or silhouettes.
    pub fn fade_shape(
        mut self,
        shape: impl Fn(&Theme, Rectangle) -> (Rectangle, iced::border::Radius) + 'a,
    ) -> Self {
        self.fade_shape = Some(std::rc::Rc::new(shape));
        self
    }

    /// Build an entrance whose fade geometry comes from the very same style callback
    /// applied to its target container. The current theme resolves both at draw time.
    /// Use for filled rounded-rectangle surfaces; transparent content and shadows are
    /// not composited by this approximation. This replaces the container's style.
    pub fn container(
        content: iced::widget::Container<'a, M, Theme>,
        offset: Vector,
        style: impl Fn(&Theme) -> iced::widget::container::Style + 'a,
    ) -> Self
    where
        M: 'a,
    {
        let style = std::rc::Rc::new(style);
        let target = style.clone();
        entrance(content.style(move |theme| target(theme)), offset)
            .fade_shape(move |theme, bounds| (bounds, style(theme).border.radius))
    }
    pub fn spec(mut self, spec: Spec) -> Self {
        self.spec = spec;
        self
    }
    /// Fades a known target surface in instead of sliding it. Iced has no layer opacity,
    /// so a window-color cover clears away over the exact target geometry. Without a
    /// [`Self::fade_shape`] or [`Self::container`] geometry,
    /// this safely shows the content without a mask instead of guessing square corners.
    /// Input remains blocked until the entrance settles, as with a sliding entrance.
    pub fn fade(mut self) -> Self {
        self.enter = Effect::Fade;
        self.exit = Effect::Fade;
        self
    }
    /// The effect of the way in and of the way out.
    pub fn effects(mut self, enter: Effect, exit: Effect) -> Self {
        self.enter = enter;
        self.exit = exit;
        self
    }
    /// The value its effect has when the way in begins, 0 (the default) to 1: how far in a slide
    /// already is, how dark the scrim already is.
    pub fn starting_value(mut self, value: f32) -> Self {
        self.starting = value.clamp(0.0, 1.0);
        self
    }
    /// The value its effect has when the way out is done, 0 (the default) to 1, at which the
    /// content is discarded.
    pub fn ending_value(mut self, value: f32) -> Self {
        self.ending = value.clamp(0.0, 1.0);
        self
    }
    /// Whether the content is wanted. `false` runs the way out, after which the content is
    /// neither drawn nor reached; until then it must still be given. Default `true`.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    /// How long the way in takes (and the way out, unless [`exit_duration`](Self::exit_duration)
    /// says otherwise). A spring is stretched to it.
    pub fn duration(mut self, d: std::time::Duration) -> Self {
        self.spec = self.spec.with_duration(d);
        if let Some(e) = self.exit_spec {
            self.exit_spec = Some(e.with_duration(d));
        }
        self
    }
    /// How long the way out takes, when it is not the way in's.
    pub fn exit_duration(mut self, d: std::time::Duration) -> Self {
        self.exit_spec = Some(self.exit_spec.unwrap_or(self.spec).with_duration(d));
        self
    }
    /// The spec of the way out, when it is not the way in's.
    pub fn exit_spec(mut self, spec: Spec) -> Self {
        self.exit_spec = Some(spec);
        self
    }
    /// Sent once the way out has finished: the content is gone and the app can stop giving it.
    pub fn on_settled(mut self, message: M) -> Self
    where
        M: Clone + 'a,
    {
        self.on_settled = Some(Box::new(move || message.clone()));
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
    /// Which way it was last heading, for the effect to draw.
    heading_open: bool,
    /// The settled message of this way out has been sent.
    settled_sent: bool,
    /// Where the way out ends, taken from the widget on each frame.
    ending: f32,
}

impl EntranceState {
    /// Closed, and the way out is done: nothing to draw or reach.
    fn gone(&self) -> bool {
        !self.heading_open && self.progress.shown <= self.ending + 0.001
    }
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for Entrance<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<EntranceState>()
    }
    fn state(&self) -> tree::State {
        // Starts hidden and animates to 1 on the first redraw (mounting is the trigger).
        tree::State::new(EntranceState {
            progress: Progress::new(if self.trigger.animated() {
                self.starting
            } else {
                1.0
            }),
            heading_open: true,
            settled_sent: false,
            ending: self.ending,
        })
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

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
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
        // Coming in from out of sight starts from the starting value, not from nothing.
        if self.open && !state.heading_open && state.progress.shown < self.starting {
            state.progress.set_immediate(self.starting);
        }
        state.heading_open = self.open;
        state.ending = self.ending;
        let (effect, spec) = if self.open {
            (self.enter, self.spec)
        } else {
            (self.exit, self.exit_spec.unwrap_or(self.spec))
        };
        let spec = if effect.is_none() {
            Spec::Instant
        } else {
            self.trigger.spec(spec)
        };
        advance(
            &mut state.progress,
            event,
            if self.open { 1.0 } else { self.ending },
            spec,
            shell,
            false,
        );
        if self.open {
            state.settled_sent = false;
        } else if state.gone() && !state.settled_sent {
            state.settled_sent = true;
            if let Some(message) = &self.on_settled {
                shell.publish(message());
            }
        }
        if state.gone() {
            return;
        }
        let settled = self.open && (state.progress.shown - 1.0).abs() < 0.002;
        if settled {
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
        } else if matches!(event, Event::Mouse(_) | Event::Touch(_))
            && cursor.is_over(layout.bounds())
        {
            shell.capture_event(); // no click-through while sliding in
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, Theme, iced::Renderer>> {
        if tree.state.downcast_ref::<EntranceState>().gone() {
            return None;
        }
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next()?,
            renderer,
            viewport,
            translation,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        if tree.state.downcast_ref::<EntranceState>().gone() {
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
        let state = tree.state.downcast_ref::<EntranceState>();
        if state.gone() {
            return;
        }
        let p = state.progress.shown;
        let effect = if state.heading_open {
            self.enter
        } else {
            self.exit
        };
        let background = if self.surface_scrim || self.scrim_shape.is_some() {
            background_shape(self.scrim_shape.as_ref(), theme, layout.bounds())
        } else {
            None
        };
        if let Some(SurfaceShape { bounds, radius }) = background {
            renderer.fill_quad(
                Quad {
                    bounds,
                    border: Border::default().rounded(radius),
                    shadow: Default::default(),
                    snap: false,
                },
                Background::Color(scrim(theme, p.min(1.0))),
            );
        }
        let d = if effect.slides() {
            Vector::new(self.offset.x * (1.0 - p), self.offset.y * (1.0 - p))
        } else {
            Vector::ZERO
        };
        draw_translated(renderer, layout.bounds(), viewport, d, |r, view| {
            self.content.as_widget().draw(
                &tree.children[0],
                r,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                view,
            );
        });
        if let Some(shape) = self
            .fade_shape
            .as_ref()
            .filter(|_| effect.fades() && p < 0.999)
        {
            // A later layer, so the window colour covers the content's text as well as its quads.
            let bg = theme.semantic.window.bg;
            let (bounds, radius) = shape(theme, layout.children().next().unwrap().bounds());
            renderer.with_layer(layout.bounds(), |r| {
                r.fill_quad(
                    Quad {
                        bounds,
                        border: Border::default().rounded(radius),
                        shadow: Default::default(),
                        snap: false,
                    },
                    Background::Color(Color {
                        a: bg.a * fade_cover(p),
                        ..bg
                    }),
                );
            });
        }
    }
}

impl<'a, M: 'a> From<Entrance<'a, M>> for Element<'a, M, Theme> {
    fn from(e: Entrance<'a, M>) -> Self {
        Element::new(e)
    }
}

// ---------------------------------------------------------------------------
// FadeCover
// ---------------------------------------------------------------------------

/// Fades its content in and out by laying the window colour over it, over the content's **own**
/// bounds and corners: it needs no help from where the content sits, so it works for a surface
/// that is pushed, floated, full-screen or small. Iced has no layer opacity, so it is a cover
/// that clears away (a fill-coloured, rounded surface is what it suits).
///
/// It runs its own progress toward `open`, by the spec of the way it is going. It starts hidden
/// when `mount_hidden` (a layer that appears) and shown otherwise (a panel that was there).
pub struct FadeCover<'a, M> {
    content: Element<'a, M, Theme>,
    open: bool,
    enter: Effect,
    exit: Effect,
    spec: Spec,
    exit_spec: Option<Spec>,
    mount_hidden: bool,
    trigger: Trigger,
    /// The opacity it has when the way in begins (0 to 1).
    starting: f32,
    /// The opacity it has when the way out is done, before it is discarded (0 to 1).
    ending: f32,
    radius: Box<dyn Fn(&Theme) -> iced::border::Radius + 'a>,
    share: Option<std::rc::Rc<std::cell::Cell<f32>>>,
    backdrop: Option<Box<dyn Fn(&Theme) -> Color + 'a>>,
}

struct FadeCoverState {
    progress: Progress,
    heading_open: bool,
}

impl<'a, M> FadeCover<'a, M> {
    pub fn new(content: impl Into<Element<'a, M, Theme>>, open: bool) -> Self {
        Self {
            content: content.into(),
            open,
            enter: Effect::Fade,
            exit: Effect::Fade,
            spec: Spec::Timed(presets::REVEAL.0, presets::REVEAL.1),
            exit_spec: None,
            mount_hidden: false,
            trigger: Trigger::Dialog,
            starting: 0.0,
            ending: 0.0,
            radius: Box::new(|_| iced::border::Radius::default()),
            share: None,
            backdrop: None,
        }
    }
    /// The colour the surface fades to and from: whatever shows behind it (default the window
    /// colour). The cover is laid over the surface **and** what is on it, so its own fill fades
    /// as well as its contents; with the wrong backdrop the fill would stay and only the contents
    /// would fade.
    pub fn backdrop(mut self, color: impl Fn(&Theme) -> Color + 'a) -> Self {
        self.backdrop = Some(Box::new(color));
        self
    }
    /// [`backdrop`](Self::backdrop), when there is one.
    pub fn backdrop_opt(mut self, color: Option<std::rc::Rc<dyn Fn(&Theme) -> Color + 'a>>) -> Self {
        if let Some(f) = color {
            self.backdrop = Some(Box::new(move |t| f(t)));
        }
        self
    }
    /// Writes the progress it shows (0 hidden to 1 shown) into `cell` on every frame, for
    /// whatever has to wait for the fade (a room that closes after it).
    pub fn share(mut self, cell: std::rc::Rc<std::cell::Cell<f32>>) -> Self {
        self.share = Some(cell);
        self
    }
    /// Which ways fade (the others show and hide at once).
    pub fn effects(mut self, enter: Effect, exit: Effect) -> Self {
        self.enter = enter;
        self.exit = exit;
        self
    }
    pub fn spec(mut self, spec: Spec) -> Self {
        self.spec = spec;
        self
    }
    pub fn exit_spec(mut self, spec: Spec) -> Self {
        self.exit_spec = Some(spec);
        self
    }
    /// The opacity it has when the way in begins, 0 (the default: hidden) to 1. A page that
    /// opens at 0 is a blank window for a moment; one that opens at 0.5 is already half there.
    pub fn starting_value(mut self, opacity: f32) -> Self {
        self.starting = opacity.clamp(0.0, 1.0);
        self
    }
    /// The opacity it has when the way out is done, 0 (the default) to 1, at which it is
    /// discarded.
    pub fn ending_value(mut self, opacity: f32) -> Self {
        self.ending = opacity.clamp(0.0, 1.0);
        self
    }
    /// Which trigger this is (its `[motion]` setting can turn the animation off).
    pub fn trigger(mut self, trigger: Trigger) -> Self {
        self.trigger = trigger;
        self
    }
    /// Starts hidden and fades in on mounting, rather than being there already.
    pub fn mount_hidden(mut self, hidden: bool) -> Self {
        self.mount_hidden = hidden;
        self
    }
    /// The corners of the content, which the cover follows.
    pub fn radius(mut self, radius: impl Fn(&Theme) -> iced::border::Radius + 'a) -> Self {
        self.radius = Box::new(radius);
        self
    }
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for FadeCover<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<FadeCoverState>()
    }
    fn state(&self) -> tree::State {
        let start = if self.mount_hidden && self.trigger.animated() {
            self.starting
        } else if self.open || self.mount_hidden {
            1.0
        } else {
            0.0
        };
        tree::State::new(FadeCoverState {
            progress: Progress::new(start),
            heading_open: self.open,
        })
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
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
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
        let state = tree.state.downcast_mut::<FadeCoverState>();
        if self.open && !state.heading_open && state.progress.shown < self.starting {
            state.progress.set_immediate(self.starting);
        }
        state.heading_open = self.open;
        let (effect, spec) = if self.open {
            (self.enter, self.spec)
        } else {
            (self.exit, self.exit_spec.unwrap_or(self.spec))
        };
        // A way that does not fade jumps; one that does follows the trigger's switch.
        let spec = if effect.fades() {
            self.trigger.spec(spec)
        } else {
            Spec::Instant
        };
        advance(
            &mut state.progress,
            event,
            if self.open { 1.0 } else { self.ending },
            spec,
            shell,
            false,
        );
        // What the room waits for: how far it still is from its ending value.
        if let Some(cell) = &self.share {
            cell.set((state.progress.shown - self.ending).max(0.0));
        }
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
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next()?,
            renderer,
            viewport,
            translation,
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
        let state = tree.state.downcast_ref::<FadeCoverState>();
        let p = state.progress.shown;
        let effect = if state.heading_open {
            self.enter
        } else {
            self.exit
        };
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
        if effect.fades() && p < 0.999 {
            let bg = self
                .backdrop
                .as_ref()
                .map_or(theme.semantic.window.bg, |f| f(theme));
            let bounds = layout.bounds();
            renderer.with_layer(bounds, |r| {
                r.fill_quad(
                    Quad {
                        bounds,
                        border: Border::default().rounded((self.radius)(theme)),
                        shadow: Default::default(),
                        snap: false,
                    },
                    Background::Color(Color {
                        a: bg.a * fade_cover(p),
                        ..bg
                    }),
                );
            });
        }
    }
}

impl<'a, M: 'a> From<FadeCover<'a, M>> for Element<'a, M, Theme> {
    fn from(f: FadeCover<'a, M>) -> Self {
        Element::new(f)
    }
}

// ---------------------------------------------------------------------------
// Page transition (content cross-fade)
// ---------------------------------------------------------------------------

/// Fades `content` in whenever `key` changes: the `AdwViewStack` cross-fade (200ms,
/// ease-out-quad) for switching pages. The page below is not kept, so the new page fades
/// up out of the window background rather than blending with the old one. A cover is
/// only painted with known target geometry ([`PageTransition::container`] or
/// [`PageTransition::fade_shape`]); arbitrary content is safely drawn unmasked.
pub struct PageTransition<'a, M> {
    content: Element<'a, M, Theme>,
    key: u64,
    scroll_key: Option<u64>,
    trigger: Trigger,
    fade_shape: Option<EffectShape<'a>>,
    duration: Option<std::time::Duration>,
}

pub fn page_transition<'a, M>(
    content: impl Into<Element<'a, M, Theme>>,
    key: impl Into<u64>,
) -> PageTransition<'a, M> {
    PageTransition {
        content: content.into(),
        key: key.into(),
        scroll_key: None,
        trigger: Trigger::PageChange,
        fade_shape: None,
        duration: None,
    }
}

impl<'a, M> PageTransition<'a, M> {
    /// How long the cross-fade takes (default 200 ms).
    pub fn duration(mut self, d: std::time::Duration) -> Self {
        self.duration = Some(d);
        self
    }
    /// Exact filled surface to cover while fading. Iced has no composited opacity;
    /// transparent/arbitrary silhouettes cannot be faded this way. Without a known
    /// shape the content is drawn unmasked; keys, scroll restoration and events work
    /// independently of this presentation choice.
    pub fn fade_shape(
        mut self,
        shape: impl Fn(&Theme, Rectangle) -> (Rectangle, iced::border::Radius) + 'a,
    ) -> Self {
        self.fade_shape = Some(std::rc::Rc::new(shape));
        self
    }

    /// Share the actual target container style with its fade geometry resolver.
    /// Replaces the supplied container's style. Appropriate for filled surfaces;
    /// this window-color cover does not fade transparent contents or shadows.
    pub fn container(
        content: iced::widget::Container<'a, M, Theme>,
        key: impl Into<u64>,
        style: impl Fn(&Theme) -> iced::widget::container::Style + 'a,
    ) -> Self
    where
        M: 'a,
    {
        let style = std::rc::Rc::new(style);
        let target = style.clone();
        page_transition(content.style(move |theme| target(theme)), key)
            .fade_shape(move |theme, bounds| (bounds, style(theme).border.radius))
    }

    /// Which trigger this fade is (default [`Trigger::PageChange`]); its `[motion]` setting, or
    /// reduced motion, can turn it off.
    pub fn trigger(mut self, t: Trigger) -> Self {
        self.trigger = t;
        self
    }

    /// Identifies the page for its scroll position, when that differs from the fade's key (a
    /// page that is not faded, like the parent page of a collapsed split view, still scrolls on
    /// its own). Defaults to the fade's key.
    pub fn scroll_key(mut self, key: impl Into<u64>) -> Self {
        self.scroll_key = Some(key.into());
        self
    }
}

/// Where each page was scrolled to, for the session (as long as the transition stays mounted).
/// A page never seen before starts at the top; the offset of one page is never applied to
/// another.
struct PageState {
    key: u64,
    scroll_key: u64,
    progress: Progress,
    offsets: std::collections::HashMap<u64, f32>,
    restore: Option<f32>,
}

/// Reads the vertical offset of the outermost scrollable.
#[derive(Default)]
struct ReadScroll(Option<f32>);

impl iced::advanced::widget::Operation for ReadScroll {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn iced::advanced::widget::Operation)) {
        operate(self);
    }
    fn scrollable(
        &mut self,
        _id: Option<&iced::advanced::widget::Id>,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
        translation: Vector,
        _state: &mut dyn iced::advanced::widget::operation::Scrollable,
    ) {
        self.0.get_or_insert(translation.y);
    }
}

/// Scrolls the outermost scrollable to a vertical offset.
struct RestoreScroll(Option<f32>);

impl iced::advanced::widget::Operation for RestoreScroll {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn iced::advanced::widget::Operation)) {
        operate(self);
    }
    fn scrollable(
        &mut self,
        _id: Option<&iced::advanced::widget::Id>,
        _bounds: Rectangle,
        _content_bounds: Rectangle,
        _translation: Vector,
        state: &mut dyn iced::advanced::widget::operation::Scrollable,
    ) {
        if let Some(y) = self.0.take() {
            state.scroll_to(iced::widget::scrollable::AbsoluteOffset {
                x: None,
                y: Some(y),
            });
        }
    }
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for PageTransition<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<PageState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(PageState {
            key: self.key,
            scroll_key: self.scroll_key.unwrap_or(self.key),
            progress: Progress::new(1.0),
            offsets: Default::default(),
            restore: None,
        })
    }
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }
    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<PageState>();
        if state.key != self.key {
            // A different page: restart hidden, so the very next frame is already faded.
            state.key = self.key;
            state.progress = Progress::new(if self.trigger.animated() { 0.0 } else { 1.0 });
        }
        let scroll_key = self.scroll_key.unwrap_or(self.key);
        if state.scroll_key != scroll_key {
            // A different page has its own scroll position: its last one this session, else the
            // top. The old widget state is dropped so the scrollable does not keep the old
            // page's offset.
            state.scroll_key = scroll_key;
            state.restore = state.offsets.get(&scroll_key).copied();
            tree.children = vec![Tree::new(&self.content)];
        }
        tree.diff_children(&[self.content.as_widget()]);
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
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
        advance(
            &mut tree.state.downcast_mut::<PageState>().progress,
            event,
            1.0,
            self.trigger.spec(match self.duration {
                Some(d) => Spec::STYLE.with_duration(d),
                None => Spec::STYLE,
            }),
            shell,
            false,
        );
        let child_layout = layout.children().next().unwrap();
        // Back to a page seen before: put its scroll position back once it has a layout.
        if matches!(
            event,
            Event::Window(iced::window::Event::RedrawRequested(_))
        ) && let Some(y) = tree.state.downcast_mut::<PageState>().restore.take()
        {
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                child_layout,
                renderer,
                &mut RestoreScroll(Some(y)),
            );
            shell.request_redraw();
        }
        self.content.as_widget_mut().update(
            &mut tree.children[0],
            event,
            child_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
        // Remember where this page is scrolled to (wheel, drag, keys).
        if matches!(
            event,
            Event::Mouse(_) | Event::Touch(_) | Event::Keyboard(_)
        ) {
            let mut read = ReadScroll::default();
            self.content.as_widget_mut().operate(
                &mut tree.children[0],
                child_layout,
                renderer,
                &mut read,
            );
            if let Some(y) = read.0 {
                let state = tree.state.downcast_mut::<PageState>();
                let key = state.scroll_key;
                state.offsets.insert(key, y);
            }
        }
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next()?,
            renderer,
            viewport,
            translation,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &tree.children[0],
            layout.children().next().unwrap(),
            cursor,
            viewport,
            renderer,
        )
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn iced::advanced::widget::Operation,
    ) {
        self.content.as_widget_mut().operate(
            &mut tree.children[0],
            layout.children().next().unwrap(),
            renderer,
            operation,
        );
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
        if p >= 0.999 || self.fade_shape.is_none() {
            return self.content.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                layout.children().next().unwrap(),
                cursor,
                viewport,
            );
        }
        let bounds = layout.bounds();
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
        // Fading the page in = fading the window background out over it. A later layer, so
        // it covers the page's text as well as its quads.
        let bg = theme.semantic.window.bg;
        let (target_bounds, radius) =
            self.fade_shape.as_ref().unwrap()(theme, layout.children().next().unwrap().bounds());
        renderer.with_layer(bounds, |r| {
            r.fill_quad(
                Quad {
                    bounds: target_bounds,
                    border: Border::default().rounded(radius),
                    shadow: Default::default(),
                    snap: false,
                },
                Background::Color(Color {
                    a: 1.0 - p.clamp(0.0, 1.0),
                    ..bg
                }),
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
/// Dimming requires known root geometry; arbitrary targets keep their own backing
/// and shadow with no guessed rectangle added. See [`PushStack::containers`].
pub struct PushStack<'a, M> {
    root: Element<'a, M, Theme>,
    top: Element<'a, M, Theme>,
    pushed: bool,
    root_shape: Option<EffectShape<'a>>,
    duration: Option<std::time::Duration>,
}

pub fn push_stack<'a, M>(
    root: impl Into<Element<'a, M, Theme>>,
    top: impl Into<Element<'a, M, Theme>>,
    pushed: bool,
) -> PushStack<'a, M> {
    PushStack {
        root: root.into(),
        top: top.into(),
        pushed,
        root_shape: None,
        duration: None,
    }
}

impl<'a, M> PushStack<'a, M> {
    /// How long a push or pop takes; the navigation spring is stretched to it.
    pub fn duration(mut self, d: std::time::Duration) -> Self {
        self.duration = Some(d);
        self
    }
    /// Exact root surface geometry in its untranslated child coordinates. The
    /// dimmer moves with that target. Without known geometry it is not painted.
    pub fn root_shape(
        mut self,
        shape: impl Fn(&Theme, Rectangle) -> (Rectangle, iced::border::Radius) + 'a,
    ) -> Self {
        self.root_shape = Some(std::rc::Rc::new(shape));
        self
    }

    /// Apply the actual styles to both target containers and derive dimming from
    /// the root's same style resolver. Each target owns its real background, border
    /// and shadow, including during motion; no synthetic top backing is added.
    /// Replaces both supplied containers' styles. Unknown/transparent silhouettes
    /// need true compositing and should use the unmasked [`push_stack`] instead.
    pub fn containers(
        root: iced::widget::Container<'a, M, Theme>,
        top: iced::widget::Container<'a, M, Theme>,
        pushed: bool,
        root_style: impl Fn(&Theme) -> iced::widget::container::Style + 'a,
        top_style: impl Fn(&Theme) -> iced::widget::container::Style + 'a,
    ) -> Self
    where
        M: 'a,
    {
        let style = std::rc::Rc::new(root_style);
        let target = style.clone();
        push_stack(
            root.style(move |theme| target(theme)),
            top.style(top_style),
            pushed,
        )
        .root_shape(move |theme, bounds| (bounds, style(theme).border.radius))
    }
}

struct PushState {
    progress: Progress,
}

fn push_root_translation(width: f32, progress: f32) -> Vector {
    Vector::new(-width / 3.0 * progress, 0.0)
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for PushStack<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<PushState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(PushState {
            progress: Progress::new(if self.pushed { 1.0 } else { 0.0 }),
        })
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
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let max = limits.max();
        let l = layout::Limits::new(max, max);
        let root = self
            .root
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, &l);
        let top = self
            .top
            .as_widget_mut()
            .layout(&mut tree.children[1], renderer, &l);
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
        advance(
            &mut state.progress,
            event,
            if self.pushed { 1.0 } else { 0.0 },
            Trigger::PagePush.spec(match self.duration {
                Some(d) => Spec::Spring(presets::navigation()).with_duration(d),
                None => Spec::Spring(presets::navigation()),
            }),
            shell,
            false,
        );
        let p = state.progress.shown;
        let mut kids = layout.children();
        let (root_l, top_l) = (kids.next().unwrap(), kids.next().unwrap());
        if matches!(event, Event::Keyboard(_)) {
            // Keys reach both pages, so shortcuts registered by a hidden page still work.
            self.root.as_widget_mut().update(
                &mut tree.children[0],
                event,
                root_l,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
            self.top.as_widget_mut().update(
                &mut tree.children[1],
                event,
                top_l,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        } else if p <= 0.001 {
            self.root.as_widget_mut().update(
                &mut tree.children[0],
                event,
                root_l,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        } else if p >= 0.999 {
            self.top.as_widget_mut().update(
                &mut tree.children[1],
                event,
                top_l,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        } else if matches!(event, Event::Mouse(_) | Event::Touch(_)) {
            shell.capture_event(); // no clicks while a page is sliding
        }
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, Theme, iced::Renderer>> {
        // Popovers belong to the page being shown: the pushed one past halfway.
        let p = tree.state.downcast_ref::<PushState>().progress.shown;
        let top = p > 0.5;
        let d = if p >= 0.999 {
            Vector::ZERO
        } else if top {
            Vector::new(layout.bounds().width * (1.0 - p), 0.0)
        } else if p > 0.001 {
            push_root_translation(layout.bounds().width, p)
        } else {
            Vector::ZERO
        };
        let view = translated_viewport(layout.bounds(), viewport, d);
        let mut kids = layout.children();
        let (root_l, top_l) = (kids.next()?, kids.next()?);
        let (a, b) = tree.children.split_at_mut(1);
        if top {
            self.top
                .as_widget_mut()
                .overlay(&mut b[0], top_l, renderer, &view, translation + d)
        } else {
            self.root
                .as_widget_mut()
                .overlay(&mut a[0], root_l, renderer, &view, translation + d)
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
            self.root.as_widget().mouse_interaction(
                &tree.children[0],
                root_l,
                cursor,
                viewport,
                renderer,
            )
        } else if p >= 0.999 {
            self.top.as_widget().mouse_interaction(
                &tree.children[1],
                top_l,
                cursor,
                viewport,
                renderer,
            )
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
            self.root
                .as_widget_mut()
                .operate(&mut a[0], root_l, renderer, operation);
        }
        if p > 0.001 {
            self.top
                .as_widget_mut()
                .operate(&mut b[0], top_l, renderer, operation);
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
        let draw_root = |r: &mut iced::Renderer, view: &Rectangle| {
            self.root
                .as_widget()
                .draw(&tree.children[0], r, theme, style, root_l, cursor, view);
        };
        let draw_top = |r: &mut iced::Renderer, view: &Rectangle| {
            self.top
                .as_widget()
                .draw(&tree.children[1], r, theme, style, top_l, cursor, view);
        };
        if p <= 0.001 {
            return draw_root(renderer, viewport);
        }
        if p >= 0.999 {
            return draw_top(renderer, viewport);
        }
        // Each part in its own layer: within one layer all quads paint before all text, so the
        // old page's text would show through the new page's backing.
        let root_shift = push_root_translation(bounds.width, p);
        renderer.with_layer(bounds, |r| {
            draw_translated(r, bounds, viewport, root_shift, |r, view| {
                draw_root(r, view)
            });
        });
        // Dimming follows the actual root in its translated coordinates, not the
        // stationary stack rectangle. Unknown targets are never guessed.
        if let Some(shape) = self.root_shape.as_ref() {
            let (target_bounds, radius) = shape(theme, root_l.bounds());
            renderer.with_layer(bounds, |r| {
                draw_translated(r, bounds, viewport, root_shift, |r, _| {
                    r.fill_quad(
                        Quad {
                            bounds: target_bounds,
                            border: Border::default().rounded(radius),
                            shadow: Default::default(),
                            snap: false,
                        },
                        Background::Color(crate::color::alpha(theme.semantic.shade, p)),
                    );
                });
            });
        }
        let dx = bounds.width * (1.0 - p);
        renderer.with_layer(bounds, |r| {
            draw_translated(r, bounds, viewport, Vector::new(dx, 0.0), |r, view| {
                draw_top(r, view);
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
    paint: HighlightPaint,
    duration: Option<std::time::Duration>,
}

enum HighlightPaint {
    Background { radius: f32, alpha: f32 },
    Target(std::rc::Rc<std::cell::Cell<f32>>),
}

pub fn highlight<'a, M>(
    content: impl Into<Element<'a, M, Theme>>,
    selected: bool,
    radius: f32,
    alpha: f32,
) -> Highlight<'a, M> {
    Highlight {
        content: content.into(),
        selected,
        paint: HighlightPaint::Background { radius, alpha },
        duration: None,
    }
}

impl<'a, M: Clone + 'a> Highlight<'a, M> {
    /// Animate a button effect through its actual target style rather than drawing
    /// a second, independently shaped quad. `style` receives the native button status
    /// and current selection progress, including on the first draw and during fade-out.
    ///
    /// Resolve the target's complete style first, then apply a color-only effect to it.
    /// The button retains ownership of bounds, corners, border, shadow and input.
    pub fn button(
        content: iced::widget::Button<'a, M, Theme>,
        selected: bool,
        style: impl Fn(&Theme, iced::widget::button::Status, f32) -> iced::widget::button::Style + 'a,
    ) -> Self {
        let progress = std::rc::Rc::new(std::cell::Cell::new(if selected { 1.0 } else { 0.0 }));
        let shown = progress.clone();
        Self {
            content: content
                .style(move |theme, status| style(theme, status, shown.get()))
                .into(),
            selected,
            paint: HighlightPaint::Target(progress),
            duration: None,
        }
    }
}

impl<'a, M> Highlight<'a, M> {
    /// How long the highlight takes to move or fade (default 200 ms).
    pub fn duration(mut self, d: std::time::Duration) -> Self {
        self.duration = Some(d);
        self
    }
}

struct HighlightState {
    progress: Progress,
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for Highlight<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<HighlightState>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(HighlightState {
            progress: Progress::new(if self.selected { 1.0 } else { 0.0 }),
        })
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
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let child = self
            .content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);
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
        advance(
            &mut tree.state.downcast_mut::<HighlightState>().progress,
            event,
            if self.selected { 1.0 } else { 0.0 },
            Trigger::Selection.spec(match self.duration {
                Some(d) => Spec::STYLE.with_duration(d),
                None => Spec::STYLE,
            }),
            shell,
            false,
        );
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
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<iced::advanced::overlay::Element<'b, M, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(
            &mut tree.children[0],
            layout.children().next()?,
            renderer,
            viewport,
            translation,
        )
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
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
        let p = tree.state.downcast_ref::<HighlightState>().progress.shown;
        match &self.paint {
            HighlightPaint::Background { radius, alpha } if p > 0.001 => {
                renderer.fill_quad(
                    Quad {
                        bounds: layout.bounds(),
                        border: Border::default().rounded(*radius),
                        shadow: Default::default(),
                        snap: false,
                    },
                    Background::Color(crate::color::alpha(theme.semantic.sidebar.fg, alpha * p)),
                );
            }
            HighlightPaint::Target(progress) => progress.set(p),
            _ => {}
        }
        self.content.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.children().next().unwrap(),
            cursor,
            viewport,
        );
    }
}

impl<'a, M: 'a> From<Highlight<'a, M>> for Element<'a, M, Theme> {
    fn from(h: Highlight<'a, M>) -> Self {
        Element::new(h)
    }
}

#[cfg(test)]
mod highlight_target_tests {
    use super::*;
    use iced::advanced::{clipboard, renderer::Headless};
    use iced::widget::{button, space};
    use std::{cell::RefCell, rc::Rc};

    fn renderer() -> iced::Renderer {
        iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("software renderer")
    }

    #[test]
    fn target_callback_receives_native_status_current_theme_and_animation_progress() {
        use iced::widget::button::{Status, Style};
        let mut renderer = renderer();
        let viewport = Rectangle::with_size(Size::new(100.0, 60.0));
        let limits = layout::Limits::new(Size::ZERO, viewport.size());
        for enabled in [false, true] {
            let seen = Rc::new(RefCell::new(vec![]));
            let capture = seen.clone();
            let content = button(space())
                .width(80)
                .height(40)
                .padding(0)
                .on_press_maybe(enabled.then_some(()));
            let mut highlight = Highlight::button(content, true, move |theme, status, p| {
                capture
                    .borrow_mut()
                    .push((status, p, theme.metrics.radius_m));
                Style {
                    border: Border::default().rounded(theme.metrics.radius_m),
                    ..Default::default()
                }
            });
            let mut tree = Tree::new(&highlight as &dyn Widget<(), Theme, iced::Renderer>);
            let node = highlight.layout(&mut tree, &renderer, &limits);
            let layout = Layout::new(&node);
            let mut messages = vec![];
            for (status, cursor, press) in [
                (Status::Active, mouse::Cursor::Unavailable, false),
                (
                    Status::Hovered,
                    mouse::Cursor::Available(iced::Point::new(40.0, 20.0)),
                    false,
                ),
                (
                    Status::Pressed,
                    mouse::Cursor::Available(iced::Point::new(40.0, 20.0)),
                    true,
                ),
            ] {
                if press {
                    highlight.update(
                        &mut tree,
                        &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                        layout,
                        cursor,
                        &renderer,
                        &mut clipboard::Null,
                        &mut Shell::new(&mut messages),
                        &viewport,
                    );
                }
                highlight.update(
                    &mut tree,
                    &Event::Window(window::Event::RedrawRequested(Instant::now())),
                    layout,
                    cursor,
                    &renderer,
                    &mut clipboard::Null,
                    &mut Shell::new(&mut messages),
                    &viewport,
                );
                for radius in [0.0, 23.0] {
                    let mut theme = Theme::light();
                    theme.metrics.radius_m = radius;
                    for progress in [0.0, 0.25, 1.0, 0.5, 0.0] {
                        tree.state
                            .downcast_mut::<HighlightState>()
                            .progress
                            .set_immediate(progress);
                        renderer.reset(viewport);
                        highlight.draw(
                            &tree,
                            &mut renderer,
                            &theme,
                            &renderer::Style {
                                text_color: Color::BLACK,
                            },
                            layout,
                            cursor,
                            &viewport,
                        );
                        assert_eq!(
                            seen.borrow().last(),
                            Some(&(
                                if enabled { status } else { Status::Disabled },
                                progress,
                                radius
                            ))
                        );
                    }
                }
            }
            highlight.update(
                &mut tree,
                &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                layout,
                mouse::Cursor::Available(iced::Point::new(40.0, 20.0)),
                &renderer,
                &mut clipboard::Null,
                &mut Shell::new(&mut messages),
                &viewport,
            );
            assert_eq!(messages.len(), usize::from(enabled));
            // New views retain the original animation state and child's tree position.
            tree.state
                .downcast_mut::<HighlightState>()
                .progress
                .set_immediate(0.4);
            let replacement = Highlight::button(button(space()).on_press(()), false, |_, _, _| {
                Style::default()
            });
            tree.diff(&replacement as &dyn Widget<(), Theme, iced::Renderer>);
            assert_eq!(
                tree.state.downcast_ref::<HighlightState>().progress.shown,
                0.4
            );
            assert_eq!(tree.children.len(), 1);
        }
    }

    #[test]
    fn highlight_draws_its_explicit_radius_and_alpha() {
        let mut renderer = renderer();
        let mut theme = Theme::light();
        theme.semantic.sidebar.fg = Color::BLACK;
        let mut highlight = highlight::<()>(space().width(80).height(40), true, 20.0, 0.4);
        let mut tree = Tree::new(&highlight as &dyn Widget<(), Theme, iced::Renderer>);
        tree.state
            .downcast_mut::<HighlightState>()
            .progress
            .set_immediate(0.5);
        let viewport = Rectangle::with_size(Size::new(100.0, 60.0));
        let node = highlight.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, viewport.size()),
        );
        renderer.reset(viewport);
        highlight.draw(
            &tree,
            &mut renderer,
            &theme,
            &renderer::Style {
                text_color: Color::BLACK,
            },
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &viewport,
        );
        let pixels = renderer.screenshot(Size::new(100, 60), 1.0, Color::WHITE);
        let pixel = |x: usize, y: usize| &pixels[(y * 100 + x) * 4..(y * 100 + x) * 4 + 4];
        assert_eq!(pixel(1, 1), &[255, 255, 255, 255]);
        assert!((i16::from(pixel(40, 20)[0]) - 204).abs() <= 1);
        assert_eq!(pixel(90, 20), &[255, 255, 255, 255]);
    }
}

#[cfg(test)]
mod entrance_shape_tests {
    use super::*;
    use iced::advanced::{clipboard, renderer::Headless};
    use iced::widget::{button, container, space};

    fn renderer() -> iced::Renderer {
        iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("software renderer")
    }

    #[test]
    fn rendered_fade_uses_current_target_style_for_square_pill_and_asymmetric_corners() {
        let mut renderer = renderer();
        let viewport = Rectangle::with_size(Size::new(80.0, 40.0));
        for asymmetric in [false, true] {
            let target_style = move |theme: &Theme| iced::widget::container::Style {
                background: Some(Color::BLACK.into()),
                border: Border::default().rounded(if asymmetric {
                    iced::border::Radius {
                        top_left: theme.metrics.radius_m,
                        top_right: 0.0,
                        bottom_right: theme.metrics.radius_m,
                        bottom_left: 0.0,
                    }
                } else {
                    theme.metrics.radius_m.into()
                }),
                ..Default::default()
            };
            let mut entrance = Entrance::container(
                container(space()).width(80).height(40),
                Vector::ZERO,
                target_style,
            )
            .fade();
            let mut tree = Tree::new(&entrance as &dyn Widget<(), Theme, iced::Renderer>);
            let node = entrance.layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, viewport.size()),
            );
            // Same view/tree, different current theme: neither style nor effect may cache it.
            for radius in [0.0, 20.0] {
                let mut theme = Theme::light();
                theme.metrics.radius_m = radius;
                theme.semantic.window.bg = Color::WHITE;
                for p in [0.0, 0.25, 0.5, 1.0] {
                    tree.state
                        .downcast_mut::<EntranceState>()
                        .progress
                        .set_immediate(p);
                    renderer.reset(viewport);
                    entrance.draw(
                        &tree,
                        &mut renderer,
                        &theme,
                        &renderer::Style {
                            text_color: Color::BLACK,
                        },
                        Layout::new(&node),
                        mouse::Cursor::Unavailable,
                        &viewport,
                    );
                    let actual =
                        renderer.screenshot(Size::new(80, 40), 1.0, Color::from_rgb(0.0, 1.0, 0.0));
                    // Independent reference raster: the target surface, then a cover with its
                    // actual four corners. Green exposes any paint outside that silhouette.
                    renderer.reset(viewport);
                    let quad = Quad {
                        bounds: viewport,
                        border: target_style(&theme).border,
                        shadow: Default::default(),
                        snap: false,
                    };
                    renderer.fill_quad(quad, Color::BLACK);
                    renderer.with_layer(viewport, |r| {
                        r.fill_quad(
                            quad,
                            Color {
                                a: 1.0 - p,
                                ..Color::WHITE
                            },
                        );
                    });
                    let expected =
                        renderer.screenshot(Size::new(80, 40), 1.0, Color::from_rgb(0.0, 1.0, 0.0));
                    assert_eq!(
                        actual, expected,
                        "radius={radius}, asymmetric={asymmetric}, p={p}"
                    );
                }
            }
        }
    }

    #[test]
    fn unknown_shape_does_not_paint_a_square_mask_and_scrim_does_not_define_foreground() {
        let mut renderer = renderer();
        let viewport = Rectangle::with_size(Size::new(80.0, 40.0));
        let mut entrance = entrance::<()>(space().width(80).height(40), Vector::ZERO)
            .scrim_shape(|_, bounds| (bounds, 20.0.into()))
            .fade();
        assert!(entrance.fade_shape.is_none());
        let mut tree = Tree::new(&entrance as &dyn Widget<(), Theme, iced::Renderer>);
        tree.state
            .downcast_mut::<EntranceState>()
            .progress
            .set_immediate(0.5);
        let node = entrance.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, viewport.size()),
        );
        let mut theme = Theme::light();
        theme.semantic.shade = Color::TRANSPARENT;
        renderer.reset(viewport);
        entrance.draw(
            &tree,
            &mut renderer,
            &theme,
            &renderer::Style {
                text_color: Color::BLACK,
            },
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &viewport,
        );
        let pixels = renderer.screenshot(Size::new(80, 40), 1.0, Color::from_rgb(0.0, 1.0, 0.0));
        assert!(
            pixels
                .chunks_exact(4)
                .all(|pixel| pixel == [0, 255, 0, 255])
        );
        let entrance = entrance.scrim(7.0);
        assert!(entrance.fade_shape.is_none());
        let (bounds, radius) = entrance.scrim_shape.unwrap()(&theme, viewport);
        assert_eq!(bounds, viewport);
        assert_eq!(radius, 7.0.into());
    }

    #[test]
    fn actual_tooltip_overlay_is_forwarded_and_background_scrim_keeps_target_geometry() {
        let renderer = renderer();
        let viewport = Rectangle::with_size(Size::new(80.0, 40.0));
        let content = iced::widget::tooltip(
            space().width(80).height(40),
            iced::widget::text("tooltip"),
            iced::widget::tooltip::Position::Top,
        );
        let mut entrance = Entrance::container(container(content), Vector::ZERO, |_| {
            iced::widget::container::Style {
                border: Border::default().rounded(iced::border::Radius {
                    top_left: 20.0,
                    top_right: 0.0,
                    bottom_right: 20.0,
                    bottom_left: 0.0,
                }),
                ..Default::default()
            }
        })
        .fade()
        .scrim(3.0);
        let theme = Theme::light();
        assert_eq!(
            entrance.fade_shape.as_ref().unwrap()(&theme, viewport).1,
            iced::border::Radius {
                top_left: 20.0,
                top_right: 0.0,
                bottom_right: 20.0,
                bottom_left: 0.0,
            }
        );
        let mut tree = Tree::new(&entrance as &dyn Widget<(), Theme, iced::Renderer>);
        let node = entrance.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, viewport.size()),
        );
        tree.state
            .downcast_mut::<EntranceState>()
            .progress
            .set_immediate(1.0);
        entrance.update(
            &mut tree,
            &Event::Mouse(mouse::Event::CursorMoved {
                position: iced::Point::new(40.0, 20.0),
            }),
            Layout::new(&node),
            mouse::Cursor::Available(iced::Point::new(40.0, 20.0)),
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut vec![]),
            &viewport,
        );
        for p in [1.0, 0.5, 0.0] {
            tree.state
                .downcast_mut::<EntranceState>()
                .progress
                .set_immediate(p);
            assert!(
                entrance
                    .overlay(
                        &mut tree,
                        Layout::new(&node),
                        &renderer,
                        &viewport,
                        Vector::new(5.0, 7.0)
                    )
                    .is_some()
            );
        }
    }

    #[test]
    fn shaped_entrance_preserves_settled_button_events_and_blocks_intermediate_clicks() {
        let renderer = renderer();
        let viewport = Rectangle::with_size(Size::new(80.0, 40.0));
        let mut entrance = Entrance::container(
            container(button(space()).width(80).height(40).on_press(42)),
            Vector::ZERO,
            |theme| iced::widget::container::Style {
                border: Border::default().rounded(theme.metrics.radius_m),
                ..Default::default()
            },
        )
        .fade();
        let mut tree = Tree::new(&entrance as &dyn Widget<i32, Theme, iced::Renderer>);
        let node = entrance.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, viewport.size()),
        );
        for p in [0.5, 1.0] {
            tree.state
                .downcast_mut::<EntranceState>()
                .progress
                .set_immediate(p);
            let mut messages = vec![];
            for event in [
                mouse::Event::ButtonPressed(mouse::Button::Left),
                mouse::Event::ButtonReleased(mouse::Button::Left),
            ] {
                let mut shell = Shell::new(&mut messages);
                entrance.update(
                    &mut tree,
                    &Event::Mouse(event),
                    Layout::new(&node),
                    mouse::Cursor::Available(iced::Point::new(40.0, 20.0)),
                    &renderer,
                    &mut clipboard::Null,
                    &mut shell,
                    &viewport,
                );
                assert!(shell.is_event_captured());
            }
            assert_eq!(messages, if p == 1.0 { vec![42] } else { vec![] });
        }
    }
}

#[cfg(test)]
mod remaining_effect_shape_tests {
    use super::*;
    use iced::advanced::{clipboard, renderer::Headless};
    use iced::widget::{button, container, space, tooltip};

    fn renderer() -> iced::Renderer {
        iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("software renderer")
    }

    fn viewport() -> Rectangle {
        Rectangle::with_size(Size::new(120.0, 60.0))
    }
    fn pixels(renderer: &mut iced::Renderer) -> Vec<u8> {
        renderer.screenshot(Size::new(120, 60), 1.0, Color::from_rgb(0.0, 1.0, 0.0))
    }
    fn radius(theme: &Theme, asymmetric: bool) -> iced::border::Radius {
        if asymmetric {
            iced::border::Radius {
                top_left: theme.metrics.radius_m,
                bottom_right: theme.metrics.radius_m,
                ..Default::default()
            }
        } else {
            theme.metrics.radius_m.into()
        }
    }
    fn surface(theme: &Theme, asymmetric: bool, top: bool) -> iced::widget::container::Style {
        iced::widget::container::Style {
            background: Some(if top { Color::BLACK } else { Color::WHITE }.into()),
            border: Border::default().rounded(radius(theme, asymmetric)),
            shadow: if top {
                theme.shadows.dialog().shadow
            } else {
                Default::default()
            },
            ..Default::default()
        }
    }
    fn quad(bounds: Rectangle, radius: iced::border::Radius) -> Quad {
        Quad {
            bounds,
            border: Border::default().rounded(radius),
            shadow: Default::default(),
            snap: false,
        }
    }
    fn draw<M>(
        widget: &dyn Widget<M, Theme, iced::Renderer>,
        tree: &Tree,
        node: &layout::Node,
        renderer: &mut iced::Renderer,
        theme: &Theme,
    ) -> Vec<u8> {
        renderer.reset(viewport());
        widget.draw(
            tree,
            renderer,
            theme,
            &renderer::Style {
                text_color: Color::BLACK,
            },
            Layout::new(node),
            mouse::Cursor::Unavailable,
            &viewport(),
        );
        pixels(renderer)
    }

    #[test]
    fn scoped_background_scrims_use_nearest_actual_surface_without_leaking() {
        let mut renderer = renderer();
        let mut theme = Theme::light();
        theme.semantic.shade = Color {
            a: 0.3,
            ..Color::BLACK
        };
        let outer = SurfaceShape::new(viewport(), 0.0);
        for corners in [
            0.0.into(),
            25.0.into(),
            iced::border::Radius {
                top_left: 25.0,
                bottom_right: 12.0,
                ..Default::default()
            },
        ] {
            let target = SurfaceShape::new(
                Rectangle {
                    x: 10.0,
                    y: 5.0,
                    width: 100.0,
                    height: 50.0,
                },
                corners,
            );
            for mut widget in [
                Element::from(drawer_layer::<()>(space(), true, 20.0)),
                Element::from(entrance::<()>(space(), Vector::ZERO).surface_scrim().fade()),
            ] {
                let mut tree = Tree::new(widget.as_widget());
                let node = widget.as_widget_mut().layout(
                    &mut tree,
                    &renderer,
                    &layout::Limits::new(Size::ZERO, viewport().size()),
                );
                // Both animations use the same intermediate opacity; the empty
                // foreground must not acquire a window-color fade mask.
                if tree.tag == tree::Tag::of::<DrawerState>() {
                    tree.state
                        .downcast_mut::<DrawerState>()
                        .progress
                        .set_immediate(0.5);
                } else {
                    tree.state
                        .downcast_mut::<EntranceState>()
                        .progress
                        .set_immediate(0.5);
                }
                let actual = boundary::with_surface(outer, || {
                    boundary::with_surface(target, || {
                        draw(widget.as_widget(), &tree, &node, &mut renderer, &theme)
                    })
                });
                renderer.reset(viewport());
                renderer.fill_quad(quad(target.bounds, target.radius), scrim(&theme, 0.5));
                assert_eq!(actual, pixels(&mut renderer));
                assert!(boundary::current_surface().is_none());
                let unscoped = draw(widget.as_widget(), &tree, &node, &mut renderer, &theme);
                renderer.reset(viewport());
                assert_eq!(unscoped, pixels(&mut renderer));
            }
        }
    }

    #[test]
    fn parent_surface_never_supplies_arbitrary_foreground_masks_or_enables_entrance_scrims() {
        let mut renderer = renderer();
        let theme = Theme::light();
        let parent = SurfaceShape::new(viewport(), 30.0);
        for mut widget in [
            Element::from(
                entrance::<()>(
                    container(space()).width(40).height(20).style(|_| {
                        iced::widget::container::Style {
                            background: Some(Color::WHITE.into()),
                            ..Default::default()
                        }
                    }),
                    Vector::ZERO,
                )
                .fade(),
            ),
            Element::from(page_transition::<()>(
                container(space())
                    .width(40)
                    .height(20)
                    .style(|_| iced::widget::container::Style {
                        background: Some(Color::WHITE.into()),
                        ..Default::default()
                    }),
                0_u64,
            )),
        ] {
            let mut tree = Tree::new(widget.as_widget());
            let node = widget.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, viewport().size()),
            );
            if tree.tag == tree::Tag::of::<EntranceState>() {
                tree.state
                    .downcast_mut::<EntranceState>()
                    .progress
                    .set_immediate(0.5);
            } else {
                tree.state
                    .downcast_mut::<PageState>()
                    .progress
                    .set_immediate(0.5);
            }
            let baseline = draw(widget.as_widget(), &tree, &node, &mut renderer, &theme);
            let scoped = boundary::with_surface(parent, || {
                draw(widget.as_widget(), &tree, &node, &mut renderer, &theme)
            });
            assert_eq!(scoped, baseline);
        }
    }

    #[test]
    fn translated_draw_rebases_all_ancestors_and_restores_nested_scopes() {
        let mut renderer = renderer();
        let outer = SurfaceShape::new(viewport(), 10.0);
        let inner = SurfaceShape::new(
            Rectangle {
                x: 10.0,
                y: 5.0,
                width: 100.0,
                height: 50.0,
            },
            20.0,
        );
        boundary::with_surface(outer, || {
            boundary::with_surface(inner, || {
                draw_translated(
                    &mut renderer,
                    viewport(),
                    &viewport(),
                    Vector::new(17.0, -3.0),
                    |r, _| {
                        let shapes = boundary::current_boundary().shapes;
                        assert_eq!(shapes.len(), 2);
                        assert_eq!(shapes[0].bounds.x, outer.bounds.x - 17.0);
                        assert_eq!(shapes[1].bounds.y, inner.bounds.y + 3.0);
                        assert_eq!(shapes[1].radius, inner.radius);
                        draw_translated(
                            r,
                            viewport(),
                            &viewport(),
                            Vector::new(-4.0, 8.0),
                            |_, _| {
                                let nearest = boundary::current_surface().unwrap();
                                assert_eq!(nearest.bounds.x, inner.bounds.x - 13.0);
                                assert_eq!(nearest.bounds.y, inner.bounds.y - 5.0);
                                boundary::with_portal(|| {
                                    assert!(boundary::current_surface().is_none())
                                });
                            },
                        );
                        assert_eq!(boundary::current_boundary().shapes, shapes);
                    },
                );
                assert_eq!(boundary::current_boundary().shapes, vec![outer, inner]);
                let panic = std::panic::catch_unwind(|| {
                    with_translated_surfaces(Vector::new(2.0, 3.0), || panic!("scope probe"))
                });
                assert!(panic.is_err());
                assert_eq!(boundary::current_boundary().shapes, vec![outer, inner]);
            })
        });
        assert!(boundary::current_surface().is_none());
    }

    #[test]
    fn translated_scoped_scrim_stays_stationary_in_renderer_space() {
        let mut renderer = renderer();
        let theme = Theme::light();
        let target = SurfaceShape::new(
            Rectangle {
                x: 20.0,
                y: 10.0,
                width: 80.0,
                height: 40.0,
            },
            20.0,
        );
        renderer.reset(viewport());
        boundary::with_surface(target, || {
            draw_translated(
                &mut renderer,
                viewport(),
                &viewport(),
                Vector::new(13.0, 7.0),
                |r, _| {
                    let shape = background_shape(None, &theme, viewport()).unwrap();
                    r.fill_quad(quad(shape.bounds, shape.radius), scrim(&theme, 0.5));
                },
            );
        });
        let actual = pixels(&mut renderer);
        renderer.reset(viewport());
        renderer.fill_quad(quad(target.bounds, target.radius), scrim(&theme, 0.5));
        assert_eq!(actual, pixels(&mut renderer));
        let explicit: EffectShape<'_> = std::rc::Rc::new(|_, bounds| (bounds, 0.0.into()));
        boundary::with_surface(target, || {
            assert_eq!(
                background_shape(Some(&explicit), &theme, viewport()),
                Some(SurfaceShape::new(viewport(), 0.0))
            );
        });
    }

    #[test]
    fn rendered_page_cover_matches_actual_square_pill_asymmetric_target_and_current_theme() {
        let mut renderer = renderer();
        for asymmetric in [false, true] {
            let mut page = PageTransition::container(
                container(space()).width(120).height(60),
                0_u64,
                move |t| surface(t, asymmetric, false),
            );
            let mut tree = Tree::new(&page as &dyn Widget<(), Theme, iced::Renderer>);
            let node = page.layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, viewport().size()),
            );
            for r in [0.0, 30.0] {
                let mut theme = Theme::light();
                theme.metrics.radius_m = r;
                theme.semantic.window.bg = Color::BLACK;
                for p in [0.0, 0.25, 0.5, 1.0] {
                    tree.state
                        .downcast_mut::<PageState>()
                        .progress
                        .set_immediate(p);
                    let actual = draw(&page, &tree, &node, &mut renderer, &theme);
                    renderer.reset(viewport());
                    renderer.fill_quad(quad(viewport(), radius(&theme, asymmetric)), Color::WHITE);
                    renderer.with_layer(viewport(), |r| {
                        r.fill_quad(
                            quad(viewport(), radius(&theme, asymmetric)),
                            Color {
                                a: 1.0 - p,
                                ..Color::BLACK
                            },
                        )
                    });
                    assert_eq!(
                        actual,
                        pixels(&mut renderer),
                        "r={r}, asymmetric={asymmetric}, p={p}"
                    );
                }
            }
        }
    }

    #[test]
    fn rendered_drawer_scrim_uses_exact_background_bounds_and_current_corners() {
        let mut renderer = renderer();
        for asymmetric in [false, true] {
            let mut drawer =
                drawer_layer::<()>(space(), true, 20.0).scrim_shape(move |theme, bounds| {
                    (
                        Rectangle {
                            x: bounds.x + 10.0,
                            y: bounds.y + 5.0,
                            width: bounds.width - 20.0,
                            height: bounds.height - 10.0,
                        },
                        radius(theme, asymmetric),
                    )
                });
            let mut tree = Tree::new(&drawer as &dyn Widget<(), Theme, iced::Renderer>);
            let node = drawer.layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, viewport().size()),
            );
            for r in [0.0, 25.0] {
                let mut theme = Theme::light();
                theme.metrics.radius_m = r;
                theme.semantic.shade = Color {
                    a: 0.3,
                    ..Color::BLACK
                };
                for p in [0.0, 0.25, 0.5, 1.0] {
                    tree.state
                        .downcast_mut::<DrawerState>()
                        .progress
                        .set_immediate(p);
                    let actual = draw(&drawer, &tree, &node, &mut renderer, &theme);
                    renderer.reset(viewport());
                    renderer.fill_quad(
                        quad(
                            Rectangle {
                                x: 10.0,
                                y: 5.0,
                                width: 100.0,
                                height: 50.0,
                            },
                            radius(&theme, asymmetric),
                        ),
                        Color {
                            a: 0.6 * p,
                            ..Color::BLACK
                        },
                    );
                    assert_eq!(
                        actual,
                        pixels(&mut renderer),
                        "r={r}, asymmetric={asymmetric}, p={p}"
                    );
                }
            }
        }
    }

    #[test]
    fn rendered_push_dimmer_tracks_root_and_native_top_owns_backing_corners_and_shadow() {
        let mut renderer = renderer();
        for asymmetric in [false, true] {
            let mut stack = PushStack::containers(
                container(space()),
                container(space()),
                true,
                move |t| surface(t, asymmetric, false),
                move |t| surface(t, !asymmetric, true),
            );

            let mut tree = Tree::new(&stack as &dyn Widget<(), Theme, iced::Renderer>);
            let node = stack.layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, viewport().size()),
            );
            for r in [0.0, 30.0] {
                let mut theme = Theme::light();
                theme.metrics.radius_m = r;
                theme.semantic.shade = Color {
                    a: 0.4,
                    ..Color::BLACK
                };
                theme.shadows.dialog =
                    crate::shadow::Elevation::parse("-4px 0px 6px 0px rgba(0,0,0,0.6)").unwrap();
                for p in [0.0, 0.25, 0.5, 0.75, 1.0] {
                    tree.state
                        .downcast_mut::<PushState>()
                        .progress
                        .set_immediate(p);
                    let actual = draw(&stack, &tree, &node, &mut renderer, &theme);
                    renderer.reset(viewport());
                    let root = quad(viewport(), radius(&theme, asymmetric));
                    let top = Quad {
                        shadow: theme.shadows.dialog().shadow,
                        ..quad(viewport(), radius(&theme, !asymmetric))
                    };
                    if p == 0.0 {
                        renderer.fill_quad(root, Color::WHITE);
                    } else if p == 1.0 {
                        renderer.fill_quad(top, Color::BLACK);
                    } else {
                        let shift = Vector::new(-40.0 * p, 0.0);
                        renderer.with_layer(viewport(), |r| {
                            r.with_translation(shift, |r| r.fill_quad(root, Color::WHITE))
                        });
                        renderer.with_layer(viewport(), |r| {
                            r.with_translation(shift, |r| {
                                r.fill_quad(
                                    root,
                                    Color {
                                        a: 0.4 * p,
                                        ..Color::BLACK
                                    },
                                )
                            })
                        });
                        renderer.with_layer(viewport(), |r| {
                            r.with_translation(Vector::new(120.0 * (1.0 - p), 0.0), |r| {
                                r.fill_quad(top, Color::BLACK)
                            })
                        });
                    }
                    assert_eq!(
                        actual,
                        pixels(&mut renderer),
                        "r={r}, asymmetric={asymmetric}, p={p}"
                    );
                }
            }
        }
    }

    #[test]
    fn unknown_targets_are_unmasked_and_drawer_radius_is_explicit() {
        let mut renderer = renderer();
        let mut widgets: Vec<Element<'_, (), Theme>> = vec![
            page_transition(space().width(120).height(60), 0_u64).into(),
            drawer_layer(space(), true, 20.0).into(),
            push_stack(space(), space(), true).into(),
        ];
        for (index, widget) in widgets.iter_mut().enumerate() {
            let mut tree = Tree::new(&*widget);
            let node = widget.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, viewport().size()),
            );
            match index {
                0 => tree
                    .state
                    .downcast_mut::<PageState>()
                    .progress
                    .set_immediate(0.5),
                1 => tree
                    .state
                    .downcast_mut::<DrawerState>()
                    .progress
                    .set_immediate(0.5),
                _ => tree
                    .state
                    .downcast_mut::<PushState>()
                    .progress
                    .set_immediate(0.5),
            }
            let actual = draw(
                widget.as_widget(),
                &tree,
                &node,
                &mut renderer,
                &Theme::light(),
            );
            assert!(actual.chunks_exact(4).all(|p| p == [0, 255, 0, 255]));
        }
        let theme = Theme::light();
        let drawer = drawer_layer::<()>(space(), true, 20.0).window_radius(12.0);
        assert_eq!(
            drawer.scrim_shape.unwrap()(&theme, viewport()),
            (viewport(), 12.0.into())
        );
    }

    #[test]
    fn explicit_drawer_radius_and_root_shape_render_with_current_theme_shade() {
        let mut renderer = renderer();
        let mut theme = Theme::light();
        theme.semantic.shade = Color {
            a: 0.4,
            ..Color::BLACK
        };

        let mut drawer = drawer_layer::<()>(space(), true, 20.0).window_radius(25.0);
        let mut tree = Tree::new(&drawer as &dyn Widget<(), Theme, iced::Renderer>);
        let node = drawer.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, viewport().size()),
        );
        tree.state
            .downcast_mut::<DrawerState>()
            .progress
            .set_immediate(0.5);
        let actual = draw(&drawer, &tree, &node, &mut renderer, &theme);
        renderer.reset(viewport());
        renderer.fill_quad(
            quad(viewport(), 25.0.into()),
            Color {
                a: 0.4,
                ..Color::BLACK
            },
        );
        assert_eq!(actual, pixels(&mut renderer));

        let mut stack = push_stack::<()>(space(), space(), true).root_shape(|_, b| {
            (
                b,
                iced::border::Radius {
                    top_left: 30.0,
                    ..Default::default()
                },
            )
        });
        let mut tree = Tree::new(&stack as &dyn Widget<(), Theme, iced::Renderer>);
        let node = stack.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, viewport().size()),
        );
        tree.state
            .downcast_mut::<PushState>()
            .progress
            .set_immediate(0.5);
        let actual = draw(&stack, &tree, &node, &mut renderer, &theme);
        renderer.reset(viewport());
        renderer.with_layer(viewport(), |r| {
            r.with_translation(Vector::new(-20.0, 0.0), |r| {
                r.fill_quad(
                    quad(
                        viewport(),
                        iced::border::Radius {
                            top_left: 30.0,
                            ..Default::default()
                        },
                    ),
                    Color {
                        a: 0.2,
                        ..Color::BLACK
                    },
                );
            })
        });

        assert_eq!(actual, pixels(&mut renderer));
    }

    struct OverlayProbe(std::rc::Rc<std::cell::Cell<(Vector, Rectangle)>>);
    impl Widget<(), Theme, iced::Renderer> for OverlayProbe {
        fn size(&self) -> Size<Length> {
            Size::new(Length::Fill, Length::Fill)
        }
        fn layout(
            &mut self,
            _: &mut Tree,
            _: &iced::Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            layout::Node::new(limits.max())
        }
        fn draw(
            &self,
            _: &Tree,
            _: &mut iced::Renderer,
            _: &Theme,
            _: &renderer::Style,
            _: Layout<'_>,
            _: mouse::Cursor,
            _: &Rectangle,
        ) {
        }
        fn overlay<'b>(
            &'b mut self,
            _: &'b mut Tree,
            _: Layout<'b>,
            _: &iced::Renderer,
            viewport: &Rectangle,
            translation: Vector,
        ) -> Option<iced::advanced::overlay::Element<'b, (), Theme, iced::Renderer>> {
            self.0.set((translation, *viewport));
            None
        }
    }

    #[test]
    fn drawer_and_push_overlay_translation_matches_visible_target_including_overshoot() {
        let renderer = renderer();
        let seen = std::rc::Rc::new(std::cell::Cell::new((Vector::ZERO, Rectangle::default())));
        let probe = || Element::new(OverlayProbe(seen.clone()));
        let mut drawer = drawer_layer(probe(), true, 40.0);
        let mut tree = Tree::new(&drawer as &dyn Widget<(), Theme, iced::Renderer>);
        let node = drawer.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, viewport().size()),
        );
        tree.state
            .downcast_mut::<DrawerState>()
            .progress
            .set_immediate(0.5);
        drawer.overlay(
            &mut tree,
            Layout::new(&node),
            &renderer,
            &viewport(),
            Vector::new(5.0, 7.0),
        );
        assert_eq!(
            seen.get(),
            (
                Vector::new(-15.0, 7.0),
                Rectangle {
                    x: 20.0,
                    ..viewport()
                }
            )
        );

        let mut stack = push_stack(probe(), probe(), true);
        let mut tree = Tree::new(&stack as &dyn Widget<(), Theme, iced::Renderer>);
        let node = stack.layout(
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, viewport().size()),
        );
        for (p, dx) in [
            (0.0, 0.0),
            (0.25, -10.0),
            (0.75, 30.0),
            (1.0, 0.0),
            (1.1, 0.0),
        ] {
            tree.state
                .downcast_mut::<PushState>()
                .progress
                .set_immediate(p);
            stack.overlay(
                &mut tree,
                Layout::new(&node),
                &renderer,
                &viewport(),
                Vector::new(5.0, 7.0),
            );
            assert_eq!(
                seen.get(),
                (
                    Vector::new(5.0 + dx, 7.0),
                    Rectangle {
                        x: -dx,
                        ..viewport()
                    }
                ),
                "p={p}"
            );
        }
    }

    fn interactive(message: i32) -> Element<'static, i32, Theme> {
        tooltip(
            button(space()).width(120).height(60).on_press(message),
            iced::widget::text("tip"),
            tooltip::Position::Top,
        )
        .into()
    }

    #[test]
    fn actual_buttons_and_tooltip_overlays_keep_layout_and_forwarding_rules() {
        let renderer = renderer();
        let mut widgets: Vec<Element<'_, i32, Theme>> = vec![
            page_transition(interactive(1), 0_u64).into(),
            drawer_layer(interactive(2), true, 120.0)
                .scrim_shape(|_, b| (b, 30.0.into()))
                .into(),
            push_stack(interactive(3), interactive(4), false).into(),
        ];
        for (index, widget) in widgets.iter_mut().enumerate() {
            let mut tree = Tree::new(&*widget);
            let node = widget.as_widget_mut().layout(
                &mut tree,
                &renderer,
                &layout::Limits::new(Size::ZERO, viewport().size()),
            );
            assert_eq!(node.size(), viewport().size());
            for p in [0.0, 1.0, 0.25, 0.75] {
                match index {
                    0 => tree
                        .state
                        .downcast_mut::<PageState>()
                        .progress
                        .set_immediate(p),
                    1 => tree
                        .state
                        .downcast_mut::<DrawerState>()
                        .progress
                        .set_immediate(p),
                    _ => tree
                        .state
                        .downcast_mut::<PushState>()
                        .progress
                        .set_immediate(p),
                }
                let mut messages = vec![];
                for event in [
                    mouse::Event::CursorMoved {
                        position: iced::Point::new(60.0, 30.0),
                    },
                    mouse::Event::ButtonPressed(mouse::Button::Left),
                    mouse::Event::ButtonReleased(mouse::Button::Left),
                ] {
                    widget.as_widget_mut().update(
                        &mut tree,
                        &Event::Mouse(event),
                        Layout::new(&node),
                        mouse::Cursor::Available(iced::Point::new(60.0, 30.0)),
                        &renderer,
                        &mut clipboard::Null,
                        &mut Shell::new(&mut messages),
                        &viewport(),
                    );
                }
                let expected = match index {
                    0 => vec![1],
                    1 if p == 1.0 => vec![2],
                    2 if p == 0.0 => vec![3],
                    2 if p == 1.0 => vec![4],
                    _ => vec![],
                };
                assert_eq!(messages, expected, "family={index}, p={p}");
                // At this point each shown target has opened a real native tooltip.
                if index != 1 || p > 0.0 {
                    assert!(
                        widget
                            .as_widget_mut()
                            .overlay(
                                &mut tree,
                                Layout::new(&node),
                                &renderer,
                                &viewport(),
                                Vector::new(5.0, 7.0)
                            )
                            .is_some(),
                        "family={index}, p={p}"
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod fade_tests {
    use super::*;

    #[test]
    fn a_fading_layer_starts_covered_and_ends_clear() {
        assert_eq!(fade_cover(0.0), 1.0);
        assert_eq!(fade_cover(1.0), 0.0);
        assert!((fade_cover(0.25) - 0.75).abs() < 1e-6);
        // A spring can overshoot; the cover never goes outside 0..=1.
        assert_eq!(fade_cover(1.08), 0.0);
        assert_eq!(fade_cover(-0.1), 1.0);
    }
}

#[cfg(test)]
mod layered_tests {
    use super::*;
    use iced::advanced::widget::Tree;

    #[test]
    fn content_keeps_its_tree_position_with_or_without_a_layer() {
        let bare: iced::Element<'_, (), Theme> = layered(iced::widget::text("content"), None);
        let over: iced::Element<'_, (), Theme> = layered(
            iced::widget::text("content"),
            Some(iced::widget::text("layer").into()),
        );
        let (a, b) = (Tree::new(&bare), Tree::new(&over));
        // Same shape: the content is child 0 either way, so its state survives showing
        // or hiding the layer and it never moves.
        assert_eq!(a.children.len(), 2);
        assert_eq!(b.children.len(), 2);
        assert_eq!(a.children[0].tag, b.children[0].tag);
    }
}

#[cfg(test)]
mod viewport_tests {
    use super::*;
    use iced::Point;

    #[test]
    fn a_translated_child_clips_to_the_visible_part_in_its_own_coordinates() {
        let bounds = Rectangle::new(Point::new(100.0, 50.0), Size::new(300.0, 400.0));
        let window = Rectangle::new(Point::ORIGIN, Size::new(1000.0, 800.0));
        // A page 120 px in from the end edge: the child is drawn 120 px to the right, so it may
        // paint its own x range [bounds.x - 120, bounds.x + width - 120).
        let v = translated_viewport(bounds, &window, Vector::new(120.0, 0.0));
        assert_eq!(
            v,
            Rectangle::new(Point::new(-20.0, 50.0), Size::new(300.0, 400.0))
        );
        // Shifted back left (the root under a push): the other way.
        let v = translated_viewport(bounds, &window, Vector::new(-100.0, 0.0));
        assert_eq!(v.x, 200.0);
        // Never wider than what is on screen.
        let narrow = Rectangle::new(Point::new(150.0, 0.0), Size::new(100.0, 800.0));
        let v = translated_viewport(bounds, &narrow, Vector::ZERO);
        assert_eq!((v.x, v.width), (150.0, 100.0));
    }
}

#[cfg(test)]
mod reveal_clip_tests {
    use super::*;
    use iced::advanced::renderer::Headless;
    use std::{cell::Cell, rc::Rc};

    // A descendant opening its own render layer, as scrollables and nested reveals do.
    struct NestedLayer(Rc<Cell<Option<Rectangle>>>);
    impl Widget<(), Theme, iced::Renderer> for NestedLayer {
        fn size(&self) -> Size<Length> {
            Size::new(Length::Fill, Length::Fixed(100.0))
        }
        fn layout(
            &mut self,
            _: &mut Tree,
            _: &iced::Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            layout::Node::new(Size::new(limits.max().width, 100.0))
        }
        fn update(
            &mut self,
            _: &mut Tree,
            _: &Event,
            _: Layout<'_>,
            _: mouse::Cursor,
            _: &iced::Renderer,
            _: &mut dyn Clipboard,
            _: &mut Shell<'_, ()>,
            _: &Rectangle,
        ) {
        }
        fn draw(
            &self,
            _: &Tree,
            renderer: &mut iced::Renderer,
            _: &Theme,
            _: &renderer::Style,
            _: Layout<'_>,
            _: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            self.0.set(Some(*viewport));
            renderer.with_layer(*viewport, |r| {
                r.fill_quad(
                    Quad {
                        bounds: *viewport,
                        ..Default::default()
                    },
                    Color::from_rgb(0.5, 0.5, 0.5),
                )
            });
        }
    }

    #[test]
    fn reveal_confines_nested_layers_to_its_visible_area_while_opening_and_closing() {
        let mut renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let seen = Rc::new(Cell::new(None));
        let child: Element<'_, (), Theme> = Element::new(NestedLayer(seen.clone()));
        let mut reveal = reveal(child, false);
        let mut tree = Tree::new(&reveal as &dyn Widget<(), Theme, iced::Renderer>);
        let window = Rectangle::new(iced::Point::ORIGIN, Size::new(320.0, 240.0));
        for progress in [0.25, 0.5, 1.0, 0.5, 0.25, 0.0] {
            tree.state
                .downcast_mut::<RevealState>()
                .progress
                .set_immediate(progress);
            let node = reveal
                .layout(
                    &mut tree,
                    &renderer,
                    &layout::Limits::new(Size::ZERO, Size::new(160.0, 240.0)),
                )
                .move_to(iced::Point::new(80.0, 60.0));
            let layout = Layout::new(&node);
            renderer.reset(window);
            seen.set(None);
            reveal.draw(
                &tree,
                &mut renderer,
                &Theme::light(),
                &renderer::Style {
                    text_color: Color::BLACK,
                },
                layout,
                mouse::Cursor::Unavailable,
                &window,
            );
            if progress > 0.0 {
                assert_eq!(
                    seen.get(),
                    Some(layout.bounds()),
                    "descendants must receive the reveal's clipped viewport"
                );
            } else {
                assert_eq!(seen.get(), None);
            }
            let pixels = renderer.screenshot(Size::new(320, 240), 1.0, Color::WHITE);
            // Sidebar/top bar and the space below the disclosure must stay untouched.
            for (x, y) in [(20, 100), (120, 20), (120, 200), (300, 100)] {
                let offset = (y * 320 + x) * 4;
                assert_eq!(
                    &pixels[offset..offset + 4],
                    &[255, 255, 255, 255],
                    "outside reveal at {progress}"
                );
            }
        }
    }
}

#[cfg(test)]
mod exit_tests {
    use super::*;
    use crate::layouts::animation::Easing;
    use iced::advanced::renderer::Headless;
    use iced::advanced::{clipboard, layout};
    use iced::widget::space;
    use std::time::Duration;

    /// Drives `entrance` through `tree` at `now`, returning the messages it sent.
    fn drive(entrance: &mut Entrance<'_, u8>, tree: &mut Tree, now: Instant) -> Vec<u8> {
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("software renderer");
        let viewport = Rectangle::with_size(Size::new(100.0, 60.0));
        let node = Widget::layout(
            entrance,
            tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, viewport.size()),
        );
        let mut messages = vec![];
        Widget::update(
            entrance,
            tree,
            &Event::Window(window::Event::RedrawRequested(now)),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
            &viewport,
        );
        messages
    }

    fn make(open: bool) -> Entrance<'static, u8> {
        entrance(space().width(20).height(20), Vector::new(0.0, 30.0))
            .spec(Spec::Timed(Duration::from_millis(100), Easing::Linear))
            .open(open)
            .on_settled(7)
    }

    #[test]
    fn the_way_out_runs_and_reports_once_when_it_has_finished() {
        let t0 = Instant::now();
        let mut shown = make(true);
        let mut tree = Tree::new(&shown as &dyn Widget<u8, Theme, iced::Renderer>);
        drive(&mut shown, &mut tree, t0);
        drive(&mut shown, &mut tree, t0 + Duration::from_millis(500));
        assert!(!tree.state.downcast_ref::<EntranceState>().gone());
        // Closed: the same widget in the same place, now heading out.
        let mut leaving = make(false);
        tree.diff(&leaving as &dyn Widget<u8, Theme, iced::Renderer>);
        let first = drive(&mut leaving, &mut tree, t0 + Duration::from_millis(600));
        assert!(first.is_empty(), "not finished on its first frame");
        let state = |tree: &Tree| tree.state.downcast_ref::<EntranceState>().progress.shown;
        assert!(state(&tree) > 0.0, "still on its way out");
        let done = drive(&mut leaving, &mut tree, t0 + Duration::from_millis(900));
        assert_eq!(done, vec![7], "reported when it has gone");
        assert!(tree.state.downcast_ref::<EntranceState>().gone());
        assert!(drive(&mut leaving, &mut tree, t0 + Duration::from_millis(1000)).is_empty());
        // Opened again, it comes back and can report the next exit.
        let mut back = make(true);
        tree.diff(&back as &dyn Widget<u8, Theme, iced::Renderer>);
        drive(&mut back, &mut tree, t0 + Duration::from_millis(1100));
        drive(&mut back, &mut tree, t0 + Duration::from_millis(1500));
        assert!(!tree.state.downcast_ref::<EntranceState>().gone());
    }

    #[test]
    fn a_none_effect_leaves_at_once() {
        let t0 = Instant::now();
        let mut shown = make(true).effects(Effect::None, Effect::None);
        let mut tree = Tree::new(&shown as &dyn Widget<u8, Theme, iced::Renderer>);
        drive(&mut shown, &mut tree, t0);
        let mut leaving = make(false).effects(Effect::None, Effect::None);
        tree.diff(&leaving as &dyn Widget<u8, Theme, iced::Renderer>);
        let sent = drive(&mut leaving, &mut tree, t0 + Duration::from_millis(10));
        assert_eq!(sent, vec![7]);
    }
}

#[cfg(test)]
mod fade_cover_tests {
    use super::*;
    use iced::advanced::renderer::Headless;
    use iced::advanced::{layout, renderer};
    use iced::widget::{container, space};

    /// The centre pixel of `FadeCover` around a red square, with its progress set to `p`.
    fn centre(p: f32, effect: Effect, open: bool) -> [u8; 4] {
        centre_over(p, effect, open, None)
    }

    fn centre_over(p: f32, effect: Effect, open: bool, backdrop: Option<Color>) -> [u8; 4] {
        let mut renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("software renderer");
        let size = Size::new(40.0, 40.0);
        let red = Color::from_rgb(1.0, 0.0, 0.0);
        let surface = container(space().width(40).height(40)).style(move |_| {
            container::Style {
                background: Some(Background::Color(red)),
                ..Default::default()
            }
        });
        let mut cover: FadeCover<'_, ()> = FadeCover::new(surface, open).effects(effect, effect);
        if let Some(color) = backdrop {
            cover = cover.backdrop(move |_| color);
        }
        let mut tree = Tree::new(&cover as &dyn Widget<(), Theme, iced::Renderer>);
        tree.state
            .downcast_mut::<FadeCoverState>()
            .progress
            .set_immediate(p);
        let node = Widget::layout(
            &mut cover,
            &mut tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, size),
        );
        let theme = Theme::light();
        let viewport = Rectangle::with_size(size);
        renderer.reset(viewport);
        Widget::draw(
            &cover,
            &tree,
            &mut renderer,
            &theme,
            &renderer::Style {
                text_color: Color::BLACK,
            },
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &viewport,
        );
        let px = renderer.screenshot(
            iced::Size::new(40, 40),
            1.0,
            Color::WHITE,
        );
        let at = ((20 * 40 + 20) * 4) as usize;
        [px[at], px[at + 1], px[at + 2], px[at + 3]]
    }

    #[test]
    fn the_surface_fades_to_its_backdrop_fill_and_contents_alike() {
        let green = Color::from_rgb(0.0, 1.0, 0.0);
        // Hidden, the red fill of the surface is the backdrop; settled, it is itself.
        assert_eq!(centre_over(0.0, Effect::Fade, true, Some(green))[..3], [0, 255, 0]);
        assert_eq!(centre_over(1.0, Effect::Fade, true, Some(green))[..3], [255, 0, 0]);
        let half = centre_over(0.5, Effect::Fade, true, Some(green));
        assert!(half[0] > 0 && half[0] < 255 && half[1] > 0, "{half:?}");
    }

    #[test]
    fn the_cover_clears_away_as_the_surface_settles() {
        let full = centre(1.0, Effect::Fade, true);
        let half = centre(0.5, Effect::Fade, true);
        let none = centre(0.0, Effect::Fade, true);
        assert_eq!(full[..3], [255, 0, 0], "settled: the surface itself");
        // The window colour is covering it: less red-only, more of the window colour.
        assert!(half[1] > full[1], "halfway the cover shows: {half:?}");
        assert!(none[1] > half[1], "hidden it is the window colour: {none:?}");
        // A way that does not fade draws the surface at any progress.
        assert_eq!(centre(0.0, Effect::None, true)[..3], [255, 0, 0]);
    }
}

#[cfg(test)]
mod initial_tests {
    use super::*;
    use crate::layouts::animation::Easing;
    use iced::advanced::renderer::Headless;
    use iced::advanced::{clipboard, layout};
    use iced::widget::space;
    use std::time::Duration;

    fn frame(cover: &mut FadeCover<'_, ()>, tree: &mut Tree, at: Instant) {
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("software renderer");
        let viewport = Rectangle::with_size(Size::new(40.0, 40.0));
        let node = Widget::layout(
            cover,
            tree,
            &renderer,
            &layout::Limits::new(Size::ZERO, viewport.size()),
        );
        Widget::update(
            cover,
            tree,
            &Event::Window(window::Event::RedrawRequested(at)),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut vec![]),
            &viewport,
        );
    }

    fn make(open: bool, initial: f32) -> FadeCover<'static, ()> {
        FadeCover::new(space().width(20).height(20), open)
            .spec(Spec::Timed(Duration::from_millis(1000), Easing::Linear))
            .starting_value(initial)
    }

    fn shown(tree: &Tree) -> f32 {
        tree.state.downcast_ref::<FadeCoverState>().progress.shown
    }

    #[test]
    fn a_surface_coming_in_starts_from_its_initial_opacity_not_from_nothing() {
        let t0 = Instant::now();
        for initial in [0.0, 0.5] {
            let mut closed = make(false, initial);
            let mut tree = Tree::new(&closed as &dyn Widget<(), Theme, iced::Renderer>);
            frame(&mut closed, &mut tree, t0);
            assert_eq!(shown(&tree), 0.0, "closed is hidden whatever the initial");
            let mut opening = make(true, initial);
            tree.diff(&opening as &dyn Widget<(), Theme, iced::Renderer>);
            frame(&mut opening, &mut tree, t0 + Duration::from_millis(100));
            assert!(
                shown(&tree) >= initial,
                "opening from {initial}: {}",
                shown(&tree)
            );
            // And it still ends fully shown.
            frame(&mut opening, &mut tree, t0 + Duration::from_millis(2000));
            assert_eq!(shown(&tree), 1.0);
        }
        // Leaving ends at its ending value.
        let mut leaving = make(false, 0.0).ending_value(0.3);
        let mut tree = Tree::new(&leaving as &dyn Widget<(), Theme, iced::Renderer>);
        tree.state
            .downcast_mut::<FadeCoverState>()
            .progress
            .set_immediate(1.0);
        frame(&mut leaving, &mut tree, t0);
        frame(&mut leaving, &mut tree, t0 + Duration::from_millis(2000));
        assert!((shown(&tree) - 0.3).abs() < 1e-3, "{}", shown(&tree));
        // Without one, at nothing.
        let mut leaving = make(false, 0.5);
        let mut tree = Tree::new(&leaving as &dyn Widget<(), Theme, iced::Renderer>);
        tree.state
            .downcast_mut::<FadeCoverState>()
            .progress
            .set_immediate(1.0);
        frame(&mut leaving, &mut tree, t0);
        frame(&mut leaving, &mut tree, t0 + Duration::from_millis(2000));
        assert_eq!(shown(&tree), 0.0);
    }
}
