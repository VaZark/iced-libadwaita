//! `FlowGrid`: items of equal size that flow into columns, after Mantine's `SimpleGrid`. Either a
//! fixed number of columns ([`cols`](FlowGrid::cols): one column is a list) or as many as fit,
//! each at most [`max_width`](FlowGrid::max_width) wide. The items fill their cells and keep
//! their own height (a row is as tall as its tallest item). Albums, playlists, apps, projects.
//!
//! A thin, named form of iced's [`Grid`], with the sizing that suits items. When cells should
//! differ in width, use a [`TableGrid`](crate::layouts::TableGrid).
//!
//! ```ignore
//! FlowGrid::new(albums.iter().map(album_card)).max_width(180.0)
//! FlowGrid::new(albums.iter().map(album_row)).cols(1)   // a list
//! ```

use crate::Element;
use iced::widget::grid::{self, Grid};

/// See the [module](self).
pub struct FlowGrid<'a, M> {
    items: Vec<Element<'a, M>>,
    columns: Columns,
    spacing: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Columns {
    /// As many as fit, each at most this wide.
    Fluid(f32),
    Fixed(usize),
}

impl<'a, M: 'a> FlowGrid<'a, M> {
    pub fn new(items: impl IntoIterator<Item = Element<'a, M>>) -> Self {
        Self {
            items: items.into_iter().collect(),
            columns: Columns::Fluid(200.0),
            spacing: 12.0,
        }
    }
    /// As many columns as fit, each at most `px` wide (default 200).
    pub fn max_width(mut self, px: f32) -> Self {
        self.columns = Columns::Fluid(px.max(1.0));
        self
    }
    /// Exactly `n` columns (at least one). One column is a list.
    pub fn cols(mut self, n: usize) -> Self {
        self.columns = Columns::Fixed(n.max(1));
        self
    }
    /// The space between items (default 12).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = px.max(0.0);
        self
    }
}

impl<'a, M: 'a> From<FlowGrid<'a, M>> for Element<'a, M> {
    fn from(g: FlowGrid<'a, M>) -> Self {
        let grid = Grid::with_children(g.items)
            .spacing(g.spacing)
            .height(grid::Sizing::EvenlyDistribute(iced::Length::Shrink));
        match g.columns {
            Columns::Fluid(w) => grid.fluid(w),
            Columns::Fixed(n) => grid.columns(n),
        }
        .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builder_keeps_its_sizes() {
        let g: FlowGrid<'_, ()> = FlowGrid::new([]).max_width(160.0).spacing(8.0);
        assert_eq!((g.columns, g.spacing), (Columns::Fluid(160.0), 8.0));
        let g: FlowGrid<'_, ()> = FlowGrid::new([]).cols(0);
        assert_eq!(g.columns, Columns::Fixed(1));
    }
}
