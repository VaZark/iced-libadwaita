//! `Dock`: a surface docked against an edge of a base: a footer, a banner, a drawer, a bottom
//! sheet (`AdwBottomSheet`). One widget; you define its size, its length and its place yourself:
//!
//! * **side** ([`Side`]): the edge it is docked to (default the bottom).
//! * **size** ([`DockSize`]): how far it reaches across the edge: `Auto` (its own), `Fixed(px)`, a
//!   `Fraction` of the base, or `FullScreen`: the whole base, every border flush.
//! * **length** ([`DockSize`]) and **align** ([`Align`]): how long it is along the edge (default
//!   the whole edge) and, when shorter, where along it: the start, the center or the end. This is
//!   the flex and position of a floating panel.
//! * **margin**: how far it stands off the edge it is docked to, and off that edge only. `0` (the
//!   default) is **flush** with the edge; a margin lifts it, and its corners on that edge round.
//! * **mode** ([`DockMode`]): `Push` puts it in flow: it slides in and the base gives up exactly the
//!   room it takes, like a sidebar beside the content ([`AdaptiveSplit`](crate::layouts::AdaptiveSplit)
//!   and [`layouts::dock::rule`](crate::layouts::dock::rule), the same geometry as
//!   `NavigationSplitView` and its drawer); `Overlay` floats it over the base on an
//!   [`Overlay`](crate::layouts::Overlay): the base is faded and disabled, the surface slides in from its
//!   edge, and a press outside sends [`on_outside`](Dock::on_outside).
//! * **scope** ([`DockScope`]): `Content` docks to the base as given (a pane, the content area);
//!   `Window` docks to the whole window, so the corners that touch the window's corners take its
//!   radius ([`window_radius`](Dock::window_radius)).
//!
//! A pushed dock takes the whole edge (length, align, margin and border are the overlay's).
//!
//! A hairline **border** (the gallery previews' kind) outlines the surface on the sides that are
//! not flush with the base: a docked surface standing off its edge is outlined all around, a flush
//! one only where it meets the content; `FullScreen` has none. [`border(false)`](Dock::border)
//! removes it.
//!
//! `FullScreen` covers the base and ignores margin: all its borders are flush (square, or the
//! window's radius for a window dock). The base decides the area: wrap the window's root for a
//! window dock, a pane for a content one. [`open`](Dock::open) shows or hides it without moving
//! the base in the widget tree.
//!
//! ```ignore
//! Dock::new(content, sheet)
//!     .open(self.open)
//!     .side(Side::Bottom)
//!     .size(DockSize::Fixed(240.0))
//!     .length(DockSize::Fixed(480.0))      // 480 wide ...
//!     .align(Align::Center)                // ... centered on the edge
//!     .margin(12.0)                        // floating 12 px off the bottom
//!     .mode(DockMode::Overlay)
//!     .on_outside(Message::Close)
//! ```

use crate::Element;
use crate::layouts::anchor::{Align, Dim, Side};
use crate::layouts::dock::rule::Extent;
use crate::layouts::{Overlay, adaptive_split};
use crate::styles::container as sc;
use crate::{Metrics, Theme};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Renderer as _, Shell, overlay};
use iced::widget::container;
use iced::{Event, Length, Rectangle, Size, Vector, mouse};

/// How the dock shares the window with the base.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DockMode {
    /// In flow: the base gives up the room the dock takes (a sidebar, a docked pane).
    #[default]
    Push,
    /// Over the base, which is faded and disabled while it is up (a sheet, a drawer).
    Overlay,
}

/// What the dock is docked to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DockScope {
    /// The base as given (a pane, the content area).
    #[default]
    Content,
    /// The whole window: corners on the window's corners take its radius.
    Window,
}

/// How far a dock reaches, across its edge (`size`) or along it (`length`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum DockSize {
    /// Its own size.
    #[default]
    Auto,
    Fixed(f32),
    /// A fraction of the base on that axis (0 to 1).
    Fraction(f32),
    /// The whole base: every border flush, margin ignored.
    FullScreen,
}

