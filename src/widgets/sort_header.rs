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
use crate::components::{ButtonProps, ContainerProps, TextProps};
use crate::widgets::TableHeader;

use crate::Metrics;
use crate::layouts::{Flex, FlexCells};
use crate::primitives::sorting::SortState;
use iced::alignment::Horizontal;
use iced::widget::{container, space};
use iced::{Length, Padding};

pub(crate) use super::table_header::label_floor;

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
    label_props: TextProps,
    sort_button_props: ButtonProps,
    cell_props: ContainerProps,
    container_props: ContainerProps,
}

impl<'a, K: PartialEq + Clone + 'a, M: Clone + 'a> SortHeader<'a, K, M> {
    /// `state` says which column is sorted and how; `on_sort` gets the key of a pressed label
    /// (the application calls [`SortState::activate`] with it).
    /// Presentation overrides applied after the child's defaults.
    pub fn label_props(mut self, props: TextProps) -> Self {
        self.label_props = props;
        self
    }
    /// Presentation overrides applied after the child's defaults.
    pub fn sort_button_props(mut self, props: ButtonProps) -> Self {
        self.sort_button_props = props;
        self
    }
    /// Presentation overrides applied after the child's defaults.
    pub fn cell_props(mut self, props: ContainerProps) -> Self {
        self.cell_props = props;
        self
    }
    /// Presentation overrides applied after the child's defaults.
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = props;
        self
    }
    pub fn new(state: &'a SortState<K>, on_sort: impl Fn(K) -> M + 'a) -> Self {
        Self {
            label_props: TextProps::default(),
            sort_button_props: ButtonProps::default(),
            cell_props: ContainerProps::default(),
            container_props: ContainerProps::default(),
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
                        h.cell_props.apply(container(
                            TableHeader::new(label)
                                .metrics(m)
                                .align(align)
                                .label_props(h.label_props.clone()),
                        )),
                    )
                }
                Cell::Sortable(key, label, w, align) => {
                    let flex = flex_of(w, &label);
                    let direction = h.state.direction(&key);
                    let msg = (h.on_sort)(key);
                    line.cell(
                        flex,
                        h.cell_props.apply(container(
                            TableHeader::new(label)
                                .metrics(m)
                                .align(align)
                                .label_props(h.label_props.clone())
                                .button_props(h.sort_button_props.clone())
                                .sortable(direction, msg),
                        )),
                    )
                }
            };
        }
        h.container_props
            .apply(container(line).padding(h.padding))
            .into()
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
