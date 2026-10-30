//! A two-item flex line with `flex-wrap` semantics, used by every row.
//!
//! `main` (prefix + title) and `aside` (the suffix control) share a line while
//! `main` keeps at least `main_basis` px; when they can't, `aside` wraps to its
//! own line below, like CSS `flex-wrap: wrap`. This avoids the title being
//! crushed (or the control overlapping it) in narrow windows.

use crate::{CollapseBehavior, Metrics, Theme};
use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::widget::{Operation, Tree, Widget};
use iced::advanced::{Clipboard, Shell};
use iced::{mouse, Element, Event, Length, Point, Rectangle, Size, Vector};

pub struct FlexRow<'a, M> {
    children: [Element<'a, M, Theme>; 2],
    gap: f32,
    line_gap: f32,
    main_basis: f32,
    aside_basis: Option<f32>,
    collapse: CollapseBehavior,
}

impl<'a, M> FlexRow<'a, M> {
    /// `main` takes the leftover space on the first line; `aside` is the control.
    pub fn new(main: impl Into<Element<'a, M, Theme>>, aside: impl Into<Element<'a, M, Theme>>) -> Self {
        let m = Metrics::default();
        Self {
            children: [main.into(), aside.into()],
            gap: m.row_spacing,
            line_gap: m.space_s,
            main_basis: m.row_wrap_basis,
            aside_basis: None,
            collapse: m.row_collapse,
        }
    }
    /// Space between `main` and `aside` on one line.
    pub fn spacing(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }
    /// Space between the two lines when wrapped.
    pub fn line_spacing(mut self, gap: f32) -> Self {
        self.line_gap = gap;
        self
    }
    /// The minimum width `main` keeps on the first line; below it, `aside` wraps.
    pub fn main_basis(mut self, px: f32) -> Self {
        self.main_basis = px;
        self
    }
    /// The width `aside` takes while sharing a line. Needed for `Fill`-width
    /// controls (sliders, entries), which have no natural width to measure.
    pub fn aside_basis(mut self, px: f32) -> Self {
        self.aside_basis = Some(px);
        self
    }
    /// What happens to `aside` when it wraps (see [`CollapseBehavior`]).
    pub fn collapse(mut self, c: CollapseBehavior) -> Self {
        self.collapse = c;
        self
    }
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for FlexRow<'a, M> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Shrink)
    }

    fn children(&self) -> Vec<Tree> {
        self.children.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.children.iter().map(|c| c.as_widget()).collect::<Vec<_>>());
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        let max = limits.max();
        let width = if max.width.is_finite() { max.width } else { 0.0 };
        let [main, aside] = &mut self.children;
        let [t0, t1] = &mut tree.children[..] else { unreachable!() };

        // Measure the control at its natural size.
        let measure = self.aside_basis.unwrap_or(width).min(width);
        let a = aside
            .as_widget_mut()
            .layout(t1, renderer, &layout::Limits::new(Size::ZERO, Size::new(measure, f32::INFINITY)));
        let aw = a.size().width;

        let fits = width - aw - self.gap >= self.main_basis;
        if fits {
            let m = main.as_widget_mut().layout(
                t0,
                renderer,
                &layout::Limits::new(Size::ZERO, Size::new(width - aw - self.gap, f32::INFINITY)),
            );
            let h = m.size().height.max(a.size().height);
            let my = (h - m.size().height) / 2.0;
            let ay = (h - a.size().height) / 2.0;
            layout::Node::with_children(
                Size::new(width, h),
                vec![m.move_to(Point::new(0.0, my)), a.move_to(Point::new(width - aw, ay))],
            )
        } else {
            let m = main.as_widget_mut().layout(
                t0,
                renderer,
                &layout::Limits::new(Size::ZERO, Size::new(width, f32::INFINITY)),
            );
            let mh = m.size().height;
            // Stacked: the control takes 100% width, so `Fill` children stretch.
            let a = match self.collapse {
                CollapseBehavior::Flex => a,
                CollapseBehavior::Stacked => aside.as_widget_mut().layout(
                    t1,
                    renderer,
                    &layout::Limits::new(Size::new(width, 0.0), Size::new(width, f32::INFINITY)),
                ),
            };
            let ah = a.size().height;
            layout::Node::with_children(
                Size::new(width, mh + self.line_gap + ah),
                vec![
                    m.move_to(Point::new(0.0, 0.0)),
                    a.move_to(Point::new(0.0, mh + self.line_gap)),
                ],
            )
        }
    }

    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &iced::Renderer, operation: &mut dyn Operation) {
        operation.container(None, layout.bounds());
        for ((c, t), l) in self.children.iter_mut().zip(&mut tree.children).zip(layout.children()) {
            c.as_widget_mut().operate(t, l, renderer, operation);
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
        for ((c, t), l) in self.children.iter_mut().zip(&mut tree.children).zip(layout.children()) {
            c.as_widget_mut().update(t, event, l, cursor, renderer, clipboard, shell, viewport);
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
            .map(|((c, t), l)| c.as_widget().mouse_interaction(t, l, cursor, viewport, renderer))
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
        for ((c, t), l) in self.children.iter().zip(&tree.children).zip(layout.children()) {
            c.as_widget().draw(t, renderer, theme, style, l, cursor, viewport);
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
        overlay::from_children(&mut self.children, tree, layout, renderer, viewport, translation)
    }
}

impl<'a, M: 'a> From<FlexRow<'a, M>> for Element<'a, M, Theme> {
    fn from(f: FlexRow<'a, M>) -> Self {
        Element::new(f)
    }
}