impl DockSize {
    fn extent(self) -> Extent {
        match self {
            Self::Auto => Extent::Auto,
            Self::Fixed(px) => Extent::Px(px),
            Self::Fraction(f) => Extent::Fraction(f),
            Self::FullScreen => Extent::Fraction(1.0),
        }
    }
    fn dim(self) -> Dim {
        match self {
            Self::Auto => Dim::Auto,
            Self::Fixed(px) => Dim::Fixed(px),
            Self::Fraction(f) => Dim::Fraction(f),
            Self::FullScreen => Dim::MatchAnchor,
        }
    }
    /// Whether it spans the whole axis.
    fn spans(self) -> bool {
        matches!(self, Self::FullScreen) || matches!(self, Self::Fraction(f) if f >= 1.0)
    }
}

pub struct Dock<'a, M> {
    base: Element<'a, M>,
    surface: Element<'a, M>,
    open: bool,
    side: Side,
    size: DockSize,
    length: DockSize,
    align: Align,
    mode: DockMode,
    scope: DockScope,
    scrim: bool,
    on_outside: Option<M>,
    window_radius: f32,
    margin: f32,
    border: bool,
    initial_focus: Option<iced::widget::Id>,
}

impl<'a, M: Clone + 'a> Dock<'a, M> {
    /// `surface` docked against the bottom edge of `base`, open, pushing the base aside, its own
    /// size and as long as the edge.
    pub fn new(base: impl Into<Element<'a, M>>, surface: impl Into<Element<'a, M>>) -> Self {
        Self {
            base: base.into(),
            surface: surface.into(),
            open: true,
            side: Side::Bottom,
            size: DockSize::Auto,
            length: DockSize::FullScreen,
            align: Align::Start,
            mode: DockMode::Push,
            scope: DockScope::Content,
            scrim: true,
            on_outside: None,
            window_radius: Metrics::desktop().radius_window,
            margin: 0.0,
            border: true,
            initial_focus: None,
        }
    }
    /// Whether the surface shows (default true).
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    /// The edge it is docked to (default the bottom).
    pub fn side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }
    /// How far it reaches across the edge (default its own size); `FullScreen` covers the base.
    pub fn size(mut self, size: DockSize) -> Self {
        self.size = size;
        self
    }
    /// How long it is along the edge (default the whole edge).
    pub fn length(mut self, length: DockSize) -> Self {
        self.length = length;
        self
    }
    /// Where it sits along the edge when it is shorter than it (default the start).
    pub fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
    pub fn mode(mut self, mode: DockMode) -> Self {
        self.mode = mode;
        self
    }
    pub fn scope(mut self, scope: DockScope) -> Self {
        self.scope = scope;
        self
    }
    /// Fades the base behind a modal dock (default on).
    pub fn scrim(mut self, on: bool) -> Self {
        self.scrim = on;
        self
    }
    /// Sent for a press outside a modal dock: close it from the app. Leave it unset for a dock
    /// that a press outside does not close.
    pub fn on_outside(mut self, m: M) -> Self {
        self.on_outside = Some(m);
        self
    }
    /// The window's corner radius, used by a [`Window`](DockScope::Window) dock
    /// (`window::State::radius`).
    pub fn window_radius(mut self, r: f32) -> Self {
        self.window_radius = r;
        self
    }
    /// The child of a modal dock that takes focus when it opens, by its widget id (default: the
    /// first focusable one).
    pub fn initial_focus(mut self, id: impl Into<iced::widget::Id>) -> Self {
        self.initial_focus = Some(id.into());
        self
    }
    /// Whether the surface has its hairline border, on the sides that are not flush (default
    /// true).
    pub fn border(mut self, on: bool) -> Self {
        self.border = on;
        self
    }
    /// How far it stands off the edge it is docked to, and off that edge only (default 0: flush).
    pub fn margin(mut self, px: f32) -> Self {
        self.margin = px.max(0.0);
        self
    }
}

