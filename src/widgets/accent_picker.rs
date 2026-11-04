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
    selected: AccentColor,
    on_select: std::rc::Rc<dyn Fn(AccentColor) -> M + 'a>,
    custom: Option<Custom<'a, M>>,
    dialog: bool,
    scrim: bool,
    close_on_outside: bool,
    metrics: Metrics,
}

pub fn accent_picker<'a, M: Clone + 'a>(
    selected: AccentColor,
    on_select: impl Fn(AccentColor) -> M + 'a,
) -> AccentPicker<'a, M> {
    AccentPicker {
        selected,
        on_select: std::rc::Rc::new(on_select),
        custom: None,
        dialog: false,
        scrim: true,
        close_on_outside: true,
        metrics: Metrics::desktop(),
    }
}

impl<'a, M: Clone + 'a> AccentPicker<'a, M> {
    /// Adds the custom swatch. `text` is what the popover's entry shows (the app keeps it, so
    /// typing is not interrupted); `on_text` gets what was typed. Parse it with
    /// [`AccentColor::parse`] and select the result.
    pub fn custom(mut self, text: &'a str, on_text: impl Fn(String) -> M + 'a) -> Self {
        self.custom = Some((text, std::rc::Rc::new(on_text)));
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
fn round_swatch<'a, M: 'a>(color: iced::Color, on: bool) -> Element<'a, M> {
    let content: Element<'a, M> = if on {
        Icon::Check
            .colored(14.0, crate::color::readable_on(color))
            .into()
    } else {
        iced::widget::space().into()
    };
    container(content)
        .center(Length::Fixed(26.0))
        .style(move |_: &Theme| iced::widget::container::Style {
            background: Some(iced::Background::Color(color)),
            border: iced::Border::default().rounded(999.0),
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
                button(round_swatch(a.to_color(), *a == p.selected))
                    .padding(0)
                    .style(|_: &Theme, _| iced::widget::button::Style::default())
                    .on_press((p.on_select)(*a))
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
                round_swatch(current, true)
            } else {
                container(Icon::ListAdd.view(14.0))
                    .center(Length::Fixed(26.0))
                    .style(|t: &Theme| iced::widget::container::Style {
                        border: iced::Border {
                            color: t.semantic.border,
                            width: 1.5,
                            radius: 999.0.into(),
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
        row(swatches).spacing(m.space_s).wrap().into()
    }
}
