//! The drawing pieces of the chart widgets: series colors, paths, axes and grids, tooltips and
//! legends. They draw what the [`primitives::chart`](crate::primitives::chart) layouts computed.
//!
//! * [`color`]: the categorical series colors, by theme;
//! * [`path`]: path commands to a canvas path;
//! * [`style`]: which elements a chart draws and in which colors ([`ChartStyle`](style::ChartStyle));
//! * [`axis`]: gridlines, a value axis and a band axis;
//! * [`tooltip`]: the box that follows the pointer;
//! * [`legend`]: swatches with names (an element, not a canvas drawing).

pub mod axis;
pub mod color;
pub mod legend;
pub mod path;
pub mod style;
pub mod tooltip;

use crate::Theme;
use iced::alignment::Vertical;
use iced::widget::canvas::{Frame, Text};
use iced::{Color, Pixels, Point};

/// Text size of axis labels and tooltips.
pub const LABEL_SIZE: f32 = 12.0;

/// Where a label sits against its position.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    /// The position is the label's left edge.
    Start,
    Middle,
    /// The position is the label's right edge.
    End,
}

/// Draws `content` at `at`, vertically centered on it.
pub fn label(
    frame: &mut Frame,
    content: impl Into<String>,
    at: Point,
    anchor: Anchor,
    color: Color,
) {
    frame.fill_text(Text {
        content: content.into(),
        position: at,
        color,
        size: Pixels(LABEL_SIZE),
        align_x: match anchor {
            Anchor::Start => iced::widget::text::Alignment::Left,
            Anchor::Middle => iced::widget::text::Alignment::Center,
            Anchor::End => iced::widget::text::Alignment::Right,
        },
        align_y: Vertical::Center,
        ..Text::default()
    });
}

/// The color of axis text and gridlines for `theme`: `(text, grid)`.
pub fn ink(theme: &Theme) -> (Color, Color) {
    (
        theme.semantic.dim(),
        crate::color::alpha(theme.semantic.window.fg, 0.12),
    )
}

/// A rough width of `text` at [`LABEL_SIZE`], for sizing margins without measuring.
pub fn text_width(text: &str) -> f32 {
    text.chars().count() as f32 * LABEL_SIZE * 0.6
}
