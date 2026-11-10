//! Full-width application rows. EntryInput selects the rendered control; rows
//! construct the same EntryRow with an appropriate input.
mod surface;

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
/// The shared framing for an input inside a row: flat or boxed, embedded, full width.
macro_rules! embedded {
    ($input:expr, $flat:expr, $metrics:expr) => {
        $input
            .flat($flat)
            .embedded()
            .metrics($metrics)
            .width(Length::Fill)
    };
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
            EntryInput::Text(input) | EntryInput::Action(input) => {
                let insets = input.label_insets(&m);
                let input = embedded!(input, e.flat, m);
                surface(
                    column![container(label()).padding(insets), input]
                        .spacing(2)
                        .width(Length::Fill),
                    m,
                )
            }
            EntryInput::Number(input) => surface(line(embedded!(input, e.flat, m).into()), m),
            EntryInput::Date(input) => surface(line(embedded!(input, e.flat, m).into()), m),
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
        let presentation =
            crate::primitives::validation::present(&e.validity, e.interaction.revealed());
        let focus_ring = e.flat
            && !matches!(
                presentation,
                crate::primitives::validation::Presentation::Invalid
                    | crate::primitives::validation::Presentation::Warning
            );
        let content = surface::outlined(content, focus_ring, m);
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
impl<'a, M: Clone + 'a> EntryRow<'a, M> {
    /// A single-line text entry. Configure it further (sections, password) with
    /// `EntryRow::new(label, EntryInput::Text(entry))`.
    pub fn text(
        label: impl Into<String>,
        value: impl Into<String>,
        on_input: impl Fn(String) -> M + 'a,
    ) -> Self {
        Self::new(
            label,
            EntryInput::Text(Entry::new("", value).on_input(on_input)),
        )
    }
    pub fn number(
        label: impl Into<String>,
        value: &'a str,
        on_input: impl Fn(String) -> M + 'a,
    ) -> Self {
        Self::new(label, EntryInput::Number(NumberInput::new(value, on_input)))
    }
    pub fn date(
        label: impl Into<String>,
        value: &'a str,
        view: crate::primitives::date::Date,
        on_input: impl Fn(String) -> M + 'a,
    ) -> Self {
        Self::new(
            label,
            EntryInput::Date(DateInput::new(value, view, on_input)),
        )
    }
    pub fn switch(
        label: impl Into<String>,
        active: bool,
        on_toggle: impl Fn(bool) -> M + 'a,
    ) -> Self {
        Self::new(
            label,
            EntryInput::Switch {
                active,
                on_toggle: Box::new(on_toggle),
            },
        )
    }
    /// The text renderer, read-only with exactly one trailing action.
    pub fn action(
        label: impl Into<String>,
        value: impl Into<String>,
        icon: impl Into<IconSource>,
        action: M,
    ) -> Self {
        Self::new(
            label,
            EntryInput::Action(
                Entry::new("", value)
                    .right_section(crate::components::section_button(icon, action)),
            ),
        )
    }
    pub fn select<T: ToString + PartialEq + Clone + 'a>(
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
        Self::new(
            label,
            EntryInput::Select {
                value,
                options: super::popover(items, 280.0),
                multiple: false,
            },
        )
    }
    pub fn multiselect<T: ToString + PartialEq + Clone + 'a>(
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
        Self::new(
            label,
            EntryInput::Select {
                value,
                options: super::popover(items, 280.0),
                multiple: true,
            },
        )
    }
    pub fn chips<K: Clone + PartialEq + 'a>(
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
        Self::new(
            label,
            EntryInput::Select {
                value,
                options: super::popover([chips], 320.0),
                multiple: true,
            },
        )
    }
    /// A growing multi-line editor. Restrict it (single line) with
    /// `EntryRow::new(label, EntryInput::Multiline(text_view(..)))`.
    pub fn multiline(
        label: impl Into<String>,
        content: &'a iced::widget::text_editor::Content,
        on_action: impl Fn(iced::widget::text_editor::Action) -> M + 'a,
    ) -> Self {
        Self::new(
            label,
            EntryInput::Multiline(
                crate::components::text_view(content)
                    .max_lines(12)
                    .on_action(on_action),
            ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::advanced::{Shell, clipboard, layout, renderer::Headless, widget::Tree};
    use iced::{Event, Point, Rectangle, Size, Vector, mouse};

    #[test]
    fn row_sections_center_on_the_input_line_below_the_label() {
        use iced::advanced::widget::Operation;
        #[derive(Default)]
        struct Geometry {
            label: Option<Rectangle>,
            input: Option<Rectangle>,
            prefix: Option<Rectangle>,
            suffix: Option<Rectangle>,
        }
        impl Operation for Geometry {
            fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
                operate(self);
            }
            fn text(&mut self, _: Option<&iced::widget::Id>, bounds: Rectangle, value: &str) {
                match value {
                    "Label" => self.label = Some(bounds),
                    "P" => self.prefix = Some(bounds),
                    "S" => self.suffix = Some(bounds),
                    _ => {}
                }
            }
            fn text_input(
                &mut self,
                _: Option<&iced::widget::Id>,
                bounds: Rectangle,
                _: &mut dyn iced::advanced::widget::operation::TextInput,
            ) {
                self.input = Some(bounds);
            }
        }
        let renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        for metrics in [Metrics::desktop(), Metrics::compact()] {
            for flat in [false, true] {
                let mut row: Element<'_, ()> = EntryRow::new(
                    "Label",
                    EntryInput::Text(
                        Entry::new("", "Value")
                            .on_input(|_| ())
                            .left_section(text("P"))
                            .right_section(text("S")),
                    ),
                )
                .flat(flat)
                .metrics(metrics)
                .into();
                let mut tree = Tree::new(&row);
                let node = row.as_widget_mut().layout(
                    &mut tree,
                    &renderer,
                    &layout::Limits::new(Size::ZERO, Size::new(360.0, 200.0)),
                );
                let mut geometry = Geometry::default();
                row.as_widget_mut().operate(
                    &mut tree,
                    layout::Layout::new(&node),
                    &renderer,
                    &mut geometry,
                );
                let label = geometry.label.unwrap();
                let input = geometry.input.unwrap();
                assert!(
                    label.y + label.height <= input.y,
                    "label stays above the input line"
                );
                for section in [geometry.prefix.unwrap(), geometry.suffix.unwrap()] {
                    assert!(
                        (section.center_y() - input.center_y()).abs() < 0.01,
                        "section {section:?} must align with input {input:?}"
                    );
                    assert!(
                        section.y >= label.y + label.height,
                        "sections must not overlap the label"
                    );
                }
            }
        }
    }

    #[test]
    fn flat_focus_ring_encloses_the_label_prefix_and_suffix_in_both_schemes() {
        use iced::advanced::{
            Renderer as _,
            widget::{Operation, operation::Focusable},
        };
        struct Focus(bool);
        impl Operation for Focus {
            fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
                operate(self);
            }
            fn focusable(
                &mut self,
                _: Option<&iced::widget::Id>,
                _: Rectangle,
                state: &mut dyn Focusable,
            ) {
                if self.0 {
                    state.focus();
                } else {
                    state.unfocus();
                }
            }
        }
        let mut renderer = iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap();
        for theme in [crate::Theme::light(), crate::Theme::dark()] {
            for kind in 0..4 {
                let text_content = iced::widget::text_editor::Content::with_text("Notes");
                let mut element: Element<'_, ()> = match kind {
                    0 => EntryRow::new(
                        "Name",
                        EntryInput::Text(
                            Entry::new("", "Ada")
                                .on_input(|_| ())
                                .left_section(crate::Icon::Search.view(16.0))
                                .right_section(text("@app")),
                        ),
                    )
                    .flat(true)
                    .into(),
                    1 => EntryRow::number("Amount", "12", |_| ()).flat(true).into(),
                    2 => EntryRow::date(
                        "Date",
                        "2026-11-08",
                        crate::primitives::date::Date::today(),
                        |_| (),
                    )
                    .flat(true)
                    .into(),
                    _ => EntryRow::multiline("Notes", &text_content, |_| ())
                        .flat(true)
                        .into(),
                };
                let mut tree = Tree::new(&element);
                let node = element.as_widget_mut().layout(
                    &mut tree,
                    &renderer,
                    &layout::Limits::new(Size::ZERO, Size::new(360.0, 300.0)),
                );
                let bounds = node.bounds();
                let viewport = Rectangle::new(Point::ORIGIN, Size::new(360.0, 300.0));
                for focused in [true, false] {
                    element.as_widget_mut().operate(
                        &mut tree,
                        layout::Layout::new(&node),
                        &renderer,
                        &mut Focus(focused),
                    );
                    element.as_widget_mut().update(
                        &mut tree,
                        &Event::Window(iced::window::Event::RedrawRequested(
                            std::time::Instant::now(),
                        )),
                        layout::Layout::new(&node),
                        mouse::Cursor::Unavailable,
                        &renderer,
                        &mut clipboard::Null,
                        &mut Shell::new(&mut vec![]),
                        &viewport,
                    );
                    renderer.reset(viewport);
                    element.as_widget().draw(
                        &tree,
                        &mut renderer,
                        &theme,
                        &iced::advanced::renderer::Style {
                            text_color: theme.semantic.window.fg,
                        },
                        layout::Layout::new(&node),
                        mouse::Cursor::Unavailable,
                        &viewport,
                    );
                    let pixels =
                        renderer.screenshot(Size::new(360, 300), 1.0, theme.semantic.window.bg);
                    let expected = if focused {
                        theme.semantic.accent.bg
                    } else {
                        theme.semantic.window.bg
                    }
                    .into_rgba8();
                    // Top border is above the label; side borders are outside every section.
                    for (x, y) in [
                        (180usize, 1usize),
                        (1, bounds.height as usize / 2),
                        (358, bounds.height as usize / 2),
                    ] {
                        let offset = (y * 360 + x) * 4;
                        assert_eq!(
                            &pixels[offset..offset + 4],
                            &expected,
                            "kind={kind} focused={focused} at {x},{y}"
                        );
                    }
                }
            }
        }
    }

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
                        0 => EntryRow::select("Team", &OPTIONS, Some("Design"), |v| v).into(),
                        1 => EntryRow::multiselect("Teams", &OPTIONS, &OPTIONS[..1], |v| v).into(),
                        _ => EntryRow::chips(
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
