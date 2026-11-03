//! Headless logic of a reorderable list: the preview / edit [`Mode`], the draft that edit mode
//! works on, and what a row drag means.
//!
//! The shape is Chakra UI's [Editable](https://chakra-ui.com/docs/components/editable): a value
//! with a **preview** view and an **edit** view. Here the value is the order of the items.
//! Entering edit mode starts a *draft* of the order; reordering changes the draft; **submit**
//! commits it and **cancel** throws it away.
//!
//! * [`ReorderState`]: the committed items and the draft (`edit`, `reorder`, `submit`, `cancel`);
//! * [`Mode`]: the key a [`Trigger::selected`](super::meta::Trigger::selected) picks the view by;
//! * [`target`] / [`shift`]: where a dragged row would land, and how far the other rows move out
//!   of its way while it is dragged. Pure functions over row spans, so the drag math is tested
//!   without a window.

/// Which view of the list shows: Chakra's `EditablePreview` or its input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Mode {
    /// Plain rows; nothing can be dragged.
    #[default]
    Preview,
    /// Rows with drag handles.
    Edit,
}

impl Mode {
    pub const fn from_editing(editing: bool) -> Self {
        if editing { Self::Edit } else { Self::Preview }
    }
}

/// What happens to a reorderable list: the single message a list emits, applied to a
/// [`ReorderState`] with [`apply`](ReorderState::apply).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    /// Edit was pressed: start a draft.
    Edit,
    /// A row was dropped: it was at `from` and now belongs at `to`.
    Reorder { from: usize, to: usize },
    /// Done was pressed: keep the draft.
    Submit,
    /// Cancel was pressed: throw the draft away.
    Cancel,
}

/// Moves the item at `from` so that it ends up at index `to`, shifting those in between.
/// Returns whether anything moved; out-of-range indices change nothing.
pub fn move_item<T>(items: &mut Vec<T>, from: usize, to: usize) -> bool {
    if from == to || from >= items.len() || to >= items.len() {
        return false;
    }
    let item = items.remove(from);
    items.insert(to, item);
    true
}

/// The committed order and, while editing, a draft of it.
///
/// ```
/// use iced_libadwaita::primitives::reorder::ReorderState;
///
/// let mut s = ReorderState::new(vec!["a", "b", "c"]);
/// s.edit();
/// s.reorder(0, 2);
/// assert_eq!(s.items(), ["b", "c", "a"]);   // the draft
/// assert_eq!(s.committed(), ["a", "b", "c"]);
/// s.cancel();
/// assert_eq!(s.items(), ["a", "b", "c"]);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ReorderState<T> {
    committed: Vec<T>,
    draft: Option<Vec<T>>,
}

impl<T: Clone> ReorderState<T> {
    pub fn new(items: Vec<T>) -> Self {
        Self {
            committed: items,
            draft: None,
        }
    }
    pub fn mode(&self) -> Mode {
        Mode::from_editing(self.draft.is_some())
    }
    pub fn is_editing(&self) -> bool {
        self.draft.is_some()
    }
    /// What the list shows: the draft while editing, else the committed order.
    pub fn items(&self) -> &[T] {
        self.draft.as_deref().unwrap_or(&self.committed)
    }
    /// The last committed order.
    pub fn committed(&self) -> &[T] {
        &self.committed
    }
    /// Whether the draft differs from the committed order (to enable a Done button).
    pub fn is_dirty(&self) -> bool
    where
        T: PartialEq,
    {
        self.draft.as_ref().is_some_and(|d| *d != self.committed)
    }
    /// Starts a draft. Does nothing when already editing.
    pub fn edit(&mut self) {
        self.draft.get_or_insert_with(|| self.committed.clone());
    }
    /// Moves a row in the draft (`to` is the index it ends up at). Ignored outside edit mode.
    /// Returns whether the order changed.
    pub fn reorder(&mut self, from: usize, to: usize) -> bool {
        self.draft.as_mut().is_some_and(|d| move_item(d, from, to))
    }
    /// Commits the draft and returns to preview. Returns whether the order changed.
    pub fn submit(&mut self) -> bool
    where
        T: PartialEq,
    {
        match self.draft.take() {
            Some(d) if d != self.committed => {
                self.committed = d;
                true
            }
            _ => false,
        }
    }
    /// Throws the draft away and returns to preview.
    pub fn cancel(&mut self) {
        self.draft = None;
    }
    /// Applies a list's [`Event`]: the whole `update` of a reorderable list. Returns whether the
    /// *committed* order changed (only `Submit` can).
    pub fn apply(&mut self, event: Event) -> bool
    where
        T: PartialEq,
    {
        match event {
            Event::Edit => self.edit(),
            Event::Reorder { from, to } => {
                self.reorder(from, to);
            }
            Event::Submit => return self.submit(),
            Event::Cancel => self.cancel(),
        }
        false
    }
    /// Replaces the committed items (a row added or removed elsewhere). Any draft is dropped.
    pub fn set(&mut self, items: Vec<T>) {
        self.committed = items;
        self.draft = None;
    }
}

