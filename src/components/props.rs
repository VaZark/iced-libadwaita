//! Owned, typed presentation overrides for concrete nested widgets.
//!
//! DECISION (user, 2026-11): typed `<Widget>Props`, not a string-keyed child-style map. Each is a
//! safe subset of the nested widget's real builder props, exposed case by case; parent-owned
//! values, events, state, validation and security are never overridable. Caller-supplied
//! Element slots and custom-drawn primitives get no fake props API. Revisit if a parent needs
//! to expose more than a safe subset.
//!
//! Parents expose these case-by-case (for example `label_props(TextProps)`),
//! then apply them to the widget they already construct. There is no string-keyed
//! part lookup, widget-tree inspection, or replacement of children. Values,
//! placeholders, IDs, event handlers, and security settings remain parent-owned.
//! Every field is optional: defaults invoke no widget builder methods and add no
//! wrappers. Style callbacks are owned, cloneable, and evaluated with the current
//! theme (and button status); an explicit style replaces, rather than merges,
//! the existing widget style. Applying props never borrows the props object.
//!
//! ```
//! use iced_adaptive_design::components::{ButtonProps, TextProps, TextOverflow};
//! use iced_adaptive_design::Theme;
//! use iced::widget::{button, text};
//!
//! let label = TextProps::default().size(18).overflow(TextOverflow::Wrap)
//!     .apply::<()>(text("A long Unicode label: 日本語 🦀"));
//! let button = ButtonProps::default().padding(12)
//!     .apply(button::<(), Theme, iced::Renderer>(label));
//! ```

use std::sync::Arc;

pub use super::child_props::TextOverflow;
use super::{Entry, EntryState, child_props::apply_overflow, entry::TextInputProps};
use crate::{Element, Metrics, Theme};
use iced::widget::{Button, Container, Text, button, container, text};
use iced::{Color, Font, Length, Padding, Pixels, alignment};

pub type TextStyle = Arc<dyn Fn(&Theme) -> text::Style + Send + Sync + 'static>;
pub type ContainerStyle = Arc<dyn Fn(&Theme) -> container::Style + Send + Sync + 'static>;
pub type ButtonStyle = Arc<dyn Fn(&Theme, button::Status) -> button::Style + Send + Sync + 'static>;

// Each generated setter corresponds to an actual builder on the target widget.
macro_rules! setters {
    ($($name:ident: $ty:ty),* $(,)?) => { $(
        pub fn $name(mut self, value: impl Into<$ty>) -> Self {
            self.$name = Some(value.into());
            self
        }
    )* };
}
macro_rules! apply_fields {
    ($props:ident, $widget:ident; $($field:ident),* $(,)?) => { $(
        if let Some(value) = $props.$field { $widget = $widget.$field(value); }
    )* };
}

// Aggregate adapters overlay presentation without clearing fields the caller omitted.
macro_rules! merge_fields {
    ($target:ident, $overrides:ident; $($field:ident),* $(,)?) => { $(
        if let Some(value) = &$overrides.$field {
            $target.$field = Some(value.clone());
        }
    )* };
}
pub(crate) use merge_fields;

// Optional child objects preserve absence and recursively overlay present fields.
macro_rules! merge_optional {
    ($target:ident, $overrides:ident; $($field:ident),* $(,)?) => { $(
        if let Some(props) = &$overrides.$field && !props.is_empty() {
            match &mut $target.$field {
                Some(existing) => existing.merge(props),
                None => $target.$field = Some(props.clone()),
            }
        }
    )* };
}
pub(crate) use merge_optional;

macro_rules! impl_merge {
    ($ty:ty; fields: [$($field:ident),*]; optional: [$($optional:ident),*]; nested: [$($nested:ident),*]) => {
        impl $ty {
            pub(crate) fn is_empty(&self) -> bool {
                true $(&& self.$field.is_none())*
                    $(&& self.$optional.as_ref().is_none_or(|props| props.is_empty()))*
                    $(&& self.$nested.is_empty())*
            }

            pub(crate) fn merge(&mut self, overrides: &Self) {
                if overrides.is_empty() { return; }
                $crate::components::props::merge_fields!(self, overrides; $($field),*);
                $crate::components::props::merge_optional!(self, overrides; $($optional),*);
                $(self.$nested.merge(&overrides.$nested);)*
            }
        }
    };
}
pub(crate) use impl_merge;

