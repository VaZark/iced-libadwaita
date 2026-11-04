//! Choice and display components: check and radio buttons, scale, link button, tags and color swatches.
//!
//! Each is a thin, themed wrapper with libadwaita-style properties, so a layout (code, XML or
//! JSX-like) can use one vocabulary instead of raw iced widgets.

use crate::Element;
use crate::styles;
use crate::{Metrics, Role, Surface, Swatch, Theme};
use iced::widget::{Button, Checkbox, Radio, Slider};
use iced::widget::{button, checkbox, container, radio, slider, text};
use iced::{Length, Padding};
use std::ops::RangeInclusive;

// ------------------------------------------------- CheckButton / RadioButton

/// `GtkCheckButton`: chain `.on_toggle(..)`; without it the button is disabled.
pub fn check_button<'a, M>(label: impl Into<String>, active: bool) -> Checkbox<'a, M, Theme> {
    checkbox(active).label(label.into())
}

/// `GtkCheckButton` in a group (radio): `value` is this button's value, `selected` the
/// group's current one.
pub fn radio_button<'a, V, M>(
    label: impl Into<String>,
    value: V,
    selected: Option<V>,
    on_select: impl FnOnce(V) -> M,
) -> Radio<'a, M, Theme>
where
    V: Copy + Eq,
    M: Clone,
{
    radio(label.into(), value, selected, on_select)
}

// ------------------------------------------------------------------ Scale

/// `GtkScale`: a horizontal slider filling the available width.
pub fn scale<'a, T, M>(
    range: RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> M + 'a,
) -> Slider<'a, T, M, Theme>
where
    T: Copy + From<u8> + PartialOrd + 'a,
    f64: From<T>,
    M: Clone + 'a,
{
    slider(range, value, on_change).width(Length::Fill)
}

// --------------------------------------------------- LinkButton / card button

/// `GtkLinkButton`: accent-colored text button (`.link`).
pub fn link_button<'a, M: 'a>(label: impl Into<String>) -> Button<'a, M, Theme> {
    button(crate::components::typography::label(label.into()))
        .padding(Metrics::desktop().space_xs)
        .style(styles::button::link)
}

// -------------------------------------------------------------------- Tag

/// A small rounded label tinted with a [`Role`] (status chip). Not a libadwaita widget; the
/// pattern behind `.tag`-style badges in GNOME apps.
pub fn tag<'a, M: 'a>(label: impl Into<String>, role: Role) -> Element<'a, M> {
    container(
        text(label.into())
            .size(Metrics::desktop().space_m + 1.0)
            .wrapping(text::Wrapping::None),
    )
    .padding(Padding::from([2.0, 10.0]))
    .style(styles::container::tinted(role))
    .into()
}

// ----------------------------------------------------------------- Swatches

/// A labeled color sample (`bg` with `fg` text), e.g. to document a theme's surfaces.
pub fn color_swatch<'a, M: 'a>(label: impl Into<String>, surface: Surface) -> Element<'a, M> {
    let m = Metrics::desktop();
    container(text(label.into()).size(12).color(surface.fg))
        .padding(m.space_m)
        .width(Length::Fill)
        .style(move |t: &Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(surface.bg)),
            border: iced::Border {
                color: crate::color::alpha(surface.fg, 0.2),
                width: 1.0,
                radius: t.metrics.radius_m.into(),
            },
            ..Default::default()
        })
        .into()
}

/// [`color_swatch`] for a status [`Swatch`] (`bg` / `fg`).
pub fn status_swatch<'a, M: 'a>(label: impl Into<String>, swatch: Swatch) -> Element<'a, M> {
    color_swatch(label, swatch.surface())
}
