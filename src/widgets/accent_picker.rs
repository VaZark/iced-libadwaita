//! `AccentPicker`: the accent color chooser.

use crate::Element;
use crate::primitives::color_picker::{Format as ColorFormat, Hsva};
use crate::widgets::ColorInput;
use crate::{AccentColor, Icon, Metrics, Theme};
use iced::Length;
use iced::widget::{button, container, row};

/// The accent color chooser: a row of round swatches, one per desktop accent, with a check on
/// the selected one. With [`custom`](AccentPicker::custom) a last swatch opens a
/// [`ColorPicker`](crate::widgets::ColorPicker) (with a text input) in a popover for any other color, and
/// the choice is an [`AccentColor::Custom`].
/// The text the custom accent's entry shows and what typing sends.
type Custom<'a, M> = (&'a str, std::rc::Rc<dyn Fn(String) -> M + 'a>);

pub struct AccentPicker<'a, M> {
    button_props: crate::components::ButtonProps,
    entry_props: crate::components::EntryProps,
    selected: AccentColor,
    on_select: std::rc::Rc<dyn Fn(AccentColor) -> M + 'a>,
    custom: Option<Custom<'a, M>>,
    dialog: bool,
    scrim: bool,
    close_on_outside: bool,
    spacing: Option<f32>,
    metrics: Metrics,
}

pub fn accent_picker<'a, M: Clone + 'a>(
    selected: AccentColor,
    on_select: impl Fn(AccentColor) -> M + 'a,
) -> AccentPicker<'a, M> {
    AccentPicker {
        button_props: Default::default(),
        entry_props: Default::default(),
        selected,
        on_select: std::rc::Rc::new(on_select),
        custom: None,
        dialog: false,
        scrim: true,
        close_on_outside: true,
        spacing: None,
        metrics: Metrics::desktop(),
    }
}

impl<'a, M: Clone + 'a> AccentPicker<'a, M> {
    pub fn button_props(mut self, props: crate::components::ButtonProps) -> Self {
        self.button_props = props;
        self
    }
    pub fn entry_props(mut self, props: crate::components::EntryProps) -> Self {
        self.entry_props = props;
        self
    }

    /// Adds the custom swatch. `text` is what the popover's entry shows (the app keeps it, so
    /// typing is not interrupted); `on_text` gets what was typed. Parse it with
    /// [`AccentColor::parse`] and select the result.
    pub fn custom(mut self, text: &'a str, on_text: impl Fn(String) -> M + 'a) -> Self {
        self.custom = Some((text, std::rc::Rc::new(on_text)));
        self
    }
    /// The gap between swatches (default `space_s`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
        self
    }
    /// Opens the color picker as a dialog centered in the window (default: a popover).
    pub fn dialog(mut self, on: bool) -> Self {
        self.dialog = on;
        self
    }
    /// Fades the window behind the dialog (default on).
    /// Whether a press outside the dialog closes it (default true).
    pub fn close_on_outside(mut self, on: bool) -> Self {
        self.close_on_outside = on;
        self
    }
    pub fn scrim(mut self, on: bool) -> Self {
        self.scrim = on;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

/// A round swatch of `color`, with a check when `on`.
fn round_swatch<'a, M: 'a>(color: iced::Color, on: bool, m: Metrics) -> Element<'a, M> {
    let content: Element<'a, M> = if on {
        Icon::Check
            .colored(m.icon_size, crate::color::readable_on(color))
            .into()
    } else {
        iced::widget::space().into()
    };
    container(content)
        .center(Length::Fixed(m.button_min_height))
        .style(move |t: &Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border::default().rounded(t.metrics.radius_pill),
            ..Default::default()
        })
        .into()
}

