//! `FlowGrid`: items of equal size that flow into columns, after Mantine's `SimpleGrid`. Either a
//! fixed number of columns ([`cols`](FlowGrid::cols): one column is a list) or as many as fit,
//! each at most [`max_width`](FlowGrid::max_width) wide. The items fill their cells, and **a row
//! shares one height**, that of its tallest item: every item is laid out again at that height, so
//! a card whose title wraps does not leave its neighbours short (CSS grid's `align-items:
//! stretch`, the rows of a subgrid). An item stretches if its own surface fits its content
//! (`Length::Shrink`), as a card does. Albums, playlists, apps, projects.
//!
//! [`row_height`](FlowGrid::row_height) chooses: [`Shared`](RowHeight::Shared) (default),
//! [`Own`](RowHeight::Own) or [`Fixed`](RowHeight::Fixed).
//!
//! The layout is the grid of iced's `Grid` (same column rule), with that second pass. When cells
//! should differ in width, use a [`TableGrid`](crate::layouts::TableGrid).
//!
//! ```ignore
//! FlowGrid::new(albums.iter().map(album_card)).max_width(180.0)
//! FlowGrid::new(albums.iter().map(album_row)).cols(1)   // a list
//! ```

use crate::{Element, Theme};
use iced::advanced::widget::{Operation, Tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer};
use iced::{Event, Length, Rectangle, Size, Vector};

/// How tall the items of a row are: see [`FlowGrid::row_height`].
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum RowHeight {
    /// The height of the row's tallest item, for all of them (CSS grid's `stretch`, a subgrid).
    #[default]
    Shared,
    /// Each item at its own height, top-aligned; the row is as tall as its tallest.
    Own,
    /// Every item exactly this tall, whatever it holds.
    Fixed(f32),
}

/// See the [module](self).
pub struct FlowGrid<'a, M> {
    items: Vec<Element<'a, M>>,
    columns: Columns,
    spacing: f32,
    row_height: RowHeight,
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
            row_height: RowHeight::Shared,
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
    /// How tall the items of a row are: [`Shared`](RowHeight::Shared) (the default) gives them all
    /// the height of the tallest, [`Own`](RowHeight::Own) lets each keep its own, and
    /// [`Fixed`](RowHeight::Fixed) sets one height for all.
    pub fn row_height(mut self, row_height: RowHeight) -> Self {
        self.row_height = row_height;
        self
    }
    /// The space between items (default 12).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = px.max(0.0);
        self
    }
}

/// How many columns fit in `width`, and how wide each is.
fn cells(columns: Columns, width: f32, spacing: f32) -> (usize, f32) {
    let n = match columns {
        // As in iced's `Grid`: the count that fits when each column may be up to `max` wide.
        Columns::Fluid(max) => ((width + spacing) / (max + spacing)).ceil() as usize,
        Columns::Fixed(n) => n,
    }
    .max(1);
    (n, (width - spacing * (n - 1) as f32) / n as f32)
}

impl<'a, M: 'a> Widget<M, Theme, iced::Renderer> for FlowGrid<'a, M> {
    fn children(&self) -> Vec<Tree> {
        self.items.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.items);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(Length::Fill).height(Length::Shrink);
        let width = limits.max().width;
        let (per_row, cell_width) = cells(self.columns, width, self.spacing);
        let mut nodes = Vec::with_capacity(self.items.len());
        let mut y = 0.0f32;

        let rows = self
            .items
            .chunks_mut(per_row)
            .zip(tree.children.chunks_mut(per_row));
        for (row, states) in rows {
            // First pass: each item at its own height, to find the row's tallest.
            let own = layout::Limits::new(
                Size::new(cell_width, 0.0),
                Size::new(cell_width, f32::INFINITY),
            );
            let mut measured: Vec<layout::Node> = row
                .iter_mut()
                .zip(states.iter_mut())
                .map(|(item, state)| item.as_widget_mut().layout(state, renderer, &own))
                .collect();
            let tallest = measured
                .iter()
                .map(|n| n.size().height)
                .filter(|h| h.is_finite())
                .fold(0.0f32, f32::max);
            let (tallest, stretch) = match self.row_height {
                RowHeight::Shared => (tallest, true),
                RowHeight::Own => (tallest, false),
                RowHeight::Fixed(px) => (px.max(0.0), true),
            };

            // Second pass (not for `Own`): every item is given that height, so a card's surface
            // fills its cell like a subgrid row. Items that cannot be measured (they fill the
            // height) keep the first pass.
            if stretch && tallest > 0.0 {
                let shared = layout::Limits::new(
                    Size::new(cell_width, tallest),
                    Size::new(cell_width, tallest),
                );
                for ((item, state), node) in
                    row.iter_mut().zip(states.iter_mut()).zip(&mut measured)
                {
                    *node = item.as_widget_mut().layout(state, renderer, &shared);
                }
            }

            for (i, node) in measured.into_iter().enumerate() {
                let x = i as f32 * (cell_width + self.spacing);
                nodes.push(node.move_to((x, y)));
            }
            y += tallest + self.spacing;
        }
        if !nodes.is_empty() {
            y -= self.spacing;
        }
        layout::Node::with_children(Size::new(width, y.max(0.0)), nodes)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.items
                .iter_mut()
                .zip(&mut tree.children)
                .zip(layout.children())
                .for_each(|((item, state), layout)| {
                    item.as_widget_mut()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &iced::Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, M>,
        viewport: &Rectangle,
    ) {
        for ((item, state), layout) in self
            .items
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            item.as_widget_mut().update(
                state, event, layout, cursor, renderer, clipboard, shell, viewport,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.items
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((item, state), layout)| {
                item.as_widget()
                    .mouse_interaction(state, layout, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let Some(viewport) = layout.bounds().intersection(viewport) else {
            return;
        };
        for ((item, state), layout) in self
            .items
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .filter(|(_, layout)| layout.bounds().intersects(&viewport))
        {
            item.as_widget()
                .draw(state, renderer, theme, style, layout, cursor, &viewport);
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        overlay::from_children(
            &mut self.items,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, M: 'a> From<FlowGrid<'a, M>> for Element<'a, M> {
    fn from(g: FlowGrid<'a, M>) -> Self {
        iced::Element::new(g)
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
        assert_eq!(
            g.row_height,
            RowHeight::Shared,
            "rows share a height by default"
        );
        let g = g.row_height(RowHeight::Own);
        assert_eq!(g.row_height, RowHeight::Own);
    }
}
