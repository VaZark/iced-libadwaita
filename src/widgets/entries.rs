//! Full-width application rows. EntryInput selects the rendered control; rows
//! construct the same EntryRow with an appropriate input.
mod surface;

use super::{ChipGroup, ChipOption, DateInput, NumberInput};
use crate::components::{
    Entry, InputWrapper, InputWrapperProps, TextProps, TextView, TextViewProps,
};
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
    dropdown_props: super::popover::PopoverProps,
    button_props: crate::components::ButtonProps,
    label_props: crate::components::TextProps,
    entry_props: Option<crate::components::EntryProps>,
    value_props: TextProps,
    wrapper_props: InputWrapperProps,
    text_view_props: TextViewProps,
    label: String,
    input: EntryInput<'a, M>,
    description: Option<String>,
    validity: Validity,
    interaction: Interaction,
    required: bool,
    flat: bool,
    spacing: Option<f32>,
    padding: Option<iced::Padding>,
    value_spacing: Option<f32>,
    metrics: Metrics,
}
impl<'a, M: 'a> EntryRow<'a, M> {
    pub fn dropdown_props(mut self, props: super::popover::PopoverProps) -> Self {
        self.dropdown_props = props;
        self
    }
    pub fn button_props(mut self, props: crate::components::ButtonProps) -> Self {
        self.button_props = props;
        self
    }
    pub fn label_props(mut self, props: crate::components::TextProps) -> Self {
        self.label_props = props;
        self
    }

    pub fn entry_props(mut self, props: crate::components::EntryProps) -> Self {
        self.entry_props = Some(props);
        self
    }

    /// Presentation of a selection's displayed value, including its empty placeholder.
    /// Does not alter the selected value or the caller-owned option elements.
    pub fn value_props(mut self, props: TextProps) -> Self {
        self.value_props = props;
        self
    }

    /// Presentation of the outer description and validation-message children.
    /// The wrapper's label is hidden; use `label_props` for the row's own label.
    pub fn wrapper_props(mut self, props: InputWrapperProps) -> Self {
        self.wrapper_props = props;
        self
    }

    /// Presentation of the native editor in supplied or convenience-built multiline rows.
    /// Content, actions, single-line restrictions and growing-height policy stay unchanged.
    pub fn text_view_props(mut self, props: TextViewProps) -> Self {
        self.text_view_props = props;
        self
    }

    pub fn new(label: impl Into<String>, input: EntryInput<'a, M>) -> Self {
        Self {
            dropdown_props: Default::default(),
            button_props: Default::default(),
            label_props: Default::default(),
            entry_props: Default::default(),
            value_props: Default::default(),
            wrapper_props: Default::default(),
            text_view_props: Default::default(),

            label: label.into(),
            input,
            description: None,
            validity: Validity::Ok,
            interaction: Interaction::default(),
            required: false,
            flat: false,
            spacing: None,
            padding: None,
            value_spacing: None,
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
    /// The gap between the label and the input (default: the theme's `space_xxs`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px.max(0.0));
        self
    }
    /// The padding around the row's content (default: the theme's `row_padding`).
    pub fn padding(mut self, p: impl Into<iced::Padding>) -> Self {
        self.padding = Some(p.into());
        self
    }
    /// For a select row: the gap between the value and the arrow (default: the theme's `space_s`).
    pub fn value_spacing(mut self, px: f32) -> Self {
        self.value_spacing = Some(px.max(0.0));
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
fn surface<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    m: Metrics,
    padding: iced::Padding,
) -> Element<'a, M> {
    container(super::rows::row_frame(content, &m))
        .padding(padding)
        .width(Length::Fill)
        .into()
}
impl<'a, M: Clone + 'a> From<EntryRow<'a, M>> for Element<'a, M> {
    fn from(e: EntryRow<'a, M>) -> Self {
        let m = e.metrics;
        let pad = e.padding.unwrap_or(m.row_padding);
        let gap = e.spacing.unwrap_or(m.space_xxs);
        let label_text = format!("{}{}", e.label, if e.required { " *" } else { "" });
        let label = || {
            e.label_props.apply(
                crate::Typography::desktop()
                    .caption(label_text.clone())
                    .style(crate::styles::text::dim),
            )
        };
        let line = |input: Element<'a, M>| -> Element<'a, M> {
            column![label(), input]
                .spacing(gap)
                .width(Length::Fill)
                .into()
        };
        let content =
            match e.input {
                EntryInput::Text(input) | EntryInput::Action(input) => {
                    let insets = input.label_insets(&m);
                    let input = embedded!(input, e.flat, m);
                    let input = if let Some(props) = &e.entry_props {
                        props.apply(input)
                    } else {
                        input
                    };
                    surface(
                        column![container(label()).padding(insets), input]
                            .spacing(gap)
                            .width(Length::Fill),
                        m,
                        pad,
                    )
                }
                EntryInput::Number(input) => {
                    let input = embedded!(input, e.flat, m);
                    let input = if let Some(props) = &e.entry_props {
                        input.entry_props(props.clone())
                    } else {
                        input
                    };
                    surface(line(input.into()), m, pad)
                }
                EntryInput::Date(input) => {
                    let input = embedded!(input, e.flat, m);
                    let input = if let Some(props) = &e.entry_props {
                        input.entry_props(props.clone())
                    } else {
                        input
                    };
                    surface(line(input.into()), m, pad)
                }
                EntryInput::Multiline(input) => surface(
                    line(
                        e.text_view_props
                            .apply(input.flat(e.flat).embedded(m))
                            .into(),
                    ),
                    m,
                    pad,
                ),
                EntryInput::Custom(input) => surface(line(input), m, pad),
                EntryInput::Switch { active, on_toggle } => {
                    let toggle = on_toggle(!active);
                    let sw = crate::components::switch(active).on_toggle(on_toggle);
                    e.button_props
                        .apply(
                            button(surface(
                                row![
                                    container(e.label_props.apply(
                                        crate::Typography::desktop().body(label_text.clone())
                                    ))
                                    .width(Length::Fill),
                                    sw
                                ]
                                .align_y(iced::Alignment::Center),
                                m,
                                pad,
                            ))
                            .padding(0)
                            .width(Length::Fill)
                            .style(crate::styles::button::list_row)
                            .on_press(toggle),
                        )
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
                        container(line(e.value_props.apply(value))).width(Length::Fill),
                        EntryIcon::Select.view(m.icon_size)
                    ]
                    .align_y(iced::Alignment::Center)
                    .spacing(e.value_spacing.unwrap_or(m.space_s));
                    e.dropdown_props
                        .apply(
                            super::Popover::new(surface(trigger, m, pad), options)
                                .width(Length::Fill)
                                .padding(0)
                                .appearance(if e.flat {
                                    super::PopoverAppearance::Flat
                                } else {
                                    super::PopoverAppearance::Row
                                })
                                .keep_open(multiple),
                        )
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
        e.wrapper_props.apply(wrapper).into()
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

    #[test]
    fn an_entry_row_takes_its_gaps_and_padding() {
        let r = EntryRow::<u8>::text("L", "v", |_| 0)
            .spacing(1.0)
            .padding([2.0, 3.0])
            .value_spacing(4.0);
        assert_eq!((r.spacing, r.value_spacing), (Some(1.0), Some(4.0)));
        assert_eq!(r.padding, Some(iced::Padding::from([2.0, 3.0])));
        let _: Element<'_, u8> = r.into();
    }
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

#[cfg(test)]
mod presentation_merge_tests {
    use super::*;
    use crate::components::{ButtonProps, EntryProps, TextProps};

    #[test]
    fn empty_and_partial_overlays_preserve_existing_row_children() {
        let row = EntryRow::<()>::new(
            "Title",
            EntryInput::Switch {
                active: true,
                on_toggle: Box::new(|_| ()),
            },
        )
        .required(true)
        .label_props(TextProps::default().width(173).height(31))
        .button_props(ButtonProps::default().width(211).clip(true))
        .entry_props(EntryProps::default().width(187).flat(true));
        let row = EntryRowProps::default().apply(row);
        let row = EntryRowProps::default()
            .label_props(TextProps::default().height(37))
            .button_props(ButtonProps::default().clip(false))
            .entry_props(EntryProps::default().padding(0))
            .apply(row);
        assert_eq!(row.label_props.width, Some(Length::Fixed(173.0)));
        assert_eq!(row.label_props.height, Some(Length::Fixed(37.0)));
        assert_eq!(row.button_props.width, Some(Length::Fixed(211.0)));
        assert_eq!(row.button_props.clip, Some(false));
        let entry = row.entry_props.unwrap();
        assert_eq!(entry.width, Some(Length::Fixed(187.0)));
        assert_eq!(entry.flat, Some(true));
        assert_eq!(entry.padding, Some(iced::Padding::ZERO));
        assert!(row.required);
        assert!(matches!(row.input, EntryInput::Switch { active: true, .. }));
    }

    #[test]
    fn empty_optional_props_do_not_create_entry_presentation() {
        let row = EntryRow::<()>::new("Title", EntryInput::Custom(iced::widget::space().into()));
        let row = EntryRowProps::default()
            .entry_props(EntryProps::default())
            .apply(row);
        assert!(row.entry_props.is_none());
    }
}

#[cfg(test)]
mod child_api_tests {
    use super::*;
    use crate::components::{TextEditorProps, TextOverflow};
    use iced::advanced::{
        Shell, clipboard, layout,
        renderer::Headless,
        widget::{Operation, Tree},
    };
    use iced::{Event, Point, Rectangle, Size, mouse};
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    fn renderer() -> iced::Renderer {
        iced::futures::executor::block_on(iced::Renderer::new(
            iced::Font::DEFAULT,
            iced::Pixels(14.0),
            Some("tiny-skia"),
        ))
        .unwrap()
    }

    fn layout<'a>(
        row: EntryRow<'a, u8>,
        renderer: &iced::Renderer,
    ) -> (Element<'a, u8>, Tree, layout::Node) {
        let mut element: Element<'a, u8> = row.into();
        let mut tree = Tree::new(&element);
        let node = element.as_widget_mut().layout(
            &mut tree,
            renderer,
            &layout::Limits::new(Size::ZERO, Size::new(420.0, 800.0)),
        );
        (element, tree, node)
    }

    fn sizes(node: &layout::Node) -> Vec<Size> {
        let mut result = vec![node.size()];
        for child in node.children() {
            result.extend(sizes(child));
        }
        result
    }

    #[derive(Default)]
    struct Texts(Vec<(String, Rectangle)>);
    impl Operation for Texts {
        fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation)) {
            operate(self);
        }
        fn text(&mut self, _: Option<&iced::widget::Id>, bounds: Rectangle, value: &str) {
            self.0.push((value.into(), bounds));
        }
    }

