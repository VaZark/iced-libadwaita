//! Split buttons: a button joined to a menu arrow.
//!
//! * [`SplitButton`] (`AdwSplitButton`): a main action plus a menu of *other* actions. Pressing
//!   the button runs its action; pressing the arrow opens the menu and choosing an item runs
//!   that item.
//! * [`SplitButtonAction`]: the same shape where the menu **chooses what the button does**. The
//!   button shows the selected item and runs it; picking another item in the menu changes the
//!   selection (and so the button's label and action) without running anything. This is the
//!   "Merge / Squash / Rebase" pattern.
//!
//! Both are linked groups (`.linked`): the button and the arrow share their inner corners and are
//! separated by a hairline, like a two-segment [`ToggleGroup`](crate::components::ToggleGroup).

use crate::Element;
use crate::components::ButtonContent;
use crate::components::button::sized;
use crate::components::{ButtonProps, ContainerProps, TextProps};
use crate::styles::button::{self as sb, Position};
use crate::widgets::Menu;
use crate::widgets::popover::{Align, Appearance, Popover, popover};
use crate::{Icon, Metrics, Theme};
use iced::Length;
use iced::widget::{container, row, space};

/// The arrow half of a split button: a linked-end dropdown that opens `popover`.
fn joined<'a, M: Clone + 'a>(
    main: Element<'a, M>,
    popover: Element<'a, M>,
    disabled: bool,
    open: bool,
    m: &Metrics,
    arrow_props: &ContainerProps,
    separator_props: &ContainerProps,
) -> Element<'a, M> {
    let arrow = arrow_props.apply(
        container(Icon::GoDown.view(m.icon_size).style(move |t: &Theme, _| {
            iced::widget::svg::Style {
                color: Some(if disabled {
                    crate::color::fade(t.semantic.window.fg, t.semantic.opacity.disabled)
                } else {
                    t.semantic.window.fg
                }),
            }
        }))
        .center_y(Length::Fixed(m.button_min_height - m.button_padding.y())),
    );
    let menu = Popover::new(arrow, popover)
        .appearance(Appearance::Linked(Position::End))
        .align(Align::End)
        .popover_width(220.0)
        .disabled(disabled)
        .open_initially(open);
    row![
        main,
        separator_props.apply(
            container(space().width(1).height(Length::Fill))
                .style(crate::styles::container::bar_separator)
        ),
        menu,
    ]
    .height(Length::Shrink)
    .into()
}

/// `AdwSplitButton`: a button with a menu of other actions.
pub struct SplitButton<'a, M> {
    content: Element<'a, M>,
    on_press: M,
    menu: Menu<M>,
    disabled: bool,
    open: bool,
    metrics: Metrics,
    main_button_props: ButtonProps,
    arrow_props: ContainerProps,
    separator_props: ContainerProps,
}

/// A [`SplitButton`] with `content` as its label, `on_press` as its action and `menu` behind the
/// arrow.
pub fn split_button<'a, M: Clone + 'a>(
    content: impl Into<Element<'a, M>>,
    on_press: M,
    menu: Menu<M>,
) -> SplitButton<'a, M> {
    SplitButton::new(content, on_press, menu)
}

impl<'a, M: Clone + 'a> SplitButton<'a, M> {
    /// Presentation overrides applied after the child's defaults.
    pub fn main_button_props(mut self, props: ButtonProps) -> Self {
        self.main_button_props = props;
        self
    }
    pub fn arrow_props(mut self, props: ContainerProps) -> Self {
        self.arrow_props = props;
        self
    }
    pub fn separator_props(mut self, props: ContainerProps) -> Self {
        self.separator_props = props;
        self
    }
    pub fn new(content: impl Into<Element<'a, M>>, on_press: M, menu: Menu<M>) -> Self {
        Self {
            arrow_props: ContainerProps::default(),
            separator_props: ContainerProps::default(),
            main_button_props: ButtonProps::default(),
            content: content.into(),
            on_press,
            menu,
            disabled: false,
            open: false,
            metrics: Metrics::desktop(),
        }
    }
    /// Insensitive: the button is drawn disabled and the menu cannot open.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Start with the menu open (screenshots, demos).
    pub fn open_initially(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: Clone + 'a> From<SplitButton<'a, M>> for Element<'a, M> {
    fn from(s: SplitButton<'a, M>) -> Self {
        let m = s.metrics;
        let main = sized(s.content, &m).style(sb::linked(Position::Start));
        let main = s.main_button_props.apply(main);
        let main = if s.disabled {
            main
        } else {
            main.on_press(s.on_press)
        };
        let button = joined(
            main.into(),
            crate::widgets::menu::popover_content(&s.menu, 360.0),
            s.disabled,
            s.open,
            &m,
            &s.arrow_props,
            &s.separator_props,
        );
        // Like a menu button, it registers the accelerators its menu shows.
        crate::shortcut::Shortcuts::new()
            .from_menu(&s.menu)
            .scope(button)
    }
}

/// One choice of a [`SplitButtonAction`]: a value, the label the button shows when it is
/// selected, and an optional icon.
pub struct SplitAction<T> {
    value: T,
    label: String,
    icon: Option<Icon>,
    disabled: bool,
}

