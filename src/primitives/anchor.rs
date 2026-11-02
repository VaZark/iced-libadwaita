//! CSS anchor-positioning style placement: where a floating surface sits relative to an anchor.
//!
//! The pure function [`place`] takes the anchor rectangle, the size of the floating surface and
//! the viewport, and returns a position. It is what `position-area` / `position-try-fallbacks:
//! flip-block` / `inset` do in CSS: pick a [`Side`] of the anchor, [`Align`] along that side, leave
//! an `offset`, then **flip** to the opposite side if the surface does not fit and **shift**
//! along the side so it stays inside the viewport (minus a `margin`).
//!
//! Four kinds of placement ([`Relation`]): **outside** the anchor (popovers: a side, an
//! alignment, a gap, flip and shift), **inside** it docked to an edge (footers, banners, drawers,
//! sheets, toasts: the edge, an alignment along it, an inset) and **centered** in it (dialogs) and **straddling** an edge or corner (badges and indicators on an avatar or button).
//! [`place_sized`] also decides the surface's size from [`Sizing`]: `Auto`, `Fixed`, as large as
//! the anchor (`MatchAnchor`), a fraction of it, or `Fill`, with min / max and a `fit` that
//! shrinks a popover to the room it has, like CSS `anchor-size()`.
//!
//! [`Dropdown`](crate::widget::Dropdown) and `Autocomplete` use it for their popovers.

use iced::{Point, Rectangle, Size};

/// The side of the anchor the surface sits on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Side {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

impl Side {
    pub fn opposite(self) -> Self {
        match self {
            Side::Top => Side::Bottom,
            Side::Bottom => Side::Top,
            Side::Left => Side::Right,
            Side::Right => Side::Left,
        }
    }
    fn vertical(self) -> bool {
        matches!(self, Side::Top | Side::Bottom)
    }
}

/// How the surface lines up with the anchor along the side (`Start`: left or top edges flush).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Align {
    #[default]
    Start,
    Center,
    End,
}

/// Where the surface sits relative to the anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Relation {
    /// Beside the anchor, outside it (popovers, tooltips, menus): `side` is the side of the
    /// anchor it touches, with a gap, and it flips and shifts to stay in the viewport.
    #[default]
    Outside,
    /// Docked inside the anchor against the edge `side`, `offset` from it (footers, banners,
    /// drawers, bottom sheets, toasts): `align` runs along that edge. It never leaves the anchor.
    Inside,
    /// In the middle of the anchor (dialogs, status overlays); `side` is ignored.
    Center,
    /// Centered on an edge or corner of the anchor, half outside and half in (a badge or an
    /// indicator dot on an avatar or button, a notification count, a ribbon). `side` is the edge,
    /// `align` the corner (`Start` / `End`) or the middle of it (`Center`); `offset` moves it
    /// inward from the corner on both axes (Mantine `Indicator`'s `offset`). Unlike `Inside` with
    /// a negative inset, the badge's own size is accounted for, so any shape sits exactly on the
    /// corner. It is not kept inside the anchor or the viewport.
    Straddle,
}

/// A placement request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub side: Side,
    pub align: Align,
    pub relation: Relation,
    /// `Outside`: the gap between the anchor and the surface. `Inside`: the inset from the
    /// anchor's edges (all four, so a `Fill` surface keeps this much on each side).
    pub offset: f32,
    /// Sliding along the edge (the cross axis), after `align`: positive is right or down.
    pub cross_offset: f32,
    /// Minimum distance kept from the viewport edges (`Outside` only).
    pub margin: f32,
    /// Move to the opposite side when the preferred one does not fit (`Outside` only).
    pub flip: bool,
    /// Slide along the side to stay inside the viewport (`Outside`) or the anchor (`Inside`).
    pub shift: bool,
}

impl Default for Placement {
    fn default() -> Self {
        Self {
            side: Side::Bottom,
            align: Align::Start,
            relation: Relation::Outside,
            offset: 4.0,
            cross_offset: 0.0,
            margin: 6.0,
            flip: true,
            shift: true,
        }
    }
}