    fn texts(
        element: &mut Element<'_, u8>,
        tree: &mut Tree,
        node: &layout::Node,
        renderer: &iced::Renderer,
    ) -> Texts {
        let mut result = Texts::default();
        element
            .as_widget_mut()
            .operate(tree, layout::Layout::new(node), renderer, &mut result);
        result
    }

    fn draw(
        element: &Element<'_, u8>,
        tree: &Tree,
        node: &layout::Node,
        renderer: &mut iced::Renderer,
    ) {
        element.as_widget().draw(
            tree,
            renderer,
            &crate::Theme::light(),
            &iced::advanced::renderer::Style::default(),
            layout::Layout::new(node),
            mouse::Cursor::Unavailable,
            &Rectangle::with_size(Size::new(420.0, 800.0)),
        );
    }

    fn styled_text(calls: Arc<AtomicUsize>, width: u32, height: u32) -> TextProps {
        TextProps::default()
            .width(width)
            .height(height)
            .style(move |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                iced::widget::text::Style {
                    color: Some(iced::Color::from_rgb8(31, 47, 63)),
                }
            })
    }

    fn row<'a>(kind: usize, content: &'a iced::widget::text_editor::Content) -> EntryRow<'a, u8> {
        let input = match kind {
            0 => EntryInput::Text(
                Entry::new("Hint", "Secret")
                    .password(true)
                    .on_input(|_| 1)
                    .on_submit(2),
            ),
            1 => EntryInput::Action(Entry::new("Hint", "Value").on_input(|_| 1)),
            2 => EntryInput::Number(NumberInput::new("42", |_| 1)),
            3 => EntryInput::Date(DateInput::new(
                "2026-11-08",
                crate::primitives::date::Date::new(2026, 11, 8).unwrap(),
                |_| 1,
            )),
            4 => EntryInput::Switch {
                active: true,
                on_toggle: Box::new(|_| 3),
            },
            5 => EntryInput::Select {
                value: "Selected".into(),
                options: text("Option").into(),
                multiple: false,
            },
            6 => EntryInput::Multiline(crate::components::text_view(content).on_action(|_| 7)),
            7 => EntryInput::Custom(
                button(text("Action"))
                    .width(137)
                    .height(43)
                    .on_press(9)
                    .into(),
            ),
            _ => unreachable!(),
        };
        EntryRow::new("Label", input)
    }

    #[test]
    fn default_and_explicit_empty_child_props_preserve_every_input_branch() {
        let renderer = renderer();
        let content = iced::widget::text_editor::Content::with_text("Retained");
        for kind in 0..8 {
            let (_, _, base) = layout(row(kind, &content), &renderer);
            let (_, _, default) = layout(
                EntryRowProps::default().apply(row(kind, &content)),
                &renderer,
            );
            let empty = EntryRowProps::default()
                .value_props(TextProps::default())
                .wrapper_props(InputWrapperProps::default())
                .text_view_props(TextViewProps::default());
            let (_, _, explicit) = layout(empty.apply(row(kind, &content)), &renderer);
            assert_eq!(sizes(&base), sizes(&default), "branch {kind}");
            assert_eq!(
                sizes(&base),
                sizes(&explicit),
                "branch {kind}, explicit empty"
            );
        }
    }

    #[test]
    fn selected_value_props_reach_single_multiple_and_placeholder_text_and_style() {
        let mut renderer = renderer();
        for multiple in [false, true] {
            for value in ["", "日本語 🦀 selected"] {
                let calls = Arc::new(AtomicUsize::new(0));
                let row = EntryRow::new(
                    "Label",
                    EntryInput::Select {
                        value: value.into(),
                        options: text("Option").into(),
                        multiple,
                    },
                )
                .value_props(styled_text(calls.clone(), 137, 29));
                let row = EntryRowProps::default()
                    .value_props(TextProps::default().height(37))
                    .apply(row);
                let (mut element, mut tree, node) = layout(row, &renderer);
                let expected = if value.is_empty() { "Select…" } else { value };
                let values = texts(&mut element, &mut tree, &node, &renderer);
                assert_eq!(
                    values
                        .0
                        .iter()
                        .find(|(text, _)| text == expected)
                        .unwrap()
                        .1
                        .size(),
                    Size::new(137.0, 37.0)
                );
                draw(&element, &tree, &node, &mut renderer);
                assert!(calls.load(Ordering::SeqCst) > 0);
                element.as_widget_mut().update(
                    &mut tree,
                    &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                    layout::Layout::new(&node),
                    mouse::Cursor::Available(Point::new(30.0, 30.0)),
                    &renderer,
                    &mut clipboard::Null,
                    &mut Shell::new(&mut vec![]),
                    &Rectangle::with_size(Size::new(420.0, 800.0)),
                );
                // Opening remains the native popover's policy, not a value-presentation field.
                assert!(
                    element
                        .as_widget_mut()
                        .overlay(
                            &mut tree,
                            layout::Layout::new(&node),
                            &renderer,
                            &Rectangle::with_size(Size::new(420.0, 800.0)),
                            iced::Vector::ZERO
                        )
                        .is_some()
                );
            }
        }
    }

    #[test]
    fn wrapper_props_reach_all_branches_without_revealing_or_replacing_validation() {
        let mut renderer = renderer();
        let content = iced::widget::text_editor::Content::with_text("Retained");
        for kind in 0..8 {
            for revealed in [false, true] {
                let description_calls = Arc::new(AtomicUsize::new(0));
                let message_calls = Arc::new(AtomicUsize::new(0));
                let wrapper = InputWrapperProps::default()
                    .description_props(styled_text(description_calls.clone(), 137, 29))
                    .message_props(styled_text(message_calls.clone(), 151, 31));
                let row = row(kind, &content)
                    .description("Help")
                    .validity(Validity::Error("Invalid".into()))
                    .interaction(Interaction {
                        touched: revealed,
                        submitted: false,
                    })
                    .wrapper_props(wrapper)
                    .required(true);
                let row = EntryRowProps::default()
                    .wrapper_props(
                        InputWrapperProps::default()
                            .description_props(TextProps::default().height(37))
                            .message_props(TextProps::default().height(41)),
                    )
                    .apply(row);
                assert_eq!(row.validity, Validity::Error("Invalid".into()));
                assert_eq!(row.interaction.revealed(), revealed);
                assert!(row.required);
                let (mut element, mut tree, node) = layout(row, &renderer);
                let values = texts(&mut element, &mut tree, &node, &renderer);
                let expected = if revealed { "Invalid" } else { "Help" };
                let size = if revealed {
                    Size::new(151.0, 41.0)
                } else {
                    Size::new(137.0, 37.0)
                };
                assert_eq!(
                    values
                        .0
                        .iter()
                        .find(|(text, _)| text == expected)
                        .unwrap()
                        .1
                        .size(),
                    size,
                    "branch {kind}"
                );
                assert!(
                    !values
                        .0
                        .iter()
                        .any(|(text, _)| text == if revealed { "Help" } else { "Invalid" })
                );
                assert!(values.0.iter().any(|(text, _)| text == "Label *"));
                draw(&element, &tree, &node, &mut renderer);
                assert_eq!(description_calls.load(Ordering::SeqCst) > 0, !revealed);
                assert_eq!(message_calls.load(Ordering::SeqCst) > 0, revealed);
            }
        }
    }

    #[test]
    fn multiline_props_change_native_layout_and_style_and_preserve_content_actions() {
        let mut renderer = renderer();
        let content = iced::widget::text_editor::Content::with_text("One\nTwo\nThree");
        let (_, _, base) = layout(EntryRow::multiline("Label", &content, |_| 7), &renderer);
        let calls = Arc::new(AtomicUsize::new(0));
        let style_calls = calls.clone();
        let row = EntryRow::multiline("Label", &content, |_| 7).text_view_props(
            TextViewProps::default().editor_props(
                TextEditorProps::default()
                    .size(28)
                    .font(iced::Font::MONOSPACE)
                    .style(move |theme, status| {
                        style_calls.fetch_add(1, Ordering::SeqCst);
                        crate::styles::text_editor::default(theme, status)
                    }),
            ),
        );
        let row = EntryRowProps::default()
            .text_view_props(
                TextViewProps::default().editor_props(TextEditorProps::default().padding(11)),
            )
            .apply(row);
        assert_eq!(
            row.text_view_props.editor_props.size,
            Some(iced::Pixels(28.0))
        );
        assert_eq!(
            row.text_view_props.editor_props.font,
            Some(iced::Font::MONOSPACE)
        );
        let (mut element, mut tree, node) = layout(row, &renderer);
        assert!(node.size().height > base.size().height);
        draw(&element, &tree, &node, &mut renderer);
        assert!(calls.load(Ordering::SeqCst) > 0);
        fn editor(layout: layout::Layout<'_>) -> Option<Rectangle> {
            if layout.children().next().is_none() && layout.bounds().height > 80.0 {
                Some(layout.bounds())
            } else {
                layout.children().find_map(editor)
            }
        }
        let target = editor(layout::Layout::new(&node)).expect("native multiline editor");
        let mut messages = vec![];
        element.as_widget_mut().update(
            &mut tree,
            &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            layout::Layout::new(&node),
            mouse::Cursor::Available(target.center()),
            &renderer,
            &mut clipboard::Null,
            &mut Shell::new(&mut messages),
            &Rectangle::with_size(Size::new(420.0, 800.0)),
        );
        assert!(messages.contains(&7));
        assert_eq!(content.text(), "One\nTwo\nThree");
    }

    #[test]
    fn unrelated_branch_actions_survive_wrapper_and_multiline_presentation() {
        let renderer = renderer();
        let content = iced::widget::text_editor::Content::new();
        let row = EntryRowProps::default()
            .wrapper_props(
                InputWrapperProps::default()
                    .description_props(TextProps::default().overflow(TextOverflow::Wrap)),
            )
            .text_view_props(
                TextViewProps::default().editor_props(TextEditorProps::default().size(28)),
            )
            .apply(row(7, &content).description("Help"));
        let (mut element, mut tree, node) = layout(row, &renderer);
        fn action(layout: layout::Layout<'_>) -> Option<Rectangle> {
            if layout.bounds().size() == Size::new(137.0, 43.0) {
                Some(layout.bounds())
            } else {
                layout.children().find_map(action)
            }
        }
        let target = action(layout::Layout::new(&node)).unwrap();
        let mut messages = vec![];
        for event in [
            mouse::Event::ButtonPressed(mouse::Button::Left),
            mouse::Event::ButtonReleased(mouse::Button::Left),
        ] {
            element.as_widget_mut().update(
                &mut tree,
                &Event::Mouse(event),
                layout::Layout::new(&node),
                mouse::Cursor::Available(target.center()),
                &renderer,
                &mut clipboard::Null,
                &mut Shell::new(&mut messages),
                &Rectangle::with_size(Size::new(420.0, 800.0)),
            );
        }
        assert_eq!(messages, vec![9]);
    }
}

