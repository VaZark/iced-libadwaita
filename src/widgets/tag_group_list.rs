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
    groups: Vec<Vec<TagItem>>,
    separator: String,
    size: ChipSize,
    on_remove: Option<OnRemove<'a, M>>,
    #[allow(dead_code)]
    metrics: Metrics,
}

impl<'a, M: Clone + 'a> TagGroupList<'a, M> {
    pub fn new(groups: Vec<Vec<TagItem>>) -> Self {
        Self {
            groups,
            separator: String::new(),
            size: ChipSize::Small,
            on_remove: None,
            metrics: Metrics::desktop(),
        }
    }

    /// What goes between groups. Nothing by default.
    pub fn separator(mut self, s: impl Into<String>) -> Self {
        self.separator = s.into();
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
                        text(list.separator.clone())
                            .style(crate::styles::text::dim)
                            .into(),
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
                Element::from(chip)
            });
            flow.push(
                row(chips)
                    .spacing(4)
                    .align_y(Alignment::Center)
                    .wrap()
                    .into(),
            );
        }
        token_flow(flow)
    }
}