/// Overrides for an existing [`Text`], preserving its content and typography baseline.
#[derive(Clone, Default)]
pub struct TextProps {
    pub size: Option<Pixels>,
    pub line_height: Option<text::LineHeight>,
    pub font: Option<Font>,
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub align_x: Option<text::Alignment>,
    pub align_y: Option<alignment::Vertical>,
    pub shaping: Option<text::Shaping>,
    pub wrapping: Option<text::Wrapping>,
    pub color: Option<Color>,
    pub style: Option<TextStyle>,
    /// If supplied, takes precedence over `wrapping`.
    pub overflow: Option<TextOverflow>,
}

impl TextProps {
    pub(crate) fn is_empty(&self) -> bool {
        self.size.is_none()
            && self.line_height.is_none()
            && self.font.is_none()
            && self.width.is_none()
            && self.height.is_none()
            && self.align_x.is_none()
            && self.align_y.is_none()
            && self.shaping.is_none()
            && self.wrapping.is_none()
            && self.color.is_none()
            && self.style.is_none()
            && self.overflow.is_none()
    }

    pub(crate) fn merge(&mut self, overrides: &Self) {
        merge_fields!(self, overrides; size, line_height, font, width, height,
            align_x, align_y, shaping, wrapping, color, style, overflow);
    }

    setters! {
        size: Pixels, line_height: text::LineHeight, font: Font,
        width: Length, height: Length, align_x: text::Alignment,
        align_y: alignment::Vertical, shaping: text::Shaping,
        wrapping: text::Wrapping, color: Color, overflow: TextOverflow,
    }
    pub fn style(mut self, style: impl Fn(&Theme) -> text::Style + Send + Sync + 'static) -> Self {
        self.style = Some(Arc::new(style));
        self
    }

    /// Configure a `Text` without wrapping it in another widget.
    /// `Clip` and `Grow` disable line wrapping here, but require [`Self::apply`]
    /// for their clipping / unconstrained-layout behavior. No string is truncated.
    /// If both style and color are supplied, color wins, matching `.style().color()`.
    pub fn apply_to_text<'a>(&self, mut widget: Text<'a, Theme>) -> Text<'a, Theme> {
        apply_fields!(self, widget; size, line_height, font, width, height, align_x, align_y, shaping, wrapping);
        if let Some(style) = self.style.clone() {
            widget = widget.style(move |theme| style(theme));
        }
        if let Some(color) = self.color {
            widget = widget.color(color);
        }
        if let Some(policy) = self.overflow {
            widget = widget.wrapping(match policy {
                TextOverflow::Wrap => text::Wrapping::WordOrGlyph,
                TextOverflow::Clip | TextOverflow::Grow => text::Wrapping::None,
            });
        }
        widget
    }

    /// Configure text, adding a wrapper only for explicitly requested Clip/Grow.
    /// Grow releases horizontal layout constraints; ancestor clipping still applies.
    pub fn apply<'a, M: 'a>(&self, widget: Text<'a, Theme>) -> Element<'a, M> {
        let widget = self.apply_to_text(widget);
        apply_overflow(widget, self.overflow)
    }
}

/// Overrides applied directly to an existing [`Container`], never around an arbitrary element.
#[derive(Clone, Default)]
pub struct ContainerProps {
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub max_width: Option<Pixels>,
    pub max_height: Option<Pixels>,
    pub padding: Option<Padding>,
    pub align_x: Option<alignment::Horizontal>,
    pub align_y: Option<alignment::Vertical>,
    pub clip: Option<bool>,
    pub style: Option<ContainerStyle>,
}

