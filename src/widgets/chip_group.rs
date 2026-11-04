//! `ChipGroup`: a wrapping set of toggle chips, optionally filtered by a find-as-you-type entry.
//! [`MultiSelect`](crate::widgets::MultiSelect) is a dropdown; this is the in-page version, for tags,
//! genres, moods and filters.
//!
//! The application owns the selection ([controlled](crate::widgets)) and the query. Each option
//! has a key, a label and optionally an icon and a color of its own; the chips are
//! [`Chip`](crate::components::Chip)s, so [`size`](ChipGroup::size) and [`on_color`](ChipGroup::on_color)
//! apply to all.
//!
//! ```ignore
//! ChipGroup::new(
//!     tags.iter().map(|t| ChipOption::new(t.id, &t.name).color(t.color)),
//!     &self.selected,
//!     Msg::Toggle,                                   // gets the key of the pressed chip
//! )
//! .search(&self.query, Msg::Query)                  // an entry above filters by label
//! ```

use crate::Element;
use crate::components::{Chip, ChipSize, entry, typography as ty};
use crate::styles::text as st;
use crate::{IconSource, Metrics};
use iced::Color;
use iced::widget::{column, row};

/// One chip of a [`ChipGroup`].
#[derive(Clone)]
pub struct ChipOption<K> {
    key: K,
    label: String,
    icon: Option<IconSource>,
    color: Option<Color>,
}

impl<K> ChipOption<K> {
    pub fn new(key: K, label: impl Into<String>) -> Self {
        Self {
            key,
            label: label.into(),
            icon: None,
            color: None,
        }
    }
    /// An icon of the chip's own (a built-in or your own SVG).
    pub fn icon(mut self, icon: impl Into<IconSource>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    /// A color of the chip's own, in place of the group's role.
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

/// Whether `label` matches the `query`: every word of the query is somewhere in it, ignoring
/// case. An empty query matches everything.
pub fn matches(label: &str, query: &str) -> bool {
    let label = label.to_lowercase();
    query
        .to_lowercase()
        .split_whitespace()
        .all(|word| label.contains(word))
}

/// See the [module](self).
pub struct ChipGroup<'a, K, M> {
    options: Vec<ChipOption<K>>,
    selected: &'a [K],
    on_toggle: Box<dyn Fn(K) -> M + 'a>,
    query: Option<&'a str>,
    on_query: Option<Box<dyn Fn(String) -> M + 'a>>,
    placeholder: String,
    empty: String,
    size: ChipSize,
    on_color: Option<Color>,
    min_contrast: f32,
    metrics: Metrics,
}

impl<'a, K: PartialEq + Clone + 'a, M: Clone + 'a> ChipGroup<'a, K, M> {
    /// `selected` are the keys that are on; `on_toggle` gets the key of a pressed chip.
    pub fn new(
        options: impl IntoIterator<Item = ChipOption<K>>,
        selected: &'a [K],
        on_toggle: impl Fn(K) -> M + 'a,
    ) -> Self {
        Self {
            options: options.into_iter().collect(),
            selected,
            on_toggle: Box::new(on_toggle),
            query: None,
            on_query: None,
            placeholder: "Search".into(),
            empty: "No matches".into(),
            size: ChipSize::Medium,
            on_color: None,
            min_contrast: 4.5,
            metrics: Metrics::desktop(),
        }
    }
    /// An entry above the chips that filters them by label as you type. `query` is what it holds
    /// (controlled); `on_query` gets each edit.
    pub fn search(mut self, query: &'a str, on_query: impl Fn(String) -> M + 'a) -> Self {
        self.query = Some(query);
        self.on_query = Some(Box::new(on_query));
        self
    }
    /// Filters the chips by `query` **without drawing an entry**: the application owns the field
    /// (shared by several groups, in a dialog's own layout). [`search`](Self::search) is this plus
    /// the entry.
    pub fn query(mut self, query: &'a str) -> Self {
        self.query = Some(query);
        self
    }
    /// The entry's placeholder (default "Search").
    pub fn placeholder(mut self, text: impl Into<String>) -> Self {
        self.placeholder = text.into();
        self
    }
    /// What shows when no chip matches (default "No matches").
    pub fn empty_text(mut self, text: impl Into<String>) -> Self {
        self.empty = text.into();
        self
    }
    pub fn size(mut self, size: ChipSize) -> Self {
        self.size = size;
        self
    }
    /// The text and icon color of selected chips that have a color of their own (see
    /// [`Chip::on_color`]).
    pub fn on_color(mut self, color: Color) -> Self {
        self.on_color = Some(color);
        self
    }
    /// The least contrast of unselected chips' own-colored text against the page (default 4.5;
    /// `0.0` keeps the colors as given). See [`Chip::min_contrast`].
    pub fn min_contrast(mut self, ratio: f32) -> Self {
        self.min_contrast = ratio;
        self
    }
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }

