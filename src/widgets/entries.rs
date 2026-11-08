//! Full-width application rows. EntryInput selects the rendered control; typed entries
//! construct the same EntryRow with an appropriate input.
use super::{ChipGroup, ChipOption, DateInput, NumberInput};
use crate::components::{Entry, InputWrapper, TextView};
use crate::primitives::validation::{Interaction, Validity};
use crate::{Element, EntryIcon, IconSource, Metrics};
use iced::{
    Length,
    widget::{button, column, container, row, text},
};

/// The control an EntryRow renders. Selection options are rendered in a popover whose
/// trigger is the entire row, including its label and padding.
pub enum EntryInput<'a, M> {
    Text(Entry<'a, M>),
    Number(NumberInput<'a, M>),
    Date(DateInput<'a, M>),
    Switch {
        active: bool,
        on_toggle: Box<dyn Fn(bool) -> M + 'a>,
    },
    Action(Entry<'a, M>),
    Select {
        value: String,
        options: Element<'a, M>,
        multiple: bool,
    },
    Multiline(TextView<'a, M>),
    Custom(Element<'a, M>),
}

/// Standard full-width application control with its label inside the row.
pub struct EntryRow<'a, M> {
    label: String,
    input: EntryInput<'a, M>,
    description: Option<String>,
    validity: Validity,
    interaction: Interaction,
    required: bool,
    flat: bool,
    metrics: Metrics,
}
impl<'a, M: 'a> EntryRow<'a, M> {
    pub fn new(label: impl Into<String>, input: EntryInput<'a, M>) -> Self {
        Self {
            label: label.into(),
            input,
            description: None,
            validity: Validity::Ok,
            interaction: Interaction::default(),
            required: false,
            flat: false,
            metrics: Metrics::desktop(),
        }
    }
    /// Transparent controls with flat row hover styling, suitable outside a boxed list.
    pub fn flat(mut self, flat: bool) -> Self {
        self.flat = flat;
        self
    }
    pub fn description(mut self, value: impl Into<String>) -> Self {
        self.description = Some(value.into());
        self
    }
    pub fn validity(mut self, value: Validity) -> Self {
        self.validity = value;
        self
    }
    pub fn interaction(mut self, value: Interaction) -> Self {
        self.interaction = value;
        self
    }
    pub fn required(mut self, value: bool) -> Self {
        self.required = value;
        self
    }
    pub fn metrics(mut self, value: Metrics) -> Self {
        self.metrics = value;
        self
    }
}
fn surface<'a, M: 'a>(content: impl Into<Element<'a, M>>, m: Metrics) -> Element<'a, M> {
    container(super::rows::row_frame(content, &m))
        .padding(m.row_padding)
        .width(Length::Fill)
        .into()
}
impl<'a, M: Clone + 'a> From<EntryRow<'a, M>> for Element<'a, M> {
    fn from(e: EntryRow<'a, M>) -> Self {
        let m = e.metrics;
        let label_text = format!("{}{}", e.label, if e.required { " *" } else { "" });
        let label = || {
            crate::Typography::default()
                .caption(label_text.clone())
                .style(crate::styles::text::dim)
        };
        let line = |input: Element<'a, M>| -> Element<'a, M> {
            column![label(), input]
                .spacing(2)
                .width(Length::Fill)
                .into()
        };
        let content = match e.input {
            EntryInput::Text(input) | EntryInput::Action(input) => surface(
                input
                    .flat(e.flat)
                    .embedded()
                    .row_label(label_text.clone())
                    .metrics(m)
                    .width(Length::Fill),
                m,
            ),
            EntryInput::Number(input) => surface(
                input
                    .flat(e.flat)
                    .embedded()
                    .row_label(label_text.clone())
                    .metrics(m)
                    .width(Length::Fill),
                m,
            ),
            EntryInput::Date(input) => surface(
                input
                    .flat(e.flat)
                    .embedded()
                    .row_label(label_text.clone())
                    .metrics(m)
                    .width(Length::Fill),
                m,
            ),
            EntryInput::Multiline(input) => surface(line(input.flat(e.flat).embedded(m).into()), m),
            EntryInput::Custom(input) => surface(line(input), m),
            EntryInput::Switch { active, on_toggle } => {
                let toggle = on_toggle(!active);
                let sw = crate::components::switch(active).on_toggle(on_toggle);
                button(surface(
                    row![
                        container(crate::Typography::default().body(label_text.clone()))
                            .width(Length::Fill),
                        sw
                    ]
                    .align_y(iced::Alignment::Center),
                    m,
                ))
                .padding(0)
                .width(Length::Fill)
                .style(crate::styles::button::list_row)
                .on_press(toggle)
                .into()
            }
            EntryInput::Select {
                value,
                options,
                multiple,
            } => {
                let value = text(if value.is_empty() {
                    "Select…".into()
                } else {
                    value
                })
                .width(Length::Fill);
                let trigger = row![
                    container(line(value.into())).width(Length::Fill),
                    EntryIcon::Select.view(m.icon_size)
                ]
                .align_y(iced::Alignment::Center)
                .spacing(m.space_s);
                super::Popover::new(surface(trigger, m), options)
                    .width(Length::Fill)
                    .padding(0)
                    .appearance(if e.flat {
                        super::PopoverAppearance::Flat
                    } else {
                        super::PopoverAppearance::Row
                    })
                    .keep_open(multiple)
                    .into()
            }
        };
        let mut wrapper = InputWrapper::new("", content)
            .hide_label()
            .validity(e.validity)
            .interaction(e.interaction)
            .metrics(m);
        if let Some(description) = e.description {
            wrapper = wrapper.description(description);
        }
        wrapper.into()
    }
}
macro_rules! entry_type {
    ($name:ident) => {
        pub struct $name<'a, M>(EntryRow<'a, M>);
        impl<'a, M: 'a> $name<'a, M> {
            pub fn flat(mut self, flat: bool) -> Self {
                self.0 = self.0.flat(flat);
                self
            }
            pub fn description(mut self, value: impl Into<String>) -> Self {
                self.0 = self.0.description(value);
                self
            }
            pub fn validity(mut self, value: Validity) -> Self {
                self.0 = self.0.validity(value);
                self
            }
            pub fn interaction(mut self, value: Interaction) -> Self {
                self.0 = self.0.interaction(value);
                self
            }
            pub fn required(mut self, value: bool) -> Self {
                self.0 = self.0.required(value);
                self
            }
            pub fn metrics(mut self, value: Metrics) -> Self {
                self.0 = self.0.metrics(value);
                self
            }
        }
        impl<'a, M: Clone + 'a> From<$name<'a, M>> for Element<'a, M> {
            fn from(value: $name<'a, M>) -> Self {
                value.0.into()
            }
        }
    };
}
entry_type!(TextEntry);
entry_type!(NumberEntry);
entry_type!(DateEntry);
entry_type!(SwitchEntry);
entry_type!(ActionEntry);
entry_type!(SelectEntry);
entry_type!(MultiselectEntry);
entry_type!(ChipGroupEntry);
entry_type!(MultilineEntry);
impl<'a, M: Clone + 'a> TextEntry<'a, M> {
    pub fn new(
        label: impl Into<String>,
        value: impl Into<String>,
        on_input: impl Fn(String) -> M + 'a,
    ) -> Self {
        Self::from_input(label, Entry::new("", value).on_input(on_input))
    }
    pub fn from_input(label: impl Into<String>, input: Entry<'a, M>) -> Self {
        Self(EntryRow::new(label, EntryInput::Text(input)))
    }
    pub fn prefix(mut self, prefix: impl Into<Element<'a, M>>) -> Self {
        if let EntryInput::Text(input) = self.0.input {
            self.0.input = EntryInput::Text(input.left_section(prefix));
        }
        self
    }
    pub fn suffix(mut self, suffix: impl Into<Element<'a, M>>) -> Self {
        if let EntryInput::Text(input) = self.0.input {
            self.0.input = EntryInput::Text(input.right_section(suffix));
        }
        self
    }
    pub fn password(mut self, secure: bool) -> Self {
        if let EntryInput::Text(input) = self.0.input {
            self.0.input = EntryInput::Text(input.password(secure));
        }
        self
    }
}
impl<'a, M: Clone + 'a> NumberEntry<'a, M> {
    pub fn new(
        label: impl Into<String>,
        value: &'a str,
        on_input: impl Fn(String) -> M + 'a,
    ) -> Self {
        Self::from_input(label, NumberInput::new(value, on_input))
    }
    pub fn from_input(label: impl Into<String>, input: NumberInput<'a, M>) -> Self {
        Self(EntryRow::new(label, EntryInput::Number(input)))
    }
}
impl<'a, M: Clone + 'a> DateEntry<'a, M> {
    pub fn new(
        label: impl Into<String>,
        value: &'a str,
        view: crate::primitives::date::Date,
        on_input: impl Fn(String) -> M + 'a,
    ) -> Self {
        Self::from_input(label, DateInput::new(value, view, on_input))
    }
    pub fn from_input(label: impl Into<String>, input: DateInput<'a, M>) -> Self {
        Self(EntryRow::new(label, EntryInput::Date(input)))
    }
    pub fn calendar(mut self, calendar: crate::primitives::calendar::CalendarView) -> Self {
        if let EntryInput::Date(input) = self.0.input {
            self.0.input = EntryInput::Date(input.calendar(calendar));
        }
        self
    }
    pub fn on_nav(
        mut self,
        on_nav: impl Fn(crate::primitives::calendar::CalendarView) -> M + 'a,
    ) -> Self {
        if let EntryInput::Date(input) = self.0.input {
            self.0.input = EntryInput::Date(input.on_nav(on_nav));
        }
        self
    }
    /// false: dropdown calendar; true: modal calendar.
    pub fn dialog(mut self, modal: bool) -> Self {
        if let EntryInput::Date(input) = self.0.input {
            self.0.input = EntryInput::Date(input.dialog(modal));
        }
        self
    }
}
impl<'a, M: Clone + 'a> SwitchEntry<'a, M> {
    pub fn new(label: impl Into<String>, active: bool, on_toggle: impl Fn(bool) -> M + 'a) -> Self {
        Self(EntryRow::new(
            label,
            EntryInput::Switch {
                active,
                on_toggle: Box::new(on_toggle),
            },
        ))
    }
}
impl<'a, M: Clone + 'a> ActionEntry<'a, M> {
    /// The same text renderer as TextEntry, read-only with exactly one trailing action.
    pub fn new(
        label: impl Into<String>,
        value: impl Into<String>,
        icon: impl Into<IconSource>,
        action: M,
    ) -> Self {
        Self(EntryRow::new(
            label,
            EntryInput::Action(
                Entry::new("", value)
                    .right_section(crate::components::section_button(icon, action)),
            ),
        ))
    }
}
impl<'a, M: Clone + 'a> SelectEntry<'a, M> {
    pub fn new<T: ToString + PartialEq + Clone + 'a>(
        label: impl Into<String>,
        options: &'a [T],
        selected: Option<T>,
        on_select: impl Fn(T) -> M + 'a,
    ) -> Self {
        let value = selected
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_default();
        let items =
            super::dropdown::option_items(options, |o| selected.as_ref() == Some(o), on_select);
        Self(EntryRow::new(
            label,
            EntryInput::Select {
                value,
                options: super::popover(items, 280.0),
                multiple: false,
            },
        ))
    }
}
impl<'a, M: Clone + 'a> MultiselectEntry<'a, M> {
    pub fn new<T: ToString + PartialEq + Clone + 'a>(
        label: impl Into<String>,
        options: &'a [T],
        selected: &'a [T],
        on_toggle: impl Fn(T) -> M + 'a,
    ) -> Self {
        let value = selected
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        let items = super::dropdown::option_items(options, |o| selected.contains(o), on_toggle);
        Self(EntryRow::new(
            label,
            EntryInput::Select {
                value,
                options: super::popover(items, 280.0),
                multiple: true,
            },
        ))
    }
}
impl<'a, M: Clone + 'a> ChipGroupEntry<'a, M> {
    pub fn new<K: Clone + PartialEq + 'a>(
        label: impl Into<String>,
        options: impl IntoIterator<Item = ChipOption<K>>,
        selected: &'a [K],
        on_toggle: impl Fn(K) -> M + 'a,
    ) -> Self {
        let options: Vec<_> = options.into_iter().collect();
        let value = options
            .iter()
            .filter(|o| selected.contains(&o.key))
            .map(|o| o.label.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let chips: Element<'a, M> = ChipGroup::new(options, selected, on_toggle).into();
        Self(EntryRow::new(
            label,
            EntryInput::Select {
                value,
                options: super::popover([chips], 320.0),
                multiple: true,
            },
        ))
    }
}
impl<'a, M: Clone + 'a> MultilineEntry<'a, M> {
    pub fn new(
        label: impl Into<String>,
        content: &'a iced::widget::text_editor::Content,
        on_action: impl Fn(iced::widget::text_editor::Action) -> M + 'a,
    ) -> Self {
        Self::from_input(
            label,
            crate::components::text_view(content)
                .max_lines(12)
                .on_action(on_action),
        )
    }
    pub fn from_input(label: impl Into<String>, input: TextView<'a, M>) -> Self {
        Self(EntryRow::new(label, EntryInput::Multiline(input)))
    }
}
/// Navigation destinations are separate from value actions.
pub struct NavigationRow<'a, M>(super::ActionRow<'a, M>);
impl<'a, M> NavigationRow<'a, M> {
    pub fn new(title: impl Into<String>, destination: M) -> Self {
        Self(
            super::ActionRow::new(title)
                .chevron(true)
                .on_activate(destination),
        )
    }
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.0 = self.0.subtitle(subtitle);
        self
    }
    pub fn prefix(mut self, prefix: impl Into<Element<'a, M>>) -> Self {
        self.0 = self.0.prefix(prefix);
        self
    }
    pub fn metrics(mut self, metrics: Metrics) -> Self {
        self.0 = self.0.metrics(metrics);
        self
    }
}
impl<'a, M: Clone + 'a> From<NavigationRow<'a, M>> for Element<'a, M> {
    fn from(value: NavigationRow<'a, M>) -> Self {
        value.0.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::{Shell, clipboard, layout, renderer::Headless, widget::Tree};
    use iced::{Event, Point, Rectangle, Size, Vector, mouse};

    #[test]
    fn selection_rows_open_from_the_label_padding_and_right_edge() {
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .expect("software renderer");
        const OPTIONS: [&str; 2] = ["Design", "Development"];
        for width in [220.0, 640.0] {
            for kind in 0..3 {
                for at in [
                    Point::new(6.0, 6.0),
                    Point::new(30.0, 22.0),
                    Point::new(width - 6.0, 25.0),
                ] {
                    let mut element: Element<'_, &str> = match kind {
                        0 => SelectEntry::new("Team", &OPTIONS, Some("Design"), |v| v).into(),
                        1 => MultiselectEntry::new("Teams", &OPTIONS, &OPTIONS[..1], |v| v).into(),
                        _ => ChipGroupEntry::new(
                            "Tags",
                            OPTIONS.iter().map(|v| ChipOption::new(*v, *v)),
                            &OPTIONS[..1],
                            |v| v,
                        )
                        .into(),
                    };
                    assert_eq!(element.as_widget().size().width, Length::Fill);
                    let mut tree = Tree::new(&element);
                    let node = element.as_widget_mut().layout(
                        &mut tree,
                        &renderer,
                        &layout::Limits::new(Size::ZERO, Size::new(width, 1000.0)),
                    );
                    let layout = layout::Layout::new(&node);
                    assert_eq!(node.size().width, width);
                    assert!(
                        element
                            .as_widget_mut()
                            .overlay(
                                &mut tree,
                                layout,
                                &renderer,
                                &Rectangle::new(Point::ORIGIN, Size::new(width, 1000.0)),
                                Vector::ZERO
                            )
                            .is_none()
                    );
                    let mut messages = vec![];
                    for event in [
                        mouse::Event::ButtonPressed(mouse::Button::Left),
                        mouse::Event::ButtonReleased(mouse::Button::Left),
                    ] {
                        element.as_widget_mut().update(
                            &mut tree,
                            &Event::Mouse(event),
                            layout,
                            mouse::Cursor::Available(at),
                            &renderer,
                            &mut clipboard::Null,
                            &mut Shell::new(&mut messages),
                            &Rectangle::new(Point::ORIGIN, Size::new(width, 1000.0)),
                        );
                    }
                    assert!(
                        element
                            .as_widget_mut()
                            .overlay(
                                &mut tree,
                                layout,
                                &renderer,
                                &Rectangle::new(Point::ORIGIN, Size::new(width, 1000.0)),
                                Vector::ZERO
                            )
                            .is_some(),
                        "kind={kind}, width={width}, at={at:?}"
                    );
                    assert!(messages.is_empty(), "Opening must not change the selection");
                }
            }
        }
    }
}
