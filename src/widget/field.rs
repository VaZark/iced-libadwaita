//! `Field`: a label, a control and a message, with one validation presentation for every
//! form control (Radix `Form.Field`, HTML label + control + message).
//!
//! A thin applier of [`validation`](crate::primitives::validation): the application owns the
//! value, the `touched` flag and the [`Validity`]; the field only decides what to show
//! (pristine until touched, then valid / warning / invalid) and draws an outline around the
//! control plus the message below it. The control is any element (entry, text view,
//! dropdown, check button, scale, rows …), kept at a fixed position in the tree so its state
//! and overlays (dropdown popovers) survive validity changes.

use super::Element;
use crate::primitives::validation::{self, Interaction, Presentation, Validity};
use crate::{Icon, Metrics, Theme};
use iced::widget::{column, container, row, space, text};
use iced::{Alignment, Border, Length, Padding};

/// Where the label sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FieldLayout {
    /// Label above the control (the standard form layout).
    #[default]
    Stacked,
    /// Label and description at the start, the control at the end, like a libadwaita row;
    /// the message sits below.
    Row,
}

pub struct Field<'a, M> {
    label: String,
    control: Element<'a, M>,
    description: Option<String>,
    validity: Validity,
    required: bool,
    touched: bool,
    submitted: bool,
    disabled: bool,
    inline: bool,
    show_valid: bool,
    layout: FieldLayout,
    metrics: Metrics,
}

impl<'a, M: 'a> Field<'a, M> {
    pub fn new(label: impl Into<String>, control: impl Into<Element<'a, M>>) -> Self {
        Self {
            label: label.into(),
            control: control.into(),
            description: None,
            validity: Validity::Ok,
            required: false,
            touched: false,
            submitted: false,
            disabled: false,
            inline: false,
            show_valid: false,
            layout: FieldLayout::Stacked,
            metrics: Metrics::desktop(),
        }
    }
    /// Also draw the valid state (a success ring). Off by default: like CSS, only
    /// `:user-invalid` (and a warning) is highlighted, and `:user-valid` adds nothing special.
    pub fn show_valid(mut self, on: bool) -> Self {
        self.show_valid = on;
        self
    }
    /// Help text under the label (stacked) or the control; replaced by the message while one
    /// is shown.
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    /// The current [`Validity`] of the value (shown once touched or submitted).
    pub fn validity(mut self, v: Validity) -> Self {
        self.validity = v;
        self
    }
    /// Shorthand for `.validity(Validity::Error(msg))`, or `Ok` for `None`.
    pub fn error(mut self, msg: Option<impl Into<String>>) -> Self {
        self.validity = msg.map_or(Validity::Ok, |m| Validity::Error(m.into()));
        self
    }
    /// Marks the field required (an asterisk after the label).
    pub fn required(mut self, r: bool) -> Self {
        self.required = r;
        self
    }
    /// The user has edited or left the control.
    pub fn touched(mut self, t: bool) -> Self {
        self.touched = t;
        self
    }
    /// A submit was attempted: reveals validity like `touched`.
    pub fn submitted(mut self, s: bool) -> Self {
        self.submitted = s;
        self
    }
    /// Sets `touched` and `submitted` together from an [`Interaction`], e.g. the one a
    /// [`FormState`](crate::primitives::FormState) computes for this field's key.
    pub fn interaction(mut self, i: Interaction) -> Self {
        self.touched = i.touched;
        self.submitted = i.submitted;
        self
    }
    /// Dims the label and description and hides validation. Disable the control itself by
    /// leaving off its `on_*` callbacks.
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    /// The control has no box of its own (check or radio button, scale): pad it inside the
    /// outline so the ring does not touch its content.
    pub fn inline(mut self, i: bool) -> Self {
        self.inline = i;
        self
    }
    pub fn layout(mut self, l: FieldLayout) -> Self {
        self.layout = l;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }

    fn revealed(&self) -> bool {
        !self.disabled && (self.touched || self.submitted)
    }
    /// What this field currently shows.
    pub fn presentation(&self) -> Presentation {
        validation::present(&self.validity, self.revealed())
    }
}

/// The outline color for a presentation, if any.
fn ring_color(t: &Theme, p: Presentation) -> Option<iced::Color> {
    match p {
        Presentation::Pristine => None,
        Presentation::Valid => Some(t.semantic.success.color),
        Presentation::Warning => Some(t.semantic.warning.color),
        Presentation::Invalid => Some(t.semantic.error.color),
    }
}

/// Wraps `control` in the shared validation outline: a ring just outside it, transparent
/// when pristine so the layout does not shift.
pub fn outlined<'a, M: 'a>(
    control: impl Into<Element<'a, M>>,
    p: Presentation,
    m: Metrics,
) -> Element<'a, M> {
    let control = control.into();
    // Hug a control that does not fill (a dropdown trigger, a check button).
    let width = match control.as_widget().size().width {
        Length::Shrink | Length::Fixed(_) => Length::Shrink,
        _ => Length::Fill,
    };
    outlined_width(control, p, m, width)
}

