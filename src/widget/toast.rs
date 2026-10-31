//! `AdwToast` and `AdwToastOverlay`.

use super::Element;
use crate::{Metrics, Role, Theme};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, container, row, text};
use iced::Length;

/// The toast's dark surface in the light scheme; see [`Toast`].
const DARK_SURFACE: iced::Color = iced::Color { r: 0.14, g: 0.14, b: 0.15, a: 1.0 };

/// Where a toast sits inside its target ([`ToastTarget`]): top or bottom, and left, center or
/// right. The default comes from the desktop config (`[toasts] position`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToastPosition {
    TopLeft,
    TopCenter,
    TopRight,
    BottomLeft,
    #[default]
    BottomCenter,
    BottomRight,
}

impl ToastPosition {
    pub const ALL: [ToastPosition; 6] = [
        ToastPosition::TopLeft,
        ToastPosition::TopCenter,
        ToastPosition::TopRight,
        ToastPosition::BottomLeft,
        ToastPosition::BottomCenter,
        ToastPosition::BottomRight,
    ];

    pub fn vertical(self) -> Vertical {
        match self {
            Self::TopLeft | Self::TopCenter | Self::TopRight => Vertical::Top,
            _ => Vertical::Bottom,
        }
    }

    pub fn horizontal(self) -> Horizontal {
        match self {
            Self::TopLeft | Self::BottomLeft => Horizontal::Left,
            Self::TopCenter | Self::BottomCenter => Horizontal::Center,
            _ => Horizontal::Right,
        }
    }

    /// The `[toasts] position` config value.
    pub fn key(self) -> &'static str {
        match self {
            Self::TopLeft => "top-left",
            Self::TopCenter => "top-center",
            Self::TopRight => "top-right",
            Self::BottomLeft => "bottom-left",
            Self::BottomCenter => "bottom-center",
            Self::BottomRight => "bottom-right",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|p| p.key() == key)
    }
}

impl std::fmt::Display for ToastPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::TopLeft => "Top left",
            Self::TopCenter => "Top center",
            Self::TopRight => "Top right",
            Self::BottomLeft => "Bottom left",
            Self::BottomCenter => "Bottom center",
            Self::BottomRight => "Bottom right",
        })
    }
}

/// The container a toast is placed in: the **whole window** (over the sidebar too) or only the
/// **content area** (beside the sidebar). The widget fills whatever it wraps, so the app chooses
/// by wrapping the window or the content pane with [`toast_overlay`]; this is the shared setting
/// to read ([`toast_target`], `[toasts] target` in the desktop config) so every app agrees.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ToastTarget {
    Window,
    #[default]
    Content,
}

impl ToastTarget {
    pub const ALL: [ToastTarget; 2] = [ToastTarget::Window, ToastTarget::Content];

    /// The `[toasts] target` config value.
    pub fn key(self) -> &'static str {
        match self {
            Self::Window => "window",
            Self::Content => "content",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.key() == key)
    }
}

impl std::fmt::Display for ToastTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Window => "Window",
            Self::Content => "Content area",
        })
    }
}

/// The target in effect: the desktop config's `[toasts] target`.
pub fn toast_target() -> ToastTarget {
    crate::config::current().toast_target
}

/// `AdwToast`.
///
/// The toast is a solid, always-dark surface (no opacity): the theme's elevated surface in the
/// dark scheme and a dark ink tone in the light scheme, never a light slab on a dark window.
/// The text and icons are chosen by contrast with it, like CSS `color-contrast()`. Three ways
/// to change it, narrowest last:
///
/// * [`role`](Toast::role): a third option, a status fill (accent, error, …) with contrast text;
/// * [`background`](Toast::background): a hard-coded surface (text still chosen by contrast);
/// * [`text_color`](Toast::text_color): a hard-coded text color.
pub struct Toast<M> {
    title: String,
    action: Option<(String, M)>,
    on_dismiss: Option<M>,
    background: Option<iced::Color>,
    text_color: Option<iced::Color>,
    role: Option<Role>,
    status: Option<Role>,
    position: Option<ToastPosition>,
    radius: Option<f32>,
}

