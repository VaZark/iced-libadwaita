//! Choice and display components: check and radio buttons, scale, link button, tags and color swatches.
//!
//! Each is a thin, themed wrapper with libadwaita-style properties, so a layout (code, XML or
//! JSX-like) can use one vocabulary instead of raw iced widgets.

use crate::Element;
use crate::styles;
use crate::{Metrics, Role, Surface, Swatch, Theme};
use iced::Length;
use iced::widget::{Button, Checkbox, Radio};
use iced::widget::{button, checkbox, container, radio, text};
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

/// `GtkScale`: a single-thumb slider using the shared oriented scrub track.
/// Horizontal by default; vertical values increase upward.
pub fn scale<'a, T, M>(
    range: RangeInclusive<T>,
    value: T,
    on_change: impl Fn(T) -> M + 'a,
) -> super::Scale<'a, T, M>
where
    T: crate::primitives::range::Scalar + 'a,
    M: 'a,
{
    super::Scale::new(range, value, on_change)
}

// --------------------------------------------------- LinkButton / card button

/// `GtkLinkButton`: accent-colored text button (`.link`).
pub fn link_button<'a, M: 'a>(label: impl Into<String>) -> Button<'a, M, Theme> {
    link_button_with_props(label, LinkButtonProps::default())
}

pub fn link_button_with_props<'a, M: 'a>(
    label: impl Into<String>,
    props: LinkButtonProps,
) -> Button<'a, M, Theme> {
    props.button_props.apply(
        button(
            props
                .label_props
                .apply(crate::components::typography::label(label.into())),
        )
        .padding(Metrics::desktop().space_xs)
        .style(styles::button::link),
    )
}

// -------------------------------------------------------------------- Tag

/// A small rounded label tinted with a [`Role`] (status chip). Not a libadwaita widget; the
/// pattern behind `.tag`-style badges in GNOME apps.
pub fn tag<'a, M: 'a>(label: impl Into<String>, role: Role) -> Element<'a, M> {
    tag_with_props(label, role, TagProps::default())
}

pub fn tag_with_props<'a, M: 'a>(
    label: impl Into<String>,
    role: Role,
    props: TagProps,
) -> Element<'a, M> {
    props
        .container_props
        .apply(
            container(
                props.label_props.apply(
                    text(label.into())
                        .size(crate::Typography::desktop().small)
                        .wrapping(text::Wrapping::None),
                ),
            )
            .padding(Metrics::desktop().small_button_padding)
            .style(styles::container::tinted(role)),
        )
        .into()
}

// ----------------------------------------------------------------- Swatches

/// A labeled color sample (`bg` with `fg` text), e.g. to document a theme's surfaces.
pub fn color_swatch<'a, M: 'a>(label: impl Into<String>, surface: Surface) -> Element<'a, M> {
    color_swatch_with_props(label, surface, ColorSwatchProps::default())
}

pub fn color_swatch_with_props<'a, M: 'a>(
    label: impl Into<String>,
    surface: Surface,
    props: ColorSwatchProps,
) -> Element<'a, M> {
    let m = Metrics::desktop();
    props
        .container_props
        .apply(
            container(
                props
                    .label_props
                    .apply(text(label.into()).size(12).color(surface.fg)),
            )
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
            }),
        )
        .into()
}

/// [`color_swatch`] for a status [`Swatch`] (`bg` / `fg`).
pub fn status_swatch<'a, M: 'a>(label: impl Into<String>, swatch: Swatch) -> Element<'a, M> {
    status_swatch_with_props(label, swatch, ColorSwatchProps::default())
}

#[derive(Clone, Default)]
pub struct LinkButtonProps {
    pub label_props: super::props::TextProps,
    pub button_props: super::props::ButtonProps,
}
impl LinkButtonProps {
    pub fn label_props(mut self, props: super::props::TextProps) -> Self {
        self.label_props = props;
        self
    }
    pub fn button_props(mut self, props: super::props::ButtonProps) -> Self {
        self.button_props = props;
        self
    }
}

#[derive(Clone, Default)]
pub struct TagProps {
    pub label_props: super::props::TextProps,
    pub container_props: super::props::ContainerProps,
}
impl TagProps {
    pub fn label_props(mut self, props: super::props::TextProps) -> Self {
        self.label_props = props;
        self
    }
    pub fn container_props(mut self, props: super::props::ContainerProps) -> Self {
        self.container_props = props;
        self
    }
}

#[derive(Clone, Default)]
pub struct ColorSwatchProps {
    pub label_props: super::props::TextProps,
    pub container_props: super::props::ContainerProps,
}
impl ColorSwatchProps {
    pub fn label_props(mut self, props: super::props::TextProps) -> Self {
        self.label_props = props;
        self
    }
    pub fn container_props(mut self, props: super::props::ContainerProps) -> Self {
        self.container_props = props;
        self
    }
}

pub fn status_swatch_with_props<'a, M: 'a>(
    label: impl Into<String>,
    swatch: Swatch,
    props: ColorSwatchProps,
) -> Element<'a, M> {
    color_swatch_with_props(label, swatch.surface(), props)
}
