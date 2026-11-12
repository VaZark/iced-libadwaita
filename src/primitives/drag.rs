//! Dragging something onto something: what the drop target says about the drop, and which of
//! several nested targets gets it. Headless: the widgets (`DragSource`, `DropZone`) apply it.
//!
//! A drop target may sit inside another (a task row inside a list, each of which accepts a
//! drop). The rule is the usual one: the innermost target under the pointer takes the drop and
//! the hint, and the ones around it do not. Targets are visited inside-out, so "first to claim
//! wins", and the outermost one resets the claim when it has been visited.

use std::cell::{Cell, RefCell};

/// What a drop target tells the person while a drag is over it: the floating copy shows it.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DropHint {
    /// Said beside the copy: "Make a subtask of Pack suitcase".
    pub label: String,
    /// How big the copy becomes over this target (1.0 is as picked up). A copy that is about to
    /// become part of what it is over shrinks.
    pub scale: f32,
}

impl DropHint {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            scale: 1.0,
        }
    }

    /// The copy shrinks: it is going inside the target, not beside it.
    pub fn shrink(mut self) -> Self {
        self.scale = SHRUNK;
        self
    }
}

/// The scale of a copy that is going inside its target.
pub const SHRUNK: f32 = 0.88;

/// Where on a target a drop lands: before it, inside it or after it. The same type [`Tree`](crate::widgets::Tree)
/// uses for its drops, so a list of rows and a tree divide a row the same way: the thin band at
/// the top is *before* it, the one at the bottom *after* it, and the middle is *inside* it (a task
/// dropped on a task is its subtask), as in GTK's list drop indicators and libadwaita's tab bar.
pub use crate::primitives::tree::Position as Place;

/// How a target is divided into [`Place`]s.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Targets {
    /// One place, [`Place::Inside`]: the whole target.
    #[default]
    Whole,
    /// A list that is only reordered: the upper half is [`Place::Before`], the lower
    /// [`Place::After`]. There is no inside.
    Between,
    /// Edges first, then a middle, as [`tree::drop_position`](crate::primitives::tree::drop_position)
    /// does: the top quarter of the height is before, the bottom quarter after, and only the
    /// middle half is inside, so a drop is an insertion unless the pointer is well within the row
    /// (a band is never thinner than [`MIN_EDGE`] pixels, so a short row keeps a usable edge).
    /// For a list whose rows can also take what is dropped on them.
    Edges,
}

/// The least thickness, in pixels, of the before / after band of [`Targets::Edges`].
pub const MIN_EDGE: f32 = 8.0;

impl Targets {
    /// The place at height `y` of a target `height` tall (both in the same unit, `y` from its top).
    pub fn place_at(self, y: f32, height: f32) -> Place {
        match self {
            Targets::Whole => Place::Inside,
            Targets::Between => {
                if y < height / 2.0 {
                    Place::Before
                } else {
                    Place::After
                }
            }
            Targets::Edges => {
                // Never more than a third, so a short row keeps a middle.
                let band = (height * 0.25).max(MIN_EDGE).min(height / 3.0);
                if y < band {
                    Place::Before
                } else if y > height - band {
                    Place::After
                } else {
                    Place::Inside
                }
            }
        }
    }
}

/// How much of a row the carried copy covers, from 0 to 1, for it to count as *on* the row
/// ([`Place::Inside`]) rather than between it and a neighbour.
pub const INSIDE_OVERLAP: f32 = 0.75;

impl Targets {
    /// The place of a carried copy over a target, by where the *copy* is, not the pointer: the
    /// target `zone` and the copy `ghost` are each a `(top, height)`. `None` when the copy's
    /// middle is not over the target. Edges come first: the copy is [`Place::Inside`] only when
    /// it covers [`INSIDE_OVERLAP`] of the smaller of the two; otherwise it is before the target
    /// when its middle is in the upper half, after it when in the lower.
    pub fn place_of_ghost(self, zone: (f32, f32), ghost: (f32, f32)) -> Option<Place> {
        let (top, height) = zone;
        let middle = ghost.0 + ghost.1 / 2.0;
        if middle < top || middle >= top + height {
            return None;
        }
        Some(match self {
            Targets::Whole => Place::Inside,
            Targets::Between => self.place_at(middle - top, height),
            Targets::Edges => {
                let overlap = (top + height).min(ghost.0 + ghost.1) - top.max(ghost.0);
                let smaller = height.min(ghost.1).max(f32::EPSILON);
                if overlap / smaller >= INSIDE_OVERLAP {
                    Place::Inside
                } else if middle - top < height / 2.0 {
                    Place::Before
                } else {
                    Place::After
                }
            }
        })
    }
}

thread_local! {
    /// The carried copy's `(top, height)`, kept by its source while it is carried.
    static GHOST: Cell<Option<(f32, f32)>> = const { Cell::new(None) };
    static DEPTH: Cell<u32> = const { Cell::new(0) };
    static DROP_TAKEN: Cell<bool> = const { Cell::new(false) };
    /// `Some(None)` is a target that took the hover and has nothing to say.
    static HOVER: RefCell<Option<Option<DropHint>>> = const { RefCell::new(None) };
}