impl Placement {
    /// Docked inside the anchor at the edge `side` (footer: `Bottom`, banner: `Top`, drawer:
    /// `Left` or `Right`), flush with the edge unless [`offset`](Self::offset) insets it.
    pub fn inside(side: Side) -> Self {
        Self {
            side,
            relation: Relation::Inside,
            offset: 0.0,
            ..Self::default()
        }
    }
    /// Centered on the edge `side` of the anchor, and with `align` on its corner: a badge on the
    /// top-right of an avatar is `Placement::straddle(Side::Top)` with `align: Align::End`.
    pub fn straddle(side: Side) -> Self {
        Self {
            side,
            relation: Relation::Straddle,
            offset: 0.0,
            ..Self::default()
        }
    }
    /// Centered in the anchor.
    pub fn centered() -> Self {
        Self {
            relation: Relation::Center,
            offset: 0.0,
            ..Self::default()
        }
    }
}

/// How one dimension of the surface is decided (CSS `width` with `anchor-size()`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Dim {
    /// The content's own size (the `intrinsic` size passed to [`place_sized`]).
    #[default]
    Auto,
    Fixed(f32),
    /// As large as the anchor on that axis (`anchor-size(width)`): a popover as wide as its
    /// field, a footer as wide as its window.
    MatchAnchor,
    /// A fraction of the anchor on that axis (a drawer of 0.8 of the width).
    Fraction(f32),
    /// All the room: the anchor minus its insets when `Inside`, the room left on that side (main
    /// axis) or in the viewport (cross axis) when `Outside`.
    Fill,
}

/// The size rules of the surface.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sizing {
    pub width: Dim,
    pub height: Dim,
    pub min: Size,
    pub max: Size,
    /// `Outside`: shrink the surface on the main axis to the room on its side (a menu that
    /// scrolls instead of leaving the screen), never below `min`.
    pub fit: bool,
}

impl Default for Sizing {
    fn default() -> Self {
        Self {
            width: Dim::Auto,
            height: Dim::Auto,
            min: Size::ZERO,
            max: Size::new(f32::INFINITY, f32::INFINITY),
            fit: false,
        }
    }
}

impl Sizing {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn width(mut self, d: Dim) -> Self {
        self.width = d;
        self
    }
    pub fn height(mut self, d: Dim) -> Self {
        self.height = d;
        self
    }
    pub fn min(mut self, size: Size) -> Self {
        self.min = size;
        self
    }
    pub fn max(mut self, size: Size) -> Self {
        self.max = size;
        self
    }
    pub fn fit(mut self, fit: bool) -> Self {
        self.fit = fit;
        self
    }

    fn resolve(&self, intrinsic: Size, anchor: Size, available: Size) -> Size {
        let one = |d: Dim, own: f32, anchor: f32, avail: f32, min: f32, max: f32| {
            let v = match d {
                Dim::Auto => own,
                Dim::Fixed(px) => px,
                Dim::MatchAnchor => anchor,
                Dim::Fraction(f) => anchor * f,
                Dim::Fill => avail,
            };
            v.max(min).min(max.max(min))
        };
        Size::new(
            one(
                self.width,
                intrinsic.width,
                anchor.width,
                available.width,
                self.min.width,
                self.max.width,
            ),
            one(
                self.height,
                intrinsic.height,
                anchor.height,
                available.height,
                self.min.height,
                self.max.height,
            ),
        )
    }
}

/// The result of [`place`] / [`place_sized`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    /// Top-left of the surface.
    pub position: Point,
    /// The size to lay the surface out at (the input size for [`place`]).
    pub size: Size,
    /// The side actually used (differs from the request after a flip).
    pub side: Side,
}

/// Room available on `side` of `anchor` inside `bounds`, after the offset and margin.
fn room(anchor: &Rectangle, bounds: Size, p: &Placement, side: Side) -> f32 {
    match side {
        Side::Top => anchor.y - p.offset - p.margin,
        Side::Bottom => bounds.height - (anchor.y + anchor.height) - p.offset - p.margin,
        Side::Left => anchor.x - p.offset - p.margin,
        Side::Right => bounds.width - (anchor.x + anchor.width) - p.offset - p.margin,
    }
}

/// Positions a surface of `size` against `anchor` inside a viewport of `bounds`.
pub fn place(anchor: Rectangle, size: Size, bounds: Size, p: Placement) -> Placed {
    place_sized(anchor, size, Sizing::default(), bounds, p)
}