impl ContainerProps {
    pub(crate) fn is_empty(&self) -> bool {
        self.width.is_none()
            && self.height.is_none()
            && self.max_width.is_none()
            && self.max_height.is_none()
            && self.padding.is_none()
            && self.align_x.is_none()
            && self.align_y.is_none()
            && self.clip.is_none()
            && self.style.is_none()
    }

    pub(crate) fn merge(&mut self, overrides: &Self) {
        merge_fields!(self, overrides; width, height, max_width, max_height,
            padding, align_x, align_y, clip, style);
    }

    setters! {
        width: Length, height: Length, max_width: Pixels, max_height: Pixels,
        padding: Padding, align_x: alignment::Horizontal, align_y: alignment::Vertical, clip: bool,
    }
    pub fn style(
        mut self,
        style: impl Fn(&Theme) -> container::Style + Send + Sync + 'static,
    ) -> Self {
        self.style = Some(Arc::new(style));
        self
    }
    pub fn apply<'a, M: 'a>(&self, mut widget: Container<'a, M, Theme>) -> Container<'a, M, Theme> {
        apply_fields!(self, widget; width, height, max_width, max_height, padding, align_x, align_y, clip);
        if let Some(style) = self.style.clone() {
            widget = widget.style(move |theme| style(theme));
        }
        widget
    }
}

/// Overrides applied directly to an iced [`Button`]. Its action and child stay intact.
#[derive(Clone, Default)]
pub struct ButtonProps {
    pub width: Option<Length>,
    pub height: Option<Length>,
    pub padding: Option<Padding>,
    pub clip: Option<bool>,
    pub style: Option<ButtonStyle>,
}

impl ButtonProps {
    pub(crate) fn is_empty(&self) -> bool {
        self.width.is_none()
            && self.height.is_none()
            && self.padding.is_none()
            && self.clip.is_none()
            && self.style.is_none()
    }

    pub(crate) fn merge(&mut self, overrides: &Self) {
        merge_fields!(self, overrides; width, height, padding, clip, style);
    }

    setters! { width: Length, height: Length, padding: Padding, clip: bool }
    pub fn style(
        mut self,
        style: impl Fn(&Theme, button::Status) -> button::Style + Send + Sync + 'static,
    ) -> Self {
        self.style = Some(Arc::new(style));
        self
    }
    pub fn apply<'a, M: 'a>(&self, mut widget: Button<'a, M, Theme>) -> Button<'a, M, Theme> {
        apply_fields!(self, widget; width, height, padding, clip);
        if let Some(style) = self.style.clone() {
            widget = widget.style(move |theme, status| style(theme, status));
        }
        widget
    }
}

/// Presentation builders actually supported by the crate's [`Entry`].
/// No value, placeholder, password mode, events, IDs, or section children are forwarded.
/// Native text-input presentation is forwarded explicitly through `input_props`.
/// When absent, existing input props on the Entry are left untouched.
#[derive(Clone, Default)]
pub struct EntryProps {
    pub width: Option<Length>,
    pub padding: Option<Padding>,
    pub section_spacing: Option<f32>,
    pub left_section_width: Option<f32>,
    pub right_section_width: Option<f32>,
    pub flat: Option<bool>,
    pub state: Option<EntryState>,
    pub color: Option<Color>,
    pub metrics: Option<Metrics>,
    /// Field-wise overrides for the Entry's existing native input presentation.
    pub input_props: Option<TextInputProps>,
}

impl_merge!(EntryProps;
    fields: [width, padding, section_spacing, left_section_width, right_section_width, flat, state, color, metrics];
    optional: [input_props]; nested: []);

impl EntryProps {
    setters! {
        width: Length, padding: Padding, section_spacing: f32,
        left_section_width: f32, right_section_width: f32,
        flat: bool, state: EntryState, color: Color, metrics: Metrics,
        input_props: TextInputProps,
    }
    pub fn apply<'a, M>(&self, mut widget: Entry<'a, M>) -> Entry<'a, M> {
        apply_fields!(self, widget; width, padding, section_spacing, left_section_width, right_section_width, flat, state, color);
        if let Some(metrics) = self.metrics.clone() {
            widget = widget.metrics(metrics);
        }
        if let Some(input_props) = self.input_props.clone() {
            widget = super::entry::EntryInputProps { input_props }.apply(widget);
        }
        widget
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use iced::Size;
    use iced::advanced::{Widget, widget::Tree};