    /// The options the query leaves.
    fn visible(&self) -> Vec<&ChipOption<K>> {
        let q = self.query.unwrap_or("");
        self.options
            .iter()
            .filter(|o| matches(&o.label, q))
            .collect()
    }
}

impl<'a, K: PartialEq + Clone + 'a, M: Clone + 'a> From<ChipGroup<'a, K, M>> for Element<'a, M> {
    fn from(g: ChipGroup<'a, K, M>) -> Self {
        let m = g.metrics;
        let visible = g.visible();
        let body: Element<'a, M> = if visible.is_empty() {
            ty::caption(g.empty.clone()).style(st::dim).into()
        } else {
            let chips = visible.into_iter().map(|o| {
                let mut c = Chip::new(o.label.clone())
                    .size(g.size)
                    .selected(g.selected.contains(&o.key))
                    .on_press((g.on_toggle)(o.key.clone()));
                if let Some(i) = o.icon.clone() {
                    c = c.icon(i);
                }
                if let Some(col) = o.color {
                    c = c.color(col).min_contrast(g.min_contrast);
                    if let Some(on) = g.on_color {
                        c = c.on_color(on);
                    }
                }
                Element::from(c)
            });
            row(chips)
                .spacing(m.space_s)
                .width(iced::Length::Fill)
                .wrap()
                .vertical_spacing(m.space_s)
                .into()
        };
        // The entry is drawn only when `search` gave it somewhere to send what is typed.
        match (g.query, g.on_query) {
            (Some(q), Some(on_query)) => column![
                entry(g.placeholder, q.to_string())
                    .on_input(on_query)
                    .metrics(m),
                body
            ]
            .spacing(m.space_m)
            .into(),
            _ => body,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_word_of_the_query_must_match() {
        assert!(matches("Late Night Drive", "late drive"));
        assert!(matches("Late Night Drive", ""));
        assert!(matches("Late Night Drive", "NIGHT"));
        assert!(!matches("Late Night Drive", "late day"));
    }

    #[test]
    fn a_query_filters_without_an_entry() {
        let sel: [u8; 0] = [];
        let g: ChipGroup<'_, u8, ()> = ChipGroup::new(
            [ChipOption::new(1, "Jazz"), ChipOption::new(2, "Rock")],
            &sel,
            |_| (),
        )
        .query("rock");
        assert!(g.on_query.is_none());
        assert_eq!(g.visible().len(), 1);
    }

    #[test]
    fn the_query_filters_the_options() {
        let sel: [u8; 0] = [];
        let g: ChipGroup<'_, u8, ()> = ChipGroup::new(
            [
                ChipOption::new(1, "Jazz"),
                ChipOption::new(2, "Jazz fusion"),
                ChipOption::new(3, "Rock"),
            ],
            &sel,
            |_| (),
        )
        .search("jazz", |_| ());
        let labels: Vec<&str> = g.visible().iter().map(|o| o.label.as_str()).collect();
        assert_eq!(labels, vec!["Jazz", "Jazz fusion"]);
    }
}
