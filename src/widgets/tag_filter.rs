//! `TagFilter`: a tags entry with the vocabulary as toggle chips below it. Picked tags become
//! pills in the entry, and typing in the entry narrows the chips (find-as-you-type).
//!
//! By default it edits **one group**: a track needs every picked tag. With
//! [`multiple(true)`](TagFilter::multiple) it edits several **groups** that are alternatives: a
//! track needs every tag of at least one group (`a` and `b`, or `c`). The pills of a group sit
//! together and groups are separated by commas. Click a group to edit it, press the arrow keys at the
//! edges of the field to move between groups, and press the add button to finish the group and
//! start a new one.
//!
//! The application owns the [`TagGroups`] and the query ([controlled](crate::widgets)) and
//! hands the events to [`TagGroups::apply`]:
//!
//! ```ignore
//! TagFilter::new(&self.groups, tags.iter().map(|t| ChipOption::new(t.id, &t.name)),
//!                &self.query, Msg::Groups, Msg::Query)
//!     .multiple(true)                              // omit for a single group
//! // update
//! Msg::Groups(event) => self.groups.apply(event),
//! Msg::Query(q) => self.query = q,
//! // filter
//! self.groups.matches(&track.tags)
//! ```
use super::{ChipGroup, ChipOption, EntryInput, EntryRow};
use crate::binds::token_nav::between_groups;
use crate::components::{Chip, ChipSize, TooltipPosition, section_button, tooltip, typography as ty};
use crate::layouts::{token_flow, token_flow_with_caret};
use crate::primitives::tag_groups::{GroupEvent, Step, TagGroups};
use crate::{Element, Icon, Metrics};
use iced::widget::{column, mouse_area, row, text_input};
use iced::{Alignment, Length};
use std::rc::Rc;

/// See the [module](self).
pub struct TagFilter<'a, K, M> {
    groups: &'a TagGroups<K>,
    options: Vec<ChipOption<K>>,
    query: &'a str,
    on_change: Rc<dyn Fn(GroupEvent<K>) -> M + 'a>,
    on_query: Rc<dyn Fn(String) -> M + 'a>,
    multiple: bool,
    label: String,
    placeholder: String,
    empty: String,
    max_lines: Option<usize>,
    flat: bool,
    metrics: Metrics,
}

impl<'a, K: PartialEq + Clone + ToString + 'a, M: Clone + 'a> TagFilter<'a, K, M> {
    /// `options` is every tag on offer, `groups` what is picked and `query` what the field holds.
    pub fn new(
        groups: &'a TagGroups<K>,
        options: impl IntoIterator<Item = ChipOption<K>>,
        query: &'a str,
        on_change: impl Fn(GroupEvent<K>) -> M + 'a,
        on_query: impl Fn(String) -> M + 'a,
    ) -> Self {
        Self {
            groups,
            options: options.into_iter().collect(),
            query,
            on_change: Rc::new(on_change),
            on_query: Rc::new(on_query),
            multiple: false,
            label: "Tags".into(),
            placeholder: "Find a tag".into(),
            empty: "No matching tags".into(),
            max_lines: None,
            flat: false,
            metrics: Metrics::desktop(),
        }
    }
    /// Several alternative groups (true) or a single one (false, the default). A single-group
    /// filter edits the first group only, and shows no group controls.
    pub fn multiple(mut self, on: bool) -> Self {
        self.multiple = on;
        self
    }
    /// The row's label (default "Tags").
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }
    /// What the empty field says (default "Find a tag").
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }
    /// What shows when the query matches no tag.
    pub fn empty_text(mut self, text: impl Into<String>) -> Self {
        self.empty = text.into();
        self
    }
    /// Limits the chips to this many lines.
    pub fn max_lines(mut self, lines: usize) -> Self {
        self.max_lines = Some(lines);
        self
    }
    /// Transparent entry with flat row styling, for use outside a boxed list.
    pub fn flat(mut self, flat: bool) -> Self {
        self.flat = flat;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }

    /// The group being edited: always the first when single.
    fn active(&self) -> usize {
        if self.multiple { self.groups.active() } else { 0 }
    }
    fn pill(&self, group: usize, key: &K) -> Element<'a, M> {
        let option = self.options.iter().find(|o| o.key == *key);
        let label = option.map_or_else(|| key.to_string(), |o| o.label.clone());
        let on_change = self.on_change.clone();
        let mut chip = Chip::new(label)
            .size(ChipSize::Medium)
            .metrics(self.metrics)
            .on_remove(on_change(GroupEvent::Toggle(group, key.clone())));
        if let Some(icon) = option.and_then(|o| o.icon.clone()) {
            chip = chip.icon(icon);
        }
        if let Some(color) = option.and_then(|o| o.color) {
            chip = chip.color(color);
        }
        chip.into()
    }
    /// The pills of the entry in order, and where the field goes among them.
    fn pills(&self) -> (Vec<Element<'a, M>>, usize) {
        let active = self.active();
        let shown = if self.multiple { self.groups.groups().len() } else { 1 };
        let mut items: Vec<Element<'a, M>> = Vec::new();
        let mut caret = 0;
        for (index, group) in self.groups.groups().iter().take(shown).enumerate() {
            let before_filled = index > 0 && !self.groups.groups()[index - 1].is_empty();
            if before_filled && (!group.is_empty() || index == active) {
                items.push(ty::label(",").into());
            }
            if !group.is_empty() {
                let cluster: Element<'a, M> = row(group.iter().map(|key| self.pill(index, key)))
                    .spacing(4)
                    .align_y(Alignment::Center)
                    .wrap()
                    .into();
                items.push(if self.multiple {
                    mouse_area(cluster)
                        .on_press((self.on_change)(GroupEvent::Select(index)))
                        .into()
                } else {
                    cluster
                });
            }
            if index == active {
                caret = items.len();
            }
        }
        (items, caret)
    }
}

