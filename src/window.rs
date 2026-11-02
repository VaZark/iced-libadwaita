//! Window setup for Adwaita apps: **client-side decorations only**.
//!
//! Adwaita dislikes server-side decorations (SSD): the app draws its own
//! [`HeaderBar`](crate::widget::HeaderBar), which carries the title, window
//! controls and acts as the drag handle. So SSD is off by default (the toolkit config, `[windows]
//! decorations`, decides: see [`csd`]); use
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

/// Whether windows draw their own decorations: the toolkit's `[windows] decorations` setting
/// (client-side unless the desktop config says `ssd`). Apps follow it as it is; it is not a
/// per-app switch.
pub fn csd() -> bool {
    crate::config::current().decorations == crate::config::DecorationMode::Csd
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

// ---------------------------------------------------------------------------
// Client-side window chrome, and windows of their own
// ---------------------------------------------------------------------------

/// What the chrome of a window (its header bars and frame) asks of the window. A widget cannot
/// act on its window, so it sends one of these as a message and the app performs it with
/// [`Action::perform`].
#[derive(Debug, Clone, Copy)]
pub enum Action {
    /// Press on the empty header area: move the window.
    Drag,
    Minimize,
    ToggleMaximize,
    Close,
    /// Press on a resize grip of the [`frame`].
    Resize(Direction),
}

impl Action {
    /// The task that does it to window `id`.
    pub fn perform<T>(self, id: iced::window::Id) -> Task<T> {
        match self {
            Self::Drag => iced::window::drag(id),
            Self::Minimize => iced::window::minimize(id, true),
            Self::ToggleMaximize => iced::window::toggle_maximize(id),
            Self::Close => iced::window::close(id),
            Self::Resize(d) => iced::window::drag_resize(id, d),
        }
    }
}

/// The client-side decorations of one window, as the toolkit config says
/// ([`csd`]): which of its edges a header bar or surface touches, which buttons it has, and
/// whether the window is maximized. One value describes the window; ask it for the
/// [`frame`] and give it to each header bar with
/// [`HeaderBar::chrome`](crate::widget::HeaderBar::chrome).
///
/// Without client-side decorations the chrome draws nothing of its own: no buttons, drag area
/// or rounded corners, and [`Chrome::frame`] is just the window background.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Chrome {
    pub(crate) csd: bool,
    pub(crate) state: State,
    pub(crate) left: bool,
    pub(crate) right: bool,
    pub(crate) minimize: bool,
    pub(crate) maximize: bool,
}

impl Chrome {
    /// The chrome of a window in `state`, with every button, touching both side edges.
    pub fn new(state: State) -> Self {
        Self {
            csd: csd(),
            state,
            left: true,
            right: true,
            minimize: true,
            maximize: true,
        }
    }

    /// Overrides the toolkit's decoration mode (for tests; apps follow the config).
    pub fn csd_override(mut self, csd: bool) -> Self {
        self.csd = csd;
        self
    }

    /// Which side edges of the window the header bar (or surface) touches: it shows the window
    /// buttons the user's layout puts on those sides, and rounds those corners. In a split view
    /// each pane's header gets its own.
    pub fn edges(mut self, left: bool, right: bool) -> Self {
        self.left = left;
        self.right = right;
        self
    }

    /// Only a close button, no minimize or maximize: a dialog.
    pub fn close_only(mut self) -> Self {
        self.minimize = false;
        self.maximize = false;
        self
    }

    /// The window corner radius: none without client-side decorations, and none edge to edge.
    pub fn radius(&self) -> f32 {
        if self.csd {
            self.state.radius(&Metrics::default())
        } else {
            0.0
        }
    }

    /// The window around `content`: the [`frame`] (rounded, outlined, with resize grips) with
    /// client-side decorations, plain window background without. Grips send
    /// [`Action::Resize`] through `on_action`.
    pub fn frame<'a, M: Clone + 'a>(
        &self,
        content: impl Into<Element<'a, M>>,
        on_action: impl Fn(Action) -> M + 'a,
    ) -> Element<'a, M> {
        if self.csd {
            frame(content, self.state, move |d| on_action(Action::Resize(d)))
        } else {
            crate::widget::window(content)
        }
    }
}

/// A window the app opens besides its main one: a dialog in a window of its own (a
/// preferences window) or a whole second shell. It owns the window's id and its state, opens
/// with the toolkit's decorations and can disable its parent the way a regular dialog does.
///
/// Keep it in an `Option` in your state (`None` while closed), route
/// [`Action`]s from its chrome to [`Action::perform`] with its id, and clear it when iced
/// reports the window closed.
#[derive(Debug)]
pub struct Child<T> {
    id: iced::window::Id,
    modal: bool,
    /// Whatever the window needs (`()` for a static dialog).
    pub state: T,
}

impl<T> Child<T> {
    /// Opens a **dialog** window of `size`: centered, never narrower than a phone, and modal:
    /// while it is open the parent is disabled (see [`Child::disable_parent`]).
    pub fn dialog(size: Size, state: T) -> (Self, Task<iced::window::Id>) {
        Self::open(dialog_settings(size), true, state)
    }

    /// Opens an ordinary second **window** of `size` that does not disable its parent.
    pub fn window(size: Size, state: T) -> (Self, Task<iced::window::Id>) {
        Self::open(Settings { size, ..settings() }, false, state)
    }

    fn open(settings: Settings, modal: bool, state: T) -> (Self, Task<iced::window::Id>) {
        let (id, open) = iced::window::open(settings);
        (Self { id, modal, state }, open)
    }

    pub fn id(&self) -> iced::window::Id {
        self.id
    }

    /// Whether `id` is this window.
    pub fn is(&self, id: iced::window::Id) -> bool {
        self.id == id
    }

    pub fn is_modal(&self) -> bool {
        self.modal
    }

    /// Brings the window forward (what pressing the disabled parent does).
    pub fn focus<M>(&self) -> Task<M> {
        iced::window::gain_focus(self.id)
    }

    /// Closes the window. Clear the `Option` when the close event arrives.
    pub fn close<M>(&self) -> Task<M> {
        iced::window::close(self.id)
    }

    /// Disables `parent` while a **modal** child is open, like a regular dialog: a scrim over
    /// it swallows input, and pressing it sends `on_press` (use it to [`focus`](Self::focus)
    /// the child). Both windows can still be moved on the screen independently. With no child
    /// (or a non-modal one) the parent is returned in the same tree position, so opening or
    /// closing the child does not disturb it. `radius` rounds the scrim to the parent's
    /// corners ([`Chrome::radius`]).
    pub fn disable_parent<'a, M: Clone + 'a>(
        child: Option<&Self>,
        parent: impl Into<Element<'a, M>>,
        on_press: M,
        radius: f32,
    ) -> Element<'a, M> {
        crate::widget::present_dialog(
            parent,
            child.filter(|c| c.modal).map(|_| Element::from(space())),
            Some(on_press),
            radius,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chrome_rounds_only_floating_client_side_windows() {
        let floating = Chrome::new(State::default()).csd_override(true);
        assert_eq!(floating.radius(), Metrics::default().radius_window);
        let maximized = Chrome::new(State {
            maximized: true,
            fullscreen: false,
        })
        .csd_override(true);
        assert_eq!(maximized.radius(), 0.0);
        assert_eq!(floating.csd_override(false).radius(), 0.0);
    }

    #[test]
    fn chrome_builders_set_edges_and_buttons() {
        let c: Chrome = Chrome::new(State::default())
            .close_only()
            .edges(true, false);
        assert!(!c.minimize && !c.maximize && c.left && !c.right);
    }

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
