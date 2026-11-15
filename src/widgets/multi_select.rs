//! `MultiSelect`: a dropdown that picks any number of options.
//!
//! The same popover and option rows as [`dropdown`](crate::widgets::dropdown) (they share
//! `option_items`), but the popover stays open while options are toggled and the trigger shows
//! the chosen values as [`Chip`](crate::components::Chip)-like pills, `+n` when more than
//! [`max_chips`](MultiSelect::max_chips). State is controlled: pass the current selection and
//! turn `on_toggle(option)` into your own update. The trigger's pills are display-only (the
//! trigger toggles the popover); remove a value by toggling it in the list.
//!
//! [`selecting`](MultiSelect::selecting) takes the new list instead of one toggled option,
//! computed by the shared [`Selection`](crate::primitives::Selection) rule, which also gives
//! a single-choice variant (`Selection::SINGLE`) with the same widget.

use crate::Element;
use crate::components::Badge;
use crate::components::badge::BadgeProps;
use crate::components::{ContainerProps, TextProps};
use crate::widgets::dropdown::{DropdownItemProps, option_items_with_props};
use crate::widgets::popover::Popover;
use crate::widgets::popover::{PopoverProps, PopoverSurfaceProps, popover_spaced_with_props};
use crate::{Icon, Metrics, Role};
use iced::Length;
use iced::alignment::Vertical;
use iced::widget::{container, row, text};

pub struct MultiSelect<'a, T, M> {
    pill_props: BadgeProps,
    overflow_badge_props: BadgeProps,
    label_props: TextProps,
    option_props: DropdownItemProps,
    trigger_props: ContainerProps,
    combobox_props: PopoverProps,
    dropdown_props: PopoverSurfaceProps,
    options: &'a [T],
    selected: &'a [T],
    on_toggle: Box<dyn Fn(T) -> M + 'a>,
    placeholder: String,
    max_chips: usize,
    disabled: bool,
    width: Length,
    open_initially: bool,
    spacing: Option<f32>,
    metrics: Metrics,
}

impl<'a, T, M> MultiSelect<'a, T, M> {
    pub fn new(options: &'a [T], selected: &'a [T], on_toggle: impl Fn(T) -> M + 'a) -> Self {
        Self {
            pill_props: Default::default(),
            overflow_badge_props: Default::default(),
            label_props: Default::default(),
            option_props: Default::default(),
            trigger_props: Default::default(),
            combobox_props: Default::default(),
            dropdown_props: Default::default(),
            options,
            selected,
            on_toggle: Box::new(on_toggle),
            placeholder: String::from("Select…"),
            max_chips: 3,
            disabled: false,
            width: Length::Fill,
            open_initially: false,
            spacing: None,
            metrics: Metrics::desktop(),
        }
    }
    pub fn pill_props(mut self, props: BadgeProps) -> Self {
        self.pill_props = props;
        self
    }
    pub fn overflow_badge_props(mut self, props: BadgeProps) -> Self {
        self.overflow_badge_props = props;
        self
    }
    /// Text presentation for the empty-selection placeholder.
    pub fn label_props(mut self, props: TextProps) -> Self {
        self.label_props = props;
        self
    }
    pub fn option_props(mut self, props: DropdownItemProps) -> Self {
        self.option_props = props;
        self
    }
    pub fn trigger_props(mut self, props: ContainerProps) -> Self {
        self.trigger_props = props;
        self
    }
    pub fn combobox_props(mut self, props: PopoverProps) -> Self {
        self.combobox_props = props;
        self
    }
    pub fn dropdown_props(mut self, props: PopoverSurfaceProps) -> Self {
        self.dropdown_props = props;
        self
    }
    /// Shown while nothing is selected.
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }
    /// Pills shown before the rest collapse into `+n` (default 3).
    pub fn max_chips(mut self, n: usize) -> Self {
        self.max_chips = n;
        self
    }
    pub fn disabled(mut self, d: bool) -> Self {
        self.disabled = d;
        self
    }
    pub fn width(mut self, w: impl Into<Length>) -> Self {
        self.width = w.into();
        self
    }
    /// Start with the popover open (screenshots, demos).
    pub fn open_initially(mut self, open: bool) -> Self {
        self.open_initially = open;
        self
    }
    /// The gap between the chips, and between them and the chevron (default `space_s`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, T: Clone + PartialEq + 'a, M: 'a> MultiSelect<'a, T, M> {
    /// Like [`new`](Self::new), but `on_change` receives the whole new selection, worked out by
    /// `rule` ([`Selection::Multiple`](crate::primitives::Selection) for any number).
    pub fn selecting(
        options: &'a [T],
        selected: &'a [T],
        rule: crate::primitives::Selection,
        on_change: impl Fn(Vec<T>) -> M + 'a,
    ) -> Self {
        Self::new(options, selected, move |option| {
            on_change(rule.toggle(selected, &option))
        })
    }
}

