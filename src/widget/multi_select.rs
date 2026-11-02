//! `MultiSelect`: a dropdown that picks any number of options.
//!
//! The same popover and option rows as [`dropdown`](super::dropdown) (they share
//! `option_items`), but the popover stays open while options are toggled and the trigger shows
//! the chosen values as [`Chip`](super::Chip)-like pills, `+n` when more than
//! [`max_chips`](MultiSelect::max_chips). State is controlled: pass the current selection and
//! turn `on_toggle(option)` into your own update. The trigger's pills are display-only (the
//! trigger toggles the popover); remove a value by toggling it in the list.
//!
//! [`selecting`](MultiSelect::selecting) takes the new list instead of one toggled option,
//! computed by the shared [`Selection`](crate::primitives::Selection) rule, which also gives
//! a single-choice variant (`Selection::SINGLE`) with the same widget.

use super::dropdown::{Dropdown, option_items, popover};
use super::{Badge, Element};
use crate::{Icon, Metrics, Role};
use iced::Length;
use iced::alignment::Vertical;
use iced::widget::{container, row, text};

pub struct MultiSelect<'a, T, M> {
    options: &'a [T],
    selected: &'a [T],
    on_toggle: Box<dyn Fn(T) -> M + 'a>,
    placeholder: String,
    max_chips: usize,
    disabled: bool,
    width: Length,
    open_initially: bool,
    metrics: Metrics,
}

impl<'a, T, M> MultiSelect<'a, T, M> {
    pub fn new(options: &'a [T], selected: &'a [T], on_toggle: impl Fn(T) -> M + 'a) -> Self {
        Self {
            options,
            selected,
            on_toggle: Box::new(on_toggle),
            placeholder: String::from("Select…"),
            max_chips: 3,
            disabled: false,
            width: Length::Fill,
            open_initially: false,
            metrics: Metrics::desktop(),
        }
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
        let mut pills = row![].spacing(m.space_s).align_y(Vertical::Center);
        if s.selected.is_empty() {
            pills = pills.push(
                text(s.placeholder)
                    .wrapping(text::Wrapping::None)
                    .style(crate::style::text::dim),
            );
        }
        for v in s.selected.iter().take(s.max_chips) {
            pills = pills.push(Badge::new(v.to_string()).role(if disabled {
                Role::Neutral
            } else {
                Role::Accent
            }));
        }
        if s.selected.len() > s.max_chips {
            pills = pills.push(Badge::new(format!("+{}", s.selected.len() - s.max_chips)));
        }
        let chevron = Icon::GoDown
            .view(m.icon_size)
            .style(move |t: &crate::Theme, _| iced::widget::svg::Style {
                // Always a color: `None` would draw the chevron black, invisible in dark.
                color: Some(t.semantic.icon_solid(disabled)),
            });
        let trigger = container(
            row![container(pills).width(Length::Fill), chevron]
                .spacing(m.space_s)
                .align_y(Vertical::Center),
        )
        .center_y(Length::Fixed(m.button_min_height - m.button_padding.y()));
        let selected = s.selected;
        let items = option_items(s.options, |o| selected.contains(o), &s.on_toggle);
        Dropdown::custom(trigger, popover(items, 280.0))
            .width(s.width)
            .keep_open(true)
            .disabled(disabled)
            .open_initially(s.open_initially)
            .into()
    }
}
