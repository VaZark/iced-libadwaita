//! Window setup for Adwaita apps: **client-side decorations only**.
//!
//! Adwaita dislikes server-side decorations (SSD): the app draws its own
//! [`HeaderBar`](crate::widget::HeaderBar), which carries the title, window
//! controls and acts as the drag handle. So SSD is off by default; use
//! [`settings`] for the iced window and [`frame`] to get rounded corners and
//! edge resize grips, which a decoration-less window otherwise lacks.
//!
//! See `docs/hig-inferences/design-language/windows-and-csd.md`.

use crate::widget::Element;
use crate::{Metrics, Theme};
use iced::widget::{column, container, mouse_area, row, space, stack};
use iced::window::{Direction, Mode, Settings};
use iced::{Length, Size, Subscription, Task, mouse};

/// `AdwWindow`'s default minimum size.
pub const MIN_SIZE: Size = Size::new(360.0, 200.0);

/// The smallest window in effect: the desktop config's `[windows] min-width` / `min-height`
/// ([`MIN_SIZE`] is the fallback).
pub fn min_size() -> Size {
    let c = crate::config::current();
    Size::new(c.min_window_width, c.min_window_height)
}

/// Window settings following the desktop config: client-side decorations unless the config
/// says `[windows] decorations=ssd`.
pub fn settings() -> Settings {
    if crate::config::current().decorations == crate::config::DecorationMode::Ssd {
        return settings_with_ssd(true);
    }
    Settings {
        decorations: false,
        transparent: true,
        min_size: Some(min_size()),
        ..Settings::default()
    }
}

/// Settings for a dialog in its own window (a preferences window, say): client-side
/// decorations like [`settings`], resizable, opened centered, and never smaller than a phone
/// width. A window the user can move and resize independently of its parent; the parent is
/// disabled while it is open by covering it with [`dialog_in_window`](crate::widget::dialog_in_window)
/// (an empty dialog is a scrim that swallows input).
pub fn dialog_settings(size: Size) -> Settings {
    Settings {
        size,
        position: iced::window::Position::Centered,
        min_size: Some(Size::new(min_size().width, 240.0)),
        ..settings()
    }
}

/// Like [`settings`], but lets the caller opt back into server-side
/// decorations (for compositors or platforms without client-side support).
pub fn settings_with_ssd(ssd: bool) -> Settings {
    if ssd {
        Settings {
            min_size: Some(min_size()),
            ..Settings::default()
        }
    } else {
        settings()
    }
}

/// At or below this width (just under [`breakpoints::SM`](crate::breakpoints::SM)) the window is
/// in *mobile mode*: window decorations are hidden completely and menus become full pages.
/// Corners are unaffected: use [`State`] (rounded unless actually maximized/fullscreen).
pub const MOBILE_MAX_WIDTH: f32 = crate::breakpoints::SM - 1.0;

/// The mobile breakpoint in effect: the desktop config's `[windows] mobile-max-width`
/// (default [`MOBILE_MAX_WIDTH`]).
pub fn mobile_max_width() -> f32 {
    crate::config::current().mobile_max_width
}

/// Whether a window of this width is in mobile mode.
pub fn is_mobile(width: f32) -> bool {
    width <= mobile_max_width()
}

const EDGE: f32 = 5.0;
const CORNER: f32 = 10.0;

/// What the window currently is, which decides how its frame is drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct State {
    pub maximized: bool,
    pub fullscreen: bool,
}

impl State {
    /// Maximized and fullscreen windows go edge to edge: square corners, no
    /// border, no resize grips (`--window-radius` only applies to floating windows).
    pub fn is_edge_to_edge(&self) -> bool {
        self.maximized || self.fullscreen
    }

    /// The corner radius to use for this state (0 when edge to edge).
    pub fn radius(&self, metrics: &Metrics) -> f32 {
        if self.is_edge_to_edge() {
            0.0
        } else {
            metrics.radius_window
        }
    }
}

/// Emits whenever the window is resized (which includes maximize/fullscreen
/// changes). Map it to a message that runs [`refresh`].
pub fn changes() -> Subscription<()> {
    iced::window::resize_events().map(|_| ())
}