/// Decides the size (by `sizing`, from the `intrinsic` content size) and the position of a
/// surface against `anchor` inside a viewport of `bounds`. See [`Relation`] for the three kinds
/// of placement.
pub fn place_sized(
    anchor: Rectangle,
    intrinsic: Size,
    sizing: Sizing,
    bounds: Size,
    p: Placement,
) -> Placed {
    match p.relation {
        Relation::Outside => outside(anchor, intrinsic, sizing, bounds, p),
        Relation::Inside => inside(anchor, intrinsic, sizing, p),
        Relation::Center => center(anchor, intrinsic, sizing, p),
        Relation::Straddle => straddle(anchor, intrinsic, sizing, p),
    }
}

/// Alignment of a length `own` within `len` starting at `start`.
fn along(align: Align, start: f32, len: f32, own: f32) -> f32 {
    match align {
        Align::Start => start,
        Align::Center => start + (len - own) / 2.0,
        Align::End => start + len - own,
    }
}

fn outside(
    anchor: Rectangle,
    intrinsic: Size,
    sizing: Sizing,
    bounds: Size,
    p: Placement,
) -> Placed {
    let anchor_size = anchor.size();
    // The room a surface on `side` has: along the main axis what is left on that side, across it
    // the viewport minus the margins.
    let available = |side: Side| {
        let main = room(&anchor, bounds, &p, side).max(0.0);
        if side.vertical() {
            Size::new((bounds.width - 2.0 * p.margin).max(0.0), main)
        } else {
            Size::new(main, (bounds.height - 2.0 * p.margin).max(0.0))
        }
    };
    let sized = |side: Side| {
        let avail = available(side);
        let mut s = sizing.resolve(intrinsic, anchor_size, avail);
        if sizing.fit {
            if side.vertical() {
                s.height = s.height.min(avail.height).max(sizing.min.height);
            } else {
                s.width = s.width.min(avail.width).max(sizing.min.width);
            }
        }
        s
    };
    let need = |side: Side, s: Size| if side.vertical() { s.height } else { s.width };
    let mut side = p.side;
    let mut size = sized(side);
    if p.flip && room(&anchor, bounds, &p, side) < need(side, size) {
        let other = side.opposite();
        let other_size = sized(other);
        // Flip when the other side fits, or at least has more room.
        if room(&anchor, bounds, &p, other) >= need(other, other_size)
            || room(&anchor, bounds, &p, other) > room(&anchor, bounds, &p, side)
        {
            side = other;
            size = other_size;
        }
    }
    let (mut x, mut y);
    match side {
        Side::Top => {
            y = anchor.y - p.offset - size.height;
            x = along(p.align, anchor.x, anchor.width, size.width) + p.cross_offset;
        }
        Side::Bottom => {
            y = anchor.y + anchor.height + p.offset;
            x = along(p.align, anchor.x, anchor.width, size.width) + p.cross_offset;
        }
        Side::Left => {
            x = anchor.x - p.offset - size.width;
            y = along(p.align, anchor.y, anchor.height, size.height) + p.cross_offset;
        }
        Side::Right => {
            x = anchor.x + anchor.width + p.offset;
            y = along(p.align, anchor.y, anchor.height, size.height) + p.cross_offset;
        }
    }
    let clamp = |v: f32, len: f32, total: f32| v.min(total - len - p.margin).max(p.margin);
    if p.shift {
        if side.vertical() {
            x = clamp(x, size.width, bounds.width);
        } else {
            y = clamp(y, size.height, bounds.height);
        }
    }
    // The main axis never leaves the viewport either, even when neither side fits.
    if side.vertical() {
        y = y.max(p.margin);
    } else {
        x = x.max(p.margin);
    }
    Placed {
        position: Point::new(x, y),
        size,
        side,
    }
}

/// The anchor shrunk by `inset` on every side.
fn inset(anchor: Rectangle, inset: f32) -> Rectangle {
    Rectangle::new(
        Point::new(anchor.x + inset, anchor.y + inset),
        Size::new(
            (anchor.width - 2.0 * inset).max(0.0),
            (anchor.height - 2.0 * inset).max(0.0),
        ),
    )
}

/// Keeps `v .. v + len` inside `start .. start + total` (pinned to `start` if it is longer).
fn within(v: f32, len: f32, start: f32, total: f32) -> f32 {
    v.min(start + total - len).max(start)
}

