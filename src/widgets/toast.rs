//! `AdwToast` and `AdwToastOverlay`.

use crate::Element;
use crate::components::{ButtonProps, ContainerProps, TextProps};
use crate::{Metrics, Role, Theme};
use iced::Length;
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, container, row, text};

/// The toast's dark surface in the light scheme; see [`Toast`].
const DARK_SURFACE: iced::Color = iced::Color {
    r: 0.14,
    g: 0.14,
    b: 0.15,
    a: 1.0,
};

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
    title_props: TextProps,
    action_label_props: TextProps,
    action_props: ButtonProps,
    dismiss_props: ButtonProps,
    status_props: ContainerProps,
    container_props: ContainerProps,
    title: String,
    timeout: Option<u32>,
    id: Option<u64>,
    action: Option<(String, M)>,
    on_dismiss: Option<M>,
    background: Option<iced::Color>,
    text_color: Option<iced::Color>,
    role: Option<Role>,
    status: Option<Role>,
    position: Option<ToastPosition>,
    radius: Option<f32>,
    spacing: Option<f32>,
    padding: Option<iced::Padding>,
    stack_spacing: Option<f32>,
    margin: Option<iced::Padding>,
}

impl<M> Toast<M> {
    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn title_props(mut self, props: TextProps) -> Self {
        self.title_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn action_label_props(mut self, props: TextProps) -> Self {
        self.action_label_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn action_props(mut self, props: ButtonProps) -> Self {
        self.action_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn dismiss_props(mut self, props: ButtonProps) -> Self {
        self.dismiss_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn status_props(mut self, props: ContainerProps) -> Self {
        self.status_props = props;
        self
    }

    /// Presentation applied after the nested widget defaults; content and events are retained.
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = props;
        self
    }

    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title_props: TextProps::default(),
            action_label_props: TextProps::default(),
            action_props: ButtonProps::default(),
            dismiss_props: ButtonProps::default(),
            status_props: ContainerProps::default(),
            container_props: ContainerProps::default(),

            title: title.into(),
            timeout: None,
            id: None,
            action: None,
            on_dismiss: None,
            background: None,
            text_color: None,
            role: None,
            status: None,
            position: None,
            radius: None,
            spacing: None,
            padding: None,
            stack_spacing: None,
            margin: None,
        }
    }
    /// Auto-dismiss delay in milliseconds, overriding `[toasts] timeout` (default 3000).
    /// Zero keeps the toast open until dismissed. Expiry emits `on_dismiss` once when set.
    pub fn timeout(mut self, milliseconds: u32) -> Self {
        self.timeout = Some(milliseconds);
        self
    }
    /// The corner radius of this toast. Without it the theme's `radius_toast` applies
    /// (a full pill by default).
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = Some(radius.max(0.0));
        self
    }
    /// The gap between the status dot, the title, the action and the dismiss button (default:
    /// the theme's `space_m`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
        self
    }
    /// The padding inside the toast (default: the theme's `space_s` by `space_l`).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }
    /// The gap between stacked toasts. Read from the first toast of a [`toast_stack`] (default:
    /// the theme's `space_s`).
    pub fn stack_spacing(mut self, px: f32) -> Self {
        self.stack_spacing = Some(px);
        self
    }
    /// The distance a stack keeps from its target's edges. Read from the first toast of a
    /// [`toast_stack`] (default: the theme's `space_xl` on every side).
    pub fn margin(mut self, p: impl Into<iced::Padding>) -> Self {
        self.margin = Some(p.into());
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
        iced::Color {
            a: 1.0,
            ..theme.semantic.popover.bg
        }
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
        .or_else(|| {
            role.map(|r| iced::Color {
                a: 1.0,
                ..theme.semantic.swatch(r).bg
            })
        })
        .unwrap_or_else(|| default_surface(theme));
    (
        bg,
        text_color.unwrap_or_else(|| crate::color::readable_on(bg)),
    )
}