impl<'a, K: PartialEq + Clone + ToString + 'a, M: Clone + 'a> From<TagFilter<'a, K, M>>
    for Element<'a, M>
{
    fn from(f: TagFilter<'a, K, M>) -> Self {
        let m = f.metrics;
        let active = f.active();
        let (mut items, caret) = f.pills();
        let on_query = f.on_query.clone();
        let field = text_input(
            if items.is_empty() { &f.placeholder } else { "" },
            f.query,
        )
        .on_input(move |q| on_query(q))
        .style(crate::styles::text_input::embedded)
        .padding([4, 2])
        .size(14)
        .width(Length::Fill);
        items.push(field.into());
        let input = if f.multiple {
            token_flow_with_caret(
                items,
                caret,
                // Empty, the field is just a slot for the caret between the pills; typing widens it.
                if f.query.is_empty() { 16.0 } else { 120.0 },
                between_groups(
                    f.query,
                    Some((f.on_change)(GroupEvent::Step(Step::Previous))),
                    Some((f.on_change)(GroupEvent::Step(Step::Next))),
                ),
            )
        } else {
            token_flow(items)
        };
        let mut line = row![input].spacing(4).align_y(Alignment::Center);
        if f.multiple {
            line = line.push(tooltip(
                section_button(Icon::ListAdd, (f.on_change)(GroupEvent::AddGroup)).metrics(m),
                "Finish this group and start another",
                TooltipPosition::Bottom,
            ));
        }
        let entry = EntryRow::new(f.label.clone(), EntryInput::Custom(line.into()))
            .flat(f.flat)
            .metrics(m);
        let on_change = f.on_change.clone();
        let mut chips = ChipGroup::new(f.options, f.groups.groups()[active].as_slice(), move |key| {
            on_change(GroupEvent::Toggle(active, key))
        })
        .query(f.query)
        .empty_text(f.empty)
        .metrics(m);
        if let Some(lines) = f.max_lines {
            chips = chips.max_lines(lines);
        }
        column![Element::from(entry), Element::from(chips)]
            .spacing(8)
            .width(Length::Fill)
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options() -> Vec<ChipOption<&'static str>> {
        ["rock", "jazz", "calm"]
            .into_iter()
            .map(|t| ChipOption::new(t, t))
            .collect()
    }

    #[test]
    fn it_builds_single_and_multiple_with_any_selection() {
        let mut groups = TagGroups::new();
        for multiple in [false, true] {
            for _ in 0..2 {
                let _: Element<'_, GroupEvent<&str>> = TagFilter::new(
                    &groups,
                    options(),
                    "ro",
                    |e| e,
                    |_| GroupEvent::AddGroup,
                )
                .multiple(multiple)
                .into();
                groups.toggle("rock");
                groups.add_group();
                groups.toggle("calm");
            }
        }
    }

    #[test]
    fn a_single_filter_shows_the_first_group_only_and_has_no_separators() {
        let mut groups = TagGroups::new();
        groups.toggle("rock");
        groups.add_group();
        groups.toggle("jazz");
        let single = TagFilter::new(&groups, options(), "", |e| e, |_| GroupEvent::AddGroup);
        let (pills, caret) = single.pills();
        assert_eq!((pills.len(), caret), (1, 1));
        let multiple = TagFilter::new(&groups, options(), "", |e| e, |_| GroupEvent::AddGroup)
            .multiple(true);
        let (pills, caret) = multiple.pills();
        // group, comma, group; the field follows the active (last) group
        assert_eq!((pills.len(), caret), (3, 3));
    }

    #[test]
    fn the_field_sits_after_the_active_group() {
        let mut groups = TagGroups::new();
        groups.toggle("rock");
        groups.add_group();
        groups.toggle("jazz");
        groups.select(0);
        let f = TagFilter::new(&groups, options(), "", |e| e, |_| GroupEvent::AddGroup)
            .multiple(true);
        assert_eq!(f.pills().1, 1);
    }
}