/// The corner radii of a docked surface. `edge` is the radius of the corners on the docked edge
/// (square, or the window's), `inner` the others'. Standing off the edge (`lifted`) rounds all.
fn corners(side: Side, inner: f32, edge: f32, lifted: bool) -> iced::border::Radius {
    if lifted {
        return inner.into();
    }
    let (tl, tr, br, bl) = match side {
        Side::Bottom => (inner, inner, edge, edge),
        Side::Top => (edge, edge, inner, inner),
        Side::Left => (edge, inner, inner, edge),
        Side::Right => (inner, edge, edge, inner),
    };
    iced::border::Radius {
        top_left: tl,
        top_right: tr,
        bottom_right: br,
        bottom_left: bl,
    }
}

/// Which sides of a docked surface are flush with the boundary of the base it is docked to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Flush {
    top: bool,
    right: bool,
    bottom: bool,
    left: bool,
}

impl Flush {
    fn all(self) -> bool {
        self.top && self.right && self.bottom && self.left
    }
}

/// The sides of a dock that meet the base's boundary: the docked edge (unless it stands off by
/// `margin`), the opposite side (when `size` reaches it) and the two ends along the edge (when
/// `length` spans, or the one it is aligned to).
fn flush_sides(side: Side, size: DockSize, length: DockSize, align: Align, margin: f32) -> Flush {
    let edge = margin == 0.0;
    let inner = size.spans();
    let start = length.spans() || align == Align::Start;
    let end = length.spans() || align == Align::End;
    match side {
        Side::Bottom => Flush {
            bottom: edge,
            top: inner,
            left: start,
            right: end,
        },
        Side::Top => Flush {
            top: edge,
            bottom: inner,
            left: start,
            right: end,
        },
        Side::Left => Flush {
            left: edge,
            right: inner,
            top: start,
            bottom: end,
        },
        Side::Right => Flush {
            right: edge,
            left: inner,
            top: start,
            bottom: end,
        },
    }
}

/// Draws a hairline border over `content`, leaving the `flush` sides open: the border is drawn
/// around bounds grown past those sides and clipped to the content's own bounds, so the open
/// sides' lines fall outside and the corners still follow `radius`.
struct Outlined<'a, M> {
    content: Element<'a, M>,
    flush: Flush,
    radius: Box<dyn Fn(&Theme) -> iced::border::Radius + 'a>,
}

const HAIRLINE: f32 = 1.0;

impl<'a, M> Widget<M, Theme, iced::Renderer> for Outlined<'a, M> {
    fn tag(&self) -> iced::advanced::widget::tree::Tag {
        self.content.as_widget().tag()
    }
    fn state(&self) -> iced::advanced::widget::tree::State {
        self.content.as_widget().state()
    }
    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }
    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
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
        self.content.as_widget_mut().layout(tree, renderer, limits)
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
        self.content.as_widget_mut().update(
            tree, event, layout, cursor, renderer, clipboard, shell, viewport,
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
        self.content
            .as_widget()
            .mouse_interaction(tree, layout, cursor, viewport, renderer)
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
        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, layout, cursor, viewport);
        if self.flush.all() {
            return;
        }
        let b = layout.bounds();
        let f = self.flush;
        let grow = |flush: bool| if flush { HAIRLINE } else { 0.0 };
        let grown = Rectangle {
            x: b.x - grow(f.left),
            y: b.y - grow(f.top),
            width: b.width + grow(f.left) + grow(f.right),
            height: b.height + grow(f.top) + grow(f.bottom),
        };
        let border = iced::Border {
            color: theme.semantic.border,
            width: HAIRLINE,
            radius: (self.radius)(theme),
        };
        renderer.with_layer(b, |renderer| {
            renderer.fill_quad(
                Quad {
                    bounds: grown,
                    border,
                    shadow: iced::Shadow::default(),
                    snap: false,
                },
                iced::Color::TRANSPARENT,
            );
        });
    }
    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(tree, layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        self.content
            .as_widget_mut()
            .overlay(tree, layout, renderer, viewport, translation)
    }
}

