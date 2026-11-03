//! How a panel docked against an edge shares the space with the content beside it, with no
//! drawing: the geometry behind [`AdaptiveSplit`](crate::widget::AdaptiveSplit) (the sidebar of a
//! [`NavigationSplitView`](crate::widget::NavigationSplitView), beside the content or as a
//! [`CollapsedSidebar::Drawer`](crate::widget::CollapsedSidebar)) and
//! [`Dock`](crate::widget::Dock).
//!
//! A panel is [`extent`](resolve_extent) px across its edge and has slid `progress` (0 out of
//! sight, 1 fully in) in from that edge. Its [`Flow`] says what that does to the content:
//!
//! * [`Push`](Flow::Push): the panel is **in flow**: the content gives up exactly the room the
//!   panel has slid in (a sidebar beside the content, a docked pane);
//! * [`Overlay`](Flow::Overlay): the panel floats **over** the content, which keeps its size (a
//!   drawer, a sheet).
//!
//! [`arrange`] gives the rectangles, so a widget animating `progress` on a spring moves from one
//! to the other the same way for every side.

use crate::primitives::anchor::Side;
use iced::{Rectangle, Size};

/// Whether a docked panel displaces the content or floats over it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Flow {
    /// The content gives up the room the panel takes.
    #[default]
    Push,
    /// The panel is over the content, which keeps its size.
    Overlay,
}

/// How far a panel reaches across its edge, before it is resolved against the room there is.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Extent {
    /// Its own size (what it measures when laid out freely).
    #[default]
    Auto,
    Px(f32),
    /// A fraction of the room across that axis (0 to 1).
    Fraction(f32),
}

/// `extent` in px, for `total` px of room across the edge and a panel that measures `intrinsic`
/// when laid out freely. Never more than the room, never negative.
pub fn resolve_extent(extent: Extent, total: f32, intrinsic: f32) -> f32 {
    let px = match extent {
        Extent::Auto => intrinsic,
        Extent::Px(px) => px,
        Extent::Fraction(f) => total * f.clamp(0.0, 1.0),
    };
    px.clamp(0.0, total.max(0.0))
}

/// Where the parts go, in the bounds' own coordinates (origin at 0, 0).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Arrangement {
    /// The whole panel, slid `progress` of the way in (part of it is out of the bounds while it
    /// is not fully in).
    pub panel: Rectangle,
    /// The part of the panel that has slid in: what to clip it to.
    pub visible: Rectangle,
    /// The content: the bounds, less the room a [`Push`](Flow::Push) panel has taken.
    pub content: Rectangle,
}

/// The panel docked against `side` of `bounds`, `extent` px across, slid in `progress`.
pub fn arrange(bounds: Size, side: Side, extent: f32, progress: f32, flow: Flow) -> Arrangement {
    let (w, h) = (bounds.width, bounds.height);
    let e = extent.max(0.0);
    let slid = e * progress.clamp(0.0, 1.0);
    let rect = |x, y, w, h| Rectangle {
        x,
        y,
        width: w,
        height: h,
    };
    let (panel, visible, pushed) = match side {
        Side::Left => (
            rect(slid - e, 0.0, e, h),
            rect(0.0, 0.0, slid, h),
            rect(slid, 0.0, (w - slid).max(0.0), h),
        ),
        Side::Right => (
            rect(w - slid, 0.0, e, h),
            rect(w - slid, 0.0, slid, h),
            rect(0.0, 0.0, (w - slid).max(0.0), h),
        ),
        Side::Top => (
            rect(0.0, slid - e, w, e),
            rect(0.0, 0.0, w, slid),
            rect(0.0, slid, w, (h - slid).max(0.0)),
        ),
        Side::Bottom => (
            rect(0.0, h - slid, w, e),
            rect(0.0, h - slid, w, slid),
            rect(0.0, 0.0, w, (h - slid).max(0.0)),
        ),
    };
    let content = match flow {
        Flow::Push => pushed,
        Flow::Overlay => rect(0.0, 0.0, w, h),
    };
    Arrangement {
        panel,
        visible,
        content,
    }
}