/// Presentation overrides for an existing EntryRow, leaving validation and input content intact.
#[derive(Clone, Default)]
pub struct EntryRowProps {
    pub dropdown_props: Option<super::popover::PopoverProps>,
    pub button_props: Option<crate::components::ButtonProps>,
    pub label_props: Option<crate::components::TextProps>,
    pub entry_props: Option<crate::components::EntryProps>,
    pub value_props: Option<TextProps>,
    pub wrapper_props: Option<InputWrapperProps>,
    pub text_view_props: Option<TextViewProps>,
    pub spacing: Option<f32>,
    pub padding: Option<iced::Padding>,
    pub value_spacing: Option<f32>,
}
impl EntryRowProps {
    pub fn dropdown_props(mut self, props: super::popover::PopoverProps) -> Self {
        self.dropdown_props = Some(props);
        self
    }
    pub fn button_props(mut self, props: crate::components::ButtonProps) -> Self {
        self.button_props = Some(props);
        self
    }
    pub fn label_props(mut self, props: crate::components::TextProps) -> Self {
        self.label_props = Some(props);
        self
    }
    pub fn entry_props(mut self, props: crate::components::EntryProps) -> Self {
        self.entry_props = Some(props);
        self
    }
    /// Override only supplied selection-value presentation fields.
    pub fn value_props(mut self, props: TextProps) -> Self {
        self.value_props = Some(props);
        self
    }
    /// Override only supplied description/validation-message presentation fields.
    pub fn wrapper_props(mut self, props: InputWrapperProps) -> Self {
        self.wrapper_props = Some(props);
        self
    }
    /// Override only supplied native multiline-editor presentation fields.
    pub fn text_view_props(mut self, props: TextViewProps) -> Self {
        self.text_view_props = Some(props);
        self
    }
    pub fn spacing(mut self, value: f32) -> Self {
        self.spacing = Some(value);
        self
    }
    pub fn padding(mut self, value: impl Into<iced::Padding>) -> Self {
        self.padding = Some(value.into());
        self
    }
    pub fn value_spacing(mut self, value: f32) -> Self {
        self.value_spacing = Some(value);
        self
    }
    pub fn apply<'a, M: 'a>(&self, mut row: EntryRow<'a, M>) -> EntryRow<'a, M> {
        if let Some(props) = &self.label_props {
            row.label_props.merge(props);
        }
        crate::components::props::merge_optional!(row, self; entry_props);
        if let Some(props) = &self.value_props {
            row.value_props.merge(props);
        }
        if let Some(props) = &self.wrapper_props {
            row.wrapper_props.label_props.merge(&props.label_props);
            row.wrapper_props
                .description_props
                .merge(&props.description_props);
            row.wrapper_props.message_props.merge(&props.message_props);
        }
        if let Some(props) = &self.text_view_props {
            row.text_view_props.editor_props.merge(&props.editor_props);
        }
        if let Some(v) = self.spacing {
            row = row.spacing(v);
        }
        if let Some(v) = self.padding {
            row = row.padding(v);
        }
        if let Some(v) = self.value_spacing {
            row = row.value_spacing(v);
        }
        if let Some(props) = &self.dropdown_props {
            row.dropdown_props.merge(props);
        }
        if let Some(props) = &self.button_props {
            row.button_props.merge(props);
        }
        row
    }
}