/// A row's vertical extent: where it starts and how tall it is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub top: f32,
    pub height: f32,
}

impl Span {
    pub fn new(top: f32, height: f32) -> Self {
        Self { top, height }
    }
    fn mid(self) -> f32 {
        self.top + self.height / 2.0
    }
}

/// The index the dragged row `from` would take if dropped with its center at `center`: the
/// number of *other* rows whose middle is above it. Always in `0..rows.len()`.
pub fn target(rows: &[Span], from: usize, center: f32) -> usize {
    rows.iter()
        .enumerate()
        .filter(|(j, r)| *j != from && r.mid() < center)
        .count()
}

/// How far (in y) row `j` moves while row `from` is dragged over the slot `to`, where `step` is
/// the dragged row's height plus the gap between rows. Rows between the two slots shift by one
/// step toward the slot the dragged row left.
pub fn shift(j: usize, from: usize, to: usize, step: f32) -> f32 {
    if from < to && j > from && j <= to {
        -step
    } else if to < from && j >= to && j < from {
        step
    } else {
        0.0
    }
}

/// Where the top of the slot `to` is once row `from` is dropped there: the rows keep their
/// order and the gap between them (`gap`), the dragged one slotted in at index `to`.
pub fn slot_top(rows: &[Span], from: usize, to: usize, gap: f32) -> f32 {
    let Some(first) = rows.first() else {
        return 0.0;
    };
    first.top
        + rows
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != from)
            .take(to)
            .map(|(_, r)| r.height + gap)
            .sum::<f32>()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(heights: &[f32]) -> Vec<Span> {
        let mut top = 0.0;
        heights
            .iter()
            .map(|h| {
                let s = Span::new(top, *h);
                top += h + 1.0;
                s
            })
            .collect()
    }

    #[test]
    fn move_item_puts_the_row_at_the_target_index() {
        let mut v = vec!['a', 'b', 'c', 'd'];
        assert!(move_item(&mut v, 0, 2));
        assert_eq!(v, ['b', 'c', 'a', 'd']);
        assert!(move_item(&mut v, 3, 0));
        assert_eq!(v, ['d', 'b', 'c', 'a']);
    }

    #[test]
    fn move_item_ignores_noops_and_bad_indices() {
        let mut v = vec![1, 2, 3];
        assert!(!move_item(&mut v, 1, 1));
        assert!(!move_item(&mut v, 3, 0));
        assert!(!move_item(&mut v, 0, 3));
        assert_eq!(v, [1, 2, 3]);
    }

    #[test]
    fn editing_works_on_a_draft_until_submit() {
        let mut s = ReorderState::new(vec![1, 2, 3]);
        assert_eq!(s.mode(), Mode::Preview);
        assert!(!s.reorder(0, 1), "no reordering outside edit mode");
        s.edit();
        assert_eq!(s.mode(), Mode::Edit);
        assert!(!s.is_dirty());
        assert!(s.reorder(2, 0));
        assert!(s.is_dirty());
        assert_eq!(s.items(), [3, 1, 2]);
        assert_eq!(s.committed(), [1, 2, 3]);
        assert!(s.submit());
        assert_eq!((s.items(), s.mode()), (&[3, 1, 2][..], Mode::Preview));
    }

    #[test]
    fn apply_runs_the_whole_edit_cycle() {
        let mut s = ReorderState::new(vec![1, 2, 3]);
        assert!(!s.apply(Event::Edit));
        s.apply(Event::Reorder { from: 0, to: 2 });
        assert_eq!(s.items(), [2, 3, 1]);
        assert!(s.apply(Event::Submit));
        assert_eq!(s.committed(), [2, 3, 1]);
        s.apply(Event::Edit);
        s.apply(Event::Reorder { from: 0, to: 1 });
        assert!(!s.apply(Event::Cancel));
        assert_eq!((s.items(), s.mode()), (&[2, 3, 1][..], Mode::Preview));
    }

    #[test]
    fn cancel_restores_the_committed_order() {
        let mut s = ReorderState::new(vec![1, 2, 3]);
        s.edit();
        s.reorder(0, 2);
        s.cancel();
        assert_eq!(s.items(), [1, 2, 3]);
        assert!(!s.is_editing());
    }

    #[test]
    fn submitting_an_unchanged_draft_reports_no_change() {
        let mut s = ReorderState::new(vec![1, 2]);
        s.edit();
        s.reorder(0, 1);
        s.reorder(1, 0);
        assert!(!s.submit());
        assert_eq!(s.items(), [1, 2]);
    }

    #[test]
    fn editing_twice_keeps_the_draft() {
        let mut s = ReorderState::new(vec![1, 2, 3]);
        s.edit();
        s.reorder(0, 1);
        s.edit();
        assert_eq!(s.items(), [2, 1, 3]);
    }

    #[test]
    fn the_target_counts_the_rows_above_the_dragged_center() {
        // Three rows of 40: tops 0, 41, 82; middles 20, 61, 102.
        let r = rows(&[40.0, 40.0, 40.0]);
        assert_eq!(target(&r, 0, 20.0), 0);
        assert_eq!(target(&r, 0, 70.0), 1);
        assert_eq!(target(&r, 0, 500.0), 2);
        assert_eq!(target(&r, 2, -50.0), 0);
        assert_eq!(target(&r, 2, 62.0), 2);
        assert_eq!(target(&r, 1, 61.0), 1, "dropping where it is stays put");
    }

    #[test]
    fn unequal_heights_use_each_rows_middle() {
        let r = rows(&[100.0, 20.0, 20.0]); // middles 50, 111, 132
        assert_eq!(target(&r, 2, 60.0), 1);
        assert_eq!(target(&r, 2, 40.0), 0);
    }

    #[test]
    fn the_target_slot_sits_where_the_dragged_row_would_land() {
        let r = rows(&[40.0, 20.0, 60.0]); // tops 0, 41, 62
        assert_eq!(slot_top(&r, 0, 0, 1.0), 0.0);
        assert_eq!(slot_top(&r, 0, 2, 1.0), 20.0 + 1.0 + 60.0 + 1.0);
        assert_eq!(slot_top(&r, 2, 0, 1.0), 0.0);
        assert_eq!(slot_top(&r, 2, 1, 1.0), 41.0);
        assert_eq!(slot_top(&r, 1, 1, 1.0), 41.0, "unmoved: its own place");
    }

    #[test]
    fn rows_between_the_slots_make_room() {
        // Dragging row 1 down to slot 3: rows 2 and 3 move up a step.
        assert_eq!(shift(0, 1, 3, 41.0), 0.0);
        assert_eq!(shift(2, 1, 3, 41.0), -41.0);
        assert_eq!(shift(3, 1, 3, 41.0), -41.0);
        assert_eq!(shift(4, 1, 3, 41.0), 0.0);
        // Dragging row 3 up to slot 1: rows 1 and 2 move down.
        assert_eq!(shift(1, 3, 1, 41.0), 41.0);
        assert_eq!(shift(2, 3, 1, 41.0), 41.0);
        assert_eq!(shift(3, 3, 1, 41.0), 0.0);
        assert_eq!(shift(0, 3, 1, 41.0), 0.0);
        // Not moved: nothing shifts.
        assert_eq!(shift(2, 2, 2, 41.0), 0.0);
    }
}