fn outlined_width<'a, M: 'a>(
    control: impl Into<Element<'a, M>>,
    p: Presentation,
    m: Metrics,
    width: Length,
) -> Element<'a, M> {
    let w = m.focus_ring_width;
    container(control)
        .padding(w)
        .width(width)
        .style(move |t: &Theme| container::Style {
            border: Border {
                color: ring_color(t, p).unwrap_or(iced::Color::TRANSPARENT),
                width: w,
                radius: (t.metrics.radius_m + w).into(),
            },
            ..Default::default()
        })
        .into()
}

impl<'a, M: 'a> From<Field<'a, M>> for Element<'a, M> {
    fn from(f: Field<'a, M>) -> Self {
        let m = f.metrics;
        // Valid is a state the app can read, but by default nothing is drawn for it.
        let p = match f.presentation() {
            Presentation::Valid if !f.show_valid => Presentation::Pristine,
            p => p,
        };
        let msg = validation::message(&f.validity, f.revealed()).map(str::to_owned);
        let disabled = f.disabled;
        let control: Element<'a, M> = if f.inline {
            container(f.control)
                .padding(Padding::from([m.space_xxs, m.space_xs]))
                .into()
        } else {
            f.control
        };

        let dim = move |t: &Theme| {
            if disabled {
                crate::color::fade(t.semantic.window.fg, t.semantic.opacity.disabled)
            } else {
                t.semantic.window.fg
            }
        };

        // Label (+ required marker).
        let mut title = row![
            text(f.label)
                .size(m.space_m + 1.0)
                .font(iced::Font {
                    weight: iced::font::Weight::Bold,
                    ..iced::Font::DEFAULT
                })
                .style(move |t: &Theme| text::Style {
                    color: Some(dim(t))
                })
        ]
        .spacing(m.space_xxs);
        title = title.push(if f.required {
            Element::from(
                text("*")
                    .size(m.space_m + 1.0)
                    .style(move |t: &Theme| text::Style {
                        color: Some(if disabled {
                            dim(t)
                        } else {
                            t.semantic.error.color
                        }),
                    }),
            )
        } else {
            space().into()
        });

        // Description or message, always one element so positions are stable.
        let note: Element<'a, M> = if let Some(msg) = msg {
            let invalid = p == Presentation::Invalid;
            let icon = if invalid {
                Icon::DialogError
            } else {
                Icon::DialogWarning
            };
            let size = m.space_m + 1.0;
            row![
                icon.view(size)
                    .style(move |t: &Theme, _| iced::widget::svg::Style {
                        color: ring_color(t, p),
                    }),
                text(msg).size(size).style(move |t: &Theme| text::Style {
                    color: ring_color(t, p),
                })
            ]
            .spacing(m.space_xs)
            .align_y(Alignment::Center)
            .into()
        } else if let Some(d) = f.description {
            text(d)
                .size(m.space_m + 1.0)
                .style(move |t: &Theme| text::Style {
                    color: Some(crate::color::fade(
                        t.semantic.window.fg,
                        if disabled {
                            t.semantic.opacity.disabled * 0.6
                        } else {
                            0.7
                        },
                    )),
                })
                .into()
        } else {
            space().into()
        };
        let note = container(note).padding(Padding {
            top: m.space_xxs,
            ..Padding::ZERO
        });

        match f.layout {
            FieldLayout::Stacked => column![
                title,
                container(outlined(control, p, m)).padding(Padding {
                    top: m.space_xxs,
                    ..Padding::ZERO
                }),
                note
            ]
            .width(Length::Fill)
            .into(),
            FieldLayout::Row => column![
                row![
                    container(title).width(Length::Fill),
                    container(outlined(control, p, m))
                        .width(Length::FillPortion(2))
                        .align_x(Alignment::End),
                ]
                .spacing(m.space_m)
                .align_y(Alignment::Center),
                note
            ]
            .width(Length::Fill)
            .into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field() -> Field<'static, ()> {
        Field::new("Name", iced::widget::space()).validity(Validity::error("x"))
    }

    #[test]
    fn pristine_until_touched_or_submitted() {
        assert_eq!(field().presentation(), Presentation::Pristine);
        assert_eq!(field().touched(true).presentation(), Presentation::Invalid);
        assert_eq!(
            field().submitted(true).presentation(),
            Presentation::Invalid
        );
    }

    #[test]
    fn disabled_never_shows_validation() {
        assert_eq!(
            field().touched(true).disabled(true).presentation(),
            Presentation::Pristine
        );
    }
}
