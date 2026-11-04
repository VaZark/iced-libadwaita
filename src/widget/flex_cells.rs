//! `FlexCells`: a line of cells sized as CSS flex sizes them. Each cell has a [`Flex`] (basis,
//! grow, shrink, minimum), and the line divides its width with
//! [`primitives::flex::resolve`](crate::primitives::flex::resolve): free space goes to the cells
//! that grow, a shortage is taken from the cells that shrink in proportion to their basis, and
//! **no cell goes below its minimum**.
//!
//! It is the line of a table: the header and every row are a `FlexCells` over the same
//! [`Flex`] specs, so their columns line up at any width, and a small column keeps what its label
//! needs while a big one gives way ("Tim" never happens). [`ItemRow`](super::ItemRow) and
//! [`SortHeader`](super::SortHeader) are built on it.
//!
//! ```ignore
//! FlexCells::new()
//!     .cell(Flex::fixed(40.0), picture)               // never shrinks
//!     .cell(Flex::grow(1.0, 60.0), title)             // takes the free space, down to 60
//!     .cell(Flex::shrink(160.0, 80.0), album)         // starts at 160, shrinks to 80
//!     .cell(Flex::shrink(44.0, 44.0), time)           // keeps its 44
//! ```

use super::Element;
use crate::Theme;
use crate::primitives::flex::{self, Item};
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell, overlay};
use iced::{Alignment, Event, Length, Point, Rectangle, Size, Vector, mouse};

/// How one cell takes part in the line. See [`primitives::flex`](crate::primitives::flex).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Flex(pub Item);

impl Flex {
    /// Keeps its size.
    pub fn fixed(size: f32) -> Self {
        Self(Item::fixed(size))
    }
    /// Starts at `basis` and gives way down to `min`; does not grow.
    pub fn shrink(basis: f32, min: f32) -> Self {
        Self(Item::shrinking(basis, min))
    }
    /// Takes `grow` shares of the free space; starts at `min` and never goes below it. A title
    /// column is `Flex::grow(1.0, floor)`.
    pub fn grow(grow: f32, min: f32) -> Self {
        // A basis of the minimum plus a generous default, so a growing column is wider than the
        // fixed ones at rest and is the one that gives first when short.
        Self(Item::flex(min.max(120.0), grow, min))
    }
}

/// See the [module](self).
pub struct FlexCells<'a, M> {
    cells: Vec<(Flex, Element<'a, M>)>,
    gap: f32,
    align: Alignment,
}

impl<'a, M: 'a> Default for FlexCells<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, M: 'a> FlexCells<'a, M> {
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            gap: crate::Metrics::desktop().space_m,
            align: Alignment::Center,
        }
    }
    pub fn cell(mut self, flex: Flex, content: impl Into<Element<'a, M>>) -> Self {
        self.cells.push((flex, content.into()));
        self
    }
    /// The space between cells.
    pub fn spacing(mut self, gap: f32) -> Self {
        self.gap = gap.max(0.0);
        self
    }
    /// How cells sit across the height of the line (default centered).
    pub fn align_y(mut self, align: Alignment) -> Self {
        self.align = align;
        self
    }
    /// The widths the cells get in a line of `width`.
    pub fn widths(&self, width: f32) -> Vec<f32> {
        let items: Vec<Item> = self.cells.iter().map(|(f, _)| f.0).collect();
        flex::resolve(&items, width, self.gap)
    }
}

impl<'a, M: 'a> Widget<M, Theme, iced::Renderer> for FlexCells<'a, M> {
    fn children(&self) -> Vec<Tree> {
        self.cells.iter().map(|(_, e)| Tree::new(e)).collect()
    }
    fn diff(&self, tree: &mut Tree) {
        let kids: Vec<&Element<'a, M>> = self.cells.iter().map(|(_, e)| e).collect();
        tree.diff_children(&kids.iter().map(|e| e.as_widget()).collect::<Vec<_>>());
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
        let width = limits.max().width;
        let widths = self.widths(width);
        let mut x = 0.0;
        let mut nodes = Vec::with_capacity(self.cells.len());
        let mut height = 0.0f32;
        for (((_, e), t), w) in self.cells.iter_mut().zip(&mut tree.children).zip(&widths) {
            let node = e.as_widget_mut().layout(
                t,
                renderer,
                &layout::Limits::new(Size::new(*w, 0.0), Size::new(*w, f32::INFINITY)),
            );
            height = height.max(node.size().height);
            nodes.push((x, node));
            x += w + self.gap;
        }
        let nodes = nodes
            .into_iter()
            .map(|(x, node)| {
                let y = match self.align {
                    Alignment::Start => 0.0,
                    Alignment::Center => (height - node.size().height) / 2.0,
                    Alignment::End => height - node.size().height,
                };
                node.move_to(Point::new(x, y))
            })
            .collect();
        layout::Node::with_children(Size::new(width, height), nodes)
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
        for (((_, e), t), l) in self
            .cells
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            e.as_widget_mut()
                .update(t, event, l, cursor, renderer, clipboard, shell, viewport);
        }
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        for (((_, e), t), l) in self
            .cells
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            e.as_widget_mut().operate(t, l, renderer, operation);
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
        self.cells
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|(((_, e), t), l)| {
                e.as_widget()
                    .mouse_interaction(t, l, cursor, viewport, renderer)
            })
            .max()
            .unwrap_or_default()
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        let mut found = Vec::new();
        for (((_, e), t), l) in self
            .cells
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            if let Some(o) = e
                .as_widget_mut()
                .overlay(t, l, renderer, viewport, translation)
            {
                found.push(o);
            }
        }
        (!found.is_empty()).then(|| overlay::Group::with_children(found).overlay())
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
        use iced::advanced::Renderer as _;
        for (((_, e), t), l) in self.cells.iter().zip(&tree.children).zip(layout.children()) {
            // Each cell is clipped to its own width: a cell never paints into its neighbor.
            if let Some(view) = l.bounds().intersection(viewport) {
                renderer.with_layer(view, |r| {
                    e.as_widget().draw(t, r, theme, style, l, cursor, &view);
                });
            }
        }
    }
}

impl<'a, M: 'a> From<FlexCells<'a, M>> for Element<'a, M> {
    fn from(c: FlexCells<'a, M>) -> Self {
        Element::new(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cells_take_the_widths_the_flex_rules_give() {
        let c: FlexCells<'_, ()> = FlexCells::new()
            .spacing(10.0)
            .cell(Flex::fixed(40.0), iced::widget::space())
            .cell(Flex::grow(1.0, 60.0), iced::widget::space())
            .cell(Flex::shrink(44.0, 44.0), iced::widget::space());
        let w = c.widths(300.0);
        assert_eq!(w[0], 40.0);
        assert_eq!(w[2], 44.0);
        // The title takes what is left: 300 - 20 gaps - 40 - 44.
        assert!((w[1] - 196.0).abs() < 1e-3);
        // A narrow line: the small cells keep their sizes, the title gives way to its minimum.
        let w = c.widths(150.0);
        assert_eq!((w[0], w[2]), (40.0, 44.0));
        assert_eq!(w[1], 60.0);
    }
}