impl<'a, M: Clone + 'a> From<AccentPicker<'a, M>> for Element<'a, M> {
    fn from(p: AccentPicker<'a, M>) -> Self {
        let m = p.metrics;
        let mut swatches: Vec<Element<'a, M>> = AccentColor::ALL
            .iter()
            .map(|a| {
                p.button_props
                    .apply(
                        button(round_swatch(a.to_color(), *a == p.selected, m))
                            .padding(0)
                            .style(|_: &Theme, _| iced::widget::button::Style::default())
                            .on_press((p.on_select)(*a)),
                    )
                    .into()
            })
            .collect();
        if let Some((text, on_text)) = p.custom {
            let current = p.selected.to_color();
            let c = Hsva::from_rgba(current.r, current.g, current.b, 1.0);
            let on_select = p.on_select.clone();
            let custom = matches!(p.selected, AccentColor::Custom(_));
            // A custom accent shows its color; otherwise a plus invites choosing one.
            let trigger: Element<'a, M> = if custom {
                round_swatch(current, true, m)
            } else {
                container(Icon::ListAdd.view(m.icon_size))
                    .center(Length::Fixed(m.button_min_height))
                    .style(|t: &Theme| iced::widget::container::Style {
                        border: iced::Border {
                            color: t.semantic.border,
                            width: t.metrics.border_width,
                            radius: t.metrics.radius_pill.into(),
                        },
                        ..Default::default()
                    })
                    .into()
            };
            let input = ColorInput::new(
                c,
                // The text is the app's; a custom accent is its hex.
                text,
                move |t| on_text(t),
                move |h: Hsva| {
                    let [r, g, b] = h.to_rgb8();
                    on_select(AccentColor::custom(r, g, b))
                },
            );
            swatches.push(
                input
                    .entry_props(p.entry_props.clone())
                    .swatches(AccentColor::ALL.map(|a| {
                        let c = a.to_color();
                        Hsva::from_rgba(c.r, c.g, c.b, 1.0).format(ColorFormat::Hex)
                    }))
                    .trigger(trigger)
                    .dialog(p.dialog)
                    .scrim(p.scrim)
                    .close_on_outside(p.close_on_outside)
                    .metrics(m)
                    .into(),
            );
        }
        row(swatches)
            .spacing(p.spacing.unwrap_or(m.space_s))
            .wrap()
            .into()
    }
}

#[cfg(test)]
mod spacing_params {
    use super::*;

    #[test]
    fn the_swatch_gap_is_a_parameter() {
        let p = accent_picker(AccentColor::Blue, |_| ()).spacing(3.0);
        assert_eq!(p.spacing, Some(3.0));
        let _: Element<'_, ()> = p.into();
    }
}

/// Presentation-only overrides supported by [`AccentPicker`]. Values and events stay parent-owned.
#[derive(Clone, Default)]
pub struct AccentPickerProps {
    pub spacing: Option<f32>,
    pub metrics: Option<crate::Metrics>,
    pub entry_props: Option<crate::components::EntryProps>,
    pub button_props: Option<crate::components::ButtonProps>,
}
impl AccentPickerProps {
    pub fn spacing(mut self, value: impl Into<f32>) -> Self {
        self.spacing = Some(value.into());
        self
    }
    pub fn metrics(mut self, value: impl Into<crate::Metrics>) -> Self {
        self.metrics = Some(value.into());
        self
    }
    pub fn entry_props(mut self, value: impl Into<crate::components::EntryProps>) -> Self {
        self.entry_props = Some(value.into());
        self
    }
    pub fn button_props(mut self, value: impl Into<crate::components::ButtonProps>) -> Self {
        self.button_props = Some(value.into());
        self
    }
    pub fn apply<'a, M: Clone + 'a>(&self, widget: AccentPicker<'a, M>) -> AccentPicker<'a, M> {
        let mut widget = widget;
        if let Some(value) = &self.spacing {
            widget = widget.spacing(value.clone());
        }
        if let Some(value) = &self.metrics {
            widget = widget.metrics(value.clone());
        }
        if let Some(props) = &self.entry_props {
            widget.entry_props.merge(props);
        }
        if let Some(props) = &self.button_props {
            widget.button_props.merge(props);
        }
        widget
    }
}