/// One toast as an element: the surface, its content and its entrance.
fn toast_element<'a, M: Clone + 'a>(t: Toast<M>, position: ToastPosition) -> Element<'a, M> {
    let timeout = t
        .timeout
        .unwrap_or_else(|| crate::config::current().toast_timeout);
    let key = (t.title.clone(), timeout);
    let on_dismiss = t.on_dismiss.clone();
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
    let mut r = row![]
        .spacing(t.spacing.unwrap_or(m.space_m))
        .align_y(Vertical::Center);
    if let Some(status) = t.status.filter(|s| *s != Role::Neutral) {
        // The vivid fill of the status color reads on both the dark surfaces.
        let color = move |th: &Theme| th.semantic.swatch(status).bg;
        r = r.push(
            t.status_props.apply(
                container(
                    iced::widget::space()
                        .width(m.progress_height)
                        .height(m.progress_height),
                )
                .style(move |th: &Theme| iced::widget::container::Style {
                    background: Some(iced::Background::Color(color(th))),
                    border: iced::Border::default().rounded(th.metrics.radius_pill),
                    ..Default::default()
                }),
            ),
        );
    }
    r = r.push(
        t.title_props
            .apply(text(t.title).size(crate::Typography::desktop().body)),
    );
    if let Some((label, msg)) = t.action {
        r = r.push(
            t.action_props.apply(
                button(
                    t.action_label_props
                        .apply(text(label).size(crate::Typography::desktop().body)),
                )
                .padding(m.button_padding)
                .style(action_style)
                .on_press(msg),
            ),
        );
    }
    if let Some(msg) = t.on_dismiss {
        r =
            r.push(
                t.dismiss_props.apply(
                    button(crate::Icon::WindowClose.view(m.icon_size).style(
                        move |th: &Theme, _| iced::widget::svg::Style {
                            color: Some(colors(th, background, text_color, role).1),
                        },
                    ))
                    .padding(m.space_s)
                    .style(on_surface)
                    .on_press(msg),
                ),
            );
    }
    let toast = t.container_props.apply(
        container(r)
            .padding(
                t.padding
                    .unwrap_or(iced::Padding::from([m.space_s, m.space_l])),
            )
            .style(move |th: &Theme| {
                let (bg, fg) = colors(th, background, text_color, role);
                iced::widget::container::Style {
                    text_color: Some(fg),
                    background: Some(iced::Background::Color(bg)),
                    border: iced::Border::default()
                        .rounded(radius.unwrap_or(th.metrics.radius_toast)),
                    shadow: th.shadows.popover().shadow,
                    snap: false,
                }
            }),
    );
    // Slides up on show: 300ms, ADW_EASE (`SHOW_DURATION`). No hide animation: the app
    // removes the toast.
    // It slides in from the edge it is anchored to.
    let slide = if position.vertical() == Vertical::Top {
        -(m.row_min_height + m.space_xl)
    } else {
        m.row_min_height + m.space_xl
    };
    let toast = crate::layouts::entrance(toast, iced::Vector::new(0.0, slide))
        .trigger(crate::layouts::animation::Trigger::Toast)
        .spec(crate::layouts::animation::Spec::Timed(
            crate::layouts::animation::presets::TOAST_SHOW.0,
            crate::layouts::animation::presets::TOAST_SHOW.1,
        ));
    super::toast_timeout::timeout(toast, key, on_dismiss)
}

/// `AdwToastOverlay`: shows `toast` floating over `content`, at its position (default: the
/// desktop config's, bottom center). Toasts auto-dismiss after `[toasts] timeout` milliseconds
/// and emit `on_dismiss` so controlled application state can remove them.
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
        return crate::layouts::layered(content, None);
    };
    let m = Metrics::desktop();
    let stack_spacing = first.stack_spacing.unwrap_or(m.space_s);
    let margin = first.margin.unwrap_or(iced::Padding::from(m.space_xl));
    let position = first
        .position
        .unwrap_or_else(|| crate::config::current().toast_position);
    let stack = iced::widget::keyed_column(
        toasts
            .into_iter()
            .enumerate()
            .map(|(index, t)| (t.id.unwrap_or(index as u64), toast_element(t, position))),
    )
    .spacing(stack_spacing)
    .align_items(position.horizontal().into());
    crate::layouts::layered(
        content,
        Some(
            container(stack)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(position.horizontal())
                .align_y(position.vertical())
                .padding(margin)
                .into(),
        ),
    )
}

