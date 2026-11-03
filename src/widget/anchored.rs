//! `Anchored`: a surface placed against a base widget by [`primitives::anchor`](crate::primitives::anchor).
//! It is the widget that applies the anchor primitive to *inline* layout: the base keeps its
//! own size, and the floating surface is laid out against the base's bounds, over it.
//!
//! * [`Indicator`]: a badge, dot or count on a corner of an avatar, button or icon
//!   (`Relation::Straddle`, Mantine's `Indicator`).
//! * [`Dock`](super::Dock) docks a bar, pane or sheet inside the base against an edge
//!   (`Relation::Inside`) over the content, inline or on an overlay.
//!
//! The floating surface is drawn over the base and gets events first. Placement is a pure
//! function (`place_sized`, unit tested); this widget only lays the two children out and routes
//! events, draws and overlays. There is no viewport here, so an `Outside` placement does not
//! flip or shift; use [`Popover`](super::Popover) for popovers.

use super::Element;
use crate::Theme;
use crate::primitives::anchor::{Align, Placement, Relation, Side, Sizing, place_sized};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::{Event, Length, Point, Rectangle, Size, Vector, mouse};

/// A base with a floating surface placed against it. See the [module](self).
pub struct Anchored<'a, M, R = iced::Renderer> {
    /// `[base]` or `[base, floating]`.
    children: Vec<iced::Element<'a, M, Theme, R>>,
    placement: Placement,
    sizing: Sizing,
}

impl<'a, M: 'a, R: iced::advanced::Renderer + 'a> Anchored<'a, M, R> {
    /// `floating` against `base`; with `None` only the base is shown (the surface is absent, so
    /// a hidden badge costs nothing).
    pub fn new(
        base: impl Into<iced::Element<'a, M, Theme, R>>,
        floating: Option<iced::Element<'a, M, Theme, R>>,
    ) -> Self {
        let mut children = vec![base.into()];
        children.extend(floating);
        Self {
            children,
            placement: Placement::default(),
            sizing: Sizing::default(),
        }
    }
    pub fn placement(mut self, p: Placement) -> Self {
        self.placement = p;
        self
    }
    pub fn sizing(mut self, s: Sizing) -> Self {
        self.sizing = s;
        self
    }
}

/// A big finite number standing in for "no viewport".
const FAR: f32 = 1.0e6;

