//! Resolved surface geometry shared during subtree traversal.
//!
//! DECISION (user, 2026-11): effects take geometry from the actual target style; unknown targets
//! get no guessed mask or ring (legacy radius fallbacks were removed on purpose), without
//! removing controlled events or modal capture. Revisit if a real target cannot state its shape.
//!
//! Geometry is scoped to synchronous draw/update/mouse traversal, never stored in a
//! process-wide geometry registry. Scopes nest and restore on unwind. Descendants
//! can read their nearest surface and the complete intersection (not guessed radii).
//! Portals deliberately reset this context: overlays escape their owner's surface,
//! including nested overlays. A popup needing a surface must install its own Boundary.
//!
//! iced exposes rectangular renderer layers, not rounded stencil clips. We replay
//! content in disjoint conservative rectangular bands: one central clip and at most
//! one band per logical pixel in the corner regions. This clips arbitrary primitives,
//! images and text, including transparent surfaces; it never paints corner covers.
//! The staircase is inward-rounded at one logical pixel precision, not antialiased.
//! Large radii cost repeated child draws/layers; nested boundaries prune by viewport.
//! Draw implementations must therefore be replay-safe, as iced draw normally is.
//! Pixel precision is a rasterization choice, not a theme/design radius.

use std::cell::{Cell, RefCell};

use crate::{Element, Theme};
use iced::advanced::{
    Clipboard, Renderer as _, Shell,
    layout::{self, Layout},
    overlay, renderer,
    widget::{Operation, Tree, Widget, tree},
};
use iced::widget::{Container, container};
use iced::{Event, Length, Point, Rectangle, Size, Vector, border::Radius, mouse};

/// A surface's actual bounds and circular, independently rounded corners.
/// Construct with `new` to sanitize and proportionally normalize overlapping radii.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceShape {
    pub bounds: Rectangle,
    pub radius: Radius,
}

impl SurfaceShape {
    /// One shared scale ensures every adjacent pair fits, followed by iced's
    /// per-corner half-short-side cap. The latter is required for geometry to match
    /// actual quad painting (including tiny-skia), even for asymmetric corners.
    /// Nonfinite coordinates/lengths and negative lengths/radii become zero.
    pub fn new(bounds: Rectangle, radius: impl Into<Radius>) -> Self {
        fn finite(v: f32) -> f32 {
            if v.is_finite() { v } else { 0.0 }
        }
        let bounds = Rectangle {
            x: finite(bounds.x),
            y: finite(bounds.y),
            width: finite(bounds.width).max(0.0),
            height: finite(bounds.height).max(0.0),
        };
        let mut r = radius.into();
        r.top_left = finite(r.top_left).max(0.0);
        r.top_right = finite(r.top_right).max(0.0);
        r.bottom_left = finite(r.bottom_left).max(0.0);
        r.bottom_right = finite(r.bottom_right).max(0.0);
        let mut scale = 1.0_f32;
        for (extent, sum) in [
            (bounds.width, r.top_left + r.top_right),
            (bounds.width, r.bottom_left + r.bottom_right),
            (bounds.height, r.top_left + r.bottom_left),
            (bounds.height, r.top_right + r.bottom_right),
        ] {
            if sum > 0.0 {
                scale = scale.min(extent / sum);
            }
        }
        let cap = bounds.width.min(bounds.height) / 2.0;
        r.top_left = (r.top_left * scale).min(cap);
        r.top_right = (r.top_right * scale).min(cap);
        r.bottom_left = (r.bottom_left * scale).min(cap);
        r.bottom_right = (r.bottom_right * scale).min(cap);
        Self { bounds, radius: r }
    }

    /// Exact analytic hit test of the closed shape (including its edge),
    /// independent of the conservative raster approximation.
    pub fn contains(self, point: Point) -> bool {
        let b = self.bounds;
        if b.width <= 0.0
            || b.height <= 0.0
            || !point.x.is_finite()
            || !point.y.is_finite()
            || point.x < b.x
            || point.y < b.y
            || point.x > b.x + b.width
            || point.y > b.y + b.height
        {
            return false;
        }
        let (left, right) = self.horizontal_span(point.y);
        point.x >= left && point.x <= right
    }

