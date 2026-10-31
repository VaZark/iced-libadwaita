//! `GtkScrolledWindow` with Adwaita's overlay scrollbars.
//!
//! iced's `scrollable` defaults to a 10px bar. Adwaita (and GTK4 with overlay
//! scrolling) uses a thin floating indicator that does not take layout space:
//! an 8px rail around a 4px slider (3px in Adwaita; iced cannot grow it on
//! hover), 2px from the edge. See `docs/hig-inferences/gtk4/scrollbar.md`.

use crate::{Metrics, Theme};
use iced::Element;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{Scrollable, scrollable};

/// The Adwaita scrollbar configuration (overlay, thin).
pub fn scrollbar() -> Scrollbar {
    let m = Metrics::desktop();
    Scrollbar::new()
        .width(m.scrollbar_width)
        .scroller_width(m.scrollbar_slider_width)
        .margin(m.scrollbar_margin)
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