/// Queries the oldest window's maximized/fullscreen state.
pub fn refresh() -> Task<State> {
    iced::window::oldest().and_then(|id| {
        iced::window::is_maximized(id).then(move |maximized| {
            iced::window::mode(id).map(move |mode| State {
                maximized,
                fullscreen: mode == Mode::Fullscreen,
            })
        })
    })
}

/// Wraps window content with the window background, rounded corners
/// (`--window-radius`, 15px) and invisible resize grips. When `state` is
/// maximized or fullscreen the frame goes edge to edge: square, borderless, no
/// grips. `on_resize` should return `iced::window::drag_resize(id, direction)`.
///
/// iced cannot clip to a rounded rectangle, so surfaces that touch the corners
/// must round themselves: pass [`State::radius`] to
/// [`HeaderBar::window_radius`](crate::widget::HeaderBar::window_radius) and
/// [`navigation_sidebar_rounded`](crate::widget::navigation_sidebar_rounded).
pub fn frame<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M>>,
    state: State,
    on_resize: impl Fn(Direction) -> M + 'a,
) -> Element<'a, M> {
    let radius = state.radius(&Metrics::default());
    let edge = state.is_edge_to_edge();
    let body = container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .clip(true)
        .style(move |t: &Theme| iced::widget::container::Style {
            border: iced::Border {
                radius: radius.into(),
                ..iced::Border::default()
            },
            ..crate::style::container::window(t)
        });
    if edge {
        return body.into();
    }
    // The 1px outline is drawn *above* the content so nothing paints over it at the
    // edges (the way libadwaita's window outline sits over the contents).
    let outline = container(space())
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |t: &Theme| iced::widget::container::Style {
            border: iced::Border {
                color: crate::color::alpha(t.semantic.window.fg, 0.12),
                width: 1.0,
                radius: radius.into(),
            },
            ..Default::default()
        });

    let grip =
        |dir: Direction, w: Length, h: Length, cursor: mouse::Interaction| -> Element<'a, M> {
            mouse_area(space().width(w).height(h))
                .on_press(on_resize(dir))
                .interaction(cursor)
                .into()
        };
    use mouse::Interaction::{
        ResizingDiagonallyDown as Dd, ResizingDiagonallyUp as Du, ResizingHorizontally as H,
        ResizingVertically as V,
    };
    let (e, c) = (Length::Fixed(EDGE), Length::Fixed(CORNER));
    let fill = Length::Fill;
    let grips = column![
        row![
            grip(Direction::NorthWest, c, c, Dd),
            grip(Direction::North, fill, e, V),
            grip(Direction::NorthEast, c, c, Du),
        ],
        row![
            column![grip(Direction::West, e, fill, H)].height(Length::Fill),
            space().width(Length::Fill),
            column![grip(Direction::East, e, fill, H)].height(Length::Fill),
        ]
        .height(Length::Fill),
        row![
            grip(Direction::SouthWest, c, c, Du),
            grip(Direction::South, fill, e, V),
            grip(Direction::SouthEast, c, c, Dd),
        ],
    ]
    .width(Length::Fill)
    .height(Length::Fill);

    stack![body, outline, grips].into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_windows_are_decorated_by_us_and_opened_centered() {
        let s = dialog_settings(Size::new(620.0, 640.0));
        assert!(!s.decorations && s.transparent && s.resizable);
        assert_eq!(s.size, Size::new(620.0, 640.0));
        assert!(matches!(s.position, iced::window::Position::Centered));
        assert!(s.min_size.is_some());
    }

    #[test]
    fn ssd_is_off_by_default() {
        let s = settings();
        assert!(!s.decorations);
        assert_eq!(s.min_size, Some(MIN_SIZE));
        assert!(settings_with_ssd(true).decorations);
        assert!(!settings_with_ssd(false).decorations);
    }

    #[test]
    fn mobile_threshold() {
        assert!(is_mobile(360.0) && is_mobile(575.0) && !is_mobile(576.0));
    }

    #[test]
    fn edge_to_edge_when_maximized_or_fullscreen() {
        let m = Metrics::default();
        assert_eq!(State::default().radius(&m), 15.0);
        assert_eq!(
            State {
                maximized: true,
                fullscreen: false
            }
            .radius(&m),
            0.0
        );
        assert_eq!(
            State {
                maximized: false,
                fullscreen: true
            }
            .radius(&m),
            0.0
        );
        assert!(!State::default().is_edge_to_edge());
    }
}
