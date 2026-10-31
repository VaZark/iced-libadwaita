//! `AdwToast` and `AdwToastOverlay`.

use super::Element;
use crate::{Metrics, Role, Theme};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, container, row, text};
use iced::Length;

/// The toast's dark surface in the light scheme; see [`Toast`].
const DARK_SURFACE: iced::Color = iced::Color { r: 0.14, g: 0.14, b: 0.15, a: 1.0 };

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
        }
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

/// `AdwToastOverlay`: shows `toast` floating at the bottom of `content`.
/// The app is responsible for clearing it (e.g. after a timeout).
pub fn toast_overlay<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M>>,
    toast: Option<Toast<M>>,
) -> Element<'a, M> {
    let Some(t) = toast else {
        return super::layered(content, None);
    };
    let m = Metrics::desktop();
    let (background, text_color, role) = (t.background, t.text_color, t.role);
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
    let mut r = row![text(t.title)].spacing(m.space_m).align_y(Vertical::Center);
    if let Some((label, msg)) = t.action {
        r = r.push(button(text(label)).padding(m.button_padding).style(on_surface).on_press(msg));
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
            border: iced::Border::default().rounded(th.metrics.radius_pill),
            shadow: th.shadows.popover().shadow,
            snap: false,
        }
    });
    // Slides up on show: 300ms, ADW_EASE (`SHOW_DURATION`). No hide animation: the app
    // removes the toast.
    let toast = super::entrance(toast, iced::Vector::new(0.0, 60.0))
        .trigger(crate::animation::Trigger::Toast)
        .spec(crate::animation::Spec::Timed(
            crate::animation::presets::TOAST_SHOW.0,
            crate::animation::presets::TOAST_SHOW.1,
        ));
    super::layered(
        content,
        Some(
            container(toast)
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(Horizontal::Center)
                .align_y(Vertical::Bottom)
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