impl<M> Toast<M> {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            action: None,
            on_dismiss: None,
            background: None,
            text_color: None,
            role: None,
            status: None,
            position: None,
            radius: None,
        }
    }
    /// The corner radius of this toast. Without it the theme's `radius_toast` applies
    /// (a full pill by default).
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius.max(0.0));
        self
    }
    /// Where the toast sits in its target. Without this it follows the desktop config
    /// (`[toasts] position`, default bottom center).
    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = Some(position);
        self
    }
    /// The type of message (`Success`, `Warning`, `Error`, `Accent`, `Destructive`): shown as a
    /// colored dot before the title. The surface stays neutral; see [`role`](Toast::role) to fill
    /// it instead. `Role::Neutral` shows nothing.
    pub fn status(mut self, status: Role) -> Self {
        self.status = Some(status);
        self
    }
    /// Use a status fill ([`Role::Accent`], `Error`, …) instead of the inverse surface. The
    /// text color is still chosen by contrast with it.
    pub fn role(mut self, role: Role) -> Self {
        self.role = Some(role);
        self
    }
    /// Hard override for the surface color (opaque; wins over [`role`](Toast::role)).
    pub fn background(mut self, c: iced::Color) -> Self {
        self.background = Some(iced::Color { a: 1.0, ..c });
        self
    }
    /// Hard override for the text, action and close-icon color.
    pub fn text_color(mut self, c: iced::Color) -> Self {
        self.text_color = Some(c);
        self
    }
    /// `button-label` + `action-name`.
    pub fn action(mut self, label: impl Into<String>, on_press: M) -> Self {
        self.action = Some((label.into(), on_press));
        self
    }
    /// Message to send when the user dismisses the toast (adds a close button).
    pub fn on_dismiss(mut self, m: M) -> Self {
        self.on_dismiss = Some(m);
        self
    }
}

/// The toast surface is always dark, never a light slab on a dark window: the theme's own
/// elevated surface in the dark scheme (`popover`), the dark ink tone in the light scheme.
fn default_surface(theme: &Theme) -> iced::Color {
    if theme.semantic.is_dark() {
        iced::Color { a: 1.0, ..theme.semantic.popover.bg }
    } else {
        DARK_SURFACE
    }
}

/// The surface and text colors of a toast: `(background, text)`.
fn colors(
    theme: &Theme,
    background: Option<iced::Color>,
    text_color: Option<iced::Color>,
    role: Option<Role>,
) -> (iced::Color, iced::Color) {
    let bg = background
        .or_else(|| role.map(|r| iced::Color { a: 1.0, ..theme.semantic.swatch(r).bg }))
        .unwrap_or_else(|| default_surface(theme));
    (bg, text_color.unwrap_or_else(|| crate::color::readable_on(bg)))
}

/// One toast as an element: the surface, its content and its entrance.
fn toast_element<'a, M: Clone + 'a>(t: Toast<M>, position: ToastPosition) -> Element<'a, M> {
    let m = Metrics::desktop();
    let (background, text_color, role) = (t.background, t.text_color, t.role);
    let radius = t.radius;
    // Buttons on the toast: the text color, with a state layer of it over the surface.
    let on_surface = move |th: &Theme, status: button::Status| {
        let (bg, fg) = colors(th, background, text_color, role);
        let layer = match status {
            button::Status::Hovered => 0.15,
            button::Status::Pressed => 0.25,
            _ => 0.0,
        };
        button::Style {
            background: Some(iced::Background::Color(crate::color::mix(bg, fg, layer))),
            text_color: fg,
            border: iced::Border::default().rounded(th.metrics.radius_pill),
            ..Default::default()
        }
    };
    // The call to action is an accent-filled button so it reads as one (text by contrast). On a
    // toast that is itself filled with the accent it falls back to the text-colored tint.
    let action_style = move |th: &Theme, status: button::Status| {
        if role == Some(Role::Accent) {
            return on_surface(th, status);
        }
        let a = th.semantic.accent;
        let bg = match status {
            button::Status::Hovered => th.semantic.hover_fill(a.bg),
            button::Status::Pressed => th.semantic.active_fill(a.bg),
            _ => a.bg,
        };
        button::Style {
            background: Some(iced::Background::Color(bg)),
            text_color: a.fg,
            border: iced::Border::default().rounded(th.metrics.radius_pill),
            ..Default::default()
        }
    };
    let mut r = row![].spacing(m.space_m).align_y(Vertical::Center);
    if let Some(status) = t.status.filter(|s| *s != Role::Neutral) {
        // The vivid fill of the status color reads on both the dark surfaces.
        let color = move |th: &Theme| th.semantic.swatch(status).bg;
        r = r.push(container(iced::widget::space().width(10).height(10)).style(move |th: &Theme| {
            iced::widget::container::Style {
                background: Some(iced::Background::Color(color(th))),
                border: iced::Border::default().rounded(5.0),
                ..Default::default()
            }
        }));
    }
    r = r.push(text(t.title));
    if let Some((label, msg)) = t.action {
        r = r.push(button(text(label)).padding(m.button_padding).style(action_style).on_press(msg));
    }
    if let Some(msg) = t.on_dismiss {
        r = r.push(
            button(
                crate::Icon::WindowClose
                    .view(m.icon_size)
                    .style(move |th: &Theme, _| iced::widget::svg::Style {
                        color: Some(colors(th, background, text_color, role).1),
                    }),
            )
            .padding(m.space_s)
            .style(on_surface)
            .on_press(msg),
        );
    }
    let toast = container(r).padding([m.space_s, m.space_l]).style(move |th: &Theme| {
        let (bg, fg) = colors(th, background, text_color, role);
        iced::widget::container::Style {
            text_color: Some(fg),
            background: Some(iced::Background::Color(bg)),
            border: iced::Border::default().rounded(radius.unwrap_or(th.metrics.radius_toast)),
            shadow: th.shadows.popover().shadow,
            snap: false,
        }
    });
    // Slides up on show: 300ms, ADW_EASE (`SHOW_DURATION`). No hide animation: the app
    // removes the toast.
    // It slides in from the edge it is anchored to.
    let slide = if position.vertical() == Vertical::Top { -60.0 } else { 60.0 };
    let toast = super::entrance(toast, iced::Vector::new(0.0, slide))
        .trigger(crate::animation::Trigger::Toast)
        .spec(crate::animation::Spec::Timed(
            crate::animation::presets::TOAST_SHOW.0,
            crate::animation::presets::TOAST_SHOW.1,
        ));
    toast.into()
}

