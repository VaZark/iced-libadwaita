//! The tooltip every control uses: a small label in the on-screen-display style, a few pixels
//! from what it describes. Use this instead of building an `iced` tooltip by hand, so they all
//! look alike.

use crate::Element;
use iced::widget::{text, tooltip as iced_tooltip};

pub use iced_tooltip::Position as TooltipPosition;

/// The space between the tooltip and what it describes.
const GAP: f32 = 6.0;

/// `content` with a tooltip saying `label` on hover, on the `position` side of it.
pub fn tooltip<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    label: impl Into<String>,
    position: TooltipPosition,
) -> Element<'a, M> {
    iced_tooltip(content, text(label.into()).size(12), position)
        .gap(GAP)
        .style(crate::styles::container::osd)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_wraps_any_content() {
        let _: Element<'_, ()> = tooltip(text("x"), "42%", TooltipPosition::Left);
    }
}