impl<'a, M: 'a, R: iced::advanced::Renderer + 'a> Widget<M, Theme, R> for Anchored<'a, M, R> {
    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        let children: Vec<_> = self.children.iter().map(|c| c.as_widget()).collect();
        tree.diff_children(&children);
    }

    fn size(&self) -> Size<Length> {
        self.children[0].as_widget().size()
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &R, limits: &layout::Limits) -> layout::Node {
        let (base_tree, rest) = tree.children.split_first_mut().expect("a base child");
        let base = self.children[0]
            .as_widget_mut()
            .layout(base_tree, renderer, limits);
        let anchor = Rectangle::new(Point::ORIGIN, base.size());
        let mut nodes = vec![base];
        if let Some(floating) = self.children.get_mut(1) {
            let tree = &mut rest[0];
            // Content size first (inside the anchor for docked and centered surfaces, as roomy
            // as it likes for a badge), then the size the rules decide, then the position.
            let room = match self.placement.relation {
                Relation::Inside | Relation::Center => anchor.size(),
                _ => Size::new(FAR, FAR),
            };
            // One layout only (laying a text widget out twice loses its text): every dimension
            // except `Auto` is known from the anchor, so those become exact limits and `Auto`
            // ones get the room; the position then comes from the size the content took.
            let (min, max) = self.sizing.limits(anchor, &self.placement, room);
            let node =
                floating
                    .as_widget_mut()
                    .layout(tree, renderer, &layout::Limits::new(min, max));
            let placed = place_sized(
                anchor,
                node.size(),
                self.sizing,
                Size::new(FAR, FAR),
                self.placement,
            );
            nodes.push(node.move_to(placed.position));
        }
        layout::Node::with_children(anchor.size(), nodes)
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut R,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let mut kids = layout.children();
        let base = kids.next().expect("a base layout");
        self.children[0].as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            base,
            cursor,
            viewport,
        );
        if let (Some(floating), Some(l)) = (self.children.get(1), kids.next()) {
            // Its own layer: the renderer draws all the text of a layer after all of its quads,
            // so without one the base's text would show through the surface.
            renderer.with_layer(*viewport, |r| {
                floating
                    .as_widget()
                    .draw(&tree.children[1], r, theme, style, l, cursor, viewport);
            });
        }
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &R,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        let mut kids = layout.children();
        let base = kids.next().expect("a base layout");
        // The surface is on top: it sees the event first, and a press it handles does not
        // reach the base underneath.
        if let (Some(floating), Some(l)) = (self.children.get_mut(1), kids.next()) {
            floating.as_widget_mut().update(
                &mut tree.children[1],
                event,
                l,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
            if shell.is_event_captured() {
                return;
            }
        }
        self.children[0].as_widget_mut().update(
            &mut tree.children[0],
            event,
            base,
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
        renderer: &R,
    ) -> mouse::Interaction {
        let mut kids = layout.children();
        let base = kids.next().expect("a base layout");
        if let (Some(floating), Some(l)) = (self.children.get(1), kids.next())
            && cursor.is_over(l.bounds())
        {
            return floating.as_widget().mouse_interaction(
                &tree.children[1],
                l,
                cursor,
                viewport,
                renderer,
            );
        }
        self.children[0].as_widget().mouse_interaction(
            &tree.children[0],
            base,
            cursor,
            viewport,
            renderer,
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &R,
        operation: &mut dyn Operation,
    ) {
        for ((child, state), l) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().operate(state, l, renderer, operation);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &R,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, R>> {
        // Popovers of the base and of the surface (a menu button in a docked bar).
        overlay::from_children(
            &mut self.children,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, M: 'a, R: iced::advanced::Renderer + 'a> From<Anchored<'a, M, R>>
    for iced::Element<'a, M, Theme, R>
{
    fn from(a: Anchored<'a, M, R>) -> Self {
        iced::Element::new(a)
    }
}

// ---------------------------------------------------------------------------
// Indicator
// ---------------------------------------------------------------------------

/// A badge, dot or count on the corner of `child`, after Mantine's `Indicator`: the badge is
/// centered on the corner whatever its shape (`Relation::Straddle`).
///
/// ```ignore
/// Indicator::new(avatar(40.0, "Ada"), Badge::count(3, 99))          // top-right
/// Indicator::new(icon_button(Icon::Mail), dot).corner(Side::Bottom, Align::End).offset(4.0)
/// ```
pub struct Indicator<'a, M> {
    child: Element<'a, M>,
    badge: Option<Element<'a, M>>,
    side: Side,
    align: Align,
    offset: f32,
    cross_offset: f32,
}

impl<'a, M: 'a> Indicator<'a, M> {
    pub fn new(child: impl Into<Element<'a, M>>, badge: impl Into<Element<'a, M>>) -> Self {
        Self {
            child: child.into(),
            badge: Some(badge.into()),
            side: Side::Top,
            align: Align::End,
            offset: 0.0,
            cross_offset: 0.0,
        }
    }
    /// Which corner: the edge `side` and `Start` / `End` along it (`Center` for the middle of
    /// the edge). Default: top, end (top right).
    pub fn corner(mut self, side: Side, align: Align) -> Self {
        (self.side, self.align) = (side, align);
        self
    }
    /// Distance in from the corner, on both axes (Mantine `offset`). Default 0: centered on it.
    pub fn offset(mut self, px: f32) -> Self {
        self.offset = px;
        self
    }
    /// Slides the badge along the edge.
    pub fn cross_offset(mut self, px: f32) -> Self {
        self.cross_offset = px;
        self
    }
    /// Shows or hides the badge (Mantine `disabled`); hidden, only the child remains.
    pub fn show(mut self, show: bool) -> Self {
        if !show {
            self.badge = None;
        }
        self
    }
}

impl<'a, M: 'a> From<Indicator<'a, M>> for Element<'a, M> {
    fn from(i: Indicator<'a, M>) -> Self {
        Anchored::new(i.child, i.badge)
            .placement(Placement {
                align: i.align,
                offset: i.offset,
                cross_offset: i.cross_offset,
                ..Placement::straddle(i.side)
            })
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::anchor::Dim;
    use iced::advanced::widget::Tree;
    use iced::widget::space;

    type E = iced::Element<'static, (), Theme, ()>;

    fn sized(w: f32, h: f32) -> E {
        space()
            .width(Length::Fixed(w))
            .height(Length::Fixed(h))
            .into()
    }

    /// Lays `a` out in a 300x200 parent and returns (base bounds, floating bounds).
    fn lay(a: Anchored<'static, (), ()>) -> (Rectangle, Rectangle) {
        let mut el: E = a.into();
        let mut tree = Tree::new(&el);
        let node = el.as_widget_mut().layout(
            &mut tree,
            &(),
            &layout::Limits::new(Size::ZERO, Size::new(300.0, 200.0)),
        );
        let l = Layout::new(&node);
        let mut kids = l.children();
        let base = kids.next().unwrap().bounds();
        let floating = kids.next().unwrap().bounds();
        (base, floating)
    }

    #[test]
    fn a_badge_straddles_the_corner_of_its_child() {
        let (base, badge) = lay(Anchored::new(sized(40.0, 40.0), Some(sized(32.0, 12.0)))
            .placement(Placement {
                align: Align::End,
                ..Placement::straddle(Side::Top)
            }));
        assert_eq!(base.size(), Size::new(40.0, 40.0));
        // Centered on the top-right corner (40, 0).
        assert_eq!(badge.position(), Point::new(40.0 - 16.0, -6.0));
        assert_eq!(badge.size(), Size::new(32.0, 12.0));
    }

    #[test]
    fn a_footer_fills_the_width_at_the_bottom() {
        let (base, bar) = lay(Anchored::new(sized(300.0, 200.0), Some(sized(10.0, 48.0)))
            .placement(Placement::inside(Side::Bottom))
            .sizing(Sizing::new().width(Dim::MatchAnchor)));
        assert_eq!(base.size(), Size::new(300.0, 200.0));
        assert_eq!(
            (bar.position(), bar.size()),
            (Point::new(0.0, 152.0), Size::new(300.0, 48.0))
        );
    }

    #[test]
    fn a_drawer_fills_the_height_at_the_side() {
        let (_, pane) = lay(Anchored::new(sized(300.0, 200.0), Some(sized(10.0, 10.0)))
            .placement(Placement::inside(Side::Right))
            .sizing(
                Sizing::new()
                    .width(Dim::Fixed(120.0))
                    .height(Dim::MatchAnchor),
            ));
        assert_eq!(
            (pane.position(), pane.size()),
            (Point::new(180.0, 0.0), Size::new(120.0, 200.0))
        );
    }

    #[test]
    fn without_a_surface_only_the_base_is_laid_out() {
        let mut el: E = Anchored::new(sized(40.0, 40.0), None).into();
        let mut tree = Tree::new(&el);
        let node = el.as_widget_mut().layout(
            &mut tree,
            &(),
            &layout::Limits::new(Size::ZERO, Size::new(300.0, 200.0)),
        );
        assert_eq!(node.children().len(), 1);
        assert_eq!(node.size(), Size::new(40.0, 40.0));
    }
}