    fn size<M>(widget: &impl Widget<M, Theme, iced::Renderer>) -> Size<Length> {
        widget.size()
    }

    #[test]
    fn merge_helpers_preserve_absent_fields_and_replace_explicit_callbacks() {
        macro_rules! check {
            ($base:expr, $overrides:expr; $($field:ident),+ $(,)?) => {{
                let base = $base;
                let mut actual = base.clone();
                actual.merge(&Default::default());
                $(assert_eq!(actual.$field, base.$field);)+
                assert!(Arc::ptr_eq(actual.style.as_ref().unwrap(), base.style.as_ref().unwrap()));
                let overrides = $overrides;
                actual.merge(&overrides);
                $(assert_eq!(actual.$field, overrides.$field.clone().or(base.$field.clone()));)+
                assert!(Arc::ptr_eq(actual.style.as_ref().unwrap(), overrides.style.as_ref().unwrap()));
            }};
        }
        check!(
            TextProps::default().size(17).line_height(text::LineHeight::Relative(1.7))
                .font(Font::MONOSPACE).width(113).height(31)
                .align_x(text::Alignment::Center).align_y(alignment::Vertical::Bottom)
                .shaping(text::Shaping::Advanced).wrapping(text::Wrapping::WordOrGlyph)
                .color(Color::BLACK).overflow(TextOverflow::Clip)
                .style(|_| text::Style::default()),
            TextProps::default().size(21).overflow(TextOverflow::Wrap)
                .style(|_| text::Style::default());
            size, line_height, font, width, height, align_x, align_y, shaping,
            wrapping, color, overflow
        );
        check!(
            ContainerProps::default().width(137).height(43).max_width(200).max_height(100)
                .padding(7).align_x(alignment::Horizontal::Center)
                .align_y(alignment::Vertical::Bottom).clip(true)
                .style(|_| container::Style::default()),
            ContainerProps::default().height(47).clip(false)
                .style(|_| container::Style::default());
            width, height, max_width, max_height, padding, align_x, align_y, clip
        );
        check!(
            ButtonProps::default().width(137).height(43).padding(7).clip(true)
                .style(|_, _| button::Style::default()),
            ButtonProps::default().height(47).clip(false)
                .style(|_, _| button::Style::default());
            width, height, padding, clip
        );
    }

    #[test]
    fn defaults_preserve_existing_geometry_and_text_tree() {
        let label = text("日本語 🦀 e\u{301}".repeat(100)).width(117).height(31);
        let before = size::<()>(&label);
        let label = TextProps::default().apply::<()>(label);
        assert_eq!(label.as_widget().size(), before);
        assert!(Tree::new(&label).children.is_empty());

        let widget = container::<(), Theme, iced::Renderer>(text("child"))
            .width(123)
            .height(45);
        let before = size(&widget);
        let widget = ContainerProps::default().apply(widget);
        assert_eq!(size(&widget), before);

        let widget = button::<(), Theme, iced::Renderer>(text("action"))
            .width(95)
            .height(35)
            .on_press(());
        let before = size(&widget);
        let widget = ButtonProps::default().apply(widget);
        assert_eq!(size(&widget), before);

        let widget = Entry::<()>::new("placeholder", "value").width(156);
        let widget: Element<'_, ()> = EntryProps::default().apply(widget).into();
        assert_eq!(widget.as_widget().size().width, Length::Fixed(156.0));
    }

