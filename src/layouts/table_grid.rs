//! `TableGrid`: a grid of columns where each cell takes a **span**, after Mantine's `Grid`. The
//! grid has 12 columns by default ([`columns`](TableGrid::columns)); a cell spans some of them
//! ([`TableCell::span`]) and may skip some before it ([`TableCell::offset`]). Cells that do not fit
//! in the rest of a row start the next one. Use it to lay a page out in regions of different
//! widths; for equal items use a [`FlowGrid`](crate::layouts::FlowGrid).
//!
//! Built on iced's [`Row`](iced::widget::Row): a span is a `FillPortion`, so a row always fills the
//! width, and a row that is not full keeps its cells at their span (an empty cell fills the rest).
//!
//! ```ignore
//! TableGrid::new([
//!     TableCell::new(chart).span(8),
//!     TableCell::new(summary).span(4),
//!     TableCell::new(table).span(12),
//! ])
//! ```

use crate::Element;
use iced::Length;
use iced::alignment::Vertical;
use iced::widget::{column, row, space};

/// One cell of a [`TableGrid`].
pub struct TableCell<'a, M> {
    content: Element<'a, M>,
    span: usize,
    offset: usize,
}

impl<'a, M: 'a> TableCell<'a, M> {
    /// A cell spanning the whole row until [`span`](Self::span) says otherwise.
    pub fn new(content: impl Into<Element<'a, M>>) -> Self {
        Self {
            content: content.into(),
            span: usize::MAX,
            offset: 0,
        }
    }
    /// How many columns the cell spans (at least one).
    pub fn span(mut self, span: usize) -> Self {
        self.span = span.max(1);
        self
    }
    /// How many columns to leave empty before the cell.
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }
}

/// See the [module](self).
pub struct TableGrid<'a, M> {
    cells: Vec<TableCell<'a, M>>,
    columns: usize,
    spacing: f32,
    align: Vertical,
}

impl<'a, M: 'a> TableGrid<'a, M> {
    pub fn new(cells: impl IntoIterator<Item = TableCell<'a, M>>) -> Self {
        Self {
            cells: cells.into_iter().collect(),
            columns: 12,
            spacing: 12.0,
            align: Vertical::Top,
        }
    }
    /// The number of columns a row is divided into (default 12).
    pub fn columns(mut self, n: usize) -> Self {
        self.columns = n.max(1);
        self
    }
    /// The space between cells and between rows (default 12).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = px.max(0.0);
        self
    }
    /// How cells align vertically in a row (default top).
    pub fn align_y(mut self, align: Vertical) -> Self {
        self.align = align;
        self
    }
}

/// The rows the cells fall in: `(offset, span)` of each cell by row, spans clamped to the grid.
fn rows(
    cells: impl IntoIterator<Item = (usize, usize)>,
    columns: usize,
) -> Vec<Vec<(usize, usize)>> {
    let mut out: Vec<Vec<(usize, usize)>> = vec![];
    let mut used = 0;
    for (offset, span) in cells {
        let offset = offset.min(columns - 1);
        let span = span.min(columns - offset);
        if out.is_empty() || used + offset + span > columns {
            out.push(vec![]);
            used = 0;
        }
        out.last_mut()
            .expect("a row was just pushed")
            .push((offset, span));
        used += offset + span;
    }
    out
}

impl<'a, M: 'a> From<TableGrid<'a, M>> for Element<'a, M> {
    fn from(g: TableGrid<'a, M>) -> Self {
        let plan = rows(g.cells.iter().map(|c| (c.offset, c.span)), g.columns);
        let mut cells = g.cells.into_iter();
        let mut out = column![].spacing(g.spacing).width(Length::Fill);
        for line in plan {
            let used: usize = line.iter().map(|(o, s)| o + s).sum();
            let mut r = row![]
                .spacing(g.spacing)
                .align_y(g.align)
                .width(Length::Fill);
            for (offset, span) in line {
                let cell = cells.next().expect("a cell for every planned slot");
                if offset > 0 {
                    r = r.push(space().width(Length::FillPortion(offset as u16)));
                }
                r = r.push(
                    iced::widget::container(cell.content).width(Length::FillPortion(span as u16)),
                );
            }
            if used < g.columns {
                r = r.push(space().width(Length::FillPortion((g.columns - used) as u16)));
            }
            out = out.push(r);
        }
        out.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cells_wrap_when_they_do_not_fit() {
        let r = rows([(0, 8), (0, 4), (0, 12), (0, 5), (0, 5)], 12);
        assert_eq!(r.len(), 3);
        assert_eq!(r[1], vec![(0, 12)]);
        assert_eq!(r[2], vec![(0, 5), (0, 5)]);
    }

    #[test]
    fn an_offset_counts_and_spans_are_clamped() {
        let r = rows([(2, 6), (0, 5), (20, usize::MAX)], 12);
        assert_eq!(r, vec![vec![(2, 6)], vec![(0, 5)], vec![(11, 1)]]);
    }
}
