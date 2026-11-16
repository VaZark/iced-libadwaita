//! `GtkScrolledWindow` with Adwaita's scrollbar proportions.
//!
//! iced's `scrollable` defaults to a 10px bar that floats over the content. Here the bar sits
//! beside it: while the content overflows it is narrowed by the bar and `scrollbar_gutter`
//! (Mantine's `offsetScrollbars`), so rows and text never run under the bar, and nothing is
//! reserved while it fits. See `docs/hig-inferences/gtk4/scrollbar.md`.

use crate::{Metrics, Theme};
use iced::Element;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{Scrollable, scrollable};

/// The scrollbar configuration: beside the content, `scrollbar_gutter` away from it.
pub fn scrollbar() -> Scrollbar {
    let m = Metrics::desktop();
    Scrollbar::new()
        .width(m.scrollbar_width)
        .scroller_width(m.scrollbar_slider_width)
        .margin(m.scrollbar_margin)
        .spacing(m.scrollbar_gutter)
}

/// Vertical scrolling with the Adwaita scrollbar.
pub fn scrolled_window<'a, M: 'a>(
    content: impl Into<Element<'a, M, Theme>>,
) -> Scrollable<'a, M, Theme> {
    scrollable(content).direction(Direction::Vertical(scrollbar()))
}

/// Horizontal scrolling with the Adwaita scrollbar.
pub fn scrolled_window_horizontal<'a, M: 'a>(
    content: impl Into<Element<'a, M, Theme>>,
) -> Scrollable<'a, M, Theme> {
    scrollable(content).direction(Direction::Horizontal(scrollbar()))
}

/// Both axes.
pub fn scrolled_window_both<'a, M: 'a>(
    content: impl Into<Element<'a, M, Theme>>,
) -> Scrollable<'a, M, Theme> {
    scrollable(content).direction(Direction::Both {
        vertical: scrollbar(),
        horizontal: scrollbar(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::{layout, renderer::Headless, widget::Tree};
    use iced::widget::{column, container, space};
    use iced::{Length, Size};

    /// Width of the content `scrolled_window` lays out inside 300px, for content `height` tall in
    /// a 200px window.
    fn content_width(height: f32) -> f32 {
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        let body =
            column![container(space().height(height)).width(Length::Fill)].width(Length::Fill);
        let mut element: Element<'_, (), Theme> = scrolled_window(body).into();
        let mut state = Tree::new(&element);
        let node = element.as_widget_mut().layout(
            &mut state,
            &renderer,
            &layout::Limits::new(Size::ZERO, Size::new(300.0, 200.0)),
        );
        node.children()
            .first()
            .map_or(node.size().width, |c| c.size().width)
    }

    #[test]
    fn the_scrollbar_leaves_the_content_clear_only_while_it_overflows() {
        let m = Metrics::desktop();
        assert!(m.scrollbar_gutter > 0.0);
        // Laid out twice: the first pass finds that the bar is needed.
        assert!((content_width(100.0) - 300.0).abs() < 0.5, "fits: no bar");
        let tall = content_width(2000.0);
        assert!(
            (tall - (300.0 - m.scrollbar_width - m.scrollbar_gutter)).abs() < 0.5,
            "content {tall} must stop before bar {} + gutter {}",
            m.scrollbar_width,
            m.scrollbar_gutter
        );
    }
}