    #[test]
    fn entry_input_props_are_opt_in_and_forwarded_without_forced_defaults() {
        assert!(EntryProps::default().input_props.is_none());
        let entry =
            Entry::<()>::new("hint", "value").input_props(TextInputProps::default().width(173));
        let unchanged: Element<'_, ()> = EntryProps::default().apply(entry).into();
        assert_eq!(unchanged.as_widget().size().width, Length::Fixed(173.0));

        let entry =
            Entry::<()>::new("hint", "value").input_props(TextInputProps::default().width(173));
        let props =
            EntryProps::default().input_props(TextInputProps::default().width(211).size(18));
        let overridden: Element<'_, ()> = props.apply(entry).into();
        assert_eq!(overridden.as_widget().size().width, Length::Fixed(211.0));
    }

    #[test]
    fn custom_props_configure_concrete_widgets() {
        let props = TextProps::default()
            .width(75)
            .height(25)
            .size(19)
            .color(Color::BLACK);
        assert_eq!(props.size, Some(Pixels(19.0)));
        let widget = props.apply_to_text(text("label"));
        assert_eq!(
            size::<()>(&widget),
            Size::new(Length::Fixed(75.0), Length::Fixed(25.0))
        );

        let widget = ContainerProps::default()
            .width(120)
            .height(40)
            .padding(8)
            .clip(true)
            .apply(container::<(), Theme, iced::Renderer>(text("child")));
        assert_eq!(
            size(&widget),
            Size::new(Length::Fixed(120.0), Length::Fixed(40.0))
        );
        let widget = ButtonProps::default()
            .width(80)
            .height(30)
            .padding(6)
            .apply(button::<(), Theme, iced::Renderer>(text("action")).on_press(()));
        assert_eq!(
            size(&widget),
            Size::new(Length::Fixed(80.0), Length::Fixed(30.0))
        );
        let widget: Element<'_, ()> = EntryProps::default()
            .width(140)
            .flat(true)
            .padding(9)
            .apply(Entry::new("placeholder", "unchanged").on_input(|_| ()))
            .into();
        assert_eq!(widget.as_widget().size().width, Length::Fixed(140.0));
    }

    #[test]
    fn overflow_is_opt_in_and_text_only_path_never_adds_wrappers() {
        assert!(TextProps::default().overflow.is_none());
        for policy in [TextOverflow::Wrap, TextOverflow::Clip, TextOverflow::Grow] {
            let props = TextProps::default().overflow(policy);
            let raw: Element<'_, ()> = props.apply_to_text(text("界🦀".repeat(1000))).into();
            assert!(Tree::new(&raw).children.is_empty());
            let element = props.apply::<()>(text("界🦀".repeat(1000)));
            assert_eq!(
                Tree::new(&element).children.len(),
                usize::from(policy == TextOverflow::Grow)
            );
        }
    }

    #[test]
    fn callbacks_are_shared_owned_and_receive_theme_and_button_status() {
        let props = ButtonProps::default().style(|theme, status| {
            let mut style = button::Style {
                background: theme.tint.map(iced::Background::Color),
                ..button::Style::default()
            };
            style.text_color = if matches!(status, button::Status::Disabled) {
                Color::BLACK
            } else {
                Color::WHITE
            };
            style
        });
        let copy = props.clone();
        assert!(Arc::ptr_eq(
            props.style.as_ref().unwrap(),
            copy.style.as_ref().unwrap()
        ));
        let mut theme = Theme::default();
        let style = copy.style.unwrap();
        assert_eq!(style(&theme, button::Status::Active).background, None);
        theme.tint = Some(Color::BLACK);
        assert_eq!(
            style(&theme, button::Status::Active).background,
            Some(iced::Background::Color(Color::BLACK))
        );
        assert_eq!(
            style(&theme, button::Status::Disabled).text_color,
            Color::BLACK
        );
        assert_eq!(
            style(&theme, button::Status::Active).text_color,
            Color::WHITE
        );
        // The widget owns its cloned callback and does not borrow a temporary props value.
        let _widget = ContainerProps::default()
            .style(|_| container::Style::default())
            .apply(container::<(), Theme, iced::Renderer>(text("owned")));
    }
}