/// The source of a drag says where the copy is now (`None` when it is put down).
pub fn set_ghost(ghost: Option<(f32, f32)>) {
    GHOST.with(|g| g.set(ghost));
}

/// Where the carried copy is, vertically, as `(top, height)`, while one is carried.
pub fn ghost() -> Option<(f32, f32)> {
    GHOST.with(Cell::get)
}

/// A target starts handling an event: call before its content is visited.
pub fn enter() {
    DEPTH.with(|d| d.set(d.get() + 1));
}

/// A target is done with the event, after its own check. The outermost one clears the claim.
pub fn exit() {
    DEPTH.with(|d| {
        let n = d.get().saturating_sub(1);
        d.set(n);
        if n == 0 {
            DROP_TAKEN.with(|t| t.set(false));
        }
    });
}

/// Whether this target takes the drop: true for the first to ask in one event.
pub fn take_drop() -> bool {
    DROP_TAKEN.with(|t| !t.replace(true))
}

/// Whether this target is the one the pointer is hovering as far as the drawing goes: true for
/// the first to ask in one frame (the innermost). `hint` is shown by the copy.
pub fn take_hover(hint: Option<&DropHint>) -> bool {
    HOVER.with(|h| {
        let mut h = h.borrow_mut();
        if h.is_some() {
            return false;
        }
        *h = Some(hint.cloned());
        true
    })
}

/// What the frame's hovered target said, once: read by the floating copy after everything else
/// is drawn. `None` when no target was hovered.
pub fn hovered() -> Option<Option<DropHint>> {
    HOVER.with(|h| h.borrow_mut().take())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_target_is_divided_into_places() {
        assert_eq!(Targets::Whole.place_at(0.0, 40.0), Place::Inside);
        assert_eq!(Targets::Between.place_at(5.0, 40.0), Place::Before);
        assert_eq!(Targets::Between.place_at(25.0, 40.0), Place::After);
        // 40 tall: a band of 10, the middle between.
        assert_eq!(Targets::Edges.place_at(3.0, 40.0), Place::Before);
        assert_eq!(Targets::Edges.place_at(20.0, 40.0), Place::Inside);
        assert_eq!(Targets::Edges.place_at(35.0, 40.0), Place::After);
        // A tall row divides as a tree does: quarters at the edges, the middle half inside.
        for h in [60.0_f32, 100.0, 200.0] {
            for k in 0..=20 {
                let frac = k as f32 / 20.0;
                assert_eq!(
                    Targets::Edges.place_at(frac * h, h),
                    crate::primitives::tree::drop_position(frac, true),
                    "{h} at {frac}"
                );
            }
        }
        // Short rows keep a middle too.
        assert_eq!(Targets::Edges.place_at(9.0, 18.0), Place::Inside);
    }

    #[test]
    fn the_copy_is_on_a_row_only_when_it_covers_most_of_it() {
        let row = (100.0, 40.0);
        // Same height, almost exactly over it: inside.
        assert_eq!(Targets::Edges.place_of_ghost(row, (102.0, 40.0)), Some(Place::Inside));
        // Half a row up: its middle is in the upper half, so before.
        assert_eq!(Targets::Edges.place_of_ghost(row, (84.0, 40.0)), Some(Place::Before));
        // Half a row down: after.
        assert_eq!(Targets::Edges.place_of_ghost(row, (118.0, 40.0)), Some(Place::After));
        // Its middle is not over the row at all.
        assert_eq!(Targets::Edges.place_of_ghost(row, (20.0, 40.0)), None);
        // A reorder-only list has no inside.
        assert_eq!(Targets::Between.place_of_ghost(row, (102.0, 40.0)), Some(Place::After));
        assert_eq!(Targets::Between.place_of_ghost(row, (90.0, 40.0)), Some(Place::Before));
    }

    #[test]
    fn the_innermost_target_takes_the_drop() {
        enter(); // outer
        enter(); // inner
        assert!(take_drop(), "inner asks first");
        exit();
        assert!(!take_drop(), "outer is refused");
        exit();
        // The next event starts clean.
        enter();
        assert!(take_drop());
        exit();
    }

    #[test]
    fn the_innermost_hover_is_the_one_shown() {
        assert!(hovered().is_none());
        assert!(take_hover(Some(&DropHint::new("inner").shrink())));
        assert!(!take_hover(Some(&DropHint::new("outer"))));
        let said = hovered().unwrap().unwrap();
        assert_eq!((said.label.as_str(), said.scale), ("inner", SHRUNK));
        // Read once.
        assert!(hovered().is_none());
    }

    #[test]
    fn a_target_with_nothing_to_say_still_takes_the_hover() {
        assert!(take_hover(None));
        assert!(!take_hover(Some(&DropHint::new("outer"))));
        assert_eq!(hovered(), Some(None));
    }
}