    /// The intersection retains each shape; two rounded rectangles generally do
    /// not intersect to another rounded rectangle.
    pub fn intersection(self, other: Self) -> SurfaceIntersection {
        SurfaceIntersection {
            shapes: vec![self, other],
        }
    }

    fn horizontal_span(self, y: f32) -> (f32, f32) {
        let b = self.bounds;
        let r = self.radius;
        fn inset(y: f32, top: f32, bottom: f32, rt: f32, rb: f32) -> f32 {
            let (radius, distance) = if y < top + rt {
                (rt, top + rt - y)
            } else if y > bottom - rb {
                (rb, y - (bottom - rb))
            } else {
                return 0.0;
            };
            radius - (radius * radius - distance * distance).max(0.0).sqrt()
        }
        (
            b.x + inset(y, b.y, b.y + b.height, r.top_left, r.bottom_left),
            b.x + b.width - inset(y, b.y, b.y + b.height, r.top_right, r.bottom_right),
        )
    }

    /// Nonoverlapping, inward-rounded rectangular clips at `pixel` logical units.
    /// Invalid precision produces no clips. The central straight region uses one layer.
    pub fn clip_rectangles(self, viewport: Rectangle, pixel: f32) -> Vec<Rectangle> {
        if !pixel.is_finite() || pixel <= 0.0 {
            return vec![];
        }
        let Some(view) = self.bounds.intersection(&viewport) else {
            return vec![];
        };
        let ceil = |v: f32| (v / pixel).ceil() * pixel;
        let floor = |v: f32| (v / pixel).floor() * pixel;
        let central_top = ceil(self.bounds.y + self.radius.top_left.max(self.radius.top_right));
        let central_bottom = floor(
            self.bounds.y + self.bounds.height
                - self.radius.bottom_left.max(self.radius.bottom_right),
        );
        let mut y = ceil(view.y);
        let end = floor(view.y + view.height);
        let mut clips = Vec::new();
        while y < end {
            let next = if y >= central_top && y < central_bottom {
                central_bottom.min(end)
            } else {
                (y + pixel).min(end)
            };
            if next <= y {
                break;
            }
            let (l1, r1) = self.horizontal_span(y);
            let (l2, r2) = self.horizontal_span(next);
            let left = ceil(l1.max(l2).max(view.x));
            let right = floor(r1.min(r2).min(view.x + view.width));
            if right > left {
                clips.push(Rectangle {
                    x: left,
                    y,
                    width: right - left,
                    height: next - y,
                });
            }
            y = next;
        }
        clips
    }
}

/// Exact intersection of the active ancestor surfaces, nearest last.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SurfaceIntersection {
    pub shapes: Vec<SurfaceShape>,
}
impl SurfaceIntersection {
    pub fn contains(&self, point: Point) -> bool {
        self.shapes.iter().all(|s| s.contains(point))
    }
    /// Bounding rectangle only; use `contains` for rounded-corner exclusion.
    pub fn bounds(&self) -> Option<Rectangle> {
        let mut shapes = self.shapes.iter();
        let mut bounds = shapes.next()?.bounds;
        for shape in shapes {
            bounds = bounds.intersection(&shape.bounds)?;
        }
        Some(bounds)
    }
    pub fn intersect(mut self, shape: SurfaceShape) -> Self {
        self.shapes.push(shape);
        self
    }
    pub fn cursor(&self, cursor: mouse::Cursor) -> mouse::Cursor {
        if cursor.position().is_some_and(|p| self.contains(p)) {
            cursor
        } else {
            mouse::Cursor::Unavailable
        }
    }
}

