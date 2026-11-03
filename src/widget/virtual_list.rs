//! `VirtualList`: a scrolling list of **fixed-height rows** that builds only the rows in view.
//! The flat list: no separators and no surface of its own. [`ResponsiveTable`](super::ResponsiveTable)
//! and [`Tree`](super::Tree) virtualize their own rows; this is the plain list.
//!
//! The application keeps a [`Viewport`] fed by [`on_scroll`](VirtualList::on_scroll) and passes it
//! back; the list asks it for the [window](Viewport::window) of rows to build and fills the rest
//! with spacers, so the scroll bar stays true to the whole length.
//!
//! ```ignore
//! VirtualList::new(self.tracks.len(), 48.0, &self.viewport, |i| track_row(&self.tracks[i]))
//!     .on_scroll(Msg::Scroll)       // Msg::Scroll(vp) => self.viewport = vp
//!     .overscan(4)
//! ```
//!
//! Rows have one height, so which rows show is arithmetic ([`primitives::virtualize`](crate::primitives::virtualize)).

use super::{Element, scrolled_window};
use crate::primitives::virtualize::{Viewport, spacers};
use iced::Length;
use iced::widget::{Id, column, container, space};

/// See the [module](self).
pub struct VirtualList<'a, M> {
    total: usize,
    row_height: f32,
    viewport: Viewport,
    build: Box<dyn Fn(usize) -> Element<'a, M> + 'a>,
    on_scroll: Option<Box<dyn Fn(Viewport) -> M + 'a>>,
    overscan: usize,
    id: Option<Id>,
    width: Length,
    height: Length,
}

impl<'a, M: 'a> VirtualList<'a, M> {
    /// `total` rows of `row_height` px. `viewport` is how far the list is scrolled (from
    /// [`on_scroll`](Self::on_scroll)); `build` makes row `i`, and is called only for the rows
    /// in or near view.
    pub fn new(
        total: usize,
        row_height: f32,
        viewport: &Viewport,
        build: impl Fn(usize) -> Element<'a, M> + 'a,
    ) -> Self {
        Self {
            total,
            row_height: row_height.max(1.0),
            viewport: *viewport,
            build: Box::new(build),
            on_scroll: None,
            overscan: 4,
            id: None,
            width: Length::Fill,
            height: Length::Fill,
        }
    }
    /// The list was scrolled or resized: the viewport to keep and pass back.
    pub fn on_scroll(mut self, f: impl Fn(Viewport) -> M + 'a) -> Self {
        self.on_scroll = Some(Box::new(f));
        self
    }
    /// Rows built beyond each edge of the view, so a quick scroll shows no gap (default 4).
    pub fn overscan(mut self, rows: usize) -> Self {
        self.overscan = rows;
        self
    }
    /// Names the scrollable, for `scroll_to` and `snap_to` operations.
    pub fn id(mut self, id: impl Into<Id>) -> Self {
        self.id = Some(id.into());
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    pub fn height(mut self, h: impl Into<Length>) -> Self {
        self.height = h.into();
        self
    }

    /// The rows that are built.
    fn range(&self) -> std::ops::Range<usize> {
        self.viewport
            .window(self.total, self.row_height, self.overscan)
    }
}

impl<'a, M: 'a> From<VirtualList<'a, M>> for Element<'a, M> {
    fn from(l: VirtualList<'a, M>) -> Self {
        let range = l.range();
        let fill = spacers(l.total, &range, l.row_height);
        let rows = range.map(|i| {
            container((l.build)(i))
                .width(Length::Fill)
                .height(Length::Fixed(l.row_height))
                .into()
        });
        let body = column![
            space().height(fill.above),
            column(rows),
            space().height(fill.below)
        ];
        let mut s = scrolled_window(body).width(l.width).height(l.height);
        if let Some(id) = l.id {
            s = s.id(id);
        }
        if let Some(f) = l.on_scroll {
            s = s.on_scroll(move |v| {
                f(Viewport {
                    offset: v.absolute_offset().y,
                    height: v.bounds().height,
                })
            });
        }
        s.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(total: usize, offset: f32) -> VirtualList<'static, ()> {
        let mut vp = Viewport::default();
        vp.set(offset, 100.0);
        VirtualList::new(total, 20.0, &vp, |_| space().into()).overscan(2)
    }

    #[test]
    fn only_the_rows_in_view_are_built() {
        assert_eq!(list(1000, 400.0).range(), 18..27);
        assert_eq!(list(1000, 0.0).range(), 0..7);
    }

    #[test]
    fn a_short_list_is_built_whole() {
        assert_eq!(list(3, 0.0).range(), 0..3);
        assert!(list(0, 0.0).range().is_empty());
    }
}
