//! `TagGroupList`: tag groups, read only: the pills of a [`TagFilter`](super::TagFilter) with
//! several groups, without the field around them. Each group is the tags that go together (an item
//! needs all of them); the groups are alternatives (any one group will do). The pills of a group
//! sit close together in the same token flow the filter uses, and groups are set apart by a gap.
//! No border and no commas: it reads as the input does, but you cannot type in it. A
//! [`separator`](TagGroupList::separator) can be asked for.
//!
//! It shows groups, it does not edit them: for choosing, see [`TagFilter`](super::TagFilter).
//! Pressing a chip's ✕ is reported (`on_remove`) so a strip of active filters can drop a tag.
//!
//! ```ignore
//! TagGroupList::new(vec![
//!     vec![TagItem::new("work").color(red)],
//!     vec![TagItem::new("home"), TagItem::new("urgent")],
//! ])
//! .on_remove(|group, tag| Msg::Remove(group, tag))
//! ```

use crate::components::{Chip, ChipSize};
use crate::layouts::token_flow;
use crate::{Element, Metrics};
use iced::widget::{row, space, text};
use iced::{Alignment, Color};

/// One tag to show: its label and, when it has one, its colour.
#[derive(Debug, Clone, PartialEq)]
pub struct TagItem {
    pub label: String,
    pub color: Option<Color>,
}

impl TagItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            color: None,
        }
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

type OnRemove<'a, M> = Box<dyn Fn(usize, usize) -> M + 'a>;

pub struct TagGroupList<'a, M> {
    chip_props: crate::components::badge::ChipProps,
    label_props: crate::components::TextProps,
    groups: Vec<Vec<TagItem>>,
    separator: String,
    size: ChipSize,
    on_remove: Option<OnRemove<'a, M>>,
    spacing: Option<f32>,
    metrics: Metrics,
}

impl<'a, M: Clone + 'a> TagGroupList<'a, M> {
    pub fn chip_props(mut self, props: crate::components::badge::ChipProps) -> Self {
        self.chip_props = props;
        self
    }

    pub fn label_props(mut self, props: crate::components::TextProps) -> Self {
        self.label_props = props;
        self
    }

    pub fn new(groups: Vec<Vec<TagItem>>) -> Self {
        Self {
            chip_props: Default::default(),
            label_props: Default::default(),
            groups,
            separator: String::new(),
            size: ChipSize::Small,
            on_remove: None,
            spacing: None,
            metrics: Metrics::desktop(),
        }
    }

    /// What goes between groups. Nothing by default.
    pub fn separator(mut self, s: impl Into<String>) -> Self {
        self.separator = s.into();
        self
    }

    /// The gap between the chips of a group (default `space_xs`).
    pub fn spacing(mut self, px: f32) -> Self {
        self.spacing = Some(px);
        self
    }

    pub fn size(mut self, size: ChipSize) -> Self {
        self.size = size;
        self
    }

    /// Chips get a ✕; pressing one sends the index of its group and of the tag in it.
    pub fn on_remove(mut self, f: impl Fn(usize, usize) -> M + 'a) -> Self {
        self.on_remove = Some(Box::new(f));
        self
    }

    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
}

impl<'a, M: Clone + 'a> From<TagGroupList<'a, M>> for Element<'a, M> {
    fn from(list: TagGroupList<'a, M>) -> Self {
        // The gap between groups: wider than the pills' own, so groups read apart without a mark.
        const GAP: f32 = 10.0;
        let mut flow: Vec<Element<'a, M>> = Vec::new();
        let mut first = true;
        for (g, tags) in list
            .groups
            .iter()
            .enumerate()
            .filter(|(_, t)| !t.is_empty())
        {
            if !first {
                if list.separator.is_empty() {
                    flow.push(space().width(GAP).into());
                } else {
                    flow.push(
                        list.label_props
                            .apply(text(list.separator.clone()).style(crate::styles::text::dim)),
                    );
                }
            }
            first = false;
            let chips = tags.iter().enumerate().map(|(t, tag)| {
                let mut chip = Chip::new(tag.label.clone()).size(list.size);
                if let Some(c) = tag.color {
                    chip = chip.color(c);
                }
                if let Some(f) = &list.on_remove {
                    chip = chip.on_remove(f(g, t));
                }
                Element::from(list.chip_props.apply(chip))
            });
            flow.push(
                row(chips)
                    .spacing(list.spacing.unwrap_or(list.metrics.space_xs))
                    .align_y(Alignment::Center)
                    .wrap()
                    .into(),
            );
        }
        token_flow(flow)
    }
}

#[cfg(test)]
mod spacing_params {
    use super::*;

    #[test]
    fn the_chip_gap_is_a_parameter() {
        let l =
            TagGroupList::<()>::new(vec![vec![TagItem::new("a"), TagItem::new("b")]]).spacing(7.0);
        assert_eq!(l.spacing, Some(7.0));
        let _: Element<'_, ()> = l.into();
    }
}

/// Presentation-only overrides supported by [`TagGroupList`]. Values and events stay parent-owned.
#[derive(Clone, Default)]
pub struct TagGroupListProps {
    pub spacing: Option<f32>,
    pub metrics: Option<crate::Metrics>,
    pub label_props: Option<crate::components::TextProps>,
    pub chip_props: Option<crate::components::badge::ChipProps>,
}
impl TagGroupListProps {
    pub fn spacing(mut self, value: impl Into<f32>) -> Self {
        self.spacing = Some(value.into());
        self
    }
    pub fn metrics(mut self, value: impl Into<crate::Metrics>) -> Self {
        self.metrics = Some(value.into());
        self
    }
    pub fn label_props(mut self, value: impl Into<crate::components::TextProps>) -> Self {
        self.label_props = Some(value.into());
        self
    }
    pub fn chip_props(mut self, value: impl Into<crate::components::badge::ChipProps>) -> Self {
        self.chip_props = Some(value.into());
        self
    }
    pub fn apply<'a, M: Clone + 'a>(&self, widget: TagGroupList<'a, M>) -> TagGroupList<'a, M> {
        let mut widget = widget;
        if let Some(value) = &self.spacing {
            widget = widget.spacing(value.clone());
        }
        if let Some(value) = &self.metrics {
            widget = widget.metrics(value.clone());
        }
        if let Some(props) = &self.label_props {
            widget.label_props.merge(props);
        }
        if let Some(props) = &self.chip_props {
            widget.chip_props.merge(props);
        }
        widget
    }
}