/// [`toast_stack`] over a [`notifications::Queue`](crate::primitives::notifications::Queue):
/// `toast` turns each live item (with its id, for the dismiss and action messages) into a
/// [`Toast`]. The queue owns the limit and ids; toast expiry emits each dismiss message.
pub fn toast_stack_from<'a, M: Clone + 'a, T>(
    content: impl Into<Element<'a, M>>,
    queue: &crate::primitives::notifications::Queue<T>,
    toast: impl Fn(crate::primitives::notifications::Id, &T) -> Toast<M>,
) -> Element<'a, M> {
    toast_stack(
        content,
        queue.iter().map(|(id, t)| {
            let mut toast = toast(id, t);
            toast.id = Some(id as u64);
            toast
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::{DARK_TEXT, LIGHT_TEXT, contrast_ratio};

    #[test]
    fn the_gaps_padding_and_margin_are_parameters() {
        let t = || {
            Toast::new("Saved")
                .status(Role::Success)
                .action("Undo", 1u8)
                .spacing(2.0)
                .padding([3.0, 4.0])
                .stack_spacing(5.0)
                .margin(6.0)
        };
        assert_eq!((t().spacing, t().stack_spacing), (Some(2.0), Some(5.0)));
        let _: Element<'_, u8> = toast_stack(iced::widget::space(), [t(), t()]);
    }

    #[test]
    fn surface_is_dark_and_opaque_in_both_schemes() {
        let (bg, fg) = colors(&Theme::light(), None, None, None);
        assert_eq!(bg, DARK_SURFACE);
        assert_eq!(fg, LIGHT_TEXT);
        let dark = Theme::dark();
        let (bg, fg) = colors(&dark, None, None, None);
        assert_eq!(
            bg,
            iced::Color {
                a: 1.0,
                ..dark.semantic.popover.bg
            },
            "the theme's own elevated surface"
        );
        assert!(
            crate::color::relative_luminance(bg) < 0.1,
            "never a light slab on a dark window"
        );
        assert_eq!(fg, LIGHT_TEXT);
        assert!(contrast_ratio(bg, fg) > 7.0);
        assert_eq!(bg.a, 1.0);
    }

    #[test]
    fn role_and_hard_overrides() {
        let t = Theme::light();
        let (bg, fg) = colors(&t, None, None, Some(Role::Error));
        assert_eq!(
            bg,
            iced::Color {
                a: 1.0,
                ..t.semantic.error.bg
            }
        );
        assert_eq!(fg, crate::color::readable_on(bg));
        let custom = iced::Color::from_rgb(0.9, 0.9, 0.1);
        let (bg, fg) = colors(&t, Some(custom), None, Some(Role::Error));
        assert_eq!(
            (bg, fg),
            (custom, DARK_TEXT),
            "background wins over role; text follows contrast"
        );
        let pink = iced::Color::from_rgb(1.0, 0.0, 0.5);
        assert_eq!(
            colors(&t, Some(custom), Some(pink), None).1,
            pink,
            "text_color is a hard override"
        );
    }
}

/// Reusable, no-op-by-default presentation for [`Toast`].
#[derive(Clone, Default)]
pub struct ToastProps {
    pub title_props: Option<TextProps>,
    pub action_label_props: Option<TextProps>,
    pub action_props: Option<ButtonProps>,
    pub dismiss_props: Option<ButtonProps>,
    pub status_props: Option<ContainerProps>,
    pub container_props: Option<ContainerProps>,
}
impl ToastProps {
    pub fn title_props(mut self, props: TextProps) -> Self {
        self.title_props = Some(props);
        self
    }
    pub fn action_label_props(mut self, props: TextProps) -> Self {
        self.action_label_props = Some(props);
        self
    }
    pub fn action_props(mut self, props: ButtonProps) -> Self {
        self.action_props = Some(props);
        self
    }
    pub fn dismiss_props(mut self, props: ButtonProps) -> Self {
        self.dismiss_props = Some(props);
        self
    }
    pub fn status_props(mut self, props: ContainerProps) -> Self {
        self.status_props = Some(props);
        self
    }
    pub fn container_props(mut self, props: ContainerProps) -> Self {
        self.container_props = Some(props);
        self
    }
    pub fn apply<M>(&self, widget: Toast<M>) -> Toast<M> {
        let widget = if let Some(props) = &self.title_props {
            widget.title_props(props.clone())
        } else {
            widget
        };
        let widget = if let Some(props) = &self.action_label_props {
            widget.action_label_props(props.clone())
        } else {
            widget
        };
        let widget = if let Some(props) = &self.action_props {
            widget.action_props(props.clone())
        } else {
            widget
        };
        let widget = if let Some(props) = &self.dismiss_props {
            widget.dismiss_props(props.clone())
        } else {
            widget
        };
        let widget = if let Some(props) = &self.status_props {
            widget.status_props(props.clone())
        } else {
            widget
        };
        let widget = if let Some(props) = &self.container_props {
            widget.container_props(props.clone())
        } else {
            widget
        };
        widget
    }
}

#[cfg(test)]
mod typed_presentation_tests {
    use super::*;
    use iced::advanced::widget::Tree;
    fn same_tree(a: &Tree, b: &Tree) {
        assert!(a.tag == b.tag);
        assert_eq!(a.children.len(), b.children.len());
        for (a, b) in a.children.iter().zip(&b.children) {
            same_tree(a, b);
        }
    }
    #[test]
    fn reusable_defaults_preserve_geometry_and_tree() {
        let build = || {
            Toast::new("Title")
                .action("Undo", 7u8)
                .on_dismiss(8)
                .timeout(0)
        };
        let w = build();
        let baseline: Element<'_, u8> = toast_element(w, ToastPosition::BottomCenter);
        let w = ToastProps::default().apply(build());
        let configured: Element<'_, u8> = toast_element(w, ToastPosition::BottomCenter);
        assert_eq!(baseline.as_widget().size(), configured.as_widget().size());
        same_tree(&Tree::new(&baseline), &Tree::new(&configured));
    }
}
