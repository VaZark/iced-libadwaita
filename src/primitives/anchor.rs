//! CSS anchor-positioning style placement: where a floating surface sits relative to an anchor.
//!
//! The pure function [`place`] takes the anchor rectangle, the size of the floating surface and
//! the viewport, and returns a position. It is what `position-area` / `position-try-fallbacks:
//! flip-block` / `inset` do in CSS: pick a [`Side`] of the anchor, [`Align`] along that side, leave
//! an `offset`, then **flip** to the opposite side if the surface does not fit and **shift**
//! along the side so it stays inside the viewport (minus a `margin`).
//!
//! [`Dropdown`](crate::widget::Dropdown) uses it for its popover (`.side(..)`, `.offset(..)`).

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

/// A placement request.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub side: Side,
    pub align: Align,
    /// Gap between the anchor and the surface.
    pub offset: f32,
    /// Minimum distance kept from the viewport edges.
    pub margin: f32,
    /// Move to the opposite side when the preferred one does not fit.
    pub flip: bool,
    /// Slide along the side to stay inside the viewport.
    pub shift: bool,
}

impl Default for Placement {
    fn default() -> Self {
        Self {
            side: Side::Bottom,
            align: Align::Start,
            offset: 4.0,
            margin: 6.0,
            flip: true,
            shift: true,
        }
    }
}

/// The result of [`place`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    /// Top-left of the surface.
    pub position: Point,
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
    let need = |s: Side| {
        if s.vertical() {
            size.height
        } else {
            size.width
        }
    };
    let mut side = p.side;
    if p.flip && room(&anchor, bounds, &p, side) < need(side) {
        let other = side.opposite();
        // Flip when the other side fits, or at least has more room.
        if room(&anchor, bounds, &p, other) >= need(other)
            || room(&anchor, bounds, &p, other) > room(&anchor, bounds, &p, side)
        {
            side = other;
        }
    }
    let (mut x, mut y);
    // Cross-axis alignment first.
    let along = |start: f32, len: f32, own: f32| match p.align {
        Align::Start => start,
        Align::Center => start + (len - own) / 2.0,
        Align::End => start + len - own,
    };
    match side {
        Side::Top => {
            y = anchor.y - p.offset - size.height;
            x = along(anchor.x, anchor.width, size.width);
        }
        Side::Bottom => {
            y = anchor.y + anchor.height + p.offset;
            x = along(anchor.x, anchor.width, size.width);
        }
        Side::Left => {
            x = anchor.x - p.offset - size.width;
            y = along(anchor.y, anchor.height, size.height);
        }
        Side::Right => {
            x = anchor.x + anchor.width + p.offset;
            y = along(anchor.y, anchor.height, size.height);
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
        side,
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
}