thread_local! { static SURFACES: RefCell<Vec<SurfaceShape>> = const { RefCell::new(Vec::new()) }; }
/// Nearest resolved parent surface in the current traversal, not a previous frame.
pub fn current_surface() -> Option<SurfaceShape> {
    SURFACES.with(|s| s.borrow().last().copied())
}
/// Snapshot of all ancestor constraints in the current traversal.
pub fn current_boundary() -> SurfaceIntersection {
    SurfaceIntersection {
        shapes: SURFACES.with(|s| s.borrow().clone()),
    }
}
/// Run a subtree under a surface. No RefCell borrow survives the callback.
/// Restoration is RAII/unwind-safe; unrelated sibling traversals see no geometry.
pub fn with_surface<T>(shape: SurfaceShape, f: impl FnOnce() -> T) -> T {
    struct Pop;
    impl Drop for Pop {
        fn drop(&mut self) {
            SURFACES.with(|s| {
                s.borrow_mut().pop();
            });
        }
    }
    SURFACES.with(|s| s.borrow_mut().push(shape));
    let _pop = Pop;
    f()
}
/// Explicit portal policy: escape all ancestor surfaces for this traversal only.
/// Does not cancel renderer layers already open by a caller; draw portals separately.
pub fn with_portal<T>(f: impl FnOnce() -> T) -> T {
    struct Restore(Vec<SurfaceShape>);
    impl Drop for Restore {
        fn drop(&mut self) {
            SURFACES.with(|s| *s.borrow_mut() = std::mem::take(&mut self.0));
        }
    }
    let _restore = Restore(SURFACES.with(|s| std::mem::take(&mut *s.borrow_mut())));
    f()
}

type Underlay<'a> = Box<dyn Fn(&mut iced::Renderer, &Theme, SurfaceShape) + 'a>;
/// Rounded subtree clip, optionally owning a container's resolved surface painting.
///
/// Updates have no theme argument in iced. Styled boundaries retain only the last
/// drawn radius in their own Tree state, resolving bounds from the current layout.
/// Before the first draw, pointer targeting is disabled (keyboard/releases still
/// delivered). No global geometry persists. Fixed-radius boundaries work immediately.
/// Overlays are forwarded as portals and are never clipped by this boundary.
pub struct Boundary<'a, M> {
    content: Element<'a, M>,
    style: Box<dyn Fn(&Theme) -> container::Style + 'a>,
    radius: Option<Radius>,
    underlay: Option<Underlay<'a>>,
}
#[derive(Default)]
struct State {
    radius: Cell<Option<Radius>>,
}
impl<'a, M: 'a> Boundary<'a, M> {
    /// Clip content using explicitly supplied geometry (no background).
    pub fn new(content: impl Into<Element<'a, M>>, radius: impl Into<Radius>) -> Self {
        let radius = radius.into();
        Self {
            content: content.into(),
            style: Box::new(move |_| container::Style {
                border: iced::Border::default().rounded(radius),
                ..Default::default()
            }),
            radius: Some(radius),
            underlay: None,
        }
    }
    /// Own painting and clipping from ONE final style resolution. The container
    /// supplies layout only: its native style/clip are replaced. Apply layout props
    /// before this call, and pass the final (including user overrides) style resolver.
    /// Shadows paint outside the clip; the border paints after children and remains visible.
    pub fn surface(
        content: Container<'a, M, Theme>,
        style: impl Fn(&Theme) -> container::Style + 'a,
    ) -> Self {
        Self {
            content: content
                .style(|_| container::Style {
                    snap: false,
                    ..Default::default()
                })
                .clip(false)
                .into(),
            style: Box::new(style),
            radius: None,
            underlay: None,
        }
    }
    /// Paint elevation outside the content clip using the same resolved shape.
    pub fn underlay(
        mut self,
        draw: impl Fn(&mut iced::Renderer, &Theme, SurfaceShape) + 'a,
    ) -> Self {
        self.underlay = Some(Box::new(draw));
        self
    }
    fn shape(&self, tree: &Tree, bounds: Rectangle) -> Option<SurfaceShape> {
        self.radius
            .or(tree.state.downcast_ref::<State>().radius.get())
            .map(|r| SurfaceShape::new(bounds, r))
    }
}
impl<'a, M: 'a> Widget<M, Theme, iced::Renderer> for Boundary<'a, M> {
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }
    fn state(&self) -> tree::State {
        tree::State::new(State::default())
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
    fn size_hint(&self) -> Size<Length> {
        self.content.as_widget().size_hint()
    }
    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
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
            .operate(&mut tree.children[0], layout, renderer, operation);
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
        let shape = self
            .shape(tree, layout.bounds())
            .unwrap_or_else(|| SurfaceShape::new(Rectangle::with_size(Size::ZERO), 0.0));
        with_surface(shape, || {
            let boundary = current_boundary();
            let cursor = boundary.cursor(cursor);
            // Touch embeds its own position; reject new gestures in excluded corners.
            // Moves/releases are still delivered to let existing gestures terminate.
            if let Event::Touch(iced::touch::Event::FingerPressed { position, .. }) = event {
                if !boundary.contains(*position) {
                    return;
                }
            }
            let view = boundary
                .bounds()
                .and_then(|b| b.intersection(viewport))
                .unwrap_or_default();
            self.content.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout,
                cursor,
                renderer,
                clipboard,
                shell,
                &view,
            );
        });
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        let Some(shape) = self.shape(tree, layout.bounds()) else {
            return mouse::Interaction::None;
        };
        with_surface(shape, || {
            let boundary = current_boundary();
            let cursor = boundary.cursor(cursor);
            if cursor.position().is_none() {
                return mouse::Interaction::None;
            }
            let view = boundary
                .bounds()
                .and_then(|b| b.intersection(viewport))
                .unwrap_or_default();
            self.content.as_widget().mouse_interaction(
                &tree.children[0],
                layout,
                cursor,
                &view,
                renderer,
            )
        })
    }
    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        inherited: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let mut style = (self.style)(theme);
        // Cache the declared radius, not its size-normalized form: a subsequent
        // layout may grow before the next draw and must normalize afresh.
        tree.state
            .downcast_ref::<State>()
            .radius
            .set(Some(style.border.radius));
        let shape = SurfaceShape::new(layout.bounds(), style.border.radius);
        style.border.radius = shape.radius;
        // Fractional geometry is intentional; snapping the paint independently would
        // make its outline disagree with the geometry shared with descendants.
        style.snap = false;

        if shape.bounds.intersection(viewport).is_none() {
            return;
        }
        if let Some(draw) = &self.underlay {
            draw(renderer, theme, shape);
        }
        container::draw_background(
            renderer,
            &container::Style {
                border: iced::Border {
                    width: 0.0,
                    ..style.border
                },
                ..style
            },
            shape.bounds,
        );
        with_surface(shape, || {
            let cursor = current_boundary().cursor(cursor);
            let text_style = renderer::Style {
                text_color: style.text_color.unwrap_or(inherited.text_color),
            };
            for clip in shape.clip_rectangles(*viewport, 1.0) {
                renderer.with_layer(clip, |r| {
                    self.content.as_widget().draw(
                        &tree.children[0],
                        r,
                        theme,
                        &text_style,
                        layout,
                        cursor,
                        &clip,
                    )
                });
            }
        });
        if style.border.width > 0.0 {
            // iced batches quads ahead of child layers. A separate trailing layer
            // is necessary: submission order alone would put this UNDER content.
            renderer.with_layer(*viewport, |renderer| {
                container::draw_background(
                    renderer,
                    &container::Style {
                        background: None,
                        shadow: iced::Shadow::default(),
                        ..style
                    },
                    shape.bounds,
                );
            });
        }
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        with_portal(|| {
            self.content.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            )
        })
        .map(portal)
    }
}
impl<'a, M: 'a> From<Boundary<'a, M>> for Element<'a, M> {
    fn from(value: Boundary<'a, M>) -> Self {
        Element::new(value)
    }
}

