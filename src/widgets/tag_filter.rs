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
//! An app whose tags live in a richer model than `TagGroups` builds the groups for the view
//! ([`owned`](TagFilter::owned)) and maps the events back to its own messages. For that the
//! entry takes read-only pills ([`before`](TagFilter::before), [`after`](TagFilter::after)), a
//! per-boundary [`separator`](TagFilter::separator), extra controls
//! ([`leading`](TagFilter::leading), [`trailing`](TagFilter::trailing),
//! [`add_group_button`](TagFilter::add_group_button)), pill styles for tags that are not among the
//! options ([`picked`](TagFilter::picked)), a field id, and its own list of
//! [`choices`](TagFilter::choices) under the entry.
//!
//! A [`description`](TagFilter::description) goes under the field, as on any row. The application owns the [`TagGroups`] and the query ([controlled](crate::widgets)) and
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
use crate::components::{
    Chip, ChipSize, TooltipPosition, section_button, tooltip, typography as ty,
};
use crate::layouts::{token_flow, token_flow_with_caret};
use crate::primitives::tag_groups::{GroupEvent, Step, TagGroups};
use crate::{Element, Icon, Metrics};
use iced::widget::{column, mouse_area, row, space, text_input};
use iced::{Alignment, Length};
use std::borrow::Cow;
use std::rc::Rc;

/// See the [module](self).
pub struct TagFilter<'a, K: Clone, M> {
    groups: Cow<'a, TagGroups<K>>,
    options: Vec<ChipOption<K>>,
    query: &'a str,
    on_change: Rc<dyn Fn(GroupEvent<K>) -> M + 'a>,
    on_query: Rc<dyn Fn(String) -> M + 'a>,
    multiple: bool,
    label: String,
    description: Option<String>,
    placeholder: String,
    empty: String,
    max_lines: Option<usize>,
    flat: bool,
    metrics: Metrics,
    picked: Vec<ChipOption<K>>,
    field_id: Option<iced::widget::Id>,
    before: Vec<Element<'a, M>>,
    after: Vec<Element<'a, M>>,
    leading: Option<Element<'a, M>>,
    trailing: Option<Element<'a, M>>,
    choices: Option<Element<'a, M>>,
    add_tip: String,
    separator: Box<dyn Fn(usize) -> String + 'a>,
    add_button: Option<Element<'a, M>>,
}

