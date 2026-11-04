//! `SortHeader`: the header of a flat list ([`VirtualList`](crate::layouts::VirtualList) of
//! [`ItemRow`](crate::widgets::ItemRow)s): labels over the columns, each with an arrow for its sort
//! direction, driven by a [`SortState`]. [`ResponsiveTable`](crate::widgets::ResponsiveTable) has one
//! built in; a flat list does not.
//!
//! The columns are given in the same order and widths as the row's, so the labels sit over their
//! cells. A label that does not sort is plain; a space keeps a gap for a picture or a menu.
//!
//! ```ignore
//! SortHeader::new(&self.sort, Msg::Sort)
//!     .space(Length::Fixed(40.0))                          // over the cover
//!     .column(Col::Title, "Title")                              // every column grows by default
//!     .column(Col::Album, "Album").width(Length::Fixed(160.0))  // or is given a width
//!     .column(Col::Time, "Time").width(Length::Fixed(44.0))
//!     .space(Length::Fixed(36.0))                          // over the menu
//! ```

use crate::Element;
use crate::components::typography as ty;
use crate::layouts::{Flex, FlexCells};
use crate::primitives::sorting::{Direction, SortState};
use crate::styles::text as st;
use crate::{Icon, Metrics};
use iced::alignment::Horizontal;
use iced::widget::{button, container, row, space};
use iced::{Alignment, Length, Padding};

/// The least width a header label needs: its text and the sort arrow. A column is never made
/// narrower than this, so a header is never cut off ("Tim"): the columns that can give way give
/// way, in proportion, and a small column keeps what its label needs.
pub(crate) fn label_floor(label: &str) -> f32 {
    // A caption heading is about 7 px a character; the arrow and its gap are 16 px more.
    label.chars().count() as f32 * 7.0 + 16.0
}

/// How a column of `width` takes part in the line, for the header and the rows alike so they
/// line up. Every column is **flex**: a fixed width is a *starting* size that gives way down to
/// the column's label (never cut off), `Fill` and `FillPortion(n)` take `n` shares of the free
/// width and give way down to their label too.
pub(crate) fn flex_of(width: Length, label: &str) -> Flex {
    let floor = label_floor(label);
    match width {
        Length::Fixed(px) => Flex::shrink(px.max(floor), floor),
        Length::Fill => Flex::grow(1.0, floor),
        Length::FillPortion(n) => Flex::grow(f32::from(n), floor),
        Length::Shrink => Flex::shrink(floor, floor),
    }
}

enum Cell<K> {
    Space(Length),
    Plain(String, Length, Horizontal),
    Sortable(K, String, Length, Horizontal),
}

/// See the [module](self).
pub struct SortHeader<'a, K, M> {
    state: &'a SortState<K>,
    on_sort: Box<dyn Fn(K) -> M + 'a>,
    cells: Vec<Cell<K>>,
    padding: Padding,
    metrics: Metrics,
}

impl<'a, K: PartialEq + Clone + 'a, M: Clone + 'a> SortHeader<'a, K, M> {
    /// `state` says which column is sorted and how; `on_sort` gets the key of a pressed label
    /// (the application calls [`SortState::activate`] with it).
    pub fn new(state: &'a SortState<K>, on_sort: impl Fn(K) -> M + 'a) -> Self {
        Self {
            state,
            on_sort: Box::new(on_sort),
            cells: Vec::new(),
            padding: Padding {
                top: 4.0,
                bottom: 4.0,
                left: 8.0,
                right: 12.0,
            },
            metrics: Metrics::desktop(),
        }
    }
    /// A sortable column: its `key` and label. Like every column it **grows** to cover as much
    /// width as the others leave (`flex-grow: 1`); [`width`](Self::width) fixes it instead.
    pub fn column(mut self, key: K, label: impl Into<String>) -> Self {
        self.cells.push(Cell::Sortable(
            key,
            label.into(),
            Length::Fill,
            Horizontal::Left,
        ));
        self
    }
    /// A label that does not sort. It grows like any column.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.cells
            .push(Cell::Plain(label.into(), Length::Fill, Horizontal::Left));
        self
    }
    /// The width of the cell just added, in place of growing: a fixed width (a time, a count), or
    /// a `FillPortion` to share the free width.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        let w = width.into();
        match self.cells.last_mut() {
            Some(Cell::Space(x) | Cell::Plain(_, x, _) | Cell::Sortable(_, _, x, _)) => *x = w,
            None => {}
        }
        self
    }
    /// Aligns the label of the column or label just added: `Right` over right-aligned cells (a
    /// time, a count).
    pub fn align(mut self, align: Horizontal) -> Self {
        if let Some(Cell::Plain(_, _, a) | Cell::Sortable(_, _, _, a)) = self.cells.last_mut() {
            *a = align;
        }
        self
    }
    /// An empty cell, over a picture or a menu.
    pub fn space(mut self, width: impl Into<Length>) -> Self {
        self.cells.push(Cell::Space(width.into()));
        self
    }
    /// How many cells the header has.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.cells.len()
    }
    /// The padding around the header; match the rows'.
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

