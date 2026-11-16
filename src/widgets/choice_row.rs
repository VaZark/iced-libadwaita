//! `ChoiceRow`: a row that picks one of several options. While the row is wide enough it is a
//! [`ToggleGroupRow`] (every option visible); in a narrow container it becomes a [`ComboRow`]
//! (a dropdown), so many options never collapse or overflow.
//!
//! It is a [`MetaComponent`] with a breakpoint: two presentations of one piece of content, chosen
//! by the width of the container the row sits in.
//!
//! ```ignore
//! ChoiceRow::new("Style", &STYLES, Some(self.style), Msg::Style)
//!     .subtitle("How the button looks")
//!     .icons(|s| s.icon())        // optional: an icon per option
//! ```
//!
//! The width the toggle group needs is estimated from the labels; set it with
//! [`min_width`](ChoiceRow::min_width) when the estimate is off.

use crate::components::ToggleItem;
use crate::metas::MetaComponent;
use crate::primitives::When;
use crate::widgets::{ComboRow, ToggleGroupRow};
use crate::{Element, Icon, Metrics, Typography};
use iced::Length;
use std::rc::Rc;

/// Which presentation a [`ChoiceRow`] has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChoicePresentation {
    /// A toggle group: every option is visible.
    Toggles,
    /// A dropdown, for a narrow container.
    Dropdown,
}

type IconOf<'a, T> = Rc<dyn Fn(&T) -> Option<Icon> + 'a>;

/// Compatibility names for the native nested row presentation props.
pub type ChoiceToggleProps = crate::widgets::rows::ToggleGroupRowProps;
pub type ChoiceDropdownProps = crate::widgets::rows::ComboRowProps;

/// See the [module](self).
pub struct ChoiceRow<'a, T, M> {
    toggle_props: crate::widgets::rows::ToggleGroupRowProps,
    dropdown_props: crate::widgets::rows::ComboRowProps,
    title: String,
    subtitle: Option<String>,
    options: &'a [T],
    selected: Option<T>,
    on_select: Rc<dyn Fn(T) -> M + 'a>,
    icons: Option<IconOf<'a, T>>,
    min_width: Option<f32>,
    metrics: Metrics,
}

impl<'a, T, M> ChoiceRow<'a, T, M>
where
    T: ToString + PartialEq + Clone + 'a,
    M: Clone + 'a,
{
    pub fn new(
        title: impl Into<String>,
        options: &'a [T],
        selected: Option<T>,
        on_select: impl Fn(T) -> M + 'a,
    ) -> Self {
        Self {
            toggle_props: Default::default(),
            dropdown_props: Default::default(),
            title: title.into(),
            subtitle: None,
            options,
            selected,
            on_select: Rc::new(on_select),
            icons: None,
            min_width: None,
            metrics: Metrics::desktop(),
        }
    }
    pub fn toggle_props(mut self, props: crate::widgets::rows::ToggleGroupRowProps) -> Self {
        self.toggle_props = props;
        self
    }
    pub fn dropdown_props(mut self, props: crate::widgets::rows::ComboRowProps) -> Self {
        self.dropdown_props = props;
        self
    }
    /// Apply the same native action-row presentation to both variants.
    pub fn row_props(mut self, props: crate::widgets::rows::ActionRowProps) -> Self {
        self.toggle_props.row_props = Some(props.clone());
        self.dropdown_props.row_props = Some(props);
        self
    }
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.subtitle = Some(s.into());
        self
    }
    /// An icon per option, shown in the toggle group (the dropdown shows labels only).
    pub fn icons(mut self, f: impl Fn(&T) -> Option<Icon> + 'a) -> Self {
        self.icons = Some(Rc::new(f));
        self
    }
    /// The container width from which the toggle group is used (default: estimated from the
    /// title and the labels).
    pub fn min_width(mut self, px: f32) -> Self {
        self.min_width = Some(px.max(0.0));
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }

    /// The width the toggle group needs: the title, then each option's label, icon and padding.
    pub fn needed_width(&self) -> f32 {
        if let Some(w) = self.min_width {
            return w;
        }
        let m = &self.metrics;
        let glyph = Typography::desktop().base * 0.56;
        let text = |s: &str| s.chars().count() as f32 * glyph;
        let icon = if self.icons.is_some() {
            m.icon_size + m.space_xs
        } else {
            0.0
        };
        let toggles: f32 = self
            .options
            .iter()
            .map(|o| text(&o.to_string()) + icon + 2.0 * m.button_padding.left)
            .sum();
        let gaps = self.options.len().saturating_sub(1) as f32 * m.space_xxs;
        text(&self.title) + m.row_spacing + toggles + gaps + 2.0 * m.row_padding.left
    }

    /// The presentation a container of `width` gets.
    pub fn presentation(&self, width: f32) -> ChoicePresentation {
        if When::min_width(self.needed_width())
            .holds(width, &crate::breakpoints::Breakpoints::current())
        {
            ChoicePresentation::Toggles
        } else {
            ChoicePresentation::Dropdown
        }
    }

    /// The meta component this row is: the toggle group while it fits, a dropdown otherwise.
    pub fn into_meta(self) -> MetaComponent<'a, ChoicePresentation, M> {
        let needed = self.needed_width();
        let height = Length::Fixed(self.metrics.row_min_height);
        let this = Rc::new(self);
        let toggles = this.clone();
        let combo = this;
        MetaComponent::new()
            .width(Length::Fill)
            .height(height)
            .variant(
                ChoicePresentation::Toggles,
                When::min_width(needed),
                move |_| toggles.toggles(),
            )
            .variant(ChoicePresentation::Dropdown, When::always(), move |_| {
                combo.dropdown()
            })
    }

    fn toggles(&self) -> Element<'a, M> {
        let items = self
            .options
            .iter()
            .map(|o| {
                let mut item = ToggleItem::new(o.clone(), o.to_string());
                if let Some(icon) = self.icons.as_ref().and_then(|f| f(o)) {
                    item = item.icon(icon);
                }
                item
            })
            .collect();
        let on = self.on_select.clone();
        let mut row =
            ToggleGroupRow::new(self.title.clone(), items, self.selected.clone(), move |v| {
                on(v)
            })
            .metrics(self.metrics);
        row = self.toggle_props.apply(row);
        if let Some(s) = &self.subtitle {
            row = row.subtitle(s.clone());
        }
        row.into()
    }

    fn dropdown(&self) -> Element<'a, M> {
        let on = self.on_select.clone();
        let mut row = ComboRow::new(
            self.title.clone(),
            self.options,
            self.selected.clone(),
            move |v| on(v),
        );
        row = self.dropdown_props.apply(row);
        if let Some(s) = &self.subtitle {
            row = row.subtitle(s.clone());
        }
        row.into()
    }
}