impl<'a, K: PartialEq + Clone + ToString + 'a, M: Clone + 'a> TagFilter<'a, K, M> {
    /// `options` is every tag on offer, `groups` what is picked (owned by the app) and `query`
    /// what the field holds.
    pub fn new(
        groups: &'a TagGroups<K>,
        options: impl IntoIterator<Item = ChipOption<K>>,
        query: &'a str,
        on_change: impl Fn(GroupEvent<K>) -> M + 'a,
        on_query: impl Fn(String) -> M + 'a,
    ) -> Self {
        Self::with(Cow::Borrowed(groups), options, query, on_change, on_query)
    }
    /// Like [`new`](Self::new) for groups built for the view out of the app's own state, when the
    /// app does not keep a `TagGroups`.
    pub fn owned(
        groups: TagGroups<K>,
        options: impl IntoIterator<Item = ChipOption<K>>,
        query: &'a str,
        on_change: impl Fn(GroupEvent<K>) -> M + 'a,
        on_query: impl Fn(String) -> M + 'a,
    ) -> Self {
        Self::with(Cow::Owned(groups), options, query, on_change, on_query)
    }
    fn with(
        groups: Cow<'a, TagGroups<K>>,
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
            description: None,
            placeholder: "Find a tag".into(),
            empty: "No matching tags".into(),
            max_lines: None,
            flat: false,
            metrics: Metrics::desktop(),
            picked: Vec::new(),
            field_id: None,
            before: Vec::new(),
            after: Vec::new(),
            leading: None,
            trailing: None,
            choices: None,
            add_tip: "Finish this group and start another".into(),
            separator: Box::new(|_| ",".into()),
            add_button: None,
        }
    }
    /// Several alternative groups (true) or a single one (false, the default). A single-group
    /// filter edits the first group only, and shows no group controls.
    pub fn multiple(mut self, on: bool) -> Self {
        self.multiple = on;
        self
    }
    /// The row's label (default "Tags").
    /// Text under the field, as a row's description: what picking several tags or groups means.
    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }
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
    /// How to draw picked tags that are not among `options` (a search narrowed them away): their
    /// label, icon and color. Without it such a pill shows the key.
    pub fn picked(mut self, options: impl IntoIterator<Item = ChipOption<K>>) -> Self {
        self.picked = options.into_iter().collect();
        self
    }
    /// The id of the find field, to focus it.
    pub fn field_id(mut self, id: impl Into<iced::widget::Id>) -> Self {
        self.field_id = Some(id.into());
        self
    }
    /// Read-only items in the entry before the groups (conditions the app shows but this filter
    /// does not edit).
    pub fn before(mut self, items: impl IntoIterator<Item = Element<'a, M>>) -> Self {
        self.before = items.into_iter().collect();
        self
    }
    /// Read-only items in the entry after the groups.
    pub fn after(mut self, items: impl IntoIterator<Item = Element<'a, M>>) -> Self {
        self.after = items.into_iter().collect();
        self
    }
    /// A control at the start of the entry (a back button).
    pub fn leading(mut self, control: impl Into<Element<'a, M>>) -> Self {
        self.leading = Some(control.into());
        self
    }
    /// A control at the end of the entry, before the add-group button.
    pub fn trailing(mut self, control: impl Into<Element<'a, M>>) -> Self {
        self.trailing = Some(control.into());
        self
    }
    /// Replaces the chips under the entry (the app lists the choices itself: categories, a
    /// search of its own). `options` and `max_lines` are then unused.
    pub fn choices(mut self, choices: impl Into<Element<'a, M>>) -> Self {
        self.choices = Some(choices.into());
        self
    }
    /// What separates group `index - 1` from group `index` (default a comma; empty for nothing).
    pub fn separator(mut self, separator: impl Fn(usize) -> String + 'a) -> Self {
        self.separator = Box::new(separator);
        self
    }
    /// Your own add-group button in place of the built-in one (it should send
    /// [`GroupEvent::AddGroup`]). Only shown with [`multiple`](Self::multiple).
    pub fn add_group_button(mut self, button: impl Into<Element<'a, M>>) -> Self {
        self.add_button = Some(button.into());
        self
    }
    /// The tooltip of the built-in add-group button.
    pub fn add_group_tooltip(mut self, text: impl Into<String>) -> Self {
        self.add_tip = text.into();
        self
    }

    /// The group being edited: always the first when single.
    fn active(&self) -> usize {
        if self.multiple {
            self.groups.active()
        } else {
            0
        }
    }
    fn pill(&self, group: usize, key: &K) -> Element<'a, M> {
        let option = self
            .options
            .iter()
            .chain(&self.picked)
            .find(|o| o.key == *key);
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
    fn pills(&mut self) -> (Vec<Element<'a, M>>, usize) {
        let active = self.active();
        let shown = if self.multiple {
            self.groups.groups().len()
        } else {
            1
        };
        let mut items: Vec<Element<'a, M>> = std::mem::take(&mut self.before);
        let mut caret = items.len();
        for (index, group) in self.groups.groups().iter().take(shown).enumerate() {
            let before_filled = index > 0 && !self.groups.groups()[index - 1].is_empty();
            if before_filled && (!group.is_empty() || index == active) {
                let separator = (self.separator)(index);
                if !separator.is_empty() {
                    items.push(ty::label(separator).into());
                }
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
        items.append(&mut self.after);
        if self.multiple {
            let last = self.groups.groups().len() - 1;
            if active == last {
                // Editing the last group (or the slot for a new one): the field ends the entry.
                caret = items.len();
            } else {
                // A bit of room after the pills that selects the last group when pressed.
                items.push(
                    mouse_area(space::horizontal().width(20).height(28))
                        .on_press((self.on_change)(GroupEvent::Select(last)))
                        .into(),
                );
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
        let mut f = f;
        let (mut items, caret) = f.pills();
        let on_query = f.on_query.clone();
        let mut field = text_input(if items.is_empty() { &f.placeholder } else { "" }, f.query)
            .on_input(move |q| on_query(q))
            .style(crate::styles::text_input::embedded)
            .padding([4, 2])
            .size(14)
            .width(Length::Fill);
        if let Some(id) = f.field_id.clone() {
            field = field.id(id);
        }
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
        let mut line = row![].spacing(4).align_y(Alignment::Center);
        if let Some(leading) = f.leading.take() {
            line = line.push(leading);
        }
        line = line.push(input);
        if let Some(trailing) = f.trailing.take() {
            line = line.push(trailing);
        }
        if f.multiple {
            line = line.push(match f.add_button.take() {
                Some(button) => button,
                None => tooltip(
                    section_button(Icon::ListAdd, (f.on_change)(GroupEvent::AddGroup)).metrics(m),
                    f.add_tip.clone(),
                    TooltipPosition::Bottom,
                ),
            });
        }
        let mut entry = EntryRow::new(f.label.clone(), EntryInput::Custom(line.into()))
            .flat(f.flat)
            .metrics(m);
        if let Some(d) = f.description.take() {
            entry = entry.description(d);
        }
        let chips: Element<'a, M> = match f.choices.take() {
            Some(choices) => choices,
            None => {
                let on_change = f.on_change.clone();
                let selected: Cow<'a, [K]> = match &f.groups {
                    Cow::Borrowed(groups) => {
                        let groups: &'a TagGroups<K> = groups;
                        Cow::Borrowed(groups.groups()[active].as_slice())
                    }
                    Cow::Owned(groups) => Cow::Owned(groups.groups()[active].clone()),
                };
                let mut chips =
                    ChipGroup::new(std::mem::take(&mut f.options), selected, move |key| {
                        on_change(GroupEvent::Toggle(active, key))
                    })
                    .query(f.query)
                    .empty_text(f.empty.clone())
                    .metrics(m);
                if let Some(lines) = f.max_lines {
                    chips = chips.max_lines(lines);
                }
                chips.into()
            }
        };
        column![Element::from(entry), chips]
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
                let _: Element<'_, GroupEvent<&str>> =
                    TagFilter::new(&groups, options(), "ro", |e| e, |_| GroupEvent::AddGroup)
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
        let mut single = TagFilter::new(&groups, options(), "", |e| e, |_| GroupEvent::AddGroup);
        let (pills, caret) = single.pills();
        assert_eq!((pills.len(), caret), (1, 1));
        let mut multiple =
            TagFilter::new(&groups, options(), "", |e| e, |_| GroupEvent::AddGroup).multiple(true);
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
        let mut f =
            TagFilter::new(&groups, options(), "", |e| e, |_| GroupEvent::AddGroup).multiple(true);
        assert_eq!(f.pills().1, 1);
    }
}
