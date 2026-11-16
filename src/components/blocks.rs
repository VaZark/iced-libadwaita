//! `ProgressBlocks`: the classic progress bar, a sunken trough with a row of separate blocks.
//!
//! A theme asks for it with `[progress] style=blocks`; [`themed_progress`] then draws it, and
//! returns the stock progress bar when the theme does not. Tokens: `[progress] block-width` (8),
//! `block-gap` (2), `block-inset` (the space between the trough's frame and the blocks, 0), and
//! the trough's frame, `[frame.progress]` (`border-style`, `border-width`, the bevel colors and
//! `bg` for the trough's fill). The blocks take the accent color.

use crate::primitives::{bevel, pixmap};
use crate::{Element, Theme};
use iced::advanced::Renderer as _;
use iced::advanced::layout::{self, Layout};
use iced::advanced::renderer::{self, Quad};
use iced::advanced::widget::{Tree, Widget};
use iced::{Length, Point, Rectangle, Size, mouse};
use std::ops::RangeInclusive;

/// Whether the installed theme draws progress bars as blocks.
pub fn blocks_style() -> bool {
    crate::theme_config::current().text("progress", "style") == Some("blocks")
}

/// A progress bar as the theme draws it: blocks when `[progress] style=blocks`, else the toolkit's
/// own smooth bar.
pub fn themed_progress<'a, M: 'a>(range: RangeInclusive<f32>, value: f32) -> Element<'a, M> {
    if blocks_style() {
        ProgressBlocks::new(range, value).into()
    } else {
        crate::components::progress_bar(range, value).into()
    }
}

/// See the [module](self).
pub struct ProgressBlocks {
    fraction: f32,
    height: f32,
    width: Length,
}

impl ProgressBlocks {
    pub fn new(range: RangeInclusive<f32>, value: f32) -> Self {
        let (lo, hi) = (*range.start(), *range.end());
        let fraction = if hi > lo {
            ((value - lo) / (hi - lo)).clamp(0.0, 1.0)
        } else {
            0.0
        };
        Self {
            fraction,
            height: crate::Metrics::desktop().progress_height,
            width: Length::Fill,
        }
    }
    /// The bar's height in px (default `[metrics] progress-height`).
    pub fn height(mut self, px: f32) -> Self {
        self.height = px.max(1.0);
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    /// The filled share, 0 to 1.
    pub fn fraction(&self) -> f32 {
        self.fraction
    }
}

impl<M> Widget<M, Theme, iced::Renderer> for ProgressBlocks {
    fn size(&self) -> Size<Length> {
        Size::new(self.width, Length::Fixed(self.height))
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &iced::Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, Length::Fixed(self.height))
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut iced::Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let quad = |renderer: &mut iced::Renderer, r: Rectangle, color: iced::Color| {
            renderer.fill_quad(
                Quad {
                    bounds: Rectangle::new(Point::new(bounds.x + r.x, bounds.y + r.y), r.size()),
                    border: iced::Border::default(),
                    shadow: iced::Shadow::default(),
                    snap: true,
                },
                color,
            );
        };
        let frame = "frame.progress";
        // The trough: its fill, then its sunken frame.
        let fill = theme
            .token_color(frame, "bg")
            .unwrap_or(theme.semantic.window.bg);
        quad(renderer, Rectangle::new(Point::ORIGIN, bounds.size()), fill);
        let rings = crate::components::bevel_of(frame);
        let ring_px = rings.map_or(0.0, |(_, w)| bevel::rings(w) as f32);
        if let Some((style, w)) = rings {
            let colors = theme.bevel_colors(frame);
            for e in bevel::edges(style, bevel::rings(w), bounds.size()) {
                quad(renderer, e.rect, colors.tone(e.tone));
            }
        }
        let inset = theme.token_number("progress", "block-inset").unwrap_or(0.0) + ring_px;
        let inner_w = (bounds.width - 2.0 * inset).max(0.0);
        let inner_h = (bounds.height - 2.0 * inset).max(0.0);
        let block = theme.token_number("progress", "block-width").unwrap_or(8.0);
        let gap = theme.token_number("progress", "block-gap").unwrap_or(2.0);
        let color = theme.semantic.accent.bg;
        for (x, w) in pixmap::progress_blocks(inner_w, self.fraction, block, gap) {
            quad(
                renderer,
                Rectangle::new(Point::new(inset + x, inset), Size::new(w, inner_h)),
                color,
            );
        }
    }
}

impl<'a, M: 'a> From<ProgressBlocks> for Element<'a, M> {
    fn from(p: ProgressBlocks) -> Self {
        iced::Element::new(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fraction_follows_the_range_and_is_clamped() {
        assert_eq!(ProgressBlocks::new(0.0..=100.0, 25.0).fraction(), 0.25);
        assert_eq!(ProgressBlocks::new(0.0..=100.0, 250.0).fraction(), 1.0);
        assert_eq!(ProgressBlocks::new(10.0..=10.0, 10.0).fraction(), 0.0);
    }

    #[test]
    fn the_stock_bar_is_what_an_unset_theme_gets() {
        assert!(!blocks_style());
        let _: Element<'_, ()> = themed_progress(0.0..=1.0, 0.5);
        let _: Element<'_, ()> = ProgressBlocks::new(0.0..=1.0, 0.5).height(18.0).into();
    }
}