impl<'a, M: Clone + 'a> From<Dock<'a, M>> for Element<'a, M> {
    fn from(d: Dock<'a, M>) -> Self {
        let full = d.size == DockSize::FullScreen;
        // Full screen: every border flush, so no margin and no stand-off.
        let margin = if full { 0.0 } else { d.margin };
        let (size, length) = if full {
            (DockSize::FullScreen, DockSize::FullScreen)
        } else {
            (d.size, d.length)
        };
        // The radius of a corner on the window's own corner: only where the dock reaches it.
        let window_edge = match d.scope {
            DockScope::Window => d.window_radius,
            DockScope::Content => 0.0,
        };
        let edge_radius = if length.spans() { window_edge } else { 0.0 };
        let side = d.side;
        let flush = flush_sides(side, size, length, d.align, margin);
        let lifted_radius = margin > 0.0;
        match d.mode {
            DockMode::Push => {
                // The same geometry as a split view's sidebar: the dock is the sidebar, in flow
                // while open, sliding out of the way when not.
                // Its own size: across the edge the panel shrinks to its content, so measuring it
                // freely gives that size (and not all the room).
                let own = size == DockSize::Auto;
                let (w, h) = match side {
                    Side::Left | Side::Right => (
                        if own { Length::Shrink } else { Length::Fill },
                        Length::Fill,
                    ),
                    Side::Top | Side::Bottom => (
                        Length::Fill,
                        if own { Length::Shrink } else { Length::Fill },
                    ),
                };
                let panel: Element<'a, M> = container(d.surface)
                    .width(w)
                    .height(h)
                    .style(|t: &Theme| iced::widget::container::Style {
                        background: Some(iced::Background::Color(t.semantic.window.bg)),
                        ..Default::default()
                    })
                    .into();
                let ext = size.extent();
                adaptive_split(panel, d.base, d.open, 0.0, 0.0)
                    .side(side)
                    .extent(ext)
                    .into()
            }
            DockMode::Overlay => {
                if !d.open {
                    return crate::layouts::overlay::view::closed(d.base);
                }
                let lifted = margin > 0.0;
                let panel = container(d.surface)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(move |t: &Theme| iced::widget::container::Style {
                        border: iced::Border {
                            radius: if full {
                                edge_radius.into()
                            } else {
                                corners(side, t.metrics.radius_l, edge_radius, lifted)
                            },
                            ..Default::default()
                        },
                        ..sc::dialog(t)
                    });
                let panel: Element<'a, M> = match d.size {
                    // Its own size: across the edge the panel shrinks to its content (its width
                    // on a left or right edge, its height on a top or bottom one); along the edge
                    // it fills what the layer gives it.
                    DockSize::Auto => match side {
                        Side::Left | Side::Right => {
                            panel.width(Length::Shrink).height(Length::Fill).into()
                        }
                        Side::Top | Side::Bottom => {
                            panel.width(Length::Fill).height(Length::Shrink).into()
                        }
                    },
                    _ => panel.into(),
                };
                let panel = outlined(panel, d.border, flush, side, edge_radius, lifted_radius);
                let mut o = Overlay::new(d.base, panel)
                    .docked(side, size.dim())
                    .along(length.dim(), d.align)
                    .scrim(d.scrim)
                    .margin(margin)
                    .window_radius(window_edge);
                if let Some(m) = d.on_outside {
                    o = o.on_outside(m);
                }
                if let Some(id) = d.initial_focus {
                    o = o.initial_focus(id);
                }
                o.into()
            }
        }
    }
}

