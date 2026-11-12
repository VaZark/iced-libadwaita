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

thread_local! {
    static DEPTH: Cell<u32> = const { Cell::new(0) };
    static DROP_TAKEN: Cell<bool> = const { Cell::new(false) };
    /// `Some(None)` is a target that took the hover and has nothing to say.
    static HOVER: RefCell<Option<Option<DropHint>>> = const { RefCell::new(None) };
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