fn inside(anchor: Rectangle, intrinsic: Size, sizing: Sizing, p: Placement) -> Placed {
    let inner = inset(anchor, p.offset);
    let size = sizing.resolve(intrinsic, anchor.size(), inner.size());
    let (mut x, mut y);
    match p.side {
        Side::Top => {
            y = inner.y;
            x = along(p.align, inner.x, inner.width, size.width) + p.cross_offset;
        }
        Side::Bottom => {
            y = inner.y + inner.height - size.height;
            x = along(p.align, inner.x, inner.width, size.width) + p.cross_offset;
        }
        Side::Left => {
            x = inner.x;
            y = along(p.align, inner.y, inner.height, size.height) + p.cross_offset;
        }
        Side::Right => {
            x = inner.x + inner.width - size.width;
            y = along(p.align, inner.y, inner.height, size.height) + p.cross_offset;
        }
    }
    if p.shift {
        x = within(x, size.width, inner.x, inner.width);
        y = within(y, size.height, inner.y, inner.height);
    }
    Placed {
        position: Point::new(x, y),
        size,
        side: p.side,
    }
}

fn straddle(anchor: Rectangle, intrinsic: Size, sizing: Sizing, p: Placement) -> Placed {
    let size = sizing.resolve(intrinsic, anchor.size(), anchor.size());
    let (right, bottom) = (anchor.x + anchor.width, anchor.y + anchor.height);
    // The corner (or middle) the surface is centered on, `offset` in from it on both axes.
    let across = |start: f32, end: f32| match p.align {
        Align::Start => start + p.offset,
        Align::Center => (start + end) / 2.0,
        Align::End => end - p.offset,
    };
    let (cx, cy) = match p.side {
        Side::Top => (
            across(anchor.x, right) + p.cross_offset,
            anchor.y + p.offset,
        ),
        Side::Bottom => (across(anchor.x, right) + p.cross_offset, bottom - p.offset),
        Side::Left => (
            anchor.x + p.offset,
            across(anchor.y, bottom) + p.cross_offset,
        ),
        Side::Right => (right - p.offset, across(anchor.y, bottom) + p.cross_offset),
    };
    Placed {
        position: Point::new(cx - size.width / 2.0, cy - size.height / 2.0),
        size,
        side: p.side,
    }
}

