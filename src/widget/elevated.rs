//! Elevation that follows the shape of the surface.
//!
//! Adwaita stacks several CSS box-shadows (`0 1px 3px 1px`, `0 2px 6px 2px`, …). iced draws one
//! shadow per quad and has no *spread*, so merging the layers into one blurred shadow smears it
//! into a rectangular halo. The CSS trick for a shadow that hugs a rounded border is to draw each
//! layer as its own shape behind the surface: inflated by the layer's spread, with its corner
//! radius grown by the same amount, offset by the layer's `x y`, and blurred by its blur. This
//! widget does exactly that, one quad per layer, then draws the surface on top.

use crate::Theme;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{tree, Operation, Tree, Widget};
use iced::advanced::{overlay, Clipboard, Renderer as _, Shell};
use iced::{mouse, Background, Border, Color, Element, Event, Length, Rectangle, Shadow, Size, Vector};

/// iced's `blur_radius` falls off over a much shorter distance than a CSS `blur` (which is a
/// gaussian with `sigma = blur / 2`), so with the CSS values the shadow collapses into a thin
/// dark band that reads as an outline. This gain restores the CSS softness.
const BLUR_GAIN: f32 = 3.0;

/// A shadow wider than the surface on every side reads as an outline. Pulling the shape in by
/// `INSET` (a negative spread) and pushing it down by `LIFT` (a larger y offset) keeps the
/// shadow mostly below the card, like light from above: elevation, not a halo.
const INSET: f32 = 2.0;
const LIFT: f32 = 2.0;

/// Which elevation of the theme to draw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `.card` / `.boxed-list`, with the card radius.
    Card,
}

/// Draws the theme's layered shadow behind `content`, following its rounded border.
pub struct Elevated<'a, M> {
    content: Element<'a, M, Theme>,
    kind: Kind,
}

/// Wraps `content` (a surface with its own background and border) in a card elevation.
pub fn card_shadow<'a, M>(content: impl Into<Element<'a, M, Theme>>) -> Elevated<'a, M> {
    Elevated { content: content.into(), kind: Kind::Card }
}

impl<'a, M> Widget<M, Theme, iced::Renderer> for Elevated<'a, M> {
    fn tag(&self) -> tree::Tag {
        self.content.as_widget().tag()
    }
    fn state(&self) -> tree::State {
        self.content.as_widget().state()
    }
    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }
    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }
    fn layout(&mut self, tree: &mut Tree, renderer: &iced::Renderer, limits: &layout::Limits) -> layout::Node {
        self.content.as_widget_mut().layout(tree, renderer, limits)
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
        self.content.as_widget_mut().update(tree, event, layout, cursor, renderer, clipboard, shell, viewport);
    }
    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &iced::Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(tree, layout, cursor, viewport, renderer)
    }
    fn operate(&mut self, tree: &mut Tree, layout: Layout<'_>, renderer: &iced::Renderer, operation: &mut dyn Operation) {
        self.content.as_widget_mut().operate(tree, layout, renderer, operation);
    }
    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &iced::Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, M, Theme, iced::Renderer>> {
        self.content.as_widget_mut().overlay(tree, layout, renderer, viewport, translation)
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
        let b = layout.bounds();
        let (elevation, radius) = match self.kind {
            Kind::Card => (&theme.shadows.card, theme.metrics.radius_l),
        };
        let s = &theme.shadows;
        if s.enabled {
            for l in elevation.layers.iter().filter(|l| !l.is_ring()) {
                let spread = (l.spread - INSET) * s.scale;
                renderer.fill_quad(
                    Quad {
                        // Inflate by the spread and grow the radius with it, so the shadow's
                        // corners stay concentric with the border.
                        bounds: Rectangle {
                            x: b.x - spread,
                            y: b.y - spread,
                            width: b.width + 2.0 * spread,
                            height: b.height + 2.0 * spread,
                        },
                        border: Border::default().rounded(radius + spread),
                        shadow: Shadow {
                            color: Color { a: (l.color.a * s.intensity).min(1.0), ..l.color },
                            offset: Vector::new(l.x * s.scale, (l.y + LIFT) * s.scale),
                            blur_radius: l.blur * s.scale * BLUR_GAIN,
                        },
                        snap: false,
                    },
                    Background::Color(Color::TRANSPARENT),
                );
            }
        }
        self.content.as_widget().draw(tree, renderer, theme, style, layout, cursor, viewport);
    }
}

impl<'a, M: 'a> From<Elevated<'a, M>> for Element<'a, M, Theme> {
    fn from(e: Elevated<'a, M>) -> Self {
        Element::new(e)
    }
}