/// `AdwToastOverlay`: shows `toast` floating over `content`, at its position (default: the
/// desktop config's, bottom center). The app is responsible for clearing it (e.g. after a
/// timeout).
pub fn toast_overlay<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M>>,
    toast: Option<Toast<M>>,
) -> Element<'a, M> {
    toast_stack(content, toast)
}

/// **Stacked notifications**: any number of toasts shown at once, in one column at the position
/// of the first (`[toasts] position` by default), `toasts` ordered oldest to newest with a small
/// gap between them. At a bottom position the newest sits nearest the edge, and each toast has
/// its own entrance, call to action and dismiss. The app owns the list (add, dismiss, cap its
/// length); an empty iterator shows nothing and keeps `content`'s place in the tree.
pub fn toast_stack<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M>>,
    toasts: impl IntoIterator<Item = Toast<M>>,
) -> Element<'a, M> {
    let toasts: Vec<Toast<M>> = toasts.into_iter().collect();
    let Some(first) = toasts.first() else {
        return super::layered(content, None);
    };
    let m = Metrics::desktop();
    let position = first.position.unwrap_or_else(|| crate::config::current().toast_position);
    let stack = iced::widget::column(toasts.into_iter().map(|t| toast_element(t, position)))
        .spacing(m.space_s)
        .align_x(position.horizontal());
    super::layered(
        content,
        Some(
            container(stack)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(position.horizontal())
                .align_y(position.vertical())
                .padding(m.space_xl)
                .into(),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{contrast_ratio, DARK_TEXT, LIGHT_TEXT};

    #[test]
    fn surface_is_dark_and_opaque_in_both_schemes() {
        let (bg, fg) = colors(&Theme::light(), None, None, None);
        assert_eq!(bg, DARK_SURFACE);
        assert_eq!(fg, LIGHT_TEXT);
        let dark = Theme::dark();
        let (bg, fg) = colors(&dark, None, None, None);
        assert_eq!(bg, iced::Color { a: 1.0, ..dark.semantic.popover.bg }, "the theme's own elevated surface");
        assert!(crate::color::relative_luminance(bg) < 0.1, "never a light slab on a dark window");
        assert_eq!(fg, LIGHT_TEXT);
        assert!(contrast_ratio(bg, fg) > 7.0);
        assert_eq!(bg.a, 1.0);
    }

    #[test]
    fn role_and_hard_overrides() {
        let t = Theme::light();
        let (bg, fg) = colors(&t, None, None, Some(Role::Error));
        assert_eq!(bg, iced::Color { a: 1.0, ..t.semantic.error.bg });
        assert_eq!(fg, crate::color::readable_on(bg));
        let custom = iced::Color::from_rgb(0.9, 0.9, 0.1);
        let (bg, fg) = colors(&t, Some(custom), None, Some(Role::Error));
        assert_eq!((bg, fg), (custom, DARK_TEXT), "background wins over role; text follows contrast");
        let pink = iced::Color::from_rgb(1.0, 0.0, 0.5);
        assert_eq!(colors(&t, Some(custom), Some(pink), None).1, pink, "text_color is a hard override");
    }
}
