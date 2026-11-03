//! Headless virtualization: which rows of a long, scrolled list to build.
//!
//! Rows have a fixed height (a *pitch*: the row plus the hairline after it), so which rows show
//! is arithmetic on the scroll offset. The widget builds those rows and fills the rest with two
//! spacers, [`Spacers`], so the scrollbar and the scroll offset stay true to the full length.
//!
//! No established virtualizing widget exists for iced (0.14 has `scrollable` with `on_scroll`,
//! `sensor` and `lazy`, but no virtual list), and libcosmic has none either (its
//! `segmented_button` model is a flat list with a per-item indent). So this is the shared piece:
//! [`Tree`](crate::widget::Tree) and [`ResponsiveTable`](crate::widget::ResponsiveTable) both
//! use it, and so can any list that scrolls. Variable row heights need a measuring engine, such
//! as the headless `virtualizer` crate; this one does not do them.
//!
//! The app keeps a [`Viewport`], updates it from the scrollable's `on_scroll`, and asks it for
//! the [`window`](Viewport::window):
//!
//! ```
//! use iced_adaptive_design::primitives::virtualize::Viewport;
//!
//! let mut vp = Viewport::default();
//! vp.set(400.0, 100.0);                       // scrolled 400 px, 100 px tall
//! assert_eq!(vp.window(1000, 20.0, 2), 18..27);
//! ```

use std::ops::Range;

/// How far a list is scrolled and how tall its viewport is.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Viewport {
    /// Pixels scrolled from the top.
    pub offset: f32,
    /// The viewport's height.
    pub height: f32,
}

impl Viewport {
    /// A viewport of `height` at the top (before any scroll event; also the height assumed
    /// until the first one arrives).
    pub fn new(height: f32) -> Self {
        Self {
            offset: 0.0,
            height,
        }
    }
    /// Takes the scrollable's `on_scroll` values.
    pub fn set(&mut self, offset: f32, height: f32) {
        self.offset = offset.max(0.0);
        self.height = height.max(0.0);
    }
    /// The rows to build for `len` rows of `pitch` pixels: those in view plus `overscan` more
    /// on each side.
    pub fn window(&self, len: usize, pitch: f32, overscan: usize) -> Range<usize> {
        window(len, self.offset, self.height, pitch, overscan)
    }
    /// The first row that shows at least partly.
    pub fn first_visible(&self, pitch: f32) -> usize {
        if pitch <= 0.0 {
            0
        } else {
            (self.offset / pitch).floor() as usize
        }
    }
    /// The offset that puts row `index` at the top (to scroll to it).
    pub fn offset_of(index: usize, pitch: f32) -> f32 {
        index as f32 * pitch
    }
}

/// Which rows to build for `len` fixed-height rows in a viewport of `viewport` pixels scrolled by
/// `scroll`, plus `overscan` more on each side.
pub fn window(len: usize, scroll: f32, viewport: f32, pitch: f32, overscan: usize) -> Range<usize> {
    if len == 0 || pitch <= 0.0 {
        return 0..0;
    }
    let first = (scroll.max(0.0) / pitch).floor() as usize;
    let last = ((scroll.max(0.0) + viewport.max(0.0)) / pitch).ceil() as usize;
    first.saturating_sub(overscan).min(len)..(last + overscan).min(len)
}

/// The two fillers around the rows that are built.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spacers {
    pub above: f32,
    pub below: f32,
}

/// The space to leave above and below the built `range` of `len` rows of `pitch` pixels, so the
/// content is as long as the whole list.
pub fn spacers(len: usize, range: &Range<usize>, pitch: f32) -> Spacers {
    Spacers {
        above: range.start as f32 * pitch,
        below: len.saturating_sub(range.end) as f32 * pitch,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_window_covers_the_viewport_plus_overscan() {
        assert_eq!(window(1000, 0.0, 100.0, 20.0, 2), 0..7);
        assert_eq!(window(1000, 400.0, 100.0, 20.0, 2), 18..27);
        assert_eq!(
            window(10, 1000.0, 100.0, 20.0, 2),
            10..10,
            "scrolled past the end"
        );
        assert_eq!(window(0, 0.0, 100.0, 20.0, 2), 0..0);
        assert_eq!(window(10, 0.0, 100.0, 0.0, 2), 0..0, "no pitch, no rows");
    }

    #[test]
    fn the_window_never_leaves_the_list() {
        let w = window(5, 0.0, 10_000.0, 20.0, 50);
        assert_eq!(w, 0..5);
    }

    #[test]
    fn a_viewport_takes_scroll_values_and_clamps_them() {
        let mut v = Viewport::new(300.0);
        assert_eq!(v.window(100, 30.0, 0), 0..10);
        v.set(-20.0, 300.0);
        assert_eq!(v.offset, 0.0);
        v.set(600.0, 300.0);
        assert_eq!(v.window(100, 30.0, 0), 20..30);
        assert_eq!(v.first_visible(30.0), 20);
        assert_eq!(
            Viewport::offset_of(20, 30.0),
            600.0,
            "scroll_to is the inverse"
        );
    }

    #[test]
    fn spacers_keep_the_content_as_long_as_the_list() {
        let r = 18..27;
        let s = spacers(1000, &r, 20.0);
        assert_eq!(
            s,
            Spacers {
                above: 360.0,
                below: 973.0 * 20.0
            }
        );
        let built = (r.end - r.start) as f32 * 20.0;
        assert_eq!(s.above + built + s.below, 1000.0 * 20.0);
    }
}