/// Whether `side` is a left or right edge (the panel is a column, its extent a width).
pub fn horizontal(side: Side) -> bool {
    matches!(side, Side::Left | Side::Right)
}

#[cfg(test)]
mod tests {
    use super::*;

    const B: Size = Size::new(400.0, 300.0);

    #[test]
    fn a_pushing_panel_takes_exactly_the_room_it_has_slid_in() {
        let a = arrange(B, Side::Left, 100.0, 0.5, Flow::Push);
        assert_eq!(
            a.visible,
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: 50.0,
                height: 300.0
            }
        );
        assert_eq!(
            a.content,
            Rectangle {
                x: 50.0,
                y: 0.0,
                width: 350.0,
                height: 300.0
            }
        );
        assert_eq!(a.panel.x, -50.0, "half of it is still outside");
        let done = arrange(B, Side::Left, 100.0, 1.0, Flow::Push);
        assert_eq!(done.panel.x, 0.0);
        assert_eq!(done.content.x + done.content.width, 400.0);
    }

    #[test]
    fn an_overlay_panel_leaves_the_content_alone() {
        for p in [0.0, 0.4, 1.0] {
            let a = arrange(B, Side::Right, 120.0, p, Flow::Overlay);
            assert_eq!(
                a.content,
                Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: 400.0,
                    height: 300.0
                }
            );
        }
        let a = arrange(B, Side::Right, 120.0, 1.0, Flow::Overlay);
        assert_eq!(
            a.panel,
            Rectangle {
                x: 280.0,
                y: 0.0,
                width: 120.0,
                height: 300.0
            }
        );
    }

    #[test]
    fn every_side_slides_in_from_its_own_edge() {
        let at = |s| arrange(B, s, 100.0, 0.0, Flow::Push).panel;
        assert_eq!(at(Side::Left).x, -100.0);
        assert_eq!(at(Side::Right).x, 400.0);
        assert_eq!(at(Side::Top).y, -100.0);
        assert_eq!(at(Side::Bottom).y, 300.0);
        // Fully in: a top bar and a bottom bar leave the rest to the content.
        let top = arrange(B, Side::Top, 40.0, 1.0, Flow::Push);
        assert_eq!(
            top.content,
            Rectangle {
                x: 0.0,
                y: 40.0,
                width: 400.0,
                height: 260.0
            }
        );
        let bottom = arrange(B, Side::Bottom, 40.0, 1.0, Flow::Push);
        assert_eq!(
            bottom.content,
            Rectangle {
                x: 0.0,
                y: 0.0,
                width: 400.0,
                height: 260.0
            }
        );
    }

    #[test]
    fn out_of_range_progress_and_extent_are_kept_sane() {
        let a = arrange(B, Side::Left, 500.0, 2.0, Flow::Push);
        assert_eq!(a.visible.width, 500.0);
        assert_eq!(a.content.width, 0.0, "the content never goes negative");
        let a = arrange(B, Side::Left, 100.0, -1.0, Flow::Push);
        assert_eq!(a.visible.width, 0.0);
    }

    #[test]
    fn an_extent_resolves_against_the_room_and_never_exceeds_it() {
        assert_eq!(resolve_extent(Extent::Px(120.0), 400.0, 80.0), 120.0);
        assert_eq!(resolve_extent(Extent::Fraction(0.25), 400.0, 80.0), 100.0);
        assert_eq!(resolve_extent(Extent::Auto, 400.0, 80.0), 80.0);
        assert_eq!(resolve_extent(Extent::Px(900.0), 400.0, 80.0), 400.0);
        assert_eq!(resolve_extent(Extent::Px(-5.0), 400.0, 80.0), 0.0);
    }

    #[test]
    fn left_and_right_edges_are_horizontal() {
        assert!(horizontal(Side::Left) && horizontal(Side::Right));
        assert!(!horizontal(Side::Top) && !horizontal(Side::Bottom));
    }
}