/// `surface` with the hairline border on the sides that are not flush, following the dock's corner
/// radii (round all around when it stands off its edge). `on` false leaves it as it is.
fn outlined<'a, M: 'a>(
    surface: impl Into<Element<'a, M>>,
    on: bool,
    flush: Flush,
    side: Side,
    edge_radius: f32,
    lifted: bool,
) -> Element<'a, M> {
    let surface = surface.into();
    if !on || flush.all() {
        return surface;
    }
    iced::Element::new(Outlined {
        content: surface,
        flush,
        radius: Box::new(move |t: &Theme| corners(side, t.metrics.radius_l, edge_radius, lifted)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_edge_corners_are_square_and_the_open_side_round() {
        let r = corners(Side::Bottom, 12.0, 0.0, false);
        assert_eq!(
            (r.top_left, r.top_right, r.bottom_left, r.bottom_right),
            (12.0, 12.0, 0.0, 0.0)
        );
        let r = corners(Side::Right, 12.0, 15.0, false);
        assert_eq!(
            (r.top_left, r.bottom_left, r.top_right, r.bottom_right),
            (12.0, 12.0, 15.0, 15.0)
        );
    }

    #[test]
    fn standing_off_the_edge_rounds_every_corner() {
        let r = corners(Side::Bottom, 12.0, 0.0, true);
        assert_eq!(
            (r.top_left, r.top_right, r.bottom_left, r.bottom_right),
            (12.0, 12.0, 12.0, 12.0)
        );
    }

    #[test]
    fn a_flush_dock_is_open_on_the_edge_it_meets_and_outlined_elsewhere() {
        // A bottom sheet across the whole width: only the top meets the content.
        let f = flush_sides(
            Side::Bottom,
            DockSize::Auto,
            DockSize::FullScreen,
            Align::Start,
            0.0,
        );
        assert_eq!(
            (f.bottom, f.left, f.right, f.top),
            (true, true, true, false)
        );
        // Floating off the bottom: the bottom is no longer flush.
        let f = flush_sides(
            Side::Bottom,
            DockSize::Auto,
            DockSize::FullScreen,
            Align::Start,
            12.0,
        );
        assert!(!f.bottom && f.left && f.right && !f.top);
        // Centered and shorter: it touches neither end.
        let f = flush_sides(
            Side::Bottom,
            DockSize::Auto,
            DockSize::Fixed(320.0),
            Align::Center,
            0.0,
        );
        assert_eq!((f.left, f.right, f.bottom), (false, false, true));
        // Aligned to the end it meets the end.
        let f = flush_sides(
            Side::Bottom,
            DockSize::Auto,
            DockSize::Fixed(320.0),
            Align::End,
            0.0,
        );
        assert_eq!((f.left, f.right), (false, true));
        // A drawer on the right reaches the top and bottom only when it spans them.
        let f = flush_sides(
            Side::Right,
            DockSize::Fixed(240.0),
            DockSize::FullScreen,
            Align::Start,
            0.0,
        );
        assert_eq!(
            (f.right, f.top, f.bottom, f.left),
            (true, true, true, false)
        );
    }

    #[test]
    fn full_screen_is_flush_on_every_side_so_it_has_no_border() {
        let f = flush_sides(
            Side::Left,
            DockSize::FullScreen,
            DockSize::FullScreen,
            Align::Start,
            0.0,
        );
        assert!(f.all());
    }

    #[test]
    fn full_screen_and_a_whole_fraction_span_the_axis() {
        assert!(DockSize::FullScreen.spans() && DockSize::Fraction(1.0).spans());
        assert!(!DockSize::Fraction(0.5).spans() && !DockSize::Fixed(300.0).spans());
        assert!(!DockSize::Auto.spans());
    }

    #[test]
    fn it_builds_in_every_mode_side_size_and_place() {
        for mode in [DockMode::Push, DockMode::Overlay] {
            for side in [Side::Top, Side::Bottom, Side::Left, Side::Right] {
                for size in [
                    DockSize::Auto,
                    DockSize::Fixed(100.0),
                    DockSize::Fraction(0.4),
                    DockSize::FullScreen,
                ] {
                    for length in [
                        DockSize::FullScreen,
                        DockSize::Fixed(200.0),
                        DockSize::Fraction(0.5),
                    ] {
                        for align in [Align::Start, Align::Center, Align::End] {
                            for (open, margin) in [(true, 0.0), (true, 12.0), (false, 0.0)] {
                                let _: Element<'_, ()> =
                                    Dock::new(iced::widget::space(), iced::widget::space())
                                        .mode(mode)
                                        .side(side)
                                        .size(size)
                                        .length(length)
                                        .align(align)
                                        .open(open)
                                        .margin(margin)
                                        .into();
                            }
                        }
                    }
                }
            }
        }
    }
}