impl<'a, T, M> From<ChoiceRow<'a, T, M>> for Element<'a, M>
where
    T: ToString + PartialEq + Clone + 'a,
    M: Clone + 'a,
{
    fn from(row: ChoiceRow<'a, T, M>) -> Self {
        row.into_meta().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STYLES: [&str; 6] = [
        "Default",
        "Flat",
        "Suggested",
        "Destructive",
        "Pill",
        "Suggested pill",
    ];

    fn row() -> ChoiceRow<'static, &'static str, u8> {
        ChoiceRow::new("Style", &STYLES, Some("Flat"), |_| 0)
    }

    #[test]
    fn many_options_need_more_width_than_few() {
        let few = ChoiceRow::<&str, u8>::new("Style", &STYLES[..2], None, |_| 0);
        assert!(row().needed_width() > few.needed_width());
        assert!(row().icons(|_| Some(Icon::Check)).needed_width() > row().needed_width());
    }

    #[test]
    fn a_narrow_container_gets_the_dropdown_and_a_wide_one_the_toggles() {
        let r = row();
        let needed = r.needed_width();
        assert_eq!(r.presentation(needed - 1.0), ChoicePresentation::Dropdown);
        assert_eq!(r.presentation(needed), ChoicePresentation::Toggles);
        assert_eq!(
            row().min_width(300.0).presentation(299.0),
            ChoicePresentation::Dropdown
        );
        assert_eq!(
            row().min_width(300.0).presentation(300.0),
            ChoicePresentation::Toggles
        );
    }

    #[test]
    fn it_builds_both_presentations_and_as_an_element() {
        let r = row().subtitle("How it looks");
        let _ = r.toggles();
        let _ = r.dropdown();
        let _: Element<'_, u8> = r.into();
    }
}

#[cfg(test)]
mod typed_props_tests {
    use super::*;
    use crate::components::{ButtonProps, TextOverflow, TextProps};
    use crate::widgets::rows::{ActionRowProps, ComboRowProps, ToggleGroupRowProps};

    #[test]
    fn choice_uses_native_row_props_without_changing_default_presentation() {
        let options = ["界🦀e\u{301}".repeat(80), "other".to_owned()];
        let original = ChoiceRow::new("Title", &options, None, |_| ());
        let defaults = ChoiceRow::new("Title", &options, None, |_| ())
            .toggle_props(ToggleGroupRowProps::default())
            .dropdown_props(ComboRowProps::default());
        assert_eq!(original.needed_width(), defaults.needed_width());
        assert_eq!(
            original.toggles().as_widget().size(),
            defaults.toggles().as_widget().size()
        );
        assert_eq!(
            original.dropdown().as_widget().size(),
            defaults.dropdown().as_widget().size()
        );
        let custom = ChoiceRow::new("Title", &options, None, |_| ())
            .row_props(
                ActionRowProps::default()
                    .label_props(TextProps::default().width(80).overflow(TextOverflow::Wrap))
                    .button_props(ButtonProps::default().width(193)),
            )
            .dropdown_props(ComboRowProps::default().value_spacing(17.0_f32));
        assert_eq!(
            custom
                .toggle_props
                .row_props
                .as_ref()
                .unwrap()
                .button_props
                .as_ref()
                .unwrap()
                .width,
            Some(Length::Fixed(193.0))
        );
        let _ = custom.toggles();
        let _ = custom.dropdown();
        assert_eq!(custom.dropdown_props.value_spacing, Some(17.0));
    }
}
