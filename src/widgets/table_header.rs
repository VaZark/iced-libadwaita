//! `TableHeader`: one header cell of a table or a flat list: a label over its column that
//! **wraps** when the column is narrow. [`sortable`](TableHeader::sortable) is an option of it, like
//! the options of an entry row: the same cell then also shows the sort arrow and is pressed to
//! sort. [`ResponsiveTable`](crate::widgets::ResponsiveTable) and
//! [`SortHeader`](crate::widgets::SortHeader) draw their headers with it.
//!
//! ```ignore
//! TableHeader::new("Name")                                   // a label that wraps
//! TableHeader::new("Name").sortable(state.direction(&Col::Name), Msg::SortName)
//! ```

use crate::components::{ButtonProps, TextProps, typography as ty};
use crate::primitives::sorting::Direction;
use crate::{Element, Icon, Metrics, Theme};
use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, container, row, text};
use iced::{Length, Padding};

/// The arrow of a column: up for ascending, down for descending, both when it can sort but is not.
pub fn arrow(direction: Option<Direction>) -> Icon {
    match direction {
        Some(Direction::Asc) => Icon::GoUp,
        Some(Direction::Desc) => Icon::GoDown,
        None => Icon::GoUpDown,
    }
}

/// The least width a header label needs: its longest word (the label wraps between words), and
/// room for the sort arrow. A column is never made narrower than this, so a word is never cut off.
pub(crate) fn label_floor(label: &str) -> f32 {
    // A caption heading is about 7 px a character; the arrow and its gap are 16 px more.
    let longest = label
        .split_whitespace()
        .map(|w| w.chars().count())
        .max()
        .unwrap_or(0);
    longest as f32 * 7.0 + 16.0
}

struct Sorting<M> {
    direction: Option<Direction>,
    priority: Option<usize>,
    press: M,
}

/// See the [module](self).
pub struct TableHeader<'a, M> {
    label: String,
    sorting: Option<Sorting<M>>,
    align: Horizontal,
    width: Length,
    padding: Padding,
    metrics: Metrics,
    label_props: TextProps,
    priority_props: TextProps,
    button_props: ButtonProps,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a, M: Clone + 'a> TableHeader<'a, M> {
    /// A header with `label`; it wraps onto more lines when the column is narrower than the text.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            sorting: None,
            align: Horizontal::Left,
            width: Length::Fill,
            padding: Padding::ZERO,
            metrics: Metrics::desktop(),
            label_props: TextProps::default(),
            priority_props: TextProps::default(),
            button_props: ButtonProps::default(),
            _marker: std::marker::PhantomData,
        }
    }
    /// Makes the header sortable: it shows the arrow for `direction` (`None`: not sorted by this
    /// column) and sends `on_press` when pressed. The application owns the sort state.
    #[must_use]
    pub fn sortable(mut self, direction: Option<Direction>, on_press: M) -> Self {
        self.sorting = Some(Sorting {
            direction,
            priority: None,
            press: on_press,
        });
        self
    }
    /// With several sort keys, which one this column is (1 first). Only for a sortable header.
    #[must_use]
    pub fn priority(mut self, priority: Option<usize>) -> Self {
        if let Some(s) = &mut self.sorting {
            s.priority = priority;
        }
        self
    }
    /// Aligns the label: `Right` over right-aligned cells (a time, a count).
    #[must_use]
    pub fn align(mut self, align: Horizontal) -> Self {
        self.align = align;
        self
    }
    /// The width of the header (default `Fill`, the column that holds it).
    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
    /// Padding inside the header (default none: the label sits on the column's edge, over the cells).
    #[must_use]
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }
    #[must_use]
    pub fn metrics(mut self, m: Metrics) -> Self {
        self.metrics = m;
        self
    }
    /// Presentation overrides applied after the child's defaults.
    #[must_use]
    pub fn label_props(mut self, props: TextProps) -> Self {
        self.label_props = props;
        self
    }
    /// Presentation overrides applied after the child's defaults.
    #[must_use]
    pub fn priority_props(mut self, props: TextProps) -> Self {
        self.priority_props = props;
        self
    }
    /// Presentation overrides applied after the child's defaults.
    #[must_use]
    pub fn button_props(mut self, props: ButtonProps) -> Self {
        self.button_props = props;
        self
    }
}

impl<'a, M: Clone + 'a> From<TableHeader<'a, M>> for Element<'a, M> {
    fn from(h: TableHeader<'a, M>) -> Self {
        let m = h.metrics;
        let wrap = text::Wrapping::WordOrGlyph;
        let Some(sort) = h.sorting else {
            return container(
                h.label_props.apply(
                    ty::caption_heading(h.label)
                        .style(crate::styles::text::dim)
                        .wrapping(wrap)
                        .width(Length::Fill)
                        .align_x(h.align),
                ),
            )
            .width(h.width)
            .padding(h.padding)
            .into();
        };
        let sorted = sort.direction.is_some();
        // A sorted arrow uses the foreground; an unsorted one is dim.
        let icon = arrow(sort.direction)
            .view(m.icon_size * 0.75)
            .style(move |th: &Theme, _| iced::widget::svg::Style {
                color: Some(if sorted {
                    th.semantic.icon_color(false)
                } else {
                    th.semantic.dim()
                }),
            });
        // The label takes the width the arrow leaves and wraps in it.
        let mut content = row![
            h.label_props.apply(
                ty::caption_heading(h.label)
                    .wrapping(wrap)
                    .width(Length::Fill)
                    .align_x(h.align)
            ),
            icon
        ]
        .spacing(m.space_xs)
        .align_y(Vertical::Center);
        if let Some(n) = sort.priority {
            content = content.push(h.priority_props.apply(ty::caption(n.to_string())));
        }
        // The whole column is the click target, not just the text.
        h.button_props
            .apply(
                button(content)
                    .width(h.width)
                    .padding(h.padding)
                    .style(move |th: &Theme, status| {
                        let hot = sorted
                            || matches!(status, button::Status::Hovered | button::Status::Pressed);
                        button::Style {
                            text_color: if hot {
                                th.semantic.window.fg
                            } else {
                                th.semantic.dim()
                            },
                            ..button::Style::default()
                        }
                    })
                    .on_press(sort.press),
            )
            .into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_arrow_follows_the_direction() {
        assert_eq!(arrow(Some(Direction::Asc)), Icon::GoUp);
        assert_eq!(arrow(Some(Direction::Desc)), Icon::GoDown);
        assert_eq!(arrow(None), Icon::GoUpDown);
    }

    #[test]
    fn a_label_never_needs_more_than_its_longest_word() {
        assert_eq!(label_floor("Last modified date"), label_floor("modified"));
        assert!(label_floor("Duration") > label_floor("Time"));
        assert_eq!(label_floor(""), label_floor(" "));
    }

    #[test]
    fn sortable_is_an_option_of_the_same_header() {
        let plain = TableHeader::<u8>::new("Name");
        assert!(plain.sorting.is_none());
        let sortable = TableHeader::new("Name")
            .sortable(Some(Direction::Asc), 7u8)
            .priority(Some(2));
        let s = sortable.sorting.as_ref().unwrap();
        assert_eq!(
            (s.direction, s.priority, s.press),
            (Some(Direction::Asc), Some(2), 7)
        );
        // A priority means nothing on a header that does not sort.
        assert!(
            TableHeader::<u8>::new("Name")
                .priority(Some(1))
                .sorting
                .is_none()
        );
        let _: Element<'_, u8> = sortable.into();
        let _: Element<'_, u8> = plain.into();
    }
}