impl<T> SplitAction<T> {
    pub fn new(value: T, label: impl Into<String>) -> Self {
        Self {
            value,
            label: label.into(),
            icon: None,
            disabled: false,
        }
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    /// Not available: shown dimmed in the menu and cannot be selected.
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

type Choice<'a, T, M> = Box<dyn Fn(T) -> M + 'a>;

/// A split button whose action follows the selected menu item.
///
/// The app owns the selection: pass the selected value, and update it from `on_select`.
pub struct SplitButtonAction<'a, T, M> {
    actions: Vec<SplitAction<T>>,
    selected: T,
    on_select: Choice<'a, T, M>,
    on_press: Choice<'a, T, M>,
    disabled: bool,
    open: bool,
    metrics: Metrics,
    main_button_props: ButtonProps,
    arrow_props: ContainerProps,
    separator_props: ContainerProps,
    label_props: TextProps,
    option_props: super::dropdown::DropdownItemProps,
}

impl<'a, T: Clone + PartialEq + 'a, M: Clone + 'a> SplitButtonAction<'a, T, M> {
    /// `on_select(value)` is sent when a menu item is chosen (change the selection, do not run
    /// it); `on_press(value)` when the button is pressed, with the selected value.
    /// Presentation overrides applied after the child's defaults.
    pub fn main_button_props(mut self, props: ButtonProps) -> Self {
        self.main_button_props = props;
        self
    }
    pub fn arrow_props(mut self, props: ContainerProps) -> Self {
        self.arrow_props = props;
        self
    }
    pub fn separator_props(mut self, props: ContainerProps) -> Self {
        self.separator_props = props;
        self
    }
    pub fn label_props(mut self, props: TextProps) -> Self {
        self.label_props = props;
        self
    }
    pub fn option_props(mut self, props: super::dropdown::DropdownItemProps) -> Self {
        self.option_props = props;
        self
    }
    pub fn new(
        actions: impl IntoIterator<Item = SplitAction<T>>,
        selected: T,
        on_select: impl Fn(T) -> M + 'a,
        on_press: impl Fn(T) -> M + 'a,
    ) -> Self {
        Self {
            arrow_props: ContainerProps::default(),
            separator_props: ContainerProps::default(),
            label_props: TextProps::default(),
            option_props: super::dropdown::DropdownItemProps::default(),
            main_button_props: ButtonProps::default(),
            actions: actions.into_iter().collect(),
            selected,
            on_select: Box::new(on_select),
            on_press: Box::new(on_press),
            disabled: false,
            open: false,
            metrics: Metrics::desktop(),
        }
    }
    /// Insensitive: the button is drawn disabled and the menu cannot open.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    /// Start with the menu open (screenshots, demos).
    pub fn open_initially(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, T: Clone + PartialEq + 'a, M: Clone + 'a> From<SplitButtonAction<'a, T, M>>
    for Element<'a, M>
{
    fn from(s: SplitButtonAction<'a, T, M>) -> Self {
        let m = s.metrics;
        // The button shows the selected action (the first if the selection is not in the list).
        let current = s
            .actions
            .iter()
            .find(|a| a.value == s.selected)
            .or_else(|| s.actions.first());
        let label = current.map(|a| a.label.clone()).unwrap_or_default();
        let mut content = ButtonContent::new(label)
            .metrics(m)
            .label_props(s.label_props);
        if let Some(icon) = current.and_then(|a| a.icon) {
            content = content.icon(icon);
        }
        let main = sized(Element::from(content), &m).style(sb::linked(Position::Start));
        let main = s.main_button_props.apply(main);
        let main = if s.disabled || current.is_none() {
            main
        } else {
            main.on_press((s.on_press)(
                current.map(|a| a.value.clone()).expect("checked above"),
            ))
        };
        let items = s.actions.iter().map(|a| {
            let selected = a.value == s.selected;
            if a.disabled {
                super::dropdown::menu_item_disabled_with_props(
                    a.label.clone(),
                    selected,
                    s.option_props.clone(),
                )
            } else {
                super::dropdown::menu_item_with_props(
                    a.label.clone(),
                    selected,
                    (s.on_select)(a.value.clone()),
                    s.option_props.clone(),
                )
            }
        });
        let items: Vec<Element<'a, M>> = items.collect();
        joined(
            main.into(),
            popover(items, 280.0),
            s.disabled,
            s.open,
            &m,
            &s.arrow_props,
            &s.separator_props,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, PartialEq, Debug)]
    enum Msg {
        Press(u8),
        Select(u8),
    }

    fn actions() -> Vec<SplitAction<u8>> {
        vec![
            SplitAction::new(1, "One"),
            SplitAction::new(2, "Two").disabled(),
        ]
    }

    #[test]
    fn action_builds_for_any_selection_even_an_unknown_one() {
        for selected in [1u8, 2, 9] {
            let _: Element<'_, Msg> =
                SplitButtonAction::new(actions(), selected, Msg::Select, Msg::Press).into();
        }
        let _: Element<'_, Msg> =
            SplitButtonAction::new(Vec::new(), 0u8, Msg::Select, Msg::Press).into();
    }

    #[test]
    fn split_button_builds_with_and_without_items() {
        let _: Element<'_, Msg> =
            split_button(iced::widget::text("Go"), Msg::Press(0), Menu::new("m")).into();
        let _: Element<'_, Msg> =
            SplitButton::new(iced::widget::text("Go"), Msg::Press(0), Menu::new("m"))
                .disabled(true)
                .into();
    }
}