/// The arrow of a column: up for ascending, down for descending, both when it can sort but is not.
pub fn arrow(direction: Option<Direction>) -> Icon {
    match direction {
        Some(Direction::Asc) => Icon::GoUp,
        Some(Direction::Desc) => Icon::GoDown,
        None => Icon::GoUpDown,
    }
}

impl<'a, K: PartialEq + Clone + 'a, M: Clone + 'a> From<SortHeader<'a, K, M>> for Element<'a, M> {
    fn from(h: SortHeader<'a, K, M>) -> Self {
        let m = h.metrics;
        let mut line = FlexCells::new().spacing(m.space_m);
        for cell in h.cells {
            line = match cell {
                // A space is a picture's or a menu's room: it keeps its width.
                Cell::Space(w) => {
                    let px = if let Length::Fixed(px) = w { px } else { 0.0 };
                    line.cell(Flex::fixed(px), space())
                }
                Cell::Plain(label, w, align) => {
                    let flex = flex_of(w, &label);
                    line.cell(
                        flex,
                        container(
                            ty::caption(label)
                                .style(st::dim)
                                .wrapping(iced::widget::text::Wrapping::None),
                        )
                        .width(Length::Fill)
                        .align_x(align),
                    )
                }
                Cell::Sortable(key, label, w, align) => {
                    let flex = flex_of(w, &label);
                    let active = h.state.direction(&key).is_some();
                    let icon = arrow(h.state.direction(&key));
                    let label = if active {
                        ty::caption(label).wrapping(iced::widget::text::Wrapping::None)
                    } else {
                        ty::caption(label)
                            .style(st::dim)
                            .wrapping(iced::widget::text::Wrapping::None)
                    };
                    let msg = (h.on_sort)(key);
                    // The label is the click target over **the whole column**, not just its text,
                    // and it does not highlight: a header is not a button to look at.
                    line.cell(
                        flex,
                        button(
                            container(
                                row![label, icon.view(12.0)]
                                    .spacing(m.space_xs)
                                    .align_y(Alignment::Center),
                            )
                            .width(Length::Fill)
                            .align_x(align),
                        )
                        .width(Length::Fill)
                        .padding(0)
                        .style(|t: &crate::Theme, _| button::Style {
                            background: None,
                            text_color: t.semantic.window.fg,
                            border: iced::Border::default(),
                            shadow: iced::Shadow::default(),
                            snap: false,
                        })
                        .on_press(msg),
                    )
                }
            };
        }
        container(line).padding(h.padding).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_column_is_flex_and_keeps_what_its_label_needs() {
        // A 44 px "Time" column starts at 44 and gives way no further than its label.
        let f = flex_of(Length::Fixed(44.0), "Time").0;
        assert_eq!(f.min, label_floor("Time"));
        assert!(f.basis >= 44.0 && f.shrink > 0.0 && f.grow == 0.0);
        // A growing column takes free space, down to its label.
        let g = flex_of(Length::Fill, "Title").0;
        assert!(g.grow > 0.0 && g.min == label_floor("Title"));
        assert_eq!(flex_of(Length::FillPortion(3), "x").0.grow, 3.0);
        assert!(label_floor("Duration") > label_floor("Time"));
    }

    #[test]
    fn the_arrow_follows_the_direction() {
        assert_eq!(arrow(Some(Direction::Asc)), Icon::GoUp);
        assert_eq!(arrow(Some(Direction::Desc)), Icon::GoDown);
        assert_eq!(arrow(None), Icon::GoUpDown);
    }

    #[test]
    fn columns_keep_their_order() {
        let s = SortState::<u8>::new();
        let h: SortHeader<'_, u8, u8> = SortHeader::new(&s, |k| k)
            .space(Length::Fixed(40.0))
            .column(1, "Title")
            .label("Plain")
            .width(Length::Fixed(40.0));
        assert_eq!(h.cells.len(), 3);
        // Columns grow unless given a width.
        assert!(matches!(h.cells[1], Cell::Sortable(1, _, Length::Fill, _)));
        assert!(matches!(h.cells[2], Cell::Plain(_, Length::Fixed(_), _)));
        assert!(matches!(h.cells[1], Cell::Sortable(1, _, _, _)));
    }
}