struct Portal<'a, M>(overlay::Element<'a, M, Theme, iced::Renderer>);
fn portal<'a, M: 'a>(
    element: overlay::Element<'a, M, Theme, iced::Renderer>,
) -> overlay::Element<'a, M, Theme, iced::Renderer> {
    overlay::Element::new(Box::new(Portal(element)))
}
impl<M> overlay::Overlay<M, Theme, iced::Renderer> for Portal<'_, M> {
    fn layout(&mut self, renderer: &iced::Renderer, bounds: Size) -> layout::Node {
        with_portal(|| self.0.as_overlay_mut().layout(renderer, bounds))
    }
    fn draw(
        &self,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        with_portal(|| {
            self.0
                .as_overlay()
                .draw(renderer, theme, style, layout, cursor)
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
        with_portal(|| {
            self.0
                .as_overlay_mut()
                .update(event, layout, cursor, renderer, clipboard, shell)
        });
    }
    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        with_portal(|| {
            self.0
                .as_overlay()
                .mouse_interaction(layout, cursor, renderer)
        })
    }
    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        with_portal(|| self.0.as_overlay_mut().operate(layout, renderer, operation));
    }
    fn overlay<'b>(
        &'b mut self,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        with_portal(|| self.0.as_overlay_mut().overlay(layout, renderer)).map(portal)
    }
    fn index(&self) -> f32 {
        with_portal(|| self.0.as_overlay().index())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Color;
    use iced::advanced::{clipboard, renderer::Headless};
    use std::rc::Rc;

    fn renderer() -> iced::Renderer {
        iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("software renderer")
    }
    fn bounds() -> Rectangle {
        Rectangle::new(Point::new(4.0, 4.0), Size::new(64.0, 40.0))
    }
    fn radii() -> [Radius; 3] {
        [
            0.0.into(),
            100.0.into(),
            Radius {
                top_left: 22.0,
                top_right: 3.0,
                bottom_left: 0.0,
                bottom_right: 12.0,
            },
        ]
    }
    #[test]
    fn normalization_hit_testing_and_intersection() {
        let s = SurfaceShape::new(
            bounds(),
            Radius {
                top_left: 80.0,
                top_right: 40.0,
                bottom_left: 0.0,
                bottom_right: 0.0,
            },
        );
        assert_eq!(s.radius.top_left, 20.0);
        assert_eq!(s.radius.top_right, 20.0);
        assert!(!s.contains(Point::new(4.0, 4.0)));
        assert!(s.contains(Point::new(36.0, 30.0)));
        let square = SurfaceShape::new(
            Rectangle::new(Point::new(20.0, 20.0), Size::new(80.0, 80.0)),
            0.0,
        );
        let both = s.intersection(square);
        assert!(!both.contains(Point::new(36.0, 10.0)));
        assert!(both.contains(Point::new(36.0, 30.0)));
        assert_eq!(both.bounds(), s.bounds.intersection(&square.bounds));
        let invalid = SurfaceShape::new(
            Rectangle::new(Point::ORIGIN, Size::new(-1.0, f32::NAN)),
            f32::INFINITY,
        );
        assert_eq!(invalid.bounds.size(), Size::ZERO);
        assert!(!invalid.contains(Point::ORIGIN));
    }
    #[test]
    fn styled_radius_tracks_draw_theme_and_renormalizes_after_resize_shadow_is_not_clipped() {
        let mut r = renderer();
        let radius = Rc::new(Cell::new(100.0_f32));
        let declared = radius.clone();
        let mut boundary: Boundary<'_, ()> = Boundary::surface(
            container(iced::widget::space())
                .width(Length::Fill)
                .height(Length::Fill),
            move |theme| container::Style {
                background: Some(theme.semantic.window.bg.into()),
                border: iced::Border::default().rounded(declared.get()),
                shadow: iced::Shadow {
                    color: Color::from_rgb(0.0, 1.0, 0.0),
                    offset: Vector::new(4.0, 0.0),
                    blur_radius: 0.0,
                },
                ..Default::default()
            },
        );
        let mut tree = Tree::new(&boundary as &dyn Widget<(), Theme, iced::Renderer>);
        let node = boundary
            .layout(
                &mut tree,
                &r,
                &layout::Limits::new(bounds().size(), bounds().size()),
            )
            .move_to(bounds().position());
        assert!(
            boundary.shape(&tree, bounds()).is_none(),
            "no invented first-frame theme geometry"
        );
        let view = Rectangle::with_size(Size::new(80.0, 52.0));
        r.reset(view);
        boundary.draw(
            &tree,
            &mut r,
            &Theme::light(),
            &renderer::Style::default(),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &view,
        );
        let image = r.screenshot(Size::new(80, 52), 1.0, Color::BLACK);
        assert_eq!(
            pixel(&image, 70, 24),
            &[0, 255, 0, 255],
            "shadow outside content clip"
        );
        assert_eq!(
            boundary.shape(&tree, bounds()).unwrap().radius.top_left,
            20.0
        );
        let grown = Rectangle::with_size(Size::new(200.0, 100.0));
        assert_eq!(
            boundary.shape(&tree, grown).unwrap().radius.top_left,
            50.0,
            "cache declared radius, not normalized old geometry"
        );
        radius.set(3.0);
        r.reset(view);
        boundary.draw(
            &tree,
            &mut r,
            &Theme::dark(),
            &renderer::Style::default(),
            Layout::new(&node),
            mouse::Cursor::Unavailable,
            &view,
        );
        assert_eq!(
            boundary.shape(&tree, bounds()).unwrap().radius.top_left,
            3.0
        );
        assert_eq!(current_surface(), None);
    }

    #[test]
    fn traversal_is_nested_sibling_isolated_portal_and_unwind_safe() {
        let a = SurfaceShape::new(bounds(), 20.0);
        let b = SurfaceShape::new(bounds(), 3.0);
        assert_eq!(current_surface(), None);
        with_surface(a, || {
            with_surface(b, || {
                assert_eq!(current_surface(), Some(b));
                assert_eq!(current_boundary().shapes, vec![a, b]);
                assert!(!current_boundary().contains(Point::new(4.0, 4.0)));
                let result = std::panic::catch_unwind(|| {
                    with_portal(|| {
                        assert_eq!(current_surface(), None);
                        with_surface(a, || panic!("scope unwind"));
                    })
                });
                assert!(result.is_err());
                assert_eq!(current_boundary().shapes, vec![a, b]);
            });
            assert_eq!(current_surface(), Some(a));
        });
        assert_eq!(current_surface(), None);
        with_surface(b, || assert_eq!(current_boundary().shapes, vec![b]));
        assert_eq!(current_surface(), None);
    }
    #[test]
    fn bands_are_conservative_nonoverlapping_and_square_is_one_clip() {
        for radius in radii() {
            let shape = SurfaceShape::new(bounds(), radius);
            let clips = shape.clip_rectangles(bounds(), 1.0);
            for (i, clip) in clips.iter().enumerate() {
                for point in [
                    clip.position(),
                    Point::new(clip.x + clip.width, clip.y),
                    Point::new(clip.x, clip.y + clip.height),
                    Point::new(clip.x + clip.width, clip.y + clip.height),
                ] {
                    assert!(shape.contains(point), "{shape:?} excludes {point:?}");
                }
                for other in &clips[i + 1..] {
                    assert!(clip.intersection(other).is_none());
                }
            }
        }
        assert_eq!(
            SurfaceShape::new(bounds(), 0.0).clip_rectangles(bounds(), 1.0),
            vec![bounds()]
        );
    }

    // Arbitrary overflowing translucent content, independent of its allotted bounds.
    // Exact pixel comparison detects both escaped corners and double-painted bands.
    struct Paint;
    impl Widget<(), Theme, iced::Renderer> for Paint {
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
            r: &mut iced::Renderer,
            _: &Theme,
            _: &renderer::Style,
            _: Layout<'_>,
            _: mouse::Cursor,
            _: &Rectangle,
        ) {
            assert!(current_surface().is_some());
            r.fill_quad(
                renderer::Quad {
                    bounds: Rectangle::with_size(Size::new(80.0, 52.0)),
                    snap: false,
                    ..Default::default()
                },
                Color::from_rgba(1.0, 0.0, 0.0, 0.5),
            );
        }
    }
    fn measure(element: &mut Element<'_, ()>, r: &iced::Renderer) -> (Tree, layout::Node) {
        let mut tree = Tree::new(&*element);
        let node = element
            .as_widget_mut()
            .layout(
                &mut tree,
                r,
                &layout::Limits::new(bounds().size(), bounds().size()),
            )
            .move_to(bounds().position());
        (tree, node)
    }
    fn draw(
        element: &Element<'_, ()>,
        tree: &Tree,
        node: &layout::Node,
        r: &mut iced::Renderer,
    ) -> Vec<u8> {
        let viewport = Rectangle::with_size(Size::new(80.0, 52.0));
        r.reset(viewport);
        element.as_widget().draw(
            tree,
            r,
            &Theme::light(),
            &renderer::Style::default(),
            Layout::new(node),
            mouse::Cursor::Unavailable,
            &viewport,
        );
        assert_eq!(current_surface(), None);
        r.screenshot(Size::new(80, 52), 1.0, Color::BLACK)
    }
    fn pixel(bytes: &[u8], x: usize, y: usize) -> &[u8] {
        &bytes[(y * 80 + x) * 4..(y * 80 + x + 1) * 4]
    }
    // Independent convex-shape oracle: all four corners of a pixel must be inside.
    fn pixel_inside(shape: SurfaceShape, x: usize, y: usize) -> bool {
        [
            (x as f32, y as f32),
            (x as f32 + 1.0, y as f32),
            (x as f32, y as f32 + 1.0),
            (x as f32 + 1.0, y as f32 + 1.0),
        ]
        .into_iter()
        .all(|(x, y)| shape.contains(Point::new(x, y)))
    }
    #[test]
    fn rendered_square_pill_asymmetric_and_nested_clips_match_pixel_oracle() {
        let mut r = renderer();
        let mut square: Element<'_, ()> = Boundary::new(Element::new(Paint), 0.0).into();
        let (tree, node) = measure(&mut square, &r);
        let baseline = draw(&square, &tree, &node, &mut r);
        let red = pixel(&baseline, 30, 25);
        let black = pixel(&baseline, 0, 0);
        assert_ne!(red, black);
        for radius in radii() {
            for nested in [false, true] {
                let child: Element<'_, ()> = if nested {
                    Boundary::new(
                        Element::new(Paint),
                        Radius {
                            top_left: 0.0,
                            top_right: 18.0,
                            bottom_left: 15.0,
                            bottom_right: 0.0,
                        },
                    )
                    .into()
                } else {
                    Element::new(Paint)
                };
                let mut element: Element<'_, ()> = Boundary::new(child, radius).into();
                let (tree, node) = measure(&mut element, &r);
                let image = draw(&element, &tree, &node, &mut r);
                let shape = SurfaceShape::new(bounds(), radius);
                let inner = SurfaceShape::new(
                    bounds(),
                    Radius {
                        top_left: 0.0,
                        top_right: 18.0,
                        bottom_left: 15.0,
                        bottom_right: 0.0,
                    },
                );
                for y in 0..52 {
                    for x in 0..80 {
                        let inside =
                            pixel_inside(shape, x, y) && (!nested || pixel_inside(inner, x, y));
                        assert_eq!(
                            pixel(&image, x, y),
                            if inside { red } else { black },
                            "pixel ({x},{y}), radius={radius:?}, nested={nested}"
                        );
                    }
                }
            }
        }
    }

    #[derive(Clone)]
    struct Events(Rc<RefCell<Vec<(bool, usize)>>>);
    impl Widget<(), Theme, iced::Renderer> for Events {
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
        fn update(
            &mut self,
            _: &mut Tree,
            _: &Event,
            _: Layout<'_>,
            cursor: mouse::Cursor,
            _: &iced::Renderer,
            _: &mut dyn Clipboard,
            _: &mut Shell<'_, ()>,
            _: &Rectangle,
        ) {
            self.0
                .borrow_mut()
                .push((cursor.position().is_some(), current_boundary().shapes.len()));
        }
        fn mouse_interaction(
            &self,
            _: &Tree,
            _: Layout<'_>,
            _: mouse::Cursor,
            _: &Rectangle,
            _: &iced::Renderer,
        ) -> mouse::Interaction {
            mouse::Interaction::Pointer
        }
        fn overlay<'b>(
            &'b mut self,
            _: &'b mut Tree,
            _: Layout<'b>,
            _: &iced::Renderer,
            _: &Rectangle,
            translation: Vector,
        ) -> Option<overlay::Element<'b, (), Theme, iced::Renderer>> {
            assert_eq!(current_surface(), None);
            Some(overlay::Element::new(Box::new(Popup(translation))))
        }
    }
    struct Popup(Vector);
    impl overlay::Overlay<(), Theme, iced::Renderer> for Popup {
        fn layout(&mut self, _: &iced::Renderer, _: Size) -> layout::Node {
            assert_eq!(current_surface(), None);
            layout::Node::new(Size::new(3.0, 3.0))
                .move_to(Point::new(4.0 + self.0.x, 4.0 + self.0.y))
        }
        fn draw(
            &self,
            r: &mut iced::Renderer,
            _: &Theme,
            _: &renderer::Style,
            layout: Layout<'_>,
            _: mouse::Cursor,
        ) {
            assert_eq!(current_surface(), None);
            r.fill_quad(
                renderer::Quad {
                    bounds: layout.bounds(),
                    snap: false,
                    ..Default::default()
                },
                Color::from_rgb(0.0, 0.0, 1.0),
            );
        }
        fn update(
            &mut self,
            _: &Event,
            _: Layout<'_>,
            _: mouse::Cursor,
            _: &iced::Renderer,
            _: &mut dyn Clipboard,
            shell: &mut Shell<'_, ()>,
        ) {
            assert_eq!(current_surface(), None);
            shell.publish(());
        }
        fn mouse_interaction(
            &self,
            _: Layout<'_>,
            _: mouse::Cursor,
            _: &iced::Renderer,
        ) -> mouse::Interaction {
            assert_eq!(current_surface(), None);
            mouse::Interaction::Pointer
        }
        fn overlay<'b>(
            &'b mut self,
            _: Layout<'b>,
            _: &iced::Renderer,
        ) -> Option<overlay::Element<'b, (), Theme, iced::Renderer>> {
            assert_eq!(current_surface(), None);
            Some(overlay::Element::new(Box::new(Popup(self.0))))
        }
    }
    #[test]
    fn excluded_corner_blocks_targeting_but_delivers_releases_and_portal_escapes() {
        let mut r = renderer();
        let seen = Rc::new(RefCell::new(vec![]));
        let mut element: Element<'_, ()> =
            Boundary::new(Boundary::new(Element::new(Events(seen.clone())), 0.0), 20.0).into();
        let (mut tree, node) = measure(&mut element, &r);
        let layout = Layout::new(&node);
        let view = Rectangle::with_size(Size::new(80.0, 52.0));
        let corner = mouse::Cursor::Available(Point::new(4.5, 4.5));
        let center = mouse::Cursor::Available(Point::new(36.0, 24.0));
        let mut messages = vec![];
        for event in [
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
        ] {
            element.as_widget_mut().update(
                &mut tree,
                &event,
                layout,
                corner,
                &r,
                &mut clipboard::Null,
                &mut Shell::new(&mut messages),
                &view,
            );
        }
        assert_eq!(*seen.borrow(), vec![(false, 2), (false, 2)]);
        let touch = Event::Touch(iced::touch::Event::FingerPressed {
            id: iced::touch::Finger(1),
            position: Point::new(4.5, 4.5),
        });
        element.as_widget_mut().update(
            &mut tree,
            &touch,
            layout,
            corner,
            &r,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
            &view,
        );
        assert_eq!(seen.borrow().len(), 2);
        assert_eq!(
            element
                .as_widget()
                .mouse_interaction(&tree, layout, corner, &view, &r),
            mouse::Interaction::None
        );
        assert_eq!(
            element
                .as_widget()
                .mouse_interaction(&tree, layout, center, &view, &r),
            mouse::Interaction::Pointer
        );
        let shape = SurfaceShape::new(bounds(), 20.0);
        let mut popup = with_surface(shape, || {
            element
                .as_widget_mut()
                .overlay(&mut tree, layout, &r, &view, Vector::ZERO)
        })
        .unwrap();
        let popup_node = with_surface(shape, || popup.as_overlay_mut().layout(&r, view.size()));
        r.reset(view);
        with_surface(shape, || {
            popup.as_overlay().draw(
                &mut r,
                &Theme::light(),
                &renderer::Style::default(),
                Layout::new(&popup_node),
                corner,
            );
            popup.as_overlay_mut().update(
                &Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)),
                Layout::new(&popup_node),
                corner,
                &r,
                &mut clipboard::Null,
                &mut Shell::new(&mut messages),
            );
            assert_eq!(
                popup
                    .as_overlay()
                    .mouse_interaction(Layout::new(&popup_node), corner, &r),
                mouse::Interaction::Pointer
            );
            let mut nested = popup
                .as_overlay_mut()
                .overlay(Layout::new(&popup_node), &r)
                .unwrap();
            let nested_node = nested.as_overlay_mut().layout(&r, view.size());
            nested.as_overlay().draw(
                &mut r,
                &Theme::light(),
                &renderer::Style::default(),
                Layout::new(&nested_node),
                corner,
            );
        });
        assert_eq!(messages, vec![()]);
        let image = r.screenshot(Size::new(80, 52), 1.0, Color::BLACK);
        assert_eq!(pixel(&image, 4, 4), &[0, 0, 255, 255]);
        assert_eq!(current_surface(), None);
    }
}