fn center(anchor: Rectangle, intrinsic: Size, sizing: Sizing, p: Placement) -> Placed {
    let inner = inset(anchor, p.offset);
    let size = sizing.resolve(intrinsic, anchor.size(), inner.size());
    let mut x = inner.x + (inner.width - size.width) / 2.0 + p.cross_offset;
    let mut y = inner.y + (inner.height - size.height) / 2.0;
    if p.shift {
        x = within(x, size.width, inner.x, inner.width);
        y = within(y, size.height, inner.y, inner.height);
    }
    Placed {
        position: Point::new(x, y),
        size,
        side: p.side,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEW: Size = Size::new(400.0, 300.0);
    const S: Size = Size::new(100.0, 80.0);

    fn anchor(x: f32, y: f32) -> Rectangle {
        Rectangle::new(Point::new(x, y), Size::new(60.0, 30.0))
    }
    fn p(side: Side, align: Align) -> Placement {
        Placement {
            side,
            align,
            offset: 4.0,
            margin: 6.0,
            ..Placement::default()
        }
    }

    #[test]
    fn sits_below_with_the_offset_and_start_alignment() {
        let r = place(anchor(50.0, 50.0), S, VIEW, p(Side::Bottom, Align::Start));
        assert_eq!((r.position, r.side), (Point::new(50.0, 84.0), Side::Bottom));
    }

    #[test]
    fn aligns_center_and_end() {
        let c = place(anchor(100.0, 50.0), S, VIEW, p(Side::Bottom, Align::Center));
        assert_eq!(c.position.x, 80.0);
        let e = place(anchor(100.0, 50.0), S, VIEW, p(Side::Bottom, Align::End));
        assert_eq!(e.position.x, 60.0);
    }

    #[test]
    fn flips_above_when_there_is_no_room_below() {
        let r = place(anchor(50.0, 250.0), S, VIEW, p(Side::Bottom, Align::Start));
        assert_eq!(r.side, Side::Top);
        assert_eq!(r.position.y, 250.0 - 4.0 - 80.0);
    }

    #[test]
    fn does_not_flip_when_disabled_or_when_the_other_side_is_worse() {
        let mut q = p(Side::Bottom, Align::Start);
        q.flip = false;
        assert_eq!(place(anchor(50.0, 250.0), S, VIEW, q).side, Side::Bottom);
        // Anchor near the top of a short viewport: below is tight but above is worse.
        let r = place(
            anchor(50.0, 10.0),
            S,
            Size::new(400.0, 100.0),
            p(Side::Bottom, Align::Start),
        );
        assert_eq!(r.side, Side::Bottom);
    }

    #[test]
    fn shifts_along_the_side_to_stay_inside() {
        let r = place(anchor(380.0, 50.0), S, VIEW, p(Side::Bottom, Align::Start));
        assert_eq!(r.position.x, 400.0 - 100.0 - 6.0);
        let l = place(anchor(-20.0, 50.0), S, VIEW, p(Side::Bottom, Align::Start));
        assert_eq!(l.position.x, 6.0);
    }

    #[test]
    fn side_placements_flip_horizontally() {
        let r = place(anchor(330.0, 100.0), S, VIEW, p(Side::Right, Align::Start));
        assert_eq!(r.side, Side::Left);
        assert_eq!(r.position.x, 330.0 - 4.0 - 100.0);
        let ok = place(anchor(100.0, 100.0), S, VIEW, p(Side::Right, Align::Center));
        assert_eq!(ok.position, Point::new(164.0, 75.0));
    }

    // -- inside, centered, sizing -------------------------------------------------------------

    const PARENT: Rectangle = Rectangle {
        x: 10.0,
        y: 20.0,
        width: 300.0,
        height: 200.0,
    };

    fn dock(side: Side, align: Align, inset: f32) -> Placement {
        Placement {
            align,
            offset: inset,
            ..Placement::inside(side)
        }
    }

    #[test]
    fn all_twelve_outside_placements() {
        let a = Rectangle::new(Point::new(150.0, 120.0), Size::new(60.0, 30.0));
        let s = Size::new(40.0, 20.0);
        let gap = 4.0;
        for side in [Side::Top, Side::Bottom, Side::Left, Side::Right] {
            for align in [Align::Start, Align::Center, Align::End] {
                let r = place(
                    a,
                    s,
                    Size::new(600.0, 600.0),
                    Placement {
                        side,
                        align,
                        offset: gap,
                        flip: false,
                        ..Placement::default()
                    },
                );
                let (x, y) = (r.position.x, r.position.y);
                // The gap to the anchor is exact on the main axis...
                match side {
                    Side::Top => assert_eq!(y + s.height + gap, a.y, "{side:?} {align:?}"),
                    Side::Bottom => assert_eq!(y, a.y + a.height + gap, "{side:?} {align:?}"),
                    Side::Left => assert_eq!(x + s.width + gap, a.x, "{side:?} {align:?}"),
                    Side::Right => assert_eq!(x, a.x + a.width + gap, "{side:?} {align:?}"),
                }
                // ...and the alignment is along the edge.
                let (own, len, at, start) = if side.vertical() {
                    (s.width, a.width, x, a.x)
                } else {
                    (s.height, a.height, y, a.y)
                };
                let want = match align {
                    Align::Start => start,
                    Align::Center => start + (len - own) / 2.0,
                    Align::End => start + len - own,
                };
                assert_eq!(at, want, "{side:?} {align:?}");
            }
        }
    }

    #[test]
    fn the_cross_offset_slides_along_the_edge() {
        let mut q = p(Side::Bottom, Align::Start);
        q.cross_offset = 12.0;
        assert_eq!(place(anchor(50.0, 50.0), S, VIEW, q).position.x, 62.0);
        let mut q = p(Side::Right, Align::Start);
        q.cross_offset = -8.0;
        assert_eq!(place(anchor(100.0, 100.0), S, VIEW, q).position.y, 92.0);
    }

    #[test]
    fn a_footer_docks_to_the_bottom_edge_and_fills_the_width() {
        // Inside, bottom, as wide as the parent less the inset on both sides, its own height.
        let sizing = Sizing::new().width(Dim::Fill);
        let r = place_sized(
            PARENT,
            Size::new(0.0, 48.0),
            sizing,
            VIEW,
            dock(Side::Bottom, Align::Start, 8.0),
        );
        assert_eq!(r.size, Size::new(284.0, 48.0));
        assert_eq!(r.position, Point::new(18.0, 20.0 + 200.0 - 8.0 - 48.0));
    }

    #[test]
    fn a_banner_docks_to_the_top_edge() {
        let sizing = Sizing::new().width(Dim::MatchAnchor);
        let r = place_sized(
            PARENT,
            Size::new(0.0, 40.0),
            sizing,
            VIEW,
            Placement::inside(Side::Top),
        );
        assert_eq!(r.position, Point::new(10.0, 20.0));
        assert_eq!(r.size, Size::new(300.0, 40.0));
    }

    #[test]
    fn a_drawer_docks_to_a_side_and_fills_the_height() {
        let sizing = Sizing::new().width(Dim::Fixed(120.0)).height(Dim::Fill);
        let l = place_sized(
            PARENT,
            Size::ZERO,
            sizing,
            VIEW,
            Placement::inside(Side::Left),
        );
        assert_eq!(
            (l.position, l.size),
            (Point::new(10.0, 20.0), Size::new(120.0, 200.0))
        );
        let r = place_sized(
            PARENT,
            Size::ZERO,
            sizing,
            VIEW,
            Placement::inside(Side::Right),
        );
        assert_eq!(r.position, Point::new(190.0, 20.0));
        // A fraction of the parent, as a drawer that takes most of a phone.
        let f = Sizing::new().width(Dim::Fraction(0.8)).height(Dim::Fill);
        let r = place_sized(PARENT, Size::ZERO, f, VIEW, Placement::inside(Side::Right));
        assert_eq!((r.size.width, r.position.x), (240.0, 70.0));
    }

    #[test]
    fn an_inside_surface_aligns_along_its_edge_and_stays_in_the_parent() {
        let own = Size::new(60.0, 30.0);
        let c = place_sized(
            PARENT,
            own,
            Sizing::new(),
            VIEW,
            dock(Side::Bottom, Align::Center, 0.0),
        );
        assert_eq!(c.position, Point::new(10.0 + 120.0, 190.0));
        let e = place_sized(
            PARENT,
            own,
            Sizing::new(),
            VIEW,
            dock(Side::Bottom, Align::End, 0.0),
        );
        assert_eq!(e.position.x, 250.0);
        // Slid past the edge by the cross offset: held inside the parent.
        let mut q = dock(Side::Bottom, Align::End, 0.0);
        q.cross_offset = 40.0;
        assert_eq!(
            place_sized(PARENT, own, Sizing::new(), VIEW, q).position.x,
            250.0
        );
        q.shift = false;
        assert_eq!(
            place_sized(PARENT, own, Sizing::new(), VIEW, q).position.x,
            290.0
        );
    }

    #[test]
    fn a_surface_can_be_centered_in_the_anchor() {
        let r = place_sized(
            PARENT,
            Size::new(100.0, 60.0),
            Sizing::new(),
            VIEW,
            Placement::centered(),
        );
        assert_eq!(r.position, Point::new(110.0, 90.0));
    }

    #[test]
    fn a_popover_can_match_the_anchor_width() {
        let sizing = Sizing::new().width(Dim::MatchAnchor);
        let r = place_sized(
            anchor(50.0, 50.0),
            Size::new(10.0, 80.0),
            sizing,
            VIEW,
            p(Side::Bottom, Align::Start),
        );
        assert_eq!(r.size, Size::new(60.0, 80.0));
        // With min and max applied.
        let sizing = Sizing::new()
            .width(Dim::MatchAnchor)
            .min(Size::new(120.0, 0.0));
        let r = place_sized(
            anchor(50.0, 50.0),
            Size::new(10.0, 80.0),
            sizing,
            VIEW,
            p(Side::Bottom, Align::Start),
        );
        assert_eq!(r.size.width, 120.0);
    }

    #[test]
    fn fit_shrinks_the_surface_to_the_room_on_its_side() {
        // 300 high, anchor at y 50..80: 300 - 80 - 4 - 6 = 210 below, 40 above. A 400 px
        // surface that cannot fit either side stays below and shrinks to the room.
        let sizing = Sizing::new().fit(true);
        let r = place_sized(
            anchor(50.0, 50.0),
            Size::new(100.0, 400.0),
            sizing,
            VIEW,
            p(Side::Bottom, Align::Start),
        );
        assert_eq!((r.side, r.size.height), (Side::Bottom, 210.0));
        // Never below the minimum.
        let sizing = Sizing::new().fit(true).min(Size::new(0.0, 250.0));
        let r = place_sized(
            anchor(50.0, 50.0),
            Size::new(100.0, 400.0),
            sizing,
            VIEW,
            p(Side::Bottom, Align::Start),
        );
        assert_eq!(r.size.height, 250.0);
    }

    #[test]
    fn fill_outside_takes_the_room_on_the_main_axis() {
        let sizing = Sizing::new().height(Dim::Fill);
        let r = place_sized(
            anchor(50.0, 50.0),
            Size::new(100.0, 10.0),
            sizing,
            VIEW,
            p(Side::Bottom, Align::Start),
        );
        assert_eq!(r.size.height, 210.0);
    }

    #[test]
    fn place_is_place_sized_with_the_content_size() {
        let a = place(anchor(50.0, 50.0), S, VIEW, p(Side::Bottom, Align::Start));
        assert_eq!(a.size, S);
    }

    #[test]
    fn a_negative_inset_overflows_a_square_badge_onto_the_corner() {
        // Inside, top-right, inset -6: a 12x12 badge's center lands on the corner (310, 20).
        let q = Placement {
            align: Align::End,
            offset: -6.0,
            ..Placement::inside(Side::Top)
        };
        let r = place_sized(PARENT, Size::new(12.0, 12.0), Sizing::new(), VIEW, q);
        assert_eq!(r.position, Point::new(304.0, 14.0));
        // A pill is not: one scalar cannot center both axes (x is 10 px short of the corner).
        let r = place_sized(PARENT, Size::new(32.0, 12.0), Sizing::new(), VIEW, q);
        assert_eq!(r.position, Point::new(284.0, 14.0));
    }

    #[test]
    fn straddle_centers_a_badge_on_the_corner_whatever_its_shape() {
        let corner = |side, align, size: Size, offset| {
            let q = Placement {
                align,
                offset,
                ..Placement::straddle(side)
            };
            let r = place_sized(PARENT, size, Sizing::new(), VIEW, q);
            // The center of the surface.
            Point::new(
                r.position.x + size.width / 2.0,
                r.position.y + size.height / 2.0,
            )
        };
        let dot = Size::new(12.0, 12.0);
        let pill = Size::new(32.0, 12.0);
        // Top-right (310, 20), top-left (10, 20), bottom-right (310, 220), bottom-left (10, 220).
        assert_eq!(
            corner(Side::Top, Align::End, dot, 0.0),
            Point::new(310.0, 20.0)
        );
        assert_eq!(
            corner(Side::Top, Align::End, pill, 0.0),
            Point::new(310.0, 20.0)
        );
        assert_eq!(
            corner(Side::Top, Align::Start, pill, 0.0),
            Point::new(10.0, 20.0)
        );
        assert_eq!(
            corner(Side::Bottom, Align::End, pill, 0.0),
            Point::new(310.0, 220.0)
        );
        assert_eq!(
            corner(Side::Bottom, Align::Start, dot, 0.0),
            Point::new(10.0, 220.0)
        );
        // Side edges: the left edge, top and bottom corners, and the middle.
        assert_eq!(
            corner(Side::Left, Align::Start, pill, 0.0),
            Point::new(10.0, 20.0)
        );
        assert_eq!(
            corner(Side::Right, Align::End, pill, 0.0),
            Point::new(310.0, 220.0)
        );
        assert_eq!(
            corner(Side::Right, Align::Center, dot, 0.0),
            Point::new(310.0, 120.0)
        );
        assert_eq!(
            corner(Side::Top, Align::Center, dot, 0.0),
            Point::new(160.0, 20.0)
        );
    }

    #[test]
    fn straddle_offset_moves_the_badge_inward_along_both_axes() {
        // An avatar's badge sits 4 px in from the corner (Mantine Indicator's `offset`).
        let q = Placement {
            align: Align::End,
            offset: 4.0,
            ..Placement::straddle(Side::Top)
        };
        let r = place_sized(PARENT, Size::new(12.0, 12.0), Sizing::new(), VIEW, q);
        assert_eq!(r.position, Point::new(310.0 - 4.0 - 6.0, 20.0 + 4.0 - 6.0));
        // And the cross offset slides it along the edge.
        let q = Placement {
            cross_offset: -3.0,
            ..q
        };
        assert_eq!(
            place_sized(PARENT, Size::new(12.0, 12.0), Sizing::new(), VIEW, q)
                .position
                .x,
            297.0
        );
    }
}