impl<'a, T, M> From<MultiSelect<'a, T, M>> for Element<'a, M>
where
    T: ToString + PartialEq + Clone + 'a,
    M: Clone + 'a,
{
    fn from(s: MultiSelect<'a, T, M>) -> Self {
        let m = s.metrics;
        let disabled = s.disabled;
        let gap = s.spacing.unwrap_or(m.space_s);
        let mut pills = row![].spacing(gap).align_y(Vertical::Center);
        if s.selected.is_empty() {
            pills = pills.push(
                s.label_props.apply(
                    text(s.placeholder)
                        .wrapping(text::Wrapping::None)
                        .style(crate::styles::text::dim),
                ),
            );
        }
        for v in s.selected.iter().take(s.max_chips) {
            pills = pills.push(
                s.pill_props
                    .apply(Badge::new(v.to_string()).role(if disabled {
                        Role::Neutral
                    } else {
                        Role::Accent
                    })),
            );
        }
        if s.selected.len() > s.max_chips {
            pills = pills.push(
                s.overflow_badge_props
                    .apply(Badge::new(format!("+{}", s.selected.len() - s.max_chips))),
            );
        }
        let chevron = Icon::GoDown
            .view(m.icon_size)
            .style(move |t: &crate::Theme, _| iced::widget::svg::Style {
                // Match the trigger foreground and disabled opacity.
                color: Some(t.semantic.icon_color(disabled)),
            });
        let trigger = container(
            row![container(pills).width(Length::Fill), chevron]
                .spacing(gap)
                .align_y(Vertical::Center),
        );
        let wraps = s.label_props.overflow == Some(crate::components::TextOverflow::Wrap)
            || s.pill_props.label_props.overflow == Some(crate::components::TextOverflow::Wrap)
            || s.overflow_badge_props.label_props.overflow
                == Some(crate::components::TextOverflow::Wrap);
        let trigger = if wraps {
            trigger
        } else {
            trigger.center_y(Length::Fixed(m.button_min_height - m.button_padding.y()))
        };
        let selected = s.selected;
        let items = option_items_with_props(
            s.options,
            |o| selected.contains(o),
            &s.on_toggle,
            &s.option_props,
        );
        s.combobox_props
            .apply(
                Popover::new(
                    s.trigger_props.apply(trigger),
                    popover_spaced_with_props(
                        items,
                        280.0,
                        m.space_xxs,
                        m.popover_padding,
                        s.dropdown_props,
                    ),
                )
                .width(s.width)
                .keep_open(true)
                .disabled(disabled)
                .open_initially(s.open_initially),
            )
            .into()
    }
}

#[cfg(test)]
mod spacing_params {
    use super::*;

    #[test]
    fn the_gap_is_a_parameter() {
        let opts = [1u8, 2];
        let m = MultiSelect::<u8, ()>::new(&opts, &opts, |_| ()).spacing(5.0);
        assert_eq!(m.spacing, Some(5.0));
        let _: Element<'_, ()> = m.into();
    }
}

#[cfg(test)]
mod typed_props_tests {
    use super::*;
    #[test]
    fn typed_pill_and_overflow_badges_are_independent_and_popover_props_are_native() {
        let values = ["界🦀".repeat(80), "other".to_owned()];
        let plain: Element<'_, ()> = MultiSelect::new(&values, &values, |_| ()).into();
        let defaults: Element<'_, ()> = MultiSelect::new(&values, &values, |_| ())
            .pill_props(BadgeProps::default())
            .overflow_badge_props(BadgeProps::default())
            .combobox_props(PopoverProps::default())
            .into();
        assert_eq!(plain.as_widget().size(), defaults.as_widget().size());
        let custom = MultiSelect::new(&values, &values, |_| ())
            .max_chips(1)
            .pill_props(
                BadgeProps::default().label_props(
                    TextProps::default()
                        .width(70)
                        .overflow(crate::components::TextOverflow::Wrap),
                ),
            )
            .overflow_badge_props(BadgeProps::default().padding(9))
            .combobox_props(PopoverProps::default().width(191));
        assert_eq!(
            custom.pill_props.label_props.width,
            Some(Length::Fixed(70.0))
        );
        assert_eq!(
            custom.overflow_badge_props.padding,
            Some(iced::Padding::from(9))
        );
        let custom: Element<'_, ()> = custom.into();
        assert_eq!(custom.as_widget().size().width, Length::Fixed(191.0));
    }
}
