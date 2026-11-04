//! Three-slot bar layout with `AdwHeaderBar`'s "loose" centering policy.
//!
//! The title is centered in the bar when both sides leave room for it;
//! otherwise it shifts toward the free side, and as a last resort it is
//! clipped. The start and end slots are never squeezed or clipped, so window
//! controls stay reachable at any width (a plain `Row` with equal fill
//! portions overflows instead).

use crate::Theme;
use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Renderer as _, Shell};
use iced::{Element, Event, Length, Point, Rectangle, Size, Vector, mouse};

pub struct Slots<'a, M> {
    children: [Element<'a, M, Theme>; 3],
    height: f32,
    spacing: f32,
}

pub fn slots<'a, M>(
    start: Element<'a, M, Theme>,
    title: Element<'a, M, Theme>,
    end: Element<'a, M, Theme>,
    height: f32,
    spacing: f32,
) -> Slots<'a, M> {
    Slots {
        children: [start, title, end],
        height,
        spacing,
    }
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for Slots<'a, M> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fixed(self.height))
    }

    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(
            &self
                .children
                .iter()
                .map(|c| c.as_widget())
                .collect::<Vec<_>>(),
        );
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let width = limits.max().width.min(f32::MAX / 4.0);
        let height = self.height;
        let natural = layout::Limits::new(Size::ZERO, Size::new(width, height));
        let gap = self.spacing;

        let [start, title, end] = &mut self.children;
        let [t0, t1, t2] = &mut tree.children[..] else {
            unreachable!()
        };
        let s = start.as_widget_mut().layout(t0, renderer, &natural);
        let e = end.as_widget_mut().layout(t2, renderer, &natural);
        let (sw, ew) = (s.size().width, e.size().width);

        // Room for the title if centered in the bar, and if merely between the sides.
        let side = sw.max(ew);
        let centered_room = (width - 2.0 * side - 2.0 * gap).max(0.0);
        let between_room = (width - sw - ew - 2.0 * gap).max(0.0);

        let t = title.as_widget_mut().layout(
            t1,
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(between_room.max(0.0), height)),
        );
        let tw = t.size().width;

        let tx = if tw <= centered_room {
            (width - tw) / 2.0
        } else {
            // Loose: sit as close to center as the sides allow.
            let lo = sw + gap;
            let hi = (width - ew - gap - tw).max(lo);
            ((width - tw) / 2.0).clamp(lo, hi)
        };

        let center_y = |n: layout::Node, x: f32| {
            let y = (height - n.size().height) / 2.0;
            n.move_to(Point::new(x, y.max(0.0)))
        };
        layout::Node::with_children(
            Size::new(width, height),
            vec![center_y(s, 0.0), center_y(t, tx), center_y(e, width - ew)],
        )
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &iced::Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        for ((child, t), l) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child.as_widget_mut().operate(t, l, renderer, operation);
        }
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
        for ((child, t), l) in self
            .children
            .iter_mut()
            .zip(&mut tree.children)
            .zip(layout.children())
        {
            child
                .as_widget_mut()
                .update(t, event, l, cursor, renderer, clipboard, shell, viewport);
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
        self.children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .map(|((c, t), l)| {
                c.as_widget()
                    .mouse_interaction(t, l, cursor, viewport, renderer)
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
        for (i, ((child, t), l)) in self
            .children
            .iter()
            .zip(&tree.children)
            .zip(layout.children())
            .enumerate()
        {
            if i == 1 {
                // Clip the title to its own bounds so it can never paint over the sides.
                renderer.with_layer(l.bounds(), |r| {
                    child
                        .as_widget()
                        .draw(t, r, theme, style, l, cursor, viewport)
                });
            } else {
                child
                    .as_widget()
                    .draw(t, renderer, theme, style, l, cursor, viewport);
            }
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
            &mut self.children,
            tree,
            layout,
            renderer,
            viewport,
            translation,
        )
    }
}

impl<'a, M: 'a> From<Slots<'a, M>> for Element<'a, M, Theme> {
    fn from(s: Slots<'a, M>) -> Self {
        Element::new(s)
    }
}
